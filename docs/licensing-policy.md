# FASE 20 — BLOCO 7
## Política de Bloqueio Suave, Alertas e Regras Operacionais de Licença

Este bloco implementa a política operacional inteligente e offline-first de licenciamento do PDV Aureon. O PDV agora calcula localmente a saúde e elegibilidade operacional do terminal com base no banco de dados SQLite local, sem travar abertura de caixa ou vendas no MVP (bloqueio suave).

### Níveis da Política Operacional
O motor local de decisão calcula e expõe os seguintes estados:
1. **OK**: Licença ativa e perfeitamente regularizada com check-in dentro do limite.
2. **ALERTA_VENCIMENTO**: Faltando 7 dias ou menos para expirar.
3. **TOLERANCIA_OFFLINE**: Licença expirou mas ainda está operando sob a carência offline (10 dias padrão).
4. **EXPIRADA**: Fora do prazo de validade e estourou os dias de tolerância offline.
5. **BLOQUEADA**: Bloqueada manualmente por decisão administrativa da retaguarda.
6. **MODO_DEV**: Demonstração local ativa.
7. **SEM_LICENCA**: Nenhuma licença local configurada.

### Comportamento de Bloqueio Suave (MVP)
- Mesmo se a política operacional retornar `pode_operar = false` (nos estados `EXPIRADA` ou `BLOQUEADA`), o PDV não trava as vendas estruturais ou a abertura de caixa neste bloco.
- Isso preserva a resiliência máxima na implantação, limitando-se a apresentar avisos informativos claros e recomendações de ações sugeridas.
- Os eventos `LICENCA_EXPIRADA_DETECTADA`, `LICENCA_BLOQUEADA` e `LICENCA_TOLERANCIA_OFFLINE` são devidamente logados para auditoria local.
