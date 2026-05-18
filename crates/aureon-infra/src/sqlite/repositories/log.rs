use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use aureon_core::{AureonError, Resultado};
use aureon_domain::repositories::{LogRepository, EntradaLog};

pub struct LogSqliteRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LogSqliteRepository {
    pub fn novo(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl LogRepository for LogSqliteRepository {
    fn gravar(&self, nivel: &str, componente: &str, mensagem: &str) -> Resultado<()> {
        let conn = self.conn.lock().map_err(|e| {
            AureonError::ConexaoSqlite(format!("Mutex envenenado: {e}"))
        })?;

        conn.execute(
            "INSERT INTO logs_locais (nivel, componente, mensagem) VALUES (?1, ?2, ?3)",
            rusqlite::params![nivel, componente, mensagem],
        )
        .map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

        Ok(())
    }

    fn listar_recentes(&self, limite: u32) -> Resultado<Vec<EntradaLog>> {
        let conn = self.conn.lock().map_err(|e| {
            AureonError::ConexaoSqlite(format!("Mutex envenenado: {e}"))
        })?;

        let mut stmt = conn
            .prepare(
                "SELECT id, nivel, componente, mensagem, criado_em
                 FROM logs_locais
                 ORDER BY criado_em DESC
                 LIMIT ?1",
            )
            .map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

        let logs = stmt
            .query_map([limite], |r| {
                Ok(EntradaLog {
                    id:         r.get(0)?,
                    nivel:      r.get(1)?,
                    componente: r.get(2)?,
                    mensagem:   r.get(3)?,
                    criado_em:  r.get(4)?,
                })
            })
            .map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(logs)
    }
}
