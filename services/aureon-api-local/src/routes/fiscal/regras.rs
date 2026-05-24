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
pub struct RegraMestreReq {
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub pais_fiscal: String,
    pub tipo_operacao: Option<String>,
    pub uf_origem: Option<String>,
    pub uf_destino: Option<String>,
    pub ncm_id: Option<String>,
    pub cfop_id: Option<String>,
    pub cst_csosn_id: Option<String>,
    pub iva_id: Option<String>,
    pub aliquota_icms_escala6: Option<i64>,
    pub aliquota_pis_escala6: Option<i64>,
    pub aliquota_cofins_escala6: Option<i64>,
    pub aliquota_iva_escala6: Option<i64>,
    pub reducao_base_escala6: Option<i64>,
    pub prioridade: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegraMestreResp {
    pub id: String,
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub pais_fiscal: String,
    pub tipo_operacao: Option<String>,
    pub uf_origem: Option<String>,
    pub uf_destino: Option<String>,
    pub ncm_id: Option<String>,
    pub cfop_id: Option<String>,
    pub cst_csosn_id: Option<String>,
    pub iva_id: Option<String>,
    pub aliquota_icms_escala6: Option<i64>,
    pub aliquota_pis_escala6: Option<i64>,
    pub aliquota_cofins_escala6: Option<i64>,
    pub aliquota_iva_escala6: Option<i64>,
    pub reducao_base_escala6: Option<i64>,
    pub prioridade: Option<i32>,
    pub ativo: bool,
}

// ----------------------------------------------------------------
// Handlers
// ----------------------------------------------------------------

/// GET /fiscal/regras
pub async fn obter_regras(State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };

    match sqlx::query(
        "SELECT id, empresa_id, filial_id, pais_fiscal, tipo_operacao, uf_origem, uf_destino,
                ncm_id, cfop_id, cst_csosn_id, iva_id,
                aliquota_icms_escala6, aliquota_pis_escala6, aliquota_cofins_escala6,
                aliquota_iva_escala6, reducao_base_escala6, prioridade, ativo
         FROM fiscal_regras_tributarias_mestre
         ORDER BY COALESCE(prioridade, 0) DESC, criado_em DESC"
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => {
            let resp: Vec<RegraMestreResp> = rows.iter().map(|r| RegraMestreResp {
                id: r.get::<Uuid, _>("id").to_string(),
                empresa_id: r.get::<Option<Uuid>, _>("empresa_id").map(|u| u.to_string()),
                filial_id: r.get::<Option<Uuid>, _>("filial_id").map(|u| u.to_string()),
                pais_fiscal: r.get("pais_fiscal"),
                tipo_operacao: r.get("tipo_operacao"),
                uf_origem: r.get("uf_origem"),
                uf_destino: r.get("uf_destino"),
                ncm_id: r.get::<Option<Uuid>, _>("ncm_id").map(|u| u.to_string()),
                cfop_id: r.get::<Option<Uuid>, _>("cfop_id").map(|u| u.to_string()),
                cst_csosn_id: r.get::<Option<Uuid>, _>("cst_csosn_id").map(|u| u.to_string()),
                iva_id: r.get::<Option<Uuid>, _>("iva_id").map(|u| u.to_string()),
                aliquota_icms_escala6: r.get("aliquota_icms_escala6"),
                aliquota_pis_escala6: r.get("aliquota_pis_escala6"),
                aliquota_cofins_escala6: r.get("aliquota_cofins_escala6"),
                aliquota_iva_escala6: r.get("aliquota_iva_escala6"),
                reducao_base_escala6: r.get("reducao_base_escala6"),
                prioridade: r.get("prioridade"),
                ativo: r.get("ativo"),
            }).collect();
            (StatusCode::OK, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

/// POST /fiscal/regras
pub async fn criar_regra(State(state): State<AppState>, Json(payload): Json<RegraMestreReq>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };

    let id = Uuid::new_v4();
    let empresa_uuid: Option<Uuid> = payload.empresa_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let filial_uuid: Option<Uuid> = payload.filial_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let ncm_uuid: Option<Uuid> = payload.ncm_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let cfop_uuid: Option<Uuid> = payload.cfop_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let cst_uuid: Option<Uuid> = payload.cst_csosn_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let iva_uuid: Option<Uuid> = payload.iva_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    match sqlx::query(
        "INSERT INTO fiscal_regras_tributarias_mestre
         (id, empresa_id, filial_id, pais_fiscal, tipo_operacao, uf_origem, uf_destino,
          ncm_id, cfop_id, cst_csosn_id, iva_id,
          aliquota_icms_escala6, aliquota_pis_escala6, aliquota_cofins_escala6,
          aliquota_iva_escala6, reducao_base_escala6, prioridade)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)"
    )
    .bind(id)
    .bind(empresa_uuid).bind(filial_uuid)
    .bind(&payload.pais_fiscal)
    .bind(&payload.tipo_operacao).bind(&payload.uf_origem).bind(&payload.uf_destino)
    .bind(ncm_uuid).bind(cfop_uuid).bind(cst_uuid).bind(iva_uuid)
    .bind(payload.aliquota_icms_escala6).bind(payload.aliquota_pis_escala6).bind(payload.aliquota_cofins_escala6)
    .bind(payload.aliquota_iva_escala6).bind(payload.reducao_base_escala6).bind(payload.prioridade)
    .execute(pool)
    .await
    {
        Ok(_) => {
            let resp = RegraMestreResp {
                id: id.to_string(),
                empresa_id: empresa_uuid.map(|u| u.to_string()),
                filial_id: filial_uuid.map(|u| u.to_string()),
                pais_fiscal: payload.pais_fiscal,
                tipo_operacao: payload.tipo_operacao,
                uf_origem: payload.uf_origem,
                uf_destino: payload.uf_destino,
                ncm_id: ncm_uuid.map(|u| u.to_string()),
                cfop_id: cfop_uuid.map(|u| u.to_string()),
                cst_csosn_id: cst_uuid.map(|u| u.to_string()),
                iva_id: iva_uuid.map(|u| u.to_string()),
                aliquota_icms_escala6: payload.aliquota_icms_escala6,
                aliquota_pis_escala6: payload.aliquota_pis_escala6,
                aliquota_cofins_escala6: payload.aliquota_cofins_escala6,
                aliquota_iva_escala6: payload.aliquota_iva_escala6,
                reducao_base_escala6: payload.reducao_base_escala6,
                prioridade: payload.prioridade,
                ativo: true,
            };
            (StatusCode::CREATED, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

/// PUT /fiscal/regras/:id
pub async fn atualizar_regra(Path(id): Path<String>, State(state): State<AppState>, Json(payload): Json<RegraMestreReq>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };

    let regra_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response(),
    };

    let empresa_uuid: Option<Uuid> = payload.empresa_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let filial_uuid: Option<Uuid> = payload.filial_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let ncm_uuid: Option<Uuid> = payload.ncm_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let cfop_uuid: Option<Uuid> = payload.cfop_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let cst_uuid: Option<Uuid> = payload.cst_csosn_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let iva_uuid: Option<Uuid> = payload.iva_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    match sqlx::query(
        "UPDATE fiscal_regras_tributarias_mestre SET
         empresa_id=$2, filial_id=$3, pais_fiscal=$4, tipo_operacao=$5, uf_origem=$6, uf_destino=$7,
         ncm_id=$8, cfop_id=$9, cst_csosn_id=$10, iva_id=$11,
         aliquota_icms_escala6=$12, aliquota_pis_escala6=$13, aliquota_cofins_escala6=$14,
         aliquota_iva_escala6=$15, reducao_base_escala6=$16, prioridade=$17, atualizado_em=now()
         WHERE id=$1"
    )
    .bind(regra_id)
    .bind(empresa_uuid).bind(filial_uuid)
    .bind(&payload.pais_fiscal)
    .bind(&payload.tipo_operacao).bind(&payload.uf_origem).bind(&payload.uf_destino)
    .bind(ncm_uuid).bind(cfop_uuid).bind(cst_uuid).bind(iva_uuid)
    .bind(payload.aliquota_icms_escala6).bind(payload.aliquota_pis_escala6).bind(payload.aliquota_cofins_escala6)
    .bind(payload.aliquota_iva_escala6).bind(payload.reducao_base_escala6).bind(payload.prioridade)
    .execute(pool)
    .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"ok": true, "id": id}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

/// PUT /fiscal/regras/:id/inativar
pub async fn inativar_regra(Path(id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };
    let uid = match Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response() };
    match sqlx::query("UPDATE fiscal_regras_tributarias_mestre SET ativo = false, atualizado_em = now() WHERE id = $1")
        .bind(uid).execute(pool).await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"ok": true}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}
