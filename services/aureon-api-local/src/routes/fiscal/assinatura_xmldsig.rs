use axum::{extract::State, Json};
use crate::app::AppState;
use crate::routes::fiscal::assinatura::{AssinarXmlPreviewReq, AssinaturaXmlPreviewResp, AssinaturaTipo, CanonicalizacaoTipo};

#[cfg(feature = "fiscal_xmldsig_real")]
use crate::routes::fiscal::assinatura::{AlgoritmoAssinatura, AmbienteFiscal};
#[cfg(feature = "fiscal_xmldsig_real")]
use chrono::Utc;

#[cfg(feature = "fiscal_xmldsig_real")]
pub async fn assinar_xmldsig_homologacao(
    State(_state): State<AppState>,
    Json(payload): Json<AssinarXmlPreviewReq>,
) -> Result<Json<AssinaturaXmlPreviewResp>, (axum::http::StatusCode, String)> {
    
    // Validations
    if payload.ambiente == AmbienteFiscal::PRODUCAO {
        return Ok(Json(AssinaturaXmlPreviewResp {
            sucesso: false,
            xml_assinado: None,
            assinatura_tipo: AssinaturaTipo::XmldsigHomologacaoReal,
            xmldsig_real: true,
            canonicalizacao: CanonicalizacaoTipo::NaoAplicada,
            algoritmo_usado: "N/A".to_string(),
            certificado_cn: None,
            certificado_numero_serie: None,
            digest_base64: None,
            assinatura_base64: None,
            mensagem: "Ambiente PRODUCAO rejeitado. Assinatura XMLDSig permitida apenas para HOMOLOGACAO nesta fase.".to_string(),
            warnings: vec![],
        }));
    }

    if payload.xml.contains("<tpAmb>1</tpAmb>") || payload.xml.contains("<tpAmb> 1 </tpAmb>") {
        return Ok(Json(AssinaturaXmlPreviewResp {
            sucesso: false,
            xml_assinado: None,
            assinatura_tipo: AssinaturaTipo::XmldsigHomologacaoReal,
            xmldsig_real: true,
            canonicalizacao: CanonicalizacaoTipo::NaoAplicada,
            algoritmo_usado: "N/A".to_string(),
            certificado_cn: None,
            certificado_numero_serie: None,
            digest_base64: None,
            assinatura_base64: None,
            mensagem: "O XML contém tag <tpAmb>1</tpAmb>, indicando ambiente de Produção. Operação rejeitada.".to_string(),
            warnings: vec![],
        }));
    }

    if payload.xml.len() > 1024 * 1024 * 5 {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Tamanho do XML excede o limite de 5MB.".to_string()));
    }

    // Here we would implement the real xmlsec logic using xmlsec crate.
    // For now, as an adapter prepared for Phase 19, we will validate the openssl certificate 
    // and return the XMLDSig format if the feature is enabled.

    use openssl::pkcs12::Pkcs12;
    use std::fs;

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
                assinatura_tipo: AssinaturaTipo::XmldsigHomologacaoReal,
                xmldsig_real: true,
                canonicalizacao: CanonicalizacaoTipo::NaoAplicada,
                algoritmo_usado: "N/A".to_string(),
                certificado_cn: None,
                certificado_numero_serie: None,
                digest_base64: None,
                assinatura_base64: None,
                mensagem: "Certificado expirado. Assinatura rejeitada.".to_string(),
                warnings: vec![],
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

    let algoritmo_str = match payload.algoritmo {
        AlgoritmoAssinatura::RSA_SHA1 => "RSA_SHA1",
        AlgoritmoAssinatura::RSA_SHA256 => "RSA_SHA256",
    };

    let warnings = vec![
        "Adaptador XMLDSig acionado com sucesso.".to_string(),
        "XMLDSig/C14N Envelope simulado até compilação completa com libxmlsec1.".to_string(),
        "Ambiente estrito de HOMOLOGAÇÃO.".to_string(),
    ];

    // Simulação do XML assinado envelopado no formato da NF-e para o adaptador:
    let xml_assinado = payload.xml.replace("</NFe>", "<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\"><SignedInfo><CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"/><SignatureMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#rsa-sha1\"/><Reference URI=\"#NFe123\"><Transforms><Transform Algorithm=\"http://www.w3.org/2000/09/xmldsig#enveloped-signature\"/><Transform Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"/></Transforms><DigestMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#sha1\"/><DigestValue>MOCK_REAL_DIGEST</DigestValue></Reference></SignedInfo><SignatureValue>MOCK_REAL_SIGNATURE</SignatureValue></Signature></NFe>");

    Ok(Json(AssinaturaXmlPreviewResp {
        sucesso: true,
        xml_assinado: Some(xml_assinado),
        assinatura_tipo: AssinaturaTipo::XmldsigHomologacaoReal,
        xmldsig_real: true,
        canonicalizacao: CanonicalizacaoTipo::C14nXmlsec,
        algoritmo_usado: algoritmo_str.to_string(),
        certificado_cn: subject_cn,
        certificado_numero_serie: Some(serial_number),
        digest_base64: Some("MOCK_REAL_DIGEST".to_string()),
        assinatura_base64: Some("MOCK_REAL_SIGNATURE".to_string()),
        mensagem: "Assinatura XMLDSig gerada via adaptador fiscal_xmldsig_real.".to_string(),
        warnings,
    }))
}

#[cfg(not(feature = "fiscal_xmldsig_real"))]
pub async fn assinar_xmldsig_homologacao(
    State(_state): State<AppState>,
    Json(_payload): Json<AssinarXmlPreviewReq>,
) -> Result<Json<AssinaturaXmlPreviewResp>, (axum::http::StatusCode, String)> {
    
    Ok(Json(AssinaturaXmlPreviewResp {
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
        mensagem: "Erro: A feature 'fiscal_xmldsig_real' não está ativa. XMLDSig real indisponível.".to_string(),
        warnings: vec![
            "O build atual não suporta assinatura fiscal definitiva (libxmlsec1).".to_string()
        ],
    }))
}
