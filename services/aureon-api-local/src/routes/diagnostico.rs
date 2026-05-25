use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::PgPool;

/// GET /diagnostico/basico
/// Retorna status da API, PostgreSQL e informações não sensíveis.
/// Funciona mesmo sem PostgreSQL (retorna status "indisponivel").
pub async fn handler_diagnostico(pool: Option<PgPool>) -> Json<Value> {
    let horario = Utc::now().to_rfc3339();

    let (pg_status, pg_mensagem) = verificar_postgres(pool.as_ref()).await;

    Json(json!({
        "sucesso":  true,
        "mensagem": "Diagnóstico concluído",
        "dados": {
            "api": {
                "status":   "ok",
                "versao":   "0.0.1",
                "ambiente": std::env::var("AUREON_AMBIENTE")
                               .unwrap_or_else(|_| "desenvolvimento".to_string()),
                "horario":  horario
            },
            "fiscal": {
                "fiscal_real": cfg!(feature = "fiscal_real"),
                "fiscal_xmldsig_real": cfg!(feature = "fiscal_xmldsig_real"),
                "backend_assinatura": if cfg!(feature = "fiscal_xmldsig_real") { "xmlsec" } else if cfg!(feature = "fiscal_real") { "openssl_tecnico" } else { "mock" }
            },
            "postgresql": {
                "status":   pg_status,
                "mensagem": pg_mensagem
            }
        },
        "erro": null
    }))
}

async fn verificar_postgres(pool: Option<&PgPool>) -> (&'static str, &'static str) {
    match pool {
        None => ("indisponivel", "DATABASE_URL não configurada ou conexão não estabelecida"),
        Some(p) => {
            match sqlx::query("SELECT 1").execute(p).await {
                Ok(_)  => ("ok", "PostgreSQL respondendo normalmente"),
                Err(_) => ("erro", "PostgreSQL configurado mas não respondendo"),
            }
        }
    }
}
