# Fase 17 — Sincronização Fiscal, Dicionários Mestres e Retaguarda Fiscal

## Resumo

A Fase 17 estabeleceu a **arquitetura fiscal estrutural completa** do Sistema Aureon, criando:

1. A **Retaguarda/PostgreSQL** como fonte mestre absoluta de todos os dicionários, regras e configurações fiscais.
2. Um sistema de **publicação versionada** de pacotes fiscais.
3. O **PDV/SQLite** como consumidor idempotente desses pacotes, aplicando-os nas tabelas `fiscal_*_cache`.
4. Interfaces Blazor completas na Retaguarda e no PDV para administração e diagnóstico fiscal.

> ⚠️ **A Fase 17 NÃO implementou emissão fiscal real.** Nenhum documento fiscal (NF-e, NFC-e, SAT, SIFEN) foi gerado, assinado, transmitido ou autorizado.

---

## Blocos Implementados

| Bloco | Descrição | Commit |
|-------|-----------|--------|
| Bloco 1 | Migrations PostgreSQL de Retaguarda Fiscal Mestre | `217f614` |
| Bloco 2 | API Axum — 18 endpoints de Dicionários, Regras, Versões | `c758709` |
| Bloco 3 | Publicação Fiscal Versionada + Integração Sync | `383bb79` |
| Bloco 4 | Commands Tauri / Aplicação SQLite no PDV | `f50e569` |
| Bloco 5 | UI Blazor Retaguarda — Dicionários, Regras, Publicação | `417a879` |
| Bloco 6 | UI Blazor PDV — FiscalSyncPdv (Diagnóstico/Status) | `b237f2e` |
| Bloco 7 | Documentação Final e Checklist de Aceite | *(este commit)* |

---

## Migrations Criadas

### PostgreSQL (Retaguarda)
- `database/migrations/postgresql/013_fase17_fiscal_mestre.sql`
- `database/seeds/dev/postgresql/seed_fase17_fiscal_dev.sql` *(seed dev mínimo, não carga legal oficial)*

### SQLite (PDV)
- `database/migrations/sqlite/013_fase17_sync_fiscal.sql`

---

## Tabelas PostgreSQL Criadas

| Tabela | Finalidade |
|--------|-----------|
| `fiscal_dicionario_ncm` | Nomenclatura Comum do Mercosul |
| `fiscal_dicionario_cfop` | Código Fiscal de Operações e Prestações |
| `fiscal_dicionario_cst_csosn` | Código de Situação Tributária / CSOSN |
| `fiscal_dicionario_iva` | Impuesto al Valor Agregado (PY) |
| `fiscal_empresas_config` | Configuração fiscal por empresa/filial |
| `fiscal_numeracao_mestre` | Numeração e série por tipo de documento |
| `fiscal_regras_tributarias_mestre` | Matriz de regras e alíquotas |
| `fiscal_versoes_publicacao` | Controle de versões dos pacotes fiscais |
| `fiscal_versoes_publicacao_itens` | Itens individuais por versão |
| `fiscal_auditoria_mestre` | Rastreabilidade de todas as alterações |

---

## Tabelas SQLite Criadas (PDV Cache — Fase 17)

| Tabela | Finalidade |
|--------|-----------|
| `fiscal_versoes_aplicadas_cache` | Histórico de pacotes aplicados no PDV |
| `fiscal_sync_logs` | Logs técnicos de aplicação de pacotes |

*(As tabelas `fiscal_*_cache` da Fase 16 foram reaproveitadas como destino dos pacotes)*

---

## Endpoints API Criados (aureon-api-local)

### Configuração Fiscal
- `GET /fiscal/configuracoes`
- `POST /fiscal/configuracoes`
- `PUT /fiscal/configuracoes/{id}`

### Dicionários
- `GET/POST /fiscal/dicionarios/ncm`
- `PUT /fiscal/dicionarios/ncm/{id}` + `/inativar`
- `GET/POST /fiscal/dicionarios/cfop`
- `PUT /fiscal/dicionarios/cfop/{id}` + `/inativar`
- `GET/POST /fiscal/dicionarios/cst-csosn`
- `PUT /fiscal/dicionarios/cst-csosn/{id}` + `/inativar`
- `GET/POST /fiscal/dicionarios/iva`
- `PUT /fiscal/dicionarios/iva/{id}` + `/inativar`

### Regras
- `GET/POST /fiscal/regras`
- `PUT /fiscal/regras/{id}` + `/inativar`

### Versões e Publicação
- `GET /fiscal/versoes`
- `POST /fiscal/versoes/rascunho`
- `PUT /fiscal/versoes/{id}/cancelar`
- `GET /fiscal/versoes/{id}/itens`
- `POST /fiscal/versoes/{id}/publicar`
- `POST /fiscal/versoes/{id}/reprocessar`
- `GET /fiscal/versoes/{id}/payload`
- `GET /fiscal/publicacoes`
- `GET /fiscal/publicacoes/{id}`

### Auditoria
- `GET /fiscal/auditoria`

---

## Commands Tauri Criados (aureon-pdv)

| Command | Arquivo |
|---------|---------|
| `aplicar_pacote_fiscal` | `commands_sync_fiscal.rs` |
| `obter_status_versao_fiscal_local` | `commands_sync_fiscal.rs` |
| `listar_logs_sync_fiscal` | `commands_sync_fiscal.rs` |
| `validar_pacote_fiscal_local` | `commands_sync_fiscal.rs` |

---

## Telas Blazor Retaguarda Criadas

| Arquivo | Rota | Função |
|---------|------|--------|
| `FiscalAdmin.razor` | `/fiscal` | Hub central com cards |
| `DicionariosFiscaisAdmin.razor` | `/fiscal/dicionarios` | NCM/CFOP/CST/IVA em abas |
| `RegrasTributariasAdmin.razor` | `/fiscal/regras` | Matriz tributária com conversão escala 6 |
| `ConfiguracoesFiscaisAdmin.razor` | `/fiscal/configuracoes` | Ambiente, país, forma de emissão |
| `PublicacaoFiscalAdmin.razor` | `/fiscal/publicacao` | Versões, publicação e payload |

---

## Tela Blazor PDV Criada

| Arquivo | Rota | Função |
|---------|------|--------|
| `FiscalSyncPdv.razor` | `/fiscal/sync` | Diagnóstico de sync fiscal local |

---

## Estratégia de Versionamento Fiscal

- Cada versão começa como `RASCUNHO`.
- A publicação (`POST /fiscal/versoes/{id}/publicar`) consolida os dicionários/regras ativos e muda o status para `PUBLICADA`.
- O reprocessamento cria um novo hash e registro de auditoria, marcando como `REPROCESSADA`.
- Versões canceladas ficam como `CANCELADA` — sem exclusão física.

---

## Estratégia de Publicação

1. Consultar 7 tabelas fiscais mestre no PostgreSQL.
2. Montar payload JSON estruturado com blocos separados.
3. Calcular hash determinístico do payload.
4. Inserir em `pacotes_sincronizacao` com `tipo_pacote = 'SYNC_FISCAL'`.
5. Inserir itens em `pacotes_sincronizacao_itens`.
6. Registrar em `fiscal_auditoria_mestre`.
7. Garantir idempotência via `sync_idempotencia`.

---

## Estratégia de Aplicação Local (PDV)

1. Receber pacote com `payload_json`, `versao`, `payload_hash`.
2. Verificar idempotência: se `payload_hash` já foi `APLICADO`, retornar OK sem reaplicar.
3. Abrir transação SQLite.
4. Aplicar cada bloco do payload nas tabelas `fiscal_*_cache` via `ON CONFLICT DO UPDATE` (UPSERT) ou `UPDATE SET ativo = 0` (DELETE_LOGICO).
5. Atualizar `fiscal_versoes_aplicadas_cache` com status `APLICADO`.
6. Registrar log `FISCAL_PACOTE_APLICADO`.
7. Commit — ou rollback automático em caso de falha.

---

## Estratégia de Idempotência

- **Na Retaguarda:** Chave única via `sync_idempotencia`. Reenvio não duplica pacotes.
- **No PDV:** `payload_hash` verificado antes da transação. Payload já aplicado → retorno imediato sem reprocessamento.

---

## Limitações Conhecidas

1. **Sem chunking/paginação:** O payload fiscal é enviado como JSON único. Bases com >10.000 registros NCM/CFOP podem gerar payloads grandes. Paginação por fase futura.
2. **Hash não criptográfico:** Algoritmo determinístico simples (não SHA-256). Adequado para versionamento de app, não para assinatura.
3. **Seed dev ≠ base legal:** Os registros do `seed_fase17_fiscal_dev.sql` são apenas para desenvolvimento. Não substituem o cadastro oficial de NCM/CFOP da Receita Federal.
4. **Modal de criação de dicionários** na UI Retaguarda está como placeholder (sem form completo).
5. **Controle de perfil** na tela `FiscalSyncPdv` não implementado — acesso aberto a todos os usuários do PDV.

---

## O que Ficou Fora do Escopo (Proposital)

- ❌ Emissão NF-e / NFC-e / NFS-e / SAT / SIFEN
- ❌ Transmissão SEFAZ / DNIT / SIFEN
- ❌ XML assinado / DANFE / KuDE / QR Fiscal
- ❌ Certificado digital operacional
- ❌ Contingência fiscal real (DPEC, NFC-e offline autorizada)
- ❌ Importador massivo de NCM/CFOP (via CSV/API oficial)
- ❌ Validação de CNPJ fiscal / IE via Sintegra/SEFAZ
- ❌ Cálculo de ST (Substituição Tributária) complexo
- ❌ Chunking/paginação de payload fiscal
