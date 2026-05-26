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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreVendaItemResp {
    pub id: String,
    pub pre_venda_id: String,
    pub produto_id: String,
    pub descricao: String,
    pub quantidade_escala3: i64,
    pub preco_unitario_minor: i64,
    pub desconto_minor: i64,
    pub total_minor: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreVendaResp {
    pub id: String,
    pub numero: String,
    pub cliente_id: Option<String>,
    pub vendedor_id: Option<String>,
    pub total_minor: i64,
    pub status: String,
    pub validade: Option<String>,
    pub criado_em: String,
    pub itens: Vec<PreVendaItemResp>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConverterPreVendaReq {
    pub pre_venda_id: String,
    pub sessao_caixa_id: String,
    pub usuario_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrcamentoItemResp {
    pub id: String,
    pub orcamento_id: String,
    pub produto_id: String,
    pub descricao: String,
    pub quantidade_escala3: i64,
    pub preco_unitario_minor: i64,
    pub desconto_minor: i64,
    pub total_minor: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrcamentoResp {
    pub id: String,
    pub numero: String,
    pub cliente_id: Option<String>,
    pub vendedor_id: Option<String>,
    pub total_minor: i64,
    pub status: String,
    pub validade: Option<String>,
    pub criado_em: String,
    pub itens: Vec<OrcamentoItemResp>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConverterOrcamentoReq {
    pub orcamento_id: String,
    pub sessao_caixa_id: String,
    pub usuario_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClienteResp {
    pub id: String,
    pub nome: String,
    pub documento: Option<String>,
    pub ativo: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssociarClienteReq {
    pub venda_id: String,
    pub cliente_id: String,
}

// --- DTOs Fase 9: PDV Gourmet ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MesaOperacionalResp {
    pub id: Option<String>,
    pub mesa_id: String,
    pub mesa_numero: i32,
    pub nome_exibicao: String,
    pub cliente_nome_informal: Option<String>,
    pub cliente_id: Option<String>,
    pub status: String, // 'LIVRE', 'ABERTA', 'RESERVADA', 'BLOQUEADA', 'FECHADA', 'CANCELADA'
    pub usuario_abertura_id: Option<String>,
    pub sessao_caixa_id: Option<String>,
    pub observacao: Option<String>,
    pub aberta_em: Option<String>,
    pub total_consumo_minor: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AbrirMesaReq {
    pub mesa_numero: i32,
    pub nome_exibicao: String,
    pub cliente_nome_informal: Option<String>,
    pub cliente_id: Option<String>,
    pub usuario_id: String,
    pub sessao_caixa_id: String,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReservarMesaReq {
    pub mesa_numero: i32,
    pub nome_exibicao: String,
    pub cliente_nome_informal: Option<String>,
    pub cliente_id: Option<String>,
    pub usuario_id: String,
    pub sessao_caixa_id: String,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BloquearMesaReq {
    pub mesa_numero: i32,
    pub usuario_id: String,
    pub sessao_caixa_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelarMesaReq {
    pub mesa_numero: i32,
    pub usuario_cancelamento_id: String,
    pub motivo_cancelamento: String,
    pub supervisor_id: Option<String>,
    pub autorizacao_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComandaOperacionalResp {
    pub id: Option<String>,
    pub comanda_id: String,
    pub numero_comanda: i32,
    pub codigo_barras_qr: Option<String>,
    pub cliente_nome_informal: Option<String>,
    pub cliente_id: Option<String>,
    pub status: String, // 'LIVRE', 'ABERTA', 'BLOQUEADA', 'FECHADA', 'CANCELADA'
    pub usuario_abertura_id: Option<String>,
    pub sessao_caixa_id: Option<String>,
    pub observacao: Option<String>,
    pub aberta_em: Option<String>,
    pub total_consumo_minor: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AbrirComandaReq {
    pub numero_comanda: i32,
    pub codigo_barras_qr: Option<String>,
    pub cliente_nome_informal: Option<String>,
    pub cliente_id: Option<String>,
    pub usuario_id: String,
    pub sessao_caixa_id: String,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BloquearComandaReq {
    pub numero_comanda: i32,
    pub usuario_id: String,
    pub sessao_caixa_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelarComandaReq {
    pub numero_comanda: i32,
    pub usuario_cancelamento_id: String,
    pub motivo_cancelamento: String,
    pub supervisor_id: Option<String>,
    pub autorizacao_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GourmetItemResp {
    pub id: String,
    pub origem_tipo: String, // 'MESA' | 'COMANDA'
    pub origem_id: String,
    pub produto_id: String,
    pub descricao_produto: String,
    pub codigo_produto: String,
    pub quantidade_escala3: i64,
    pub preco_unitario_minor: i64,
    pub desconto_item_minor: i64,
    pub acrescimo_item_minor: i64,
    pub total_item_minor: i64,
    pub observacao_producao: Option<String>,
    pub local_producao_id: String,
    pub status: String, // 'PENDENTE', 'ENVIADO_PRODUCAO', 'CANCELADO', 'TRANSFERIDO', 'FECHADO'
    pub enviado_producao: bool,
    pub enviado_producao_em: Option<String>,
    pub cancelado: bool,
    pub cancelado_em: Option<String>,
    pub motivo_cancelamento: Option<String>,
    pub supervisor_id: Option<String>,
    pub autorizacao_id: Option<String>,
    pub criado_em: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdicionarItemGourmetReq {
    pub origem_tipo: String, // 'MESA' | 'COMANDA'
    pub origem_id: String, // ID operacional
    pub produto_id: String,
    pub quantidade_escala3: i64,
    pub preco_unitario_minor: i64,
    pub desconto_item_minor: i64,
    pub acrescimo_item_minor: i64,
    pub observacao_producao: Option<String>,
    pub local_producao_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelarItemGourmetReq {
    pub item_id: String,
    pub usuario_cancelamento_id: String,
    pub motivo_cancelamento: String,
    pub supervisor_id: Option<String>,
    pub autorizacao_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MesaDetalheResp {
    pub mesa: MesaOperacionalResp,
    pub itens: Vec<GourmetItemResp>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComandaDetalheResp {
    pub comanda: ComandaOperacionalResp,
    pub itens: Vec<GourmetItemResp>,
}

// --- DTOs Bloco 3 Fase 9: Transferências, Produção e Fechamento em Venda ---

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferirTotalReq {
    pub origem_tipo: String,  // 'MESA' | 'COMANDA'
    pub origem_id: String,    // UUID do ciclo operacional de origem
    pub destino_tipo: String, // 'MESA' | 'COMANDA'
    pub destino_id: String,   // UUID do ciclo operacional de destino
    pub usuario_id: String,
    pub motivo: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferenciaItemReq {
    pub item_origem_id: String,
    pub quantidade_escala3: i64, // quantidade a transferir (parcial ou total do item)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferirItensReq {
    pub origem_tipo: String,
    pub origem_id: String,
    pub destino_tipo: String,
    pub destino_id: String,
    pub usuario_id: String,
    pub motivo: String,
    pub itens: Vec<TransferenciaItemReq>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransferenciaResp {
    pub transferencia_id: String,
    pub origem_tipo: String,
    pub origem_id: String,
    pub destino_tipo: String,
    pub destino_id: String,
    pub total: bool,
    pub itens_transferidos: i64,
    pub criado_em: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnviarProducaoReq {
    pub origem_tipo: String,
    pub origem_id: String,
    pub usuario_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProducaoEnvioItemResp {
    pub item_id: String,
    pub produto_id: String,
    pub descricao_produto: String,
    pub quantidade_escala3: i64,
    pub observacao_producao: Option<String>,
    pub cancelamento: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProducaoEnvioResp {
    pub id: String,
    pub origem_tipo: String,
    pub origem_id: String,
    pub setor_producao_id: String,
    pub usuario_id: String,
    pub status: String,
    pub texto_producao: String,
    pub itens: Vec<ProducaoEnvioItemResp>,
    pub criado_em: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReimpressaoProducaoReq {
    pub envio_id: String,
    pub usuario_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FecharEmVendaReq {
    pub origem_tipo: String,   // 'MESA' | 'COMANDA'
    pub origem_id: String,     // UUID do ciclo operacional
    pub usuario_id: String,
    pub sessao_caixa_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FechamentoEmVendaResp {
    pub venda_id: String,
    pub origem_tipo: String,
    pub origem_id: String,
    pub total_minor: i64,
    pub total_itens: i64,
    pub status_venda: String,
}

// --- FASE 10: DELIVERY ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntregadorResp {
    pub id: String,
    pub nome: String,
    pub documento: Option<String>,
    pub ativo: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeliveryItemResp {
    pub id: String,
    pub delivery_id: String,
    pub produto_id: String,
    pub descricao_produto: String,
    pub codigo_produto: Option<String>,
    pub quantidade_escala3: i64,
    pub preco_unitario_minor: i64,
    pub desconto_item_minor: i64,
    pub acrescimo_item_minor: i64,
    pub total_item_minor: i64,
    pub observacao_producao: Option<String>,
    pub local_producao_id: Option<String>,
    pub status: String,
    pub enviado_producao: bool,
    pub cancelado: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeliveryOperacionalResp {
    pub id: String,
    pub numero_pedido: i64,
    pub cliente_id: Option<String>,
    pub nome_cliente_informal: String,
    pub telefone: String,
    pub endereco_completo: Option<String>,
    pub tipo_pedido: String,
    pub status: String,
    pub origem: String,
    pub entregador_id: Option<String>,
    pub taxa_entrega_minor: i64,
    pub total_consumo_minor: i64,
    pub sessao_caixa_id: Option<String>,
    pub observacao: Option<String>,
    pub previsao_entrega: Option<String>,
    pub aberto_em: String,
    pub fechado_em: Option<String>,
    pub itens: Vec<DeliveryItemResp>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CriarPedidoLocalReq {
    pub nome_cliente_informal: String,
    pub telefone: String,
    pub endereco_completo: Option<String>,
    pub tipo_pedido: String,
    pub taxa_entrega_minor: i64,
    pub observacao: Option<String>,
    pub sessao_caixa_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecusarPedidoOnlineReq {
    pub delivery_id: String,
    pub motivo: String,
    pub sessao_caixa_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AtualizarStatusDeliveryReq {
    pub delivery_id: String,
    pub novo_status: String,
    pub sessao_caixa_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefinirEntregadorReq {
    pub delivery_id: String,
    pub entregador_id: String,
    pub sessao_caixa_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdicionarItemDeliveryReq {
    pub delivery_id: String,
    pub produto_id: String,
    pub quantidade_escala3: i64,
    pub acrescimo_item_minor: i64,
    pub desconto_item_minor: i64,
    pub observacao_producao: Option<String>,
    pub usuario_id: String,
    pub sessao_caixa_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelarItemDeliveryReq {
    pub item_id: String,
    pub motivo_cancelamento: String,
    pub usuario_cancelamento_id: String,
    pub supervisor_id: Option<String>,
    pub sessao_caixa_id: String,
}

// --- FASE 11: ESTOQUE OPERACIONAL ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EstoqueSaldoResp {
    pub produto_id: String,
    pub codigo: Option<String>,
    pub descricao: String,
    pub controla_estoque: bool,
    pub quantidade_escala3: i64,
    pub atualizado_em: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EstoqueMovimentacaoResp {
    pub id: String,
    pub produto_id: String,
    pub quantidade_escala3: i64,
    pub saldo_apos_escala3: i64,
    pub tipo_movimentacao: String,
    pub origem_tipo: String,
    pub origem_id: String,
    pub motivo: Option<String>,
    pub usuario_id: String,
    pub criado_em: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AjusteEstoqueReq {
    pub idempotency_key: Option<String>,
    pub produto_id: String,
    pub tipo_ajuste: String, // "AJUSTE_ENTRADA" | "AJUSTE_SAIDA"
    pub quantidade_escala3: i64,
    pub motivo: String,
    pub usuario_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContagemInventario {
    pub produto_id: String,
    pub saldo_real_escala3: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InventarioEstoqueReq {
    pub idempotency_key: Option<String>,
    pub contagens: Vec<ContagemInventario>,
    pub motivo: Option<String>,
    pub usuario_id: String,
}

// --- FASE 12: COMPRAS E ENTRADA MANUAL ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FornecedorResp {
    pub id: String,
    pub nome: String,
    pub documento: Option<String>,
    pub ativo: bool,
    pub atualizado_em: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompraItemResp {
    pub id: String,
    pub compra_id: String,
    pub produto_id: String,
    pub descricao_produto_snapshot: String,
    pub quantidade_escala3: i64,
    pub custo_unitario_minor: i64,
    pub total_item_minor: i64,
    pub lote: Option<String>,
    pub validade: Option<String>,
    pub serial: Option<String>,
    pub imei: Option<String>,
    pub cancelado: bool,
    pub criado_em: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompraResp {
    pub id: String,
    pub fornecedor_id: String,
    pub fornecedor_nome_snapshot: String,
    pub numero_nota: Option<String>,
    pub serie: Option<String>,
    pub chave_acesso_xml_fiscal: Option<String>,
    pub data_emissao: Option<String>,
    pub status: String,
    pub moeda_codigo: String,
    pub taxa_cambio_escala6: i64,
    pub subtotal_itens_minor: i64,
    pub desconto_total_minor: i64,
    pub frete_total_minor: i64,
    pub outras_despesas_minor: i64,
    pub impostos_total_minor: i64,
    pub total_compra_minor: i64,
    pub observacao: Option<String>,
    pub criado_em: String,
    pub atualizado_em: String,
    pub finalizada_em: Option<String>,
    pub cancelada_em: Option<String>,
    pub motivo_cancelamento: Option<String>,
    pub usuario_id: String,
    pub itens: Vec<CompraItemResp>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IniciarCompraReq {
    pub fornecedor_id: String,
    pub numero_nota: Option<String>,
    pub serie: Option<String>,
    pub chave_acesso_xml_fiscal: Option<String>,
    pub data_emissao: Option<String>,
    pub moeda_codigo: String,
    pub taxa_cambio_escala6: i64,
    pub observacao: Option<String>,
    pub usuario_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdicionarItemCompraReq {
    pub compra_id: String,
    pub produto_id: String,
    pub quantidade_escala3: i64,
    pub custo_unitario_minor: i64,
    pub lote: Option<String>,
    pub validade: Option<String>,
    pub serial: Option<String>,
    pub imei: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelarCompraEmAndamentoReq {
    pub compra_id: String,
    pub motivo: String,
    pub usuario_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelarCompraFinalizadaReq {
    pub compra_id: String,
    pub motivo: String,
    pub usuario_id: String,
}

// --- FASE 13: FINANCEIRO BASE ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContaPagarResp {
    pub id: String,
    pub fornecedor_id: Option<String>,
    pub fornecedor_nome_snapshot: Option<String>,
    pub compra_id: Option<String>,
    pub descricao: String,
    pub moeda_codigo: String,
    pub valor_original_minor: i64,
    pub taxa_cambio_escala6: i64,
    pub valor_original_principal_minor: i64,
    pub data_emissao: String,
    pub data_vencimento: String,
    pub status: String,
    pub saldo_pendente_minor: i64,
    pub criado_em: String,
    pub atualizado_em: String,
    pub usuario_id: String,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContaReceberResp {
    pub id: String,
    pub cliente_id: Option<String>,
    pub cliente_nome_snapshot: Option<String>,
    pub venda_id: Option<String>,
    pub descricao: String,
    pub moeda_codigo: String,
    pub valor_original_minor: i64,
    pub taxa_cambio_escala6: i64,
    pub valor_original_principal_minor: i64,
    pub data_emissao: String,
    pub data_vencimento: String,
    pub status: String,
    pub saldo_pendente_minor: i64,
    pub criado_em: String,
    pub atualizado_em: String,
    pub usuario_id: String,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinanceiroLancamentoResp {
    pub id: String,
    pub conta_pagar_id: Option<String>,
    pub conta_receber_id: Option<String>,
    pub sessao_caixa_id: Option<String>,
    pub tipo_lancamento: String,
    pub forma_pagamento: String,
    pub moeda_codigo: String,
    pub valor_informado_minor: i64,
    pub taxa_cambio_escala6: i64,
    pub valor_principal_minor: i64,
    pub data_pagamento: String,
    pub usuario_id: String,
    pub observacao: Option<String>,
    pub criado_em: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrarDespesaReq {
    pub fornecedor_id: Option<String>,
    pub descricao: String,
    pub moeda_codigo: String,
    pub valor_original_minor: i64,
    pub taxa_cambio_escala6: i64,
    pub data_emissao: String,
    pub data_vencimento: String,
    pub usuario_id: String,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaixarContaPagarReq {
    pub conta_pagar_id: String,
    pub sessao_caixa_id: String,
    pub forma_pagamento: String,
    pub moeda_codigo: String,
    pub valor_informado_minor: i64,
    pub taxa_cambio_escala6: i64,
    pub usuario_id: String,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelarContaPagarReq {
    pub conta_pagar_id: String,
    pub motivo: String,
    pub usuario_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaixarContaReceberReq {
    pub conta_receber_id: String,
    pub sessao_caixa_id: String,
    pub forma_pagamento: String,
    pub moeda_codigo: String,
    pub valor_informado_minor: i64,
    pub taxa_cambio_escala6: i64,
    pub usuario_id: String,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelarContaReceberReq {
    pub conta_receber_id: String,
    pub motivo: String,
    pub usuario_id: String,
}

// --- DTOs da Fase 14: Relatórios Operacionais e Dashboard ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FiltrosRelatorio {
    pub data_inicio: Option<String>,
    pub data_fim: Option<String>,
    pub usuario_id: Option<String>,
    pub sessao_caixa_id: Option<String>,
    pub moeda_codigo: Option<String>,
    pub forma_pagamento: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TotalPorMoeda {
    pub moeda_codigo: String,
    pub valor_minor: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndicadoresDashboardResp {
    pub faturamento_por_moeda: Vec<TotalPorMoeda>,
    pub despesas_por_moeda: Vec<TotalPorMoeda>,
    pub total_vendas_quantidade: i64,
    pub total_vendas_itens_quantidade_escala3: i64,
    pub produtos_estoque_critico: i64,
    pub contas_pagar_vencidas_por_moeda: Vec<TotalPorMoeda>,
    pub contas_pagar_a_vencer_por_moeda: Vec<TotalPorMoeda>,
    pub contas_receber_vencidas_por_moeda: Vec<TotalPorMoeda>,
    pub contas_receber_a_vencer_por_moeda: Vec<TotalPorMoeda>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelatorioVendasItem {
    pub id: String,
    pub numero_venda: Option<String>,
    pub data_venda: String,
    pub total_bruto_minor: i64,
    pub desconto_total_minor: i64,
    pub acrescimo_total_minor: i64,
    pub total_liquido_minor: i64,
    pub status: String,
    pub cliente_nome: Option<String>,
    pub usuario_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VendasPorFormaPagamento {
    pub forma_pagamento: String,
    pub moeda_codigo: String,
    pub total_minor: i64,
    pub quantidade: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelatorioVendasResp {
    pub vendas: Vec<RelatorioVendasItem>,
    pub totais_por_moeda: Vec<TotalPorMoeda>,
    pub vendas_por_forma: Vec<VendasPorFormaPagamento>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelatorioCaixaItem {
    pub id: String,
    pub operador_id: String,
    pub terminal_id: String,
    pub status: String,
    pub aberto_em: String,
    pub fechado_em: Option<String>,
    pub moeda_codigo: String,
    pub valor_abertura_minor: i64,
    pub valor_fechamento_esperado_minor: i64,
    pub valor_fechamento_informado_minor: i64,
    pub diferenca_minor: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelatorioCaixaResp {
    pub sessoes: Vec<RelatorioCaixaItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelatorioFinanceiroResp {
    pub contas_pagar: Vec<ContaPagarResp>,
    pub contas_receber: Vec<ContaReceberResp>,
    pub lancamentos: Vec<FinanceiroLancamentoResp>,
    pub total_pagar_pendente: Vec<TotalPorMoeda>,
    pub total_receber_pendente: Vec<TotalPorMoeda>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EstoqueKardexItem {
    pub id: String,
    pub produto_id: String,
    pub produto_nome: String,
    pub produto_sku: String,
    pub tipo_movimentacao: String,
    pub quantidade_escala3: i64,
    pub data_movimentacao: String,
    pub origem_id: Option<String>,
    pub usuario_id: String,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PosicaoEstoqueItem {
    pub produto_id: String,
    pub produto_nome: String,
    pub produto_sku: String,
    pub controla_estoque: bool,
    pub quantidade_escala3: i64,
    pub estoque_minimo_escala3: i64,
    pub ultimo_custo_minor: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelatorioEstoqueResp {
    pub itens_kardex: Vec<EstoqueKardexItem>,
    pub posicao_estoque: Vec<PosicaoEstoqueItem>,
    pub custo_total_estimado_brl: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompraRelatorioItem {
    pub id: String,
    pub fornecedor_nome: String,
    pub data_compra: String,
    pub status: String,
    pub moeda_codigo: String,
    pub total_original_minor: i64,
    pub total_principal_brl_minor: i64,
    pub total_itens_escala3: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompraFornecedorTotal {
    pub fornecedor_nome: String,
    pub total_principal_brl_minor: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelatorioComprasResp {
    pub compras: Vec<CompraRelatorioItem>,
    pub total_por_fornecedor: Vec<CompraFornecedorTotal>,
    pub total_por_moeda: Vec<TotalPorMoeda>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProdutoMaisVendidoResp {
    pub produto_id: String,
    pub produto_nome: String,
    pub produto_sku: String,
    pub quantidade_vendida_escala3: i64,
    pub faturamento_bruto_minor: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeliveryStatusContagem {
    pub status: String,
    pub quantidade: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelatorioGourmetDeliveryResp {
    pub total_pedidos_delivery: i64,
    pub delivery_por_status: Vec<DeliveryStatusContagem>,
    pub faturamento_delivery_moeda: Vec<TotalPorMoeda>,
    pub taxa_entrega_total_minor: i64,
    pub total_atendimentos_gourmet: i64,
    pub faturamento_gourmet_moeda: Vec<TotalPorMoeda>,
    pub ticket_medio_gourmet_brl_minor: i64,
}






// ==========================================
// DTOs de Impressão Operacional (Fase 15)
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoDestinoImpressao {
    TcpIp,
    WindowsRaw,
    Simulador,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpressoraDestinoReq {
    pub impressora_id: Option<String>,
    pub nome: String,
    pub tipo_destino: TipoDestinoImpressao,
    pub endereco_ip: Option<String>,
    pub porta: Option<u16>,
    pub nome_spooler: Option<String>,
    pub caminho_simulador: Option<String>,
    pub largura_colunas: u8, // 32, 42, 48
    pub cortar_papel: bool,
    pub abrir_gaveta: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TesteImpressoraReq {
    pub destino: ImpressoraDestinoReq,
    pub texto_teste: Option<String>,
    pub usuario_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpressaoResultadoResp {
    pub sucesso: bool,
    pub mensagem: String,
    pub destino_usado: String,
    pub caminho_arquivo_simulado: Option<String>,
    pub bytes_gerados: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprimirVendaReq {
    pub venda_id: String,
    pub destino: ImpressoraDestinoReq,
    pub usuario_id: Option<String>,
    pub numero_via: Option<i32>,
    pub imprimir_itens_cancelados: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReimprimirVendaReq {
    pub venda_id: String,
    pub destino: ImpressoraDestinoReq,
    pub usuario_id: Option<String>,
    pub motivo_reimpressao: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprimirBaixaFinanceiraReq {
    pub lancamento_id: String,
    pub destino: ImpressoraDestinoReq,
    pub usuario_id: Option<String>,
    pub numero_via: Option<i32>,
}

// --- DTOs Fase 15 Bloco 3: Comprovantes de Caixa ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprimirMovimentacaoCaixaReq {
    pub movimentacao_id: String,
    pub destino: ImpressoraDestinoReq,
    pub usuario_id: Option<String>,
    pub numero_via: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprimirSessaoCaixaReq {
    pub sessao_caixa_id: String,
    pub destino: ImpressoraDestinoReq,
    pub usuario_id: Option<String>,
    pub numero_via: Option<i32>,
}

// --- DTOs Fase 15 Bloco 4: Produção, Delivery e Gaveta ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprimirProducaoReq {
    pub envio_id: String,
    pub destino: ImpressoraDestinoReq,
    pub usuario_id: Option<String>,
    pub numero_via: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprimirCancelamentoProducaoReq {
    pub origem_tipo: String,
    pub origem_id: String,
    pub item_id: Option<String>,
    pub destino: ImpressoraDestinoReq,
    pub usuario_id: Option<String>,
    pub motivo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprimirRomaneioDeliveryReq {
    pub delivery_id: String,
    pub destino: ImpressoraDestinoReq,
    pub usuario_id: Option<String>,
    pub numero_via: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbrirGavetaReq {
    pub destino: ImpressoraDestinoReq,
    pub usuario_id: Option<String>,
    pub motivo: Option<String>,
}

// =========================================
// FASE 16 - DTOs FISCAIS (ESTRUTURA E PREVIEW)
// =========================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FiscalEmpresaConfigResp {
    pub id: String,
    pub pais_fiscal: String,
    pub regime_fiscal: Option<String>,
    pub ambiente: String,
    pub forma_emissao: String,
    pub certificado_alias: Option<String>,
    pub certificado_caminho: Option<String>,
    pub configuracao_json: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SalvarFiscalEmpresaConfigReq {
    pub pais_fiscal: String,
    pub regime_fiscal: Option<String>,
    pub ambiente: String,
    pub forma_emissao: String,
    pub certificado_alias: Option<String>,
    pub certificado_caminho: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FiscalNcmResp {
    pub id: String,
    pub codigo: String,
    pub descricao: Option<String>,
    pub ativo: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FiscalCfopResp {
    pub id: String,
    pub codigo: String,
    pub descricao: Option<String>,
    pub tipo_operacao: Option<String>,
    pub ativo: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FiscalCstCsosnResp {
    pub id: String,
    pub codigo: String,
    pub tipo: String,
    pub descricao: Option<String>,
    pub ativo: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FiscalIvaResp {
    pub id: String,
    pub codigo: String,
    pub descricao: Option<String>,
    pub aliquota_escala6: i64,
    pub ativo: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SalvarFiscalIvaReq {
    pub codigo: String,
    pub descricao: Option<String>,
    pub aliquota_escala6: i64,
    pub ativo: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FiscalRegraTributariaResp {
    pub id: String,
    pub pais_fiscal: String,
    pub tipo_operacao: String,
    pub uf_origem: Option<String>,
    pub uf_destino: Option<String>,
    pub ncm_id: Option<String>,
    pub cfop_id: Option<String>,
    pub cst_csosn_id: Option<String>,
    pub iva_id: Option<String>,
    pub aliquota_icms_escala6: i64,
    pub aliquota_pis_escala6: i64,
    pub aliquota_cofins_escala6: i64,
    pub aliquota_iva_escala6: i64,
    pub reducao_base_escala6: i64,
    pub ativo: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SalvarFiscalRegraTributariaReq {
    pub pais_fiscal: String,
    pub tipo_operacao: String,
    pub uf_origem: Option<String>,
    pub uf_destino: Option<String>,
    pub ncm_id: Option<String>,
    pub cfop_id: Option<String>,
    pub cst_csosn_id: Option<String>,
    pub iva_id: Option<String>,
    pub aliquota_icms_escala6: i64,
    pub aliquota_pis_escala6: i64,
    pub aliquota_cofins_escala6: i64,
    pub aliquota_iva_escala6: i64,
    pub reducao_base_escala6: i64,
    pub ativo: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VincularFiscalProdutoReq {
    pub produto_id: String,
    pub ncm_id: Option<String>,
    pub iva_id: Option<String>,
    pub cst_csosn_id: Option<String>,
    pub cfop_padrao_id: Option<String>,
    pub origem_mercadoria: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FiscalEventoLogResp {
    pub id: String,
    pub venda_id: Option<String>,
    pub tipo_evento: String,
    pub origem: Option<String>,
    pub payload_preview: Option<String>,
    pub mensagem: Option<String>,
    pub criado_em: String,
}

// =========================================
// FASE 16 BLOCO 3 — DTOs ESPELHO FISCAL
// Preview técnico sem emissão ou transmissão
// =========================================

/// Item individual de validação cadastral fiscal
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidacaoFiscalItemResp {
    /// "empresa" | "produto:{id}" | "cliente:{id}"
    pub entidade: String,
    /// "OK" | "AVISO" | "ERRO"
    pub nivel: String,
    pub mensagem: String,
}

/// Resultado completo da validação cadastral fiscal
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidacaoFiscalResp {
    pub valido: bool,
    pub pais_fiscal: Option<String>,
    pub ambiente: Option<String>,
    pub total_erros: i32,
    pub total_avisos: i32,
    pub itens: Vec<ValidacaoFiscalItemResp>,
}

/// Espelho fiscal de um item da venda (preview técnico)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EspelhoFiscalItemResp {
    pub venda_item_id: String,
    pub produto_id: String,
    pub descricao_produto: String,
    pub ncm_id: Option<String>,
    pub cfop_id: Option<String>,
    pub cst_csosn_id: Option<String>,
    pub iva_id: Option<String>,
    /// Base de cálculo em minor unit (centavos/guaranis)
    pub base_minor: i64,
    /// Alíquota em escala 6 (ex: 10% = 100000)
    pub aliquota_escala6: i64,
    /// Imposto calculado = base * aliquota / 1_000_000
    pub imposto_minor: i64,
    pub origem_regra: String, // "REGRA_TRIBUTARIA" | "VINCULO_PRODUTO" | "SEM_DADOS"
}

/// Espelho fiscal da venda completa (preview técnico)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EspelhoFiscalVendaResp {
    pub venda_id: String,
    pub pais_fiscal: String,
    pub ambiente: String,
    pub modelo_preview: String, // "NFC-E_BR" | "SIFEN_PY" | "GENERICO"
    pub status_preparacao: String, // "PREVIEW_OK" | "PREVIEW_COM_ALERTAS" | "PREVIEW_INCOMPLETO"
    pub total_base_minor: i64,
    pub total_imposto_minor: i64,
    pub calculado_em: String,
    pub itens: Vec<EspelhoFiscalItemResp>,
    pub alertas: Vec<String>,
}

/// Requisição para calcular o espelho fiscal de uma venda
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CalcularEspelhoFiscalVendaReq {
    pub venda_id: String,
    /// Tipo de operação para busca de regras: ex. "VENDA_BALCAO", "VENDA_ENTREGA"
    pub tipo_operacao: Option<String>,
}

/// Requisição para obter o espelho fiscal já calculado
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObterEspelhoFiscalVendaReq {
    pub venda_id: String,
}

// =========================================
// FASE 17 BLOCO 2 — DTOs MESTRE FISCAL (RETAGUARDA)
// =========================================

// --- Configurações ---
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FiscalEmpresaConfigMestreReq {
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub pais_fiscal: String,
    pub regime_fiscal: Option<String>,
    pub ambiente: String,
    pub forma_emissao: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FiscalEmpresaConfigMestreResp {
    pub id: String,
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub pais_fiscal: String,
    pub regime_fiscal: Option<String>,
    pub ambiente: String,
    pub forma_emissao: String,
    pub ativo: bool,
}

// --- Dicionários ---
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DicionarioNcmReq {
    pub codigo: String,
    pub descricao: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DicionarioNcmResp {
    pub id: String,
    pub codigo: String,
    pub descricao: Option<String>,
    pub ativo: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DicionarioCfopReq {
    pub codigo: String,
    pub descricao: Option<String>,
    pub tipo_operacao: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DicionarioCfopResp {
    pub id: String,
    pub codigo: String,
    pub descricao: Option<String>,
    pub tipo_operacao: Option<String>,
    pub ativo: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DicionarioCstCsosnReq {
    pub codigo: String,
    pub tipo: String, // "CST" | "CSOSN" | "PIS" | "COFINS"
    pub descricao: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DicionarioCstCsosnResp {
    pub id: String,
    pub codigo: String,
    pub tipo: String,
    pub descricao: Option<String>,
    pub ativo: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DicionarioIvaReq {
    pub codigo: String,
    pub descricao: Option<String>,
    pub pais_fiscal: String,
    pub aliquota_escala6: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DicionarioIvaResp {
    pub id: String,
    pub codigo: String,
    pub descricao: Option<String>,
    pub pais_fiscal: String,
    pub aliquota_escala6: i64,
    pub ativo: bool,
}

// --- Regras Tributárias ---
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegraTributariaMestreReq {
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub pais_fiscal: String,
    pub tipo_operacao: String,
    pub uf_origem: Option<String>,
    pub uf_destino: Option<String>,
    pub ncm_id: Option<String>,
    pub cfop_id: Option<String>,
    pub cst_csosn_id: Option<String>,
    pub iva_id: Option<String>,
    pub aliquota_icms_escala6: i64,
    pub aliquota_pis_escala6: i64,
    pub aliquota_cofins_escala6: i64,
    pub aliquota_iva_escala6: i64,
    pub reducao_base_escala6: i64,
    pub prioridade: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegraTributariaMestreResp {
    pub id: String,
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub pais_fiscal: String,
    pub tipo_operacao: String,
    pub uf_origem: Option<String>,
    pub uf_destino: Option<String>,
    pub ncm_id: Option<String>,
    pub cfop_id: Option<String>,
    pub cst_csosn_id: Option<String>,
    pub iva_id: Option<String>,
    pub aliquota_icms_escala6: i64,
    pub aliquota_pis_escala6: i64,
    pub aliquota_cofins_escala6: i64,
    pub aliquota_iva_escala6: i64,
    pub reducao_base_escala6: i64,
    pub prioridade: i32,
    pub ativo: bool,
}

// --- Versionamento ---
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersaoFiscalReq {
    pub versao: String,
    pub pais_fiscal: Option<String>,
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub observacao: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersaoFiscalResp {
    pub id: String,
    pub versao: String,
    pub pais_fiscal: Option<String>,
    pub empresa_id: Option<String>,
    pub filial_id: Option<String>,
    pub status: String,
    pub total_registros: i32,
    pub publicado_em: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersaoFiscalItemResp {
    pub id: String,
    pub versao_id: String,
    pub tipo_dado: String,
    pub registro_id: Option<String>,
    pub operacao: String,
}

// --- Auditoria ---
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditoriaMestreResp {
    pub id: String,
    pub entidade: String,
    pub entidade_id: Option<String>,
    pub acao: String,
    pub usuario_id: Option<String>,
    pub criado_em: String,
}

// --- Licenciamento Local (Fase 20) ---
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicencaStatusResp {
    pub installation_id: String,
    pub empresa_id: Option<String>,
    pub terminal_id: Option<String>,
    pub terminal_nome: Option<String>,
    pub plano_codigo: String,
    pub status: String,
    pub modo: String,
    pub validade_fim: Option<String>,
    pub dias_restantes: Option<i32>,
    pub tolerancia_offline_dias: i32,
    pub bloqueio_total: i32,
    pub motivo_bloqueio: Option<String>,
    pub pode_operar: bool,
    pub mensagem: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AtivarLicencaReq {
    pub empresa_id: String,
    pub terminal_nome: String,
    pub modo: String,
}

// ================================================================
// DTOs DE LICENCIAMENTO MESTRE (RETAGUARDA)
// ================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicPlanoDto {
    pub id: String,
    pub codigo: String,
    pub nome: String,
    pub descricao: Option<String>,
    pub max_empresas: i32,
    pub max_terminais: i32,
    pub permite_pdv: bool,
    pub permite_retaguarda: bool,
    pub permite_delivery: bool,
    pub permite_gourmet: bool,
    pub permite_fiscal: bool,
    pub ativo: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CriarLicPlanoReq {
    pub codigo: String,
    pub nome: String,
    pub descricao: Option<String>,
    pub max_empresas: i32,
    pub max_terminais: i32,
    pub permite_pdv: bool,
    pub permite_retaguarda: bool,
    pub permite_delivery: bool,
    pub permite_gourmet: bool,
    pub permite_fiscal: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicEmpresaDto {
    pub id: String,
    pub empresa_id: String,
    pub nome_empresa: String,
    pub documento: Option<String>,
    pub pais: Option<String>,
    pub status: String,
    pub plano_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CriarLicEmpresaReq {
    pub empresa_id: String,
    pub nome_empresa: String,
    pub documento: Option<String>,
    pub pais: Option<String>,
    pub status: String,
    pub plano_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AtualizarStatusReq {
    pub status: String,
    pub motivo: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicLicencaDto {
    pub id: String,
    pub empresa_licenciada_id: String,
    pub plano_id: String,
    pub status: String,
    pub modo: String,
    pub validade_inicio: Option<String>,
    pub validade_fim: Option<String>,
    pub tolerancia_offline_dias: i32,
    pub bloqueio_total: bool,
    pub motivo_bloqueio: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CriarLicLicencaReq {
    pub empresa_licenciada_id: String,
    pub plano_id: String,
    pub modo: String,
    pub validade_fim: Option<String>,
    pub tolerancia_offline_dias: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicTerminalDto {
    pub id: String,
    pub licenca_id: String,
    pub installation_id: Option<String>,
    pub terminal_id: Option<String>,
    pub terminal_nome: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AutorizarTerminalReq {
    pub installation_id: Option<String>,
    pub terminal_id: Option<String>,
    pub terminal_nome: Option<String>,
    pub dispositivo_hash: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicEventoDto {
    pub id: String,
    pub empresa_licenciada_id: Option<String>,
    pub licenca_id: Option<String>,
    pub terminal_id: Option<String>,
    pub tipo_evento: String,
    pub mensagem: Option<String>,
    pub criado_em: String,
}

// ================================================================

// ================================================================
// DTOs FASE 20 - BLOCO 3: CHECK-IN DE LICENCA (PDV -> NUVEM)
// ================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicencaCheckInReq {
    pub installation_id: String,
    pub empresa_id: String,
    pub terminal_id: Option<String>,
    pub terminal_nome: String,
    pub dispositivo_hash: String,
    pub app_versao: String,
    pub sistema_operacional: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicencaPayloadResp {
    pub sucesso: bool,
    pub pode_operar: bool,
    pub status: String,
    pub modo: String,
    pub empresa_id: String,
    pub licenca_id: Option<String>,
    pub plano_codigo: Option<String>,
    pub terminal_id: Option<String>,
    pub terminal_status: Option<String>,
    pub validade_inicio: Option<String>,
    pub validade_fim: Option<String>,
    pub tolerancia_offline_dias: i32,
    pub bloqueio_total: bool,
    pub motivo_bloqueio: Option<String>,
    pub ultimo_check_em: Option<String>,
    pub assinatura_licenca: Option<String>,
    pub payload_licenca_json: Option<serde_json::Value>,
    pub mensagem: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidarTerminalReq {
    pub licenca_id: String,
    pub installation_id: String,
    pub terminal_id: Option<String>,
    pub terminal_nome: String,
    pub dispositivo_hash: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidarTerminalResp {
    pub sucesso: bool,
    pub terminal_id: Option<String>,
    pub status: String,
    pub autorizado: bool,
    pub mensagem: Option<String>,
    pub warnings: Vec<String>,
}

// ================================================================
// DTOs FASE 20 - BLOCO 4: ASSINATURA CRIPTOGRAFICA DE LICENCA
// ================================================================

/// Resposta do endpoint GET /licenciamento/licencas/{id}/payload-assinado
/// Retorna o payload de licenca com assinatura Ed25519 real.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicencaPayloadAssinadoResp {
    /// Indica se a assinatura foi gerada com sucesso
    pub sucesso: bool,
    /// Payload canonico da licenca em formato JSON deterministico (string)
    pub payload_licenca_json: String,
    /// Algoritmo utilizado: Ed25519
    pub algoritmo_assinatura: String,
    /// Identificador da chave publica usada (rotacao futura)
    pub key_id: String,
    /// Assinatura do payload_hash em base64 (64 bytes Ed25519)
    pub assinatura_licenca: String,
    /// SHA-256 do payload_licenca_json em hex
    pub payload_hash: String,
    /// Timestamp ISO-8601 de emissao do payload
    pub emitido_em: String,
    /// Mensagem descritiva da operacao
    pub mensagem: Option<String>,
    /// Avisos nao bloqueantes (ex: chave dev em uso)
    pub warnings: Vec<String>,
}

/// Requisicao do endpoint POST /licenciamento/licencas/verificar-payload
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerificarLicencaPayloadReq {
    /// Payload canonico recebido (string JSON deterministica)
    pub payload_licenca_json: String,
    /// Algoritmo declarado pelo emissor: Ed25519
    pub algoritmo_assinatura: String,
    /// Identificador da chave usada na assinatura
    pub key_id: String,
    /// Assinatura em base64
    pub assinatura_licenca: String,
    /// Hash SHA-256 do payload (opcional - verificado se presente)
    pub payload_hash: Option<String>,
}

/// Resposta do endpoint POST /licenciamento/licencas/verificar-payload
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerificarLicencaPayloadResp {
    /// true = payload autentico e nao adulterado
    pub valido: bool,
    /// Algoritmo declarado na verificacao
    pub algoritmo_assinatura: String,
    /// key_id utilizado
    pub key_id: String,
    /// Hash SHA-256 calculado localmente
    pub payload_hash: String,
    /// Mensagem de resultado (sucesso ou erro)
    pub mensagem: String,
    /// Avisos nao bloqueantes
    pub warnings: Vec<String>,
}

/// Resposta do endpoint GET /licenciamento/chaves/status
/// Expoe apenas informacoes publicas - NUNCA a chave privada.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatusChavesResp {
    /// Modo de chave ativo: DEV | PRODUCAO
    pub modo_chave: String,
    /// Identificador da chave ativa
    pub key_id: String,
    /// Chave publica em base64 (32 bytes Ed25519)
    pub chave_publica_base64: String,
    /// Algoritmo registrado
    pub algoritmo: String,
    /// Aviso de seguranca se modo DEV
    pub warnings: Vec<String>,
}

// ================================================================
// DTOs FASE 20 - BLOCO 5: APLICACAO LOCAL DE LICENCA ASSINADA (PDV)
// ================================================================

/// Requisicao para verificar payload assinado no PDV (sem aplicar).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerificarLicencaAssinadaReq {
    /// Payload canonico da licenca (string JSON)
    pub payload_licenca_json: String,
    /// Algoritmo: Ed25519
    pub algoritmo_assinatura: String,
    /// Identificador da chave usada
    pub key_id: String,
    /// Assinatura em base64
    pub assinatura_licenca: String,
    /// Hash SHA-256 do payload (opcional)
    pub payload_hash: Option<String>,
    /// Chave publica Ed25519 em base64 (obrigatorio para verificacao)
    pub chave_publica_base64: String,
}

/// Resposta da verificacao local de payload assinado.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerificarLicencaAssinadaResp {
    /// true = assinatura valida e payload integro
    pub valido: bool,
    /// Hash SHA-256 calculado localmente
    pub payload_hash_calculado: String,
    /// key_id verificado
    pub key_id: String,
    /// Mensagem de resultado
    pub mensagem: String,
    /// Avisos nao bloqueantes
    pub warnings: Vec<String>,
}

/// Requisicao para aplicar payload assinado no PDV (atualiza licenca_local).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AplicarLicencaAssinadaReq {
    /// Payload canonico da licenca (string JSON)
    pub payload_licenca_json: String,
    /// Algoritmo: Ed25519
    pub algoritmo_assinatura: String,
    /// Identificador da chave usada
    pub key_id: String,
    /// Assinatura em base64
    pub assinatura_licenca: String,
    /// Hash SHA-256 do payload
    pub payload_hash: Option<String>,
    /// Chave publica Ed25519 em base64 (obrigatorio)
    pub chave_publica_base64: String,
}

/// Resposta da aplicacao local de payload assinado.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AplicarLicencaAssinadaResp {
    /// true = payload valido e aplicado com sucesso
    pub sucesso: bool,
    /// true = assinatura verificada com sucesso
    pub assinatura_valida: bool,
    /// Status da licenca apos aplicacao
    pub status: String,
    /// Modo da licenca (DEV | ATIVA | etc.)
    pub modo: String,
    /// Empresa da licenca
    pub empresa_id: String,
    /// ID da licenca
    pub licenca_id: String,
    /// Codigo do plano
    pub plano_codigo: String,
    /// Terminal registrado
    pub terminal_id: Option<String>,
    /// Data de validade (ISO-8601 ou null)
    pub validade_fim: Option<String>,
    /// Tolerancia offline em dias
    pub tolerancia_offline_dias: i64,
    /// true = pode operar
    pub pode_operar: bool,
    /// Mensagem descritiva
    pub mensagem: String,
    /// Avisos nao bloqueantes
    pub warnings: Vec<String>,
}

// ================================================================
// DTOs FASE 20 - BLOCO 6: SINCRONIZACAO ONLINE DE LICENCA
// ================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SincronizarLicencaReq {
    pub url_retaguarda: String,
    pub empresa_id: Option<String>,
    pub forcar_checkin: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SincronizarLicencaResp {
    pub sucesso: bool,
    pub online: bool,
    pub checkin_realizado: bool,
    pub assinatura_valida: bool,
    pub aplicado_localmente: bool,
    pub status: String,
    pub modo: String,
    pub empresa_id: String,
    pub licenca_id: String,
    pub plano_codigo: String,
    pub terminal_id: Option<String>,
    pub validade_fim: Option<String>,
    pub pode_operar: bool,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigLicenciamentoReq {
    pub url_retaguarda: String,
    pub chave_publica_base64: Option<String>,
    pub key_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigLicenciamentoResp {
    pub sucesso: bool,
    pub url_retaguarda: String,
    pub key_id: String,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

// ================================================================
// DTOs FASE 20 - BLOCO 7: POLITICA DE BLOQUEIO SUAVE E ALERTAS
// ================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicencaPoliticaResp {
    pub nivel: String, // OK | ALERTA_VENCIMENTO | TOLERANCIA_OFFLINE | EXPIRADA | BLOQUEADA | MODO_DEV | SEM_LICENCA
    pub pode_operar: bool,
    pub deve_exibir_alerta: bool,
    pub deve_sincronizar: bool,
    pub dias_restantes: Option<i64>,
    pub dias_desde_ultimo_check: i64,
    pub tolerancia_offline_dias: i64,
    pub status: String,
    pub modo: String,
    pub bloqueio_total: i32,
    pub motivo_bloqueio: Option<String>,
    pub mensagem: String,
    pub acoes_recomendadas: Vec<String>,
    pub warnings: Vec<String>,
}

// ================================================================
// DTOs FASE 20 - BLOCO 8: GUARDA OPERACIONAL DE LICENCA
// ================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerificarOperacaoLicencaReq {
    pub operacao: String,
    pub contexto_id: Option<String>,
    pub origem: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerificarOperacaoLicencaResp {
    pub permitido: bool,
    pub nivel: String,
    pub status: String,
    pub modo: String,
    pub operacao: String,
    pub mensagem: String,
    pub motivo_bloqueio: Option<String>,
    pub acoes_recomendadas: Vec<String>,
    pub warnings: Vec<String>,
}

// ================================================================
// DTOs FASE 20 - BLOCO 9: BACKUP E RESTAURACAO LOCAL
// ================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CriarBackupReq {
    pub destino_dir: Option<String>,
    pub motivo: Option<String>,
    pub incluir_metadados: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupResp {
    pub sucesso: bool,
    pub backup_id: String,
    pub arquivo: String,
    pub metadados_arquivo: Option<String>,
    pub tamanho_bytes: u64,
    pub sha256: String,
    pub criado_em: String,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupInfoResp {
    pub backup_id: String,
    pub arquivo: String,
    pub metadados_arquivo: Option<String>,
    pub tamanho_bytes: u64,
    pub sha256: String,
    pub criado_em: String,
    pub empresa_id: Option<String>,
    pub installation_id: Option<String>,
    pub terminal_id: Option<String>,
    pub app_versao: Option<String>,
    pub valido: Option<bool>,
    pub mensagem: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidarBackupReq {
    pub arquivo: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidarBackupResp {
    pub valido: bool,
    pub arquivo: String,
    pub tamanho_bytes: u64,
    pub sha256: String,
    pub sqlite_integrity_ok: bool,
    pub migrations_ok: bool,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RestaurarBackupReq {
    pub arquivo: String,
    pub confirmacao_texto: String,
    pub motivo: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RestaurarBackupResp {
    pub sucesso: bool,
    pub backup_restaurado: String,
    pub backup_pre_restauracao: Option<String>,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticoBancoResp {
    pub sqlite_integrity_ok: bool,
    pub tamanho_bytes: u64,
    pub caminho_banco: String,
    pub migrations_count: i64,
    pub ultima_migration: Option<String>,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

// ==========================================
// DTOs de Diagnostico de Sistema (Fase 20, Bloco 10)
// ==========================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiagnosticoSistemaResp {
    pub sucesso: bool,
    pub sistema_operacional: String,
    pub arquitetura: String,
    pub app_versao: String,
    pub caminho_base: String,
    pub caminho_banco: String,
    pub caminho_backups: String,
    pub caminho_logs: String,
    pub caminho_print_sim: String,
    pub espaco_livre_bytes: Option<u64>,
    pub pode_escrever_base: bool,
    pub pode_escrever_backups: bool,
    pub banco_existe: bool,
    pub pastas_ok: bool,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

// ==========================================
// DTOs de Seguranca Operacional (Fase 21, Bloco 1)
// ==========================================

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct LoginLocalReq {
    pub login: String,
    pub senha_pura: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct LoginLocalResp {
    pub sucesso: bool,
    pub usuario_id: Option<String>,
    pub login: Option<String>,
    pub nome: Option<String>,
    pub sessao_id: Option<String>,
    pub perfis: Vec<String>,
    pub permissoes: Vec<String>,
    pub exige_troca_senha: bool,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct SessaoUsuarioResp {
    pub autenticado: bool,
    pub usuario_id: Option<String>,
    pub login: Option<String>,
    pub nome: Option<String>,
    pub sessao_id: Option<String>,
    pub perfis: Vec<String>,
    pub permissoes: Vec<String>,
    pub aberta_em: Option<String>,
    pub mensagem: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct UsuarioLocalResp {
    pub id: String,
    pub nome: String,
    pub login: String,
    pub ativo: bool,
    pub perfis: Vec<String>,
    pub ultimo_login_em: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct PerfilLocalResp {
    pub id: String,
    pub codigo: String,
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct PermissaoLocalResp {
    pub id: String,
    pub codigo: String,
    pub modulo: String,
    pub acao: String,
    pub descricao: Option<String>,
    pub risco: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct UsuarioTemPermissaoReq {
    pub usuario_id: Option<String>,
    pub permissao_codigo: String,
    pub modulo: Option<String>,
    pub acao: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct UsuarioTemPermissaoResp {
    pub permitido: bool,
    pub usuario_id: Option<String>,
    pub permissao_codigo: String,
    pub mensagem: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct VerificarPermissaoOperacaoReq {
    pub permissao_codigo: String,
    pub modulo: Option<String>,
    pub acao: Option<String>,
    pub contexto_id: Option<String>,
    pub origem: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct VerificarPermissaoOperacaoResp {
    pub permitido: bool,
    pub usuario_id: Option<String>,
    pub login: Option<String>,
    pub permissao_codigo: String,
    pub mensagem: String,
    pub motivo_negacao: Option<String>,
    pub warnings: Vec<String>,
}
