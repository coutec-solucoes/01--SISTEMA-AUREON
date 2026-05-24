# PDV Fiscal Preview (Espelho Técnico)

## Objetivo
O Espelho Técnico é um mecanismo que valida regras e dicionários fiscais em cima de uma `venda` já existente no banco de dados e salva seus totais em colunas chamadas `fiscal_*_preview` na `vendas` e `venda_itens`.

## Restrições de Operação
O processo de validar e calcular o espelho fiscal **nunca altera o valor de uma venda**, nem reflete nos recebíveis, caixa ou controle de estoque do PDV. A matemática roda exclusivamente em paralelo para gerar e simular as variáveis do imposto (como `fiscal_imposto_minor`).

## Fluxo
1. `validar_dados_cadastrais_fiscais`: Inspeciona CNPJ, Inscrição Estadual, presença de NCM e preenchimento das regras obrigatórias de acordo com o país.
2. `calcular_espelho_fiscal_venda`: Acumula bases e alíquotas (convertidas para escala 6).
3. `obter_espelho_fiscal_venda`: Devolve o resultado (Preview) para visualização na UI (EspelhoFiscalModal.razor).
