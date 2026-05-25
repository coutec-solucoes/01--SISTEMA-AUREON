# Validação Cloud e Check-in da Licença (Fase 20 - Bloco 3)

Este documento detalha o fluxo de validação (check-in) entre o terminal PDV e a Retaguarda Mestre, garantindo a verificação comercial sem causar bloqueios abruptos offline.

## Fluxo de Check-in

1. O PDV (via seu background job local ou inicialização) faz uma chamada para `POST /licenciamento/check-in` na Retaguarda.
2. O PDV envia um payload contendo seu identificador da instalação (`installation_id`), ID e hash do dispositivo, e a chave da empresa.
3. A Retaguarda valida:
   - Se a empresa existe e está com status permitido.
   - Se existe uma licença ativa para essa empresa.
   - Se o plano está vigente.
   - Se o terminal já está autorizado ou se há limite para autorizar um novo automaticamente.
4. A Retaguarda retorna o objeto `LicencaPayloadResp`.

## Payload da Licença

O payload recebido da nuvem contém a flag `pode_operar`, que instrui o PDV sobre o status geral comercial. Além disso, retorna as seguintes chaves no objeto JSON `payload_licenca_json`:
- `empresa_id`
- `licenca_id`
- `plano_codigo`
- `permissoes` (ex: `["PDV", "FISCAL"]`)
- `validade`
- `terminal`
- `tolerancia_offline_dias`
- `emitido_em`

Esse payload deve ser feito cache pelo PDV.

## Regras do Terminal

- Se o PDV não enviar um `terminal_id` válido, a nuvem tenta vinculá-lo.
- Se o plano da empresa não permitir mais terminais (excedeu `max_terminais`), o retorno trará status pendente ou bloqueado, e `pode_operar = false`.
- Terminais explicitamente BLOQUEADOS perdem o acesso à licença e devem suspender as operações.

## Assinatura Futura

Atualmente, `assinatura_licenca` retorna `"ASSINATURA_FUTURA_NAO_IMPLEMENTADA"`.
No próximo bloco, será implementada a assinatura criptográfica (ex: HMAC SHA-256 ou Ed25519) utilizando a chave mestra da Retaguarda, garantindo que o PDV possa confiar matematicamente no cache offline sem exigir internet.

## O que ainda não está implementado (Escopo bloqueado nesta etapa)
- Bloqueio automático de caixa e de venda no PDV se a licença falhar.
- Cobrança recorrente (gateways de pagamento).
- Assinatura criptográfica real.
- Aplicação do payload no banco local (SQLite) do PDV.
