# Registro de Decisões de Projeto (ADR) — Fase 5

Este documento compila as decisões de arquitetura e padrões técnicos adotados durante o desenvolvimento da Fase 5.

---

## 🛡️ ADR 01: Segurança via Token Opaco UUID
- **Contexto**: A API necessita validar sessões de usuário e chaves de segurança da empresa para cada transação e alteração de parâmetros.
- **Decisão**: Rejeitado o uso de JWT (Json Web Tokens) para manter o alinhamento estrito com o padrão estabelecido na Fase 3. As requisições locais enviam o cabeçalho `Authorization: Bearer <token_uuid>`, validado diretamente na tabela `sessoes_usuarios` com hash SHA-256 no banco de dados local.
- **Consequência**: Garantia de revogabilidade imediata de chaves e sessões e menor sobrecarga computacional em hardware modesto local, mantendo a arquitetura offline simples e robusta.

---

## 🔌 ADR 02: Padronização Rígida de Rotas Operacionais
- **Contexto**: Diversos endpoints operacionais e cadastros de hardware foram propostos sob diferentes nomenclaturas em fases anteriores.
- **Decisão**: Padronizar rigidamente o prefixo `/configuracoes/operacionais` para todos os 17 endpoints operacionais. Foi banido completamente o uso do termo `/configuracoes/operacoes/`.
- **Consequência**: Uniformidade no roteamento Axum, facilidade de auditoria centralizada nas rotas locais de rede e consistência absoluta no consumo de APIs na retaguarda Blazor.

---

## ⚡ ADR 03: Separação de Parâmetros e Funcionamento Operacional Real
- **Contexto**: A Fase 5 foca em configurações e preparação física do ecossistema. Funcionalidades como transações financeiras, fechamentos, escuta real de balanças ou chamadores ativos de senhas eletrônicas exigiriam bibliotecas nativas de sistema operacional (Tauri/APS) que não pertencem ao escopo da retaguarda web.
- **Decisão**: Todos os endpoints de testes físicos (`/impressoras/{id}/testar`, `/perifericos/{id}/testar`, `/senhas-chamadas/{id}/testar` e `/balancas/{id}/ler-peso`) funcionam de forma simulada/mockada em ambiente web. O banco de dados armazena os parâmetros reais que serão consumidos futuramente pelo executável do PDV offline nativo na Fase 6.
- **Consequência**: Agilidade na homologação da retaguarda administrativa WebAssembly, isolando os drivers de hardware para o escopo nativo apropriado.

---

# Registro de Decisões de Projeto (ADR) — Fase 6

Decisões de arquitetura adotadas na Fase 6 — Sincronização Base e Publicação para Terminais.

---

## 🔄 ADR 04: Reaproveitamento de sync_idempotencia (PostgreSQL)
- **Contexto**: A migration `009_sync_base.sql` precisaria criar controle de idempotência para operações de publicação e confirmação de pacotes.
- **Decisão**: A tabela `sync_idempotencia` **já existia** na migration `001_schema_inicial.sql` com os campos `idempotency_key (PK)`, `event_type`, `processado_em` e `resultado`. **Não foi recriada nem alterada** — campos existentes são suficientes para o escopo da Fase 6.
- **Consequência**: Zero risco de perda de dados de idempotência registrados em fases anteriores. Reutilização direta pelos novos endpoints de sync.

---

## 🔄 ADR 05: Reaproveitamento de eventos_publicacao (PostgreSQL)
- **Contexto**: A Fase 6 requer eventos de publicação como `TERMINAL_REGISTRADO`, `PUBLICACAO_CRIADA`, etc.
- **Decisão**: A tabela `eventos_publicacao` **já existia** na migration `006_cadastros_pessoas.sql` com estrutura genérica (`tipo_evento`, `entidade`, `entidade_id`, `payload`, `processado`). **Não foi recriada**. Os novos tipos de evento da Fase 6 serão inseridos via INSERT durante a operação normal da API.
- **Consequência**: Histórico completo de eventos preservado. Tabela genérica cobre todos os novos tipos sem alteração estrutural.

---

## 🔧 ADR 06: ALTER TABLE terminais_pdv (PostgreSQL)
- **Contexto**: A tabela `terminais_pdv` existia desde a migration `008_configuracoes_operacionais.sql` mas sem os campos necessários para controle de sincronização da Fase 6.
- **Decisão**: Aplicado `ALTER TABLE` idempotente usando bloco `DO $$ ... IF NOT EXISTS ... $$` para adicionar **5 colunas novas**: `chave_terminal`, `status_sync`, `ultima_versao_recebida`, `ultima_sincronizacao`, `primeiro_sync_concluido`. Nenhuma coluna existente foi alterada ou removida.
- **Consequência**: Registros existentes preservados com valores padrão nas novas colunas (`status_sync = 'PENDENTE'`, `primeiro_sync_concluido = FALSE`). Migration é segura para re-execução.

---

## 📦 ADR 07: Migration SQLite como versão 002 (em vez de 001_schema_local)
- **Contexto**: O prompt sugeria criar `001_schema_local.sql` no SQLite, mas já existia `001_schema_inicial.sql` com `sync_inbox`, `sync_outbox`, `sync_logs`, `configuracoes_locais` e `terminais`.
- **Decisão**: Criada `002_sync_fase6.sql` como **segunda migration** no sistema versionado existente. As tabelas já presentes na migration 001 **não foram duplicadas**. O arquivo `crates/aureon-infra/src/sqlite/migrations.rs` foi atualizado para registrar a versão 2.
- **Consequência**: Sistema de migrations preserva o histórico. O PDV nunca re-executa migrations já aplicadas (verificação por `schema_migrations_local`). Rollback seguro se a migration 002 falhar na inicialização.

---

## 🔄 ADR 08: Reaproveitamento de sync_outbox, sync_inbox e sync_logs (SQLite)
- **Contexto**: A migration SQLite 002 precisaria dessas tabelas de controle de fila e log.
- **Decisão**: `sync_outbox`, `sync_inbox` e `sync_logs` **já existiam** na migration `001_schema_inicial.sql` com estrutura compatível. **Não foram recriadas** na migration 002.
- **Consequência**: Dados de fila e log existentes no SQLite preservados. A migration 002 apenas adiciona tabelas novas sem tocar nas existentes.

---

## 🔒 ADR 09: Armazenamento seguro da chave_terminal no SQLite
- **Contexto**: O terminal PDV precisa armazenar sua `chave_terminal` (token opaco UUID) localmente para autenticar chamadas subsequentes à API.
- **Decisão**: Em produção, o valor sensível é gravado na tabela `configuracoes_locais` (campo `valor_criptografado`). A coluna `chave_terminal` em `terminal_local` serve apenas como referência de status — nunca é exposta em `sync_logs` ou `logs_locais`.
- **Consequência**: Proteção dupla: dado sensível criptografado + log sem exposição de segredos. Segue o padrão oficial da Fase 3 de não logar tokens.
