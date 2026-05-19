use rusqlite::Connection;
use aureon_core::AureonError;
use tracing::{info, warn};

/// Migrations SQLite versionadas.
/// Cada migration é identificada por versão e nome.
struct Migration {
    versao: i32,
    nome:   &'static str,
    sql:    &'static str,
}

/// Lista de migrations em ordem de execução.
/// NUNCA alterar migrations existentes — apenas adicionar novas.
fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            versao: 1,
            nome:   "schema_inicial",
            sql:    include_str!("../../../../database/migrations/sqlite/001_schema_inicial.sql"),
        },
        Migration {
            versao: 2,
            nome:   "sync_fase6",
            sql:    include_str!("../../../../database/migrations/sqlite/002_sync_fase6.sql"),
        },
        Migration {
            versao: 3,
            nome:   "venda_nucleo",
            sql:    include_str!("../../../../database/migrations/sqlite/003_venda_nucleo.sql"),
        },
    ]
}

/// Executa todas as migrations pendentes na ordem correta.
pub fn executar_migrations(conn: &Connection) -> Result<(), AureonError> {
    info!(
        componente = "aureon-infra::migrations",
        "Verificando migrations SQLite pendentes"
    );

    // Garante que a tabela de controle existe
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations_local (
            versao     INTEGER PRIMARY KEY,
            nome       TEXT NOT NULL,
            aplicado_em TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| AureonError::Migracao(e.to_string()))?;

    for m in migrations() {
        let ja_aplicada: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM schema_migrations_local WHERE versao = ?1",
                [m.versao],
                |r| r.get(0),
            )
            .unwrap_or(false);

        if ja_aplicada {
            warn!(
                componente = "aureon-infra::migrations",
                versao = m.versao,
                nome   = m.nome,
                "Migration já aplicada — pulando"
            );
            continue;
        }

        info!(
            componente = "aureon-infra::migrations",
            versao = m.versao,
            nome   = m.nome,
            "Aplicando migration SQLite"
        );

        conn.execute_batch(m.sql)
            .map_err(|e| AureonError::Migracao(format!("Falha na migration v{}: {e}", m.versao)))?;

        conn.execute(
            "INSERT INTO schema_migrations_local (versao, nome) VALUES (?1, ?2)",
            rusqlite::params![m.versao, m.nome],
        )
        .map_err(|e| AureonError::Migracao(e.to_string()))?;

        info!(
            componente = "aureon-infra::migrations",
            versao = m.versao,
            nome   = m.nome,
            "Migration SQLite aplicada com sucesso"
        );
    }

    Ok(())
}
