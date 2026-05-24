# Aplicação de Pacotes Fiscais no PDV (SQLite)

## Conceito

O PDV é **consumidor passivo** de regras fiscais. Ele **nunca cria, edita ou transmite** dados fiscais para autoridades.

A aplicação de um pacote fiscal é uma **sincronização de cache local** — equivalente a uma atualização de configuração, não a uma operação transacional de negócio.

---

## Tabelas Envolvidas

### Tabelas de Controle (criadas na Fase 17)

| Tabela | Finalidade |
|--------|-----------|
| `fiscal_versoes_aplicadas_cache` | Registro de pacotes recebidos e aplicados |
| `fiscal_sync_logs` | Logs técnicos de cada evento de sync |

### Tabelas de Cache (criadas na Fase 16 e populadas aqui)

| Tabela | Dados Aplicados |
|--------|----------------|
| `fiscal_empresa_cache` | Config. de empresa/ambiente/forma de emissão |
| `fiscal_numeracao_cache` | Numeração e série por tipo de documento |
| `fiscal_ncm_cache` | Dicionário NCM |
| `fiscal_cfop_cache` | Dicionário CFOP |
| `fiscal_cst_csosn_cache` | Dicionário CST/CSOSN |
| `fiscal_iva_cache` | Dicionário IVA (PY) |
| `fiscal_regras_tributarias_cache` | Matriz tributária parametrizada |

---

## Fluxo de Aplicação

```
Receber pacote (pacote_id, versao, payload_hash, payload_json)
    ↓
Registrar log: FISCAL_PACOTE_RECEBIDO
    ↓
Verificar idempotência:
  payload_hash já existe como APLICADO?
    → Sim: log FISCAL_PACOTE_IGNORADO_IDEMPOTENTE, retornar OK
    → Não: continuar
    ↓
Parse do JSON + validar blocos
    ↓
Registrar log: FISCAL_PACOTE_VALIDADO
    ↓
Abrir transação SQLite
    ↓
Para cada bloco do payload:
  UPSERT → ON CONFLICT DO UPDATE SET
  DELETE_LOGICO → UPDATE SET ativo = 0
    ↓
Atualizar fiscal_versoes_aplicadas_cache: status = APLICADO
    ↓
Registrar log: FISCAL_PACOTE_APLICADO (dentro da transação)
    ↓
COMMIT
    ↓
Retornar sucesso
```

Em caso de falha em qualquer etapa dentro da transação:
```
ROLLBACK automático (Drop da Transaction Rust)
    ↓
Atualizar fiscal_versoes_aplicadas_cache: status = ERRO, erro = mensagem
    ↓
Registrar log: FISCAL_PACOTE_ERRO
    ↓
Retornar erro ao chamador
```

---

## Operações por Bloco

### UPSERT
```sql
INSERT INTO fiscal_ncm_cache (id, codigo, descricao, ativo)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(id) DO UPDATE SET
  codigo = excluded.codigo,
  descricao = excluded.descricao,
  ativo = excluded.ativo,
  atualizado_em = CURRENT_TIMESTAMP
```

### DELETE_LOGICO
```sql
UPDATE fiscal_ncm_cache
SET ativo = 0, atualizado_em = CURRENT_TIMESTAMP
WHERE id = ?1
```

> Nunca é executado `DELETE FROM` em tabelas de cache fiscal. Os dados são mantidos historicamente com `ativo = 0`.

---

## Idempotência

A idempotência é garantida em dois níveis:

1. **Por `payload_hash`:** Se o hash do payload já consta como `APLICADO` na tabela `fiscal_versoes_aplicadas_cache`, a aplicação é abortada imediatamente com retorno de sucesso.
2. **Por transação SQLite:** O uso de `ON CONFLICT DO UPDATE` garante que reaplicar um pacote idêntico não duplica dados.

---

## Logs Técnicos

| Evento | Quando |
|--------|--------|
| `FISCAL_PACOTE_RECEBIDO` | Ao receber o command `aplicar_pacote_fiscal` |
| `FISCAL_PACOTE_VALIDADO` | Após parse JSON bem-sucedido |
| `FISCAL_PACOTE_APLICADO` | Após commit com sucesso |
| `FISCAL_PACOTE_IGNORADO_IDEMPOTENTE` | Quando hash já foi aplicado |
| `FISCAL_PACOTE_ERRO` | Em qualquer falha (JSON inválido, SQL, etc.) |

---

## Commands Tauri

| Command | Assinatura |
|---------|-----------|
| `aplicar_pacote_fiscal` | `(estado: State<EstadoApp>, req: AplicarPacoteFiscalReq) → Result<String, String>` |
| `validar_pacote_fiscal_local` | `(req: AplicarPacoteFiscalReq) → Result<bool, String>` |
| `obter_status_versao_fiscal_local` | `(estado: State<EstadoApp>) → Result<StatusVersaoFiscalResp, String>` |
| `listar_logs_sync_fiscal` | `(estado: State<EstadoApp>) → Result<Vec<LogSyncFiscalResp>, String>` |

---

## Regras de Domínio — O que NÃO é feito aqui

- ❌ Não emite NF-e, NFC-e, SAT, SIFEN
- ❌ Não gera XML assinado
- ❌ Não gera DANFE, KuDE ou QR Fiscal
- ❌ Não transmite para SEFAZ, DNIT ou SIFEN
- ❌ Não altera venda, caixa, estoque, financeiro, compras, delivery ou gourmet
- ❌ Não usa certificado digital
- ❌ Não usa float/double (alíquotas em i64 escala 6, valores em minor unit)
