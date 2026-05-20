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

