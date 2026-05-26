# Controle de Usuários, Permissões e Segurança (Offline-First)

## Identidade Local
O PDV utiliza uma base offline de usuários, perfis e permissões sincronizável futuramente com a retaguarda, mas que possui autonomia total para operar em eventos de desconexão.

### RBAC (Role-Based Access Control)
As permissões são vinculadas a **Perfis** (`perfis_local`), os quais são atrelados aos **Usuários** (`usuarios_local`). As permissões são validadas a nível de `Módulo` + `Ação` e registradas na `permissoes_local`.

### Senhas Fortes
Foi adotado o **Argon2id** para garantir a segurança em repouso dos hashes de senha. O cálculo do hash é gerado diretamente pela infraestrutura em Rust e é exposto ao SQLite e Tauri. Nunca mantemos senhas em texto plano.

### Auditoria e Sessões
- A tabela `sessoes_usuario_local` gerencia a trilha ativa de logins.
- A tabela `auditoria_operacional_local` faz o log de acessos sensíveis (ex: logins falhos, logins corretos e logouts manuais).

## Como Logar (Modo Dev)
O usuário padrão (seeded) do sistema em modo dev é:
- **Login:** `admin`
- **Senha:** `admin123`
*(Será necessário forçar a troca de senha na versão de produção).*
