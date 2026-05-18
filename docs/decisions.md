# Aureon Sistema Inteligente — Decisões Técnicas

> Registro de decisões arquiteturais relevantes tomadas durante o desenvolvimento.

---

## DT-001 — Blazor WASM como UI (não Blazor Server)

**Data:** 2026-05-18  
**Decisão:** Usar Blazor WebAssembly, não Blazor Server.  
**Motivo:** O sistema é offline-first. Blazor Server requer conexão permanente com servidor; Blazor WASM roda completamente no cliente, ideal para app Tauri desktop sem internet.  
**Consequência:** Tamanho inicial maior (download do runtime .NET), mas performance superior após carregamento.

---

## DT-002 — rusqlite com feature "bundled"

**Data:** 2026-05-18  
**Decisão:** Usar `rusqlite` com feature `bundled` (SQLite compilado dentro do binário).  
**Motivo:** Elimina dependência de SQLite instalado no sistema do usuário. O instalador do Aureon não precisa distribuir SQLite separadamente.  
**Consequência:** Binário maior, mas instalação mais simples e confiável.

---

## DT-003 — API Local separada do Tauri

**Data:** 2026-05-18  
**Decisão:** A API Local Rust (Axum) é um processo separado do app Tauri.  
**Motivo:** Separação de responsabilidades. O Tauri cuida da UI; a API cuida do acesso ao PostgreSQL. Permite que a API rode como serviço Windows independente no futuro.  
**Consequência:** Dois processos para gerenciar. Comunicação via HTTP local (localhost:7070).

---

## DT-004 — Criptografia AES-256-GCM com chave gerada na instalação

**Data:** 2026-05-18  
**Decisão:** Chave de criptografia gerada aleatoriamente na primeira inicialização; NÃO derivada de identificador fixo do terminal.  
**Motivo:** Chave derivada de ID fixo é previsível e insegura. Chave aleatória garante unicidade por instalação.  
**Evolução futura:** Windows DPAPI para proteger o arquivo `.keystore`.

---

## DT-005 — Mutex para acesso ao SQLite

**Data:** 2026-05-18  
**Decisão:** `Arc<Mutex<Connection>>` para compartilhar conexão SQLite entre threads.  
**Motivo:** `rusqlite::Connection` não é `Send + Sync`. O Mutex garante acesso serial seguro.  
**Alternativa considerada:** `r2d2-sqlite` (pool de conexões) — descartado por complexidade desnecessária na Fase 0.

---

## DT-006 — Nomes de tabelas em português, snake_case, sem acentos

**Data:** 2026-05-18  
**Decisão:** Todas as tabelas e colunas usam português sem acentos, em snake_case.  
**Motivo:** Padronização definida no prompt do projeto. Garante consistência e evita problemas de encoding em migrações.  
**Exemplos:** `configuracoes_locais`, `sync_outbox`, `logs_locais` (nunca `config`, `outbox`, `logs`).

---

## DT-007 — Modo degradado para PostgreSQL

**Data:** 2026-05-18  
**Decisão:** A API Local funciona sem PostgreSQL, retornando `"status": "indisponivel"` no diagnóstico.  
**Motivo:** O PDV deve ser operacional mesmo sem PostgreSQL disponível (offline-first). A API não pode falhar criticamente por isso.  
**Consequência:** Endpoints que dependem de PG retornam erro claro e descritivo, não exception não tratada.

---

## DT-008 — Porta 7070 para API Local

**Data:** 2026-05-18  
**Decisão:** Porta padrão `7070` para a API Local.  
**Motivo:** Evitar conflito com portas comuns (3000, 8080, 5000). Configurável via `AUREON_API_PORTA`.

---

## DT-009 — Workspace Cargo único na raiz

**Data:** 2026-05-18  
**Decisão:** Um único `Cargo.toml` de workspace na raiz engloba todas as crates e binários.  
**Motivo:** Compilação unificada, versões compartilhadas de dependências, IDE com visão completa do projeto.  
**Consequência:** `cargo build` na raiz compila tudo. Dependências duplicadas são eliminadas automaticamente.
