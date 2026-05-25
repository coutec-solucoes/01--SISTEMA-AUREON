use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::fs;
use crate::app::AppState;

#[derive(Debug, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AlgoritmoAssinatura {
    RSA_SHA1,
    RSA_SHA256,
}

#[derive(Debug, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AmbienteFiscal {
    HOMOLOGACAO,
    PRODUCAO,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssinaturaTipo {
    PreviewTecnica,
    XmldsigHomologacaoReal,
    MockIndisponivel,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CanonicalizacaoTipo {
    C14nExclusive,
    C14nXmlsec,
    NaoAplicada,
}


#[derive(Debug, Deserialize)]
pub struct AssinarXmlPreviewReq {
    pub xml: String,
    pub caminho_pfx: Option<String>,
    pub conteudo_base64: Option<String>,
    pub senha_pfx: String,
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub algoritmo: AlgoritmoAssinatura,
    pub ambiente: AmbienteFiscal,
}

#[derive(Debug, Serialize)]
pub struct AssinaturaXmlPreviewResp {
    pub sucesso: bool,
    pub xml_assinado: Option<String>,
    pub assinatura_tipo: AssinaturaTipo,
    pub xmldsig_real: bool,
    pub canonicalizacao: CanonicalizacaoTipo,
    pub algoritmo_usado: String,
    pub certificado_cn: Option<String>,
    pub certificado_numero_serie: Option<String>,
    pub digest_base64: Option<String>,
    pub assinatura_base64: Option<String>,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct VerificarXmlPreviewReq {
    pub xml_assinado: String,
}

#[derive(Debug, Serialize)]
pub struct VerificarXmlPreviewResp {
    pub valido: bool,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

const MAX_XML_SIZE: usize = 1024 * 1024 * 5; // 5 MB

fn validar_regras_basicas_xml(xml: &str, ambiente: &AmbienteFiscal) -> Result<(), String> {
    if xml.len() > MAX_XML_SIZE {
        return Err("Tamanho do XML excede o limite de 5MB.".to_string());
    }

    if *ambiente == AmbienteFiscal::PRODUCAO {
        return Err("Ambiente PRODUCAO rejeitado. Assinatura permitida apenas para HOMOLOGACAO.".to_string());
    }

    // Regra rígida: bloqueia tpAmb=1 (Produção)
    if xml.contains("<tpAmb>1</tpAmb>") || xml.contains("<tpAmb> 1 </tpAmb>") {
        return Err("O XML contém tag <tpAmb>1</tpAmb>, indicando ambiente de Produção. Operação rejeitada.".to_string());
    }

    Ok(())
}

#[cfg(feature = "fiscal_real")]
pub async fn assinar_preview(
    State(_state): State<AppState>,
    Json(payload): Json<AssinarXmlPreviewReq>,
) -> Result<Json<AssinaturaXmlPreviewResp>, (axum::http::StatusCode, String)> {
    
    use openssl::pkcs12::Pkcs12;
    use openssl::sign::Signer;
    use openssl::hash::MessageDigest;
    use chrono::Utc;

    let mut warnings = vec![
        "Assinatura técnica apenas para preview. Canonicalização (XMLDSig/C14N) real não aplicada nesta etapa.".to_string(),
        "Nenhum documento fiscal gerado ou autorizado.".to_string(),
    ];

    if let Err(msg) = validar_regras_basicas_xml(&payload.xml, &payload.ambiente) {
        return Ok(Json(AssinaturaXmlPreviewResp {
            sucesso: false,
            xml_assinado: None,
            assinatura_tipo: AssinaturaTipo::PreviewTecnica,
            xmldsig_real: false,
            canonicalizacao: CanonicalizacaoTipo::NaoAplicada,
            algoritmo_usado: "N/A".to_string(),
            certificado_cn: None,
            certificado_numero_serie: None,
            digest_base64: None,
            assinatura_base64: None,
            mensagem: msg,
            warnings,
        }));
    }

    let pfx_bytes = if let Some(ref base64_str) = payload.conteudo_base64 {
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.decode(base64_str).map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("Erro ao decodificar base64 do PFX: {}", e)))?
    } else if let Some(ref path_str) = payload.caminho_pfx {
        fs::read(path_str).map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("Erro ao ler arquivo PFX: {}", e)))?
    } else {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Caminho PFX ou conteúdo base64 devem ser informados".to_string()));
    };

    let pkcs12 = Pkcs12::from_der(&pfx_bytes).map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("PFX inválido: {}", e)))?;
    let parsed = pkcs12.parse(&payload.senha_pfx).map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("Senha incorreta ou PFX inválido: {}", e)))?;
    
    let cert = parsed.cert;
    let pkey = parsed.pkey;

    // Verificar validade
    let parse_asn1_time = |time_str: &str| -> Option<chrono::DateTime<Utc>> {
        chrono::NaiveDateTime::parse_from_str(time_str, "%b %e %H:%M:%S %Y %Z")
            .or_else(|_| chrono::NaiveDateTime::parse_from_str(time_str, "%b %d %H:%M:%S %Y %Z"))
            .ok()
            .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    };

    if let Some(fim) = parse_asn1_time(&cert.not_after().to_string()) {
        if Utc::now() > fim {
            return Ok(Json(AssinaturaXmlPreviewResp {
                sucesso: false,
                xml_assinado: None,
                assinatura_tipo: AssinaturaTipo::PreviewTecnica,
                xmldsig_real: false,
                canonicalizacao: CanonicalizacaoTipo::NaoAplicada,
                algoritmo_usado: "N/A".to_string(),
                certificado_cn: None,
                certificado_numero_serie: None,
                digest_base64: None,
                assinatura_base64: None,
                mensagem: "Certificado expirado. Assinatura rejeitada.".to_string(),
                warnings,
            }));
        }
    }

    let mut subject_cn = None;
    for entry in cert.subject_name().entries() {
        if entry.object().nid() == openssl::nid::Nid::COMMONNAME {
            if let Ok(s) = entry.data().as_utf8() {
                subject_cn = Some(s.to_string());
                break;
            }
        }
    }

    let serial_number = cert.serial_number().to_bn().map(|bn| bn.to_hex_str().unwrap().to_string().to_ascii_uppercase()).unwrap_or_default();

    // Assinatura Técnica
    let digest_alg = match payload.algoritmo {
        AlgoritmoAssinatura::RSA_SHA1 => {
            warnings.push("Atenção: RSA_SHA1 é mantido por compatibilidade legada, prefira RSA_SHA256.".to_string());
            MessageDigest::sha1()
        },
        AlgoritmoAssinatura::RSA_SHA256 => MessageDigest::sha256(),
    };

    let mut signer = Signer::new(digest_alg, &pkey).map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Erro ao preparar signer: {}", e)))?;
    signer.update(payload.xml.as_bytes()).map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Erro ao calcular digest: {}", e)))?;
    
    let signature = signer.sign_to_vec().map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Erro ao assinar: {}", e)))?;
    
    use base64::{Engine as _, engine::general_purpose};
    let assinatura_base64 = general_purpose::STANDARD.encode(&signature);
    
    // Hash (Digest) do XML 
    let mut hasher = openssl::hash::Hasher::new(digest_alg).unwrap();
    hasher.update(payload.xml.as_bytes()).unwrap();
    let digest_bytes = hasher.finish().unwrap();
    let digest_base64 = general_purpose::STANDARD.encode(&*digest_bytes);

    let algoritmo_str = match payload.algoritmo {
        AlgoritmoAssinatura::RSA_SHA1 => "RSA_SHA1",
        AlgoritmoAssinatura::RSA_SHA256 => "RSA_SHA256",
    };

    // Simulando injeção do bloco de assinatura no XML (como é um preview técnico, faremos algo rústico apenas para visualizar)
    let xml_assinado = payload.xml.replace("</NFe>", &format!("<Signature><SignedInfo><DigestValue>{}</DigestValue></SignedInfo><SignatureValue>{}</SignatureValue></Signature></NFe>", digest_base64, assinatura_base64));
    let xml_assinado = if xml_assinado == payload.xml {
        // Se nao for NFe, adiciona no final
        format!("{}\n<!-- SIGNATURE PREVIEW -->\n<Signature><SignatureValue>{}</SignatureValue></Signature>", payload.xml, assinatura_base64)
    } else {
        xml_assinado
    };

    Ok(Json(AssinaturaXmlPreviewResp {
        sucesso: true,
        xml_assinado: Some(xml_assinado),
        assinatura_tipo: AssinaturaTipo::PreviewTecnica,
        xmldsig_real: false,
        canonicalizacao: CanonicalizacaoTipo::NaoAplicada,
        algoritmo_usado: algoritmo_str.to_string(),
        certificado_cn: subject_cn,
        certificado_numero_serie: Some(serial_number),
        digest_base64: Some(digest_base64),
        assinatura_base64: Some(assinatura_base64),
        mensagem: "Assinatura técnica realizada com sucesso usando OpenSSL (Feature: fiscal_real).".to_string(),
        warnings,
    }))
}

#[cfg(not(feature = "fiscal_real"))]
pub async fn assinar_preview(
    State(_state): State<AppState>,
    Json(payload): Json<AssinarXmlPreviewReq>,
) -> Result<Json<AssinaturaXmlPreviewResp>, (axum::http::StatusCode, String)> {
    
    let mut warnings = vec![
        "Assinatura técnica MOCKADA apenas para preview. Canonicalização (XMLDSig/C14N) real não aplicada nesta etapa.".to_string(),
        "Nenhum documento fiscal gerado ou autorizado.".to_string(),
    ];

    if let Err(msg) = validar_regras_basicas_xml(&payload.xml, &payload.ambiente) {
        return Ok(Json(AssinaturaXmlPreviewResp {
            sucesso: false,
            xml_assinado: None,
            assinatura_tipo: AssinaturaTipo::MockIndisponivel,
            xmldsig_real: false,
            canonicalizacao: CanonicalizacaoTipo::NaoAplicada,
            algoritmo_usado: "N/A".to_string(),
            certificado_cn: None,
            certificado_numero_serie: None,
            digest_base64: None,
            assinatura_base64: None,
            mensagem: msg,
            warnings,
        }));
    }

    let algoritmo_str = match payload.algoritmo {
        AlgoritmoAssinatura::RSA_SHA1 => {
            warnings.push("Atenção: RSA_SHA1 é mantido por compatibilidade legada, prefira RSA_SHA256.".to_string());
            "RSA_SHA1"
        },
        AlgoritmoAssinatura::RSA_SHA256 => "RSA_SHA256",
    };

    let assinatura_base64 = "MOCK_SIGNATURE_BASE64_==".to_string();
    let digest_base64 = "MOCK_DIGEST_BASE64_==".to_string();

    let xml_assinado = payload.xml.replace("</NFe>", &format!("<Signature><SignedInfo><DigestValue>{}</DigestValue></SignedInfo><SignatureValue>{}</SignatureValue></Signature></NFe>", digest_base64, assinatura_base64));
    let xml_assinado = if xml_assinado == payload.xml {
        format!("{}\n<!-- SIGNATURE PREVIEW -->\n<Signature><SignatureValue>{}</SignatureValue></Signature>", payload.xml, assinatura_base64)
    } else {
        xml_assinado
    };

    Ok(Json(AssinaturaXmlPreviewResp {
        sucesso: true,
        xml_assinado: Some(xml_assinado),
        assinatura_tipo: AssinaturaTipo::PreviewTecnica,
        xmldsig_real: false,
        canonicalizacao: CanonicalizacaoTipo::NaoAplicada,
        algoritmo_usado: algoritmo_str.to_string(),
        certificado_cn: Some("MOCK EMPRESA LTDA".to_string()),
        certificado_numero_serie: Some("A1B2C3D4E5F6".to_string()),
        digest_base64: Some(digest_base64),
        assinatura_base64: Some(assinatura_base64),
        mensagem: "[DEV/MOCK] Assinatura gerada via mock. Ative a feature 'fiscal_real' para usar o OpenSSL.".to_string(),
        warnings,
    }))
}

pub async fn testar_assinatura(
    State(state): State<AppState>,
    Json(payload): Json<AssinarXmlPreviewReq>,
) -> Result<Json<AssinaturaXmlPreviewResp>, (axum::http::StatusCode, String)> {
    // Endpoint para testes isolados
    assinar_preview(State(state), Json(payload)).await
}

pub async fn verificar_preview(
    State(_state): State<AppState>,
    Json(payload): Json<VerificarXmlPreviewReq>,
) -> Result<Json<VerificarXmlPreviewResp>, (axum::http::StatusCode, String)> {
    let warnings = vec![
        "Verificação técnica simplificada. Validação criptográfica de Signature não implementada sem XMLDSig real.".to_string()
    ];

    if payload.xml_assinado.contains("<SignatureValue>") && payload.xml_assinado.contains("</SignatureValue>") {
        Ok(Json(VerificarXmlPreviewResp {
            valido: true,
            mensagem: "XML contém tag de assinatura. Assumindo como válido para fins de preview técnico.".to_string(),
            warnings,
        }))
    } else {
        Ok(Json(VerificarXmlPreviewResp {
            valido: false,
            mensagem: "XML não contém tag <SignatureValue>.".to_string(),
            warnings,
        }))
    }
}
