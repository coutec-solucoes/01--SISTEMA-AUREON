# Fase 16 - Fiscal Base e Espelho Técnico Sem Emissão

## Resumo da Fase
A Fase 16 foca na criação da base de dados estrutural para parâmetros fiscais, tanto para o Brasil (NCM, CFOP, CST) quanto para o Paraguai (IVA, Timbrado). O objetivo principal foi implementar o "Espelho Fiscal Técnico", um simulador/preview de impostos que não tem validade jurídica, não emite notas, e não transmite arquivos para o Fisco. 

## Blocos Implementados
1. **Bloco 1**: Migration SQLite/PostgreSQL de Parâmetros Fiscais Base.
2. **Bloco 2**: Commands Rust/Tauri de Leitura e Manutenção de Parâmetros Fiscais Estruturais.
3. **Bloco 3**: Integração Fiscal Estrutural com Produtos, Empresa, Cliente e Venda — Espelho Fiscal Sem Emissão.
4. **Bloco 4**: UI Blazor de Configurações Fiscais e Espelho Fiscal Técnico.
5. **Bloco 5**: Pré-visualização Fiscal Não Autorizada / Espelho Técnico Não Fiscal no PDV.

## Commits Principais
- `b4ee0cb` - Migration Fiscal Base.
- `a5d1b1b` - Commands Fiscais Rust.
- `2ed7449` - Espelho Fiscal e Regras Tributárias Rust.
- `5fa0e30` - UI Blazor de Configurações Fiscais.
- `10b51f9` - Pré-visualização Fiscal Não Autorizada.

## Migration Criada
- `database/migrations/sqlite/012_fase16_fiscal_base.sql`

## Tabelas Criadas
- `fiscal_empresa_cache`
- `fiscal_numeracao_cache`
- `fiscal_ncm_cache`
- `fiscal_cfop_cache`
- `fiscal_cst_csosn_cache`
- `fiscal_iva_cache`
- `fiscal_regras_tributarias_cache`
- `fiscal_eventos_logs`

## Campos Adicionados
- **`produtos_cache`**: `fiscal_ncm_id`, `fiscal_iva_id`, `fiscal_cst_csosn_id`, `fiscal_cfop_padrao_id`, `fiscal_origem_mercadoria`.
- **`vendas`**: `fiscal_ambiente`, `fiscal_forma_emissao`, `fiscal_modelo_preview`, `fiscal_status_preview`, `fiscal_total_base_minor`, `fiscal_total_imposto_minor`.
- **`venda_itens`**: `fiscal_cfop_aplicado`, `fiscal_cst_csosn_aplicado`, `fiscal_aliquota_escala6`, `fiscal_base_minor`, `fiscal_imposto_minor`.

## Commands Criados
- `obter_configuracao_fiscal_empresa`
- `salvar_configuracao_fiscal_empresa`
- `listar_fiscal_ncm`, `listar_fiscal_cfop`, `listar_fiscal_cst_csosn`, `listar_fiscal_iva`, `listar_fiscal_regras_tributarias`
- `obter_regra_tributaria`, `salvar_fiscal_iva`, `salvar_regra_tributaria`
- `vincular_fiscal_produto`
- `listar_fiscal_eventos_logs`
- `validar_dados_cadastrais_fiscais`
- `calcular_espelho_fiscal_venda`
- `obter_espelho_fiscal_venda`
- `limpar_espelho_fiscal_venda`

## Telas Blazor Criadas
- `FiscalConfiguracoes.razor`
- `FiscalDicionarios.razor`
- `FiscalRegrasTributarias.razor`
- `VincularFiscalProdutoModal.razor`
- `EspelhoFiscalModal.razor`

## Estratégia Brasil
Suporte estrutural a NCM, CFOP, CST/CSOSN, Origem da Mercadoria (0 a 8) e Regime Fiscal. 

## Estratégia Paraguai
Suporte estrutural a IVA (e.g. 5%, 10%), país PY, controle de Timbrado e Série.

## Escala 6 para Alíquotas
Para evitar problemas de arredondamento em float, as alíquotas são armazenadas utilizando `INTEGER` (minor unit escala 6). Por exemplo, `10.5%` torna-se `105000`.

## Espelho Fiscal Técnico
Um mecanismo de visualização técnica dos totais de impostos baseados nos itens vendidos e regras parametrizadas. Ele **não** altera o estoque, financeiro ou valor da venda original.

## Limitações Conhecidas
- O sistema não transmite e não emite documentos com validade jurídica.
- Sincronização dos dicionários massivos (NCM, CFOP) via Retaguarda ficou adiada para fase específica de `sync`.
- O Espelho Fiscal foi acoplado apenas na "Central de Reimpressões" e necessita futuramente de acesso rápido direto pela tela limpa do PDV.

## O que Ficou Fora do Escopo
- Emissão real de NF-e, NFC-e, NFS-e, SAT, ou SIFEN.
- Geração de arquivos XML assinados, DANFE, KuDE, e QR Code Fiscal.
- Autenticação via Certificado Digital.
