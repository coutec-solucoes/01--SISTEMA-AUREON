# Fase 11 - Estoque Operacional, Baixa de Estoque, Kardex e Inventário Base

## Resumo da Fase
A Fase 11 introduziu o módulo de estoque local offline-first no AUREON PDV. O objetivo principal foi permitir a baixa automática do estoque durante as vendas e gerenciar o Kardex (trilha de auditoria imutável) sem bloquear a fila do caixa, garantindo alta disponibilidade (saldo negativo é permitido). Tudo foi integrado na base SQLite do terminal, sem depender de sincronização constante com a nuvem, operando de forma assíncrona via `sync_outbox`.

## Blocos Implementados
- **Bloco 1**: Estruturação de banco de dados (Migration 009) para `produtos_estoque_cache`, `estoque_movimentacoes` e flag `controla_estoque` nos produtos.
- **Bloco 2**: Commands Rust e DTOs para consultas e apontamentos de estoque (`consultar_saldos_estoque`, `listar_kardex_produto`, `ajustar_estoque_manual`, `registrar_inventario`).
- **Bloco 3**: Acoplamento das regras de baixa e estorno na finalização e cancelamento de vendas no PDV (mesma transação SQLite, garantindo atomicidade).
- **Bloco 4**: Interface do usuário no Blazor para consulta, Kardex, ajuste e inventário.
- **Bloco 5**: Validações de fluxo completo e documentação (este checklist e os arquivos na pasta docs).

## Banco de Dados
**Migrations Criadas:**
- `009_fase11_estoque.sql`

**Tabelas Criadas/Alteradas:**
- `produtos_cache` (ADD COLUMN `controla_estoque INTEGER NOT NULL DEFAULT 1`)
- `produtos_estoque_cache` (Mantém o saldo consolidado local - `quantidade_escala3`)
- `estoque_movimentacoes` (Kardex: Histórico de todas as transações, estritamente de inserção - imutável)
- `estoque_lotes` (Estrutura básica projetada, mas inoperante nesta fase).

## Commands Criados
- `consultar_saldos_estoque`: Retorna os itens ativos e seus respectivos saldos.
- `listar_kardex_produto`: Traz o histórico de um item específico.
- `ajustar_estoque_manual`: Lança uma quebra, bonificação ou ajuste cego informando tipo (Entrada/Saída) e quantidade.
- `registrar_inventario`: Processa um levantamento global, calculando o delta do estoque contábil pelo físico (gerado em lote).
- *(Internos)* `processar_baixa_venda` e `processar_estorno_venda`: Integrados no ciclo de vida de `commands_pagamento` e `commands_venda`.

## Telas Blazor Criadas
- `EstoquePdv.razor`: Listagem e busca de estoque.
- `EstoqueKardexModal.razor`: Visualização do rastreamento (Kardex).
- `EstoqueAjusteModal.razor`: Fluxo rápido de correção de estoque.
- `EstoqueInventarioModal.razor`: Contagem massiva em lista por delta.

## Regras Implementadas
**Baixa:**
- Somente ocorre em `finalizar_venda` antes de `tx.commit()`.
- Lançamentos em `venda_itens` de itens com `controla_estoque = 1` são agrupados por `produto_id` e abatidos do `produtos_estoque_cache`.
- Saldo pode ficar negativo. Sem validações bloqueantes.
- É idempotente: verifica na tabela de `estoque_movimentacoes` se a baixa para esta venda já foi gravada antes de executar.

**Estorno:**
- Somente ocorre em `cancelar_venda` e *apenas* se a venda estava `FINALIZADA` (vendas em andamento canceladas não batem no estoque).
- Ação totalmente atômica na mesma transaction.
- Soma o estoque de volta no cache e insere uma nova movimentação `ESTORNO_VENDA`. Nunca apaga ou faz update na baixa anterior.

**Inventário:**
- Informa-se o saldo real. O sistema calcula a diferença (delta) para igualar o cache ao físico.
- Se o delta for 0, nenhuma movimentação inútil é gerada.

**Eventos Sync Outbox:**
- Ao registrar movimentos no Kardex, insere na tabela `sync_outbox` o evento `ESTOQUE_MOVIMENTACAO_GERADA`, que sincronizará com o backend e arquivará.

## Limitações Conhecidas e Escopo Não Implementado
- **Inflação de Linhas do Kardex:** A tabela `estoque_movimentacoes` no SQLite tenderá a ficar com milhões de registros. Requer serviço de expurgo local assim que bater no Sync Cloud.
- **Inventário em Massa (UX):** A listagem em tela inteira será custosa se o banco tiver mais de 10.000 itens (precisa de paginação no Blazor).
- **Fora do Escopo:** Compras (XML), Ficha Técnica/Insumos (pizza 1/2), Lote/Validade (FIFO), Serial/IMEI, Planograma.
