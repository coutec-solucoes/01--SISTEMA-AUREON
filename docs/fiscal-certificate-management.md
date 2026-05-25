# Gestão de Certificados Fiscais A1 — Aureon

**Fase:** 18 — Bloco 1 e 1.1  
**Status:** Implementado com feature `fiscal_real`

---

## Escopo

O módulo de certificados gerencia o ciclo de leitura e validação de certificados digitais A1 (PFX/P12) para uso na assinatura técnica de documentos fiscais de preview na Retaguarda.

## Tipos Suportados

| Tipo | Suportado | Notas |
|---|---|---|
| A1 (PFX/P12) | ✅ | Suporte completo com `fiscal_real` |
| A3/smartcard | ❌ | Fora do escopo da Fase 18 |
| HSM/token USB | ❌ | Fora do escopo da Fase 18 |
| Assinatura remota | ❌ | Fora do escopo da Fase 18 |

## Endpoints

| Método | Rota | Descrição |
|---|---|---|
| POST | `/fiscal/certificado/validar` | Valida um PFX/P12 e retorna metadados |
| GET | `/fiscal/certificado/status` | Retorna status do último certificado validado |

## Regras de Segurança

- **Senha:** Nunca persistida. Usada apenas em memória durante a validação.
- **Chave privada:** Nunca retornada na resposta da API.
- **Chave privada:** Nunca enviada ao PDV em qualquer circunstância.
- **PFX completo:** Nunca armazenado em banco de dados.
- **Logs:** O conteúdo binário do PFX não é logado.

## Metadados Retornados

```json
{
  "valido": true,
  "cn": "RAZAO SOCIAL EMPRESA LTDA",
  "cnpj": "12345678000195",
  "numero_serie": "0A1B2C...",
  "validade_inicio": "2024-01-01T00:00:00Z",
  "validade_fim": "2026-01-01T00:00:00Z",
  "dias_para_expirar": 220,
  "expirado": false,
  "alerta_expiracao": false,
  "mensagem": "Certificado válido"
}
```

## Feature Flag `fiscal_real`

A validação real de PFX/P12 depende da crate `openssl`. Para evitar quebra do build Windows (que requer Perl e vcpkg para compilar openssl nativo), a dependência é opcional:

```toml
openssl = { version = "0.10", optional = true, features = ["vendored"] }

[features]
fiscal_real = ["openssl"]
```

- **Build padrão:** mock de diagnóstico retorna dados simulados.
- **Build com `--features fiscal_real`:** validação real do PFX/P12 via OpenSSL.

## Alerta de Expiração

- Se o certificado expira em ≤ 30 dias: `alerta_expiracao = true`.
- Se o certificado já expirou: `expirado = true`.

## Migration PostgreSQL

**Arquivo:** `database/migrations/postgresql/014_fase18_certificados_fiscais.sql`  
**Conteúdo:** Adiciona campos de metadados de certificado (sem armazenar a chave privada ou senha).
