# PDV Gourmet (Visão Geral)

O módulo **PDV Gourmet** permite ao Aureon PDV gerenciar operações contínuas (contas em aberto) típicas de bares, restaurantes, lanchonetes e casas noturnas.

Diferente do "Caixa Rápido" (Fase 7), onde a venda nasce e morre imediatamente em frente ao cliente, no fluxo Gourmet a "Venda" é adiada até o momento do pagamento. O consumo é registrado em recipientes chamados **Mesas** ou **Comandas**.

## Arquitetura de Estado

1. **Origens**: O cliente consome utilizando uma `Mesa` (física no salão) ou uma `Comanda` (cartão enumerado ou QR Code).
2. **Registro de Consumo**: Os itens consumidos são inseridos no recipiente de origem (`gourmet_itens`). Estes itens não são vendas fiscais ainda.
3. **Produção**: O sistema entende a necessidade de preparo. Quando os itens são despachados (`enviar_itens_producao`), o PDV agrupa tudo por praça (ex: Cozinha, Bar) e emite ordens (mockadas como TXT).
4. **Transformação em Venda**: No momento em que o cliente pede a conta, a Mesa/Comanda é "Fechada em Venda". O Aureon PDV pega todo o consumo não cancelado, cria um espelho desse consumo na estrutura de Venda (`vendas` / `vendas_itens`) com status `EM_ANDAMENTO`.
5. **Pagamento (Fase 7)**: O caixa processa os pagamentos. Somente quando a venda finaliza e quita o total, a Mesa/Comanda original assume o status final `FECHADA`.

## Eventos e Tolerância a Falhas

Como todo o ecossistema Aureon, o PDV Gourmet atua de maneira tolerante a falhas na comunicação (`Offline First`). Toda abertura, adição, transferência e fechamento gera um evento no `sync_outbox` (`MESA_ABERTA`, `MESA_ITEM_ADICIONADO`, `COMANDA_CONVERTIDA_EM_VENDA`, etc.), que subirá ao Master Data em background (Fase 6).

## Bloqueio Operacional

Para manter a consistência, uma Mesa ou Comanda que possui uma Venda `EM_ANDAMENTO` entra em modo de bloqueio lógico:
- Não pode receber novos itens.
- Não pode cancelar itens.
- Não pode ser transferida.

Isso previne a corrupção de valores que já foram apresentados ao cliente para pagamento. Caso o cliente desista do pagamento e queira continuar consumindo, a venda `EM_ANDAMENTO` deve ser cancelada no caixa (`cancelar_venda`), o que devolve o recipiente ao estado de edição.
