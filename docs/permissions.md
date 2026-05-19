# Sistema de Permissões — Aureon

## Visão Geral

O Aureon usa um modelo **RBAC (Role-Based Access Control)** com dois níveis:

1. **Administrador Global** (`is_admin = true`): acesso irrestrito a tudo, sem necessidade de permissões por menu.
2. **Perfis com Permissões por Menu**: cada perfil define o que pode fazer em cada módulo.

---

## Estrutura de Permissões

Cada registro em `permissoes` associa um `perfil_id` a um `menu` com 5 flags booleanas:

| Campo | Descrição |
|---|---|
| `ler` | Visualizar listagens e registros |
| `criar` | Criar novos registros |
| `editar` | Alterar registros existentes |
| `excluir` | Remover registros |
| `exportar` | Exportar dados (relatórios, PDF, Excel) |

---

## Menus Disponíveis para Controle de Acesso

| Menu (chave) | Descrição |
|---|---|
| `SEGURANCA_USUARIOS` | Gestão de usuários |
| `SEGURANCA_PERFIS` | Gestão de perfis e permissões |
| `SEGURANCA_SUPERVISORES` | Gestão de supervisores |
| `SEGURANCA_AUTORIZACOES` | Visualização de autorizações do PDV |
| `SEGURANCA_LOGS` | Visualização de logs de segurança |
| `EMPRESA_CONFIGURACAO` | Configuração da empresa |
| `EMPRESA_MOEDAS` | Configuração de moedas e câmbio |
| (outros módulos serão adicionados nas fases seguintes) |

---

## Como Funciona na API

A função `tem_permissao` em `seguranca.rs` é chamada em cada endpoint protegido. Ela:

1. Verifica se o usuário tem `is_admin = true` → libera tudo.
2. Busca o `perfil_id` do usuário.
3. Consulta a tabela `permissoes` para o menu e ação solicitados.
4. Retorna `true` se a flag estiver ativa, `false` caso contrário.

---

## Perfil ADMINISTRADOR

- O perfil de nome `ADMINISTRADOR` é reservado pelo sistema.
- Não pode ser renomeado.
- Não pode ter permissões críticas removidas via API.
- Usuários com `is_admin = true` associados a ele têm acesso irrestrito.

---

## Regra do Último Administrador

O sistema bloqueia qualquer operação que resultaria em **zero administradores ativos** no sistema:
- Inativar o último admin ativo → bloqueado.
- Bloquear o último admin ativo → bloqueado.
- Remover o perfil ADMINISTRADOR do último admin → bloqueado.

Essa verificação é feita no backend e não pode ser contornada pelo frontend.
