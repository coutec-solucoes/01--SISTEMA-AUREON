# Publicação de Dados Mestres

A publicação de dados mestres da Retaguarda para os PDVs é efetuada de forma empacotada e versionada, garantindo consistência total nas frentes de caixa.

## Fluxo de Publicação e Empacotamento

1. **Trigger de Publicação**:
   - Quando dados cadastrais são alterados na retaguarda (ex: produtos, preços, configurações), o administrador dispara o comando de publicação via API ou interface Blazor.
   - Isso incrementa as versões na tabela `sync_versoes_dados` da retaguarda.

2. **Geração do Pacote (API)**:
   - A rota `POST /sync/primeira-sincronizacao` é invocada pelo PDV para obter os pacotes completos.
   - A API busca os dados das tabelas reais no PostgreSQL (`empresas`, `moedas`, `usuarios`, `produtos`, `produtos_fiscal`, `adicionais`, `configuracoes_pdv`, `regras_venda`, `perifericos`).
   - Ela gera um payload JSON consolidado e insere na tabela `pacotes_sincronizacao` e `pacotes_sincronizacao_itens` com as respectivas versões aplicadas.
   - A API assina o pacote com um hash geral para garantia de integridade.
