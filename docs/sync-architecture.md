# Arquitetura de Sincronização — Aureon Sync

A arquitetura de sincronização do Aureon adota um modelo **Hub-and-Spoke**, onde o banco de dados central (PostgreSQL na Nuvem/Retaguarda) atua como a fonte única da verdade (Hub), e cada terminal PDV atua como um nó local independente (Spoke) rodando SQLite.

## Visão Geral do Fluxo

```
[ Retaguarda (Postgres) ]
           │
           ▼
 [ aureon-api-local ] <─── HTTP / REST ───> [ aureon-pdv (Rust/Tauri) ]
           │                                            │
           ▼ (Pacote JSON Real)                         ▼ (Aplicação SQL)
 [ pacotes_sincronizacao ]                     [ sqlite_local.db ]
```

## Diretrizes de Confiabilidade

1. **Local-First para Vendas**: O PDV opera 100% offline se necessário. Todas as vendas são salvas localmente no SQLite primeiro.
2. **Puxada de Dados Mestres (Pull)**: O PDV baixa dados de catálogo, preços, fiscal e configurações da Retaguarda através da API.
3. **Idempotência Garantida**: Todos os pacotes possuem chaves únicas de idempotência que evitam reprocessamento acidental de transações.
4. **Isolamento de Segurança**: Dados sensíveis (como hashes de senhas administrativas) são limpos na origem e não transitam pela API de sincronização dos PDVs.
