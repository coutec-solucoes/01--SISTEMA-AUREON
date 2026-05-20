# Sistema de Autorização do Supervisor Local (PDV)

Este documento especifica o fluxo de segurança local para liberação de operações restritas por gerentes/supervisores.

---

## 🔑 Cache de Supervisores

Como o PDV funciona offline-first, a validação de credenciais de privilégios elevados ocorre localmente na tabela `supervisores_cache`:

| Campo | Tipo SQLite | Descrição |
| :--- | :--- | :--- |
| `id` | TEXT (PK) | Identificador único do supervisor |
| `nome` | TEXT | Nome legível do supervisor |
| `pin_hash` | TEXT | Hash Bcrypt do PIN de autorização |
| `ativo` | BOOLEAN | Indica se o supervisor pode atuar |
| `atualizado_em`| TEXT | Registro de timestamp ISO 8601 |

---

## ⚡ Fluxo de Autorização (Tauri/Rust)

1. A UI Blazor abre o componente modal `SupervisorModal`.
2. O usuário digita o PIN (caracteres mascarados).
3. A chamada via Tauri (`solicitar_autorizacao_supervisor`) envia o PIN e os dados do evento.
4. O Rust recupera os registros ativos de `supervisores_cache`.
5. Valida-se o hash usando a biblioteca `bcrypt::verify`.
6. O resultado (Aprovado ou Negado) é persistido transacionalmente em `supervisor_autorizacoes_local`.
7. O evento de sync correspondente é lançado no `sync_outbox`:
   - `SUPERVISOR_AUTORIZACAO_APROVADA` (se a validação passar)
   - `SUPERVISOR_AUTORIZACAO_NEGADA` (se o PIN for inválido ou supervisor inativo)

---

## 🚫 Regras Críticas de Segurança

- **Proibido Salvar em Texto Puro**: O PIN inserido pelo usuário jamais é armazenado nas tabelas locais ou arquivos de log em formato cru.
- **Log Protegido**: Payloads enviados para o outbox ou log de depuração do Tauri não contêm o PIN, apenas a identificação do supervisor e o status da autorização.
