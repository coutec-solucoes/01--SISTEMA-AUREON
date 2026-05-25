# Builder JSON SIFEN/DTE — Preview de Homologação (PY)

**Fase:** 18 — Bloco 4  
**Status:** Preview técnico implementado. Transmissão DNIT/SIFEN/SET NÃO implementada.

---

## ⚠️ AVISO CRÍTICO

> **O JSON gerado por estes endpoints é um documento técnico de homologação.**
> Ambiente `HOMOLOGACAO` é obrigatório e imutável.
> **O JSON NÃO é transmitido para DNIT/SIFEN/SET.**
> **O JSON NÃO possui autorização.**
> **O CDC gerado é técnico/simulado — NÃO é CDC oficialmente registrado.**
> **O JSON NÃO possui validade fiscal ou jurídica.**

---

## Endpoints

| Método | Rota | Descrição |
|---|---|---|
| POST | `/fiscal/sifen/preview/montar` | Monta JSON preview SIFEN/DTE |
| GET | `/fiscal/sifen/preview/venda/{venda_id}` | Recupera preview por venda |

## Estrutura do JSON Preview

O JSON gerado é inspirado na estrutura do `rDE` (Registro del Documento Electrónico) do SIFEN/DNIT, mas é **técnico/simulado**:

```json
{
  "rDE": {
    "DE": {
      "gTimb": { "iTiDE": 1, "dNumTim": "00000000", ... },
      "gDatGralOpe": { "dFeEmiDE": "2026-05-25", ... },
      "gDtipDE": { ... },
      "gTotSub": {
        "dSubExe": 0,
        "dSubExo": 0,
        "dSub5": 5000,
        "dSub10": 10000,
        "dIVA5": 238,
        "dIVA10": 909,
        "dTotIVA": 1147
      }
    },
    "Signature": null
  }
}
```

## Regras do JSON Preview

1. `Signature` sempre `null` no preview.
2. CDC gerado é técnico/simulado (44 posições).
3. IVA segregado em 10%, 5% e isento.
4. Valores PYG sem centavos (moeda sem frações decimais).
5. Ambiente `PRODUCAO` é **bloqueado**.
6. O JSON contém o marcador: `DOCUMENTO TECNICO DE HOMOLOGACAO SEM VALIDADE FISCAL`.

## O Que Este Builder NÃO Faz

- ❌ Não transmite para DNIT/SIFEN/SET
- ❌ Não registra o documento no DNIT
- ❌ Não gera CDC oficial
- ❌ Não obtém aprovação do SET
- ❌ Não gera KuDE (Kua DE — documento fiscal impresso PY)
- ❌ Não gera QR Code fiscal oficial SET
- ❌ Não assina com a estrutura criptográfica oficial SIFEN

## Cálculo de IVA (Matemática)

| Operação | Tipo | Alíquota |
|---|---|---|
| IVA10 | `valor * 10 / 110` | 10% incluído no preço |
| IVA5 | `valor * 5 / 105` | 5% incluído no preço |
| Isento | `0` | Sem IVA |

- **Sem `float`/`f64`.** Todos os cálculos em `i64`.
- PYG: sem divisão por centavos (1 PYG = 1 unidade).

## Tipos de Documento Suportados (Preview)

| Código | Tipo |
|---|---|
| 1 | Factura Electrónica (preview) |
| 5 | Nota de Crédito (preview) |
