use sqlx::PgPool;
use crate::config::Config;
use aureon_core::AureonError;
use tracing::info;

/// Cria pool de conexão PostgreSQL.
/// Retorna Err se DATABASE_URL não estiver configurada ou PG estiver indisponível.
/// A API funciona em modo degradado sem PostgreSQL.
pub async fn conectar_postgres(config: &Config) -> Result<PgPool, AureonError> {
    let url = config.database_url.as_ref().ok_or_else(|| {
        AureonError::Configuracao(
            "DATABASE_URL não configurada — PostgreSQL indisponível".to_string(),
        )
    })?;

    info!(
        componente = "aureon-api-local::db",
        "Tentando conectar ao PostgreSQL (URL omitida por segurança)"
    );

    let pool = PgPool::connect(url)
        .await
        .map_err(|e| AureonError::ConexaoPostgres(e.to_string()))?;

    // Verifica saúde com query simples
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| AureonError::ConexaoPostgres(format!("Healthcheck PG falhou: {e}")))?;

    Ok(pool)
}
