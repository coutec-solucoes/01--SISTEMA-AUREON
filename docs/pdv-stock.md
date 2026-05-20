# Regras de Estoque no PDV

Este documento descreve as diretrizes operacionais do módulo de Estoque embarcado no terminal local (offline-first).

## Arquitetura: Offline-First e Async
O estoque local é operado pela tabela `produtos_estoque_cache`, cujo papel é unicamente apresentar e interagir em tempo real no PDV. Sincronizações assíncronas acontecem via eventos como `ESTOQUE_MOVIMENTACAO_GERADA` depositados no `sync_outbox`.

## Tipagem (Quantidade em Escala 3)
Assim como os valores financeiros nunca trafegam em Float, as quantidades de estoque seguem a mesma regra:
- **NENHUM FLOAT/DOUBLE** em Rust ou no SQLite.
- A quantidade física é representada pelo campo `quantidade_escala3` (INTEGER no SQLite, i64 no Rust).
- Exemplo: `1,500 kg` = `1500`.
- Apenas a interface de usuário (Blazor) exibe como Decimal e se encarrega de multiplicar e dividir por `1000m` na ponte UI <=> API.

## Controle de Estoque
A tabela base do catálogo (`produtos_cache`) possui a flag `controla_estoque (1 ou 0)`. Produtos taxativos, gorjetas, serviços ou itens sem tangibilidade física têm este flag desligado.
- Nas integrações (Baixa, Inventário e Ajuste), as rotinas varrem as linhas das tabelas buscando e respeitando exclusivamente os itens que tenham esse indicador ligado.

## Baixa Negativa Permitida
- Um operador de PDV nunca pode ser impedido de faturar uma mercadoria se o sistema local entender que o estoque não é suficiente. 
- A rotina debita abertamente deixando saldos negativos, sem a presença de Constraints `CHECK(quantidade >= 0)` em banco de dados.

## Ciclo de Baixa de Venda
A baixa dos itens é efetuada no instante cirúrgico da **Finalização da Venda**.
- Em Vendas Rápidas: Ao processar o pagamento integral.
- Em Mesas e Comandas: A baixa de estoque obedece a mesma regra de não faturar previamente os consumos, e debita o total dos itens apenas quando a Mesa for liquidada por completo no processo de pagamento e conversão para Venda Finalizada.
