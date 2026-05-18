use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Erros centralizados do sistema Aureon
#[derive(Debug, Error, Serialize, Deserialize, Clone)]
pub enum AureonError {
    #[error("Erro de conexão SQLite: {0}")]
    ConexaoSqlite(String),

    #[error("Erro de conexão PostgreSQL: {0}")]
    ConexaoPostgres(String),

    #[error("Erro de configuração: {0}")]
    Configuracao(String),

    #[error("Erro de criptografia: {0}")]
    Criptografia(String),

    #[error("Erro de migração: {0}")]
    Migracao(String),

    #[error("Erro de validação: {0}")]
    Validacao(String),

    #[error("Erro interno: {0}")]
    Interno(String),

    #[error("Recurso não encontrado: {0}")]
    NaoEncontrado(String),
}

impl AureonError {
    /// Retorna o código de erro padronizado (sem espaços, sem acentos)
    pub fn codigo(&self) -> &'static str {
        match self {
            AureonError::ConexaoSqlite(_)  => "ERRO_CONEXAO_SQLITE",
            AureonError::ConexaoPostgres(_) => "ERRO_CONEXAO_POSTGRES",
            AureonError::Configuracao(_)   => "ERRO_CONFIGURACAO",
            AureonError::Criptografia(_)   => "ERRO_CRIPTOGRAFIA",
            AureonError::Migracao(_)       => "ERRO_MIGRACAO",
            AureonError::Validacao(_)      => "ERRO_VALIDACAO",
            AureonError::Interno(_)        => "ERRO_INTERNO",
            AureonError::NaoEncontrado(_)  => "ERRO_NAO_ENCONTRADO",
        }
    }
}
