# UI da Retaguarda Fiscal (Backoffice Blazor)

## Visão Geral

A Retaguarda possui 5 telas dedicadas ao módulo fiscal, todas acessíveis pelo menu lateral sob a seção **"Fiscal"**.

> ⚠️ **Aviso em todas as telas:** "Retaguarda Fiscal Mestre — publicação de regras, não emissão fiscal."

---

## FiscalAdmin (`/fiscal`)

Hub central com cards de navegação para:
- Configurações Fiscais
- Dicionários Fiscais
- Regras Tributárias
- Publicações e Versões
- Auditoria Fiscal *(em breve)*

---

## DicionariosFiscaisAdmin (`/fiscal/dicionarios`)

Interface em **abas** para gerenciar:

| Aba | Tabela Mestre | Campos Chave |
|-----|--------------|-------------|
| NCM | `fiscal_dicionario_ncm` | código, descrição |
| CFOP | `fiscal_dicionario_cfop` | código, descrição, tipo_operação |
| CST/CSOSN | `fiscal_dicionario_cst_csosn` | código, tipo (CST/CSOSN), descrição |
| IVA | `fiscal_dicionario_iva` | código, alíquota (visual %) |

**Regras visuais:**
- Registro inativo exibe badge vermelho `Não`
- Botão "Inativar" desabilitado para registros já inativos
- Proibida exclusão física
- Aviso: "Dicionários oficiais devem ser importados por rotina controlada. Esta tela não substitui validação fiscal."

---

## RegrasTributariasAdmin (`/fiscal/regras`)

Tabela com todas as regras mestre.

**Conversão percentual → Escala 6:**
- Usuário digita `18.00` (decimal visual)
- Sistema converte: `(18.00 × 10000) = 180000` (long)
- Enviado para a API como `aliquota_icms_escala6: 180000`

**Campos do modal de criação:**
- País Fiscal (BR/PY)
- Tipo Operação* (ENTRADA/SAIDA)
- ICMS %, PIS %, COFINS %, IVA % *(decimais visuais)*
- Prioridade (≥ 0)

---

## ConfiguracoesFiscaisAdmin (`/fiscal/configuracoes`)

Formulário de configuração estrutural:

| Campo | Valores Aceitos |
|-------|----------------|
| País Fiscal | BR / PY |
| Ambiente | HOMOLOGACAO / PRODUCAO |
| Forma de Emissão | NORMAL / CONTINGENCIA_OFFLINE |
| Regime Fiscal | SIMPLES_NACIONAL / LUCRO_PRESUMIDO / LUCRO_REAL |
| Certificado Alias | Texto livre (referência futura) |

> ⚠️ O certificado **não é carregado nem operado** nesta fase. Campo apenas estrutural.

---

## PublicacaoFiscalAdmin (`/fiscal/publicacao`)

Controle completo do ciclo de vida de uma versão fiscal:

| Ação | Quando disponível | Endpoint |
|------|------------------|---------|
| Criar Rascunho | Sempre | `POST /fiscal/versoes/rascunho` |
| Publicar Versão | Status = RASCUNHO | `POST /fiscal/versoes/{id}/publicar` |
| Cancelar | Status = RASCUNHO | `PUT /fiscal/versoes/{id}/cancelar` |
| Visualizar Payload | Status = PUBLICADA ou REPROCESSADA | `GET /fiscal/versoes/{id}/payload` |
| Reprocessar Pacote | Status = PUBLICADA ou REPROCESSADA | `POST /fiscal/versoes/{id}/reprocessar` |

**Aviso obrigatório na tela:**
> "Publicar versão fiscal apenas sincroniza dicionários/regras para PDVs. Não emite nota fiscal."

---

## Terminologia Obrigatória

| ✅ Usar | ❌ Proibido |
|---------|-----------|
| Publicar Versão | Emitir NF-e |
| Reprocessar Pacote | Autorizar |
| Visualizar Payload | Transmitir SEFAZ |
| Dicionário Fiscal | Gerar DANFE |
| Regra Tributária | Gerar KuDE |
| Configuração Mestre | Gerar QR Fiscal |
| Sincronização Fiscal | Emitir NFC-e |

---

## Serviços C#

| Arquivo | Propósito |
|---------|----------|
| `Services/FiscalModels.cs` | DTOs tipados para todos os objetos fiscais |
| `Services/FiscalApiClient.cs` | Cliente HTTP para os 23 endpoints fiscais |

O `FiscalApiClient` é injetado via DI no `Program.cs`:
```csharp
builder.Services.AddScoped<FiscalApiClient>();
```
