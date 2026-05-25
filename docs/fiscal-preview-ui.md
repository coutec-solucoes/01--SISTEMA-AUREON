# UI Preview Fiscal Técnico — FiscalPreviewAssinado

**Fase:** 18 — Bloco 7  
**Arquivo:** `apps/aureon-retaguarda/ui-blazor/Pages/Fiscal/FiscalPreviewAssinado.razor`  
**Rota:** `/fiscal/preview-assinado`  
**Acesso:** Menu Fiscal > Preview Técnico

---

## ⚠️ AVISO CRÍTICO (Exibido na Tela)

> **DOCUMENTO EM HOMOLOGAÇÃO — SEM VALIDADE FISCAL**
> NÃO FOI TRANSMITIDO, AUTORIZADO OU VALIDADO POR ÓRGÃO FISCAL.
> Este módulo é exclusivamente para diagnóstico e preparação técnica.

---

## Estrutura da Tela

### Banner Fixo (topo, vermelho/laranja)
- Sempre visível em todas as abas.
- Exibe o aviso obrigatório de homologação.
- Não pode ser fechado ou ocultado.

### Abas

| Aba | Descrição |
|---|---|
| 🔐 Certificado A1 | Validação de certificado PFX/P12 |
| 📄 NFC-e/NF-e XML Preview | Montagem de XML técnico de homologação |
| 🧾 SIFEN JSON Preview | Montagem de JSON técnico SIFEN (PY) |
| ✅ Validação Local | Validação estrutural de XML/JSON preview |
| 📷 QR Code Preview | Geração e visualização de QR técnico |

---

## Nomenclatura Permitida (Botões e Textos)

| ✅ Permitido | ❌ Proibido |
|---|---|
| Montar Preview | Emitir NF-e |
| Montar XML Preview | Emitir NFC-e |
| Montar JSON Preview SIFEN | Autorizar |
| Montar + Assinar Preview Técnico | Transmitir SEFAZ |
| Validar Localmente | Transmitir SIFEN |
| Gerar QR Preview | Gerar DANFE |
| Exportar XML Técnico | Gerar KuDE Fiscal |
| Exportar JSON Técnico | QR Fiscal Oficial |
| Buscar Preview por Venda | — |
| Validar Certificado Técnico | — |

---

## Aba Certificado A1

- Campo: caminho do PFX no servidor (opcional)
- Campo: PFX em Base64 (opcional)
- Campo: senha (type="password" — nunca exibida)
- Botão: **Validar Certificado Técnico**
- Resultado: CN, CNPJ, número de série, validade, dias para expirar
- Alerta visual se expirado ou expirando em 30 dias
- Aviso fixo: "O certificado fica na Retaguarda e não vai para o PDV"
- A senha é limpa da memória imediatamente após a validação

---

## Aba NFC-e/NF-e XML Preview

- Campo: Venda ID
- Campo: Modelo (NFC-e / NF-e)
- Campo: UF
- Campo: Ambiente (fixo: HOMOLOGACAO, desabilitado)
- Botão: **Montar XML Preview**
- Botão: **Montar + Assinar Preview Técnico**
- Botão: Buscar Preview por Venda
- Atalho: Usar na Validação Local
- Atalho: Exportar XML Técnico (nome: `preview_homologacao_sem_validade_nfce.xml`)
- Badge vermelho: HOMOLOGAÇÃO — SEM VALIDADE FISCAL

---

## Aba SIFEN JSON Preview

- Campo: Venda ID
- Campo: Ambiente (fixo: HOMOLOGACAO, desabilitado)
- Botão: **Montar JSON Preview SIFEN**
- Botão: Buscar Preview por Venda
- Atalho: Usar na Validação Local
- Atalho: Exportar JSON Técnico (nome: `preview_homologacao_sem_validade_sifen.json`)
- Atalho: Gerar QR Preview deste CDC
- Badge vermelho: HOMOLOGAÇÃO — SEM VALIDADE FISCAL
- Exibe CDC Preview (simulado técnico)

---

## Aba Validação Local

- Campo: Tipo de Preview (NFCE_XML / NFE_XML / SIFEN_JSON)
- Campo: Conteúdo (XML ou JSON — pode ser colado manualmente ou preenchido pelas abas)
- Campo: Ambiente (fixo: HOMOLOGACAO, desabilitado)
- Botão: **Validar Localmente**
- Resultado: total de erros, lista de erros com código/campo/mensagem/severidade
- Aviso explícito: "Validação local não representa autorização fiscal"

---

## Aba QR Code Preview

- Campo: Tipo (NFCE / NFE / SIFEN)
- Campo: Chave Preview (BR)
- Campo: CDC Preview (PY)
- Campo: Ambiente (fixo: HOMOLOGACAO, desabilitado)
- Botão: **Gerar QR Preview**
- Resultado: imagem SVG renderizada (via `data:image/svg+xml;base64,...`)
- Exibe conteúdo textual do QR
- Badge vermelho: QR TÉCNICO — SEM VALIDADE FISCAL — NÃO É QR OFICIAL
- Legenda na imagem: "QR Code técnico de homologação. Não possui validade fiscal."

---

## Exportação de Arquivos

- Nome obrigatório para XML: `preview_homologacao_sem_validade_nfce.xml`
- Nome obrigatório para JSON: `preview_homologacao_sem_validade_sifen.json`
- O conteúdo inclui o aviso de sem validade fiscal
- Exportação via JSRuntime está marcada como placeholder (não implementada neste bloco)

---

## O Que Esta UI NÃO Faz

- ❌ Não emite nota fiscal
- ❌ Não transmite para SEFAZ
- ❌ Não transmite para DNIT/SIFEN/SET
- ❌ Não autoriza documento
- ❌ Não altera venda, caixa, estoque, financeiro ou fiscal oficial
- ❌ Não gera DANFE, KuDE ou QR fiscal oficial
