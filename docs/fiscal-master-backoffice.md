# Retaguarda Fiscal Mestre — Backoffice PostgreSQL

## Conceito

A **Retaguarda** é a única fonte de verdade (single source of truth) de todos os dados fiscais estruturais do Sistema Aureon.

Nenhum PDV cria ou edita regras fiscais. Os PDVs apenas recebem e aplicam pacotes publicados pela Retaguarda.

---

## Dicionários Fiscais

### NCM — Nomenclatura Comum do Mercosul
- Tabela: `fiscal_dicionario_ncm`
- Campos: `codigo` (ex: `0101.21.00`), `descricao`, `ativo`
- Operações permitidas: criar, editar, inativar (nunca excluir fisicamente)

### CFOP — Código Fiscal de Operações e Prestações
- Tabela: `fiscal_dicionario_cfop`
- Campos: `codigo`, `descricao`, `tipo_operacao` (ENTRADA/SAIDA), `ativo`

### CST/CSOSN — Código de Situação Tributária
- Tabela: `fiscal_dicionario_cst_csosn`
- Campos: `codigo`, `descricao`, `tipo` (CST/CSOSN), `ativo`

### IVA — Impuesto al Valor Agregado (Paraguai)
- Tabela: `fiscal_dicionario_iva`
- Campos: `codigo`, `descricao`, `pais_fiscal` = `PY`, `aliquota_escala6` (INTEGER escala 6)
- Exemplo: 10% = `100000000` (10 × 10⁷? — verificar escala), IVA 5% = `50000`

> **Regra matemática:** `aliquota_escala6 = aliquota_percentual × 1.000.000`
> Ex: 10% → `10000000 / 10000000 * 100 = 10%`

---

## Regras Tributárias

- Tabela: `fiscal_regras_tributarias_mestre`
- Campos: `pais_fiscal`, `tipo_operacao`, `uf_origem`, `uf_destino`, `ncm_id`, `cfop_id`, `cst_csosn_id`, `iva_id`
- Alíquotas: `aliquota_icms_escala6`, `aliquota_pis_escala6`, `aliquota_cofins_escala6`, `aliquota_iva_escala6`, `reducao_base_escala6`
- `prioridade`: inteiro ≥ 0 — regras mais específicas têm prioridade maior
- Suporte a `empresa_id` e `filial_id` opcionais para parametrização por filial

### Conversão Visual → Escala 6
| Percentual Visual | Valor Escala 6 |
|-------------------|----------------|
| 18,00% (ICMS) | 180.000 |
| 10,00% (IVA PY) | 100.000 |
| 5,00% (IVA PY red.) | 50.000 |
| 1,65% (PIS) | 16.500 |
| 7,60% (COFINS) | 76.000 |

---

## Configuração Fiscal da Empresa

- Tabela: `fiscal_empresas_config`
- Campos: `pais_fiscal` (BR/PY), `regime_fiscal`, `ambiente` (HOMOLOGACAO/PRODUCAO), `forma_emissao` (NORMAL/CONTINGENCIA_OFFLINE), `certificado_alias`
- O campo `certificado_alias` é apenas referência estrutural — **nenhum certificado é carregado ou operado na Fase 17**

---

## Numeração Mestre

- Tabela: `fiscal_numeracao_mestre`
- Campos: `tipo_documento`, `serie`, `proximo_numero`, `empresa_id`, `filial_id`, `pais_fiscal`
- Base para futura emissão sequencial de documentos fiscais (NF-e, NFC-e, KuDE)

---

## Versionamento e Publicação

- Tabela: `fiscal_versoes_publicacao`
- Status possíveis: `RASCUNHO` → `PUBLICADA` → `REPROCESSADA` | `CANCELADA`
- Itens por versão: `fiscal_versoes_publicacao_itens` (snapshot de cada entidade)

---

## Auditoria

- Tabela: `fiscal_auditoria_mestre`
- Registra: entidade, entidade_id, ação, usuário_id, detalhes, criado_em
- Imutável — não permite edição ou exclusão de logs de auditoria
- Ações registradas: `PUBLICAR_VERSAO_FISCAL`, `REPROCESSAR_VERSAO_FISCAL`, `GERAR_PAYLOAD_FISCAL`, `CRIAR_NCM`, `INATIVAR_NCM`, etc.
