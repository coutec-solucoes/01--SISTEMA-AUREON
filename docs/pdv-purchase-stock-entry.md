# Entrada de Mercadorias e Estorno no Estoque

A entrada de mercadorias e a reposição de estoque local no PDV local ocorrem de forma integrada com a finalização e o cancelamento (estorno) de compras finalizadas. Ambas as ações utilizam transações atômicas no SQLite e afetam as movimentações no Kardex local.

## Entrada de Mercadorias (Finalização)

Ao finalizar uma compra (`finalizar_compra`), o sistema executa os seguintes passos dentro de uma única transação SQLite:

1. **Validação de Estado**: A compra deve estar no status `EM_ANDAMENTO` e conter pelo menos 1 item ativo (não cancelado).
2. **Atualização de Status**: O status da compra muda para `FINALIZADA` e o campo `finalizada_em` é preenchido com o timestamp atual UTC.
3. **Alimentação de Estoque**:
   - Para cada item ativo na compra, o sistema verifica no cache de produtos (`produtos_cache`) se a flag `controla_estoque` está configurada como `1`.
   - Se `controla_estoque = 1`:
     - Adiciona a quantidade (`quantidade_escala3`) do item ao saldo do produto na tabela `produtos_estoque_cache`.
     - Insere um registro de movimentação do tipo `ENTRADA_COMPRA` na tabela `estoque_movimentacoes` (Kardex local).
   - Se `controla_estoque = 0`: O estoque físico e o Kardex são ignorados para este produto (útil para serviços ou produtos sem controle de estoque).
4. **Registro de Custo**: Atualiza o último custo base do produto na moeda local (BRL).
5. **Fila de Sincronização**: Insere os eventos `COMPRA_FINALIZADA` e `ESTOQUE_MOVIMENTACAO_GERADA` na tabela `sync_outbox`.

## Estorno de Mercadorias (Cancelamento Finalizado)

Caso uma nota fiscal de compra finalizada precise ser invalidada ou devolvida, o usuário pode cancelar a compra finalizada (`cancelar_compra_finalizada`). A operação é totalmente auditável e gera movimentações reversas compensatórias no estoque físico local e no Kardex:

1. **Validação de Estado**: A compra deve estar com o status `FINALIZADA`.
2. **Atualização de Status**: O status da compra muda para `CANCELADA` e os campos `cancelada_em` e `motivo_cancelamento` são preenchidos.
3. **Estorno de Estoque**:
   - Para cada item na compra (que não estivesse cancelado), o sistema verifica se `controla_estoque = 1`.
   - Se `controla_estoque = 1`:
     - Deduz a quantidade (`quantidade_escala3`) do produto na tabela `produtos_estoque_cache` (subtraindo do saldo atual, permitindo saldos negativos se necessário).
     - Insere uma nova movimentação do tipo `ESTORNO_ENTRADA_COMPRA` com quantidade com sinal inverso (`-quantidade_escala3`) na tabela `estoque_movimentacoes` (Kardex).
4. **Histórico Preservado**: A movimentação anterior `ENTRADA_COMPRA` original **nunca é excluída ou alterada**, garantindo a imutabilidade do histórico do Kardex.
5. **Fila de Sincronização**: Insere os eventos `COMPRA_CANCELADA` e `ESTOQUE_MOVIMENTACAO_GERADA` no outbox de sincronização.

## Kardex - Estrutura de Movimentações

As duas novas movimentações de Kardex criadas nesta fase são definidas na tabela `estoque_movimentacoes` com a seguinte lógica:

| Tipo Movimentação | Sinal da Quantidade | Origem ID | Descrição Histórica |
| :--- | :--- | :--- | :--- |
| `ENTRADA_COMPRA` | `+` (Positivo) | `compra_id` | Lançamento de nota de compra manual / Entrada de mercadorias |
| `ESTORNO_ENTRADA_COMPRA` | `-` (Negativo) | `compra_id` | Estorno automático devido a cancelamento de nota de compra |
