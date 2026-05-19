# Fase 3 — Segurança, Usuários, Permissões e Supervisores

**Status:** APROVADA E ENCERRADA  
**Commit:** feb2c98  
**Branch:** main  
**Data:** 2026-05-19

---

## Objetivo da Fase

Implementar a infraestrutura completa de segurança do sistema Aureon: autenticação segura, controle de acesso baseado em perfis (RBAC), gestão de usuários e supervisores, e rastreabilidade por auditoria.

---

## Blocos Entregues

### Bloco 1 — Estrutura de Banco de Dados
- Migração `005_seguranca.sql`: Tabelas `usuarios`, `perfis`, `permissoes`, `sessoes_usuarios`, `supervisores`, `autorizacoes`, `logs_seguranca`.

### Bloco 2 — API de Autenticação (Rust/Axum)
- Hash de senha com **Argon2id**.
- Token de sessão **opaco UUID** — banco armazena apenas o **hash SHA-256**.
- Sessão expira por **1 hora de inatividade**, renovada automaticamente.
- Endpoints públicos: `POST /auth/setup`, `POST /auth/login`.
- Endpoints protegidos: `GET /auth/me`, `POST /auth/logout`.
- Middleware de autenticação aplicado a todas as rotas protegidas.

### Bloco 3 — CRUD de Segurança (API)
- **Usuários**: criar, editar, bloquear/ativar, redefinir senha (`PUT /seguranca/usuarios/:id/redefinir-senha`).
- **Perfis**: criar, editar, proteger perfil `ADMINISTRADOR`.
- **Permissões**: salvar/obter por perfil por menu/ação (LER, CRIAR, EDITAR, EXCLUIR, EXPORTAR).
- **Supervisores**: criar, editar, PIN hashed com Argon2, controle de autorizações por operação.
- **Autorizações**: histórico de liberações do PDV (somente leitura na Retaguarda).
- **Logs**: histórico de eventos de segurança (somente leitura na Retaguarda).
- **Regra do Último Administrador**: sistema impede inativação/exclusão do único admin ativo.

### Bloco 4 — Frontend Blazor WASM
- `AureonAuthStateProvider`: baseado em token opaco + `/auth/me` (sem JWT).
- `AuthService`: login/logout, localStorage provisório (ver `docs/security.md`).
- `AureonHttpInterceptor`: redirecionamento automático em caso de `401`.
- Proteção global de rotas via `CascadingAuthenticationState` + `AuthorizeRouteView`.
- Sidebar condicional: visível apenas para usuários autenticados.

**Telas criadas:**
| Tela | Rota |
|---|---|
| Login | `/login` |
| Setup Inicial Admin | `/setup` |
| Gestão de Usuários | `/seguranca/usuarios` |
| Perfis & Permissões | `/seguranca/perfis` |
| Supervisores | `/seguranca/supervisores` |
| Autorizações | `/seguranca/autorizacoes` |
| Auditoria & Logs (2 abas) | `/seguranca/auditoria` |

---

## Decisões Técnicas
- Ver `docs/decisions.md` para decisões arquiteturais.
- Ver `docs/security.md` para análise de riscos e mitigações.
- Ver `docs/permissions.md` para estrutura do sistema de permissões.
- Ver `docs/audit.md` para estrutura e eventos do sistema de auditoria.

---

## Próxima Fase
**Fase 4 — Cadastros Base: Pessoas, Produtos, Grupos, Subgrupos e Marcas.**
