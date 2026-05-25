use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::fs;
use sqlx::Row;
use crate::app::AppState;
use chrono::{NaiveDateTime, DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct ValidarCertificadoFiscalReq {
    pub caminho_pfx: Option<String>,
    pub senha_pfx: String,
    pub conteudo_base64: Option<String>,
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CertificadoFiscalStatusResp {
    pub valido: bool,
    pub cn: Option<String>,
    pub cnpj_titular: Option<String>,
    pub numero_serie: Option<String>,
    pub validade_inicio: Option<String>,
    pub validade_fim: Option<String>,
    pub dias_para_expirar: Option<i64>,
    pub expirado: bool,
    pub alerta_expira_30_dias: bool,
    pub mensagem: String,
}

#[cfg(feature = "fiscal_real")]
pub async fn validar_certificado(
    State(state): State<AppState>,
    Json(payload): Json<ValidarCertificadoFiscalReq>,
) -> Result<Json<CertificadoFiscalStatusResp>, (axum::http::StatusCode, String)> {
    
    use openssl::pkcs12::Pkcs12;

    let pfx_bytes = if let Some(ref base64_str) = payload.conteudo_base64 {
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.decode(base64_str).map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("Erro ao decodificar base64: {}", e)))?
    } else if let Some(ref path_str) = payload.caminho_pfx {
        fs::read(path_str).map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("Erro ao ler arquivo PFX: {}", e)))?
    } else {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Caminho PFX ou conteúdo base64 devem ser informados".to_string()));
    };

    let pkcs12 = Pkcs12::from_der(&pfx_bytes).map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("PFX inválido: {}", e)))?;
    let parsed = pkcs12.parse(&payload.senha_pfx).map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("Senha incorreta ou PFX inválido: {}", e)))?;
    let cert = parsed.cert;

    let not_before = cert.not_before().to_string();
    let not_after = cert.not_after().to_string();

    let mut subject_cn = None;
    for entry in cert.subject_name().entries() {
        if entry.object().nid() == openssl::nid::Nid::COMMONNAME {
            if let Ok(s) = entry.data().as_utf8() {
                subject_cn = Some(s.to_string());
                break;
            }
        }
    }

    let cnpj_titular = if let Some(ref cn_str) = subject_cn {
        if let Some(idx) = cn_str.rfind(':') {
            let possible_cnpj = &cn_str[idx+1..];
            if possible_cnpj.len() == 14 && possible_cnpj.chars().all(char::is_numeric) {
                Some(possible_cnpj.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let serial_number = cert.serial_number().to_bn().map(|bn| bn.to_hex_str().unwrap().to_string().to_ascii_uppercase()).unwrap_or_default();

    let parse_asn1_time = |time_str: &str| -> Option<DateTime<Utc>> {
        NaiveDateTime::parse_from_str(time_str, "%b %e %H:%M:%S %Y %Z")
            .or_else(|_| NaiveDateTime::parse_from_str(time_str, "%b %d %H:%M:%S %Y %Z"))
            .ok()
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    };

    let validade_inicio_dt = parse_asn1_time(&not_before);
    let validade_fim_dt = parse_asn1_time(&not_after);

    let mut expirado = false;
    let mut alerta_expira_30_dias = false;
    let mut dias_para_expirar = None;

    if let Some(fim) = validade_fim_dt {
        let now = Utc::now();
        if now > fim {
            expirado = true;
        } else {
            let diff = fim.signed_duration_since(now).num_days();
            dias_para_expirar = Some(diff);
            if diff <= 30 {
                alerta_expira_30_dias = true;
            }
        }
    }

    if let Some(pool) = &state.pool {
        let (empresa_id_query, filial_id_query) = if let (Some(e), Some(f)) = (&payload.empresa_id, &payload.filial_id) {
            (format!("empresa_id = '{}'", e), format!("filial_id = '{}'", f))
        } else {
            ("empresa_id IS NULL".to_string(), "filial_id IS NULL".to_string())
        };

        let existe: bool = sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM fiscal_empresas_config WHERE {} AND {})",
            empresa_id_query, filial_id_query
        ))
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if existe {
            let validade_inicio_db = validade_inicio_dt.map(|d| d.naive_utc());
            let validade_fim_db = validade_fim_dt.map(|d| d.naive_utc());

            let _ = sqlx::query(&format!(
                "UPDATE fiscal_empresas_config SET 
                 certificado_cn = $1, 
                 certificado_cnpj = $2, 
                 certificado_numero_serie = $3, 
                 certificado_validade_inicio = $4, 
                 certificado_validade_fim = $5,
                 certificado_caminho = $6,
                 certificado_atualizado_em = CURRENT_TIMESTAMP
                 WHERE {} AND {}",
                 empresa_id_query, filial_id_query
            ))
            .bind(&subject_cn)
            .bind(&cnpj_titular)
            .bind(&serial_number)
            .bind(validade_inicio_db)
            .bind(validade_fim_db)
            .bind(&payload.caminho_pfx)
            .execute(pool)
            .await;
        }
    }

    Ok(Json(CertificadoFiscalStatusResp {
        valido: !expirado,
        cn: subject_cn,
        cnpj_titular,
        numero_serie: Some(serial_number),
        validade_inicio: Some(not_before),
        validade_fim: Some(not_after),
        dias_para_expirar,
        expirado,
        alerta_expira_30_dias,
        mensagem: if expirado { "Certificado expirado.".to_string() } else if alerta_expira_30_dias { "Certificado expira em menos de 30 dias.".to_string() } else { "Certificado válido e extraído com sucesso.".to_string() }
    }))
}

#[cfg(not(feature = "fiscal_real"))]
pub async fn validar_certificado(
    State(state): State<AppState>,
    Json(payload): Json<ValidarCertificadoFiscalReq>,
) -> Result<Json<CertificadoFiscalStatusResp>, (axum::http::StatusCode, String)> {
    
    let subject_cn = Some("MOCK EMPRESA LTDA:12345678000199".to_string());
    let cnpj_titular = Some("12345678000199".to_string());
    let serial_number = "A1B2C3D4E5F6".to_string();
    
    let now = Utc::now();
    let fim_dt = now + chrono::Duration::days(365);
    let inicio_dt = now - chrono::Duration::days(1);
    
    let not_before = inicio_dt.format("%b %e %H:%M:%S %Y GMT").to_string();
    let not_after = fim_dt.format("%b %e %H:%M:%S %Y GMT").to_string();

    let dias_para_expirar = Some(365);
    let expirado = false;
    let alerta_expira_30_dias = false;

    if let Some(pool) = &state.pool {
        let (empresa_id_query, filial_id_query) = if let (Some(e), Some(f)) = (&payload.empresa_id, &payload.filial_id) {
            (format!("empresa_id = '{}'", e), format!("filial_id = '{}'", f))
        } else {
            ("empresa_id IS NULL".to_string(), "filial_id IS NULL".to_string())
        };

        let existe: bool = sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM fiscal_empresas_config WHERE {} AND {})",
            empresa_id_query, filial_id_query
        ))
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if existe {
            let _ = sqlx::query(&format!(
                "UPDATE fiscal_empresas_config SET 
                 certificado_cn = $1, 
                 certificado_cnpj = $2, 
                 certificado_numero_serie = $3, 
                 certificado_validade_inicio = $4, 
                 certificado_validade_fim = $5,
                 certificado_caminho = $6,
                 certificado_atualizado_em = CURRENT_TIMESTAMP
                 WHERE {} AND {}",
                 empresa_id_query, filial_id_query
            ))
            .bind(&subject_cn)
            .bind(&cnpj_titular)
            .bind(&serial_number)
            .bind(inicio_dt.naive_utc())
            .bind(fim_dt.naive_utc())
            .bind(&payload.caminho_pfx)
            .execute(pool)
            .await;
        }
    }

    Ok(Json(CertificadoFiscalStatusResp {
        valido: !expirado,
        cn: subject_cn,
        cnpj_titular,
        numero_serie: Some(serial_number),
        validade_inicio: Some(not_before),
        validade_fim: Some(not_after),
        dias_para_expirar,
        expirado,
        alerta_expira_30_dias,
        mensagem: "[DEV/MOCK] Certificado extraído via mock. Ative a feature 'fiscal_real' para validação com OpenSSL.".to_string()
    }))
}

pub async fn status_certificado(
    State(state): State<AppState>,
) -> Result<Json<CertificadoFiscalStatusResp>, (axum::http::StatusCode, String)> {
    let pool = state.pool.as_ref().ok_or((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Sem banco de dados".to_string()))?;

    let row = sqlx::query(
        "SELECT certificado_cn, certificado_cnpj, certificado_numero_serie, certificado_validade_inicio, certificado_validade_fim 
         FROM fiscal_empresas_config LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Erro DB: {}", e)))?;

    if let Some(r) = row {
        let cn: Option<String> = r.try_get("certificado_cn").unwrap_or(None);
        let cnpj: Option<String> = r.try_get("certificado_cnpj").unwrap_or(None);
        let serie: Option<String> = r.try_get("certificado_numero_serie").unwrap_or(None);
        let inicio: Option<chrono::NaiveDateTime> = r.try_get("certificado_validade_inicio").unwrap_or(None);
        let fim: Option<chrono::NaiveDateTime> = r.try_get("certificado_validade_fim").unwrap_or(None);

        if cn.is_none() || fim.is_none() {
            return Ok(Json(CertificadoFiscalStatusResp {
                valido: false,
                cn: None,
                cnpj_titular: None,
                numero_serie: None,
                validade_inicio: None,
                validade_fim: None,
                dias_para_expirar: None,
                expirado: false,
                alerta_expira_30_dias: false,
                mensagem: "Nenhum certificado configurado ou metadados incompletos.".to_string()
            }));
        }

        let fim_dt = DateTime::<Utc>::from_naive_utc_and_offset(fim.unwrap(), Utc);
        let now = Utc::now();
        let mut expirado = false;
        let mut alerta_expira_30_dias = false;
        let mut dias_para_expirar = None;

        if now > fim_dt {
            expirado = true;
        } else {
            let diff = fim_dt.signed_duration_since(now).num_days();
            dias_para_expirar = Some(diff);
            if diff <= 30 {
                alerta_expira_30_dias = true;
            }
        }

        let mut msg = if expirado { "Certificado expirado.".to_string() } else if alerta_expira_30_dias { "Certificado expira em menos de 30 dias.".to_string() } else { "Certificado configurado e válido.".to_string() };
        
        #[cfg(not(feature = "fiscal_real"))]
        {
            msg = format!("[MOCK] {}", msg);
        }

        Ok(Json(CertificadoFiscalStatusResp {
            valido: !expirado,
            cn,
            cnpj_titular: cnpj,
            numero_serie: serie,
            validade_inicio: inicio.map(|d| d.to_string()),
            validade_fim: fim.map(|d| d.to_string()),
            dias_para_expirar,
            expirado,
            alerta_expira_30_dias,
            mensagem: msg
        }))
    } else {
        Ok(Json(CertificadoFiscalStatusResp {
            valido: false,
            cn: None,
            cnpj_titular: None,
            numero_serie: None,
            validade_inicio: None,
            validade_fim: None,
            dias_para_expirar: None,
            expirado: false,
            alerta_expira_30_dias: false,
            mensagem: "Configuração fiscal não encontrada.".to_string()
        }))
    }
}
