# Pagamento Multimoeda

A lógica central financeira para regiões fronteiriças (USD, PYG e BRL circulando juntos).

## Proteção Inteira em Câmbio
Taxas de Câmbio frequentemente usam muitas casas decimais. Para fugir do `Float`, a taxa de conversão (`TaxaCambioEscala6`) aplica o multiplicador de `1.000.000`.
- Se USD vale R$ 5,20, a taxa no backend é `5200000`.

## Conversão Dinâmica
Se um cliente paga $10 USD em uma venda orçada em Reais:
- `valor_informado_minor` = `1000` (10 dólares = 1000 cents)
- Cálculo: `(1000 * 5200000) / 1000000` = `5200` Minor BRL (R$ 52,00 na moeda principal do país).
Esse valor entra para quitar a venda, de forma 100% inteira sem arredondamentos perigosos.

## Snapshot
O pagamento grava em definitivo a data e a cotação exata da hora do fato (`data_cotacao_usada`). Assim, variações cambiais de fechamento de caixa ou D+1 não distorcerão vendas passadas.

## Troco (`calcular_troco`)
Calculado usando as bases absolutas nativas já convertidas. O front-end permite assinalar qual `moeda_troco_codigo` o cliente quer, mas o motor do PDV guarda esse saldo para análise futura.
