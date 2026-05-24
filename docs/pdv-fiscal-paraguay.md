# PDV Fiscal - Paraguai

## Suporte Estrutural
Para operações parametrizadas no Paraguai (PY), o PDV implementou um foco forte no IVA (Imposto de Valor Agregado) e Timbrados/Séries de Controle.

## IVA
- `fiscal_iva_cache` armazena códigos IVA (ex: Exento, 5%, 10%) que são aplicados como regras em cima da "Base Imponível".

## Emissão Fiscal Inexistente
Nesta fase, não geramos transações para a SET. Não construímos arquivos assinados para o SIFEN (DTE). O KuDE e seus códigos de consulta (CDC/QR) não são gerados pelo software durante a impressão de cupons ou encerramento de caixa.
