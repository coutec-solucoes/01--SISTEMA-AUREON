use axum::{
    routing::{get, post, put},
    Router, middleware,
};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use crate::routes::{health, diagnostico, empresa, auth, cadastros::{pessoas, grupos}};
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
        
        // Módulo de Segurança (Fase 3 - Bloco 3)
        .route("/seguranca/usuarios", get(crate::routes::seguranca::listar_usuarios).post(crate::routes::seguranca::criar_usuario))
        .route("/seguranca/usuarios/:id", put(crate::routes::seguranca::atualizar_usuario))
        .route("/seguranca/usuarios/:id/redefinir-senha", put(crate::routes::seguranca::redefinir_senha))
        
        .route("/seguranca/perfis", get(crate::routes::seguranca::listar_perfis).post(crate::routes::seguranca::criar_perfil))
        .route("/seguranca/perfis/:id", put(crate::routes::seguranca::atualizar_perfil))
        .route("/seguranca/perfis/:id/permissoes", get(crate::routes::seguranca::obter_permissoes).put(crate::routes::seguranca::salvar_permissoes))
        
        .route("/seguranca/supervisores", get(crate::routes::seguranca::listar_supervisores).post(crate::routes::seguranca::criar_supervisor))
        .route("/seguranca/supervisores/:id", put(crate::routes::seguranca::atualizar_supervisor))
        
        .route("/seguranca/autorizacoes", get(crate::routes::seguranca::listar_autorizacoes))
        .route("/seguranca/logs", get(crate::routes::seguranca::listar_logs))
        
        // Módulo de Cadastros (Fase 4 - Bloco 2)
        .route("/cadastros/pessoas", get(pessoas::listar_pessoas).post(pessoas::criar_pessoa))
        .route("/cadastros/pessoas/:id", get(pessoas::obter_pessoa).put(pessoas::atualizar_pessoa))
        .route("/cadastros/pessoas/:id/inativar", put(pessoas::inativar_pessoa))
        .route("/cadastros/clientes", get(pessoas::listar_clientes))
        .route("/cadastros/fornecedores", get(pessoas::listar_fornecedores))
        .route("/cadastros/funcionarios", get(pessoas::listar_funcionarios))
        .route("/cadastros/vendedores", get(pessoas::listar_vendedores))
        .route("/cadastros/entregadores", get(pessoas::listar_entregadores))
        .route("/cadastros/transportadoras", get(pessoas::listar_transportadoras))
        
        .route("/cadastros/produtos/grupos", get(grupos::listar_grupos).post(grupos::criar_grupo))
        .route("/cadastros/produtos/grupos/:id", put(grupos::atualizar_grupo))
        .route("/cadastros/produtos/subgrupos", get(grupos::listar_subgrupos).post(grupos::criar_subgrupo))
        .route("/cadastros/produtos/subgrupos/:id", put(grupos::atualizar_subgrupo))
        .route("/cadastros/produtos/marcas", get(grupos::listar_marcas).post(grupos::criar_marca))
        .route("/cadastros/produtos/marcas/:id", put(grupos::atualizar_marca))
        
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
