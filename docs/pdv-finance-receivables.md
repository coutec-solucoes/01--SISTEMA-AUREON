# Contas a Receber e Crediário de Clientes — PDV Local

O módulo de Contas a Receber da Fase 13 é responsável pelo controle de créditos de clientes (crediário / fiado), permitindo que vendas sejam quitadas a prazo e baixadas posteriormente de forma offline-first.

## 1. Regras de Geração (Checkout de Vendas)

* **Associação Obrigatória de Cliente**: Para finalizar uma venda com a forma de pagamento `CREDITO_CLIENTE`, o caixa exige que um cliente ativo esteja associado à venda. Caso o campo `cliente_id` esteja vazio, o command `finalizar_venda` abortará a operação retornando um erro impeditivo.
* **Geração Automática de Título**: Ao finalizar com sucesso a venda, o sistema insere automaticamente um registro em `contas_receber` com status `PENDENTE` correspondendo ao valor total financiado pelo cliente.
* **Vencimento Padrão**: O prazo de vencimento padrão é fixado em **30 dias** após a data da venda (`data_venda + 30 dias`).
* **Isolamento de Caixa Inicial**: O valor financiado no crediário **não soma** ao saldo físico esperado do caixa ativo no ato do fechamento da venda.

## 2. Estrutura de Banco de Dados (`contas_receber`)

A tabela armazena a ficha financeira do cliente localmente no SQLite:
* `id` (`TEXT PRIMARY KEY`): UUID do título.
* `cliente_id` (`TEXT NOT NULL`): Identificador do cliente vinculado (`clientes_cache`).
* `cliente_nome_snapshot` (`TEXT NOT NULL`): Nome do cliente no momento da venda para auditoria rápida.
* `venda_id` (`TEXT NULL`): UUID da venda de origem.
* `descricao` (`TEXT NOT NULL`): Descrição textual (ex: "Crediário gerado pela Venda #1024").
* `moeda_codigo` (`TEXT NOT NULL`): Código da moeda (BRL, USD, EUR).
* `valor_original_minor` (`INTEGER NOT NULL`): Valor em unidades menores (minor units).
* `taxa_cambio_escala6` (`INTEGER NOT NULL`): Taxa de câmbio snapshot em escala 6.
* `valor_original_principal_minor` (`INTEGER NOT NULL`): Valor convertido para BRL na cotação fixada.
* `data_emissao` (`TEXT NOT NULL`), `data_vencimento` (`TEXT NOT NULL`).
* `status` (`TEXT NOT NULL`): `PENDENTE`, `PAGO_PARCIAL`, `PAGO`, `CANCELADO`.
* `saldo_pendente_minor` (`INTEGER NOT NULL`): Saldo restante a receber.

## 3. Fluxo de Baixa e Liquidação

* **Baixa Parcial**: O cliente realiza o pagamento de uma parte da dívida. O status muda para `PAGO_PARCIAL` e o `saldo_pendente_minor` é reduzido.
* **Baixa Total**: Liquidação completa. O status muda para `PAGO` e o `saldo_pendente_minor` vira `0`.
* **Sessão de Caixa Obrigatória**: Todo recebimento financeiro exige que o caixa do operador esteja **aberto**. O valor do recebimento é adicionado ao saldo físico esperado do caixa da sessão ativa na respectiva moeda de pagamento.

## 4. Regras de Cancelamento

* Apenas títulos com o status inicial `PENDENTE` podem ser cancelados diretamente via UI/Tauri.
* Cancelar títulos `PAGO_PARCIAL` ou `PAGO` é terminantemente bloqueado no backend Rust para evitar fraudes ou inconsistências no fechamento de caixa.

## 5. Eventos no `sync_outbox`

Sempre que a tabela de contas a receber sofre alterações, os respectivos eventos são inseridos na outbox local:
* `CONTA_RECEBER_CRIADA`
* `CONTA_RECEBER_BAIXADA`
* `CONTA_RECEBER_CANCELADA`
