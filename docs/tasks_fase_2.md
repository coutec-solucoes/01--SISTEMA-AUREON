# 📋 Cronograma de Tarefas — AUREON (Fase 2)
Este documento descreve os blocos de desenvolvimento e a ordem correta de execução para a **Fase 2 — Configuração da Empresa, Multimoeda e Fiscal Base**.

---

## 🟩 BLOCO 1: Estrutura Física de Dados (PostgreSQL) — [CONCLUÍDO]
*   [x] Criar arquivo de migration PostgreSQL `004_configuracao_empresa.sql`.
*   [x] Mapear tabelas de dados gerais, identificação, contatos, endereços, logos, multimoedas, cotações, parâmetros, fiscal base (Brasil/Paraguai), auditoria e eventos de sync.
*   [x] Atualizar a constante `MIGRATIONS_PG` no Tauri `commands.rs` para executar a nova migration de forma nativa.
*   [x] Compilar (`tauri build`) e testar a aplicação no PostgreSQL local (Tabelas criadas com sucesso no banco `couto_bd`).

---

## 🟩 BLOCO 2: Estrutura e Endpoints da API Local Rust (Axum) — [CONCLUÍDO]
*   [x] Configurar as dependências necessárias no backend (como `rust_decimal` para cotações e `serde_json` para JSONB).
*   [x] Criar os **Models, DTOs e Validadores** (em snake_case) no Rust para representar:
    *   Configurações da Empresa, Contatos, Endereços, Logos.
    *   Moedas (BRL, PYG, USD) e taxas (Cotação direta e inversa).
    *   Parâmetros operacionais e eventos de auditoria.
*   [x] Implementar as camadas de **Repository e Service** para processamento lógico:
    *   `EmpresaService` (CRUD de dados e validações).
    *   `CotacaoService` (Cálculo automático e preciso de taxas inversas usando `rust_decimal`).
    *   `ParametrosService` (Atualização de limites e regras).
    *   `AuditoriaService` (Escrita e gravação de logs históricos).
*   [x] Criar as **Rotas HTTP Axum** em `services/aureon-api-local/src/routes/empresa.rs`:
    *   `GET /empresa/configuracao`
    *   `POST/PUT /empresa/configuracao`
    *   `GET /empresa/moedas` | `PUT /empresa/moedas`
    *   `GET /empresa/cotacoes` | `POST /empresa/cotacoes`
    *   `PUT /empresa/cotacoes/{id}/cancelar`
    *   `GET /empresa/parametros-operacionais` | `PUT /empresa/parametros-operacionais`
    *   `GET /empresa/auditoria`
*   [x] Registrar as rotas no Axum Router da API local e rodar testes de chamadas via HTTP.

---

## 🟩 BLOCO 3: Estruturação Física da Retaguarda (Blazor WASM) — [CONCLUÍDO]
*   [x] Criar a estrutura física inicial do projeto de Retaguarda em `apps/aureon-retaguarda/ui-blazor`.
*   [x] Configurar o arquivo de projeto `.csproj`, imports padrões e o `Program.cs`.
*   [x] Estruturar o layout básico administrativo (`MainLayout.razor`) e o menu lateral (`NavMenu.razor`).
*   [x] Configurar o **HttpClient** para apontar para a URL da API Local (`aureon-api-local`).
*   [x] Registrar a rota principal `/configuracoes/empresa`.
*   [x] Compilar e validar o build inicial da retaguarda Blazor WASM.

---

## 🟩 BLOCO 4: Construção da Tela de Configuração da Empresa — [CONCLUÍDO]
*   [x] Criar a página principal `ConfiguracaoEmpresa.razor` em `apps/aureon-retaguarda/ui-blazor/Pages/`.
*   [x] Implementar as 13 abas dinâmicas obrigatórias:
    *   `Dados Gerais`
    *   `País / Fiscal` (Brasil ou Paraguai)
    *   `Idioma` (Português ou Espanhol)
    *   `Identificação` (CNPJ, CPF, RUC, C.I)
    *   `Contato`
    *   `Endereço`
    *   `Logo` (Upload/referência de imagem)
    *   `Multimoeda` (Seleção sem repetição e definição de moeda principal)
    *   `Cotações` (Inserção manual de taxas e exibição em tempo real das inversas calculadas)
    *   `Fiscal Brasil` (Exibição apenas se País for Brasil)
    *   `Fiscal Paraguai` (Exibição apenas se País for Paraguai)
    *   `Parâmetros Operacionais`
    *   `Auditoria` (Lista e histórico de alterações em tempo real)
*   [x] Garantir a reatividade da interface (abas fiscais sumindo/aparecendo dinamicamente).
*   [x] Validar que alterações na moeda principal ou país fiscal solicitem confirmação forte do usuário.

---

## 🟩 BLOCO 5: Validações, Logs de Segurança e Auditoria — [CONCLUÍDO]
*   [x] Validar formatos básicos de documentos (CNPJ, CPF, RUC, C.I) e campos de preenchimento obrigatório na UI e no backend.
*   [x] Garantir que logs do console do Blazor ou terminal Rust **nunca exponham credenciais sensíveis** do PostgreSQL ou segredos.
*   [x] Implementar a exibição da aba de `Auditoria` consumindo o endpoint `GET /empresa/auditoria` mostrando o log das ações críticas com seus estados `anterior` e `novo` formatados.
*   [x] Validar a escrita de eventos na tabela `eventos_publicacao_configuracao` ao atualizar taxas de cotação ou moedas.

---

## 🟩 BLOCO 6: Homologação, Documentação e Build Final — [CONCLUÍDO]
*   [x] Executar um fluxo de teste de ponta a ponta na Retaguarda (gravar empresa, trocar país, cadastrar cotação direta, validar cálculo inverso automático e checar auditoria).
*   [x] Criar os arquivos de documentação técnica exigidos:
    *   `docs/phase-2.md`
    *   `docs/decisions.md`
    *   `docs/company-configuration.md`
    *   `docs/multicurrency.md`
    *   `docs/fiscal-base.md`
*   [x] Validar que toda a aplicação compila sem erros in release.
*   [x] Realizar o commit no GitHub seguindo o padrão de datas/horários exigido.
