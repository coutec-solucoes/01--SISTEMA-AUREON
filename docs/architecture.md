# Aureon Sistema Inteligente — Arquitetura

> Versão: 0.0.1 | Fase: 0 — Fundação Técnica

---

## Visão Geral

```
UI Blazor WASM
     ↓  (JS Interop)
Tauri IPC
     ↓  (invoke)
Tauri Commands — Rust
     ↓
Domain Services locais
     ↓
Repositories locais
     ↓
SQLite local (WAL mode)
     ↓
[futuro: Sync Outbox]
     ↓
API Local Rust (Axum)
     ↓
PostgreSQL local da empresa
     ↓
[futuro: VPS opcional]
```

---

## Papel de cada camada

### Blazor WebAssembly (UI)
- Responsável **exclusivamente** pela interface do usuário.
- Roda no próprio processo do Tauri (sem servidor web).
- **Nunca acessa banco de dados diretamente.**
- Comunica-se com o Rust apenas via `TauriService` → `tauri-interop.js` → `window.__TAURI__.core.invoke()`.

### Tauri 2.0 (Container Desktop)
- Hospeda o Blazor WASM como frontend.
- Expõe os Tauri Commands (funções Rust) para a UI via IPC.
- Gerencia o ciclo de vida da janela e permissões (capabilities).
- Prepara estrutura para futura versão mobile (Android/iOS).

### Rust Local — Tauri Commands
- Recebe chamadas da UI com DTOs tipados.
- Valida entrada.
- Chama os Domain Services.
- Retorna `RespostaBase<T>` padronizado.
- Registra logs sem expor dados sensíveis.

### Domain Services / Repositories
- **Domain**: define regras de negócio e interfaces (traits) — sem dependências de infraestrutura.
- **Infra**: implementa as interfaces usando SQLite (rusqlite, bundled).
- Separação permite trocar banco sem afetar regras de negócio.

### SQLite Local
- Banco principal do terminal PDV.
- Arquivo: `C:/Aureon/data/aureon-local.db` (produção).
- Modo WAL habilitado para performance e segurança.
- Migrations versionadas controladas pela tabela `schema_migrations_local`.
- **Todo dado nasce aqui primeiro** (offline-first).

### API Local Rust (Axum)
- Processo separado do Tauri, rodando localmente.
- Porta padrão: `7070`.
- **Único componente que acessa o PostgreSQL.**
- Funciona em modo degradado se PostgreSQL estiver indisponível.
- Endpoints desta fase: `GET /health`, `GET /diagnostico/basico`.

### PostgreSQL Local da Empresa
- Banco central da empresa (servidor local ou rede local).
- **Nunca acessado diretamente pelo PDV** — apenas pela API Local.
- Recebe dados sincronizados do SQLite via Sync Outbox (fase futura).
- Migrations versionadas na tabela `schema_migrations`.

---

## Fluxo Offline-First

```
1. Usuário realiza ação na UI (Blazor)
2. UI chama TauriService.InvocarAsync("command_name", args)
3. Tauri repassa ao Command Rust correspondente
4. Command chama Domain Service
5. Service valida e chama Repository
6. Repository grava NO SQLite local (SEMPRE primeiro)
7. Evento é enfileirado no sync_outbox (fase futura)
8. API Local sincroniza com PostgreSQL em background (fase futura)
9. Resposta retorna pela cadeia até a UI
```

### Por que o PDV não acessa PostgreSQL diretamente?

| Razão | Impacto |
|---|---|
| **Resiliência** | PDV funciona sem internet ou rede |
| **Performance** | SQLite local é mais rápido que rede |
| **Segurança** | Credenciais do PG nunca chegam ao terminal |
| **Integridade** | Dados passam pela API (validação centralizada) |
| **Escalabilidade** | Múltiplos PDVs sem sobrecarregar PG |

---

## Configuração Segura

- Chave AES-256 gerada aleatoriamente na **primeira inicialização**.
- Armazenada em: `C:/Aureon/config/.keystore` com permissões restritas ao SO.
- **Nunca** salva senhas ou tokens em texto puro.
- Valores de configuração armazenados como `valor_criptografado` no SQLite.
- Evolução futura: Windows DPAPI / TPM para proteger o `.keystore`.

---

## Requisitos de Ambiente

| Componente | Versão mínima | Obrigatório |
|---|---|---|
| Rust | 1.75+ | Sim |
| .NET SDK | 8.0+ | Sim |
| Node.js | 18+ (apenas build Tauri) | Sim |
| PostgreSQL | 14+ | Não (modo degradado) |
| Windows | 10+ | Sim (fase atual) |
