# Mesas e Comandas (Tabs)

## Mesas
Entidade que representa um local físico no estabelecimento. Geralmente identificada por um número, com capacidade fixa de clientes, mantida de forma estática no Master Data (`mesas_cache`).

**Fluxo de Vida:**
1. **Livre**: A mesa consta no cache, mas não há operação aberta (`mesas_operacionais`). O PDV infere este status na interface.
2. **Aberta / Reservada**: Um garçom ou o caixa ativa a mesa (`abrir_mesa`). Status `ABERTA`. Pode-se informar o nome do cliente.
3. **Em Venda**: Ao solicitar a conta, converte-se em venda. O PDV bloqueia manipulações até o término do pagamento.
4. **Fechada**: O pagamento finaliza a venda. A mesa marca-se como `FECHADA` (historizada para sincronização) e o PDV limpa a interface. A próxima consulta de mesas no cache, ao cruzar com as operacionais em aberto, a trará novamente como `LIVRE`.

## Comandas
Entidade flutuante, podendo ser atrelada a uma mesa ou operada de forma avulsa (ex: baladas).

**Fluxo de Vida:**
1. **Livre**: Semelhante à mesa. O Master Data fornece as numerações possíveis (`comandas_cache`).
2. **Aberta**: É atrelada ao consumo ativo do cliente.
3. **Bloqueada**: Status especial caso o cartão físico da comanda seja perdido ou furtado (`bloquear_comanda`). Impede que seja fechada ou manipulada até desbloqueio por supervisor.
4. **Fechada**: Ciclo idêntico ao da mesa após o pagamento da venda originária.

## Implementação Técnica
As entidades compartilham 90% da lógica de negócio. Para facilitar a manutenção sem engessar a base em polimorfismos excessivos em SQL, foram mantidas separadas em tabelas:
- `mesas_operacionais` / `gourmet_itens` (com origem_tipo = 'MESA')
- `comandas_operacionais` / `gourmet_itens` (com origem_tipo = 'COMANDA')
