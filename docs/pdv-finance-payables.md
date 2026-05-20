# Contas a Pagar e Despesas Manuais — PDV Local

O módulo de Contas a Pagar da Fase 13 introduz o controle de saídas financeiras diretamente no PDV local (offline-first). Ele gerencia tanto despesas gerais do terminal (como contas de luz, aluguel) quanto obrigações originadas por notas de compra manuais.

## 1. Origem dos Títulos

As contas a pagar podem ser criadas de duas formas:
1. **Lançamento Manual (Despesas)**: Realizado pela UI Blazor (`FinanceiroContasPagar.razor` através do modal `DespesaManualModal.razor`). Permite associar um fornecedor de forma opcional.
2. **Geração Automática via Compras**: Ao finalizar uma Nota de Compra (Fase 12), o sistema cria automaticamente um registro em `contas_pagar` com o status inicial `PENDENTE` e vencimento padrão de 30 dias.

## 2. Estrutura de Banco de Dados (`contas_pagar`)

A tabela é definida no SQLite local com as seguintes colunas principais:
* `id` (`TEXT PRIMARY KEY`): UUID do título.
* `fornecedor_id` (`TEXT NULL`): Vínculo com a tabela de fornecedores locais.
* `fornecedor_nome_snapshot` (`TEXT NULL`): Nome do fornecedor no momento da criação, mantendo histórico legível.
* `compra_id` (`TEXT NULL`): Vínculo com a compra originadora (se aplicável).
* `descricao` (`TEXT NOT NULL`): Detalhamento do título.
* `moeda_codigo` (`TEXT NOT NULL`): Código da moeda (BRL, USD, EUR).
* `valor_original_minor` (`INTEGER NOT NULL`): Valor em unidades menores (ex: R$ 100,00 = `10000`).
* `taxa_cambio_escala6` (`INTEGER NOT NULL`): Taxa de câmbio snapshot em escala 6 (ex: `5.250000` = `5250000`).
* `valor_original_principal_minor` (`INTEGER NOT NULL`): Valor convertido na moeda principal (BRL) baseado na cotação acima.
* `data_emissao` (`TEXT NOT NULL`), `data_vencimento` (`TEXT NOT NULL`).
* `status` (`TEXT NOT NULL`): `PENDENTE`, `PAGO_PARCIAL`, `PAGO`, `CANCELADO`.
* `saldo_pendente_minor` (`INTEGER NOT NULL`): Saldo restante a ser pago.

## 3. Fluxo de Baixa e Liquidação

* **Baixa Parcial**: Permite amortizar uma parte do saldo pendente. O status do título é alterado para `PAGO_PARCIAL` e o `saldo_pendente_minor` é reduzido pelo valor pago.
* **Baixa Total**: Liquida completamente o título. O status muda para `PAGO` e o `saldo_pendente_minor` passa a ser `0`.
* **Sessão de Caixa Obrigatória**: Qualquer baixa exige que a registradora local tenha uma sessão de caixa aberta (`status = 'ABERTO'`). O valor pago é deduzido do saldo físico do caixa ativo na respectiva moeda.

## 4. Regras de Cancelamento

* O cancelamento só é permitido para títulos com status `PENDENTE`.
* Caso o título seja `PAGO_PARCIAL` ou `PAGO`, a operação de cancelamento é explicitamente bloqueada na API e na UI.
* Se uma nota de compra finalizada for cancelada, o sistema realiza o cancelamento automático do título `contas_pagar` vinculado, desde que este ainda esteja `PENDENTE`. Caso o título já possua baixas parciais ou totais, o cancelamento da compra é bloqueado.

## 5. Eventos no `sync_outbox`

Toda movimentação em `contas_pagar` gera eventos estruturados para sincronização posterior com a retaguarda PostgreSQL:
* `CONTA_PAGAR_CRIADA`: Disparado na criação manual de despesa ou automática via compra.
* `CONTA_PAGAR_BAIXADA`: Disparado a cada amortização (parcial ou total).
* `CONTA_PAGAR_CANCELADA`: Disparado no cancelamento do título.
