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


// --- DTOs de Sincronizacao (API Local -> PDV) ---

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistroTerminalReq {
    pub codigo_terminal: String,
    pub nome_terminal: String,
    pub identificador_maquina: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistroTerminalResp {
    pub terminal_id: String,
    pub chave_terminal: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusTerminalResp {
    pub terminal_id: String,
    pub ativo: bool,
    pub autorizado: bool,
    pub status_sync: String,
    pub primeiro_sync_concluido: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrimeiraSyncReq {
    pub terminal_id: String,
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PacoteSyncResp {
    pub pacote_id: String,
    pub idempotency_key: String,
    pub status: String,
    pub hash_geral: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmacaoAplicacaoReq {
    pub pacote_id: String,
    pub terminal_id: String,
    pub idempotency_key: String,
    pub sucesso: bool,
    pub erro_detalhes: Option<String>,
}

