# Assinatura Digital XML — Técnica de Homologação (Preview)

**Fase:** 18 — Bloco 2  
**Status:** Assinatura PREVIEW implementada. XMLDSig/C14N oficial é PENDÊNCIA.

---

## ⚠️ AVISO CRÍTICO

> **A assinatura implementada na Fase 18 é uma assinatura técnica de preview.**
> Ela não corresponde ao XMLDSig/C14N exigido pelo MOC da SEFAZ para NF-e/NFC-e real.
> Um documento assinado por estes endpoints **NÃO possui validade fiscal**.
> **Não transmita para SEFAZ. Não use como documento autorizado.**

---

## Escopo da Fase 18

A assinatura de preview permite:
- Validar que o certificado A1 pode assinar XML tecnicamente.
- Verificar a estrutura do XML após assinatura.
- Testar o fluxo completo de preview sem transmissão.

## Endpoints

| Método | Rota | Descrição |
|---|---|---|
| POST | `/fiscal/assinatura/testar` | Testa conectividade e certificado |
| POST | `/fiscal/assinatura/assinar-preview` | Assina XML em modo preview/homologação |
| POST | `/fiscal/assinatura/verificar-preview` | Verifica assinatura de um XML preview |

## Algoritmos Suportados

| Algoritmo | Suporte | Notas |
|---|---|---|
| RSA_SHA256 | ✅ | Padrão recomendado |
| RSA_SHA1 | ✅ | Legado, evitar em novos documentos |

## Regras Obrigatórias

- O ambiente `PRODUCAO` é **bloqueado** em todos os endpoints.
- O `tpAmb=1` é **bloqueado** — retorna erro explícito.
- A senha do certificado é usada apenas em memória durante a assinatura.
- A chave privada não é persistida, retornada ou enviada ao PDV.
- O XML deve ter no máximo **5 MB**.
- A assinatura só ocorre em ambiente `HOMOLOGACAO`.

## O Que Esta Implementação NÃO Faz

- ❌ Não gera assinatura XMLDSig/C14N conforme MOC SEFAZ
- ❌ Não insere `<Signature>` conforme padrão ICP-Brasil/W3C
- ❌ Não realiza canonicalização C14N do XML
- ❌ Não gera `DigestValue` e `SignatureValue` fiscalmente válidos
- ❌ Não assina `infNFe` conforme leiaute NF-e v4.00
- ❌ Não transmite para SEFAZ
- ❌ Não gera protocolo de autorização

## Pendência Técnica Documentada

> **XMLDSig/C14N definitivo** ficará para fase futura.
>
> Opções para implementação futura:
> - `xmlsec` crate (Rust binding para libxmlsec1)
> - `libxmlsec1` via FFI
> - Serviço externo de assinatura (ex: Assinar Online, Certisign)
>
> Pré-requisitos para fase futura:
> - Ambiente Linux ou compilação cruzada com libxmlsec1
> - Definição de política de chave (A1 local, A3, HSM)
> - MOC NF-e/NFC-e v4.00 para validar o leiaute da assinatura

## Exemplo de Requisição

```json
POST /fiscal/assinatura/assinar-preview
{
  "xml_conteudo": "<NFe>...</NFe>",
  "pfx_base64": "MIIJ...",
  "senha": "senha_cert",
  "ambiente": "HOMOLOGACAO"
}
```

## Exemplo de Resposta

```json
{
  "sucesso": true,
  "xml_assinado": "<NFe>...<Signature>PREVIEW_TECNICO</Signature></NFe>",
  "resumo": "SHA256:abcdef...",
  "ambiente": "HOMOLOGACAO",
  "mensagem": "Assinatura técnica de preview aplicada. NÃO é XMLDSig fiscal definitivo.",
  "warnings": [
    "ESTA ASSINATURA É TÉCNICA DE HOMOLOGAÇÃO. NÃO POSSUI VALIDADE FISCAL.",
    "XMLDSig/C14N definitivo é pendência para fase futura."
  ]
}
```
