# Produção e Rotas (Gourmet)

Em estabelecimentos gastronômicos, os itens solicitados precisam ser roteados para suas respectivas praças de preparo (Cozinha, Bar, Copa, etc.).

O Aureon PDV centraliza isso através da tabela `producao_envios` e de seu endpoint `enviar_itens_producao`.

## O Processo de Envio

1. O caixa ou o garçom seleciona "Enviar Produção" na tela de detalhes da Mesa/Comanda.
2. O sistema busca na origem todos os itens onde `cancelado = false` e `enviado_producao = false`.
3. Para cada item válido, o sistema agrupa pela propriedade `local_producao_id` (Ex: "BAR", "COZINHA", "GERAL").
4. Para cada grupo (setor), gera-se um registro mestre de envio em `producao_envios`.
5. Os itens do grupo são marcados como `enviado_producao = true` no `gourmet_itens` e atrelados ao envio correspondente na tabela auxiliar `producao_envios_itens`.
6. Eventos `ITEM_ENVIADO_PRODUCAO` e `PRODUCAO_ENVIO_GERADO` são disparados no `sync_outbox`.

## Cancelamentos Pós-Envio

Se um item é cancelado ANTES de ir para produção, a lógica finaliza imediatamente a nível de conta.
Se o item JÁ FOI para produção, além de ser extirpado da conta do cliente, a cozinha/bar precisa ser notificada fisicamente. O cancelamento nesse caso gera um evento específico (`ITEM_CANCELAMENTO_ENVIADO_PRODUCAO` e `PRODUCAO_CANCELAMENTO_GERADO`) projetado para cuspir um ticket impresso com o rótulo **[CANCELAMENTO]** na impressora do setor.

## Geração de Ticket (KDS Mock)

Na Fase 9, optou-se por focar na persistência e na lógica de agrupamento. A impressão física (USB/Rede EPSON/Daruma) e a interface viva de Kitchen Display System (KDS) não foram exigidas.
No lugar disso, geramos um bloco em plain text (`gerar_texto_producao`) que simula fielmente a estrutura do papel térmico (Cabeçalho, Mesa Origem, Hora, Itens c/ Quantidade, Observações).

Isso prepara 100% da lógica para plugar um driver local de impressora na Fase 10/11, caso seja necessário, sem qualquer retrabalho nas lógicas de agrupamento e estado do Aureon.
