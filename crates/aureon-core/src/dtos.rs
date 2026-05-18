use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Retorno do command obter_status_local
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusLocalDto {
    pub app_versao:     String,
    pub sqlite_status:  String,
    pub terminal_id:    String,
    pub horario:        DateTime<Utc>,
}

/// Entrada do command gravar_log_local
#[derive(Debug, Serialize, Deserialize)]
pub struct GravarLogDto {
    pub nivel:      String,   // INFO | WARN | ERROR | DEBUG
    pub componente: String,
    pub mensagem:   String,
}

/// Retorno de teste de conexão SQLite
#[derive(Debug, Serialize, Deserialize)]
pub struct TesteConexaoDto {
    pub sqlite_ok: bool,
    pub mensagem:  String,
}

/// Retorno de configuração local (sem expor valor puro)
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfiguracaoLocalDto {
    pub chave:               String,
    pub valor_criptografado: String,
    pub atualizado_em:       DateTime<Utc>,
}

/// Entrada para salvar configuração local
#[derive(Debug, Serialize, Deserialize)]
pub struct SalvarConfiguracaoDto {
    pub chave:       String,
    pub valor_puro:  String,  // recebido da UI; criptografado antes de persistir
}
