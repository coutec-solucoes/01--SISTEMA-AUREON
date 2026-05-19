# Idempotência e Confiabilidade de Sincronização

A sincronização de faturamento e dados cadastrais em sistemas distribuídos está sujeita a instabilidades de rede, o que pode causar duplicações ou perdas de dados. No Aureon, aplicamos políticas rígidas de idempotência.

## Idempotência na Retaguarda (API)

- **Chave de Idempotência Obrigatória**: A rota de sincronização requer um parâmetro `idempotency_key` no cabeçalho ou payload de requisição do PDV.
- **Tabela de Controle**: A tabela `sync_idempotencia` no PostgreSQL grava o resultado da transação indexado pela chave.
- **Política de Reenvio**: Se o terminal enviar uma requisição com a mesma chave (ex: porque caiu a internet antes de receber a resposta da API), a retaguarda intercepta a chamada, busca o payload correspondente gravado anteriormente e devolve sem reprocessar regras ou gerar novos pacotes duplicados.

## Idempotência no PDV (Local)

- **Versionamento no SQLite**: O PDV mantém uma tabela `sync_idempotencia_local` para gravar os pacotes já aplicados localmente com sucesso.
- **Prevenção de Reaplicação**: Se o pacote recebido tiver uma versão geral que já foi processada localmente, o PDV pula a execução e sinaliza sucesso imediatamente, impedindo a sobregravação desnecessária de dados locais.
