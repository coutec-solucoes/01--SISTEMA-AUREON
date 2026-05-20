# Impacto das Movimentações Financeiras no Caixa — PDV Local

O controle financeiro da Fase 13 integra diretamente as operações de contas a pagar (despesas) e contas a receber (crediário) ao fluxo de caixa diário do operador de PDV.

## 1. Princípio do Saldo Físico Real (Isolamento de Crediário)

* **Venda em Crediário**: Ao finalizar uma venda com a forma de pagamento `CREDITO_CLIENTE`, nenhum dinheiro entra na gaveta do caixa. Portanto, para o fechamento de caixa, as vendas com `CREDITO_CLIENTE` **não somam** no saldo esperado da sessão.
* **Baixa de Crediário**: O dinheiro correspondente ao crediário entra no caixa **somente** no momento em que o cliente realiza a quitação (total ou parcial) do título a receber (`contas_receber`). Nesse instante, a baixa gera um lançamento financeiro que incrementa o saldo esperado do caixa.

## 2. Integração no Cálculo de Saldo Esperado

O saldo esperado final do caixa para cada moeda é calculado de forma dinâmica integrando as vendas com pagamentos imediatos (Dinheiro, PIX, Cartão) e os lançamentos do livro-caixa (`financeiro_lancamentos`).

### Fórmula de Fechamento de Caixa por Moeda
Para cada moeda da sessão de caixa:

$$\text{Saldo Esperado} = \text{Saldo Inicial} + \text{Vendas Físicas} + \text{Suprimentos} - \text{Sangrias} - \text{Vales} + \text{Recebimentos Financeiros} - \text{Pagamentos Financeiros}$$

Onde:
* **Vendas Físicas**: Vendas concluídas em formas de pagamento que representam fluxo imediato no caixa (ex: Dinheiro, PIX, Cartão). Exclui `CREDITO_CLIENTE`.
* **Recebimentos Financeiros**: Total de baixas (parciais ou totais) de `contas_receber` realizadas sob a sessão de caixa corrente.
* **Pagamentos Financeiros**: Total de baixas (parciais ou totais) de `contas_pagar` (incluindo despesas manuais) efetuadas sob a sessão de caixa corrente.

## 3. Comportamento no Back-end Rust

A integração ocorre em dois comandos principais de negócio:

### A. `obter_resumo_caixa`
Retorna uma visão detalhada para a UI Blazor contendo os agregados por moeda. A consulta calcula:
```sql
-- Soma dos recebimentos (entradas)
SELECT COALESCE(SUM(valor_informado_minor), 0) FROM financeiro_lancamentos
WHERE sessao_caixa_id = ? AND tipo_lancamento = 'RECEBIMENTO' AND moeda_codigo = ?

-- Soma dos pagamentos/despesas (saídas)
SELECT COALESCE(SUM(valor_informado_minor), 0) FROM financeiro_lancamentos
WHERE sessao_caixa_id = ? AND tipo_lancamento = 'PAGAMENTO' AND moeda_codigo = ?
```

### B. `fechar_caixa`
Valida as diferenças entre o valor informado pelo operador e o valor esperado (computando as vendas e os lançamentos de baixa/despesa descritos acima). Caso a diferença por moeda seja apurada, ela é gravada na tabela `sessoes_caixa_moedas` e a sessão é marcada como `FECHADO`.

## 4. Segurança Relacional

* **Imutabilidade**: O livro-caixa (`financeiro_lancamentos`) é imutável (`INSERT ONLY`). Isso impede que qualquer operador ou script altere ou delete registros históricos de caixa, garantindo que o saldo esperado auditado reflita fielmente as movimentações do banco.
