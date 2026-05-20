# PDV Delivery — Máquina de Estados

## Estados do delivery

| Status | Descrição |
|--------|-----------|
| `NOVO` | Pedido online recebido aguardando aceite ou recusa |
| `ACEITO` | Pedido aceito pelo operador. Pedidos locais nascem aqui |
| `PREPARANDO` | Em preparo na cozinha/setor |
| `PRONTO` | Pronto para entrega ou retirada |
| `DESPACHADO` | Saiu para entrega (apenas ENTREGA) |
| `FECHADO` | Convertido em venda EM_ANDAMENTO. Pagamento ocorre no PDV |
| `CANCELADO` | Pedido cancelado. Sem geração de venda |

---

## Transições permitidas

```
NOVO ──────────────► ACEITO
NOVO ──────────────► CANCELADO

ACEITO ─────────────► PREPARANDO
ACEITO ─────────────► FECHADO *
ACEITO ─────────────► CANCELADO

PREPARANDO ─────────► PRONTO
PREPARANDO ─────────► FECHADO *
PREPARANDO ─────────► CANCELADO

PRONTO ─────────────► DESPACHADO (ENTREGA com entregador obrigatório)
PRONTO ─────────────► FECHADO *
PRONTO ─────────────► CANCELADO

DESPACHADO ─────────► FECHADO *
DESPACHADO ─────────► CANCELADO

FECHADO ─────────────► [BLOQUEADO — sem transição]
CANCELADO ───────────► [BLOQUEADO — sem transição]
```

> *`FECHADO` é atingido pelo command `fechar_delivery_em_venda`, não pelo `atualizar_status_delivery`.

---

## Regras de transição

### Pedido LOCAL
- Nasce diretamente em `ACEITO`.
- Não passa por `NOVO`.

### Pedido ONLINE
- Nasce em `NOVO`.
- Operador deve explicitamente aceitar (`aceitar_pedido_online`) ou recusar (`recusar_pedido_online`).
- Recusa exige motivo e resulta em `CANCELADO`.

### DESPACHADO (somente ENTREGA)
- Exige entregador vinculado (`entregador_id IS NOT NULL`).
- A UI bloqueia o botão de avanço para DESPACHADO se o entregador não estiver definido.
- Para RETIRADA, o status DESPACHADO não se aplica — pode ir direto para FECHADO.

### FECHADO
- Atingido **exclusivamente** por `fechar_delivery_em_venda`.
- Significa: *pedido convertido em venda local EM_ANDAMENTO, aguardando pagamento*.
- Não significa que o pagamento já ocorreu.
- Não é possível alterar itens, status ou entregador após FECHADO.

### CANCELADO
- Pedido encerrado sem geração de venda.
- Não é possível qualquer alteração posterior.

---

## Validação de dupla conversão

O command `fechar_delivery_em_venda` verifica, **antes de qualquer operação**, se já existe uma venda com:

```sql
origem_tipo = 'DELIVERY'
AND origem_id = delivery_id
AND status IN ('EM_ANDAMENTO', 'FINALIZADA')
```

Se existir, retorna erro imediato. Isso impede duplicações mesmo em cenários de falha parcial ou retry.

---

## FECHADO no delivery ≠ PAGO

Esta distinção é fundamental para a operação:

| Situação | Status Delivery | Status Venda |
|----------|----------------|--------------|
| Pedido sendo preparado | PREPARANDO | — |
| Pedido convertido, aguardando pagamento | FECHADO | EM_ANDAMENTO |
| Pagamento realizado no caixa | FECHADO | FINALIZADA |

O `numero_venda` é gerado **somente** na finalização do pagamento (Fase 7). Enquanto a venda estiver `EM_ANDAMENTO`, `numero_venda = NULL`.

---

## Cancelamento da venda originada de delivery

Se a venda `EM_ANDAMENTO` gerada a partir de um delivery for cancelada no PDV:

- O delivery permanece com status `FECHADO`.
- Não há lógica automática de "reabrir" o delivery ou recriá-lo.
- Comportamento documentado: o operador deve criar um novo pedido manualmente se necessário.
- Esta regra será revisada em versão futura se o volume de casos operacionais justificar automação.
