using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace AureonPdvUi.Services
{
    // === DTOs de Caixa ===

    public record SaldoMoedaResp(
        string MoedaCodigo,
        long ValorAberturaMinor,
        long? ValorFechamentoInformadoMinor,
        long? ValorEsperadoMinor,
        long? DiferencaMinor
    );

    public record SessaoCaixaResp(
        string Id,
        string RegistradoraId,
        string UsuarioId,
        string Status,
        string AbertoEm,
        string? FechadoEm,
        string? Observacao,
        List<SaldoMoedaResp> Saldos
    );

    public record SaldoMoedaReq(
        string moeda_codigo,
        long valor_minor
    );

    public record AbrirCaixaReq(
        string registradora_id,
        string usuario_id,
        List<SaldoMoedaReq> saldos_abertura
    );

    public record FecharCaixaReq(
        string sessao_id,
        string usuario_id,
        List<SaldoMoedaReq> saldos_fechamento,
        string? observacao
    );

    // === DTOs de Venda ===

    public record VendaResumoResp(
        string Id,
        long? NumeroVenda,
        string Status,
        string TipoVenda,
        long SubtotalMinor,
        long DescontoTotalMinor,
        long AcrescimoTotalMinor,
        long TotalMinor,
        long TotalItens
    );

    public record ProdutoPdvResp(
        string ProdutoId,
        string Codigo,
        string? CodigoBarras,
        string Nome,
        string UnidadeMedida,
        long PrecoVendaMinor,
        bool Ativo
    );

    public record VendaItemResp(
        string Id,
        string VendaId,
        string ProdutoId,
        string DescricaoProduto,
        string? CodigoProduto,
        long QuantidadeEscala3,
        long PrecoUnitarioMinor,
        long DescontoItemMinor,
        long TotalItemMinor,
        bool Cancelado,
        string? CanceladoEm,
        string? MotivoCancelamento,
        string CriadoEm
    );

    public record VendaDetalheResp(
        VendaResumoResp Venda,
        List<VendaItemResp> Itens
    );

    public record AdicionarItemReq(
        string venda_id,
        string produto_id,
        string descricao_produto,
        string? codigo_produto,
        string? codigo_barras,
        long quantidade_escala3,
        long preco_unitario_minor,
        long desconto_item_minor
    );

    public record CancelarItemReq(
        string item_id,
        string usuario_cancelamento_id,
        string motivo_cancelamento,
        string? supervisor_id,
        string? autorizacao_id
    );

    public record CancelarVendaReq(
        string venda_id,
        string usuario_cancelamento_id,
        string motivo_cancelamento,
        string? supervisor_id,
        string? autorizacao_id
    );

    // === DTOs de Pagamento ===

    public record PagamentoResp(
        string Id,
        string VendaId,
        string FormaPagamento,
        string MoedaCodigo,
        long ValorInformadoMinor,
        string MoedaPrincipalCodigo,
        long ValorConvertidoMinor,
        long TaxaCambioEscala6,
        string DataCotacaoUsada,
        long TrocoMinor,
        string? MoedaTrocoCodigo,
        string CriadoEm
    );

    public record RegistrarPagamentoReq(
        string venda_id,
        string forma_pagamento,
        string moeda_codigo,
        long valor_informado_minor,
        string? moeda_troco_codigo
    );

    public record TrocoResp(
        long TotalVendaMinor,
        long TotalPagoMinor,
        long TrocoMinor,
        bool Quitado
    );

    // === DTOs da Fase 8 (Operacional, Supervisor, Reimpressão, Pré-Venda, Orçamento, Cliente) ===

    public record ResumoCaixaResp(
        string MoedaCodigo,
        long AberturaMinor,
        long VendasMinor,
        long SuprimentosMinor,
        long SangriasMinor,
        long ValesFuncionariosMinor,
        long EsperadoMinor
    );

    public record CaixaMovimentacaoReq(
        string sessao_caixa_id,
        string usuario_id,
        string tipo_movimentacao, // SANGRIA, SUPRIMENTO, VALE_FUNCIONARIO
        string moeda_codigo,
        long valor_minor,
        string? motivo,
        string? funcionario_id,
        string? supervisor_id,
        string? autorizacao_id
    );

    public record CaixaMovimentacaoResp(
        string Id,
        string SessaoCaixaId,
        string UsuarioId,
        string TipoMovimentacao,
        string MoedaCodigo,
        long ValorMinor,
        string? Motivo,
        string? FuncionarioId,
        string? SupervisorId,
        string? AutorizacaoId,
        bool Cancelado,
        string? CanceladoEm,
        string? UsuarioCancelamentoId,
        string? MotivoCancelamento,
        string CriadoEm
    );

    public record CancelarMovimentacaoReq(
        string movimentacao_id,
        string usuario_cancelamento_id,
        string motivo_cancelamento,
        string? supervisor_id,
        string? autorizacao_id
    );

    public record SolicitarAutorizacaoReq(
        string pin_supervisor,
        string operacao,
        string usuario_solicitante_id,
        string sessao_caixa_id,
        string? entidade_tipo,
        string? entidade_id,
        string? motivo
    );

    public record AutorizacaoResp(
        string Id,
        string Operacao,
        string UsuarioSolicitanteId,
        string SupervisorId,
        bool Aprovado,
        string? Motivo,
        string CriadoEm
    );

    public record ReimpressaoReq(
        string venda_id,
        string usuario_id,
        string? motivo,
        string? supervisor_id
    );

    public record PreVendaItemResp(
        string Id,
        string PreVendaId,
        string ProdutoId,
        string Descricao,
        long QuantidadeEscala3,
        long PrecoUnitarioMinor,
        long DescontoMinor,
        long TotalMinor
    );

    public record PreVendaResp(
        string Id,
        string Numero,
        string? ClienteId,
        string? VendedorId,
        long TotalMinor,
        string Status,
        string? Validade,
        string CriadoEm,
        List<PreVendaItemResp> Itens
    );

    public record ConverterPreVendaReq(
        string pre_venda_id,
        string sessao_caixa_id,
        string usuario_id
    );

    public record OrcamentoItemResp(
        string Id,
        string OrcamentoId,
        string ProdutoId,
        string Descricao,
        long QuantidadeEscala3,
        long PrecoUnitarioMinor,
        long DescontoMinor,
        long TotalMinor
    );

    public record OrcamentoResp(
        string Id,
        string Numero,
        string? ClienteId,
        string? VendedorId,
        long TotalMinor,
        string Status,
        string? Validade,
        string CriadoEm,
        List<OrcamentoItemResp> Itens
    );

    public record ConverterOrcamentoReq(
        string orcamento_id,
        string sessao_caixa_id,
        string usuario_id
    );

    public record ClienteResp(
        string Id,
        string Nome,
        string? Documento,
        bool Ativo
    );

    public record AssociarClienteReq(
        string venda_id,
        string cliente_id
    );

    // === DTOs de PDV Gourmet (Fase 9 Bloco 2) ===

    public record MesaOperacionalResp(
        string? Id,
        string MesaId,
        int MesaNumero,
        string NomeExibicao,
        string? ClienteNomeInformal,
        string? ClienteId,
        string Status,
        string? UsuarioAberturaId,
        string? SessaoCaixaId,
        string? Observacao,
        string? AbertaEm,
        long TotalConsumoMinor
    );

    public record AbrirMesaReq(
        int mesa_numero,
        string nome_exibicao,
        string? cliente_nome_informal,
        string? cliente_id,
        string usuario_id,
        string sessao_caixa_id,
        string? observacao
    );

    public record ReservarMesaReq(
        int mesa_numero,
        string nome_exibicao,
        string? cliente_nome_informal,
        string? cliente_id,
        string usuario_id,
        string sessao_caixa_id,
        string? observacao
    );

    public record BloquearMesaReq(
        int mesa_numero,
        string usuario_id,
        string sessao_caixa_id
    );

    public record CancelarMesaReq(
        int mesa_numero,
        string usuario_cancelamento_id,
        string motivo_cancelamento,
        string? supervisor_id,
        string? autorizacao_id
    );

    public record ComandaOperacionalResp(
        string? Id,
        string ComandaId,
        int NumeroComanda,
        string? CodigoBarrasQr,
        string? ClienteNomeInformal,
        string? ClienteId,
        string Status,
        string? UsuarioAberturaId,
        string? SessaoCaixaId,
        string? Observacao,
        string? AbertaEm,
        long TotalConsumoMinor
    );

    public record AbrirComandaReq(
        int numero_comanda,
        string? codigo_barras_qr,
        string? cliente_nome_informal,
        string? cliente_id,
        string usuario_id,
        string sessao_caixa_id,
        string? observacao
    );

    public record BloquearComandaReq(
        int numero_comanda,
        string usuario_id,
        string sessao_caixa_id
    );

    public record CancelarComandaReq(
        int numero_comanda,
        string usuario_cancelamento_id,
        string motivo_cancelamento,
        string? supervisor_id,
        string? autorizacao_id
    );

    public record GourmetItemResp(
        string Id,
        string OrigemTipo,
        string OrigemId,
        string ProdutoId,
        string DescricaoProduto,
        string CodigoProduto,
        long QuantidadeEscala3,
        long PrecoUnitarioMinor,
        long DescontoItemMinor,
        long AcrescimoItemMinor,
        long TotalItemMinor,
        string? ObservacaoProducao,
        string LocalProducaoId,
        string Status,
        bool EnviadoProducao,
        string? EnviadoProducaoEm,
        bool Cancelado,
        string? CanceladoEm,
        string? MotivoCancelamento,
        string? SupervisorId,
        string? AutorizacaoId,
        string CriadoEm
    );

    public record AdicionarItemGourmetReq(
        string origem_tipo,
        string origem_id,
        string produto_id,
        long quantidade_escala3,
        long preco_unitario_minor,
        long desconto_item_minor,
        long acrescimo_item_minor,
        string? observacao_producao,
        string local_producao_id
    );

    public record CancelarItemGourmetReq(
        string item_id,
        string usuario_cancelamento_id,
        string motivo_cancelamento,
        string? supervisor_id,
        string? autorizacao_id
    );

    public record MesaDetalheResp(
        MesaOperacionalResp Mesa,
        List<GourmetItemResp> Itens
    );

    public record ComandaDetalheResp(
        ComandaOperacionalResp Comanda,
        List<GourmetItemResp> Itens
    );

    // ========================================================================
    // BLOCO 3: TRANSFERÊNCIAS, PRODUÇÃO E FECHAMENTO
    // ========================================================================

    public record TransferirTotalReq(
        string origem_id,
        string destino_id,
        string usuario_id,
        string motivo
    );

    public record TransferirItensReq(
        string origem_id,
        string destino_id,
        List<string> itens_ids,
        string usuario_id,
        string motivo
    );

    public record EnviarProducaoReq(
        string origem_tipo,
        string origem_id,
        string usuario_id
    );

    public record ProducaoEnvioResp(
        string Id,
        string OrigemTipo,
        string OrigemId,
        string SetorProducaoId,
        string Status,
        string CriadoEm
    );

    public record FecharEmVendaReq(
        string origem_id,
        string sessao_caixa_id,
        string usuario_id,
        string origem_tipo = "MESA"
    );

    public record FechamentoEmVendaResp(
        string VendaId,
        string OrigemTipo,
        string OrigemId,
        long TotalMinor,
        long TotalItens,
        string StatusVenda
    );

    // ========================================================================
    // DELIVERY OPERACIONAL — DTOs
    // ========================================================================

    public record DeliveryItemResp(
        string Id,
        string DeliveryId,
        string ProdutoId,
        string DescricaoProduto,
        string? CodigoProduto,
        long QuantidadeEscala3,
        long PrecoUnitarioMinor,
        long DescontoItemMinor,
        long AcrescimoItemMinor,
        long TotalItemMinor,
        string? ObservacaoProducao,
        string? LocalProducaoId,
        string Status,
        bool EnviadoProducao,
        bool Cancelado,
        string CriadoEm = ""
    );

    public record DeliveryOperacionalResp(
        string Id,
        int NumeroPedido,
        string? ClienteId,
        string NomeClienteInformal,
        string Telefone,
        string? EnderecoCompleto,
        string TipoPedido,
        string Status,
        string Origem,
        string? EntregadorId,
        long TaxaEntregaMinor,
        long TotalConsumoMinor,
        string? SessaoCaixaId,
        string? Observacao,
        string? PrevisaoEntrega,
        string AbertaEm,
        string? FechadoEm,
        List<DeliveryItemResp> Itens
    );

    public record EntregadorResp(
        string Id,
        string Nome,
        string? Documento,
        bool Ativo
    );

    public record CriarPedidoLocalReq(
        string nome_cliente_informal,
        string telefone,
        string tipo_pedido,
        string? endereco_completo,
        long taxa_entrega_minor,
        string sessao_caixa_id,
        string? observacao,
        string? cliente_id,
        string usuario_id
    );

    public record AceitarPedidoReq(
        string delivery_id,
        string sessao_caixa_id
    );

    public record RecusarPedidoReq(
        string delivery_id,
        string motivo,
        string usuario_id
    );

    public record AtualizarStatusDeliveryReq(
        string delivery_id,
        string novo_status,
        string usuario_id
    );

    public record DefinirEntregadorReq(
        string delivery_id,
        string entregador_id,
        string usuario_id
    );

    public record AdicionarItemDeliveryReq(
        string delivery_id,
        string produto_id,
        string descricao_produto,
        string? codigo_produto,
        long quantidade_escala3,
        long preco_unitario_minor,
        long desconto_item_minor,
        long acrescimo_item_minor,
        string? observacao_producao,
        string? local_producao_id,
        string usuario_id
    );

    public record CancelarItemDeliveryReq(
        string item_id,
        string motivo_cancelamento,
        string usuario_id,
        string? supervisor_id,
        string? autorizacao_id
    );

    // --- FASE 11: ESTOQUE OPERACIONAL ---

    public record EstoqueSaldoResp(
        string ProdutoId,
        string? Codigo,
        string Descricao,
        bool ControlaEstoque,
        long QuantidadeEscala3,
        string? AtualizadoEm
    );

    public record EstoqueMovimentacaoResp(
        string Id,
        string ProdutoId,
        long QuantidadeEscala3,
        long SaldoAposEscala3,
        string TipoMovimentacao,
        string OrigemTipo,
        string OrigemId,
        string? Motivo,
        string UsuarioId,
        string CriadoEm
    );

    public record AjusteEstoqueReq(
        string idempotency_key,
        string produto_id,
        string tipo_ajuste,
        long quantidade_escala3,
        string motivo,
        string usuario_id
    );

    public record ContagemInventario(
        string produto_id,
        long saldo_real_escala3
    );

    public record InventarioEstoqueReq(
        string idempotency_key,
        List<ContagemInventario> contagens,
        string? motivo,
        string usuario_id
    );

    // --- FASE 12: COMPRAS E ENTRADA MANUAL ---

    public record FornecedorResp(
        string Id,
        string Nome,
        string? Documento,
        bool Ativo,
        string AtualizadoEm
    );

    public record CompraItemResp(
        string Id,
        string CompraId,
        string ProdutoId,
        string DescricaoProdutoSnapshot,
        long QuantidadeEscala3,
        long CustoUnitarioMinor,
        long TotalItemMinor,
        string? Lote,
        string? Validade,
        string? Serial,
        string? Imei,
        bool Cancelado,
        string CriadoEm
    );

    public record CompraResp(
        string Id,
        string FornecedorId,
        string FornecedorNomeSnapshot,
        string? NumeroNota,
        string? Serie,
        string? ChaveAcessoXmlFiscal,
        string? DataEmissao,
        string Status,
        string MoedaCodigo,
        long TaxaCambioEscala6,
        long SubtotalItensMinor,
        long DescontoTotalMinor,
        long FreteTotalMinor,
        long OutrasDespesasMinor,
        long ImpostosTotalMinor,
        long TotalCompraMinor,
        string? Observacao,
        string CriadoEm,
        string AtualizadoEm,
        string? FinalizadaEm,
        string? CanceladaEm,
        string? MotivoCancelamento,
        string UsuarioId,
        List<CompraItemResp> Itens
    );

    public record IniciarCompraReq(
        string fornecedor_id,
        string? numero_nota,
        string? serie,
        string? chave_acesso_xml_fiscal,
        string? data_emissao,
        string moeda_codigo,
        long taxa_cambio_escala6,
        string? observacao,
        string usuario_id
    );

    public record AdicionarItemCompraReq(
        string compra_id,
        string produto_id,
        long quantidade_escala3,
        long custo_unitario_minor,
        string? lote,
        string? validade,
        string? serial,
        string? imei
    );

    public record CancelarCompraEmAndamentoReq(
        string compra_id,
        string motivo,
        string usuario_id
    );

    public record CancelarCompraFinalizadaReq(
        string compra_id,
        string motivo,
        string usuario_id
    );

    // --- FASE 13: FINANCEIRO BASE ---

    public record ContaPagarResp(
        string Id,
        string? FornecedorId,
        string? FornecedorNomeSnapshot,
        string? CompraId,
        string Descricao,
        string MoedaCodigo,
        long ValorOriginalMinor,
        long TaxaCambioEscala6,
        long ValorOriginalPrincipalMinor,
        string DataEmissao,
        string DataVencimento,
        string Status,
        long SaldoPendenteMinor,
        string CriadoEm,
        string AtualizadoEm,
        string UsuarioId,
        string? Observacao
    );

    public record ContaReceberResp(
        string Id,
        string? ClienteId,
        string? ClienteNomeSnapshot,
        string? VendaId,
        string Descricao,
        string MoedaCodigo,
        long ValorOriginalMinor,
        long TaxaCambioEscala6,
        long ValorOriginalPrincipalMinor,
        string DataEmissao,
        string DataVencimento,
        string Status,
        long SaldoPendenteMinor,
        string CriadoEm,
        string AtualizadoEm,
        string UsuarioId,
        string? Observacao
    );

    public record FinanceiroLancamentoResp(
        string Id,
        string? ContaPagarId,
        string? ContaReceberId,
        string? SessaoCaixaId,
        string TipoLancamento,
        string FormaPagamento,
        string MoedaCodigo,
        long ValorInformadoMinor,
        long TaxaCambioEscala6,
        long ValorPrincipalMinor,
        string DataPagamento,
        string UsuarioId,
        string? Observacao,
        string CriadoEm
    );

    public record RegistrarDespesaReq(
        string? fornecedor_id,
        string descricao,
        string moeda_codigo,
        long valor_original_minor,
        long taxa_cambio_escala6,
        string data_emissao,
        string data_vencimento,
        string usuario_id,
        string? observacao
    );

    public record BaixarContaPagarReq(
        string conta_pagar_id,
        string sessao_caixa_id,
        string forma_pagamento,
        string moeda_codigo,
        long valor_informado_minor,
        long taxa_cambio_escala6,
        string usuario_id,
        string? observacao
    );

    public record CancelarContaPagarReq(
        string conta_pagar_id,
        string motivo,
        string usuario_id
    );

    public record BaixarContaReceberReq(
        string conta_receber_id,
        string sessao_caixa_id,
        string forma_pagamento,
        string moeda_codigo,
        long valor_informado_minor,
        long taxa_cambio_escala6,
        string usuario_id,
        string? observacao
    );

    public record CancelarContaReceberReq(
        string conta_receber_id,
        string motivo,
        string usuario_id
    );

    // === DTOs da Fase 14 (Relatórios e Dashboard) ===

    public record TotalPorMoeda(
        string MoedaCodigo,
        long ValorMinor
    );

    public record FiltrosRelatorio(
        string? data_inicio,
        string? data_fim,
        string? usuario_id,
        string? sessao_caixa_id,
        string? moeda_codigo,
        string? forma_pagamento
    );

    public record IndicadoresDashboardResp(
        List<TotalPorMoeda> FaturamentoPorMoeda,
        List<TotalPorMoeda> DespesasPorMoeda,
        long TotalVendasQuantidade,
        long TotalVendasItensQuantidadeEscala3,
        long ProdutosEstoqueCritico,
        List<TotalPorMoeda> ContasPagarVencidasPorMoeda,
        List<TotalPorMoeda> ContasPagarAVencerPorMoeda,
        List<TotalPorMoeda> ContasReceberVencidasPorMoeda,
        List<TotalPorMoeda> ContasReceberAVencerPorMoeda
    );

    public record RelatorioVendasItem(
        string Id,
        long? NumeroVenda,
        string DataVenda,
        long TotalBrutoMinor,
        long DescontoTotalMinor,
        long AcrescimoTotalMinor,
        long TotalLiquidoMinor,
        string Status,
        string? ClienteNome,
        string UsuarioId
    );

    public record VendasPorFormaPagamento(
        string FormaPagamento,
        string MoedaCodigo,
        long TotalMinor,
        long Quantidade
    );

    public record RelatorioVendasResp(
        List<RelatorioVendasItem> Vendas,
        List<TotalPorMoeda> TotaisPorMoeda,
        List<VendasPorFormaPagamento> VendasPorForma
    );

    public record RelatorioCaixaItem(
        string Id,
        string OperadorId,
        string TerminalId,
        string Status,
        string AbertoEm,
        string? FechadoEm,
        string MoedaCodigo,
        long ValorAberturaMinor,
        long ValorFechamentoEsperadoMinor,
        long ValorFechamentoInformadoMinor,
        long DiferencaMinor
    );

    public record RelatorioCaixaResp(
        List<RelatorioCaixaItem> Sessoes
    );

    public record RelatorioFinanceiroResp(
        List<ContaPagarResp> ContasPagar,
        List<ContaReceberResp> ContasReceber,
        List<FinanceiroLancamentoResp> Lancamentos,
        List<TotalPorMoeda> TotalPagarPendente,
        List<TotalPorMoeda> TotalReceberPendente
    );

    public record PosicaoEstoqueItem(
        string ProdutoId,
        string ProdutoNome,
        string ProdutoSku,
        bool ControlaEstoque,
        long QuantidadeEscala3,
        long EstoqueMinimoEscala3,
        long UltimoCustoMinor
    );

    public record EstoqueKardexItem(
        string Id,
        string ProdutoId,
        string ProdutoNome,
        string ProdutoSku,
        string TipoMovimentacao,
        long QuantidadeEscala3,
        string DataMovimentacao,
        string? OrigemId,
        string UsuarioId,
        string? Observacao
    );

    public record RelatorioEstoqueResp(
        List<PosicaoEstoqueItem> PosicaoEstoque,
        List<EstoqueKardexItem> ItensKardex,
        long CustoTotalEstimadoBrl
    );

    public record CompraRelatorioItem(
        string Id,
        string FornecedorNome,
        string DataCompra,
        string Status,
        string MoedaCodigo,
        long TotalOriginalMinor,
        long TotalPrincipalBrlMinor,
        long TotalItensEscala3
    );

    public record CompraFornecedorTotal(
        string FornecedorNome,
        long TotalPrincipalBrlMinor
    );

    public record RelatorioComprasResp(
        List<CompraRelatorioItem> Compras,
        List<CompraFornecedorTotal> TotalPorFornecedor,
        List<TotalPorMoeda> TotalPorMoeda
    );

    public record ProdutoMaisVendidoResp(
        string ProdutoId,
        string ProdutoNome,
        string ProdutoSku,
        long QuantidadeVendidaEscala3,
        long FaturamentoBrutoMinor
    );

    public record DeliveryStatusContagem(
        string Status,
        long Quantidade
    );

    public record RelatorioGourmetDeliveryResp(
        long TotalPedidosDelivery,
        List<DeliveryStatusContagem> DeliveryPorStatus,
        List<TotalPorMoeda> FaturamentoDeliveryMoeda,
        long TaxaEntregaTotalMinor,
        long TotalAtendimentosGourmet,
        List<TotalPorMoeda> FaturamentoGourmetMoeda,
        long TicketMedioGourmetBrlMinor
    );

    // ==========================================
    // DTOs de Impressão Operacional (Fase 15)
    // ==========================================

    [JsonConverter(typeof(JsonStringEnumConverter))]
    public enum TipoDestinoImpressao
    {
        TCP_IP,
        WINDOWS_RAW,
        SIMULADOR
    }

    public class ImpressoraDestinoReq
    {
        [JsonPropertyName("impressora_id")]
        public string? ImpressoraId { get; set; }

        [JsonPropertyName("nome")]
        public string Nome { get; set; } = string.Empty;

        [JsonPropertyName("tipo_destino")]
        public TipoDestinoImpressao TipoDestino { get; set; } = TipoDestinoImpressao.SIMULADOR;

        [JsonPropertyName("endereco_ip")]
        public string? EnderecoIp { get; set; }

        [JsonPropertyName("porta")]
        public ushort? Porta { get; set; }

        [JsonPropertyName("nome_spooler")]
        public string? NomeSpooler { get; set; }

        [JsonPropertyName("caminho_simulador")]
        public string? CaminhoSimulador { get; set; }

        [JsonPropertyName("largura_colunas")]
        public byte LarguraColunas { get; set; } = 48;

        [JsonPropertyName("cortar_papel")]
        public bool CortarPapel { get; set; } = true;

        [JsonPropertyName("abrir_gaveta")]
        public bool AbrirGaveta { get; set; } = false;
    }

    public record TesteImpressoraReq
    {
        [JsonPropertyName("destino")]
        public ImpressoraDestinoReq Destino { get; init; } = new();

        [JsonPropertyName("texto_teste")]
        public string? TextoTeste { get; init; }

        [JsonPropertyName("usuario_id")]
        public string? UsuarioId { get; init; }
    }

    public record ImpressaoResultadoResp
    {
        [JsonPropertyName("sucesso")]
        public bool Sucesso { get; init; }

        [JsonPropertyName("mensagem")]
        public string Mensagem { get; init; } = string.Empty;

        [JsonPropertyName("destino_usado")]
        public string DestinoUsado { get; init; } = string.Empty;

        [JsonPropertyName("caminho_arquivo_simulado")]
        public string? CaminhoArquivoSimulado { get; init; }

        [JsonPropertyName("bytes_gerados")]
        public int BytesGerados { get; init; }
    }
    public record ImprimirVendaReq
    {
        [JsonPropertyName("venda_id")]
        public string VendaId { get; init; } = string.Empty;

        [JsonPropertyName("destino")]
        public ImpressoraDestinoReq Destino { get; init; } = new();

        [JsonPropertyName("usuario_id")]
        public string? UsuarioId { get; init; }

        [JsonPropertyName("numero_via")]
        public int? NumeroVia { get; init; }

        [JsonPropertyName("imprimir_itens_cancelados")]
        public bool ImprimirItensCancelados { get; init; }
    }

    public record ReimprimirVendaReq
    {
        [JsonPropertyName("venda_id")]
        public string VendaId { get; init; } = string.Empty;

        [JsonPropertyName("destino")]
        public ImpressoraDestinoReq Destino { get; init; } = new();

        [JsonPropertyName("usuario_id")]
        public string? UsuarioId { get; init; }

        [JsonPropertyName("motivo_reimpressao")]
        public string MotivoReimpressao { get; init; } = string.Empty;
    }

    public record ImprimirBaixaFinanceiraReq
    {
        [JsonPropertyName("lancamento_id")]
        public string LancamentoId { get; init; } = string.Empty;

        [JsonPropertyName("destino")]
        public ImpressoraDestinoReq Destino { get; init; } = new();

        [JsonPropertyName("usuario_id")]
        public string? UsuarioId { get; init; }

        [JsonPropertyName("numero_via")]
        public int? NumeroVia { get; init; }
    }

    // --- DTOs Fase 15 Bloco 3: Comprovantes de Caixa ---

    public record ImprimirMovimentacaoCaixaReq
    {
        [JsonPropertyName("movimentacao_id")]
        public string MovimentacaoId { get; init; } = string.Empty;

        [JsonPropertyName("destino")]
        public ImpressoraDestinoReq Destino { get; init; } = new();

        [JsonPropertyName("usuario_id")]
        public string? UsuarioId { get; init; }

        [JsonPropertyName("numero_via")]
        public int? NumeroVia { get; init; }
    }

    public record ImprimirSessaoCaixaReq
    {
        [JsonPropertyName("sessao_caixa_id")]
        public string SessaoCaixaId { get; init; } = string.Empty;

        [JsonPropertyName("destino")]
        public ImpressoraDestinoReq Destino { get; init; } = new();

        [JsonPropertyName("usuario_id")]
        public string? UsuarioId { get; init; }

        [JsonPropertyName("numero_via")]
        public int? NumeroVia { get; init; }
    }

    // --- DTOs Fase 15 Bloco 4: Produção, Delivery e Gaveta ---

    public record ImprimirProducaoReq
    {
        [JsonPropertyName("envio_id")]
        public string EnvioId { get; init; } = string.Empty;

        [JsonPropertyName("destino")]
        public ImpressoraDestinoReq Destino { get; init; } = new();

        [JsonPropertyName("usuario_id")]
        public string? UsuarioId { get; init; }

        [JsonPropertyName("numero_via")]
        public int? NumeroVia { get; init; }
    }

    public record ImprimirCancelamentoProducaoReq
    {
        [JsonPropertyName("origem_tipo")]
        public string OrigemTipo { get; init; } = string.Empty;

        [JsonPropertyName("origem_id")]
        public string OrigemId { get; init; } = string.Empty;

        [JsonPropertyName("item_id")]
        public string? ItemId { get; init; }

        [JsonPropertyName("destino")]
        public ImpressoraDestinoReq Destino { get; init; } = new();

        [JsonPropertyName("usuario_id")]
        public string? UsuarioId { get; init; }

        [JsonPropertyName("motivo")]
        public string Motivo { get; init; } = string.Empty;
    }

    public record ImprimirRomaneioDeliveryReq
    {
        [JsonPropertyName("delivery_id")]
        public string DeliveryId { get; init; } = string.Empty;

        [JsonPropertyName("destino")]
        public ImpressoraDestinoReq Destino { get; init; } = new();

        [JsonPropertyName("usuario_id")]
        public string? UsuarioId { get; init; }

        [JsonPropertyName("numero_via")]
        public int? NumeroVia { get; init; }
    }

    public record AbrirGavetaReq
    {
        [JsonPropertyName("destino")]
        public ImpressoraDestinoReq Destino { get; init; } = new();

        [JsonPropertyName("usuario_id")]
        public string? UsuarioId { get; init; }

        [JsonPropertyName("motivo")]
        public string? Motivo { get; init; }
    }
    // === DTOs Fiscais (Fase 16 Bloco 2) ===

    public class FiscalEmpresaConfigResp {
        public string Id { get; set; } = string.Empty;
        public string Pais_fiscal { get; set; } = string.Empty;
        public string? Regime_fiscal { get; set; }
        public string Ambiente { get; set; } = string.Empty;
        public string Forma_emissao { get; set; } = string.Empty;
        public string? Certificado_alias { get; set; }
        public string? Certificado_caminho { get; set; }
        public string? Configuracao_json { get; set; }
    }

    public class SalvarFiscalEmpresaConfigReq {
        public string pais_fiscal { get; set; } = string.Empty;
        public string? regime_fiscal { get; set; }
        public string ambiente { get; set; } = string.Empty;
        public string forma_emissao { get; set; } = string.Empty;
        public string? certificado_alias { get; set; }
        public string? certificado_caminho { get; set; }
    }

    public class FiscalNcmResp {
        public string Id { get; set; } = string.Empty;
        public string Codigo { get; set; } = string.Empty;
        public string? Descricao { get; set; }
        public bool Ativo { get; set; }
    }

    public class FiscalCfopResp {
        public string Id { get; set; } = string.Empty;
        public string Codigo { get; set; } = string.Empty;
        public string? Descricao { get; set; }
        public string? Tipo_operacao { get; set; }
        public bool Ativo { get; set; }
    }

    public class FiscalCstCsosnResp {
        public string Id { get; set; } = string.Empty;
        public string Codigo { get; set; } = string.Empty;
        public string Tipo { get; set; } = string.Empty;
        public string? Descricao { get; set; }
        public bool Ativo { get; set; }
    }

    public class FiscalIvaResp {
        public string Id { get; set; } = string.Empty;
        public string Codigo { get; set; } = string.Empty;
        public string? Descricao { get; set; }
        public long Aliquota_escala6 { get; set; }
        public bool Ativo { get; set; }
    }

    public class SalvarFiscalIvaReq {
        public string codigo { get; set; } = string.Empty;
        public string? descricao { get; set; }
        public long aliquota_escala6 { get; set; }
        public bool ativo { get; set; }
    }

    public class FiscalRegraTributariaResp {
        public string Id { get; set; } = string.Empty;
        public string Pais_fiscal { get; set; } = string.Empty;
        public string Tipo_operacao { get; set; } = string.Empty;
        public string? Uf_origem { get; set; }
        public string? Uf_destino { get; set; }
        public string? Ncm_id { get; set; }
        public string? Cfop_id { get; set; }
        public string? Cst_csosn_id { get; set; }
        public string? Iva_id { get; set; }
        public long Aliquota_icms_escala6 { get; set; }
        public long Aliquota_pis_escala6 { get; set; }
        public long Aliquota_cofins_escala6 { get; set; }
        public long Aliquota_iva_escala6 { get; set; }
        public long Reducao_base_escala6 { get; set; }
        public bool Ativo { get; set; }
    }

    public class SalvarFiscalRegraTributariaReq {
        public string pais_fiscal { get; set; } = string.Empty;
        public string tipo_operacao { get; set; } = string.Empty;
        public string? uf_origem { get; set; }
        public string? uf_destino { get; set; }
        public string? ncm_id { get; set; }
        public string? cfop_id { get; set; }
        public string? cst_csosn_id { get; set; }
        public string? iva_id { get; set; }
        public long aliquota_icms_escala6 { get; set; }
        public long aliquota_pis_escala6 { get; set; }
        public long aliquota_cofins_escala6 { get; set; }
        public long aliquota_iva_escala6 { get; set; }
        public long reducao_base_escala6 { get; set; }
        public bool ativo { get; set; }
    }

    public class VincularFiscalProdutoReq {
        public string produto_id { get; set; } = string.Empty;
        public string? ncm_id { get; set; }
        public string? iva_id { get; set; }
        public string? cst_csosn_id { get; set; }
        public string? cfop_padrao_id { get; set; }
        public string? origem_mercadoria { get; set; }
    }

    public class FiscalEventoLogResp {
        public string Id { get; set; } = string.Empty;
        public string? Venda_id { get; set; }
        public string Tipo_evento { get; set; } = string.Empty;
        public string? Origem { get; set; }
        public string? Payload_preview { get; set; }
        public string? Mensagem { get; set; }
        public string Criado_em { get; set; } = string.Empty;
    }

    // =========================================
    // FASE 16 BLOCO 3 — DTOs ESPELHO FISCAL
    // Preview técnico sem emissão ou transmissão
    // =========================================

    public class ValidacaoFiscalItemResp {
        [JsonPropertyName("entidade")]
        public string Entidade { get; set; } = string.Empty;
        [JsonPropertyName("nivel")]
        public string Nivel { get; set; } = string.Empty; // "OK" | "AVISO" | "ERRO"
        [JsonPropertyName("mensagem")]
        public string Mensagem { get; set; } = string.Empty;
    }

    public class ValidacaoFiscalResp {
        [JsonPropertyName("valido")]
        public bool Valido { get; set; }
        [JsonPropertyName("pais_fiscal")]
        public string? PaisFiscal { get; set; }
        [JsonPropertyName("ambiente")]
        public string? Ambiente { get; set; }
        [JsonPropertyName("total_erros")]
        public int TotalErros { get; set; }
        [JsonPropertyName("total_avisos")]
        public int TotalAvisos { get; set; }
        [JsonPropertyName("itens")]
        public List<ValidacaoFiscalItemResp> Itens { get; set; } = new();
    }

    public class EspelhoFiscalItemResp {
        [JsonPropertyName("venda_item_id")]
        public string VendaItemId { get; set; } = string.Empty;
        [JsonPropertyName("produto_id")]
        public string ProdutoId { get; set; } = string.Empty;
        [JsonPropertyName("descricao_produto")]
        public string DescricaoProduto { get; set; } = string.Empty;
        [JsonPropertyName("ncm_id")]
        public string? NcmId { get; set; }
        [JsonPropertyName("cfop_id")]
        public string? CfopId { get; set; }
        [JsonPropertyName("cst_csosn_id")]
        public string? CstCsosnId { get; set; }
        [JsonPropertyName("iva_id")]
        public string? IvaId { get; set; }
        /// <summary>Base de cálculo em minor unit (centavos/guaranis)</summary>
        [JsonPropertyName("base_minor")]
        public long BaseMinor { get; set; }
        /// <summary>Alíquota escala 6 (10% = 100000)</summary>
        [JsonPropertyName("aliquota_escala6")]
        public long AliquotaEscala6 { get; set; }
        /// <summary>Imposto = base * aliquota / 1_000_000 (sem float)</summary>
        [JsonPropertyName("imposto_minor")]
        public long ImpostoMinor { get; set; }
        [JsonPropertyName("origem_regra")]
        public string OrigemRegra { get; set; } = string.Empty;
    }

    public class EspelhoFiscalVendaResp {
        [JsonPropertyName("venda_id")]
        public string VendaId { get; set; } = string.Empty;
        [JsonPropertyName("pais_fiscal")]
        public string PaisFiscal { get; set; } = string.Empty;
        [JsonPropertyName("ambiente")]
        public string Ambiente { get; set; } = string.Empty;
        [JsonPropertyName("modelo_preview")]
        public string ModeloPreview { get; set; } = string.Empty;
        [JsonPropertyName("status_preparacao")]
        public string StatusPreparacao { get; set; } = string.Empty;
        [JsonPropertyName("total_base_minor")]
        public long TotalBaseMinor { get; set; }
        [JsonPropertyName("total_imposto_minor")]
        public long TotalImpostoMinor { get; set; }
        [JsonPropertyName("calculado_em")]
        public string CalculadoEm { get; set; } = string.Empty;
        [JsonPropertyName("itens")]
        public List<EspelhoFiscalItemResp> Itens { get; set; } = new();
        [JsonPropertyName("alertas")]
        public List<string> Alertas { get; set; } = new();
    }

    public class CalcularEspelhoFiscalVendaReq {
        [JsonPropertyName("venda_id")]
        public string venda_id { get; set; } = string.Empty;
        [JsonPropertyName("tipo_operacao")]
        public string? tipo_operacao { get; set; }
    }

    public class ObterEspelhoFiscalVendaReq {
        [JsonPropertyName("venda_id")]
        public string venda_id { get; set; } = string.Empty;
    }

    // === DTOs de Sync Fiscal (Fase 17 Bloco 4) ===

    public class AplicarPacoteFiscalReq {
        [JsonPropertyName("pacote_id")]
        public string? pacote_id { get; set; }
        [JsonPropertyName("versao")]
        public string versao { get; set; } = string.Empty;
        [JsonPropertyName("payload_hash")]
        public string payload_hash { get; set; } = string.Empty;
        [JsonPropertyName("payload_json")]
        public string payload_json { get; set; } = string.Empty;
        [JsonPropertyName("idempotency_key")]
        public string? idempotency_key { get; set; }
    }

    public class StatusVersaoFiscalResp {
        [JsonPropertyName("versao_atual")]
        public string? VersaoAtual { get; set; }
        [JsonPropertyName("pacote_id")]
        public string? PacoteId { get; set; }
        [JsonPropertyName("payload_hash")]
        public string? PayloadHash { get; set; }
        [JsonPropertyName("status")]
        public string? Status { get; set; }
        [JsonPropertyName("total_registros")]
        public long TotalRegistros { get; set; }
        [JsonPropertyName("aplicado_em")]
        public string? AplicadoEm { get; set; }
        [JsonPropertyName("ultimo_erro")]
        public string? UltimoErro { get; set; }
    }

    public class LogSyncFiscalResp {
        [JsonPropertyName("id")]
        public string Id { get; set; } = string.Empty;
        [JsonPropertyName("pacote_id")]
        public string? PacoteId { get; set; }
        [JsonPropertyName("versao")]
        public string? Versao { get; set; }
        [JsonPropertyName("tipo_evento")]
        public string TipoEvento { get; set; } = string.Empty;
        [JsonPropertyName("mensagem")]
        public string? Mensagem { get; set; }
        [JsonPropertyName("criado_em")]
        public string CriadoEm { get; set; } = string.Empty;
    }
}

