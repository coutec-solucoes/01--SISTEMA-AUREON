# Licensing Signature — Fase 20 Bloco 4
## Assinatura Criptográfica de Payload de Licença (Ed25519)

---

## Por que assinatura assimétrica?

A assinatura assimétrica resolve o problema de **autenticidade sem conectividade permanente**:

- A Retaguarda (servidor) assina o payload com sua **chave privada** — mantida secreta.
- O PDV (cliente) pode verificar a assinatura usando apenas a **chave pública** — distribuída livremente.
- Se o payload for adulterado localmente, a verificação **falha** — mesmo sem internet.
- A chave privada **nunca sai da Retaguarda**.

Alternativa simétrica (HMAC) foi descartada porque exigiria que o PDV possuísse o segredo compartilhado, criando risco de exposição.

---

## Algoritmo: Ed25519

| Propriedade         | Valor                        |
|---------------------|------------------------------|
| Algoritmo           | Ed25519 (RFC 8032)           |
| Tamanho da chave    | 32 bytes (privada), 32 bytes (pública) |
| Tamanho da assinatura | 64 bytes                  |
| Encoding de saída   | Base64 padrão (RFC 4648)     |
| Hash coberto        | SHA-256 do payload canônico  |
| Crate               | `ed25519-dalek v2`           |

Ed25519 foi escolhido por:
- **Simplicidade**: API limpa, sem parâmetros de curva para configurar.
- **Velocidade**: Mais rápido que ECDSA P-256 para assinar e verificar.
- **Assinatura curta**: 64 bytes em base64 = ~88 chars — cabe confortavelmente em JSON.
- **Segurança**: Resistente a ataques de timing por design (nonce determinístico).

---

## O que é `payload_hash`?

O `payload_hash` é o **SHA-256 em hexadecimal** do `payload_licenca_json` canônico (string).

```
payload_hash = SHA-256(payload_licenca_json_canonico)
```

**Uso dual**:
1. A assinatura Ed25519 é calculada **sobre o hash** (não o payload bruto).
2. O PDV pode verificar o hash antes de verificar a assinatura, detectando adulteração rápido.

**Formato de saída**: lowercase hex, 64 caracteres (32 bytes SHA-256).

---

## O que é `key_id`?

O `key_id` identifica qual chave foi usada para assinar o payload.

| Valor                | Significado                               |
|----------------------|-------------------------------------------|
| `dev-efemero-v1`     | Chave DEV gerada em runtime (não persiste)|
| `prod-v1`            | Primeira chave de produção configurada    |

O `key_id` permite **rotação futura de chaves**: o PDV pode ter múltiplas chaves públicas
armazenadas e selecionar a correta pelo `key_id` presente no payload.

---

## Como o payload é canonicalizado?

O JSON canônico é construído com campos em **ordem fixa e explícita**, sem depender de `serde_json` (que não garante ordem):

```json
{"empresa_id":"...","licenca_id":"...","plano_codigo":"...","status":"...","validade":"...","terminal":"...","tolerancia_offline_dias":10,"emitido_em":"..."}
```

**Regras de canonicalização**:
1. Campos sempre na mesma ordem — declarada em `PayloadCanonicoLicenca`.
2. Sem espaços extras, sem quebras de linha (`compact`).
3. Todos os campos obrigatórios — nenhum opcional pode estar ausente.
4. Sem `float`/`double` — apenas strings e inteiros (`i64`).
5. Strings escapadas conforme JSON RFC 8259.
6. `tolerancia_offline_dias` é inteiro sem aspas.

**Campos obrigatórios para assinatura**:

| Campo                     | Tipo    | Descrição                        |
|---------------------------|---------|----------------------------------|
| `empresa_id`              | string  | ID da empresa licenciada         |
| `licenca_id`              | string  | ID da licença                    |
| `plano_codigo`            | string  | Código do plano                  |
| `status`                  | string  | Status da licença (`ATIVA`, etc.)|
| `validade`                | string  | Data de validade ou `"null"`     |
| `terminal`                | string  | ID ou nome do terminal           |
| `tolerancia_offline_dias` | integer | Dias de tolerância offline       |
| `emitido_em`              | string  | Timestamp ISO-8601 da emissão    |

---

## Como o PDV validará futuramente (offline)

O PDV receberá do payload assinado:
1. `payload_licenca_json` — string canônica
2. `algoritmo_assinatura` — `"Ed25519"`
3. `key_id` — identificador da chave
4. `assinatura_licenca` — base64 de 64 bytes
5. `payload_hash` — hex SHA-256

**Processo de verificação offline no PDV**:
```
1. Calcular SHA-256(payload_licenca_json) → hash_local
2. Comparar hash_local == payload_hash (adulteração rápida)
3. Buscar chave pública pelo key_id na lista local
4. Ed25519.verify(hash_local, assinatura_licenca, chave_pública)
5. Se válido → pode_operar = true (dentro da tolerância)
```

A chave pública será distribuída ao PDV na sincronização inicial (Bloco futuro).

---

## Onde fica a chave privada

| Local                            | Status                         |
|----------------------------------|--------------------------------|
| Retaguarda (memória estática)    | ✅ Único local permitido       |
| Variável de ambiente             | ✅ `AUREON_LICENSE_PRIVATE_KEY_B64` |
| Código-fonte / git               | ❌ PROIBIDO                    |
| PDV / cliente                    | ❌ PROIBIDO                    |
| Logs / respostas HTTP            | ❌ PROIBIDO                    |
| Banco de dados                   | ❌ Não recomendado sem HSM     |

---

## Riscos da chave DEV (modo efêmero)

Em modo DEV (`AUREON_LICENSE_PRIVATE_KEY_B64` ausente):
- A chave é gerada em memória com `OsRng` no startup.
- Cada reinício gera uma **chave diferente** — payloads assinados anteriormente **deixam de ser válidos**.
- O sistema retorna `warnings` explícitos avisando sobre modo DEV.
- **NÃO use em produção com chave efêmera.**

---

## Rotação futura de chaves

1. Gerar nova chave com `AUREON_LICENSE_PRIVATE_KEY_B64` com novo valor.
2. Incrementar `key_id` → `prod-v2`.
3. Distribuir nova chave pública ao PDV via sincronização.
4. PDV mantém lista de chaves públicas por `key_id`.
5. Payloads antigos (assinados com `prod-v1`) continuam válidos enquanto a chave `prod-v1` estiver na lista do PDV.

---

## Endpoints implementados

| Método | Endpoint                                        | Descrição                          |
|--------|-------------------------------------------------|------------------------------------|
| GET    | `/licenciamento/licencas/{id}/payload-assinado` | Gera payload com assinatura real   |
| POST   | `/licenciamento/licencas/verificar-payload`     | Verifica assinatura de um payload  |
| GET    | `/licenciamento/chaves/status`                  | Retorna chave pública e modo ativo |

---

## Segurança: O que NUNCA fazer

- ❌ Logar a chave privada em qualquer nível de log.
- ❌ Retornar a chave privada em qualquer endpoint.
- ❌ Enviar a chave privada ao PDV.
- ❌ Hardcodar segredo no código-fonte.
- ❌ Assinar payload com campos críticos ausentes.
- ❌ Usar algoritmo simétrico compartilhado com o PDV.
