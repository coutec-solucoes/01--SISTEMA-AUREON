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
}
