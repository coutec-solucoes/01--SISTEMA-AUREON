# Cadastro de Produtos & Categorias

## 📂 Organização por Grupos, Subgrupos e Marcas
O catálogo de produtos do Aureon possui uma estrutura de categorização em três níveis:
1.  **Grupos (`grupos`)**: A classificação de nível mais alto (ex: Bebidas, Pizzas, Mercearia). A vinculação a um grupo é de preenchimento obrigatório para qualquer produto.
2.  **Subgrupos (`subgrupos`)**: Permite refinar a categorização interna de um grupo (ex: Grupo *Bebidas* -> Subgrupo *Refrigerantes*, *Cervejas*, *Vinhos*). É opcional e dependente do grupo pai.
3.  **Marcas (`marcas`)**: Identificação da marca fabricante do produto (ex: Coca-Cola, Heinz). É opcional.

---

## 📦 Cadastro Principal do Produto
A tabela `produtos` centraliza as informações cadastrais e as propriedades básicas do item comercializado, tais como:
*   `descricao` (Nome curto exibido no cupom e telas de PDV)
*   `descricao_detalhada` (Especificações detalhadas, peso, composição ou receita)
*   `referencia` (Código alfanumérico interno exclusivo da loja)
*   `codigo_barras` (Código de barras oficial de padrão EAN/GTIN)
*   `unidade_medida` (Unidade padrão de movimentação: UN, KG, LT, CX, etc.)

---

## 📊 Controle de Estoque Base Cadastral
O controle de estoque cadastral inicial está estruturado com os seguintes atributos na tabela principal:
*   `controla_estoque` (Flag booleana indicando se o sistema deve rastrear saldo para este item)
*   `estoque_atual_base` (Saldo físico em estoque inicial na moeda ou unidade padrão)
*   `estoque_minimo` (Limite inferior de alerta de reposição nas listagens da retaguarda)

---

## ⚖️ Integração com Balança e Pesagem
O sistema está preparado para ambientes de açougue, padaria e hortifrúti através das flags de balança:
*   `produto_balanca` (Indica que o produto é vendido por peso e deve ser lido ou pesado no caixa)
*   `reconfirmacao_pesagem` (Exige que a balança do caixa reconfirme o peso antes da finalização)
*   `leitura_etiqueta` (Habilita o decodificador interno de códigos de barra gerados por balanças rotuladoras de gôndola, interpretando preço ou peso embutidos na etiqueta)

---

## 📅 Validade, Lotes e Produção Física
*   **Controle de Validade (`controla_validade`)**: Flag indicadora de que o produto possui data de vencimento restrita, preparando a estrutura para receber tabelas de lotes futuros.
*   **Local de Produção (`local_producao_id`)**: Associa o produto a uma praça de destinação física específica (Cozinha, Bar, Copa, etc.). Nas fases seguintes, ao lançar um pedido de mesa ou PDV, a retaguarda direcionará as impressões físicas de produção diretamente para o canal selecionado.

---

## 🛑 Status Ativo / Inativo e Catálogo Futuro
Os produtos inativos (`ativo = false`) são ocultados dos mecanismos de busca de vendas rápidas e PDVs locais, mantendo o histórico de relatórios intacto. Esta flag prepara a base de dados para atuar em sincronia de catálogo híbrido futuro (atualização dinâmica de preços para o caixa sem interrupção operacional).
