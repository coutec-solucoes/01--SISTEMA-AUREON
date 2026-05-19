# Estruturas Especiais: Pizzas, Combos e Adicionais

## 🍕 Estrutura de Sabores & Tamanhos de Pizza
Para atender pizzarias de forma nativa e profissional, o Aureon implementa um ecossistema estrutural de pizzas:
*   **Sabores (`sabores_pizza`)**: Cadastro de sabores independentes da especificação de tamanho (ex: Calabresa, Margherita).
*   **Preços por Sabor/Tamanho (`produtos_sabores_precos`)**: Permite vincular preços de venda específicos para cada combinação de Sabor e Tamanho de Pizza (GIGANTE, GRANDE, MEDIA, PEQUENA, BROTO).

---

## 🍔 Estrutura de Combos (Kits)
O sistema suporta a venda agrupada de múltiplos produtos através do cadastro de **Combos**:
*   **Produto Principal**: O produto agregador do combo (que possui a flag `produto_combo = true`).
*   **Componentes de Combo (`produtos_combos_itens`)**: Associação N:N contendo quais produtos fazem parte deste combo, a respectiva `quantidade` e se há algum `preco_adicional` para cada item selecionado.

---

## ➕ Estrutura de Adicionais & Complementos
Ideal para lanchonetes e restaurantes personalizarem pedidos de forma elegante:
*   **Adicionais (`adicionais`)**: Cadastro global de ingredientes adicionais com seus preços padrão (ex: Bacon, Queijo Extra, Molho Especial).
*   **Vínculos Produto/Adicional (`produtos_adicionais_vinculos`)**: Mapeia quais adicionais são válidos para um produto específico (só visível se `permite_adicionais = true`), permitindo também fixar um `preco_venda_diferenciado` se aplicável.

---

## 🛑 Limitações e Escopo desta Fase
> [!WARNING]
> Esta infraestrutura serve para **configuração cadastral** na retaguarda.
> *   **Sem Venda Multisabores**: O PDV não realiza a montagem dinâmica de pizzas de dois ou mais sabores (meio-a-meio) nesta fase.
> *   **Sem Baixa de Estoque de Combo**: A venda do produto combo principal não explode em baixas automáticas de estoque físico de seus componentes associados nesta fase.
> *   **Sem Cobrança de Adicionais**: O PDV ainda não renderiza o modificador visual de adicionais na hora da venda.
