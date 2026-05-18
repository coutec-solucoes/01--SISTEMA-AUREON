use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::prelude::*;

/// Inicializa o sistema de logs com o nível informado.
/// Nunca registrar senhas, tokens ou dados sensíveis nos logs.
pub fn inicializar_logs(nivel: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(nivel));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true).with_thread_ids(false))
        .with(filter)
        .init();

    tracing::info!(
        componente = "aureon-shared::logging",
        "Sistema de logs inicializado — nível: {}",
        nivel
    );
}
