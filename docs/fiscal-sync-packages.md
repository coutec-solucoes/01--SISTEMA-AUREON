# Pacotes de Sincronização Fiscal (SYNC_FISCAL)

## Visão Geral

Quando uma versão fiscal é publicada pela Retaguarda, um **pacote de sincronização fiscal** é gerado e armazenado nas tabelas da infraestrutura de Sync da Fase 6 (`pacotes_sincronizacao` / `pacotes_sincronizacao_itens`), com `tipo_pacote = 'SYNC_FISCAL'`.

Os PDVs consomem esse pacote via endpoint de sync e aplicam localmente.

---

## Estrutura do Payload SYNC_FISCAL

```json
{
  "versao_fiscal_id": "uuid-da-versao",
  "versao": "2026.001",
  "pais_fiscal": "BR",
  "payload_hash": "hash-determinístico",
  "total_registros": 42,
  "publicado_em": "2026-05-24T22:00:00Z",
  "blocos": {
    "fiscal_empresa_config": [
      {
        "operacao": "UPSERT",
        "id": "uuid",
        "pais_fiscal": "BR",
        "ambiente": "HOMOLOGACAO",
        "forma_emissao": "NORMAL",
        "regime_fiscal": "SIMPLES_NACIONAL",
        "ativo": true
      }
    ],
    "fiscal_numeracao": [ ... ],
    "fiscal_ncm": [
      {
        "operacao": "UPSERT",
        "id": "uuid",
        "codigo": "8471.30.12",
        "descricao": "Máquinas automáticas para processamento...",
        "ativo": true
      }
    ],
    "fiscal_cfop": [ ... ],
    "fiscal_cst_csosn": [ ... ],
    "fiscal_iva": [
      {
        "operacao": "UPSERT",
        "id": "uuid",
        "codigo": "IVA10",
        "descricao": "IVA 10% Geral",
        "pais_fiscal": "PY",
        "aliquota_escala6": 100000,
        "ativo": true
      }
    ],
    "fiscal_regras_tributarias": [ ... ]
  }
}
```

---

## Operações por Item

| Operação | Efeito no Cache do PDV |
|----------|----------------------|
| `UPSERT` | INSERT ou UPDATE via `ON CONFLICT DO UPDATE` |
| `DELETE_LOGICO` | `UPDATE SET ativo = 0` — nunca DELETE físico |

---

## Publicação de Versão

1. **Endpoint:** `POST /fiscal/versoes/{id}/publicar`
2. **Pré-condição:** Versão deve estar em status `RASCUNHO`
3. **Processo:**
   - Consultar 7 tabelas mestre no PostgreSQL
   - Montar payload JSON com blocos
   - Calcular hash do payload
   - Salvar payload em `pacotes_sincronizacao_itens`
   - Mudar status da versão para `PUBLICADA`
   - Registrar auditoria

---

## Reprocessamento

1. **Endpoint:** `POST /fiscal/versoes/{id}/reprocessar`
2. **Pré-condição:** Versão deve estar `PUBLICADA` ou `REPROCESSADA`
3. **Processo:** Idêntico à publicação — cria novo hash, novo pacote, novo registro de auditoria
4. **Status final:** `REPROCESSADA`

---

## Idempotência

- Cada operação de publicação usa `idempotency_key` único (UUID)
- Tabela `sync_idempotencia` garante que o mesmo `idempotency_key` não seja processado duas vezes
- Em caso de retry, a operação anterior é retornada sem reprocessamento

---

## Integração com Infraestrutura de Sync (Fase 6)

| Tabela | Papel na Fase 17 |
|--------|-----------------|
| `pacotes_sincronizacao` | Cabeçalho do pacote SYNC_FISCAL |
| `pacotes_sincronizacao_itens` | JSON do payload fiscal serializado |
| `sync_idempotencia` | Prevenção de duplicação |
| `sync_publicacoes` | Controle de distribuição por terminal |

---

## Limitações Atuais

- **Sem chunking:** Payload completo em um único JSON. Para NCM/CFOP massivos (>10.000 itens), será necessária paginação em fase futura.
- **Hash simples:** Não criptográfico — adequado para controle de versão de app.
