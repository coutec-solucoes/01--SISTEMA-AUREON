# QR Code Fiscal Preview — Homologação Técnica

**Fase:** 18 — Bloco 6  
**Status:** QR Code preview implementado. QR fiscal oficial NÃO implementado.

---

## ⚠️ AVISO CRÍTICO

> **O QR Code gerado por estes endpoints é um QR técnico de homologação.**
> **Não há cHashQR oficial.**
> **Não há consulta à SEFAZ ou ao SET/DNIT.**
> **O QR NÃO possui validade fiscal.**
> **Não apresente este QR ao consumidor como documento fiscal oficial.**

---

## Endpoints

| Método | Rota | Descrição |
|---|---|---|
| POST | `/fiscal/preview/qrcode` | Gera QR preview por dados informados |
| GET | `/fiscal/preview/qrcode/nfce/{chave_preview}` | Gera QR preview para chave NFC-e/NF-e |
| GET | `/fiscal/preview/qrcode/sifen/{cdc_preview}` | Gera QR preview para CDC SIFEN |

## Tipos Suportados

| Tipo | Região | Descrição |
|---|---|---|
| `NFCE` | BR | NFC-e (preview) |
| `NFE` | BR | NF-e (preview) |
| `SIFEN` | PY | SIFEN/DTE (preview) |

## Formato de Saída

> **⚠️ Imprecisão semântica documentada:**
> O campo de resposta se chama `png_base64`, mas o conteúdo retornado é **SVG em Base64**.
>
> **Motivo:** A geração PNG via crate `image` apresentou conflito de versão com `qrcode` 0.14. A solução adotada foi gerar SVG vetorial, que é:
> - Mais leve
> - Compatível com `<img src="data:image/svg+xml;base64,...">` no browser
> - Sem dependência nativa de libpng
>
> Para uso no frontend:
> ```html
> <img src="data:image/svg+xml;base64,{CONTEUDO_DO_CAMPO_png_base64}" />
> ```

## Conteúdo do QR (NFC-e/NF-e BR)

```
https://homologacao.sefaz.gov.br/preview/qrcode?p=CHAVE|2|1|1|PREVIEW_SEM_VALIDADE_FISCAL_HOMOLOGACAO
```

- `CHAVE`: 44 dígitos da chave técnica de preview
- `2`: tpAmb (Homologação)
- `PREVIEW_SEM_VALIDADE_FISCAL_HOMOLOGACAO`: marcador explícito

## Conteúdo do QR (SIFEN/DTE PY)

```
https://ekuatia.set.gov.py/consultas-test/qr?nIdFisc=CDC&PREVIEW_SEM_VALIDADE_FISCAL_HOMOLOGACAO
```

- `CDC`: 44 dígitos do CDC técnico de preview
- `PREVIEW_SEM_VALIDADE_FISCAL_HOMOLOGACAO`: marcador explícito

## O Que Este Módulo NÃO Faz

- ❌ Não gera cHashQR (hash oficial do QR NFC-e)
- ❌ Não consulta a SEFAZ
- ❌ Não consulta o SET/DNIT/SIFEN
- ❌ Não valida a chave em órgão oficial
- ❌ Não gera QR fiscal homologado
- ❌ Não usa PNG nativo (SVG base64)
- ❌ Não cria vínculo com documento autorizado

## Dependência Adicionada

```toml
qrcode = { version = "0.14", features = ["image"] }
image = { version = "0.24", default-features = false, features = ["png"] }
```

A crate `image` foi mantida por compatibilidade de workspace, mas a geração real usa SVG do `qrcode` diretamente.
