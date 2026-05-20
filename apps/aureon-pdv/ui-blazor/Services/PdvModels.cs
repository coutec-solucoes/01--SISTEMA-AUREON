using System;
using System.Collections.Generic;

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
}

