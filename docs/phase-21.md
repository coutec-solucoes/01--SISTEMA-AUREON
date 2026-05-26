# Fase 21 — Usuários, Permissões, Perfis e Segurança Operacional

## Objetivo Geral
Criar a camada de controle de acesso do Aureon, permitindo login local/offline, perfis de usuário, permissões por ação crítica, auditoria de operações sensíveis e autorização de supervisor, sem depender de internet permanente. A segurança é implementada progressivamente, focando na infraestrutura (RBAC) e auditoria antes de habilitar bloqueios restritos de tela.

## Blocos

### Bloco 1: Base SQLite de Usuários, Permissões, Perfis e Sessão Local
**Status**: Concluído
- Criação das tabelas de controle de identidade: `usuarios_local`, `perfis_local`, `permissoes_local`, `perfil_permissoes_local`, `usuario_perfis_local`.
- Tabelas para rastreio: `sessoes_usuario_local` e `auditoria_operacional_local`.
- Seed dinâmico no Rust gerando o perfil `ADMIN` com o hash seguro via **Argon2id** no primeiro boot (senha default: `admin123`).
- DTOs e endpoint `login_local`, `logout_local` no Tauri (`commands_seguranca.rs`).
- Criação do frontend Blazor `LoginPdv.razor` e painel `SegurancaPdv.razor`.
- Segurança passiva/informativa (ainda não bloqueamos recursos do sistema para não impactar o desenvolvimento).
