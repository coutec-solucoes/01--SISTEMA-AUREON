# Documentação de Entrega — Fase 14: Relatórios Operacionais e Dashboard Local

A Fase 14 implementa o módulo de relatórios operacionais e dashboard executivo local do PDV Aureon. É um módulo **estritamente somente leitura** — nenhum `INSERT`, `UPDATE` ou `DELETE` operacional é executado. Toda aritmética continua em inteiros (`i64`/`long`). Multimoeda é exibida de forma segregada, sem conversões automáticas ou soma de moedas distintas.

---

## 1. Resumo da Fase

O objetivo da Fase 14 foi implementar visibilidade gerencial local para o operador e gestor do PDV sem introduzir nenhum risco às regras operacionais das fases anteriores. Os dados são consultados exclusivamente por `SELECT` nas tabelas SQLite locais. Todos os indicadores, relatórios e exportações são calculados e gerados localmente, sem dependência de internet, cloud ou BI externo.

---

## 2. Escopo Implementado

| Item | Status |
|---|---|
| Dashboard executivo local | ✅ Implementado |
| Relatório de vendas detalhado | ✅ Implementado |
| Relatório de sessões de caixa | ✅ Implementado |
| Relatório financeiro consolidado | ✅ Implementado |
| Relatório de estoque e Kardex | ✅ Implementado |
| Relatório de compras e fornecedores | ✅ Implementado |
| Ranking de produtos mais vendidos | ✅ Implementado |
| Relatório gourmet e delivery | ✅ Implementado |
| Exportação CSV local (sem internet) | ✅ Implementado |
| Impressão / PDF via navegador | ✅ Implementado |
| Filtro padrão de 30 dias | ✅ Implementado |
| Multimoeda segregada (sem conversão) | ✅ Implementado |

---

## 3. Escopo Proibido — Respeitado Integralmente

Os seguintes itens estão **fora do escopo** desta fase e não foram implementados:

- ❌ BI cloud ou serviço externo de relatórios
- ❌ Dashboard online / real-time com push de dados
- ❌ Fiscal oficial (NFC-e, SAT, MFE, NFe)
- ❌ Conciliação bancária
- ❌ DRE avançado ou plano de contas
- ❌ Gráficos com bibliotecas externas (Chart.js, ApexCharts, etc.)
- ❌ Qualquer `INSERT`, `UPDATE` ou `DELETE` operacional nos relatórios
- ❌ `float`, `f64`, `double` em regras de cálculo operacional

---

## 4. Commands Tauri/Rust Criados

Todos os commands estão em `apps/aureon-pdv/src-tauri/src/commands_relatorios.rs` e registrados em `lib.rs`.

| Command | Descrição |
|---|---|
| `obter_indicadores_dashboard` | Indicadores executivos: faturamento, despesas, estoque crítico e alertas de vencimento por moeda |
| `gerar_relatorio_vendas` | Relatório de vendas finalizadas com totais por moeda e por forma de pagamento |
| `gerar_relatorio_caixa` | Sessões de caixa com diferenças de fechamento e status |
| `gerar_relatorio_financeiro` | Consolidado de contas a pagar, a receber e lançamentos do livro-caixa |
| `gerar_relatorio_estoque_kardex` | Posição física de estoque + extrato de movimentações do Kardex |
| `gerar_relatorio_compras` | Notas de compra por fornecedor, moeda e status |
| `gerar_relatorio_produtos_mais_vendidos` | Ranking por quantidade vendida (escala 3) e faturamento bruto BRL |
| `gerar_relatorio_gourmet_delivery` | Atendimentos gourmet, ticket médio, entregas delivery por status e faturamento segregado |

**Parâmetro de filtro unificado (`FiltrosRelatorio`):**
- `data_inicio`: String `"yyyy-MM-dd HH:mm:ss"`
- `data_fim`: String `"yyyy-MM-dd HH:mm:ss"`
- `usuario_id`: Opcional — filtra por operador
- `sessao_caixa_id`: Opcional — filtra por sessão específica
- `moeda_codigo`: Opcional — filtra por moeda (`BRL`, `USD`, `EUR`, etc.)
- `forma_pagamento`: Opcional — filtra por forma de pagamento

---

## 5. DTOs Criados / Atualizados

### Rust (`crates/aureon-core/src/dtos.rs`)

- `FiltrosRelatorio`
- `IndicadoresDashboardResp`
- `ValorPorMoeda`
- `RelatorioVendasResp`, `VendaDetalheRelatorio`, `VendaPorFormaRelatorio`, `TotalPorMoedaRelatorio`
- `RelatorioCaixaResp`, `SessaoCaixaRelatorio`
- `RelatorioFinanceiroResp`, `ContaPagarRelatorio`, `ContaReceberRelatorio`, `LancamentoRelatorio`
- `RelatorioEstoqueResp`, `PosicaoEstoqueRelatorio`, `KardexItemRelatorio`
- `RelatorioComprasResp`, `CompraRelatorio`, `CompraFornecedorTotalRelatorio`
- `ProdutoMaisVendidoResp`
- `RelatorioGourmetDeliveryResp`, `ValorPorMoeda`, `DeliveryPorStatusRelatorio`

### C# (`apps/aureon-pdv/ui-blazor/Services/PdvModels.cs`)

Todos os records C# espelham os DTOs Rust acima, com campos em `PascalCase` via `JsonPropertyName`.

---

## 6. Telas Blazor Criadas

| Arquivo | Rota | Descrição |
|---|---|---|
| `DashboardPdv.razor` | `/dashboard` | Dashboard executivo com indicadores financeiros e alertas |
| `RelatoriosPdv.razor` | `/relatorios` | Hub de 7 relatórios com filtros, exportação CSV e impressão |

**Navegação:** Links adicionados no `MainLayout.razor` para **Dashboard** e **Relatórios**.

---

## 7. Estratégia de Filtro Padrão de 30 Dias

- Ao abrir o Dashboard ou o Hub de Relatórios, o período padrão é automaticamente definido como **últimos 30 dias**.
- Filtros disponíveis ao usuário:
  - Hoje
  - Últimos 7 dias
  - Mês Atual
  - Últimos 30 dias *(padrão)*
  - Período Personalizado (datas livres)
- **Motivação**: Proteger a performance do SQLite local, evitando carregar histórico completo na primeira abertura.

---

## 8. Estratégia Multimoeda — Segregação Estrita

- Totais são **sempre exibidos separados por moeda** (BRL, USD, EUR, etc.).
- **Nunca** são somadas moedas distintas em um único valor agregado.
- A conversão para BRL (usando `taxa_cambio_escala6`) é utilizada apenas para o campo **"Equivalente BRL"** de comparação, nunca substituindo o valor original.
- **Motivação**: Evitar distorções financeiras em operações multimoeda sem a taxa de câmbio correta.

---

## 9. Estratégia de Exportação CSV

- Geração feita **100% localmente**, sem envio de dados para servidores externos.
- O Blazor monta a `string` CSV em C# e invoca `aureon.downloadFile(filename, contentType, content)` via `IJSRuntime`.
- A função `aureon.downloadFile` em `tauri-interop.js` cria um `Blob`, gera uma URL temporária e dispara o download via elemento `<a>` criado dinamicamente.
- Separador: `;` (ponto-e-vírgula), compatível com planilhas brasileiras.
- Encoding: `text/csv;charset=utf-8`.

---

## 10. Estratégia de Impressão / PDF via Navegador

- A tela `RelatoriosPdv.razor` implementa dois blocos:
  1. **`no-print`**: Interface interativa completa (filtros, botões, menu lateral) — oculta na impressão.
  2. **`print-only`**: Layout limpo com cabeçalho de emissão, dados tabulares e sem elementos de UI — visível apenas na impressão.
- O CSS `@media print` é aplicado diretamente na tag `<style>` do componente.
- O botão **"Imprimir / PDF"** invoca `window.print()` via JSInterop.
- O usuário pode salvar o resultado como PDF usando o recurso nativo do navegador ("Imprimir > Salvar como PDF").

---

## 11. Confirmação: Módulo Somente Leitura

> **CONFIRMADO**: Nenhum command da Fase 14 executa `INSERT`, `UPDATE` ou `DELETE` nas tabelas operacionais.

Todas as queries em `commands_relatorios.rs` são exclusivamente `SELECT`. Nenhuma regra operacional de vendas, caixa, estoque, compras ou financeiro foi alterada.

---

## 12. Confirmação: Ausência de Float/f64 em Regras Operacionais

> **CONFIRMADO**: Nenhum valor monetário ou quantidade operacional usa `f64`, `f32`, `float` ou `double` nas queries ou no backend Rust.

- Dinheiro: `i64` (minor units, centavos).
- Quantidade: `i64` (escala 3, ex: 1 unidade = 1000).
- Cotação: `i64` (escala 6).
- A conversão para `decimal` ocorre **apenas na camada de apresentação C#** (`minor / 100m`), nunca persistida ou retornada ao Rust.

---

## 13. Resultados dos Builds

| Ferramenta | Resultado | Avisos | Erros |
|---|---|---|---|
| `cargo check -p aureon-pdv` | ✅ `Finished` | 2 (variável `mut` desnecessária, função `escala_moeda` não usada) | 0 |
| `dotnet build` | ✅ `Build succeeded` | 15 (CS8602 pré-existentes em modais de compra/estoque) | 0 |

**Nota**: Os 15 avisos `CS8602` são pré-existentes de fases anteriores (Compras, Estoque) e não foram introduzidos pela Fase 14. O aviso `CS0649` em `sessaoCaixaIdFilter` é intencional — o campo existe para filtragem futura, ainda sem UI de entrada na Fase 14.

---

## 14. Commits da Fase 14

| Descrição | Hash |
|---|---|
| Implementação técnica (Backend + Frontend) | `e29dea1` |
| Documentação oficial (Este commit) | *(gerado neste bloco)* |

---

## 15. Limitações Conhecidas e Ressalvas Controladas

1. **Sem gráficos visuais**: Intencionalmente. Indicadores são apresentados em cards e tabelas. Bibliotecas de gráficos não foram adotadas nesta fase.
2. **Sem BI cloud**: Os relatórios são locais e offline-first. Análises avançadas multi-loja não fazem parte desta fase.
3. **Sem exportação XLSX**: Apenas CSV local. Exportação para Excel avançado (XLSX nativo) não implementada.
4. **`sessaoCaixaIdFilter` reservado**: Campo de filtro por sessão de caixa está preparado no DTO mas sem campo de entrada visual na UI (previsto para fase futura).
5. **Sem fiscal oficial**: NFC-e, SAT, NFe e documentos fiscais eletrônicos são escopo de fase futura dedicada.
