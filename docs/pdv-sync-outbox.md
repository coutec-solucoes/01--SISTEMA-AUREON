# Sync Outbox no PDV (Sincronização Local)

No ambiente local do PDV, a sincronização é **Event-Driven e Assíncrona**.

## Filosofia
Toda alteração de estado sensível no banco de dados SQLite grava simultaneamente (na mesma Transação SQL local) um JSON de auditoria/evento na tabela `sync_outbox`.
Isso resolve dois problemas clássicos em arquiteturas offline:
1. **Perda de conexão**: O terminal pode passar dias isolado sem atrasar operações locais. O evento está seguro.
2. **Duplo envio**: Ao retornar à rede, o Worker secundário do Rust apenas varre essa tabela ordenadamente, sem impactar o Frontend Blazor.

## Eventos Implementados
No estado atual (Fase 7), os seguintes eventos são registrados internamente via função `outbox_inserir`:
- `CAIXA_ABERTO`
- `CAIXA_FECHADO`
- `VENDA_INICIADA`
- `VENDA_FINALIZADA`
- `VENDA_CANCELADA`
- `ITEM_CANCELADO`
- `PAGAMENTO_REGISTRADO`

*(O Worker de envio massivo que processa essa tabela para a Cloud é escopo de fases posteriores).*
