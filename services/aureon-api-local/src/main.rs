use aureon_api_local::{config::Config, app::criar_app};
use aureon_shared::logging::inicializar_logs;
use tracing::info;

#[tokio::main]
async fn main() {
    // Inicializa logs antes de qualquer coisa
    inicializar_logs("info");

    info!(componente = "aureon-api-local", "Iniciando Aureon API Local v0.0.1");

    // Carrega configuração do ambiente
    let config = Config::carregar().unwrap_or_else(|e| {
        tracing::error!(
            componente = "aureon-api-local",
            erro = %e,
            "Falha ao carregar configuração — usando defaults"
        );
        Config::padrao()
    });

    info!(
        componente = "aureon-api-local",
        porta = config.porta,
        "Configuração carregada"
    );

    // Cria pool PostgreSQL (opcional — API funciona sem PG em modo degradado)
    let pool_pg = aureon_api_local::db::conectar_postgres(&config).await;

    match &pool_pg {
        Ok(_)  => info!(componente = "aureon-api-local", "PostgreSQL conectado com sucesso"),
        Err(e) => tracing::warn!(
            componente = "aureon-api-local",
            erro = %e,
            "PostgreSQL indisponível — API funcionará em modo degradado"
        ),
    }

    // Monta e sobe o servidor
    let app = criar_app(pool_pg.ok());
    let endereco = format!("0.0.0.0:{}", config.porta);
    let listener = tokio::net::TcpListener::bind(&endereco).await
        .unwrap_or_else(|e| panic!("Não foi possível iniciar servidor em {endereco}: {e}"));

    info!(
        componente = "aureon-api-local",
        endereco = %endereco,
        "Servidor HTTP ouvindo"
    );

    axum::serve(listener, app).await
        .unwrap_or_else(|e| panic!("Erro fatal no servidor: {e}"));
}
