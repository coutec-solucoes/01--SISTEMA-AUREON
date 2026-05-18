// =============================================================
// MODELOS DE SINCRONIZAÇÃO — FASE 0 (estrutura mínima)
// Implementação real de sync será feita nas fases posteriores.
// =============================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status possíveis de um evento de sync
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusSync {
    Pendente,
    Enviando,
    Enviado,
    Erro,
    Ignorado,
}

impl std::fmt::Display for StatusSync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            StatusSync::Pendente  => "PENDENTE",
            StatusSync::Enviando  => "ENVIANDO",
            StatusSync::Enviado   => "ENVIADO",
            StatusSync::Erro      => "ERRO",
            StatusSync::Ignorado  => "IGNORADO",
        };
        write!(f, "{s}")
    }
}

/// Modelo de evento na fila outbox/inbox
/// Mapeia diretamente para as tabelas sync_outbox e sync_inbox no SQLite
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncEvento {
    /// UUID único do evento
    pub event_id:         String,
    /// Chave para garantir idempotência no servidor
    pub idempotency_key:  String,
    /// Tipo do evento, ex: "TERMINAL_REGISTRADO"
    pub event_type:       String,
    /// Versão do schema do payload (para evolução futura)
    pub schema_version:   i32,
    /// Payload JSON do evento
    pub payload:          serde_json::Value,
    /// Status atual do processamento
    pub status:           StatusSync,
    /// Número de tentativas de envio
    pub tentativas:       i32,
    /// Último erro registrado (sem dados sensíveis)
    pub ultimo_erro:      Option<String>,
    pub criado_em:        DateTime<Utc>,
    pub atualizado_em:    DateTime<Utc>,
}

impl SyncEvento {
    pub fn novo(event_type: impl Into<String>, payload: serde_json::Value) -> Self {
        let agora = Utc::now();
        let event_id = uuid::Uuid::new_v4().to_string();
        let idempotency_key = uuid::Uuid::new_v4().to_string();

        Self {
            event_id,
            idempotency_key,
            event_type: event_type.into(),
            schema_version: 1,
            payload,
            status: StatusSync::Pendente,
            tentativas: 0,
            ultimo_erro: None,
            criado_em: agora,
            atualizado_em: agora,
        }
    }
}
