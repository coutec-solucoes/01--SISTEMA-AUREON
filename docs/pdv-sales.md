# Fluxo de Vendas no PDV

O módulo de vendas foi desenhado para ser invulnerável à perdas de sequência de numeração oficial.

## Início (`iniciar_venda`)
Cria um cabeçalho de venda na tabela `vendas` com `numero_venda = NULL` e status `EM_ANDAMENTO`. Identificada internamente pelo UUID local. A numeração oficial legal não é reservada para orçamentos ou vendas abandonadas.

## Adição de Itens (`adicionar_item_venda`)
Cada item adicionado recomputa automaticamente no backend o subtotal da venda principal (`total_minor = SUM(itens)`). 
Para a quantidade, adotou-se o modelo `quantidade_escala3`. Isso elimina pontos flutuantes ao registrar um peso na balança, sendo `1,452 kg` registrado nativamente como `1452`.

## Cancelamentos (Auditoria Completa)
O cancelamento de item (`cancelar_item_venda`) ou da venda completa (`cancelar_venda`) exige:
- `usuario_cancelamento_id` (rastreabilidade de quem executou).
- `motivo_cancelamento` (descrição justificativa).
- Campos opcionais como `supervisor_id` para aprovações.
Ambas as ações geram log na `sync_outbox`.

## Finalização (`finalizar_venda`)
Apenas ocorre se o `SUM(pagamentos_minor)` for maior ou igual ao `total_venda_minor`.
Durante essa transação, a tabela `controle_numeracao` é lida, incrementada atomicamente e o número oficial é finalmente afixado na coluna `numero_venda`.
