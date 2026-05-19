# Precificação de Produtos

## 💵 Preço na Moeda Principal da Empresa
No modelo offline-first do Aureon, todo produto tem seu valor base fixado na **Moeda Principal da Empresa** (definida nas configurações corporativas gerais, tipicamente BRL no Brasil ou PYG no Paraguai). A base de dados expressa os valores financeiros em precisão decimal de quatro casas (`NUMERIC(15,4)`), prevenindo perdas por arredondamentos matemáticos.

---

## 🏷️ Estrutura de Preços do Produto
O cadastro principal do produto é composto por três pilares de precificação:
1.  **Preço de Custo (`preco_custo`)**: O valor líquido pago pela aquisição ou fabricação do item.
2.  **Margem de Lucro (`margem_lucro`)**: O markup percentual sobre o preço de custo desejado.
3.  **Preço de Venda (`preco_venda`)**: O valor final cobrado do consumidor.

---

## ⚡ Recálculo Visual Automático (UI Blazor)
Para otimizar a experiência do operador, a tela de produtos no Blazor WASM implementa reatividade bidirecional instantânea na aba de preços:
*   **Alteração de Custo ou Margem**: Atualiza o Preço de Venda na hora via fórmula:  
    `Preço de Venda = Preço de Custo * (1 + Margem / 100)`
*   **Alteração de Preço de Venda**: Recalcula a Margem de Lucro percentual automaticamente:  
    `Margem de Lucro = ((Preço de Venda - Preço de Custo) / Preço de Custo) * 100`

---

## 📈 Histórico Automático de Preços
Toda alteração de preços salva no backend Rust dispara um gatilho dentro da mesma transação SQL de persistência do produto, populando a tabela `produtos_historico_precos`.
*   Campos registrados: `data_alteracao`, `preco_custo_anterior`, `preco_custo_novo`, `preco_venda_anterior`, `preco_venda_novo`.
*   Na tela de edição de produtos, o operador pode consultar a aba **Histórico** para verificar todo o histórico inflacionário ou de reajustes do item.

---

## 🛑 Limites e Fora de Escopo da Precificação
A fim de simplificar os cadastros iniciais na Fase 4, **não foram implementados** nesta etapa:
*   Tabelas de preços adicionais por canais de venda (ex: Balcão vs. Delivery vs. Aplicativo).
*   Multimoeda dinâmica ativa no cadastro de produto (conversão de tabelas inteiras para o Paraguai). Todo produto é cadastrado e gravado na moeda principal da empresa.
