use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use crate::app::AppState;
use crate::routes::fiscal::assinatura::AmbienteFiscal;
use crate::routes::fiscal::historico_homologacao::{registrar_evento_homologacao, RegistrarEventoHomologacaoParams};

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum TipoPreviewValidacao {
    NFCE_XML,
    NFE_XML,
    SIFEN_JSON,
}

#[derive(Debug, Deserialize)]
pub struct ValidarPreviewFiscalReq {
    pub tipo: TipoPreviewValidacao,
    pub conteudo: String,
    pub ambiente: AmbienteFiscal,
}

#[derive(Debug, Serialize, Clone)]
pub enum SeveridadeErro {
    ERRO,
    WARNING,
}

#[derive(Debug, Serialize, Clone)]
pub struct ValidacaoPreviewErroResp {
    pub codigo: String,
    pub campo: Option<String>,
    pub mensagem: String,
    pub severidade: SeveridadeErro,
}

#[derive(Debug, Serialize)]
pub struct ValidacaoPreviewFiscalResp {
    pub valido: bool,
    pub tipo: String,
    pub ambiente: String,
    pub total_erros: usize,
    pub erros: Vec<ValidacaoPreviewErroResp>,
    pub warnings: Vec<ValidacaoPreviewErroResp>,
    pub mensagem: String,
}

pub async fn validar_preview(
    State(state): State<AppState>,
    Json(payload): Json<ValidarPreviewFiscalReq>,
) -> Result<Json<ValidacaoPreviewFiscalResp>, (axum::http::StatusCode, String)> {
    
    let mut erros = Vec::new();
    let mut warnings = Vec::new();

    // Regra 1 e 5: Segurança e escopo
    if payload.conteudo.len() > 5 * 1024 * 1024 {
        erros.push(ValidacaoPreviewErroResp {
            codigo: "VAL_001".to_string(),
            campo: None,
            mensagem: "Conteúdo excede limite de 5MB.".to_string(),
            severidade: SeveridadeErro::ERRO,
        });
        return Ok(Json(build_resp(payload, false, erros, warnings, "Erro de validação (Tamanho excedido)")));
    }

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
                    mensagem: "Tentativa de validar preview em PRODUCAO bloqueada.".into(),
                    payload_hash: None,
                    erro_codigo: Some("AMBIENTE_PRODUCAO_REJEITADO".into()),
                    payload_preview: None,
                }).await;
            });
        }
        erros.push(ValidacaoPreviewErroResp {
            codigo: "VAL_002".to_string(),
            campo: Some("ambiente".to_string()),
            mensagem: "Ambiente PRODUCAO bloqueado. Esta API valida apenas HOMOLOGACAO.".to_string(),
            severidade: SeveridadeErro::ERRO,
        });
        return Ok(Json(build_resp(payload, false, erros, warnings, "Ambiente inválido")));
    }

    // Regra 2 e 3: Validação estrutural de acordo com o tipo
    let mut valido = true;

    match payload.tipo {
        TipoPreviewValidacao::NFCE_XML | TipoPreviewValidacao::NFE_XML => {
            validar_xml_estrutural(&payload.conteudo, &mut erros, &mut warnings);
            
            let schema_path = if payload.tipo == TipoPreviewValidacao::NFE_XML {
                "assets/schemas_fiscal/br/nfe/PL_009_V4"
            } else {
                "assets/schemas_fiscal/br/nfce/PL_009_V4"
            };

            if std::path::Path::new(schema_path).exists() {
                warnings.push(ValidacaoPreviewErroResp {
                    codigo: "INFO_SCHEMA".to_string(),
                    campo: None,
                    mensagem: format!("Schema oficial XSD detectado em {}. O validador será migrado para usar este schema no próximo bloco.", schema_path),
                    severidade: SeveridadeErro::WARNING,
                });
            } else {
                warnings.push(ValidacaoPreviewErroResp {
                    codigo: "WARN_XSD".to_string(),
                    campo: None,
                    mensagem: "Validação estrutural XML simplificada. Schema oficial XSD não encontrado no servidor.".to_string(),
                    severidade: SeveridadeErro::WARNING,
                });
            }
        },
        TipoPreviewValidacao::SIFEN_JSON => {
            validar_sifen_estrutural(&payload.conteudo, &mut erros, &mut warnings);
            
            if std::path::Path::new("assets/schemas_fiscal/py/sifen").exists() {
                warnings.push(ValidacaoPreviewErroResp {
                    codigo: "INFO_SCHEMA".to_string(),
                    campo: None,
                    mensagem: "Pasta do Schema oficial SIFEN detectada. O validador será atualizado em breve.".to_string(),
                    severidade: SeveridadeErro::WARNING,
                });
            } else {
                warnings.push(ValidacaoPreviewErroResp {
                    codigo: "WARN_SCHEMA".to_string(),
                    campo: None,
                    mensagem: "Validação estrutural JSON simplificada. Schema SIFEN ausente.".to_string(),
                    severidade: SeveridadeErro::WARNING,
                });
            }
        }
    }

    if !erros.is_empty() {
        valido = false;
    }

    let msg = if valido {
        "Validação de preview concluída com sucesso (Estrutura Básica OK)."
    } else {
        "Erros de validação encontrados na estrutura do preview."
    };

    // Auditoria técnica — não bloqueia resposta
    if let Some(pool) = &state.pool {
        let pool = pool.clone();
        let sucesso = valido;
        let msg_clone = msg.to_string();
        let tipo_str = format!("{:?}", payload.tipo);
        tokio::spawn(async move {
            let _ = registrar_evento_homologacao(&pool, RegistrarEventoHomologacaoParams {
                tipo_evento: "PREVIEW_VALIDADO_LOCALMENTE",
                pais: None,
                modelo: Some(tipo_str),
                venda_id: None,
                chave_preview: None,
                cdc_preview: None,
                sucesso,
                mensagem: msg_clone,
                payload_hash: None,
                erro_codigo: None,
                payload_preview: None,
            }).await;
        });
    }

    Ok(Json(build_resp(payload, valido, erros, warnings, msg)))
}

pub async fn validar_xml(
    State(state): State<AppState>,
    Json(mut payload): Json<ValidarPreviewFiscalReq>,
) -> Result<Json<ValidacaoPreviewFiscalResp>, (axum::http::StatusCode, String)> {
    if payload.tipo != TipoPreviewValidacao::NFCE_XML && payload.tipo != TipoPreviewValidacao::NFE_XML {
        payload.tipo = TipoPreviewValidacao::NFCE_XML; // Fallback
    }
    validar_preview(State(state), Json(payload)).await
}

pub async fn validar_sifen(
    State(state): State<AppState>,
    Json(mut payload): Json<ValidarPreviewFiscalReq>,
) -> Result<Json<ValidacaoPreviewFiscalResp>, (axum::http::StatusCode, String)> {
    payload.tipo = TipoPreviewValidacao::SIFEN_JSON;
    validar_preview(State(state), Json(payload)).await
}

// Helpers de validação
fn build_resp(
    req: ValidarPreviewFiscalReq,
    valido: bool,
    erros: Vec<ValidacaoPreviewErroResp>,
    warnings: Vec<ValidacaoPreviewErroResp>,
    mensagem: &str,
) -> ValidacaoPreviewFiscalResp {
    ValidacaoPreviewFiscalResp {
        valido,
        tipo: format!("{:?}", req.tipo),
        ambiente: format!("{:?}", req.ambiente),
        total_erros: erros.len(),
        erros,
        warnings,
        mensagem: mensagem.to_string(),
    }
}

fn validar_xml_estrutural(conteudo: &str, erros: &mut Vec<ValidacaoPreviewErroResp>, _warnings: &mut Vec<ValidacaoPreviewErroResp>) {
    // Busca simplificada de tags essenciais
    let required_tags = [
        "<NFe", "<infNFe", "<ide>", "<emit>", "<det", "<total>", "<pag>", "<infAdic>"
    ];

    for tag in required_tags.iter() {
        if !conteudo.contains(tag) {
            erros.push(ValidacaoPreviewErroResp {
                codigo: "XML_001".to_string(),
                campo: Some(tag.to_string()),
                mensagem: format!("Tag obrigatória ausente no XML: {}", tag),
                severidade: SeveridadeErro::ERRO,
            });
        }
    }

    if conteudo.contains("<tpAmb>1</tpAmb>") {
        erros.push(ValidacaoPreviewErroResp {
            codigo: "XML_002".to_string(),
            campo: Some("tpAmb".to_string()),
            mensagem: "O XML indica tpAmb=1 (Produção). O preview de homologação exige tpAmb=2.".to_string(),
            severidade: SeveridadeErro::ERRO,
        });
    } else if !conteudo.contains("<tpAmb>2</tpAmb>") {
        erros.push(ValidacaoPreviewErroResp {
            codigo: "XML_003".to_string(),
            campo: Some("tpAmb".to_string()),
            mensagem: "A tag <tpAmb>2</tpAmb> (Homologação) é obrigatória.".to_string(),
            severidade: SeveridadeErro::ERRO,
        });
    }

    let aviso_msg = "DOCUMENTO TECNICO DE HOMOLOGACAO SEM VALIDADE FISCAL";
    if !conteudo.contains(aviso_msg) && !conteudo.contains("DOCUMENTO TÉCNICO DE HOMOLOGAÇÃO SEM VALIDADE FISCAL") {
        erros.push(ValidacaoPreviewErroResp {
            codigo: "XML_004".to_string(),
            campo: Some("infAdic".to_string()),
            mensagem: "O XML deve conter o aviso explícito de que é um documento sem validade fiscal.".to_string(),
            severidade: SeveridadeErro::ERRO,
        });
    }
}

fn validar_sifen_estrutural(conteudo: &str, erros: &mut Vec<ValidacaoPreviewErroResp>, _warnings: &mut Vec<ValidacaoPreviewErroResp>) {
    // Tenta fazer o parse do JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(conteudo);
    
    match parsed {
        Ok(json) => {
            // Verifica a raiz rDE ou equivalente
            let rde = json.get("rDE");
            if rde.is_none() {
                erros.push(ValidacaoPreviewErroResp {
                    codigo: "JSON_001".to_string(),
                    campo: Some("rDE".to_string()),
                    mensagem: "Objeto raiz 'rDE' não encontrado no JSON.".to_string(),
                    severidade: SeveridadeErro::ERRO,
                });
                return;
            }

            let rde_obj = rde.unwrap();
            let de = rde_obj.get("DE");
            if de.is_none() {
                erros.push(ValidacaoPreviewErroResp {
                    codigo: "JSON_002".to_string(),
                    campo: Some("DE".to_string()),
                    mensagem: "Objeto 'DE' não encontrado dentro de 'rDE'.".to_string(),
                    severidade: SeveridadeErro::ERRO,
                });
                return;
            }

            // Verifica o Signature = null
            if let Some(sig) = rde_obj.get("Signature") {
                if !sig.is_null() {
                    erros.push(ValidacaoPreviewErroResp {
                        codigo: "JSON_003".to_string(),
                        campo: Some("Signature".to_string()),
                        mensagem: "A tag 'Signature' deve ser nula/vazia em ambiente de preview.".to_string(),
                        severidade: SeveridadeErro::ERRO,
                    });
                }
            } else {
                erros.push(ValidacaoPreviewErroResp {
                    codigo: "JSON_004".to_string(),
                    campo: Some("Signature".to_string()),
                    mensagem: "Tag 'Signature' não encontrada (deveria ser null).".to_string(),
                    severidade: SeveridadeErro::ERRO,
                });
            }

            // Verifica o aviso de invalidade
            let json_str = json.to_string();
            let aviso_msg = "DOCUMENTO TECNICO DE HOMOLOGACAO SEM VALIDADE FISCAL";
            if !json_str.contains(aviso_msg) && !json_str.contains("DOCUMENTO TÉCNICO DE HOMOLOGAÇÃO SEM VALIDADE FISCAL") {
                erros.push(ValidacaoPreviewErroResp {
                    codigo: "JSON_005".to_string(),
                    campo: Some("observacion".to_string()),
                    mensagem: "O JSON deve conter o aviso explícito de que é um documento sem validade fiscal.".to_string(),
                    severidade: SeveridadeErro::ERRO,
                });
            }

            // Verifica as chaves principais dentro do DE
            let de_obj = de.unwrap();
            let mut chaves_ausentes = Vec::new();
            for key in &["gTimb", "gDatGralOpe", "gDtipDE", "gTotSub"] {
                if de_obj.get(key).is_none() {
                    chaves_ausentes.push(*key);
                }
            }

            if !chaves_ausentes.is_empty() {
                erros.push(ValidacaoPreviewErroResp {
                    codigo: "JSON_006".to_string(),
                    campo: Some(chaves_ausentes.join(", ")),
                    mensagem: format!("Chaves estruturais ausentes no DE: {:?}", chaves_ausentes),
                    severidade: SeveridadeErro::ERRO,
                });
            }

        },
        Err(e) => {
            erros.push(ValidacaoPreviewErroResp {
                codigo: "JSON_PARSE_ERR".to_string(),
                campo: None,
                mensagem: format!("Falha no parse do JSON SIFEN: {}", e),
                severidade: SeveridadeErro::ERRO,
            });
        }
    }
}
