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
}
