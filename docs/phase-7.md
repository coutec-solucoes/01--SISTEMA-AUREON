# Documentação da Fase 7 — Núcleo do PDV Local

## Resumo da Fase
A Fase 7 marcou o início operacional do Terminal PDV nativo em Rust/Tauri com UI em Blazor WebAssembly. O foco foi estruturar o **Núcleo de Vendas**, garantindo blindagem financeira absoluta e preparando o terreno para funcionamento offline com posterior sincronização via Outbox.

## Blocos Implementados
- **Bloco 1**: Migration 003 e 004 do SQLite (Estruturação de Caixa, Vendas e Pagamentos).
- **Bloco 2**: Regras de Negócio de Sessão de Caixa (Abertura, Fechamento e Saldos).
- **Bloco 3**: Regras de Venda Rápida (Idempotência, Início de Venda, Adição e Cancelamento de Itens).
- **Bloco 4**: Regras de Pagamento Multimoeda (Rateio, Câmbio travado e Troco).
- **Bloco 5**: Refatoração Financeira Total (Substituição de `f64`/`REAL` por Matemática Inteira de Precisão `minor unit`).
- **Bloco 6**: Construção da Interface Gráfica (Blazor UI) para Caixa e Vendas, consumindo os commands Rust de forma responsiva.
- **Bloco 7**: Auditoria, Validação de Fluxos e Documentação.

## Migrations SQLite Criadas
- `003_venda_nucleo.sql` (Descontinuada/Saturada por falha arquitetural do float).
- `004_venda_nucleo_correcao_financeira.sql`: Migration corretiva e base oficial. Estabeleceu `INTEGER` para todo campo financeiro, implementou `quantidade_escala3` e isolou saldo de caixa por moeda (`sessoes_caixa_moedas`).

## Commands Tauri (Rust) Criados
- **Caixa:** `abrir_caixa`, `fechar_caixa`, `obter_sessao_ativa`, `listar_sessoes`
- **Venda:** `iniciar_venda`, `buscar_produto_pdv`, `adicionar_item_venda`, `cancelar_item_venda`, `cancelar_venda`, `obter_venda`
- **Pagamento:** `registrar_pagamento`, `finalizar_venda`, `calcular_troco`, `listar_pagamentos_venda`

## Telas Blazor Criadas
- `Pages/Caixa.razor`: Fluxo de turno (Abertura, Fechamento Físico e Histórico).
- `Pages/Pdv.razor`: Fluxo de operação (Carrinho, Busca por Teclado, Pagamento Modal e Finalização).
- Mapeamento DTO estrito sem floats (`AureonPdvUi.Services.PdvModels`).

## Regras de Negócio Principais
1. **Fim do Float:** Dinheiro não usa ponto flutuante. `10,50 BRL` é gravado e trafegado como `1050`. Quantidades (kg, litros) usam `escala3` (`1.5` vira `1500`).
2. **Número de Venda Protegido:** Nenhuma venda em andamento consome a numeração oficial. O `numero_venda` só é incrementado no momento do pagamento completo (na função `finalizar_venda`) em transação atômica.
3. **Multimoeda Nativo:** Abertura e fechamento de caixa, assim como pagamentos, discriminam o saldo por tipo de moeda (BRL, USD, PYG). A cotação é travada em snapshot de escala (`TaxaCambioEscala6`).

## Validações Executadas
Todos os fluxos foram testados e validados contra os comandos compilados. O build local (`dotnet build`) e a verificação do Rust (`cargo check -p aureon-pdv`) finalizam o CI/CD local sem erros ou warnings de segurança.

## Limitações Conhecidas
- Sugestão de conversão física reversa de troco: estruturalmente pronta (`moeda_troco_codigo`), mas a UI atualmente exibe apenas o troco geral em BRL e o operador precisa converter de cabeça se for devolver em Guaranis, por exemplo. Isso será refinado no futuro.
- A Migration 004 usou `DROP TABLE` de forma destrutiva por estarmos em ambiente de homologação e a 003 não conter dados reais. Futuras alterações no DB do PDV deverão ser incrementais.

## Fora do Escopo desta Fase
Não foram implementados nesta fase (pertencem a fases futuras):
- Motor Fiscal Real (NFC-e, SAT, etc.) e Impressão Térmica Direta.
- Baixa real de Estoque Local ou Kardex.
- Gateway TEF ou integração direta PIX (pagamentos são apenas registros físicos declarados pelo operador).
- Fluxo de Delivery ou Comandas/Mesas.
