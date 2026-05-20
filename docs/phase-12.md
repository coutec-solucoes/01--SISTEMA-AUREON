# Fase 12 — Compras, Entrada de Mercadorias, Notas Manuais e Reposição de Estoque

Esta fase implementa o módulo offline-first de lançamento manual de compras e entradas de mercadorias no PDV local, integrando com o controle de estoque local, o Kardex de movimentações e o gerenciamento de custos em múltiplas moedas.

## Blocos Implementados

1. **Bloco 1**: Estrutura de banco de dados (Migration SQLite `010_fase12_compras.sql` contendo `fornecedores_cache`, `compras` e `compra_itens`).
2. **Bloco 2**: Tauri Commands e DTOs de gerenciamento (Iniciar, Adicionar Item, Remover Item, Cancelar Compra em Andamento).
3. **Bloco 3**: Regras operacionais de finalização e cancelamento finalizado (estorno no Kardex, atualização de custo base e atomicidade transacional).
4. **Bloco 4**: Interface do usuário em Blazor (Central de Compras, detalhes, modais de criação e adição de produtos).
5. **Bloco 5**: Homologação, fluxo de testes completos e documentação.

## Commits Relevantes
- `04cc576` — Migration SQLite 010 — Compras e Entrada Manual (Fase 12 - Bloco 1)
- `b772328` — Tauri/Rust Commands base de Compras e DTOs (Fase 12 - Bloco 2)
- `c241af5` — Finalização, Entrada de Estoque, Kardex, Estorno de Compra e Custos (Fase 12 - Bloco 3)
- `19bfc38` — UI Blazor PDV Compras e Entrada Manual (Fase 12 - Bloco 4)

## Banco de Dados (SQLite)

### Tabelas Criadas
- `fornecedores_cache`: Armazena fornecedores sincronizados (ou mockados) contendo `id`, `nome`, `documento` e `ativo`.
- `compras`: Cabeçalho das compras (`id`, `fornecedor_id`, `numero_nota`, `serie`, `chave_acesso_xml_fiscal`, `data_emissao`, `status`, `moeda_codigo`, `taxa_cambio_escala6`, `subtotal_itens_minor`, `desconto_total_minor`, `frete_total_minor`, `outras_despesas_minor`, `impostos_total_minor`, `total_compra_minor`, `observacao`, `criado_em`, `atualizado_em`, `finalizada_em`, `cancelada_em`, `motivo_cancelamento`, `usuario_id`).
- `compra_itens`: Itens da nota (`id`, `compra_id`, `produto_id`, `descricao_produto_snapshot`, `quantidade_escala3`, `custo_unitario_minor`, `total_item_minor`, `lote`, `validade`, `serial`, `imei`, `cancelado`, `criado_em`).

### Campos Adicionados
- `ultimo_custo_minor` em `produtos_cache`: Armazena o custo unitário da última entrada na moeda principal (BRL).

## Tauri Commands Criados
- `buscar_fornecedores_compra(busca)`: Pesquisa fornecedores ativos no cache.
- `listar_compras(status)`: Retorna as compras registradas com filtro opcional de status.
- `obter_compra(id)`: Retorna os detalhes de uma compra e seus itens.
- `iniciar_compra(dto)`: Abre uma nova compra com status `EM_ANDAMENTO`.
- `adicionar_item_compra(dto)`: Adiciona um produto à compra recalculando cabeçalhos.
- `remover_item_compra(item_id)`: Remove/cancela logicamente um item recalculando cabeçalhos.
- `cancelar_compra_em_andamento(dto)`: Cancela permanentemente uma compra em andamento.
- `finalizar_compra(compra_id, usuario_id)`: Finaliza a compra, alimenta saldos e gera Kardex.
- `cancelar_compra_finalizada(dto)`: Cancela a compra finalizada e gera o estorno de estoque no Kardex.

## UI Blazor Criada
- **Central de Compras (`ComprasPdv.razor`)**: Painel de visualização com filtros por status e atalhos.
- **Detalhes da Compra (`CompraDetalhe.razor`)**: Tela unificada de visualização/edição, operações de itens e finalização/estorno.
- **Modais Auxiliares**:
  - `NovaCompraModal.razor`: Abertura de compras.
  - `CompraItemModal.razor`: Inserção de itens.

## Regras Operacionais Principais

### Compra Manual e Moedas
- Toda compra requer um fornecedor ativo existente no `fornecedores_cache`.
- Suporta BRL (taxa fixa escala 6 = `1000000`), USD e EUR (taxa snapshot definida na criação da compra).

### Entrada e Estorno no Estoque
- **Finalização**: Adiciona quantidades apenas para produtos com `controla_estoque = 1` no `produtos_estoque_cache`. Gera Kardex `ENTRADA_COMPRA` com sinal de adição.
- **Cancelamento Finalizado**: Gera movimentação compensatória `ESTORNO_ENTRADA_COMPRA` com a quantidade inversa (`-quantidade`) no Kardex e remove o saldo do estoque local.

### Atualização de Custo
- O campo `ultimo_custo_minor` em `produtos_cache` é atualizado na moeda principal (BRL) usando a cotação travada da nota:
  $$\text{custo\_convertido\_minor} = \frac{\text{custo\_unitario\_minor} \times \text{taxa\_cambio\_escala6}}{1.000.000}$$

### Eventos Sincronizados (Outbox)
Os eventos abaixo são registrados transacionalmente para sync com a retaguarda:
- `COMPRA_CRIADA`
- `COMPRA_FINALIZADA`
- `COMPRA_CANCELADA`
- `ESTOQUE_MOVIMENTACAO_GERADA`

## Limitações e Fora de Escopo
- Não realiza importação nativa de XML da SEFAZ.
- Não efetua lançamentos automáticos no financeiro/contas a pagar local.
- Sem regras lógicas complexas de reajuste automático de preço de venda e sem preço médio ponderado (PMP) local.
- Lote, validade, serial e IMEI são guardados apenas estruturalmente (sem restrições lógicas no checkout).
