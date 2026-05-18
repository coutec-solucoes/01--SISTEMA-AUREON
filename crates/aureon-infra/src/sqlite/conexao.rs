use rusqlite::Connection;
use std::path::Path;
use aureon_core::AureonError;
use tracing::{info, error};

/// Gerenciador de conexão com o banco SQLite local.
/// O arquivo do banco fica em: C:/Aureon/data/aureon-local.db (produção)
pub struct ConexaoSqlite {
    pub conn: Connection,
}

impl ConexaoSqlite {
    /// Abre (ou cria) o banco SQLite no caminho informado.
    pub fn abrir(caminho: &Path) -> Result<Self, AureonError> {
        // Garante que o diretório pai existe
        if let Some(dir) = caminho.parent() {
            std::fs::create_dir_all(dir).map_err(|e| {
                AureonError::ConexaoSqlite(format!("Não foi possível criar diretório de dados: {e}"))
            })?;
        }

        info!(
            componente = "aureon-infra::sqlite",
            caminho = %caminho.display(),
            "Abrindo conexão SQLite"
        );

        let conn = Connection::open(caminho).map_err(|e| {
            error!(
                componente = "aureon-infra::sqlite",
                erro = %e,
                "Falha ao abrir SQLite"
            );
            AureonError::ConexaoSqlite(e.to_string())
        })?;

        // Habilita WAL para melhor performance e segurança
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

        info!(
            componente = "aureon-infra::sqlite",
            "SQLite conectado com sucesso (modo WAL ativado)"
        );

        Ok(Self { conn })
    }

    /// Verifica se a conexão está funcional com uma query simples
    pub fn verificar_saude(&self) -> Result<(), AureonError> {
        self.conn
            .execute_batch("SELECT 1")
            .map_err(|e| AureonError::ConexaoSqlite(e.to_string()))
    }
}
