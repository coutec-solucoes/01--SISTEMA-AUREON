# Guarda Operacional de Licença

Este documento detalha o funcionamento do **Bloqueio Suave e Guarda Operacional** de licenciamento offline implementados no PDV (Fase 20, Bloco 8).

## Política vs. Bloqueio Real

- **Política Operacional**: Apenas indica o nível de risco e a saúde da licença (OK, EXPIRADA, BLOQUEADA, MODO_DEV, etc.).
- **Bloqueio Real**: O mecanismo (guarda) que intercepta operações cruciais e impede sua execução caso a política exija.

O princípio fundamental é que o bloqueio deve ser **auditável, limitado e reversível**. O operador sempre deve ter meios de regularizar o terminal através da tela de Licença.

## Operações Controladas (Bloqueáveis)

Atualmente, as seguintes operações são interceptadas e impedidas quando a licença está bloqueada:
- **`ABRIR_CAIXA`**: O operador não consegue abrir o caixa diário.
- **`FINALIZAR_VENDA` / `FECHAR_VENDA_PAGAMENTO`**: As vendas não podem ser concluídas nem os pagamentos finalizados.

## Operações NUNCA Bloqueadas (Sempre Permitidas)

As seguintes ações **nunca** serão travadas pela licença, garantindo a transparência e possibilidade de regularização:
- Tela e configurações de `/licenca`;
- Consultas, diagnósticos e relatórios (apenas leitura);
- Sincronização online da licença;
- Importação/aplicação de licença offline (payload assinado);
- Backup dos dados do PDV;
- O uso de MODO_DEV com a respectiva chave de teste.

## Níveis de Licença

### Níveis que BLOQUEIAM a Operação
A operação será rejeitada retornando um erro amigável se a licença estiver nos seguintes níveis:
1. **`BLOQUEADA`**: Quando há um bloqueio total ativo vindo da Retaguarda.
2. **`EXPIRADA`**: Quando a validade terminou e também o período de tolerância offline acabou.

> [!WARNING]
> Mensagem padrão de bloqueio: "Operação bloqueada pela política de licença local. Acesse Sistema > Licença para sincronizar ou regularizar."

### Níveis que NÃO Bloqueiam a Operação (Mas podem Alertar)
- **`OK`**: Operação padrão.
- **`ALERTA_VENCIMENTO`**: Emissão normal, com log de warning.
- **`TOLERANCIA_OFFLINE`**: Operação permitida durante os X dias configurados de tolerância.
- **`MODO_DEV`**: Operação normal com alertas de desenvolvimento.
- **`SEM_LICENCA`**: Operação momentaneamente permitida até ser endurecida na próxima etapa.

## Auditoria e Rastreabilidade

Todas as decisões do Guarda de Licença são registradas localmente em `licenca_eventos` sob o código da licença.
- **`LICENCA_OPERACAO_PERMITIDA`**: Acesso concedido e gravado por auditoria.
- **`LICENCA_OPERACAO_BLOQUEADA`**: Bloqueio ativo. O JSON contém o motivo (ex: BLOQUEADA, EXPIRADA).
- **`LICENCA_BLOQUEIO_SUAVE_APLICADO`**: A operação foi permitida mas gerou um alerta (ex: Tolerância offline, Perto do vencimento).

## Como Regularizar
1. Navegue para `Sistema` -> `Licença`.
2. Verifique o status.
3. Se conectado, clique em **Sincronizar Online** ou garanta o Check-In.
4. Se offline, solicite o payload assinado da retaguarda e importe-o localmente no PDV.

## Próximos Passos e Riscos
- O ponto exato de criação de venda (`criar_venda`) permanece a avaliar caso exista múltiplos fluxos (gourmet, balcão) para evitar bloqueio parcial inseguro.
- Nenhuma cobrança ou gateway de pagamento é executada no PDV, minimizando a superfície de falha.
