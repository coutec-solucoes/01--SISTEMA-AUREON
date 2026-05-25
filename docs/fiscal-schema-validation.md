# Validação Local de Schema Fiscal — Preview

**Fase:** 18 — Bloco 5  
**Status:** Validação estrutural simplificada. XSD/JSON Schema oficiais são PENDÊNCIA.

---

## ⚠️ AVISO CRÍTICO

> **A validação implementada na Fase 18 é ESTRUTURAL SIMPLIFICADA.**
> Ela verifica presença de tags/campos obrigatórios, mas NÃO valida conformidade completa com XSD ou JSON Schema oficial.
> **Um preview que passa nesta validação NÃO está pronto para transmissão fiscal.**
> **A validação local NUNCA representa autorização fiscal.**

---

## Endpoints

| Método | Rota | Descrição |
|---|---|---|
| POST | `/fiscal/preview/validar` | Valida XML ou JSON conforme tipo informado |
| POST | `/fiscal/preview/validar-xml` | Atalho para validação XML NFC-e/NF-e |
| POST | `/fiscal/preview/validar-sifen` | Atalho para validação JSON SIFEN |

## Tipos Suportados

| Tipo | Valor no DTO | Descrição |
|---|---|---|
| NFC-e XML | `NFCE_XML` | XML NFC-e (modelo 65) |
| NF-e XML | `NFE_XML` | XML NF-e (modelo 55) |
| SIFEN JSON | `SIFEN_JSON` | JSON SIFEN/DTE (Paraguai) |

## Regras de Validação

### Regras Gerais
- Limite de payload: **5 MB**
- Ambiente `PRODUCAO` bloqueado
- Retorna `valido: false` se limite ou ambiente violados

### XML NFC-e/NF-e (verificações)
- Presença das tags: `<NFe`, `<infNFe`, `<ide>`, `<emit>`, `<det`, `<total>`, `<pag>`, `<infAdic>`
- `<tpAmb>2</tpAmb>` obrigatório
- `<tpAmb>1</tpAmb>` bloqueado explicitamente
- Presença do aviso: `DOCUMENTO TECNICO DE HOMOLOGACAO SEM VALIDADE FISCAL`

### JSON SIFEN (verificações)
- Parse JSON válido
- Presença de `rDE` na raiz
- Presença de `DE` dentro de `rDE`
- `Signature` deve ser `null`
- Presença das chaves: `gTimb`, `gDatGralOpe`, `gDtipDE`, `gTotSub`
- Presença do aviso: `DOCUMENTO TECNICO DE HOMOLOGACAO SEM VALIDADE FISCAL`

## Severidade dos Erros

| Código | Severidade | Significado |
|---|---|---|
| `VAL_001` | ERRO | Payload excede 5MB |
| `VAL_002` | ERRO | Ambiente PRODUCAO bloqueado |
| `XML_001` | ERRO | Tag obrigatória ausente no XML |
| `XML_002` | ERRO | tpAmb=1 (Produção) detectado |
| `XML_003` | ERRO | tpAmb=2 (Homologação) ausente |
| `XML_004` | ERRO | Aviso de sem validade fiscal ausente |
| `JSON_001..006` | ERRO | Estrutura JSON SIFEN inválida |
| `JSON_PARSE_ERR` | ERRO | JSON inválido |
| `WARN_XSD` | WARNING | XSD oficial não integrado (pendência) |
| `WARN_SCHEMA` | WARNING | JSON Schema oficial não integrado (pendência) |

## Pendências Técnicas Documentadas

> **XSD oficial NF-e/NFC-e (MOC SEFAZ)** não está integrado.
>
> - Disponível em: https://www.nfe.fazenda.gov.br/portal/listaConteudo.aspx?tipoConteudo=BMPFMBoln3w=
> - Para integração futura: `quick-xml` + validação XSD via `xmlschema` ou binding nativo.
> - Até lá, a validação é estrutural simplificada.

> **JSON Schema oficial SIFEN/DNIT** não está integrado.
>
> - Disponível via portal do SET/DNIT Paraguai.
> - Para integração futura: `jsonschema` crate ou equivalente.
> - Até lá, a validação é estrutural simplificada.

## Exemplo de Resposta

```json
{
  "valido": true,
  "tipo": "NFCE_XML",
  "ambiente": "HOMOLOGACAO",
  "total_erros": 0,
  "erros": [],
  "warnings": [
    {
      "codigo": "WARN_XSD",
      "mensagem": "Validação estrutural XML simplificada (sem XSD oficial). Pendência: Integração com schema XSD governamental.",
      "severidade": "WARNING"
    }
  ],
  "mensagem": "Validação de preview concluída com sucesso (Estrutura Básica OK)."
}
```
