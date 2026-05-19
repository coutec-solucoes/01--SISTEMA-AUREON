# Estrutura Fiscal Base do Produto

## 🇧🇷 Estrutura Fiscal Brasil (Base Cadastral)
O cadastro de produtos possui uma estrutura fundamental para atender a legislação tributária brasileira nas transações futuras de venda no caixa:
*   **NCM (Nomenclatura Comum do Mercosul) (`fiscal_ncm`)**: Campo alfanumérico que define a classificação do produto para apuração de impostos e barreiras tarifárias regionais.
*   **CEST (Código Especificador da Substituição Tributária) (`fiscal_cest`)**: Campo associado a produtos sujeitos a regimes de substituição tributária do ICMS.
*   **CFOP Padrão (Código Fiscal de Operações e Prestações) (`fiscal_cfop_padrao`)**: Indica a natureza da operação de saída padrão (ex: 5102 para mercadoria adquirida de terceiros).
*   **Origem da Mercadoria (`fiscal_origem_mercadoria`)**: Define se o produto é nacional ou importado.

---

## 🇵🇾 Estrutura Fiscal Paraguai (Base Cadastral)
Visando a expansão internacional e a compatibilidade do ecossistema Aureon em território paraguaio, a estrutura de banco de dados e as DTOs estão pré-configuradas para mapear as obrigações tributárias do Paraguai:
*   **IVA (Imposto sobre o Valor Agregado)**: Mapeado conceitualmente no banco para receber regras fiscais paraguaias (10% padrão, 5% reduzida ou isento).
*   **Campos de Contribuinte (RUC)**: O cadastro de pessoas suporta o RUC paraguaio diretamente nos validadores do backend Rust.

---

## 🛑 Ausência de Emissão Fiscal Real nesta Fase
> [!IMPORTANT]
> A Fase 4 estabelece **exclusivamente a infraestrutura cadastral**.
> Não há qualquer rotina de transmissão de cupons ou notas fiscais eletrônicas nesta etapa. A emissão real de notas fiscais (SATE ou SEFAZ) não está funcional e será abordada apenas em fases logísticas posteriores de faturamento.
