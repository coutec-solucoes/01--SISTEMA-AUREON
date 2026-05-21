# Impressão Operacional do PDV — Visão Geral

## Princípio Fundamental

A impressão no Aureon PDV é **estritamente uma saída documental**. Nenhum command de impressão executa `INSERT`, `UPDATE` ou `DELETE` em tabelas operacionais (exceto: registro pontual na `auditoria_pdv` na abertura de gaveta e registro na `reimpressoes` na reimpressão de venda).

---

## Destinos de Impressão Suportados

| Destino | Enum | Descrição |
|---|---|---|
| Simulador | `SIMULADOR` | Arquivo `.txt` local — padrão e recomendado para testes |
| Rede TCP/IP | `TCP_IP` | Impressora com IP fixo na rede local |
| Windows RAW | `WINDOWS_RAW` | Spooler Windows — stub, pendente |

O destino é configurado pelo operador via modal `ImpressoraDestinoModal` na tela `/reimpressoes`.

---

## Venda

**Command:** `imprimir_cupom_venda_nao_fiscal`

- Consulta `vendas` e `venda_itens` pelo `venda_id`.
- Exibe: cabeçalho da empresa, número da venda, data/hora, itens (quantidade × descrição × valor unitário × total), subtotal, desconto, acréscimo, taxa de entrega (se delivery), total geral.
- Exibe pagamentos registrados e troco.
- Exibe nome do cliente, se associado.
- Exibe obrigatoriamente: `DOCUMENTO NAO FISCAL` e `NAO E VALIDO COMO DOCUMENTO FISCAL`.
- Não altera a venda.

---

## Reimpressão

**Command:** `reimprimir_cupom_venda_nao_fiscal`

- Exige `motivo_reimpressao` obrigatório.
- Destaca `*** REIMPRESSAO ***` no cabeçalho do cupom.
- Registra a reimpressão na tabela `reimpressoes` (se existir no schema).
- Não cancela, altera ou estorna a venda original.
- Não afeta estoque, caixa ou financeiro.

---

## Financeiro

**Command:** `imprimir_comprovante_baixa_financeira`

- Consulta `financeiro_lancamentos` pelo `lancamento_id`.
- Exibe: tipo (`PAGAMENTO` ou `RECEBIMENTO`), forma de pagamento, moeda, valor, data.
- Não altera o lançamento financeiro.
- Documento não fiscal.

---

## Caixa

### Movimentação Avulsa
**Command:** `imprimir_comprovante_movimentacao_caixa`
- Tipos: `SANGRIA`, `SUPRIMENTO`, `VALE_FUNCIONARIO`.
- Exibe: tipo, moeda, valor, motivo, data/hora.

### Abertura de Caixa
**Command:** `imprimir_comprovante_abertura_caixa`
- Exibe: registradora, usuário, data/hora de abertura, saldos iniciais por moeda.

### Fechamento de Caixa
**Command:** `imprimir_comprovante_fechamento_caixa`
- Exibe: registradora, usuário, data/hora de fechamento, saldos por moeda.

### Resumo Gerencial
**Command:** `imprimir_resumo_gerencial_caixa`
- Exibe resumo completo da sessão: abertura, vendas, sangrias, suprimentos, vales e saldo esperado por moeda.

> Nenhum desses commands abre, fecha, corrige ou altera sessões de caixa.

---

## Produção

### Ticket de Produção
**Command:** `imprimir_ticket_producao`
- Consulta `producao_envios` e `producao_envios_itens`.
- Exibe: setor (via `setores_producao_cache`), origem (mesa/comanda/delivery), itens com quantidade e observação de produção.
- **Não exibe valores financeiros** — cozinha e bar não devem ver preços.
- Não altera o status do envio de produção.

### Ticket de Cancelamento
**Command:** `imprimir_ticket_cancelamento_producao`
- Exibe: origem, data/hora do cancelamento, item específico (se informado) ou "Cancelamento Geral".
- Exige `motivo` obrigatório.
- Não cancela o item — apenas gera o aviso para o setor de produção.

---

## Delivery

**Command:** `imprimir_romaneio_delivery`

- Consulta `delivery_operacional` e `delivery_itens`.
- Exibe: número do pedido, tipo (Entrega/Retirada), cliente, telefone, endereço, entregador (via `entregadores_cache`), itens, taxa de entrega, total.
- Não altera status do pedido delivery.
- Documento não fiscal.

---

## Gaveta

**Command:** `abrir_gaveta_dinheiro`

- Envia pulso ESC/POS (`ESC p`) para o conector da impressora.
- Exige `motivo` obrigatório na UI.
- Registra tentativa na `auditoria_pdv` (se a tabela existir).
- **Não cria sangria, suprimento, venda, lançamento financeiro ou movimentação de estoque.**
- Aviso na UI: "Abertura manual de gaveta não movimenta o saldo do caixa."
