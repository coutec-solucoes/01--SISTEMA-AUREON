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

// --- DTOs da Fase 7: Caixa, Venda e Pagamento ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessaoCaixaResp {
    pub id: String,
    pub registradora_id: String,
    pub usuario_id: String,
    pub status: String,
    pub valor_abertura: f64,
    pub valor_fechamento: Option<f64>,
    pub aberto_em: String,
    pub fechado_em: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AbrirCaixaReq {
    pub registradora_id: String,
    pub usuario_id: String,
    pub valor_abertura: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FecharCaixaReq {
    pub sessao_id: String,
    pub usuario_id: String,
    pub valor_fechamento: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VendaResumoResp {
    pub id: String,
    pub numero_venda: i64,
    pub status: String,
    pub tipo_venda: String,
    pub subtotal: f64,
    pub desconto_total: f64,
    pub acrescimo_total: f64,
    pub total: f64,
    pub total_itens: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProdutoPdvResp {
    pub produto_id: String,
    pub codigo: String,
    pub codigo_barras: Option<String>,
    pub nome: String,
    pub unidade_medida: String,
    pub preco_venda: f64,
    pub ativo: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VendaItemResp {
    pub id: String,
    pub venda_id: String,
    pub produto_id: String,
    pub descricao_produto: String,
    pub codigo_produto: Option<String>,
    pub quantidade: f64,
    pub preco_unitario: f64,
    pub desconto_item: f64,
    pub total_item: f64,
    pub cancelado: bool,
    pub criado_em: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VendaDetalheResp {
    pub venda: VendaResumoResp,
    pub itens: Vec<VendaItemResp>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PagamentoResp {
    pub id: String,
    pub venda_id: String,
    pub forma_pagamento: String,
    pub moeda_codigo: String,
    pub valor_informado: f64,
    pub valor_convertido: f64,
    pub taxa_cambio: f64,
    pub troco: f64,
    pub criado_em: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrarPagamentoReq {
    pub venda_id: String,
    pub forma_pagamento: String,
    pub moeda_codigo: String,
    pub valor_informado: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrocoResp {
    pub total_venda: f64,
    pub total_pago: f64,
    pub troco: f64,
    pub quitado: bool,
}


