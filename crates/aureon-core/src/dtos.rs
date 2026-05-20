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
// Convencao de unidade menor (_minor):
//   BRL/USD: centavos  (R$ 10,50 -> 1050)
//   PYG: guaranis      (Gs 10500 -> 10500)
//   taxa_cambio: escala 1_000_000 (1 USD = 5.52 BRL -> 5_520_000)
//   quantidade: escala 1_000 (1.500 kg -> 1500)

// --- Caixa ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaldoMoedaResp {
    pub moeda_codigo: String,
    pub valor_abertura_minor: i64,
    pub valor_fechamento_informado_minor: Option<i64>,
    pub valor_esperado_minor: Option<i64>,
    pub diferenca_minor: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessaoCaixaResp {
    pub id: String,
    pub registradora_id: String,
    pub usuario_id: String,
    pub status: String,
    pub aberto_em: String,
    pub fechado_em: Option<String>,
    pub observacao: Option<String>,
    pub saldos: Vec<SaldoMoedaResp>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaldoMoedaReq {
    pub moeda_codigo: String,
    pub valor_minor: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AbrirCaixaReq {
    pub registradora_id: String,
    pub usuario_id: String,
    pub saldos_abertura: Vec<SaldoMoedaReq>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FecharCaixaReq {
    pub sessao_id: String,
    pub usuario_id: String,
    pub saldos_fechamento: Vec<SaldoMoedaReq>,
    pub observacao: Option<String>,
}

// --- Venda ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VendaResumoResp {
    pub id: String,
    pub numero_venda: Option<i64>,  // NULL enquanto em andamento
    pub status: String,
    pub tipo_venda: String,
    pub subtotal_minor: i64,
    pub desconto_total_minor: i64,
    pub acrescimo_total_minor: i64,
    pub total_minor: i64,
    pub total_itens: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProdutoPdvResp {
    pub produto_id: String,
    pub codigo: String,
    pub codigo_barras: Option<String>,
    pub nome: String,
    pub unidade_medida: String,
    pub preco_venda_minor: i64,
    pub ativo: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VendaItemResp {
    pub id: String,
    pub venda_id: String,
    pub produto_id: String,
    pub descricao_produto: String,
    pub codigo_produto: Option<String>,
    pub quantidade_escala3: i64,
    pub preco_unitario_minor: i64,
    pub desconto_item_minor: i64,
    pub total_item_minor: i64,
    pub cancelado: bool,
    pub cancelado_em: Option<String>,
    pub motivo_cancelamento: Option<String>,
    pub criado_em: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VendaDetalheResp {
    pub venda: VendaResumoResp,
    pub itens: Vec<VendaItemResp>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdicionarItemReq {
    pub venda_id: String,
    pub produto_id: String,
    pub descricao_produto: String,
    pub codigo_produto: Option<String>,
    pub codigo_barras: Option<String>,
    /// Quantidade em escala 3 casas (ex: 1.500 kg -> 1500)
    pub quantidade_escala3: i64,
    /// Preco unitario em minor unit (centavos)
    pub preco_unitario_minor: i64,
    /// Desconto no item em minor unit
    pub desconto_item_minor: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelarItemReq {
    pub item_id: String,
    pub usuario_cancelamento_id: String,
    pub motivo_cancelamento: String,
    pub supervisor_id: Option<String>,
    pub autorizacao_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelarVendaReq {
    pub venda_id: String,
    pub usuario_cancelamento_id: String,
    pub motivo_cancelamento: String,
    pub supervisor_id: Option<String>,
    pub autorizacao_id: Option<String>,
}

// --- Pagamento ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PagamentoResp {
    pub id: String,
    pub venda_id: String,
    pub forma_pagamento: String,
    pub moeda_codigo: String,
    pub valor_informado_minor: i64,
    pub moeda_principal_codigo: String,
    pub valor_convertido_minor: i64,
    pub taxa_cambio_escala6: i64,
    pub data_cotacao_usada: String,
    pub troco_minor: i64,
    pub moeda_troco_codigo: Option<String>,
    pub criado_em: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrarPagamentoReq {
    pub venda_id: String,
    pub forma_pagamento: String,
    pub moeda_codigo: String,
    /// Valor informado pelo operador em minor unit (centavos)
    pub valor_informado_minor: i64,
    /// Moeda em que o troco deve ser devolvido (default = moeda principal)
    pub moeda_troco_codigo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrocoResp {
    pub total_venda_minor: i64,
    pub total_pago_minor: i64,
    pub troco_minor: i64,
    pub quitado: bool,
}

// --- DTOs Fase 8: Operacional, Supervisor e Historico ---

#[derive(Debug, Serialize, Deserialize)]
pub struct CaixaMovimentacaoReq {
    pub sessao_caixa_id: String,
    pub usuario_id: String,
    pub tipo_movimentacao: String, // SUPRIMENTO, SANGRIA, VALE_FUNCIONARIO
    pub moeda_codigo: String,
    pub valor_minor: i64,
    pub motivo: Option<String>,
    pub funcionario_id: Option<String>,
    pub supervisor_id: Option<String>,
    pub autorizacao_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CaixaMovimentacaoResp {
    pub id: String,
    pub sessao_caixa_id: String,
    pub usuario_id: String,
    pub tipo_movimentacao: String,
    pub moeda_codigo: String,
    pub valor_minor: i64,
    pub motivo: Option<String>,
    pub funcionario_id: Option<String>,
    pub supervisor_id: Option<String>,
    pub autorizacao_id: Option<String>,
    pub cancelado: bool,
    pub cancelado_em: Option<String>,
    pub usuario_cancelamento_id: Option<String>,
    pub motivo_cancelamento: Option<String>,
    pub criado_em: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelarMovimentacaoReq {
    pub movimentacao_id: String,
    pub usuario_cancelamento_id: String,
    pub motivo_cancelamento: String,
    pub supervisor_id: Option<String>,
    pub autorizacao_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SolicitarAutorizacaoReq {
    pub operacao: String,
    pub usuario_solicitante_id: String,
    pub pin_supervisor: String,
    pub motivo: Option<String>,
    pub sessao_caixa_id: Option<String>,
    pub entidade_tipo: Option<String>,
    pub entidade_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutorizacaoResp {
    pub id: String,
    pub operacao: String,
    pub usuario_solicitante_id: String,
    pub supervisor_id: String,
    pub aprovado: bool,
    pub motivo: Option<String>,
    pub criado_em: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReimpressaoReq {
    pub venda_id: String,
    pub usuario_id: String,
    pub motivo: Option<String>,
    pub supervisor_id: Option<String>,
}
