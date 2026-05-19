use axum::{
    routing::{get, post, put},
    Router, middleware,
};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use crate::routes::{health, diagnostico, empresa, auth};
use crate::middleware::auth_middleware;

/// Estado global compartilhado da API
#[derive(Clone)]
pub struct AppState {
    pub pool: Option<PgPool>,
}

/// Monta o roteador principal da API
pub fn criar_app(pool: Option<PgPool>) -> Router {
    let state = AppState { pool: pool.clone() };

    // Rotas protegidas (exigem token)
    let rotas_protegidas = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .route("/health",            get(health::handler_health))
        .route("/diagnostico/basico", get({
            let pool = pool.clone();
            move || diagnostico::handler_diagnostico(pool)
        }))
        
        // Autenticação (públicas)
        .route("/auth/login", post(auth::login))
        .route("/auth/setup", post(auth::setup_admin))
        
        // Endpoints do módulo Empresa (Fase 2)
        .route("/empresa/configuracao", get(empresa::obter_configuracao).post(empresa::salvar_configuracao).put(empresa::salvar_configuracao))
        .route("/empresa/moedas", get(empresa::obter_moedas).put(empresa::salvar_moedas))
        .route("/empresa/cotacoes", get(empresa::obter_cotacoes).post(empresa::criar_cotacao))
        .route("/empresa/cotacoes/:id/cancelar", put(empresa::cancelar_cotacao))
        .route("/empresa/fiscal", get(empresa::obter_fiscal).put(empresa::salvar_fiscal))
        .route("/empresa/parametros-operacionais", get(empresa::obter_parametros).put(empresa::salvar_parametros))
        .route("/empresa/auditoria", get(empresa::obter_auditoria))
        .route("/empresa/status-configuracao", get(empresa::status_configuracao))
        
        .merge(rotas_protegidas)
        .with_state(state)
        .layer(CorsLayer::permissive())
}
