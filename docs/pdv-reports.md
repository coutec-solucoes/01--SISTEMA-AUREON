# Guia de Relatórios e Dashboard — PDV Aureon

Este documento descreve todos os relatórios operacionais locais e o dashboard executivo disponíveis no PDV Aureon a partir da Fase 14. Todos os dados são lidos exclusivamente do banco de dados SQLite local, sem dependência de internet ou serviços externos.

> **Módulo Somente Leitura**: Nenhuma tela de relatório ou dashboard executa operações de INSERT, UPDATE ou DELETE. Os dados são preservados integralmente.

---

## 1. Dashboard Executivo (`/dashboard`)

O dashboard exibe os indicadores mais relevantes para a gestão diária do estabelecimento. É a primeira tela gerencial a ser acessada pelo gestor ao abrir o sistema.

### Indicadores Disponíveis

| Bloco | O que mostra |
|---|---|
| **Faturamento por Moeda** | Total de vendas finalizadas no período, segregado por moeda (BRL, USD, EUR, etc.) |
| **Despesas Pagas por Moeda** | Total de saídas registradas no livro-caixa, segregado por moeda |
| **Vendas Realizadas** | Quantidade de transações de venda com status `FINALIZADA` |
| **Itens Vendidos** | Soma das quantidades dos itens vendidos (escala 3) |
| **Estoque Crítico** | Quantidade de produtos ativos com saldo de estoque ≤ 0 (alerta visual pulsante em vermelho) |
| **Contas a Pagar Vencidas** | Saldo total de títulos com status `PENDENTE` e data de vencimento no passado, por moeda |
| **Contas a Pagar a Vencer** | Saldo de títulos ainda dentro do prazo, por moeda |
| **Crediário Vencido** | Saldo de recebíveis de clientes em atraso, por moeda |
| **Crediário a Vencer** | Saldo de recebíveis de clientes dentro do prazo, por moeda |

### Filtros de Período do Dashboard

O dashboard sempre inicia com o período padrão de **últimos 30 dias**. O gestor pode alterar para:

- **Hoje**: Das 00:00:00 às 23:59:59 do dia atual
- **Últimos 7 dias**: Janela móvel de 7 dias
- **Mês Atual**: Do dia 1 do mês corrente até agora
- **Últimos 30 dias** *(padrão)*
- **Período personalizado**: Datas de início e fim selecionáveis

---

## 2. Relatório de Vendas

**Rota:** `/relatorios` → aba **Relatório de Vendas**
**Command Tauri:** `gerar_relatorio_vendas`

### O que exibe

- **Totais Faturados por Moeda**: Soma do `total_liquido_minor` de todas as vendas `FINALIZADA` no período, agrupado por moeda.
- **Vendas por Forma de Pagamento**: Quantidade e valor total por forma (`DINHEIRO`, `PIX`, `CARTAO_DEBITO`, `CARTAO_CREDITO`, `CREDITO_CLIENTE`, etc.) e por moeda.
- **Listagem Detalhada de Cupons**: Tabela com número da venda, data/hora, nome do cliente (ou "Consumidor Final"), operador, desconto aplicado, total líquido e status.

### Filtros Disponíveis

- Período (data início / data fim)
- Operador (usuário ID)
- Moeda
- Forma de pagamento

### Exportação CSV — Colunas

```
ID;Numero;Data;Cliente;Usuario;Status;Total Bruto;Desconto;Acrescimo;Total Liquido
```

---

## 3. Relatório de Sessões de Caixa

**Rota:** `/relatorios` → aba **Sessões de Caixa**
**Command Tauri:** `gerar_relatorio_caixa`

### O que exibe

Cada sessão de caixa registrada no período, com:

- ID da sessão (primeiros 8 caracteres)
- Operador e terminal
- Data/hora de abertura e fechamento
- Moeda da sessão
- Valor de abertura declarado
- Valor esperado (calculado pelo sistema)
- Valor informado no fechamento
- Diferença (positiva = sobra, negativa = falta)
- Status (`ABERTO` ou `FECHADO`)

### Filtros Disponíveis

- Período
- Operador

### Exportação CSV — Colunas

```
ID;Abertura;Fechamento;Operador;Terminal;Status;Moeda;Vlr Abertura;Vlr Esperado;Vlr Informado;Diferenca
```

---

## 4. Relatório Financeiro Consolidado

**Rota:** `/relatorios` → aba **Financeiro Consolidado**
**Command Tauri:** `gerar_relatorio_financeiro`

### O que exibe

- **Saldo Total de Contas a Pagar** (status `PENDENTE` e `PAGO_PARCIAL`), por moeda
- **Saldo Total de Contas a Receber** (crediário em aberto), por moeda
- **Lançamentos do Livro-Caixa no Período**: Tabela cronológica com data, tipo (`RECEBIMENTO` ou `PAGAMENTO`), forma de pagamento, moeda original, valor original, valor convertido em BRL e operador

> ⚠️ Os lançamentos são **imutáveis** — são registros históricos que nunca são alterados ou excluídos.

### Filtros Disponíveis

- Período
- Operador
- Moeda
- Forma de pagamento

### Exportação CSV — Seções

1. `CONTAS A PAGAR`: ID, Fornecedor, Vencimento, Status, Moeda, Valor Original, Saldo Pendente
2. `CONTAS A RECEBER`: ID, Cliente, Vencimento, Status, Moeda, Valor Original, Saldo Pendente
3. `LANCAMENTOS FINANCEIROS`: ID, Data, Tipo, Forma, Moeda, Valor Original, Valor BRL, Operador, Observação

---

## 5. Relatório de Estoque e Kardex

**Rota:** `/relatorios` → aba **Estoque & Kardex**
**Command Tauri:** `gerar_relatorio_estoque_kardex`

### O que exibe

**Bloco 1 — Valorização do Estoque:**
- Valor estimado total do estoque em BRL, calculado multiplicando a quantidade atual pelo último custo unitário de cada produto (convertido pela taxa de câmbio da última compra).

**Bloco 2 — Posição Física Atual:**
- Tabela de todos os produtos ativos com `controla_estoque = true`
- Colunas: Nome, SKU, Quantidade atual (escala 3), Último custo (minor units BRL)
- Produtos com quantidade ≤ 0 são destacados em vermelho

**Bloco 3 — Extrato de Movimentações (Kardex):**
- Histórico cronológico de entradas e saídas no período filtrado
- Colunas: Data, Produto, Tipo (`ENTRADA` / `SAIDA` / `ENTRADA_COMPRA` / `ESTORNO_ENTRADA_COMPRA` / `AJUSTE_INVENTARIO`, etc.), Quantidade, Origem (ID da venda ou compra), Operador, Observação

### Filtros Disponíveis

- Período (afeta apenas o Kardex; a posição física é sempre atual)
- Operador (afeta apenas o Kardex)

### Exportação CSV — Seções

1. `POSICAO FISICA ATUAL`: Produto ID, Nome, SKU, Controla Estoque, Qtd, Custo Unitário
2. `KARDEX MOVIMENTACOES FILTRADAS`: ID, Produto ID, Nome, Data, Tipo, Qtd, Origem, Usuário, Obs

---

## 6. Relatório de Compras

**Rota:** `/relatorios` → aba **Relatório de Compras**
**Command Tauri:** `gerar_relatorio_compras`

### O que exibe

- **Totais por Moeda**: Soma do valor total das compras `FINALIZADA` no período, por moeda original
- **Compras por Fornecedor**: Total equivalente em BRL (convertido pela taxa snapshot de cada compra), agrupado por fornecedor
- **Listagem de Notas de Entrada**: Tabela com ID, fornecedor, data, status, moeda, valor original e valor convertido em BRL

### Filtros Disponíveis

- Período
- Moeda

### Exportação CSV — Colunas

```
ID;Fornecedor;Data;Status;Moeda;Total Original;Total BRL;Total Itens
```

---

## 7. Ranking de Produtos Mais Vendidos

**Rota:** `/relatorios` → aba **Ranking de Venda**
**Command Tauri:** `gerar_relatorio_produtos_mais_vendidos`

### O que exibe

Tabela ordenada por quantidade vendida (maior para menor), com:
- Posição no ranking
- Nome e SKU do produto
- Quantidade total vendida no período (escala 3, exibida como decimal com 3 casas)
- Faturamento bruto em BRL (soma dos valores dos itens convertidos)

### Filtros Disponíveis

- Período
- Moeda
- Forma de pagamento (filtra as vendas que contêm os itens)

### Exportação CSV — Colunas

```
Produto ID;Nome;SKU;Qtd Vendida;Faturamento Bruto BRL
```

---

## 8. Relatório Gourmet e Delivery

**Rota:** `/relatorios` → aba **Gourmet & Delivery**
**Command Tauri:** `gerar_relatorio_gourmet_delivery`

### O que exibe

**Indicadores Gerais:**
- Total de pedidos delivery no período
- Total de taxas de entrega faturadas (em BRL)
- Total de atendimentos gourmet (mesas e comandas)
- Ticket médio gourmet principal em BRL

**Faturamento Delivery por Moeda:** Agrupamento do faturamento de pedidos delivery por moeda.
**Faturamento Gourmet por Moeda:** Agrupamento do faturamento de comandas gourmet por moeda.
**Contagem de Entregas por Status:** Quantidade de pedidos em cada status (`AGUARDANDO`, `EM_ROTA`, `ENTREGUE`, `CANCELADO`, etc.).

### Filtros Disponíveis

- Período

### Exportação CSV — Estrutura

```
METRICA;MOEDA/STATUS;VALOR
Total Pedidos Delivery;;[n]
Total Taxas Entrega;;[minor]
...
Entrega Status;[STATUS];[n]
Fat Delivery Moeda;[MOEDA];[minor]
Fat Gourmet Moeda;[MOEDA];[minor]
```

---

## 9. Filtros Globais Disponíveis

Todos os relatórios no Hub de Relatórios compartilham os seguintes filtros no topo da tela:

| Filtro | Tipo | Descrição |
|---|---|---|
| `data_inicio` | Date | Data de início do período |
| `data_fim` | Date | Data de fim do período |
| `usuario_id` | Texto | Filtra por operador (ID do usuário) |
| `moeda_codigo` | Seleção | Filtra por moeda (`BRL`, `USD`, `EUR`) |
| `forma_pagamento` | Seleção | Filtra por forma de pagamento |

> **`sessao_caixa_id`** está preparado no DTO Rust mas ainda não possui campo de entrada visual na UI. Reservado para fase futura.

---

## 10. Exportação CSV Local

- Toda exportação é feita **localmente no dispositivo**, sem envio de dados para a internet.
- O arquivo CSV é gerado em C# pelo Blazor e entregue ao navegador via a função JS `aureon.downloadFile`.
- O nome do arquivo segue o padrão: `relatorio_[aba]_[yyyyMMdd_HHmmss].csv`
- Separador: **ponto-e-vírgula** (`;`), compatível com Excel e LibreOffice em configurações pt-BR.
- Encoding: `UTF-8`.

### Como usar

1. Abra o Hub de Relatórios (`/relatorios`).
2. Selecione a aba desejada no menu lateral.
3. Aplique os filtros e clique em **Consultar**.
4. Clique em **Exportar CSV** (disponível somente após uma consulta com resultado).

---

## 11. Impressão e Exportação PDF

- O botão **"Imprimir / PDF"** invoca `window.print()` via JSInterop.
- O layout de impressão é limpo: sem barra de navegação, sem botões, sem filtros. Apenas o cabeçalho com dados de emissão e as tabelas do relatório selecionado.
- Para gerar PDF: clique em **Imprimir / PDF** → selecione **"Salvar como PDF"** na caixa de diálogo do navegador.
- A impressão está formatada para papel **A4 vertical**.

---

## 12. Cuidados de Performance com SQLite Local

O banco de dados SQLite local pode crescer significativamente em estabelecimentos com alto volume de transações. Siga as boas práticas abaixo para manter a performance dos relatórios:

| Recomendação | Motivo |
|---|---|
| **Sempre use filtro de período** | Evita varredura completa das tabelas `vendas`, `financeiro_lancamentos`, `estoque_kardex`, etc. |
| **Evite períodos superiores a 12 meses** | Relatórios de período muito longo podem demorar em dispositivos com hardware limitado |
| **Prefira filtros de moeda ou operador adicionais** | Reduzem o conjunto de dados e melhoram o tempo de resposta das queries |
| **O filtro padrão de 30 dias protege a performance** | É o período ideal para uso diário sem comprometer a experiência |
| **Índices das tabelas operacionais** | As tabelas `vendas`, `contas_pagar`, `contas_receber`, `estoque_kardex` e `financeiro_lancamentos` possuem índices criados nas migrations para suportar os filtros de data, status e moeda |

---

## Referências

- Implementação técnica: [`apps/aureon-pdv/src-tauri/src/commands_relatorios.rs`](../apps/aureon-pdv/src-tauri/src/commands_relatorios.rs)
- DTOs Rust: [`crates/aureon-core/src/dtos.rs`](../crates/aureon-core/src/dtos.rs)
- DTOs C#: [`apps/aureon-pdv/ui-blazor/Services/PdvModels.cs`](../apps/aureon-pdv/ui-blazor/Services/PdvModels.cs)
- Telas: [`apps/aureon-pdv/ui-blazor/Pages/DashboardPdv.razor`](../apps/aureon-pdv/ui-blazor/Pages/DashboardPdv.razor) e [`RelatoriosPdv.razor`](../apps/aureon-pdv/ui-blazor/Pages/RelatoriosPdv.razor)
- Documentação da Fase 14: [`docs/phase-14.md`](./phase-14.md)
