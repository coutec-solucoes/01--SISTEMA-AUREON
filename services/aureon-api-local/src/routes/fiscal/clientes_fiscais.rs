/// clientes_fiscais.rs
/// Fase 19 — Bloco 4
/// Cliente Fiscal de Homologação: Conectividade mTLS Diagnóstica sem Envio de Documento
///
/// ATENÇÃO CRÍTICA:
/// - Este módulo NÃO transmite NF-e/NFC-e.
/// - Este módulo NÃO transmite DTE/SIFEN.
/// - Este módulo NÃO envia XML de documento fiscal.
/// - Este módulo NÃO autoriza documento.
/// - Este módulo NÃO consulta status de nota real.
/// - Este módulo NÃO gera protocolo.
/// - Este módulo NÃO gera DANFE/KuDE.
/// - Qualquer resposta HTTP recebida (200, 403, 405, 500) é apenas diagnóstico
///   de conectividade. Não representa autorização, protocolo ou status fiscal.
///
/// Objetivo: verificar se os endpoints de homologação são acessíveis via TLS/mTLS,
/// retornando diagnóstico de rede sem envio de payload fiscal de qualquer natureza.

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use crate::app::AppState;
use crate::routes::fiscal::homologacao::registry_endpoints_homologacao;
use crate::routes::fiscal::historico_homologacao::{registrar_evento_homologacao, RegistrarEventoHomologacaoParams};

// ─────────────────────────────────────────────
// CONSTANTES DE SEGURANÇA
// ─────────────────────────────────────────────

/// Timeout máximo permitido: 10 segundos.
const TIMEOUT_MAX_MS: u64 = 10_000;
/// Timeout padrão: 5 segundos.
const TIMEOUT_PADRAO_MS: u64 = 5_000;

// ─────────────────────────────────────────────
// DTOs
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TestarConectividadeFiscalReq {
    pub pais: String,
    pub modelo: String,
    pub uf: Option<String>,
    pub servico: String,
    pub ambiente: String,
    pub usar_mtls: Option<bool>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct TestarConectividadeFiscalResp {
    pub sucesso: bool,
    pub ambiente: String,
    pub url: String,
    pub producao_bloqueada: bool,
    pub dns_ok: bool,
    pub tls_ok: bool,
    pub mtls_usado: bool,
    pub certificado_configurado: bool,
    pub http_status: Option<u16>,
    pub tempo_ms: u64,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

// ─────────────────────────────────────────────
// HANDLER
// ─────────────────────────────────────────────

/// POST /fiscal/homologacao/testar-conectividade
///
/// Testa conectividade TLS/mTLS com endpoints de homologação.
/// NÃO envia XML de nota, NÃO envia DTE, NÃO realiza autorização.
/// Qualquer resposta HTTP (200/403/405/500) é diagnóstico de rede, não status fiscal.
pub async fn testar_conectividade(
    State(state): State<AppState>,
    Json(payload): Json<TestarConectividadeFiscalReq>,
) -> Result<Json<TestarConectividadeFiscalResp>, (axum::http::StatusCode, String)> {
    let mut warnings: Vec<String> = Vec::new();
    let usar_mtls = payload.usar_mtls.unwrap_or(false);
    let timeout_ms = payload.timeout_ms
        .map(|t| t.min(TIMEOUT_MAX_MS))
        .unwrap_or(TIMEOUT_PADRAO_MS);

    // ── Bloqueio 1: Ambiente PRODUCAO ────────────────────────────────────
    if payload.ambiente.to_uppercase() == "PRODUCAO" {
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
                    mensagem: "Tentativa de testar conectividade em PRODUCAO bloqueada.".into(),
                    payload_hash: None,
                    erro_codigo: Some("AMBIENTE_PRODUCAO_REJEITADO".into()),
                    payload_preview: None,
                }).await;
            });
        }
        return Ok(Json(TestarConectividadeFiscalResp {
            sucesso: false,
            ambiente: payload.ambiente,
            url: String::new(),
            producao_bloqueada: true,
            dns_ok: false,
            tls_ok: false,
            mtls_usado: false,
            certificado_configurado: false,
            http_status: None,
            tempo_ms: 0,
            mensagem: "BLOQUEADO: ambiente PRODUCAO é proibido neste módulo. Use HOMOLOGACAO.".into(),
            warnings: vec!["Tentativa de acesso a ambiente de produção detectada e bloqueada.".into()],
        }));
    }

    // ── Localiza endpoint no registry ────────────────────────────────────
    let registry = registry_endpoints_homologacao();
    let encontrado = registry.iter().find(|e| {
        e.pais == payload.pais.to_uppercase()
            && e.modelo == payload.modelo.to_uppercase()
            && e.servico == payload.servico
            && match &payload.uf {
                Some(uf) => e.uf.as_deref() == Some(uf.as_str()),
                None => true,
            }
    });

    let endpoint = match encontrado {
        None => {
            return Ok(Json(TestarConectividadeFiscalResp {
                sucesso: false,
                ambiente: "HOMOLOGACAO".into(),
                url: String::new(),
                producao_bloqueada: false,
                dns_ok: false,
                tls_ok: false,
                mtls_usado: false,
                certificado_configurado: false,
                http_status: None,
                tempo_ms: 0,
                mensagem: format!(
                    "Endpoint não localizado no registry para pais={}, modelo={}, servico={}.",
                    payload.pais, payload.modelo, payload.servico
                ),
                warnings,
            }));
        }
        Some(ep) => ep,
    };

    let url = endpoint.url.clone();

    // ── Bloqueio 2: URL de produção no registry ──────────────────────────
    if endpoint.producao_bloqueada {
        // producao_bloqueada=true no registry indica que o endpoint BLOQUEIA produção,
        // mas a URL é de homologação. Apenas confirmamos.
    }
    // Verificação extra: se por algum motivo a URL não contiver indicador de homologação
    let url_lower = url.to_lowercase();
    let parece_producao = !url_lower.contains("homologacao")
        && !url_lower.contains("-test")
        && !url_lower.contains("test.")
        && !url_lower.contains("hom.")
        && !url_lower.contains("sandbox");

    if parece_producao {
        return Ok(Json(TestarConectividadeFiscalResp {
            sucesso: false,
            ambiente: "HOMOLOGACAO".into(),
            url,
            producao_bloqueada: true,
            dns_ok: false,
            tls_ok: false,
            mtls_usado: false,
            certificado_configurado: false,
            http_status: None,
            tempo_ms: 0,
            mensagem: "BLOQUEADO: URL não contém indicador de homologação. Acesso negado por segurança.".into(),
            warnings: vec!["URL suspeita de ser produção foi bloqueada antes de qualquer conexão.".into()],
        }));
    }

    // ── Verifica certificado configurado (apenas metadado, sem carregar chave) ─
    let certificado_configurado = std::env::var("FISCAL_CERT_PATH").is_ok()
        || std::env::var("FISCAL_CERT_PFX_B64").is_ok();

    // ── mTLS: disponível apenas com feature fiscal_real ──────────────────
    if usar_mtls && !cfg!(feature = "fiscal_real") {
        warnings.push(
            "mTLS solicitado, mas feature 'fiscal_real' não está ativa. \
             O teste será feito com TLS básico (sem certificado de cliente). \
             Para mTLS real, compile com --features fiscal_real em ambiente Linux/WSL/CI.".into()
        );
    }
    if usar_mtls && !certificado_configurado {
        warnings.push(
            "mTLS solicitado, mas FISCAL_CERT_PATH ou FISCAL_CERT_PFX_B64 não estão configurados. \
             Configure o certificado A1 antes de usar mTLS.".into()
        );
    }

    // ── Executa verificação de conectividade ─────────────────────────────
    let resultado = verificar_conectividade_tls(
        &url,
        usar_mtls,
        certificado_configurado,
        Duration::from_millis(timeout_ms),
        &mut warnings,
    ).await;

    // Auditoria técnica — não bloqueia resposta
    if let Some(pool) = &state.pool {
        let pool = pool.clone();
        let sucesso = resultado.sucesso;
        let url_clone = resultado.url.clone();
        let msg_clone = resultado.mensagem.clone();
        let pais_clone = payload.pais.clone();
        let modelo_clone = payload.modelo.clone();
        let status_code = resultado.http_status;
        tokio::spawn(async move {
            let _ = registrar_evento_homologacao(&pool, RegistrarEventoHomologacaoParams {
                tipo_evento: "CONECTIVIDADE_HOMOLOGACAO_TESTADA",
                pais: Some(pais_clone),
                modelo: Some(modelo_clone),
                venda_id: None,
                chave_preview: None,
                cdc_preview: None,
                sucesso,
                mensagem: msg_clone,
                payload_hash: None,
                erro_codigo: None,
                payload_preview: Some(serde_json::json!({
                    "url": url_clone,
                    "http_status": status_code,
                    "aviso": "Diagnóstico de conectividade. Não representa envio de documento fiscal."
                })),
            }).await;
        });
    }

    Ok(Json(resultado))
}

// ─────────────────────────────────────────────
// LÓGICA DE CONECTIVIDADE TLS
// ─────────────────────────────────────────────

/// Executa a verificação de conectividade via reqwest com rustls.
/// Usa HEAD primeiro; se 405 (Method Not Allowed), registra como conectividade OK.
/// Não envia payload de documento fiscal.
///
/// IMPORTANTE: qualquer resposta HTTP (200, 403, 405, 500) é diagnóstico de
/// infraestrutura de rede — NÃO representa autorização, protocolo ou status fiscal.
async fn verificar_conectividade_tls(
    url: &str,
    _usar_mtls: bool,
    certificado_configurado: bool,
    timeout: Duration,
    warnings: &mut Vec<String>,
) -> TestarConectividadeFiscalResp {
    warnings.push(
        "AVISO CRÍTICO: Esta verificação testa apenas acessibilidade de rede. \
         Qualquer resposta HTTP recebida (200/403/405/500) é diagnóstico de conectividade, \
         NÃO representa autorização fiscal, protocolo ou status de nota.".into()
    );

    // mTLS real apenas disponível com feature fiscal_real em Linux/WSL
    // No build padrão Windows, usamos reqwest com rustls (TLS básico sem cert cliente)
    #[cfg(feature = "fiscal_real")]
    {
        warnings.push(
            "Feature fiscal_real ativa. mTLS com certificado A1 disponível, mas \
             requer FISCAL_CERT_PATH configurado e certificado válido em homologação.".into()
        );
    }

    let inicio = Instant::now();

    // Constrói client reqwest com rustls (sem dependência de OpenSSL no Windows)
    let client_result = reqwest::Client::builder()
        .timeout(timeout)
        .danger_accept_invalid_certs(false) // Nunca aceitar cert inválido
        .user_agent("Aureon-Fiscal-Diagnostico/1.0 (homologacao)")
        .build();

    let client = match client_result {
        Ok(c) => c,
        Err(e) => {
            let tempo_ms = inicio.elapsed().as_millis() as u64;
            return TestarConectividadeFiscalResp {
                sucesso: false,
                ambiente: "HOMOLOGACAO".into(),
                url: url.to_string(),
                producao_bloqueada: false,
                dns_ok: false,
                tls_ok: false,
                mtls_usado: false,
                certificado_configurado,
                http_status: None,
                tempo_ms,
                mensagem: format!("Falha ao construir cliente HTTP: {}", e),
                warnings: warnings.clone(),
            };
        }
    };

    // Tenta HEAD (método leve, sem envio de corpo)
    let resposta = client.head(url).send().await;
    let tempo_ms = inicio.elapsed().as_millis() as u64;

    match resposta {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let tls_ok = url.starts_with("https://");
            let dns_ok = true; // Se chegou resposta, DNS funcionou

            let mensagem = match status {
                200..=299 => format!(
                    "Conectividade OK — HTTP {}. Endpoint acessível em homologação. \
                     Nenhum documento fiscal foi enviado.",
                    status
                ),
                405 => format!(
                    "Conectividade OK — HTTP 405 (Method Not Allowed para HEAD). \
                     O servidor está acessível mas rejeita HEAD. \
                     Endpoints SOAP geralmente não aceitam HEAD — isso é comportamento normal. \
                     Nenhum documento fiscal foi enviado."
                ),
                403 => format!(
                    "Conectividade OK — HTTP 403 (Forbidden). \
                     O servidor está acessível mas requer autenticação/certificado. \
                     Configure o certificado A1 para avançar. \
                     Nenhum documento fiscal foi enviado."
                ),
                401 => format!(
                    "Conectividade OK — HTTP 401 (Unauthorized). \
                     Servidor acessível, mas requer autenticação. \
                     Nenhum documento fiscal foi enviado."
                ),
                _ => format!(
                    "Conectividade testada — HTTP {}. \
                     Este status é diagnóstico de rede, não representa status fiscal.",
                    status
                ),
            };

            TestarConectividadeFiscalResp {
                sucesso: true,
                ambiente: "HOMOLOGACAO".into(),
                url: url.to_string(),
                producao_bloqueada: false,
                dns_ok,
                tls_ok,
                mtls_usado: false, // mTLS real requer feature fiscal_real + cert configurado
                certificado_configurado,
                http_status: Some(status),
                tempo_ms,
                mensagem,
                warnings: warnings.clone(),
            }
        }
        Err(e) => {
            let e_str = e.to_string();
            let dns_ok = !e_str.contains("dns") && !e_str.contains("resolve") && !e_str.contains("Name or service");
            let tls_ok = !e_str.contains("tls") && !e_str.contains("certificate") && !e_str.contains("ssl");
            let timeout_ocorrido = e_str.contains("timed out") || e_str.contains("timeout");

            let mensagem = if timeout_ocorrido {
                format!(
                    "Timeout após {}ms. O endpoint não respondeu dentro do limite configurado. \
                     Verifique conectividade de rede e firewall.",
                    tempo_ms
                )
            } else if !dns_ok {
                format!("Falha de DNS: não foi possível resolver o host do endpoint. Verifique conectividade de rede. Detalhe: {}", e_str)
            } else if !tls_ok {
                format!("Falha de TLS/certificado: {}. Verifique se o certificado de servidor é válido.", e_str)
            } else {
                format!("Falha de conectividade: {}. Nenhum documento fiscal foi enviado.", e_str)
            };

            TestarConectividadeFiscalResp {
                sucesso: false,
                ambiente: "HOMOLOGACAO".into(),
                url: url.to_string(),
                producao_bloqueada: false,
                dns_ok,
                tls_ok,
                mtls_usado: false,
                certificado_configurado,
                http_status: None,
                tempo_ms,
                mensagem,
                warnings: warnings.clone(),
            }
        }
    }
}
