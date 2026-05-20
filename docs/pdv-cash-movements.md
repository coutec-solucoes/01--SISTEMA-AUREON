# Fluxo de Caixa Operacional: Sangria, Suprimento e Vale Funcionário

Este documento descreve as movimentações financeiras operacionais no PDV, garantindo o controle rígido de numerário na gaveta do caixa.

---

## 💵 Tipos de Movimentações

As operações de caixa local registram a circulação física de dinheiro que não é proveniente de vendas diretas:

1. **Suprimento (Fundo de Troco)**: Injeção de valores na abertura ou durante o turno para garantir troco.
2. **Sangria (Retirada)**: Retirada de excesso de numerário para guarda segura (geralmente em cofres).
3. **Vale Funcionário**: Adiantamentos ou despesas emergenciais autorizadas pagas direto com dinheiro do caixa.

---

## 🏛️ Lógica de Fechamento Financeiro

O saldo esperado final por moeda da sessão de caixa (`SessaoCaixa`) é calculado de forma puramente inteira (*Minor Unit*) seguindo a regra matemática:

$$\text{Saldo Esperado} = \text{Abertura} + \text{Vendas em Dinheiro} + \text{Suprimentos} - \text{Sangrias} - \text{Vales}$$

As vendas realizadas por cartão ou Pix são listadas no resumo como esperado mas não compõem o saldo físico de gaveta de cédulas a ser descontado pelas sangrias/vales.

---

## 🔌 Commands Tauri Relacionados

- **`registrar_suprimento`**: Insere movimentação na tabela `caixa_movimentacoes` e atualiza a sessão de caixa.
- **`registrar_sangria`**: Registra a retirada.
- **`registrar_vale_funcionario`**: Registra o adiantamento.
- **`cancelar_movimentacao_caixa`**: Cancela de forma auditada um lançamento incorreto (exige autorização prévia de supervisor local).

---

## 🔄 Eventos de Sincronização (Outbox)

Toda movimentação efetuada gera eventos instantâneos e transacionais na fila de sync:
- `CAIXA_SUPRIMENTO_REGISTRADO`
- `CAIXA_SANGRIA_REGISTRADA`
- `CAIXA_VALE_FUNCIONARIO_REGISTRADO`
- `CAIXA_MOVIMENTACAO_CANCELADA`
