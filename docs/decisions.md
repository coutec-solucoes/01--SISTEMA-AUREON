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

---

## DT-010 — Keystore Local com AES-256-GCM

**Data:** 2026-05-18  
**Decisão:** Utilização de um arquivo restrito (`C:/Aureon/config/.keystore`) que contém uma chave AES-256 randômica gerada na instalação, codificada em Base64. A criptografia dos dados utiliza AES-GCM (autenticado) gerando um `nonce`/`iv` único e aleatório para cada operação de gravação.  
**Motivo:** Base64 não é segurança, é apenas codificação. A segurança real vem da entropia da chave gerada via `rand::OsRng`, combinada com as restrições de permissão do sistema operacional (ACL) sobre o arquivo. Não sobrescrevemos o keystore automaticamente se ele já existir, para prevenir perda acidental de acesso às configurações.  
**Evolução futura:** Uso do Windows DPAPI para envelopar o conteúdo do `.keystore` tornando a chave atrelada ao usuário atual.

---

## DT-011 — AUREON Config como App Isolado

**Data:** 2026-05-18  
**Decisão:** O setup técnico (banco de dados, API, migrações) é executado por um binário Tauri separado (`apps/aureon-config`), e não pelo PDV (`apps/aureon-pdv`).  
**Motivo:** Arquitetura limpa e separação de privilégios. O PDV não deve conter dependências lógicas de instanciar bancos ou gerenciar chaves-mestre. O PDV apenas exibe "Terminal não configurado" se o `.keystore` e o `server.config.enc` não existirem.  
**Consequência:** Mantém o PDV seguro e enxuto. O AUREON Config é responsável por gerar os arquivos `.enc` e preencher os *seeds* base.

---

## DT-012 — Retaguarda/Gestor Isolada (Blazor WASM)

**Data:** 2026-05-19  
**Decisão:** A interface de Retaguarda/Gestor é estruturada como um projeto Blazor WebAssembly independente (`apps/aureon-retaguarda/ui-blazor`), separado do PDV.  
**Motivo:** Evita o acoplamento desnecessário de rotas administrativas e operacionais (PDV/Venda). Mantém o código do PDV focado em performance offline extrema e velocidade de venda, enquanto a Retaguarda gerencia regras complexas, cadastros e fiscal base.  
**Consequência:** Organização física limpa da workspace e carregamento sob demanda apenas dos componentes administrativos.

---

## DT-013 — Cotações do Dia com Taxas Inversas Reativas via rust_decimal

**Data:** 2026-05-19  
**Decisão:** Utilização do tipo de alta precisão `rust_decimal` no Rust Axum e conversão reativa em tempo real na interface da Retaguarda para cálculo da taxa inversa (`1 / taxa_direta`).  
**Motivo:** Evita erros de arredondamento de float em operações financeiras e de conversão de troco de moedas, eliminando pequenas discrepâncias centesimais nos caixas.  
**Consequência:** Precisão absoluta de até 28 casas decimais nas conversões financeiras entre BRL, USD e PYG.

---

## DT-014 — Eventos de Publicação e Sync Fiscais (Outbox no PostgreSQL)

**Data:** 2026-05-19  
**Decisão:** Persistência de alterações de moedas, cotações e parâmetros em uma tabela de outbox (`eventos_publicacao_configuracao`) e gravação detalhada na tabela `auditoria_eventos` com estado anterior/novo.  
**Motivo:** Facilita a auditoria de alterações operacionais críticas e fornece um canal idempotente para o sincronizador offline ler e propagar alterações de cotações para os caixas locais.  
**Consequência:** Logs de segurança completos de auditoria e conformidade de sincronização futura garantida.

---

## DT-009 — Duas tabelas de publicação (eventos_publicacao_configuracao vs eventos_publicacao)

**Data:** 2026-05-19  
**Decisão:** Manter `eventos_publicacao_configuracao` (Fase 2) para eventos de empresa/configuração e criar `eventos_publicacao` (Fase 4) como fila genérica para eventos de negócio (pessoas, produtos, etc.).  
**Motivo:** A tabela original era acoplada a `empresa_id` e específica para configurações. A Fase 4 precisava de uma fila extensível para qualquer entidade de negócio (produto, pessoa, grupo) sem recriar/quebrar a estrutura existente.  
**Consequência:** Existem duas tabelas com propósitos distintos e complementares. Na **Fase 6 (Sincronização)**, avaliar unificação em uma única fila genérica. Não criar novas tabelas paralelas de publicação sem decisão explícita documentada aqui.

| Tabela | Escopo | Criada em |
|---|---|---|
| `eventos_publicacao_configuracao` | Configurações da empresa (moedas, parâmetros) | Fase 2 |
| `eventos_publicacao` | Entidades de negócio (pessoas, produtos, grupos) | Fase 4 |

---

## DT-010 — auditoria_cadastros separada de logs_seguranca

**Data:** 2026-05-19  
**Decisão:** Criar tabela `auditoria_cadastros` para registrar alterações de negócio (pessoas, produtos, preços), separada de `logs_seguranca` que é exclusiva para eventos de autenticação e segurança.  
**Motivo:** Misturar eventos de negócio com eventos de segurança dificultaria consultas, relatórios e triagem de incidentes.  
**Consequência:** Dois pontos de auditoria com propósitos claros. `logs_seguranca` = segurança/acesso. `auditoria_cadastros` = negócio/dados.

