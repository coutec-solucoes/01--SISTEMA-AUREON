# 💵 Arquitetura Multimoeda — Guia de Referência

O Aureon é projetado especificamente para atuar na Tríplice Fronteira (Brasil, Paraguai, Argentina), o que exige robustez extrema em pagamentos com moedas flutuantes (Real - BRL, Dólar - USD, Guarani - PYG).

---

## 1. Moeda Principal vs Moedas Secundárias

*   **Moeda Principal**: É a moeda em que o estoque, o custo médio e todos os fechamentos de caixa contábeis são fechados. Normalmente BRL ou PYG.
*   **Moeda Secundária**: Moedas aceitas nas vendas físicas, cujas taxas de conversão são atualizadas diariamente.

---

## 2. Lançamento e Cálculo de Taxas

Para evitar perdas cambiais ou de arredondamento financeiro:
1.  O administrador lança a **Taxa Direta** de câmbio (Ex: `1 USD = 5.25 BRL`).
2.  O sistema automaticamente calcula a **Taxa Inversa** com alta precisão (`1 BRL = 0.190476 USD`) utilizando o tipo `rust_decimal` no backend Axum.
3.  Todas as cotações anteriores daquele dia e par de moedas são marcadas como `SUBSTITUIDA`, e a nova cotação entra como `ATIVA`.

---

## 3. Pagamento Múltiplo & Troco Flexível

*   **Pagamento Múltiplo**: O operador de caixa pode receber `R$ 50,00` em Real e `$ 10.00` em Dólar em uma mesma venda de `R$ 102,50`.
*   **Troco em Outra Moeda**: Se um cliente paga em Dólar, o sistema calcula o troco equivalente exato em Real, otimizando o fluxo de cédulas em caixa.
