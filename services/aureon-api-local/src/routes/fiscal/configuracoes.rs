use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;
use crate::app::AppState;

// ----------------------------------------------------------------
// DTOs locais
// ----------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FiscalConfigMestreReq {
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub pais_fiscal: String,
    pub regime_fiscal: Option<String>,
    pub ambiente: String,
    pub forma_emissao: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FiscalConfigMestreResp {
    pub id: String,
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub pais_fiscal: String,
    pub regime_fiscal: Option<String>,
    pub ambiente: String,
    pub forma_emissao: String,
    pub ativo: bool,
}

// ----------------------------------------------------------------
// Handlers
// ----------------------------------------------------------------

/// GET /fiscal/configuracoes
pub async fn obter_configuracoes(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem conexão PostgreSQL"}))).into_response(),
    };

    match sqlx::query(
        "SELECT id, empresa_id, filial_id, pais_fiscal, regime_fiscal, ambiente, forma_emissao, ativo
         FROM fiscal_empresas_config
         ORDER BY criado_em DESC"
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => {
            let resp: Vec<FiscalConfigMestreResp> = rows.iter().map(|r| FiscalConfigMestreResp {
                id: r.get::<Uuid, _>("id").to_string(),
                empresa_id: r.get::<Option<Uuid>, _>("empresa_id").map(|u| u.to_string()),
                filial_id: r.get::<Option<Uuid>, _>("filial_id").map(|u| u.to_string()),
                pais_fiscal: r.get("pais_fiscal"),
                regime_fiscal: r.get("regime_fiscal"),
                ambiente: r.get("ambiente"),
                forma_emissao: r.get("forma_emissao"),
                ativo: r.get("ativo"),
            }).collect();
            (StatusCode::OK, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

/// POST /fiscal/configuracoes
pub async fn criar_configuracao(
    State(state): State<AppState>,
    Json(payload): Json<FiscalConfigMestreReq>,
) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem conexão PostgreSQL"}))).into_response(),
    };

    let id = Uuid::new_v4();
    let empresa_uuid: Option<Uuid> = payload.empresa_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let filial_uuid: Option<Uuid> = payload.filial_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    match sqlx::query(
        "INSERT INTO fiscal_empresas_config (id, empresa_id, filial_id, pais_fiscal, regime_fiscal, ambiente, forma_emissao)
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(id)
    .bind(empresa_uuid)
    .bind(filial_uuid)
    .bind(&payload.pais_fiscal)
    .bind(&payload.regime_fiscal)
    .bind(&payload.ambiente)
    .bind(&payload.forma_emissao)
    .execute(pool)
    .await
    {
        Ok(_) => {
            let resp = FiscalConfigMestreResp {
                id: id.to_string(),
                empresa_id: empresa_uuid.map(|u| u.to_string()),
                filial_id: filial_uuid.map(|u| u.to_string()),
                pais_fiscal: payload.pais_fiscal,
                regime_fiscal: payload.regime_fiscal,
                ambiente: payload.ambiente,
                forma_emissao: payload.forma_emissao,
                ativo: true,
            };
            (StatusCode::CREATED, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

/// PUT /fiscal/configuracoes/:id
pub async fn atualizar_configuracao(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<FiscalConfigMestreReq>,
) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem conexão PostgreSQL"}))).into_response(),
    };

    let config_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response(),
    };

    let empresa_uuid: Option<Uuid> = payload.empresa_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let filial_uuid: Option<Uuid> = payload.filial_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    match sqlx::query(
        "UPDATE fiscal_empresas_config
         SET empresa_id = $2, filial_id = $3, pais_fiscal = $4, regime_fiscal = $5, ambiente = $6, forma_emissao = $7, atualizado_em = now()
         WHERE id = $1"
    )
    .bind(config_id)
    .bind(empresa_uuid)
    .bind(filial_uuid)
    .bind(&payload.pais_fiscal)
    .bind(&payload.regime_fiscal)
    .bind(&payload.ambiente)
    .bind(&payload.forma_emissao)
    .execute(pool)
    .await
    {
        Ok(_) => {
            let resp = FiscalConfigMestreResp {
                id: id.clone(),
                empresa_id: empresa_uuid.map(|u| u.to_string()),
                filial_id: filial_uuid.map(|u| u.to_string()),
                pais_fiscal: payload.pais_fiscal,
                regime_fiscal: payload.regime_fiscal,
                ambiente: payload.ambiente,
                forma_emissao: payload.forma_emissao,
                ativo: true,
            };
            (StatusCode::OK, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}
