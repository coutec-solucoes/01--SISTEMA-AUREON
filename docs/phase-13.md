# Documentação de Entrega — Fase 13: Financeiro Base

A Fase 13 introduz o controle financeiro básico offline-first no PDV Aureon, englobando contas a pagar (compras e despesas), contas a receber (crediário / fiado), lançamentos imutáveis de caixa (livro-caixa), regras de vencimento, e impacto direto no resumo e fechamento de caixa local.

## 1. Resumo da Fase

O objetivo principal da Fase 13 foi implementar uma estrutura financeira local sólida, consistente e imutável que permita a operação offline-first do PDV sem depender de rede ativa para quitação de títulos e lançamentos de despesas. A matemática financeira foi desenhada utilizando inteiros (`i64/long`), banindo floats/doubles para evitar distorções de centavos.

## 2. Blocos Implementados & Commits

* **Bloco 1**: Estrutura de Banco de Dados e Migration SQLite (`011_fase13_financeiro.sql`).
  * *Commit*: `4ff7e64`
* **Bloco 2**: Commands Rust/Tauri de Contas a Pagar e Despesas Manuais.
  * *Commit*: `85f9d66`
* **Bloco 3**: Contas a Receber, Crediário e Integração com Venda `CREDITO_CLIENTE`.
  * *Commit*: `9f003d8`
* **Bloco 4**: Integração de Compras com Contas a Pagar e Auditorias de Fechamento de Caixa.
  * *Commit*: `ca2af78`
* **Bloco 5**: UI Blazor de Contas a Pagar, Contas a Receber e Lançamentos.
  * *Commit*: `a52ae03`
* **Bloco 6**: Homologação, Testes de Fluxo Completo e Documentação Final (Este commit).

## 3. Estrutura Relacional (SQLite Local)

### Migration `011_fase13_financeiro.sql`
A migration cria as seguintes tabelas locais:
1. **`contas_pagar`**: Controle de obrigações de despesas gerais e compras.
2. **`contas_receber`**: Controle de direitos de crédito de clientes.
3. **`financeiro_lancamentos`**: Rastro contábil de movimentações financeiras no caixa local.

## 4. Commands Tauri/Rust Criados

* **Contas a Pagar / Despesas**:
  * `listar_contas_pagar`: Listagem de títulos com filtros de status.
  * `obter_conta_pagar`: Detalhes de um título a pagar específico.
  * `registrar_despesa_manual`: Criação manual de despesas (Ex: Luz, internet).
  * `baixar_conta_pagar`: Baixa (parcial ou total) de um título gerando lançamento de caixa.
  * `cancelar_conta_pagar`: Cancelamento de títulos pendentes.
* **Contas a Receber / Crediário**:
  * `listar_contas_receber`: Listagem de títulos de clientes com filtros de status.
  * `obter_conta_receber`: Detalhes de um título a receber específico.
  * `baixar_conta_receber`: Recebimento de crediário gerando lançamento de caixa.
  * `cancelar_conta_receber`: Cancelamento de títulos pendentes.
* **Lançamentos / Auditoria**:
  * `listar_lancamentos_financeiros`: Consulta de lançamentos gerados no livro-caixa.

## 5. Telas & Componentes Blazor Criados

* **Telas Principais**:
  * `/financeiro/contas-pagar` (`FinanceiroContasPagar.razor`): Listagem, atalho para despesa manual, baixa e cancelamento.
  * `/financeiro/contas-receber` (`FinanceiroContasReceber.razor`): Painel de crediário dos clientes, atalho para baixa/recebimento e cancelamento.
  * `/financeiro/lancamentos` (`FinanceiroLancamentos.razor`): Livro-caixa de auditoria, exibindo totais de entradas, saídas e saldo líquido (Totalmente somente leitura).
* **Componentes Reutilizáveis**:
  * `DespesaManualModal.razor`: Modal para inclusão de contas a pagar manuais.
  * `BaixaTituloModal.razor`: Modal unificado para realizar baixas (parcial/total) validando caixa aberto.

## 6. Regras de Negócio e Impacto de Caixa

* **Matemática de Inteiros**: Todos os valores são manipulados em unidades menores (`long` centavos no C# e `i64` no Rust). Cotações em escala 6. Nenhuma aritmética do sistema utiliza `float`/`double`.
* **Vencimento Padrão do Crediário**: Gerado de forma automática na finalização de vendas do tipo `CREDITO_CLIENTE` com vencimento para 30 dias a partir da data de emissão.
* **Validação de Cliente Obrigatório**: Vendas com pagamento em `CREDITO_CLIENTE` exigem a presença de um cliente associado, bloqueando o checkout caso esteja nulo.
* **Caixa Aberto Exigido**: Qualquer transação de baixa financeira (pagamento ou recebimento) exige que a sessão de caixa esteja ativa (`status = 'ABERTO'`). O impacto no caixa ocorre no momento exato da baixa (soma para recebimentos, subtrai para pagamentos de despesas).
* **Cancelamento**: O cancelamento só é permitido para títulos com status `PENDENTE`. Títulos com status `PAGO_PARCIAL` ou `PAGO` possuem bloqueio de cancelamento.

## 7. Eventos do `sync_outbox`

* `CONTA_PAGAR_CRIADA`, `CONTA_PAGAR_BAIXADA`, `CONTA_PAGAR_CANCELADA`
* `CONTA_RECEBER_CRIADA`, `CONTA_RECEBER_BAIXADA`, `CONTA_RECEBER_CANCELADA`
* `FINANCEIRO_LANCAMENTO_GERADO`

## 8. Limitações e Fora do Escopo

Conforme diretrizes oficiais, os seguintes itens não foram implementados nesta fase:
* Emissão de Boletos bancários.
* Integração Pix em tempo real ou TEF (são tratados apenas como declarações financeiras na baixa).
* Conciliação bancária de extratos.
* Cálculo automático de juros e multas por atraso.
* Plano de contas estruturado (DRE avançado, contas recorrentes, etc.).
