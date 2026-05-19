# Auditoria e Logs de Segurança — Aureon

## Visão Geral

O sistema registra automaticamente eventos críticos de segurança na tabela `logs_seguranca`. Esses registros são imutáveis via API — apenas leitura é permitida na Retaguarda.

---

## Onde Estão os Logs

- **Tabela:** `logs_seguranca` (PostgreSQL)
- **Endpoint de leitura:** `GET /seguranca/logs` (requer permissão `SEGURANCA_LOGS / LER`)
- **Tela Blazor:** `/seguranca/auditoria` → Aba "Logs de Segurança"

---

## Eventos Auditados

| Tipo de Evento | Quando é registrado |
|---|---|
| `LOGIN` | Usuário autenticado com sucesso |
| `LOGOUT` | Sessão encerrada pelo usuário |
| `FALHA_LOGIN` | Tentativa de login com credenciais inválidas |
| `SETUP_ADMIN` | Primeiro administrador criado via `/auth/setup` |
| `CRIAR_USUARIO` | Novo usuário criado por um admin |
| `ALTERAR_PERMISSAO` | Permissões de um perfil alteradas |
| `REDEFINIR_SENHA` | Senha de um usuário redefinida por admin |
| `SESSAO_EXPIRADA` | Sessão expirada por inatividade (1h) |

---

## Campos do Log

| Campo | Tipo | Descrição |
|---|---|---|
| `id` | bigint | Identificador sequencial |
| `usuario_id` | uuid (nullable) | Quem realizou a ação (null para sistema) |
| `tipo_evento` | text | Tipo do evento (ver tabela acima) |
| `mensagem` | text | Descrição legível do evento |
| `severidade` | text | `INFO`, `AVISO` ou `CRITICO` |
| `criado_em` | timestamptz | Data e hora do evento |

---

## Regras de Não-Exposição

Os logs **nunca contêm**:
- Senhas ou hashes de senha.
- Tokens de sessão (opacos ou seus hashes).
- PINs de supervisor.
- Dados financeiros sensíveis.

As mensagens de log descrevem a **ação**, não os dados.  
Exemplo correto: `"Senha do usuário João redefinida por admin."``  
Exemplo proibido: `"Hash nova senha: $argon2id$..."`

---

## Autorizações do PDV

As autorizações de supervisão (operações especiais liberadas no caixa por um supervisor com PIN) são registradas na tabela `autorizacoes` e acessíveis em:

- **Endpoint:** `GET /seguranca/autorizacoes`
- **Tela Blazor:** `/seguranca/auditoria` → Aba "Autorizações do PDV"

Ambas as abas estão centralizadas em `/seguranca/auditoria` para facilitar a supervisão gerencial em um único painel.
