# Transferências (Mesas e Comandas)

A dinâmica de salão comumente exige a movimentação de contas entre recipientes (ex: Juntar mesas, mover consumo da comanda do bar para a mesa do salão, rachar itens, etc.).

O Aureon lida com as transferências de forma transacional e estrita, evitando duplicidade de saldos ou itens "presos no limbo".

## Tipos de Transferência

### 1. Transferência Total
Pega-se a totalidade dos itens não-cancelados de uma origem e repassa-se ao destino.
1. O destino obrigatoriamente já deve estar `ABERTO` na base.
2. Os itens da origem têm seus `origem_id` alterados para a nova mesa/comanda.
3. O status da origem passa para `FECHADA`.
4. Evento de `TRANSFERIDA` é disparado.

### 2. Transferência Parcial
Permite ao usuário selecionar (via UI com checkboxes) itens específicos de uma conta origem para migrar.
1. O destino deve estar `ABERTO`.
2. Apenas os itens da lista passada (`itens_ids`) têm seu `origem_id` alterado.
3. A origem permanece `ABERTA` (pois houve consumo remanescente).
4. O evento gerado é `ITENS_TRANSFERIDOS`.

## Rastreabilidade

Ao longo de transferências excessivas, pode-se perder o rastro de onde um drink foi lançado inicialmente. Por isso, toda movimentação cria registros em:
- `gourmet_transferencias` (Log do cabeçalho da ação: Quem transferiu, a hora, de onde para onde).
- `gourmet_transferencias_itens` (Quais IDs de produto sofreram a mudança no lote).

Isto garante que um auditor ou gerente possa refazer o caminho lógico do item se houver divergência no salão.
