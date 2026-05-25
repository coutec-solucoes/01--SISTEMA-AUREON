use axum::{extract::{State, Path}, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use crate::app::AppState;
use crate::routes::fiscal::assinatura::AmbienteFiscal;
use crate::routes::fiscal::historico_homologacao::{registrar_evento_homologacao, RegistrarEventoHomologacaoParams};

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum TipoDocumentoSifen {
    FACTURA_ELECTRONICA,
    NOTA_CREDITO_PREVIEW,
    OUTRO,
}

#[derive(Debug, Deserialize)]
pub struct MontarSifenPreviewReq {
    pub venda_id: String,
    pub ambiente: AmbienteFiscal,
    pub tipo_documento_preview: TipoDocumentoSifen,
    pub timbrado_preview: Option<String>,
    pub numero_preview: Option<String>,
    pub serie_preview: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SifenPreviewResp {
    pub sucesso: bool,
    pub venda_id: String,
    pub ambiente: String,
    pub tipo_documento_preview: String,
    pub json_preview: Option<serde_json::Value>,
    pub cdc_preview: Option<String>,
    pub numero_preview: Option<String>,
    pub serie_preview: Option<String>,
    pub total_geral_minor: Option<i64>,
    pub total_iva_10_minor: Option<i64>,
    pub total_iva_5_minor: Option<i64>,
    pub total_exento_minor: Option<i64>,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

pub async fn montar_preview(
    State(state): State<AppState>,
    Json(payload): Json<MontarSifenPreviewReq>,
) -> Result<Json<SifenPreviewResp>, (axum::http::StatusCode, String)> {
    
    let mut warnings = vec![
        "DOCUMENTO TÉCNICO DE HOMOLOGAÇÃO SEM VALIDADE FISCAL".to_string(),
        "Nenhum KuDE ou transmissão oficial SIFEN será gerada.".to_string(),
    ];

    if payload.ambiente == AmbienteFiscal::PRODUCAO {
        // Auditoria: tentativa de PRODUCAO bloqueada
        if let Some(pool) = &state.pool {
            let pool = pool.clone();
            let vid = payload.venda_id.clone();
            tokio::spawn(async move {
                let _ = registrar_evento_homologacao(&pool, RegistrarEventoHomologacaoParams {
                    tipo_evento: "PRODUCAO_BLOQUEADA",
                    pais: Some("PY".into()),
                    modelo: Some("SIFEN".into()),
                    venda_id: uuid::Uuid::parse_str(&vid).ok(),
                    chave_preview: None,
                    cdc_preview: None,
                    sucesso: false,
                    mensagem: "Tentativa de gerar preview SIFEN em PRODUCAO bloqueada.".into(),
                    payload_hash: None,
                    erro_codigo: Some("AMBIENTE_PRODUCAO_REJEITADO".into()),
                    payload_preview: None,
                }).await;
            });
        }
        return Ok(Json(SifenPreviewResp {
            sucesso: false,
            venda_id: payload.venda_id.clone(),
            ambiente: format!("{:?}", payload.ambiente),
            tipo_documento_preview: format!("{:?}", payload.tipo_documento_preview),
            json_preview: None,
            cdc_preview: None,
            numero_preview: None,
            serie_preview: None,
            total_geral_minor: None,
            total_iva_10_minor: None,
            total_iva_5_minor: None,
            total_exento_minor: None,
            mensagem: "Ambiente de produção não permitido para geração de preview técnico SIFEN.".to_string(),
            warnings,
        }));
    }

    let pool = state.pool.as_ref().ok_or((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Sem banco de dados".to_string()))?;

    // Query genérica para a venda
    let venda_query = "SELECT id, cliente_id, total_venda, total_itens, fiscal_iva_10_valor_preview, fiscal_iva_5_valor_preview, fiscal_iva_exento_valor_preview, fiscal_pronto FROM pdv_vendas WHERE id = $1";
    
    let venda_row = match sqlx::query(venda_query).bind(&payload.venda_id).fetch_optional(pool).await {
        Ok(Some(r)) => r,
        Ok(None) => return Ok(Json(SifenPreviewResp {
            sucesso: false,
            venda_id: payload.venda_id.clone(),
            ambiente: format!("{:?}", payload.ambiente),
            tipo_documento_preview: format!("{:?}", payload.tipo_documento_preview),
            json_preview: None,
            cdc_preview: None,
            numero_preview: None,
            serie_preview: None,
            total_geral_minor: None,
            total_iva_10_minor: None,
            total_iva_5_minor: None,
            total_exento_minor: None,
            mensagem: "Venda não encontrada.".to_string(),
            warnings,
        })),
        Err(e) => {
            warnings.push(format!("Erro ao acessar pdv_vendas: {}. Usando dados mock SIFEN para preview.", e));
            return gerar_mock_preview(payload, warnings);
        }
    };

    let fiscal_pronto: bool = venda_row.try_get("fiscal_pronto").unwrap_or(false);
    if !fiscal_pronto {
        return Ok(Json(SifenPreviewResp {
            sucesso: false,
            venda_id: payload.venda_id.clone(),
            ambiente: format!("{:?}", payload.ambiente),
            tipo_documento_preview: format!("{:?}", payload.tipo_documento_preview),
            json_preview: None,
            cdc_preview: None,
            numero_preview: None,
            serie_preview: None,
            total_geral_minor: None,
            total_iva_10_minor: None,
            total_iva_5_minor: None,
            total_exento_minor: None,
            mensagem: "Espelho fiscal IVA não calculado. Calcule o imposto da venda antes de gerar o preview SIFEN.".to_string(),
            warnings,
        }));
    }

    let total_venda: i64 = venda_row.try_get("total_venda").unwrap_or(0);
    let iva_10_minor: i64 = venda_row.try_get("fiscal_iva_10_valor_preview").unwrap_or(0);
    let iva_5_minor: i64 = venda_row.try_get("fiscal_iva_5_valor_preview").unwrap_or(0);
    let iva_exento_minor: i64 = venda_row.try_get("fiscal_iva_exento_valor_preview").unwrap_or(0);

    let numero = payload.numero_preview.clone().unwrap_or_else(|| "9999999".to_string());
    let serie = payload.serie_preview.clone().unwrap_or_else(|| "AA1".to_string());
    let timbrado = payload.timbrado_preview.clone().unwrap_or_else(|| "12345678".to_string());
    let cdc = format!("01{}001{}000{}00{}1234567895", "12345678000199", timbrado, serie, numero);

    let json_preview = build_json_sifen(
        &numero,
        &serie,
        &timbrado,
        total_venda,
        iva_10_minor,
        iva_5_minor,
        iva_exento_minor,
    );

    // Auditoria técnica — não bloqueia resposta
    if let Some(pool) = &state.pool {
        let pool = pool.clone();
        let vid = payload.venda_id.clone();
        let cdc_clone = cdc.clone();
        let doc_tipo = format!("{:?}", payload.tipo_documento_preview);
        tokio::spawn(async move {
            let _ = registrar_evento_homologacao(&pool, RegistrarEventoHomologacaoParams {
                tipo_evento: "SIFEN_PREVIEW_GERADO",
                pais: Some("PY".into()),
                modelo: Some(doc_tipo),
                venda_id: uuid::Uuid::parse_str(&vid).ok(),
                chave_preview: None,
                cdc_preview: Some(cdc_clone),
                sucesso: true,
                mensagem: "Preview JSON SIFEN/DTE gerado com sucesso. Não representa autorização fiscal.".into(),
                payload_hash: None,
                erro_codigo: None,
                payload_preview: Some(serde_json::json!({
                    "aviso": "Preview técnico SIFEN, sem validade fiscal"
                })),
            }).await;
        });
    }

    Ok(Json(SifenPreviewResp {
        sucesso: true,
        venda_id: payload.venda_id.clone(),
        ambiente: format!("{:?}", payload.ambiente),
        tipo_documento_preview: format!("{:?}", payload.tipo_documento_preview),
        json_preview: Some(json_preview),
        cdc_preview: Some(cdc),
        numero_preview: Some(numero),
        serie_preview: Some(serie),
        total_geral_minor: Some(total_venda),
        total_iva_10_minor: Some(iva_10_minor),
        total_iva_5_minor: Some(iva_5_minor),
        total_exento_minor: Some(iva_exento_minor),
        mensagem: "Preview técnico JSON SIFEN/DTE gerado com sucesso.".to_string(),
        warnings,
    }))
}

pub async fn get_venda_preview(
    State(state): State<AppState>,
    Path(venda_id): Path<String>,
) -> Result<Json<SifenPreviewResp>, (axum::http::StatusCode, String)> {
    let req = MontarSifenPreviewReq {
        venda_id,
        ambiente: AmbienteFiscal::HOMOLOGACAO,
        tipo_documento_preview: TipoDocumentoSifen::FACTURA_ELECTRONICA,
        timbrado_preview: None,
        numero_preview: None,
        serie_preview: None,
    };
    montar_preview(State(state), Json(req)).await
}

fn build_json_sifen(
    numero: &str,
    serie: &str,
    timbrado: &str,
    total_minor: i64,
    iva_10_minor: i64,
    iva_5_minor: i64,
    exento_minor: i64,
) -> serde_json::Value {
    // PYG não usa centavos (minor_unit = real unit no paraguai em dinheiro vivo).
    // O sistema mapeará minor unit -> string sem decimais.
    let total_str = total_minor.to_string();
    let iva_10_str = iva_10_minor.to_string();
    let iva_5_str = iva_5_minor.to_string();
    let exento_str = exento_minor.to_string();

    serde_json::json!({
        "rDE": {
            "dVerFor": "150",
            "DE": {
                "gTimb": {
                    "iTiDE": 1,
                    "dNumTim": timbrado,
                    "dEst": serie,
                    "dPunExp": "001",
                    "dNumDoc": numero,
                    "dFeIniT": "2024-01-01",
                },
                "gDatGralOpe": {
                    "dFeEmiDE": "2026-05-25T12:00:00",
                    "gOpeCom": {
                        "iTipTra": 1,
                        "iTImp": 1,
                        "cMoneOpe": "PYG",
                    },
                    "gEmis": {
                        "dRucEm": "12345678",
                        "dDVEmi": "9",
                        "iTipCont": 1,
                        "dNomEmi": "EMPRESA MOCK SIFEN HOMOLOGACAO",
                    },
                    "gDatRec": {
                        "iNatRec": 1,
                        "iTiOpe": 1,
                        "cPaisRec": "PRY",
                        "iTiContRec": 2,
                        "dRucRec": "8888888",
                        "dDVRec": "1",
                        "dNomRec": "CONSUMIDOR FINAL PREVIEW",
                    }
                },
                "gDtipDE": {
                    "gCamItem": [
                        {
                            "dCodInt": "1",
                            "dDesProSer": "Produto SIFEN Preview",
                            "cUniMed": 77,
                            "dCantProSer": "1.000",
                            "gValorItem": {
                                "dPUniProSer": total_str,
                                "dTotBruOpeItem": total_str,
                            },
                            "gCamIVA": {
                                "iAfeIVA": 1,
                                "dPropIVA": "100",
                                "dTasaIVA": "10",
                                "dBasGravIVA": total_str,
                                "dLiqIVAItem": iva_10_str
                            }
                        }
                    ]
                },
                "gTotSub": {
                    "dSubExe": exento_str,
                    "dSub5": "0",
                    "dSub10": total_str,
                    "dTotOpe": total_str,
                    "dTotDesc": "0",
                    "dTotDescGlotem": "0",
                    "dTotAntItem": "0",
                    "dTotAnt": "0",
                    "dPorcDescTotal": "0",
                    "dTotOpeGs": total_str,
                    "dIVA5": iva_5_str,
                    "dIVA10": iva_10_str,
                    "dTotIVA": iva_10_str,
                    "dBaseGrav5": "0",
                    "dBaseGrav10": total_str,
                    "dTBasGraIVA": total_str
                },
                "gCamGen": {
                    "dObs": "DOCUMENTO TECNICO DE HOMOLOGACAO SEM VALIDADE FISCAL",
                }
            },
            "Signature": null
        }
    })
}

fn gerar_mock_preview(payload: MontarSifenPreviewReq, warnings: Vec<String>) -> Result<Json<SifenPreviewResp>, (axum::http::StatusCode, String)> {
    let numero = payload.numero_preview.clone().unwrap_or_else(|| "9999999".to_string());
    let serie = payload.serie_preview.clone().unwrap_or_else(|| "AA1".to_string());
    let timbrado = payload.timbrado_preview.clone().unwrap_or_else(|| "12345678".to_string());
    let cdc = format!("01{}001{}000{}00{}1234567895", "12345678000199", timbrado, serie, numero);
    
    // Total fictício 110000 PYG (base 100000, iva 10000)
    let json_preview = build_json_sifen(&numero, &serie, &timbrado, 110000, 10000, 0, 0);

    Ok(Json(SifenPreviewResp {
        sucesso: true,
        venda_id: payload.venda_id,
        ambiente: format!("{:?}", payload.ambiente),
        tipo_documento_preview: format!("{:?}", payload.tipo_documento_preview),
        json_preview: Some(json_preview),
        cdc_preview: Some(cdc),
        numero_preview: Some(numero),
        serie_preview: Some(serie),
        total_geral_minor: Some(110000),
        total_iva_10_minor: Some(10000),
        total_iva_5_minor: Some(0),
        total_exento_minor: Some(0),
        mensagem: "Venda simulada gerada via mock SIFEN devido à ausência das tabelas na retaguarda.".to_string(),
        warnings,
    }))
}
