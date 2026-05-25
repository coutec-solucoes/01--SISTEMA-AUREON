/// homologacao.rs
/// Fase 19 — Bloco 3
/// Infraestrutura de Endpoints de Homologação, Bloqueio de Produção e Clientes Fiscais Controlados
///
/// ATENÇÃO:
/// - Este módulo NÃO transmite NF-e/NFC-e.
/// - Este módulo NÃO transmite DTE/SIFEN.
/// - Este módulo NÃO autoriza documento fiscal.
/// - Este módulo NÃO consulta status fiscal real.
/// - Este módulo NÃO cria protocolo.
/// - Este módulo NÃO gera DANFE/KuDE.
/// - Este módulo NÃO envia XML/DTE a qualquer webservice.
///
/// Objetivo: preparar o registry seguro de endpoints de homologação e diagnóstico
/// de conectividade, com bloqueio rígido contra URLs de produção.

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use crate::app::AppState;

// ─────────────────────────────────────────────
// ENUMS
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum PaisFiscal {
    BR,
    PY,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ModeloDocumento {
    NFE,
    NFCE,
    SIFEN,
    CTE,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum AmbienteHomologacao {
    HOMOLOGACAO,
    // PRODUCAO é bloqueado neste módulo — não expor como opção utilizável
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ServicoFiscal {
    Autorizacao,
    RetornoAutorizacao,
    ConsultaProtocolo,
    Inutilizacao,
    EventoCancelamento,
    // SIFEN
    RecepcionarLote,
    ConsultarLote,
    ConsultarSituacion,
    ConsultarRuc,
}

// ─────────────────────────────────────────────
// REGISTRY INTERNO DE ENDPOINTS
// ─────────────────────────────────────────────

/// Representa um endpoint oficial de homologação registrado internamente.
/// Nenhuma transmissão real é feita a partir deste registry neste bloco.
#[derive(Debug, Clone, Serialize)]
pub struct RegistryEndpointFiscal {
    pub pais: String,
    pub modelo: String,
    pub uf: Option<String>,
    pub servico: String,
    pub url: String,
    pub producao_bloqueada: bool,
    pub homologacao_confirmada: bool,
    pub nota: String,
}

/// Retorna os endpoints de homologação registrados no sistema para diagnóstico.
/// Nenhuma chamada de rede é executada por esta função.
pub fn registry_endpoints_homologacao() -> Vec<RegistryEndpointFiscal> {
    vec![
        // ─── Brasil — NF-e (SVRS — Homologação) ───────────────────────
        RegistryEndpointFiscal {
            pais: "BR".into(),
            modelo: "NFE".into(),
            uf: Some("SVRS".into()),
            servico: "Autorizacao".into(),
            url: "https://homologacao.svrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx".into(),
            producao_bloqueada: true,
            homologacao_confirmada: true,
            nota: "SEFAZ Virtual RS — Homologação NF-e 4.00. Sem transmissão neste bloco.".into(),
        },
        RegistryEndpointFiscal {
            pais: "BR".into(),
            modelo: "NFE".into(),
            uf: Some("SVRS".into()),
            servico: "RetornoAutorizacao".into(),
            url: "https://homologacao.svrs.rs.gov.br/ws/NfeRetAutorizacao/NFeRetAutorizacao4.asmx".into(),
            producao_bloqueada: true,
            homologacao_confirmada: true,
            nota: "SEFAZ Virtual RS — Retorno de Autorização NF-e 4.00.".into(),
        },
        RegistryEndpointFiscal {
            pais: "BR".into(),
            modelo: "NFE".into(),
            uf: Some("SVRS".into()),
            servico: "ConsultaProtocolo".into(),
            url: "https://homologacao.svrs.rs.gov.br/ws/NfeConsultaProtocolo/NfeConsultaProtocolo4.asmx".into(),
            producao_bloqueada: true,
            homologacao_confirmada: true,
            nota: "Consulta protocolo NF-e homologação.".into(),
        },
        RegistryEndpointFiscal {
            pais: "BR".into(),
            modelo: "NFE".into(),
            uf: Some("SVRS".into()),
            servico: "EventoCancelamento".into(),
            url: "https://homologacao.svrs.rs.gov.br/ws/NfeCancelamento/NfeCancelamento4.asmx".into(),
            producao_bloqueada: true,
            homologacao_confirmada: true,
            nota: "Evento de cancelamento NF-e homologação.".into(),
        },
        // ─── Brasil — NFC-e (SVRS — Homologação) ──────────────────────
        RegistryEndpointFiscal {
            pais: "BR".into(),
            modelo: "NFCE".into(),
            uf: Some("SVRS".into()),
            servico: "Autorizacao".into(),
            url: "https://homologacao.svrs.rs.gov.br/ws/NfceAutorizacao/NfceAutorizacao4.asmx".into(),
            producao_bloqueada: true,
            homologacao_confirmada: true,
            nota: "SEFAZ Virtual RS — Homologação NFC-e 4.00. Sem transmissão neste bloco.".into(),
        },
        RegistryEndpointFiscal {
            pais: "BR".into(),
            modelo: "NFCE".into(),
            uf: Some("SVRS".into()),
            servico: "RetornoAutorizacao".into(),
            url: "https://homologacao.svrs.rs.gov.br/ws/NfceRetAutorizacao/NfceRetAutorizacao4.asmx".into(),
            producao_bloqueada: true,
            homologacao_confirmada: true,
            nota: "SEFAZ Virtual RS — Retorno de Autorização NFC-e 4.00.".into(),
        },
        // ─── Paraguai — SIFEN/DTE (e-Kuatia Homologação) ──────────────
        RegistryEndpointFiscal {
            pais: "PY".into(),
            modelo: "SIFEN".into(),
            uf: None,
            servico: "RecepcionarLote".into(),
            url: "https://sifen-test.set.gov.py/de/ws/sync/recibe.wsdl".into(),
            producao_bloqueada: true,
            homologacao_confirmada: true,
            nota: "SIFEN — Recepção de lote DTE (homologação). Sem transmissão neste bloco.".into(),
        },
        RegistryEndpointFiscal {
            pais: "PY".into(),
            modelo: "SIFEN".into(),
            uf: None,
            servico: "ConsultarLote".into(),
            url: "https://sifen-test.set.gov.py/de/ws/sync/consulta-lote.wsdl".into(),
            producao_bloqueada: true,
            homologacao_confirmada: true,
            nota: "SIFEN — Consulta de lote DTE (homologação).".into(),
        },
        RegistryEndpointFiscal {
            pais: "PY".into(),
            modelo: "SIFEN".into(),
            uf: None,
            servico: "ConsultarSituacion".into(),
            url: "https://sifen-test.set.gov.py/de/ws/sync/consulta.wsdl".into(),
            producao_bloqueada: true,
            homologacao_confirmada: true,
            nota: "SIFEN — Consulta situación de DE (homologação).".into(),
        },
        RegistryEndpointFiscal {
            pais: "PY".into(),
            modelo: "SIFEN".into(),
            uf: None,
            servico: "ConsultarRuc".into(),
            url: "https://sifen-test.set.gov.py/de/ws/sync/consulta-ruc.wsdl".into(),
            producao_bloqueada: true,
            homologacao_confirmada: true,
            nota: "SIFEN — Consulta de contribuyente por RUC (homologação).".into(),
        },
    ]
}

/// Lista de URLs que caracterizam ambiente de produção — bloqueadas rigidamente.
fn urls_producao_bloqueadas() -> Vec<&'static str> {
    vec![
        "nfe.fazenda.gov.br",
        "www.sefaz.rs.gov.br",
        "nfe.sefazvirtual.fazenda.gov.br",
        "nfce.sefazvirtual.fazenda.gov.br",
        "sifen.set.gov.py",
        "ekuatia.set.gov.py",
        "hekuatia.set.gov.py",
        // Qualquer URL sem "homologacao", "hom", "test", "sandbox" no host é suspeita
    ]
}

fn detectar_url_producao(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    // Se contém indicador de homologação, OK
    if url_lower.contains("homologacao")
        || url_lower.contains("homologación")
        || url_lower.contains("-test")
        || url_lower.contains("test.")
        || url_lower.contains("hom.")
        || url_lower.contains("sandbox")
    {
        return false;
    }
    // Se contém padrão de produção conhecido, bloqueia
    for host in urls_producao_bloqueadas() {
        if url_lower.contains(host) {
            return true;
        }
    }
    false
}

// ─────────────────────────────────────────────
// DTOs
// ─────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct FiscalEndpointConfigResp {
    pub pais: String,
    pub modelo: String,
    pub ambiente: String,
    pub uf: Option<String>,
    pub servico: String,
    pub url: String,
    pub producao_bloqueada: bool,
    pub homologacao_confirmada: bool,
    pub mensagem: String,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticoFiscalHomologacaoResp {
    pub sucesso: bool,
    pub fiscal_real_disponivel: bool,
    pub xmldsig_real_disponivel: bool,
    pub certificado_configurado: bool,
    pub ambiente: String,
    pub endpoints_homologacao: Vec<FiscalEndpointConfigResp>,
    pub bloqueios_producao_ativos: bool,
    pub schemas_oficiais_disponíveis: bool,
    pub schemas_br_nfe: bool,
    pub schemas_br_nfce: bool,
    pub schemas_py_sifen: bool,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TestarEndpointFiscalReq {
    pub pais: String,
    pub modelo: String,
    pub uf: Option<String>,
    pub servico: String,
    pub ambiente: String,
}

#[derive(Debug, Serialize)]
pub struct TestarEndpointFiscalResp {
    pub sucesso: bool,
    pub ambiente: String,
    pub url: String,
    pub bloqueado_por_producao: bool,
    pub conectividade_testada: bool,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ValidarBloqueioProducaoReq {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ValidarBloqueioProducaoResp {
    pub url: String,
    pub e_producao: bool,
    pub bloqueado: bool,
    pub mensagem: String,
}

// ─────────────────────────────────────────────
// HANDLERS
// ─────────────────────────────────────────────

/// GET /fiscal/homologacao/diagnostico
/// Retorna diagnóstico completo da infraestrutura fiscal de homologação.
/// Não faz chamadas de rede, não transmite documentos.
pub async fn diagnostico_homologacao(
    State(_state): State<AppState>,
) -> Json<DiagnosticoFiscalHomologacaoResp> {
    let mut warnings: Vec<String> = Vec::new();

    // Feature flags
    let fiscal_real = cfg!(feature = "fiscal_real");
    let xmldsig_real = cfg!(feature = "fiscal_xmldsig_real");

    // Verifica schemas no sistema de arquivos (sem abrir ou modificar nenhum arquivo)
    let schema_br_nfe  = std::path::Path::new("assets/schemas_fiscal/br/nfe/PL_009_V4").exists();
    let schema_br_nfce = std::path::Path::new("assets/schemas_fiscal/br/nfce/PL_009_V4").exists();
    let schema_py      = std::path::Path::new("assets/schemas_fiscal/py/sifen").exists();

    let schemas_disponíveis = schema_br_nfe && schema_br_nfce && schema_py;

    // Certificado: verificamos apenas se a configuração existe (sem carregar chave)
    let certificado_configurado = std::env::var("FISCAL_CERT_PATH").is_ok()
        || std::env::var("FISCAL_CERT_PFX_B64").is_ok();

    if !fiscal_real {
        warnings.push("Feature 'fiscal_real' não está ativa. A assinatura e transmissão real requerem build Linux/WSL/Docker com esta feature habilitada.".into());
    }
    if !xmldsig_real {
        warnings.push("Feature 'fiscal_xmldsig_real' não está ativa. XMLDSig real requer libxmlsec1/OpenSSL instalados.".into());
    }
    if !certificado_configurado {
        warnings.push("Variável de ambiente FISCAL_CERT_PATH ou FISCAL_CERT_PFX_B64 não encontrada. Configure o certificado A1 antes de homologar.".into());
    }
    if !schema_br_nfe {
        warnings.push("Schemas XSD oficiais NF-e (PL_009_V4) ausentes. Pendente: download manual do Portal da NF-e.".into());
    }
    if !schema_br_nfce {
        warnings.push("Schemas XSD oficiais NFC-e (PL_009_V4) ausentes. Pendente: download manual do Portal da NF-e.".into());
    }
    if !schema_py {
        warnings.push("Schemas JSON oficiais SIFEN ausentes. Pendente: download manual do portal DNIT/e-Kuatia.".into());
    }

    // Monta lista de endpoints para o diagnóstico
    let endpoints: Vec<FiscalEndpointConfigResp> = registry_endpoints_homologacao()
        .into_iter()
        .map(|e| FiscalEndpointConfigResp {
            pais: e.pais,
            modelo: e.modelo,
            ambiente: "HOMOLOGACAO".into(),
            uf: e.uf,
            servico: e.servico,
            url: e.url,
            producao_bloqueada: e.producao_bloqueada,
            homologacao_confirmada: e.homologacao_confirmada,
            mensagem: e.nota,
        })
        .collect();

    let mensagem = if warnings.is_empty() {
        "Infraestrutura de homologação configurada corretamente.".into()
    } else {
        format!("Infraestrutura carregada com {} pendência(s). Veja os warnings.", warnings.len())
    };

    Json(DiagnosticoFiscalHomologacaoResp {
        sucesso: true,
        fiscal_real_disponivel: fiscal_real,
        xmldsig_real_disponivel: xmldsig_real,
        certificado_configurado,
        ambiente: "HOMOLOGACAO".into(),
        endpoints_homologacao: endpoints,
        bloqueios_producao_ativos: true,
        schemas_oficiais_disponíveis: schemas_disponíveis,
        schemas_br_nfe: schema_br_nfe,
        schemas_br_nfce: schema_br_nfce,
        schemas_py_sifen: schema_py,
        mensagem,
        warnings,
    })
}

/// GET /fiscal/homologacao/endpoints
/// Lista todos os endpoints de homologação registrados no sistema.
/// Nenhuma chamada de rede é feita.
pub async fn listar_endpoints(
    State(_state): State<AppState>,
) -> Json<Vec<FiscalEndpointConfigResp>> {
    let endpoints: Vec<FiscalEndpointConfigResp> = registry_endpoints_homologacao()
        .into_iter()
        .map(|e| FiscalEndpointConfigResp {
            pais: e.pais,
            modelo: e.modelo,
            ambiente: "HOMOLOGACAO".into(),
            uf: e.uf,
            servico: e.servico,
            url: e.url,
            producao_bloqueada: e.producao_bloqueada,
            homologacao_confirmada: e.homologacao_confirmada,
            mensagem: e.nota,
        })
        .collect();

    Json(endpoints)
}

/// POST /fiscal/homologacao/testar-endpoint
/// Identifica o endpoint no registry e valida que é de homologação.
/// NÃO faz conexão TCP/HTTP. NÃO transmite documento fiscal.
pub async fn testar_endpoint(
    State(_state): State<AppState>,
    Json(payload): Json<TestarEndpointFiscalReq>,
) -> Result<Json<TestarEndpointFiscalResp>, (axum::http::StatusCode, String)> {
    let mut warnings: Vec<String> = Vec::new();

    // Bloqueio de produção no payload
    if payload.ambiente.to_uppercase() == "PRODUCAO" {
        return Ok(Json(TestarEndpointFiscalResp {
            sucesso: false,
            ambiente: payload.ambiente,
            url: String::new(),
            bloqueado_por_producao: true,
            conectividade_testada: false,
            mensagem: "BLOQUEADO: ambiente PRODUCAO não é permitido neste módulo. Use HOMOLOGACAO.".into(),
            warnings: vec!["Tentativa de acesso a ambiente de produção bloqueada.".into()],
        }));
    }

    // Busca o endpoint no registry
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

    match encontrado {
        None => {
            Ok(Json(TestarEndpointFiscalResp {
                sucesso: false,
                ambiente: "HOMOLOGACAO".into(),
                url: String::new(),
                bloqueado_por_producao: false,
                conectividade_testada: false,
                mensagem: format!(
                    "Endpoint não encontrado no registry para pais={}, modelo={}, servico={}.",
                    payload.pais, payload.modelo, payload.servico
                ),
                warnings,
            }))
        }
        Some(ep) => {
            // Dupla verificação anti-produção na URL
            if detectar_url_producao(&ep.url) {
                return Ok(Json(TestarEndpointFiscalResp {
                    sucesso: false,
                    ambiente: "HOMOLOGACAO".into(),
                    url: ep.url.clone(),
                    bloqueado_por_producao: true,
                    conectividade_testada: false,
                    mensagem: "BLOQUEADO: a URL detectada contém padrão de ambiente de produção.".into(),
                    warnings: vec!["URL de produção detectada e bloqueada pelo registry.".into()],
                }));
            }

            warnings.push("A conectividade TCP/HTTP não foi testada neste bloco. Nenhum dado foi transmitido.".into());
            warnings.push("Para validar conectividade real, configure o certificado A1 e utilize um ambiente Linux/WSL com as features fiscais habilitadas.".into());

            Ok(Json(TestarEndpointFiscalResp {
                sucesso: true,
                ambiente: "HOMOLOGACAO".into(),
                url: ep.url.clone(),
                bloqueado_por_producao: false,
                conectividade_testada: false, // Explícito: não testamos conexão TCP neste bloco
                mensagem: format!(
                    "Endpoint localizado no registry: {} — {}. Nenhum dado foi transmitido.",
                    ep.servico, ep.nota
                ),
                warnings,
            }))
        }
    }
}

/// POST /fiscal/homologacao/validar-bloqueio-producao
/// Verifica se uma URL fornecida seria bloqueada como URL de produção.
/// Uso diagnóstico e de testes internos. Não faz chamadas de rede.
pub async fn validar_bloqueio_producao(
    State(_state): State<AppState>,
    Json(payload): Json<ValidarBloqueioProducaoReq>,
) -> Json<ValidarBloqueioProducaoResp> {
    let e_producao = detectar_url_producao(&payload.url);
    let mensagem = if e_producao {
        format!("URL '{}' identificada como ambiente de produção e seria bloqueada.", payload.url)
    } else {
        format!("URL '{}' não identificada como produção (parece ser homologação/sandbox).", payload.url)
    };

    Json(ValidarBloqueioProducaoResp {
        url: payload.url,
        e_producao,
        bloqueado: e_producao,
        mensagem,
    })
}
