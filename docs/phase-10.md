# Fase 10 — Delivery Operacional

## Resumo

A Fase 10 implementa o módulo de **Delivery Operacional** do Aureon PDV. Permite que o estabelecimento gerencie pedidos de entrega e retirada diretamente no terminal local, sem dependência de app mobile de cliente, gateway de pagamento real ou integração com marketplaces externos.

O pagamento do pedido é processado **exclusivamente** pelo núcleo de venda da Fase 7, após a conversão do delivery em uma venda `EM_ANDAMENTO`.

---

## Status

```
FASE 10 — DELIVERY OPERACIONAL
STATUS: APROVADA E ENCERRADA COM RESSALVAS CONTROLADAS
Branch: main
```

---

## Blocos implementados

| Bloco | Descrição | Commit |
|-------|-----------|--------|
| Bloco 1 | Migration SQLite 008 — tabelas de delivery | 96474ba |
| Bloco 2 | Commands Rust/Tauri de operação | 5e6e83c |
| Bloco 3 | Conversão do delivery em venda EM_ANDAMENTO | 6be37e0 |
| Bloco 4 | UI Blazor — painel, detalhe, modal de criação | 94acbbf |
| Bloco 5 | Validação, documentação e commit final | ver abaixo |

---

## Migrations criadas

| Arquivo | Versão | Descrição |
|---------|--------|-----------|
| `008_fase10_delivery.sql` | 8 | Tabelas de delivery operacional, entregadores, itens e coluna `taxa_entrega_minor` em `vendas` |

---

## Tabelas criadas / alteradas

| Tabela | Operação | Descrição |
|--------|----------|-----------|
| `entregadores_cache` | CREATE | Cache de entregadores ativos |
| `delivery_operacional` | CREATE | Pedidos de delivery com status e rastreamento |
| `delivery_itens` | CREATE | Itens individuais dos pedidos |
| `vendas` | ALTER | Adicionada coluna `taxa_entrega_minor INTEGER NOT NULL DEFAULT 0` |

---

## Commands Rust/Tauri criados

Arquivo: `apps/aureon-pdv/src-tauri/src/commands_delivery.rs`

| Command | Descrição |
|---------|-----------|
| `listar_pedidos_delivery` | Lista todos os pedidos com filtros por status |
| `obter_pedido_delivery` | Retorna pedido + itens pelo ID |
| `listar_entregadores_delivery` | Lista entregadores ativos |
| `criar_pedido_local` | Cria novo pedido local (nasce ACEITO) |
| `aceitar_pedido_online` | Aceita pedido ONLINE em status NOVO |
| `recusar_pedido_online` | Recusa pedido com motivo (vira CANCELADO) |
| `atualizar_status_delivery` | Avança status conforme regras de transição |
| `definir_entregador` | Vincula entregador ativo ao pedido |
| `adicionar_item_delivery` | Adiciona item ao pedido (somente ativos) |
| `cancelar_item_delivery` | Cancela item com motivo e recalcula total |
| `fechar_delivery_em_venda` | Converte delivery em venda EM_ANDAMENTO |

---

## Telas Blazor criadas / alteradas

| Arquivo | Tipo | Descrição |
|---------|------|-----------|
| `Pages/DeliveryPdv.razor` | Página | Painel kanban por status, criação de pedido |
| `Pages/PedidoDeliveryDetalhe.razor` | Componente | Detalhe completo com itens, status, entregador e conversão |
| `Shared/NovoPedidoDeliveryModal.razor` | Modal | Criação de pedido local com tipo e taxa |
| `Shared/MainLayout.razor` | Alterado | Adicionado link "Delivery" na navegação |
| `Services/PdvModels.cs` | Alterado | Adicionados todos os DTOs de Delivery + `FechamentoEmVendaResp` |

---

## Eventos sync_outbox implementados

| Evento | Disparado quando |
|--------|-----------------|
| `DELIVERY_CRIADO` | Pedido local criado com sucesso |
| `DELIVERY_ACEITO` | Pedido online aceito pelo operador |
| `DELIVERY_RECUSADO` | Pedido online recusado com motivo |
| `DELIVERY_STATUS_ALTERADO` | Status avançado manualmente |
| `DELIVERY_ENTREGADOR_DEFINIDO` | Entregador vinculado ao pedido |
| `DELIVERY_ITEM_ADICIONADO` | Item adicionado ao pedido |
| `DELIVERY_ITEM_CANCELADO` | Item cancelado com motivo |
| `DELIVERY_CONVERTIDO_EM_VENDA` | Delivery convertido em venda EM_ANDAMENTO |

---

## Limitações conhecidas (ressalvas controladas)

1. **Status FECHADO ≠ PAGO:** O delivery vai para `FECHADO` ao converter em venda. A venda permanece `EM_ANDAMENTO` até a finalização no caixa (Fase 7). O operador deve ser orientado por meio de aviso visual na UI.
2. **Cancelamento de venda originada de delivery:** Se a venda `EM_ANDAMENTO` gerada a partir do delivery for cancelada no PDV, o delivery permanece com status `FECHADO`. Não há lógica automática de "reabrir" o delivery. Essa regra deve ser documentada e tratada em versão futura se necessário.
3. **Seed de desenvolvimento:** `seed_fase10_dev.sql` é exclusivamente para ambiente local/homologação. Jamais executar em produção.

---

## O que ficou fora do escopo (Fase 10)

- App mobile de cliente
- Gateway de pagamento real (Pix/TEF/cartão via API)
- Mapa ou roteirização de entregadores
- Integração real com iFood, Rappi ou qualquer marketplace
- Baixa de estoque ou Kardex
- Emissão fiscal (NF-e, NFC-e, SAT)
- Endpoints de API REST externos
- Cadastro completo de entregadores (apenas cache)
- Pagamento na entrega registrado no momento da entrega (ocorre no fechamento da venda no caixa)
