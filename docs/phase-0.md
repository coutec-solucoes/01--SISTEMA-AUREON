# Aureon Sistema Inteligente — Fase 0

> Status: Em implementação | Data: 2026-05-18

---

## O que foi implementado

### Estrutura de Projeto
- [x] Workspace Cargo com 6 crates + 2 binários
- [x] Estrutura de diretórios escalável

### Crates Rust
- [x] `aureon-core` — erros, DTOs, padrão de resposta
- [x] `aureon-shared` — logs (tracing) + criptografia (AES-256-GCM)
- [x] `aureon-domain` — traits de repositories + services com validação
- [x] `aureon-infra` — implementações SQLite (rusqlite bundled)
- [x] `aureon-sync` — modelos outbox/inbox com todos os campos obrigatórios

### Tauri 2.0 (aureon-pdv)
- [x] App Tauri base com `lib.rs` + `main.rs`
- [x] Estado global com conexão SQLite + migrations automáticas
- [x] 4 commands Tauri: `obter_status_local`, `testar_sqlite`, `gravar_log_local`, `obter_configuracao_local`
- [x] `tauri.conf.json` com CSP adequada para Blazor WASM
- [x] Capabilities Tauri 2.0

### Blazor WASM (ui-blazor)
- [x] Projeto .NET 8 Blazor WebAssembly
- [x] `TauriService.cs` — ponte C# → JS → Tauri
- [x] `tauri-interop.js` — interop com `window.__TAURI__.core.invoke()`
- [x] Página principal com painel de testes dos commands
- [x] Design dark mode profissional com CSS customizado

### API Local Rust (aureon-api-local)
- [x] Axum 0.7 com tokio async
- [x] `GET /health` — resposta mínima
- [x] `GET /diagnostico/basico` — status API + PostgreSQL
- [x] Modo degradado sem PostgreSQL (retorna "indisponivel")
- [x] Configuração via variáveis de ambiente (sem expor DATABASE_URL em logs)

### Banco de Dados
- [x] Migration SQLite 001 — 7 tabelas estruturais
- [x] Migration PostgreSQL 001 — 6 tabelas com multi-empresa
- [x] Runner de migrations SQLite versionado
- [x] Tabelas sync_outbox/inbox com todos os campos obrigatórios

### Documentação
- [x] `docs/architecture.md`
- [x] `docs/phase-0.md` (este arquivo)
- [x] `docs/decisions.md`

---

## O que ficou fora do escopo (intencional)

- Venda PDV real
- Abertura/fechamento de caixa
- Cadastros completos (produtos, pessoas, empresas)
- Estoque, financeiro, delivery, autoatendimento
- Fiscal, Pix, TEF, NFC-e, SIFEN
- Dashboard completo
- Sync real (estrutura criada, lógica na Fase futura)
- Runner de migrations PostgreSQL automático (feito manualmente via SQL)

---

## Como executar

### Pré-requisitos
```powershell
# Verificar Rust
rustc --version   # precisa ser 1.75+

# Verificar .NET
dotnet --version  # precisa ser 8.0+

# Instalar dependências Tauri
cargo install tauri-cli
```

### 1. Compilar as crates base
```powershell
cd "e:\01- SISTEMA AUREON"
cargo build -p aureon-core -p aureon-shared -p aureon-domain -p aureon-infra -p aureon-sync
```

### 2. Rodar a API Local
```powershell
# Sem PostgreSQL (modo degradado)
cargo run -p aureon-api-local

# Com PostgreSQL
$env:DATABASE_URL = "postgres://usuario:senha@localhost:5432/aureon"
cargo run -p aureon-api-local
```

### 3. Testar a API
```powershell
# Healthcheck
Invoke-RestMethod http://localhost:7070/health

# Diagnóstico
Invoke-RestMethod http://localhost:7070/diagnostico/basico
```

### 4. Rodar o app Tauri + Blazor
```powershell
# Publicar Blazor primeiro
dotnet publish "apps/aureon-pdv/ui-blazor/aureon-pdv-ui.csproj" -c Release

# Rodar Tauri em dev
cargo tauri dev --manifest-path apps/aureon-pdv/src-tauri/Cargo.toml
```

---

## Limitações conhecidas

1. **Migrations PostgreSQL**: executadas manualmente via psql (runner automático na próxima fase).
2. **Ícone do app**: necessita criar `apps/aureon-pdv/src-tauri/icons/icon.png`.
3. **Keystore**: `.keystore` ainda não implementado como arquivo real — chave mockada para Fase 0.
4. **Sync**: estrutura criada, lógica de sincronização real nas próximas fases.
5. **.env para API**: criar `services/aureon-api-local/.env` para configurar DATABASE_URL localmente.
