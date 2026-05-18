use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use aureon_core::{AureonError, Resultado};
use aureon_domain::repositories::TerminalRepository;

pub struct TerminalSqliteRepository {
    conn: Arc<Mutex<Connection>>,
}

impl TerminalSqliteRepository {
    pub fn novo(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl TerminalRepository for TerminalSqliteRepository {
    fn obter_id_terminal(&self) -> Resultado<Option<String>> {
        let conn = self.conn.lock().map_err(|e| {
            AureonError::ConexaoSqlite(format!("Mutex envenenado: {e}"))
        })?;

        let resultado = conn.query_row(
            "SELECT terminal_id FROM terminais WHERE ativo = 1 LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        );

        match resultado {
            Ok(id)                                      => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows)   => Ok(None),
            Err(e)                                      => Err(AureonError::ConexaoSqlite(e.to_string())),
        }
    }

    fn registrar_terminal(&self, terminal_id: &str, nome: &str) -> Resultado<()> {
        let conn = self.conn.lock().map_err(|e| {
            AureonError::ConexaoSqlite(format!("Mutex envenenado: {e}"))
        })?;

        conn.execute(
            "INSERT INTO terminais (terminal_id, nome, ativo)
             VALUES (?1, ?2, 1)
             ON CONFLICT(terminal_id) DO UPDATE SET
                nome = excluded.nome,
                atualizado_em = datetime('now')",
            rusqlite::params![terminal_id, nome],
        )
        .map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

        Ok(())
    }
}
