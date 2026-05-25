# FASE 20 — BLOCO 6
## Sincronização Manual/Online da Licença: Retaguarda → PDV

Este bloco implementa a capacidade do PDV realizar a sincronização de sua licença consultando ativamente a retaguarda, validando a assinatura recebida e atualizando o banco de dados SQLite local.

### Fluxo de Sincronização
1. **Identificação**: O PDV lê sua `installation_id` e dados de identificação da tabela `instalacao_local`.
2. **Check-in**: Envia uma requisição HTTP `POST /licenciamento/check-in` contendo as informações da máquina e da empresa para a URL da Retaguarda configurada.
3. **Validação Criptográfica**: Ao receber o payload assinado da Retaguarda, o PDV valida a assinatura Ed25519 localmente utilizando a chave pública cadastrada.
4. **Aplicação SQLite**: Se a assinatura for válida, as informações de licenciamento são salvas na tabela local `licenca_local` e o evento é logado na tabela `licenca_eventos`.

### Comportamento Offline-First (Resiliência de Rede)
- O PDV Aureon foi desenhado para operar offline. Se a sincronização falhar por problemas de rede, o status local da licença **não é afetado nem apagado**.
- Uma mensagem amigável é exibida ao usuário, registrando o evento `LICENCA_SYNC_FALHOU` internamente sem interromper as operações do caixa ou do PDV.
- Não há qualquer gateway de pagamento integrado e nenhuma ação de bloqueio automático de venda/caixa.
