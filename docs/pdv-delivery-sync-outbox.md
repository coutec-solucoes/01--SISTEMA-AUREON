# PDV Delivery — Eventos sync_outbox

## Visão geral

Todos os eventos de delivery são registrados atomicamente no `sync_outbox` local, dentro da mesma transação SQLite que altera os dados operacionais. Isso garante consistência entre o estado local e o que será sincronizado com o servidor quando a conexão estiver disponível.

---

## Tabela de eventos

| Evento | Command que dispara | Payload principal |
|--------|---------------------|-------------------|
| `DELIVERY_CRIADO` | `criar_pedido_local` | `numero_pedido`, `origem`, `status` |
| `DELIVERY_ACEITO` | `aceitar_pedido_online` | `delivery_id`, `status` |
| `DELIVERY_RECUSADO` | `recusar_pedido_online` | `delivery_id`, `motivo` |
| `DELIVERY_STATUS_ALTERADO` | `atualizar_status_delivery` | `delivery_id`, `status` |
| `DELIVERY_STATUS_ALTERADO` | `fechar_delivery_em_venda` | `delivery_id`, `status: FECHADO` |
| `DELIVERY_ENTREGADOR_DEFINIDO` | `definir_entregador` | `delivery_id`, `entregador_id` |
| `DELIVERY_ITEM_ADICIONADO` | `adicionar_item_delivery` | `item_id`, `delivery_id`, `total_item_minor` |
| `DELIVERY_ITEM_CANCELADO` | `cancelar_item_delivery` | `item_id`, `delivery_id`, `motivo` |
| `DELIVERY_CONVERTIDO_EM_VENDA` | `fechar_delivery_em_venda` | `delivery_id`, `venda_id`, `total_venda_minor` |

---

## Estrutura do sync_outbox

```sql
CREATE TABLE sync_outbox (
    id              TEXT PRIMARY KEY,
    idempotency_key TEXT NOT NULL UNIQUE,
    event_type      TEXT NOT NULL,
    payload         TEXT NOT NULL,  -- JSON serializado
    status          TEXT NOT NULL DEFAULT 'PENDENTE',
    criado_em       TEXT NOT NULL,
    enviado_em      TEXT
);
```

---

## Garantias de consistência

### Atomicidade
Todos os eventos são inseridos dentro do mesmo `transaction()` do Rusqlite que realiza as alterações nos dados operacionais. Se a transação falhar, o evento não é inserido. Se o evento falhar, a transação é revertida.

### Idempotência
Cada evento recebe um `idempotency_key` gerado por `Uuid::new_v4()`. O servidor pode usar esta chave para descartar reprocessamentos em caso de retry.

### Ordem de eventos
Os eventos são inseridos sequencialmente no mesmo lote transacional, garantindo que o servidor processe na ordem correta ao sincronizar.

---

## Exemplo de payload — DELIVERY_CONVERTIDO_EM_VENDA

```json
{
  "delivery_id": "a1b2c3d4-...",
  "venda_id": "e5f6g7h8-...",
  "total_venda_minor": 4550
}
```

> `total_venda_minor` já inclui a `taxa_entrega_minor`.

---

## Exemplo de payload — DELIVERY_ITEM_ADICIONADO

```json
{
  "item_id": "uuid-do-item",
  "delivery_id": "uuid-do-pedido",
  "total_item_minor": 1200
}
```

---

## Eventos de status duplicados intencionalmente

O evento `DELIVERY_STATUS_ALTERADO` é disparado tanto pelo `atualizar_status_delivery` quanto pelo `fechar_delivery_em_venda` (ao marcar como `FECHADO`). Isso é intencional: o servidor pode rastrear a linha do tempo completa do status, incluindo a transição final para `FECHADO` gerada pela conversão em venda.

---

## Sincronização futura

A sincronização dos eventos do `sync_outbox` com o servidor é responsabilidade de um worker de background (não implementado na Fase 10). Os eventos ficam com `status = 'PENDENTE'` até serem enviados e confirmados pelo servidor, que então marca `status = 'ENVIADO'` e preenche `enviado_em`.
