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
// DTOs
// ----------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct VersaoRascunhoReq {
    pub versao: String,
    pub pais_fiscal: String,
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersaoResp {
    pub id: String,
    pub versao: String,
    pub pais_fiscal: String,
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub status: String,
    pub total_registros: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersaoItemResp {
    pub id: String,
    pub versao_id: String,
    pub tipo_dado: String,
    pub registro_id: Option<String>,
    pub operacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditoriaResp {
    pub id: String,
    pub entidade: String,
    pub entidade_id: Option<String>,
    pub acao: String,
    pub usuario_id: Option<String>,
    pub criado_em: String,
}

// ----------------------------------------------------------------
// Handlers
// ----------------------------------------------------------------

/// GET /fiscal/versoes
pub async fn obter_versoes(State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };

    match sqlx::query(
        "SELECT id, versao, pais_fiscal, empresa_id, filial_id, status, total_registros
         FROM fiscal_versoes_publicacao
         ORDER BY criado_em DESC"
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => {
            let resp: Vec<VersaoResp> = rows.iter().map(|r| VersaoResp {
                id: r.get::<Uuid, _>("id").to_string(),
                versao: r.get("versao"),
                pais_fiscal: r.get("pais_fiscal"),
                empresa_id: r.get::<Option<Uuid>, _>("empresa_id").map(|u| u.to_string()),
                filial_id: r.get::<Option<Uuid>, _>("filial_id").map(|u| u.to_string()),
                status: r.get("status"),
                total_registros: r.get("total_registros"),
            }).collect();
            (StatusCode::OK, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

/// POST /fiscal/versoes/rascunho
pub async fn criar_versao_rascunho(State(state): State<AppState>, Json(payload): Json<VersaoRascunhoReq>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };

    let id = Uuid::new_v4();
    let empresa_uuid: Option<Uuid> = payload.empresa_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let filial_uuid: Option<Uuid> = payload.filial_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    match sqlx::query(
        "INSERT INTO fiscal_versoes_publicacao (id, versao, pais_fiscal, empresa_id, filial_id, status, observacao)
         VALUES ($1, $2, $3, $4, $5, 'RASCUNHO', $6)"
    )
    .bind(id)
    .bind(&payload.versao)
    .bind(&payload.pais_fiscal)
    .bind(empresa_uuid)
    .bind(filial_uuid)
    .bind(&payload.observacao)
    .execute(pool)
    .await
    {
        Ok(_) => {
            let resp = VersaoResp {
                id: id.to_string(),
                versao: payload.versao,
                pais_fiscal: payload.pais_fiscal,
                empresa_id: empresa_uuid.map(|u| u.to_string()),
                filial_id: filial_uuid.map(|u| u.to_string()),
                status: "RASCUNHO".to_string(),
                total_registros: Some(0),
            };
            (StatusCode::CREATED, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

/// PUT /fiscal/versoes/:id/cancelar
pub async fn cancelar_versao(Path(id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };
    let uid = match Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response() };

    match sqlx::query(
        "UPDATE fiscal_versoes_publicacao SET status = 'CANCELADA', atualizado_em = now()
         WHERE id = $1 AND status = 'RASCUNHO'"
    )
    .bind(uid)
    .execute(pool)
    .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"ok": true}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

/// GET /fiscal/versoes/:id/itens
pub async fn listar_itens_versao(Path(id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };
    let uid = match Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response() };

    match sqlx::query(
        "SELECT id, versao_id, tipo_dado, registro_id, operacao
         FROM fiscal_versoes_publicacao_itens
         WHERE versao_id = $1"
    )
    .bind(uid)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => {
            let resp: Vec<VersaoItemResp> = rows.iter().map(|r| VersaoItemResp {
                id: r.get::<Uuid, _>("id").to_string(),
                versao_id: r.get::<Uuid, _>("versao_id").to_string(),
                tipo_dado: r.get("tipo_dado"),
                registro_id: r.get::<Option<Uuid>, _>("registro_id").map(|u| u.to_string()),
                operacao: r.get("operacao"),
            }).collect();
            (StatusCode::OK, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

/// GET /fiscal/auditoria
pub async fn obter_auditoria(State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };

    match sqlx::query(
        "SELECT id, entidade, entidade_id, acao, usuario_id, criado_em
         FROM fiscal_auditoria_mestre
         ORDER BY criado_em DESC
         LIMIT 1000"
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => {
            let resp: Vec<AuditoriaResp> = rows.iter().map(|r| AuditoriaResp {
                id: r.get::<Uuid, _>("id").to_string(),
                entidade: r.get("entidade"),
                entidade_id: r.get::<Option<Uuid>, _>("entidade_id").map(|u| u.to_string()),
                acao: r.get("acao"),
                usuario_id: r.get::<Option<Uuid>, _>("usuario_id").map(|u| u.to_string()),
                criado_em: r.get::<chrono::DateTime<chrono::Utc>, _>("criado_em").to_rfc3339(),
            }).collect();
            (StatusCode::OK, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}
