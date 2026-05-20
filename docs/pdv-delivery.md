# PDV Delivery — Visão Geral Operacional

## O que é o Delivery Operacional

O módulo de Delivery do Aureon PDV permite que o estabelecimento gerencie pedidos de entrega e retirada localmente, sem qualquer integração externa obrigatória. A operação é 100% offline-first: os eventos são registrados no `sync_outbox` local e sincronizados com o servidor quando a conexão estiver disponível.

---

## Tipos de pedido

| Tipo | Descrição |
|------|-----------|
| `RETIRADA` | Cliente retira no balcão. Entregador não é obrigatório. Endereço não é obrigatório. |
| `ENTREGA` | Produto é levado ao endereço do cliente. Endereço é obrigatório. Entregador deve ser definido antes de despachar. |

---

## Origem do pedido

| Origem | Descrição |
|--------|-----------|
| `LOCAL` | Criado pelo operador diretamente no PDV. Nasce já com status `ACEITO`. |
| `ONLINE` | Recebido de uma integração futura (marketplace, app). Nasce com status `NOVO` aguardando aceite ou recusa. |

---

## Fluxo operacional simplificado

```
[CRIAÇÃO LOCAL]
    → status: ACEITO

[CRIAÇÃO ONLINE]
    → status: NOVO
    → Operador: ACEITAR → ACEITO
    → Operador: RECUSAR → CANCELADO

[ACEITO]
    → Adicionar itens
    → Avançar para PREPARANDO

[PREPARANDO]
    → Avançar para PRONTO

[PRONTO]
    → Para ENTREGA: definir entregador
    → Avançar para DESPACHADO

[DESPACHADO / PRONTO / PREPARANDO / ACEITO]
    → Converter em venda EM_ANDAMENTO → [FECHADO]

[FECHADO]
    → Venda existe no PDV em status EM_ANDAMENTO
    → Pagamento ocorre pelo núcleo da Fase 7
    → Nenhuma alteração posterior permitida no delivery

[CANCELADO]
    → Pedido encerrado sem gerar venda
    → Nenhuma alteração posterior permitida
```

---

## Dados de um pedido

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `id` | UUID | Identificador único |
| `numero_pedido` | INTEGER | Sequencial, gerado automaticamente |
| `nome_cliente_informal` | TEXT | Nome informal do cliente |
| `telefone` | TEXT | Telefone de contato |
| `endereco_completo` | TEXT? | Obrigatório somente para ENTREGA |
| `tipo_pedido` | TEXT | `RETIRADA` ou `ENTREGA` |
| `status` | TEXT | Status atual (ver tabela de status) |
| `origem` | TEXT | `LOCAL` ou `ONLINE` |
| `entregador_id` | TEXT? | Vinculado antes de DESPACHADO (apenas ENTREGA) |
| `taxa_entrega_minor` | INTEGER | Taxa em minor units (centavos). Separada do consumo |
| `total_consumo_minor` | INTEGER | Soma dos itens ativos |
| `sessao_caixa_id` | TEXT? | Caixa que aceitou o pedido |
| `observacao` | TEXT? | Observação livre |
| `aberto_em` | TEXT | Timestamp ISO 8601 |
| `fechado_em` | TEXT? | Preenchido ao converter em venda |

---

## Taxa de entrega

A taxa de entrega é registrada separadamente em `delivery_operacional.taxa_entrega_minor` e, ao converter em venda, é transferida **exclusivamente** para `vendas.taxa_entrega_minor`.

**Ela jamais deve ser somada a `acrescimo_total_minor`**, pois isso quebraria a auditoria financeira (relatório de delivery, comissão de entregador, conferência de caixa, promoções de frete grátis).

```
total_venda = soma_itens_ativos + taxa_entrega_minor
```

---

## Regras de itens

- Itens cancelados (`cancelado = 1`) são marcados mas não removidos da tabela.
- Somente itens com `cancelado = 0` são copiados para `venda_itens` na conversão.
- O `total_consumo_minor` do pedido é recalculado a cada adição ou cancelamento de item.
- A quantidade é armazenada em escala 3 (ex: 1 unidade = 1000, 1,5 kg = 1500).
- Os valores monetários são armazenados em minor units (ex: R$ 10,50 = 1050).
