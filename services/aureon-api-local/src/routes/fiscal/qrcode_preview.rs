use axum::{extract::{State, Path}, Json};
use serde::{Deserialize, Serialize};
use qrcode::QrCode;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use crate::app::AppState;
use crate::routes::fiscal::assinatura::AmbienteFiscal;
use crate::routes::fiscal::historico_homologacao::{registrar_evento_homologacao, RegistrarEventoHomologacaoParams};

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum TipoQrCodePreview {
    NFCE,
    NFE,
    SIFEN,
}

#[derive(Debug, Deserialize)]
pub struct GerarQrCodePreviewReq {
    pub tipo: TipoQrCodePreview,
    pub chave_preview: Option<String>,
    pub cdc_preview: Option<String>,
    pub uf: Option<String>,
    pub ambiente: AmbienteFiscal,
    pub url_base_preview: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QrCodePreviewResp {
    pub sucesso: bool,
    pub tipo: String,
    pub ambiente: String,
    pub conteudo_qr: Option<String>,
    pub png_base64: Option<String>,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

pub async fn gerar_qrcode(
    State(state): State<AppState>,
    Json(payload): Json<GerarQrCodePreviewReq>,
) -> Result<Json<QrCodePreviewResp>, (axum::http::StatusCode, String)> {
    
    let mut warnings = vec![
        "DOCUMENTO TÉCNICO DE HOMOLOGAÇÃO SEM VALIDADE FISCAL".to_string(),
        "Nenhuma consulta oficial ou transmissão será realizada.".to_string(),
    ];

    if payload.ambiente == AmbienteFiscal::PRODUCAO {
        // Auditoria PRODUCAO_BLOQUEADA
        if let Some(pool) = &state.pool {
            let pool = pool.clone();
            tokio::spawn(async move {
                let _ = registrar_evento_homologacao(&pool, RegistrarEventoHomologacaoParams {
                    tipo_evento: "PRODUCAO_BLOQUEADA",
                    pais: None,
                    modelo: None,
                    venda_id: None,
                    chave_preview: None,
                    cdc_preview: None,
                    sucesso: false,
                    mensagem: "Tentativa de gerar QR Code em PRODUCAO bloqueada.".into(),
                    payload_hash: None,
                    erro_codigo: Some("AMBIENTE_PRODUCAO_REJEITADO".into()),
                    payload_preview: None,
                }).await;
            });
        }
        return Ok(Json(QrCodePreviewResp {
            sucesso: false,
            tipo: format!("{:?}", payload.tipo),
            ambiente: format!("{:?}", payload.ambiente),
            conteudo_qr: None,
            png_base64: None,
            mensagem: "Ambiente PRODUCAO bloqueado. Esta API gera QR Code apenas para HOMOLOGACAO.".to_string(),
            warnings,
        }));
    }

    let conteudo_qr_string = match payload.tipo {
        TipoQrCodePreview::NFCE | TipoQrCodePreview::NFE => {
            let chave = payload.chave_preview.clone().unwrap_or_else(|| "00000000000000000000000000000000000000000000".to_string());
            if chave.is_empty() {
                return Ok(Json(build_erro_resp(payload, "Chave de acesso não informada.".to_string())));
            }
            let url = payload.url_base_preview.clone().unwrap_or_else(|| "https://homologacao.sefaz.gov.br/preview/qrcode".to_string());
            // Formato mockado de QR BR para preview: url?p=CHAVE|2|1|1|HASH_TECNICO
            format!("{}?p={}|2|1|1|PREVIEW_SEM_VALIDADE_FISCAL_HOMOLOGACAO", url, chave)
        },
        TipoQrCodePreview::SIFEN => {
            let cdc = payload.cdc_preview.clone().unwrap_or_else(|| "00000000000000000000000000000000000000000000".to_string());
            if cdc.is_empty() {
                return Ok(Json(build_erro_resp(payload, "CDC não informado.".to_string())));
            }
            let url = payload.url_base_preview.clone().unwrap_or_else(|| "https://ekuatia.set.gov.py/consultas-test/qr".to_string());
            // Formato mockado de QR PY para preview
            format!("{}?nIdFisc={}&PREVIEW_SEM_VALIDADE_FISCAL_HOMOLOGACAO", url, cdc)
        }
    };

    // Gera a matriz do QRCode
    let code = match QrCode::new(conteudo_qr_string.as_bytes()) {
        Ok(c) => c,
        Err(e) => {
            return Ok(Json(build_erro_resp(payload, format!("Erro ao processar conteúdo do QR Code: {}", e))));
        }
    };

    // Renderiza a imagem em SVG para evitar problemas com crate image nativo
    let svg = code.render::<qrcode::render::svg::Color>().build();
    
    // Converte o SVG para base64 para o front-end exibir (data:image/svg+xml;base64,...)
    let b64 = BASE64_STANDARD.encode(svg.as_bytes());

    warnings.push("QR Code gerado internamente sem cHashQR oficial e sem consulta na SEFAZ/DNIT.".to_string());

    // Auditoria técnica — não bloqueia resposta
    let pais_qr = match payload.tipo {
        TipoQrCodePreview::SIFEN => "PY",
        _ => "BR",
    };
    if let Some(pool) = &state.pool {
        let pool = pool.clone();
        let conteudo_clone = conteudo_qr_string.clone();
        let pais_str = pais_qr.to_string();
        tokio::spawn(async move {
            let _ = registrar_evento_homologacao(&pool, RegistrarEventoHomologacaoParams {
                tipo_evento: "QRCODE_PREVIEW_GERADO",
                pais: Some(pais_str),
                modelo: None,
                venda_id: None,
                chave_preview: None,
                cdc_preview: None,
                sucesso: true,
                mensagem: "QR Code preview gerado. Não representa QR fiscal oficial.".into(),
                payload_hash: None,
                erro_codigo: None,
                payload_preview: Some(serde_json::json!({"tamanho_conteudo": conteudo_clone.len()})),
            }).await;
        });
    }

    Ok(Json(QrCodePreviewResp {
        sucesso: true,
        tipo: format!("{:?}", payload.tipo),
        ambiente: format!("{:?}", payload.ambiente),
        conteudo_qr: Some(conteudo_qr_string),
        png_base64: Some(b64),
        mensagem: "QR Code preview gerado com sucesso (formato base64 SVG).".to_string(),
        warnings,
    }))
}

pub async fn qrcode_nfce(
    State(state): State<AppState>,
    Path(chave_preview): Path<String>,
) -> Result<Json<QrCodePreviewResp>, (axum::http::StatusCode, String)> {
    let req = GerarQrCodePreviewReq {
        tipo: TipoQrCodePreview::NFCE,
        chave_preview: Some(chave_preview),
        cdc_preview: None,
        uf: None,
        ambiente: AmbienteFiscal::HOMOLOGACAO,
        url_base_preview: None,
    };
    gerar_qrcode(State(state), Json(req)).await
}

pub async fn qrcode_sifen(
    State(state): State<AppState>,
    Path(cdc_preview): Path<String>,
) -> Result<Json<QrCodePreviewResp>, (axum::http::StatusCode, String)> {
    let req = GerarQrCodePreviewReq {
        tipo: TipoQrCodePreview::SIFEN,
        chave_preview: None,
        cdc_preview: Some(cdc_preview),
        uf: None,
        ambiente: AmbienteFiscal::HOMOLOGACAO,
        url_base_preview: None,
    };
    gerar_qrcode(State(state), Json(req)).await
}

// Helper
fn build_erro_resp(payload: GerarQrCodePreviewReq, mensagem: String) -> QrCodePreviewResp {
    QrCodePreviewResp {
        sucesso: false,
        tipo: format!("{:?}", payload.tipo),
        ambiente: format!("{:?}", payload.ambiente),
        conteudo_qr: None,
        png_base64: None,
        mensagem,
        warnings: vec!["DOCUMENTO TÉCNICO DE HOMOLOGAÇÃO SEM VALIDADE FISCAL".to_string()],
    }
}
