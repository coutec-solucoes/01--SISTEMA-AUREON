# Gestão de Sessão de Caixa

A sessão de caixa (`sessoes_caixa`) representa o turno do operador.

## Ciclo de Vida

### 1. Abertura (`abrir_caixa`)
- Exige identificação da `registradora_id` (terminal).
- A tabela `sessoes_caixa_moedas` recebe um `INSERT` para cada tipo de moeda declarada pelo operador. 
- O caixa assume status `ABERTO`. Um evento `CAIXA_ABERTO` é enfileirado na outbox.

### 2. Operação
- Vendas só podem ser iniciadas verificando ativamente a existência de um ID válido de sessão com status `ABERTO` (Relacionamento 1:N entre Sessão e Venda).

### 3. Fechamento (`fechar_caixa`)
Trata-se de um fechamento cego e apurado pelo sistema.
- O operador conta fisicamente as gavetas e informa o `saldos_fechamento` por moeda.
- O sistema calcula internamente o `valor_esperado_minor` realizando:
  - Soma de `valor_abertura` + Soma de pagamentos em `venda_pagamentos` originados por cada moeda nessa sessão - Soma de sangrias (futuro).
- O backend grava o `valor_esperado_minor`, o `valor_informado` e a diferença real.
- A sessão ganha o status `FECHADO` e o evento `CAIXA_FECHADO` é registrado na outbox.
