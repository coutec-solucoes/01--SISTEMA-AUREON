# Builder XML NFC-e/NF-e — Preview de Homologação (BR)

**Fase:** 18 — Bloco 3  
**Status:** Preview técnico implementado. Transmissão SEFAZ NÃO implementada.

---

## ⚠️ AVISO CRÍTICO

> **O XML gerado por estes endpoints é um documento técnico de homologação.**
> `tpAmb=2` (Homologação) é obrigatório e imutável.
> **O XML NÃO é transmitido para a SEFAZ.**
> **O XML NÃO possui protocolo de autorização.**
> **O XML NÃO possui validade fiscal ou jurídica.**

---

## Endpoints

| Método | Rota | Descrição |
|---|---|---|
| POST | `/fiscal/nfce/preview/montar` | Monta XML preview sem assinatura |
| POST | `/fiscal/nfce/preview/montar-assinar` | Monta XML preview com assinatura técnica |
| GET | `/fiscal/nfce/preview/venda/{venda_id}` | Recupera preview por venda |

## Modelos Suportados

| Modelo | Código | Descrição |
|---|---|---|
| NFC-e | 65 | Nota Fiscal de Consumidor Eletrônica |
| NF-e | 55 | Nota Fiscal Eletrônica |

## Regras do XML Preview

1. `tpAmb` sempre `2` (Homologação) — nunca `1` (Produção).
2. O XML contém o aviso: `DOCUMENTO TECNICO DE HOMOLOGACAO SEM VALIDADE FISCAL`.
3. A chave de acesso gerada é técnica/simulada (44 dígitos, sem vinculação ao SEFAZ).
4. O campo `<infProt>` (protocolo de autorização) está **ausente**.
5. O ambiente `PRODUCAO` é **bloqueado** no request.

## O Que Este Builder NÃO Faz

- ❌ Não transmite para SEFAZ
- ❌ Não gera protocolo de autorização
- ❌ Não gera `nProt` (número do protocolo)
- ❌ Não gera `chNFe` oficialmente registrada
- ❌ Não gera DANFE (PDF fiscal)
- ❌ Não gera QR Code fiscal oficial com `cHashQR`
- ❌ Não assina com XMLDSig/C14N definitivo
- ❌ Não consulta status na SEFAZ

## Matemática

- Valores monetários: `i64` em minor unit (centavos para BRL).
- Quantidades: `i64` em escala 3 (ex: `1000` = `1.000`).
- Alíquotas: `i64` em escala 6 (ex: `120000` = `12.0000%`).
- **Ausência de `float`/`f64`/`double` em todos os cálculos fiscais.**

## Exemplo de Requisição

```json
POST /fiscal/nfce/preview/montar
{
  "venda_id": "uuid-da-venda",
  "modelo": "NFCE",
  "uf": "SP",
  "ambiente": "HOMOLOGACAO"
}
```

## Exemplo de Resposta (resumida)

```json
{
  "sucesso": true,
  "xml_preview": "<?xml version=\"1.0\"?><NFe>...<tpAmb>2</tpAmb>...DOCUMENTO TECNICO DE HOMOLOGACAO SEM VALIDADE FISCAL...</NFe>",
  "chave_preview": "35240512345678000195650010000000011000000019",
  "ambiente": "HOMOLOGACAO",
  "assinado": false,
  "mensagem": "XML preview gerado com sucesso.",
  "warnings": [
    "DOCUMENTO TÉCNICO DE HOMOLOGAÇÃO SEM VALIDADE FISCAL",
    "Não transmita este XML para a SEFAZ"
  ]
}
```
