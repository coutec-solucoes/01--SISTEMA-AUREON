# Conversão de Pré-Vendas e Orçamentos no PDV

Este documento detalha o fluxo de sincronização, consulta e conversão de Pré-Vendas e Orçamentos em vendas finais.

---

## 📥 Cache de Documentos Auxiliares

Os terminais PDV recebem a sincronização de orçamentos e pré-vendas gerados na retaguarda. Eles são cacheados nas seguintes tabelas SQLite locais:

- `pre_vendas_cache` & `pre_vendas_itens_cache`
- `orcamentos_cache` & `orcamentos_itens_cache`

---

## ⚡ Fluxo de Conversão no PDV

1. **Abertura de Documento**: O operador clica em "Importar Pré-Venda" ou "Importar Orçamento".
2. **Validações Críticas (Rust)**:
   - **Data de Validade**: Se a data atual for superior à data limite do documento, a importação é bloqueada.
   - **Status**: Apenas documentos no status `PENDENTE` podem ser convertidos.
3. **Criação da Venda**:
   - Cria-se uma nova venda com o status `EM_ANDAMENTO`.
   - Copia-se todos os itens do cache de itens para a tabela `venda_itens` do PDV local.
   - Vincula-se a origem da venda: `origem_tipo` (PRE_VENDA ou ORCAMENTO), `origem_id` e o `origem_item_id` nos itens.
4. **Fechamento e Marcação de Consumo**:
   - O status do documento no cache local é atualizado para `CONVERTIDO` ou `CONSUMIDO`.
   - Lança-se o evento de outbox atômico:
     - `PRE_VENDA_CONVERTIDA_EM_VENDA`
     - `ORCAMENTO_CONVERTIDO_EM_VENDA`
