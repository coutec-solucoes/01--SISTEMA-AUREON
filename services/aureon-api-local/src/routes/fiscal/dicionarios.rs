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
// DTOs locais — Dicionários Fiscais da Retaguarda
// ----------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct NcmReq { pub codigo: String, pub descricao: String }
#[derive(Debug, Serialize, Deserialize)]
pub struct NcmResp { pub id: String, pub codigo: String, pub descricao: String, pub ativo: bool }

#[derive(Debug, Serialize, Deserialize)]
pub struct CfopReq { pub codigo: String, pub descricao: String, pub tipo_operacao: Option<String> }
#[derive(Debug, Serialize, Deserialize)]
pub struct CfopResp { pub id: String, pub codigo: String, pub descricao: String, pub tipo_operacao: Option<String>, pub ativo: bool }

#[derive(Debug, Serialize, Deserialize)]
pub struct CstCsosnReq { pub codigo: String, pub tipo: String, pub descricao: String }
#[derive(Debug, Serialize, Deserialize)]
pub struct CstCsosnResp { pub id: String, pub codigo: String, pub tipo: String, pub descricao: String, pub ativo: bool }

#[derive(Debug, Serialize, Deserialize)]
pub struct IvaReq { pub codigo: String, pub descricao: String, pub pais_fiscal: String, pub aliquota_escala6: i64 }
#[derive(Debug, Serialize, Deserialize)]
pub struct IvaResp { pub id: String, pub codigo: String, pub descricao: String, pub pais_fiscal: String, pub aliquota_escala6: i64, pub ativo: bool }

// ----------------------------------------------------------------
// NCM
// ----------------------------------------------------------------

pub async fn obter_ncms(State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    match sqlx::query("SELECT id, codigo, descricao, ativo FROM fiscal_dicionario_ncm WHERE ativo = true ORDER BY codigo ASC LIMIT 1000")
        .fetch_all(pool).await
    {
        Ok(rows) => {
            let resp: Vec<NcmResp> = rows.iter().map(|r| NcmResp { id: r.get::<Uuid, _>("id").to_string(), codigo: r.get("codigo"), descricao: r.get("descricao"), ativo: r.get("ativo") }).collect();
            (StatusCode::OK, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

pub async fn criar_ncm(State(state): State<AppState>, Json(payload): Json<NcmReq>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    let id = Uuid::new_v4();
    match sqlx::query("INSERT INTO fiscal_dicionario_ncm (id, codigo, descricao) VALUES ($1, $2, $3)")
        .bind(id).bind(&payload.codigo).bind(&payload.descricao).execute(pool).await
    {
        Ok(_) => (StatusCode::CREATED, Json(json!(NcmResp { id: id.to_string(), codigo: payload.codigo, descricao: payload.descricao, ativo: true }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

pub async fn atualizar_ncm(Path(id): Path<String>, State(state): State<AppState>, Json(payload): Json<NcmReq>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    let uid = match Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response() };
    match sqlx::query("UPDATE fiscal_dicionario_ncm SET codigo = $2, descricao = $3, atualizado_em = now() WHERE id = $1")
        .bind(uid).bind(&payload.codigo).bind(&payload.descricao).execute(pool).await
    {
        Ok(_) => (StatusCode::OK, Json(json!(NcmResp { id, codigo: payload.codigo, descricao: payload.descricao, ativo: true }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

pub async fn inativar_ncm(Path(id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    let uid = match Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response() };
    match sqlx::query("UPDATE fiscal_dicionario_ncm SET ativo = false, atualizado_em = now() WHERE id = $1").bind(uid).execute(pool).await {
        Ok(_) => (StatusCode::OK, Json(json!({"ok": true}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

// ----------------------------------------------------------------
// CFOP
// ----------------------------------------------------------------

pub async fn obter_cfops(State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    match sqlx::query("SELECT id, codigo, descricao, tipo_operacao, ativo FROM fiscal_dicionario_cfop WHERE ativo = true ORDER BY codigo ASC LIMIT 1000")
        .fetch_all(pool).await
    {
        Ok(rows) => {
            let resp: Vec<CfopResp> = rows.iter().map(|r| CfopResp { id: r.get::<Uuid, _>("id").to_string(), codigo: r.get("codigo"), descricao: r.get("descricao"), tipo_operacao: r.get("tipo_operacao"), ativo: r.get("ativo") }).collect();
            (StatusCode::OK, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

pub async fn criar_cfop(State(state): State<AppState>, Json(payload): Json<CfopReq>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    let id = Uuid::new_v4();
    match sqlx::query("INSERT INTO fiscal_dicionario_cfop (id, codigo, descricao, tipo_operacao) VALUES ($1, $2, $3, $4)")
        .bind(id).bind(&payload.codigo).bind(&payload.descricao).bind(&payload.tipo_operacao).execute(pool).await
    {
        Ok(_) => (StatusCode::CREATED, Json(json!(CfopResp { id: id.to_string(), codigo: payload.codigo, descricao: payload.descricao, tipo_operacao: payload.tipo_operacao, ativo: true }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

pub async fn atualizar_cfop(Path(id): Path<String>, State(state): State<AppState>, Json(payload): Json<CfopReq>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    let uid = match Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response() };
    match sqlx::query("UPDATE fiscal_dicionario_cfop SET codigo = $2, descricao = $3, tipo_operacao = $4, atualizado_em = now() WHERE id = $1")
        .bind(uid).bind(&payload.codigo).bind(&payload.descricao).bind(&payload.tipo_operacao).execute(pool).await
    {
        Ok(_) => (StatusCode::OK, Json(json!(CfopResp { id, codigo: payload.codigo, descricao: payload.descricao, tipo_operacao: payload.tipo_operacao, ativo: true }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

pub async fn inativar_cfop(Path(id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    let uid = match Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response() };
    match sqlx::query("UPDATE fiscal_dicionario_cfop SET ativo = false, atualizado_em = now() WHERE id = $1").bind(uid).execute(pool).await {
        Ok(_) => (StatusCode::OK, Json(json!({"ok": true}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

// ----------------------------------------------------------------
// CST/CSOSN
// ----------------------------------------------------------------

pub async fn obter_cst_csosns(State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    match sqlx::query("SELECT id, codigo, tipo, descricao, ativo FROM fiscal_dicionario_cst_csosn WHERE ativo = true ORDER BY codigo ASC")
        .fetch_all(pool).await
    {
        Ok(rows) => {
            let resp: Vec<CstCsosnResp> = rows.iter().map(|r| CstCsosnResp { id: r.get::<Uuid, _>("id").to_string(), codigo: r.get("codigo"), tipo: r.get("tipo"), descricao: r.get("descricao"), ativo: r.get("ativo") }).collect();
            (StatusCode::OK, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

pub async fn criar_cst_csosn(State(state): State<AppState>, Json(payload): Json<CstCsosnReq>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    let id = Uuid::new_v4();
    match sqlx::query("INSERT INTO fiscal_dicionario_cst_csosn (id, codigo, tipo, descricao) VALUES ($1, $2, $3, $4)")
        .bind(id).bind(&payload.codigo).bind(&payload.tipo).bind(&payload.descricao).execute(pool).await
    {
        Ok(_) => (StatusCode::CREATED, Json(json!(CstCsosnResp { id: id.to_string(), codigo: payload.codigo, tipo: payload.tipo, descricao: payload.descricao, ativo: true }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

pub async fn atualizar_cst_csosn(Path(id): Path<String>, State(state): State<AppState>, Json(payload): Json<CstCsosnReq>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    let uid = match Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response() };
    match sqlx::query("UPDATE fiscal_dicionario_cst_csosn SET codigo = $2, tipo = $3, descricao = $4, atualizado_em = now() WHERE id = $1")
        .bind(uid).bind(&payload.codigo).bind(&payload.tipo).bind(&payload.descricao).execute(pool).await
    {
        Ok(_) => (StatusCode::OK, Json(json!(CstCsosnResp { id, codigo: payload.codigo, tipo: payload.tipo, descricao: payload.descricao, ativo: true }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

pub async fn inativar_cst_csosn(Path(id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    let uid = match Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response() };
    match sqlx::query("UPDATE fiscal_dicionario_cst_csosn SET ativo = false, atualizado_em = now() WHERE id = $1").bind(uid).execute(pool).await {
        Ok(_) => (StatusCode::OK, Json(json!({"ok": true}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

// ----------------------------------------------------------------
// IVA
// ----------------------------------------------------------------

pub async fn obter_ivas(State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    match sqlx::query("SELECT id, codigo, descricao, pais_fiscal, aliquota_escala6, ativo FROM fiscal_dicionario_iva WHERE ativo = true ORDER BY codigo ASC")
        .fetch_all(pool).await
    {
        Ok(rows) => {
            let resp: Vec<IvaResp> = rows.iter().map(|r| IvaResp { id: r.get::<Uuid, _>("id").to_string(), codigo: r.get("codigo"), descricao: r.get("descricao"), pais_fiscal: r.get("pais_fiscal"), aliquota_escala6: r.get("aliquota_escala6"), ativo: r.get("ativo") }).collect();
            (StatusCode::OK, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

pub async fn criar_iva(State(state): State<AppState>, Json(payload): Json<IvaReq>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    let id = Uuid::new_v4();
    match sqlx::query("INSERT INTO fiscal_dicionario_iva (id, codigo, descricao, pais_fiscal, aliquota_escala6) VALUES ($1, $2, $3, $4, $5)")
        .bind(id).bind(&payload.codigo).bind(&payload.descricao).bind(&payload.pais_fiscal).bind(payload.aliquota_escala6).execute(pool).await
    {
        Ok(_) => (StatusCode::CREATED, Json(json!(IvaResp { id: id.to_string(), codigo: payload.codigo, descricao: payload.descricao, pais_fiscal: payload.pais_fiscal, aliquota_escala6: payload.aliquota_escala6, ativo: true }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

pub async fn atualizar_iva(Path(id): Path<String>, State(state): State<AppState>, Json(payload): Json<IvaReq>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    let uid = match Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response() };
    match sqlx::query("UPDATE fiscal_dicionario_iva SET codigo = $2, descricao = $3, pais_fiscal = $4, aliquota_escala6 = $5, atualizado_em = now() WHERE id = $1")
        .bind(uid).bind(&payload.codigo).bind(&payload.descricao).bind(&payload.pais_fiscal).bind(payload.aliquota_escala6).execute(pool).await
    {
        Ok(_) => (StatusCode::OK, Json(json!(IvaResp { id, codigo: payload.codigo, descricao: payload.descricao, pais_fiscal: payload.pais_fiscal, aliquota_escala6: payload.aliquota_escala6, ativo: true }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

pub async fn inativar_iva(Path(id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() { Some(p) => p, None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response() };
    let uid = match Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response() };
    match sqlx::query("UPDATE fiscal_dicionario_iva SET ativo = false, atualizado_em = now() WHERE id = $1").bind(uid).execute(pool).await {
        Ok(_) => (StatusCode::OK, Json(json!({"ok": true}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}
