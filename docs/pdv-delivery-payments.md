# PDV Delivery — Pagamentos e Integração com o Caixa

## Princípio fundamental

O módulo de Delivery **não processa pagamentos diretamente**. Todo pagamento ocorre pelo núcleo de venda da **Fase 7**, após a conversão do delivery em uma venda local.

---

## Fluxo de pagamento

```
1. Operador finaliza o pedido de delivery
   → Chama fechar_delivery_em_venda

2. Sistema cria venda local:
   - status = EM_ANDAMENTO
   - numero_venda = NULL
   - tipo_venda = DELIVERY
   - origem_tipo = DELIVERY
   - origem_id = delivery_id
   - taxa_entrega_minor = delivery.taxa_entrega_minor  ← separada!

3. Itens ativos do delivery são copiados para venda_itens
   - origem_item_id = delivery_itens.id (rastreabilidade)

4. Delivery vai para status FECHADO

5. Sistema redireciona operador para /pdv

6. Operador processa pagamento normalmente pelo PDV
   → Fluxo idêntico ao de uma venda de balcão

7. Ao finalizar o pagamento:
   - venda.status → FINALIZADA
   - venda.numero_venda → preenchido (sequencial)
   - venda.finalizado_em → timestamp

8. Pagamento registrado em venda_pagamentos
   (Fase 7 — sem alteração neste módulo)
```

---

## Cálculo do total da venda

```
subtotal_minor = soma(total_item_minor para itens ativos) + desconto_total - acrescimo_total
taxa_entrega_minor = delivery_operacional.taxa_entrega_minor
total_minor = subtotal_minor + taxa_entrega_minor
```

### Regra importante

A `taxa_entrega_minor` **não é somada a `acrescimo_total_minor`**. Ela fica em coluna própria (`vendas.taxa_entrega_minor`), permitindo:

- Relatórios de receita de taxa de entrega separados do consumo
- Cálculo de comissão de entregadores
- Promoções de frete grátis sem afetar a apuração de acréscimos
- Conferência de caixa com breakdown correto

---

## Pagamento na entrega

Quando o cliente paga ao entregador (dinheiro, cartão na maquininha portátil):

1. O entregador retorna ao estabelecimento
2. O operador abre a venda `EM_ANDAMENTO` no PDV
3. Registra o pagamento normalmente (dinheiro/cartão/Pix)
4. O PDV finaliza a venda e gera o `numero_venda`

> O sistema **não** registra pagamento no momento da entrega física. Essa limitação é intencional na Fase 10.

---

## Pagamento na retirada

O cliente paga no balcão no momento da retirada:

1. Operador abre a venda `EM_ANDAMENTO` no PDV
2. Processa o pagamento normalmente
3. PDV finaliza a venda

---

## numero_venda

- Nasce **NULL** na criação da venda
- É preenchido **exclusivamente** no fechamento financeiro (Fase 7)
- A UI do delivery nunca exibe nem gera o número da venda
- O operador não deve informar `numero_venda` ao cliente antes do pagamento

---

## Venda cancelada após conversão

Se a venda `EM_ANDAMENTO` for cancelada no PDV:

- O delivery permanece `FECHADO` (não é revertido automaticamente)
- Não há `numero_venda` gerado
- O operador deve criar um novo pedido se necessário
- Esta situação deve ser tratada operacionalmente por supervisão

---

## O que NÃO está implementado (Fase 10)

| Funcionalidade | Motivo de exclusão |
|---------------|-------------------|
| Gateway Pix/TEF real | Fora do escopo operacional local |
| Integração com maquininha portátil | Requer hardware específico |
| Registro de pagamento na entrega | Limitação intencional da Fase 10 |
| Parcelamento ou split de pagamento | Tratado no núcleo financeiro futuro |
| Reembolso/estorno de taxa de entrega | Tratado em versão futura |
