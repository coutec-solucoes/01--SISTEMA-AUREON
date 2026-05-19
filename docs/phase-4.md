# Fase 4 — Cadastros Base

## 📋 Resumo da Fase 4
A **Fase 4 — Cadastros Base** consolida os pilares cadastrais do Sistema Aureon em um modelo offline-first de alto desempenho. Foram implementadas as APIs transacionais robustas em **Rust (Actix/PgPool/sqlx)** e as respectivas interfaces de retaguarda ricas e glassmórficas em **Blazor WebAssembly (WASM)**, cobrindo todo o ciclo de vida de **Pessoas** (com múltiplos papéis), **Catálogo de Produtos**, **Tributação Base**, **Controle de Estoque Cadastral**, e o ecossistema de alimentação especial (pizzas, combos, adicionais e locais de produção física).

---

## 🧱 Blocos Implementados
1.  **Bloco 1**: Estrutura de banco de dados (14 migrations PostgreSQL) e auditoria de cadastros unificada.
2.  **Bloco 2**: API Rust de Pessoas com múltiplos papéis, endereços, contatos, permissões de segurança por papéis e publicação de eventos.
3.  **Bloco 3**: API Rust de Produtos com histórico automático de preços, pizzas, combos, adicionais, locais de produção e fiscal base.
4.  **Bloco 4**: Interface Blazor WASM de Pessoas, Clientes, Fornecedores, Funcionários, Vendedores, Entregadores e Transportadoras com componente unificado e filtros dinâmicos reativos.
5.  **Bloco 5**: Interface Blazor WASM de Produtos & Estruturas Auxiliares (Grupos, Subgrupos, Marcas, Pizza Sabores, Combos, Adicionais e Locais) com modal dinâmico de 10 abas.

---

## 🚀 Commits dos Blocos
*   **Bloco 1**: `db6789f`
*   **Bloco 2**: `f2ff012`
*   **Bloco 3**: `642b896`
*   **Bloco 4**: `e03f745`
*   **Bloco 5**: `aa6ece7`

---

## 🗄️ Migrations & Tabelas Criadas
Foram criadas 14 migrations na pasta `database/migrations/postgresql/` estruturando as seguintes tabelas:
1.  `pessoas` — Cadastro principal das entidades.
2.  `pessoas_papeis` — Mapeamento N:N de papéis ativos da pessoa.
3.  `pessoas_contatos` — Telefones, whatsapp, e-mail e site.
4.  `pessoas_enderecos` — Endereços nacionais e internacionais (PY).
5.  `clientes_configuracoes` — Limites e travas de crediário/venda a prazo.
6.  `fornecedores_configuracoes` — Moeda padrão de compra e prazos.
7.  `funcionarios_configuracoes` — Dados de admissão, cargo e salários.
8.  `vendedores_configuracoes` — Códigos de comissionamento ativo (fixo ou %).
9.  `entregadores_configuracoes` — Controle de veículos, placas e tipos.
10. `transportadoras_configuracoes` — Contatos de logística e rotas.
11. `auditoria_cadastros` — Registro de auditoria imutável detalhado.
12. `eventos_publicacao` — Fila transacional para integradores externos futuros.
13. `produtos` — Dados principais do cadastro do produto.
14. `produtos_historico_precos` — Log de modificações de preços automático.
15. `sabores_pizza` — Sabores das pizzas.
16. `produtos_sabores_precos` — Tabela de preços de sabores por tamanho.
17. `produtos_combos_itens` — Itens vinculados aos produtos combos.
18. `adicionais` — Cadastro principal de adicionais / complementos.
19. `produtos_adicionais_vinculos` — Mapeamento e preços diferenciados.
20. `locais_producao` — Mapeamento de destinações físicas (Bar, Cozinha, Copa).

---

## 🔗 Endpoints da API Local (Rust)
*   `GET /cadastros/pessoas` | `POST /cadastros/pessoas`
*   `GET /cadastros/pessoas/{id}` | `PUT /cadastros/pessoas/{id}`
*   `PUT /cadastros/pessoas/{id}/inativar`
*   `GET /cadastros/pessoas/papel/{papel}` (Clientes, Fornecedores, etc.)
*   `GET /cadastros/produtos` | `POST /cadastros/produtos`
*   `GET /cadastros/produtos/{id}` | `PUT /cadastros/produtos/{id}`
*   `PUT /cadastros/produtos/{id}/inativar`
*   `GET /cadastros/produtos/{id}/historico-precos`
*   `GET` / `POST` / `PUT` estruturais auxiliares (`/grupos`, `/subgrupos`, `/marcas`, `/sabores-pizza`, `/combos`, `/adicionais`, `/locais-producao`).

---

## 🖥️ Telas Blazor Criadas (Retaguarda)
1.  **Pessoas & Papéis (`/cadastros/pessoas`)**: Componente dinâmico único com filtros inteligentes e modal de abas para gerenciamento detalhado.
2.  **Produtos (`/cadastros/produtos`)**: Catálogo reativo com buscador de alta performance e modal de 10 abas para controle granular.
3.  **Grupos & Marcas (`/cadastros/produtos/grupos`)**: Painel de visualização e edição de Categorias estruturadas.
4.  **Estruturas Auxiliares (`/cadastros/produtos/sabores-pizza`)**: Tela de sabores, adicionais, locais de produção e associações de combos.

---

## 🛡️ Regras de Negócio & Segurança
*   **Permissões (API & UI)**: As requisições são blindadas via tokens JWT validados nos handlers através da função `tem_permissao(..., "CADASTROS_PESSOAS/PRODUTOS", "LER/CRIAR/EDITAR")`. Na UI, os botões e formulários são ocultados ou desativados reativamente baseados nos claims do usuário.
*   **Auditoria Automática**: Cada inserção ou modificação de dados gera logs imutáveis na tabela `auditoria_cadastros` especificando a ação, a data, o usuário responsável e os payloads de valor anterior e novo.
*   **Mensagens Amigáveis**: Erros de banco como documentos (`uq_pessoas_cpf`, etc.), códigos de barra ou referências duplicadas são interceptados na API e exibidos ao usuário final de forma amigável através de alertas estilizados.

---

## 🛑 Limitações & Fora de Escopo
O escopo da Fase 4 é estritamente de **infraestrutura cadastral**. Ficaram fora do escopo desta fase:
*   Módulo de vendas e telas de PDV real.
*   Baixa real de estoque, Kardex e inventário de perdas.
*   Lançamentos financeiros reais de contas a pagar/receber.
*   Emissão real de notas fiscais (NF-e, NFC-e ou IVA no PY).
*   Sincronização em tempo real de banco de dados offline com servidores remotos.
