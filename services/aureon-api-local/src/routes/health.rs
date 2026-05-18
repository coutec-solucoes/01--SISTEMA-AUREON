use axum::Json;
use serde_json::{json, Value};

/// GET /health
/// Resposta mínima de healthcheck — sem dados de banco ou configuração
pub async fn handler_health() -> Json<Value> {
    Json(json!({
        "status":   "ok",
        "service":  "aureon-api-local",
        "version":  "0.0.1"
    }))
}
