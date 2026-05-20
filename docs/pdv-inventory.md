# Lançamento de Inventário

O Inventário no AUREON PDV permite corrigir desvios sistêmicos massivos.

## Técnica por Cálculo de Delta (Diferença)
Diferente da baixa contábil que movimenta quantitativos pontuais transacionados, a operação de Inventário permite que o operador informe pontualmente **qual é o saldo real visto nas prateleiras físicas**, e então deixa a cargo da API fazer as conciliações e criar o Delta positivo ou negativo equivalente que iguale o sistema àquela contagem real.
- **Delta Positivo:** Adiciona a quantidade de produtos calculada e gera Kardex para equilibrar o inventário a favor (Ex: Sistema 10, Físico 15 -> Movimentação de 5 Entrada).
- **Delta Negativo:** Diminui o excesso gerando abatimento Kardex (Ex: Sistema 10, Físico 8 -> Movimentação de 2 Saída).
- **Delta Neutro (0):** Se o Físico conferido bater com o saldo Sistema, nenhuma movimentação será gerada, otimizando processamento e armazenamento (Não suja o Kardex).

## Estratégia Futura (Sessão de Contagem)
No momento, a listagem UI puxa todos os produtos. Na retaguarda ou PDV com alto volume (acima de 10k itens), o inventário usará "Sessões de Contagem" em andamento, escaneadas pontualmente via coletor biper. Atualmente a listagem já limpa e não processa os dados `null` / brancos informados no modal.
