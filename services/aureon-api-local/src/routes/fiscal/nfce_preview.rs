use axum::{extract::{State, Path}, Json};
use serde::{Deserialize, Serialize};
use std::fs;
use sqlx::Row;
use crate::app::AppState;
use crate::routes::fiscal::assinatura::{assinar_preview, AssinarXmlPreviewReq, AlgoritmoAssinatura, AmbienteFiscal};
use crate::routes::fiscal::historico_homologacao::{registrar_evento_homologacao, RegistrarEventoHomologacaoParams};

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum ModeloDocumento {
    NFCE,
    NFE,
}

#[derive(Debug, Deserialize)]
pub struct MontarNfcePreviewReq {
    pub venda_id: String,
    pub modelo: ModeloDocumento,
    pub ambiente: AmbienteFiscal,
    pub uf: String,
    pub serie_preview: Option<String>,
    pub numero_preview: Option<String>,
    pub assinar_preview: bool,
    pub caminho_pfx: Option<String>,
    pub senha_pfx: Option<String>,
    pub conteudo_base64: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NfcePreviewResp {
    pub sucesso: bool,
    pub venda_id: String,
    pub modelo: String,
    pub ambiente: String,
    pub uf: String,
    pub xml_preview: Option<String>,
    pub xml_assinado_preview: Option<String>,
    pub chave_preview: Option<String>,
    pub numero_preview: Option<String>,
    pub serie_preview: Option<String>,
    pub total_base_minor: Option<i64>,
    pub total_imposto_minor: Option<i64>,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

pub async fn montar_preview(
    State(state): State<AppState>,
    Json(payload): Json<MontarNfcePreviewReq>,
) -> Result<Json<NfcePreviewResp>, (axum::http::StatusCode, String)> {
    
    let mut warnings = vec![
        "DOCUMENTO TÉCNICO DE HOMOLOGAÇÃO SEM VALIDADE FISCAL".to_string(),
        "Nenhum protocolo ou autorização oficial será gerado.".to_string(),
    ];

    if payload.ambiente == AmbienteFiscal::PRODUCAO {
        // Auditoria: tentativa de PRODUCAO bloqueada
        if let Some(pool) = &state.pool {
            let pool = pool.clone();
            let vid = payload.venda_id.clone();
            tokio::spawn(async move {
                let _ = registrar_evento_homologacao(&pool, RegistrarEventoHomologacaoParams {
                    tipo_evento: "PRODUCAO_BLOQUEADA",
                    pais: Some("BR".into()),
                    modelo: Some("NFCE".into()),
                    venda_id: uuid::Uuid::parse_str(&vid).ok(),
                    chave_preview: None,
                    cdc_preview: None,
                    sucesso: false,
                    mensagem: "Tentativa de gerar preview em PRODUCAO bloqueada.".into(),
                    payload_hash: None,
                    erro_codigo: Some("AMBIENTE_PRODUCAO_REJEITADO".into()),
                    payload_preview: None,
                }).await;
            });
        }
        return Ok(Json(NfcePreviewResp {
            sucesso: false,
            venda_id: payload.venda_id.clone(),
            modelo: format!("{:?}", payload.modelo),
            ambiente: format!("{:?}", payload.ambiente),
            uf: payload.uf.clone(),
            xml_preview: None,
            xml_assinado_preview: None,
            chave_preview: None,
            numero_preview: None,
            serie_preview: None,
            total_base_minor: None,
            total_imposto_minor: None,
            mensagem: "Ambiente de produção não permitido para geração de preview técnico.".to_string(),
            warnings,
        }));
    }

    let pool = state.pool.as_ref().ok_or((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Sem banco de dados".to_string()))?;

    // As tabelas de vendas podem estar no PostgreSQL ou sincronizadas via eventos.
    // Usaremos uma query genérica assumindo a estrutura de vendas sincronizada.
    
    // Tenta buscar a venda
    let venda_query = "SELECT id, cliente_id, total_venda, total_itens, fiscal_icms_base_preview, fiscal_icms_valor_preview, fiscal_pronto FROM pdv_vendas WHERE id = $1";
    let venda_row = match sqlx::query(venda_query).bind(&payload.venda_id).fetch_optional(pool).await {
        Ok(Some(r)) => r,
        Ok(None) => return Ok(Json(NfcePreviewResp {
            sucesso: false,
            venda_id: payload.venda_id.clone(),
            modelo: format!("{:?}", payload.modelo),
            ambiente: format!("{:?}", payload.ambiente),
            uf: payload.uf.clone(),
            xml_preview: None,
            xml_assinado_preview: None,
            chave_preview: None,
            numero_preview: None,
            serie_preview: None,
            total_base_minor: None,
            total_imposto_minor: None,
            mensagem: "Venda não encontrada.".to_string(),
            warnings,
        })),
        Err(e) => {
            // Mock de fallback se a tabela não existir para não quebrar a homologação
            warnings.push(format!("Erro ao acessar pdv_vendas: {}. Usando dados mock para preview.", e));
            return gerar_mock_preview(payload, warnings);
        }
    };

    let fiscal_pronto: bool = venda_row.try_get("fiscal_pronto").unwrap_or(false);
    if !fiscal_pronto {
        return Ok(Json(NfcePreviewResp {
            sucesso: false,
            venda_id: payload.venda_id.clone(),
            modelo: format!("{:?}", payload.modelo),
            ambiente: format!("{:?}", payload.ambiente),
            uf: payload.uf.clone(),
            xml_preview: None,
            xml_assinado_preview: None,
            chave_preview: None,
            numero_preview: None,
            serie_preview: None,
            total_base_minor: None,
            total_imposto_minor: None,
            mensagem: "Espelho fiscal não calculado. Calcule o imposto da venda antes de gerar o preview.".to_string(),
            warnings,
        }));
    }

    let base_minor: i64 = venda_row.try_get("fiscal_icms_base_preview").unwrap_or(0);
    let imposto_minor: i64 = venda_row.try_get("fiscal_icms_valor_preview").unwrap_or(0);
    let total_venda: i64 = venda_row.try_get("total_venda").unwrap_or(0);

    let numero = payload.numero_preview.unwrap_or_else(|| "999999".to_string());
    let serie = payload.serie_preview.unwrap_or_else(|| "999".to_string());
    let chave = format!("{}{}000019955{}000{}123456789", payload.uf, "2605", serie, numero);

    let xml_preview = build_xml_string(
        &payload.uf,
        &numero,
        &serie,
        total_venda,
        base_minor,
        imposto_minor,
    );

    let mut xml_assinado = None;
    if payload.assinar_preview {
        if let Some(senha) = payload.senha_pfx.clone() {
            let req_ass = AssinarXmlPreviewReq {
                xml: xml_preview.clone(),
                caminho_pfx: payload.caminho_pfx.clone(),
                conteudo_base64: payload.conteudo_base64.clone(),
                senha_pfx: senha,
                empresa_id: None,
                filial_id: None,
                algoritmo: AlgoritmoAssinatura::RSA_SHA256,
                ambiente: AmbienteFiscal::HOMOLOGACAO,
            };

            match assinar_preview(State(state.clone()), Json(req_ass)).await {
                Ok(res) => {
                    if res.sucesso {
                        xml_assinado = res.xml_assinado.clone(); // Pega da resposta
                        warnings.push("XML assinado tecnicamente com sucesso.".to_string());
                    } else {
                        warnings.push(format!("Falha na assinatura: {}", res.mensagem));
                    }
                },
                Err((_, msg)) => {
                    warnings.push(format!("Erro interno ao assinar: {}", msg));
                }
            }
        } else {
            warnings.push("Senha do certificado não informada para assinatura.".to_string());
        }
    }

    // Auditoria técnica — não bloqueia resposta
    if let Some(pool) = &state.pool {
        let pool = pool.clone();
        let vid = payload.venda_id.clone();
        let chave_clone = chave.clone();
        let modelo_str = format!("{:?}", payload.modelo);
        tokio::spawn(async move {
            let _ = registrar_evento_homologacao(&pool, RegistrarEventoHomologacaoParams {
                tipo_evento: "NFCE_PREVIEW_GERADO",
                pais: Some("BR".into()),
                modelo: Some(modelo_str),
                venda_id: uuid::Uuid::parse_str(&vid).ok(),
                chave_preview: Some(chave_clone),
                cdc_preview: None,
                sucesso: true,
                mensagem: "Preview XML NF-e/NFC-e gerado com sucesso. Não representa autorização fiscal.".into(),
                payload_hash: None,
                erro_codigo: None,
                payload_preview: Some(serde_json::json!({
                    "aviso": "Preview técnico, sem validade fiscal"
                })),
            }).await;
        });
    }

    Ok(Json(NfcePreviewResp {
        sucesso: true,
        venda_id: payload.venda_id.clone(),
        modelo: format!("{:?}", payload.modelo),
        ambiente: format!("{:?}", payload.ambiente),
        uf: payload.uf.clone(),
        xml_preview: Some(xml_preview),
        xml_assinado_preview: xml_assinado,
        chave_preview: Some(chave),
        numero_preview: Some(numero),
        serie_preview: Some(serie),
        total_base_minor: Some(base_minor),
        total_imposto_minor: Some(imposto_minor),
        mensagem: "Preview técnico XML gerado com sucesso.".to_string(),
        warnings,
    }))
}

pub async fn montar_assinar_preview(
    State(state): State<AppState>,
    Json(mut payload): Json<MontarNfcePreviewReq>,
) -> Result<Json<NfcePreviewResp>, (axum::http::StatusCode, String)> {
    payload.assinar_preview = true;
    montar_preview(State(state), Json(payload)).await
}

pub async fn get_venda_preview(
    State(state): State<AppState>,
    Path(venda_id): Path<String>,
) -> Result<Json<NfcePreviewResp>, (axum::http::StatusCode, String)> {
    let req = MontarNfcePreviewReq {
        venda_id,
        modelo: ModeloDocumento::NFCE,
        ambiente: AmbienteFiscal::HOMOLOGACAO,
        uf: "SP".to_string(),
        serie_preview: None,
        numero_preview: None,
        assinar_preview: false,
        caminho_pfx: None,
        senha_pfx: None,
        conteudo_base64: None,
    };
    montar_preview(State(state), Json(req)).await
}

// Helpers para conversão de escala e formatação
fn minor_to_str(minor: i64) -> String {
    let reais = minor / 100;
    let centavos = minor % 100;
    format!("{}.{:02}", reais, centavos)
}

fn build_xml_string(
    uf: &str,
    numero: &str,
    serie: &str,
    total_minor: i64,
    base_minor: i64,
    imposto_minor: i64,
) -> String {
    let total_str = minor_to_str(total_minor);
    let base_str = minor_to_str(base_minor);
    let imposto_str = minor_to_str(imposto_minor);

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe">
    <infNFe Id="NFe{uf}2605{serie}{numero}123456789" versao="4.00">
        <ide>
            <cUF>{uf}</cUF>
            <natOp>VENDA PREVIEW</natOp>
            <mod>65</mod>
            <serie>{serie}</serie>
            <nNF>{numero}</nNF>
            <tpAmb>2</tpAmb>
        </ide>
        <emit>
            <CNPJ>12345678000199</CNPJ>
            <xNome>EMPRESA MOCK HOMOLOGACAO</xNome>
        </emit>
        <det nItem="1">
            <prod>
                <cProd>1</cProd>
                <xProd>Produto Teste Preview</xProd>
                <NCM>99999999</NCM>
                <CFOP>5102</CFOP>
                <qCom>1.000</qCom>
                <vUnCom>{total_str}</vUnCom>
                <vProd>{total_str}</vProd>
            </prod>
            <imposto>
                <ICMS>
                    <ICMS00>
                        <CST>00</CST>
                        <vBC>{base_str}</vBC>
                        <pICMS>10.00</pICMS>
                        <vICMS>{imposto_str}</vICMS>
                    </ICMS00>
                </ICMS>
            </imposto>
        </det>
        <total>
            <ICMSTot>
                <vBC>{base_str}</vBC>
                <vICMS>{imposto_str}</vICMS>
                <vProd>{total_str}</vProd>
                <vNF>{total_str}</vNF>
            </ICMSTot>
        </total>
        <pag>
            <detPag>
                <tPag>01</tPag>
                <vPag>{total_str}</vPag>
            </detPag>
        </pag>
        <infAdic>
            <infCpl>DOCUMENTO TECNICO DE HOMOLOGACAO SEM VALIDADE FISCAL</infCpl>
        </infAdic>
    </infNFe>
</NFe>"#
    )
}

fn gerar_mock_preview(payload: MontarNfcePreviewReq, warnings: Vec<String>) -> Result<Json<NfcePreviewResp>, (axum::http::StatusCode, String)> {
    let numero = payload.numero_preview.unwrap_or_else(|| "999999".to_string());
    let serie = payload.serie_preview.unwrap_or_else(|| "999".to_string());
    let chave = format!("{}{}000019955{}000{}123456789", payload.uf, "2605", serie, numero);
    let xml_preview = build_xml_string(&payload.uf, &numero, &serie, 10000, 10000, 1000);

    Ok(Json(NfcePreviewResp {
        sucesso: true,
        venda_id: payload.venda_id,
        modelo: format!("{:?}", payload.modelo),
        ambiente: format!("{:?}", payload.ambiente),
        uf: payload.uf,
        xml_preview: Some(xml_preview),
        xml_assinado_preview: None,
        chave_preview: Some(chave),
        numero_preview: Some(numero),
        serie_preview: Some(serie),
        total_base_minor: Some(10000),
        total_imposto_minor: Some(1000),
        mensagem: "Venda simulada gerada via mock devido à ausência das tabelas na retaguarda.".to_string(),
        warnings,
    }))
}
