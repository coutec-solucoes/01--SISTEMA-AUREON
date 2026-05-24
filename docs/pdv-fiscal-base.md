# PDV Fiscal Base

## Arquitetura Estrutural
O PDV utiliza um modelo cache local em SQLite (`fiscal_*_cache`) para armazenar os parâmetros fiscais enviados pelo Retaguarda. Estas tabelas alimentam as listagens e vínculos na camada de vendas, mas não possuem integração com a API da SEFAZ ou do SIFEN.

## Tabelas de Dicionário
- `fiscal_ncm_cache`: Armazena a Nomenclatura Comum do Mercosul.
- `fiscal_cfop_cache`: Armazena o Código Fiscal de Operações e Prestações.
- `fiscal_cst_csosn_cache`: Armazena os Códigos de Situação Tributária e CSOSN.
- `fiscal_iva_cache`: Armazena as taxas e regras de Imposto ao Valor Agregado para o PY.

## Log de Auditoria Técnica
Todas as validações e interações com o motor fiscal estrutural geram entradas na tabela `fiscal_eventos_logs` (com metadados em JSON). Estes logs são utilizados para rastrear erros de cálculo técnico e validação cadastral.
