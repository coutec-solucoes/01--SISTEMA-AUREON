use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use aureon_core::{AureonError, Resultado};
use aureon_domain::repositories::ConfiguracaoRepository;

pub struct ConfiguracaoSqliteRepository {
    conn: Arc<Mutex<Connection>>,
}

impl ConfiguracaoSqliteRepository {
    pub fn novo(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl ConfiguracaoRepository for ConfiguracaoSqliteRepository {
    fn obter(&self, chave: &str) -> Resultado<Option<String>> {
        let conn = self.conn.lock().map_err(|e| {
            AureonError::ConexaoSqlite(format!("Mutex envenenado: {e}"))
        })?;

        let resultado = conn.query_row(
            "SELECT valor_criptografado FROM configuracoes_locais WHERE chave = ?1",
            [chave],
            |r| r.get::<_, String>(0),
        );

        match resultado {
            Ok(v)                                       => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows)   => Ok(None),
            Err(e)                                      => Err(AureonError::ConexaoSqlite(e.to_string())),
        }
    }

    fn salvar(&self, chave: &str, valor_criptografado: &str) -> Resultado<()> {
        let conn = self.conn.lock().map_err(|e| {
            AureonError::ConexaoSqlite(format!("Mutex envenenado: {e}"))
        })?;

        conn.execute(
            "INSERT INTO configuracoes_locais (chave, valor_criptografado)
             VALUES (?1, ?2)
             ON CONFLICT(chave) DO UPDATE SET
                valor_criptografado = excluded.valor_criptografado,
                atualizado_em = datetime('now')",
            rusqlite::params![chave, valor_criptografado],
        )
        .map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

        Ok(())
    }

    fn remover(&self, chave: &str) -> Resultado<()> {
        let conn = self.conn.lock().map_err(|e| {
            AureonError::ConexaoSqlite(format!("Mutex envenenado: {e}"))
        })?;

        conn.execute(
            "DELETE FROM configuracoes_locais WHERE chave = ?1",
            [chave],
        )
        .map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

        Ok(())
    }

    fn listar_chaves(&self) -> Resultado<Vec<String>> {
        let conn = self.conn.lock().map_err(|e| {
            AureonError::ConexaoSqlite(format!("Mutex envenenado: {e}"))
        })?;

        let mut stmt = conn
            .prepare("SELECT chave FROM configuracoes_locais ORDER BY chave")
            .map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

        let chaves = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(chaves)
    }
}
