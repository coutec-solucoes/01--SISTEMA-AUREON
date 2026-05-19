# Arquitetura de Segurança - Aureon Retaguarda e API

Este documento detalha as estratégias de segurança implementadas na Fase 3 do sistema Aureon.

## 1. Gestão de Senhas
- As senhas dos administradores, usuários e supervisores nunca são trafegadas ou gravadas em texto plano.
- O sistema utiliza o algoritmo de derivação de chave **Argon2** com forte proteção contra ataques de força bruta, implementado na API Rust.
- Nenhuma API retorna o hash do banco.

## 2. Sessão e Token
- O sistema utiliza **Tokens Opacos (UUID)** gerados no servidor. Não utilizamos JWT.
- O Token Opaco trafega via Header `Authorization: Bearer <token>`.
- O banco de dados armazena um **hash SHA-256** do token. Caso haja um vazamento do banco, as sessões ativas não poderão ser sequestradas.
- A sessão expira após 1 hora de inatividade. O tempo é renovado automaticamente em cada requisição válida.
- Ao efetuar *Logout*, a sessão é fisicamente removida do banco de dados, revogando o token instantaneamente.

## 3. Armazenamento Front-End (Risco e Mitigação)
- O Blazor WASM, provisoriamente, armazena o token opaco no `localStorage` do navegador do usuário.
- **Risco**: `localStorage` é vulnerável a ataques XSS (Cross-Site Scripting). Se a aplicação front-end for comprometida, o token pode ser acessado pelo script malicioso.
- **Mitigação Atual**:
  - O uso de token opaco reduz a carga de informações do payload (o token não revela informações do perfil, que são buscadas via `/auth/me`).
  - Em versões futuras (empacotamento Tauri para Retaguarda Desktop), essa chave será movida para uma *Secret Storage* local fortemente criptografada gerenciada pelo Tauri, sem visibilidade no contexto do navegador.

## 4. Auditoria
- As operações sensíveis de segurança disparam logs para a tabela `logs_seguranca`, registrando a ação e o autor.

## 5. Proteção RBAC (Role-Based Access Control)
- Administradores globais (`is_admin = true`) têm acesso a tudo sem necessidade de registrar permissões por menu.
- A regra do *Último Administrador* impede que o único admin ativo seja inativado, bloqueado ou perca o acesso.
