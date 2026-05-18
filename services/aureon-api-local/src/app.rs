use axum::{routing::get, Router};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use crate::routes::{health, diagnostico};

/// Monta o roteador principal da API
pub fn criar_app(pool: Option<PgPool>) -> Router {
    Router::new()
        .route("/health",            get(health::handler_health))
        .route("/diagnostico/basico", get({
            let pool = pool.clone();
            move || diagnostico::handler_diagnostico(pool)
        }))
        .layer(CorsLayer::permissive())
}
