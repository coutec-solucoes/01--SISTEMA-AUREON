# Fase 18 — Homologação Técnica Fiscal: Certificados, Assinatura, XML/JSON Preview e Validação Local

**Status:** APROVADA E ENCERRADA COM RESSALVAS CONTROLADAS  
**Branch:** main  
**Data de início:** 2026-05  
**Data de encerramento:** 2026-05-25  

---

## Objetivo

Criar a infraestrutura técnica de **homologação fiscal** — sem emissão, sem transmissão para SEFAZ/DNIT/SIFEN e sem geração de documentos com validade jurídica.

A Fase 18 prepara o sistema para o futuro processo de integração fiscal real, estabelecendo:
- Leitura segura de certificados A1 (PFX/P12);
- Assinatura digital técnica (preview) de XML;
- Geração de XML NFC-e/NF-e de preview/homologação;
- Geração de JSON SIFEN/DTE de preview/homologação (Paraguai);
- Validação estrutural local de XML e JSON de preview;
- Geração de QR Code técnico de homologação;
- Interface Blazor para visualização e diagnóstico de todos os artefatos acima.

---

## Blocos Implementados

| Bloco | Título | Commit | Status |
|---|---|---|---|
| Bloco 1 | Gestão de Certificados Digitais A1/PFX/P12 | `1b0343f` | ✅ Aprovado |
| Bloco 1.1 | Correção da Validação Real do Certificado A1 | `878b081` | ✅ Aprovado |
| Bloco 2 | Assinatura Digital XML Técnica em Homologação | `bc4831e` | ✅ Aprovado |
| Bloco 3 | Montagem de XML NFC-e/NF-e Preview em Homologação | `5867a3e` | ✅ Aprovado |
| Bloco 4 | Montagem de JSON SIFEN/DTE Preview Paraguai | `6240138` | ✅ Aprovado |
| Bloco 5 | Validação Local de Schema — XSD e JSON Schema | `34a4a22` | ✅ Aprovado |
| Bloco 6 | QR Code Fiscal Preview — Homologação Técnica | `6ff6d38` | ✅ Aprovado |
| Bloco 7 | UI Preview Fiscal Assinado — XML/JSON/QR Técnico | `dc16d8d` | ✅ Aprovado |
| Bloco 8 | Documentação Final e Encerramento | _(este commit)_ | ✅ Aprovado |

---

## Endpoints Criados (aureon-api-local)

### Certificado A1
- `POST /fiscal/certificado/validar`
- `GET /fiscal/certificado/status`

### Assinatura Preview
- `POST /fiscal/assinatura/testar`
- `POST /fiscal/assinatura/assinar-preview`
- `POST /fiscal/assinatura/verificar-preview`

### NFC-e/NF-e Preview (BR)
- `POST /fiscal/nfce/preview/montar`
- `POST /fiscal/nfce/preview/montar-assinar`
- `GET /fiscal/nfce/preview/venda/{venda_id}`

### SIFEN/DTE Preview (PY)
- `POST /fiscal/sifen/preview/montar`
- `GET /fiscal/sifen/preview/venda/{venda_id}`

### Validação Local
- `POST /fiscal/preview/validar`
- `POST /fiscal/preview/validar-xml`
- `POST /fiscal/preview/validar-sifen`

### QR Code Preview
- `POST /fiscal/preview/qrcode`
- `GET /fiscal/preview/qrcode/nfce/{chave_preview}`
- `GET /fiscal/preview/qrcode/sifen/{cdc_preview}`

**Total: 15 endpoints de homologação técnica**

---

## Telas Criadas (Blazor Retaguarda)

| Arquivo | Rota | Descrição |
|---|---|---|
| `FiscalPreviewAssinado.razor` | `/fiscal/preview-assinado` | UI preview técnico completo |

---

## Dependências Adicionadas

| Crate | Versão | Uso | Notas |
|---|---|---|---|
| `openssl` | `0.10` | Validação real de PFX/P12 | Ativada pela feature `fiscal_real` |
| `qrcode` | `0.14` | Geração de QR Code SVG | Feature `image` habilitada |
| `image` | `0.24` | Suporte PNG (não usado; mantido como opcional) | `default-features = false` |

---

## Estratégia de Certificado A1

- Apenas certificados A1 (arquivo PFX/P12) são suportados.
- A senha é usada apenas em memória durante a validação.
- A senha nunca é persistida em banco de dados.
- A chave privada nunca é retornada em resposta de API.
- A chave privada nunca é enviada ao PDV.
- Certificados A3/HSM/smartcard/token USB ficaram **fora do escopo da Fase 18**.
- A feature `fiscal_real` ativa o `openssl` para validação real; sem ela, o build retorna diagnóstico mock.

---

## Estratégia de Assinatura Preview

- Utiliza RSA_SHA256 (padrão) ou RSA_SHA1 (legado).
- A assinatura gera um XML tecnicamente estruturado, mas **não é XMLDSig/C14N conforme MOC SEFAZ**.
- A assinatura preview é armazenada apenas em memória durante a requisição.
- A senha e a chave privada não são gravadas em nenhum log ou banco.
- O ambiente `PRODUCAO / tpAmb=1` é **bloqueado** em todos os endpoints de assinatura.
- **Pendência confirmada:** integração com `xmlsec` / `libxmlsec1` para XMLDSig/C14N definitivo ficará para fase futura.

---

## Estratégia XML NFC-e/NF-e Preview

- O XML gerado é estruturado para homologação técnica, não para transmissão.
- `tpAmb=2` é obrigatório e imutável.
- O XML contém o aviso: `DOCUMENTO TECNICO DE HOMOLOGACAO SEM VALIDADE FISCAL`.
- A tag `<infProt>` (protocolo de autorização) é **ausente** — não há simulação de protocolo.
- A chave de acesso gerada é técnica/simulada, não oficialmente válida.
- **Sem transmissão SEFAZ. Sem protocolo. Sem DANFE. Sem QR fiscal oficial.**

---

## Estratégia JSON SIFEN/DTE Preview

- JSON gerado conforme estrutura aproximada do `rDE` SIFEN/DNIT.
- `Signature` sempre `null` no preview.
- CDC gerado é técnico/simulado (44 posições, não oficialmente registrado no DNIT).
- IVA calculado em escala 6 e segregado em 10%, 5% e isento.
- PYG sem centavos — todos os valores em `i64` (sem `float/f64`).
- Ambiente `PRODUCAO` é **bloqueado**.
- **Sem transmissão DNIT/SIFEN/SET. Sem KuDE fiscal. Sem QR fiscal oficial.**

---

## Estratégia de Validação Local

- Validação estrutural simplificada por verificação de tags/chaves obrigatórias.
- Sem dependência de `libxml2` ou parser nativo complexo.
- Limite de payload: 5 MB por requisição.
- Ambiente `PRODUCAO` bloqueado.
- **Pendência confirmada:** XSD oficial NF-e/NFC-e (MOC SEFAZ) não integrado.
- **Pendência confirmada:** JSON Schema oficial SIFEN/DNIT não integrado.
- **A validação local nunca representa autorização fiscal.**

---

## Estratégia QR Code Preview

- Biblioteca: `qrcode` v0.14 (Rust).
- Formato de saída: **SVG codificado em Base64** (o campo foi nomeado `png_base64` por razões históricas — imprecisão documentada como ressalva).
- Conteúdo QR para BR: `URL_PREVIEW?p=CHAVE|2|1|1|PREVIEW_SEM_VALIDADE_FISCAL_HOMOLOGACAO`
- Conteúdo QR para PY: `URL_PREVIEW?nIdFisc=CDC&PREVIEW_SEM_VALIDADE_FISCAL_HOMOLOGACAO`
- Sem `cHashQR` oficial. Sem consulta a SEFAZ/SET.
- **O QR gerado não é QR fiscal oficial.**

---

## Checklist Final de Aceite

### ✅ Certificado A1
- [x] Migration PostgreSQL 014 criada
- [x] `POST /fiscal/certificado/validar` implementado
- [x] `GET /fiscal/certificado/status` implementado
- [x] Suporte a PFX/P12
- [x] Feature `fiscal_real` para `openssl`
- [x] Senha não persistida
- [x] Chave privada não retornada
- [x] Certificado/chave não enviados ao PDV
- [x] A3/HSM/token fora do escopo

### ✅ Assinatura XML Preview
- [x] 3 endpoints implementados
- [x] RSA_SHA1 e RSA_SHA256 suportados
- [x] Bloqueio de PRODUCAO/tpAmb=1
- [x] Assinatura é preview técnica
- [x] Não é XMLDSig/C14N definitivo (pendência documentada)

### ✅ XML NFC-e/NF-e Preview
- [x] 3 endpoints implementados
- [x] XML técnico de homologação
- [x] tpAmb=2 obrigatório
- [x] Aviso de sem validade fiscal no XML
- [x] Sem transmissão SEFAZ
- [x] Sem protocolo de autorização
- [x] Sem DANFE/QR fiscal oficial

### ✅ JSON SIFEN/DTE Preview
- [x] 2 endpoints implementados
- [x] JSON preview técnico
- [x] HOMOLOGACAO obrigatório
- [x] Signature = null
- [x] CDC preview simulado
- [x] IVA 10%, 5% e isento segregados
- [x] PYG sem centavos
- [x] Sem transmissão DNIT/SIFEN/SET
- [x] Sem KuDE/QR oficial

### ✅ Validação Local
- [x] 3 endpoints implementados
- [x] Validação estrutural simplificada
- [x] Bloqueio de PRODUCAO/tpAmb=1
- [x] Limite de 5MB
- [x] XSD oficial pendente (documentado)
- [x] JSON Schema oficial pendente (documentado)

### ✅ QR Code Preview
- [x] 3 endpoints implementados
- [x] SVG base64 gerado
- [x] Marcador PREVIEW_SEM_VALIDADE_FISCAL_HOMOLOGACAO
- [x] Sem cHashQR oficial
- [x] Sem consulta SEFAZ/SET
- [x] Não é QR fiscal oficial

### ✅ UI Preview Fiscal (Blazor)
- [x] FiscalPreviewAssinado.razor criado
- [x] Rota /fiscal/preview-assinado
- [x] 5 abas implementadas
- [x] Banner fixo de homologação sem validade fiscal
- [x] Ausência de botões Emitir/Autorizar/Transmitir
- [x] Exportação com nome preview_homologacao_sem_validade

### ✅ Fiscal Proibido — CONFIRMADO NÃO IMPLEMENTADO
- [x] Emissão NF-e → NÃO implementado
- [x] Emissão NFC-e → NÃO implementado
- [x] Emissão NFS-e → NÃO implementado
- [x] SAT → NÃO implementado
- [x] SIFEN/DTE autorizado → NÃO implementado
- [x] Transmissão SEFAZ → NÃO implementado
- [x] Transmissão DNIT/SIFEN/SET → NÃO implementado
- [x] Consulta de status fiscal → NÃO implementado
- [x] Protocolo de autorização → NÃO implementado
- [x] XML autorizado → NÃO implementado
- [x] DANFE oficial → NÃO implementado
- [x] KuDE oficial → NÃO implementado
- [x] QR fiscal oficial → NÃO implementado
- [x] Certificado A3/HSM/token → NÃO implementado
- [x] Fila de autorização fiscal → NÃO implementado

### ✅ Operacional — CONFIRMADO NÃO ALTERADO
- [x] Venda → NÃO alterada
- [x] Caixa → NÃO alterado
- [x] Estoque → NÃO alterado
- [x] Financeiro → NÃO alterado
- [x] Compras → NÃO alteradas
- [x] Delivery → NÃO alterado
- [x] Gourmet → NÃO alterado
- [x] Impressão não fiscal → NÃO alterada

### ✅ Matemática
- [x] Dinheiro em minor unit (i64)
- [x] Quantidade em escala 3
- [x] Alíquota em escala 6
- [x] Ausência de float/f64/double em cálculos fiscais críticos

---

## Limitações Conhecidas (Ressalvas Controladas)

1. Assinatura XML atual é preview técnica, não XMLDSig/C14N conforme MOC SEFAZ.
2. XSD oficial NF-e/NFC-e não integrado — validação é estrutural simplificada.
3. JSON Schema oficial SIFEN/DNIT não integrado — validação é estrutural simplificada.
4. QR Code retornado como SVG base64 (campo nomeado `png_base64` — imprecisão semântica).
5. CDC SIFEN é simulado técnico, não registrado no DNIT.
6. A feature `fiscal_real` (openssl) requer ambiente Linux/Mac para compilação real; no Windows, usa mock de diagnóstico.

---

## O Que Ficou Fora do Escopo

- Certificado A3/HSM/smartcard/token USB
- XMLDSig/C14N/xmlsec definitivo
- Transmissão para SEFAZ (webservice)
- Transmissão para DNIT/SIFEN/SET
- Autorização de NF-e/NFC-e
- Autorização de SIFEN/DTE
- DANFE (PDF fiscal)
- KuDE (fiscal PY)
- QR Code fiscal oficial
- Consulta de status em órgãos
- Protocolo de autorização
- Numeração sequencial fiscal real
- Fila de autorização
- Contingência fiscal (DPEC/offline)
- NFS-e (serviços)
- SAT/CF-e
- Fase 19 e subsequentes
