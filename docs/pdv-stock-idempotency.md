# Idempotência de Estoque

Para prevenir falhas de rede, dupla clicada no Blazor ou travamentos no Tauri, o core Rust protege a triagem de movimentações com um princípio clássico de idempotência transacional no SQLite.

## Idempotência na Baixa de Venda e Estorno
Quando uma venda é finalizada, a função `processar_baixa_venda(venda_id)` procura no Kardex por `SELECT id FROM estoque_movimentacoes WHERE origem_id = ? AND tipo_movimentacao = 'VENDA'`. 
Se a linha existir, o backend apenas engole silenciosamente retornando sucesso ao cliente chamador para prosseguir o ciclo visual, mas aborta instantaneamente qualquer soma residual matemática, evitando duplicação.
O mesmo ocorre para o estorno de vendas.

## Idempotência Manual (Ajuste/Inventário)
Para métodos que partem diretamente do UI como Ajuste e Inventário, foi acoplada a variável obrigatória `idempotency_key` no request. Caso o Client (Blazor) dispare dois request payloads idênticos em menos de 10 milissegundos por erro de renderização do DOM, o Rust checará a key informada para o tipo de movimentação de Ajuste bloqueando processamento paralelo duplo (a key é transformada em ID raiz para movimentações globais agrupadas em outbox, mas seu rastreio original mitiga a dupla entrada).
