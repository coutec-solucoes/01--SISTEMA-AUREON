# Fase 20 — Instalação, Licenciamento, Backup e Atualização

## Objetivo Geral
Transformar o Aureon de um sistema tecnicamente avançado em um produto instalável, licenciável, recuperável e operável em cliente real.

## Blocos Planejados (Iniciais)
1. **Bloco 1**: Arquitetura Base de Licenciamento Local, Identidade da Instalação e Estado Comercial da Empresa.
2. (Próximos blocos focarão em validação cloud, instalador, rotinas de backup, etc.)

## Escopo do Bloco 1
Foi construída a fundação "offline-first" do licenciamento.
O PDV agora consegue identificar-se via `installation_id` e consultar uma licença local contendo o modo (`DEV`, `MANUAL`) e o status.

### Tabelas Criadas
- `instalacao_local`: Dados de identidade do terminal e empresa.
- `licenca_local`: Dados da licença, com suporte a tolerância offline.
- `licenca_eventos`: Auditoria técnica de ativação e consultas.

### Commands Tauri Criados
- `obter_status_licenca`
- `ativar_licenca_dev`
- `registrar_evento_licenca`
- `obter_identidade_instalacao`

### Regras Offline-First
- Tolerância padrão de 10 dias.
- PDV consulta o banco local (SQLite) para tomar decisão comercial (`pode_operar`), garantindo fluidez mesmo sem internet.

### Limitações (Por Design)
- **Nenhum bloqueio automático em vendas ainda.**
- **Nenhum servidor cloud de licenças implementado.**
- Criptografia estrutural, assinatura real virá nas próximas etapas.

## Bloco 4 — Assinatura Criptografica de Licenca (Ed25519)
**Status**: IMPLEMENTADO
**Data**: 2026-05-25

### Objetivo
Criar camada de assinatura criptografica do payload de licenca, permitindo que o PDV valide autenticidade offline futuramente sem depender de internet permanente.

### Algoritmo
Ed25519 via crate ed25519-dalek v2. Assinatura de 64 bytes em base64.

### Modulo Criado
- services/aureon-api-local/src/licenca_crypto.rs
  - ChaveLicenca: gerencia chave privada/publica
  - assinar_payload(): gera assinatura Ed25519 real
  - verificar_payload(): verifica com chave publica
  - PayloadCanonicoLicenca: canonicalizacao deterministica

### Endpoints Adicionados
- GET /licenciamento/licencas/{id}/payload-assinado
- POST /licenciamento/licencas/verificar-payload
- GET /licenciamento/chaves/status

### DTOs Criados
- LicencaPayloadAssinadoResp
- VerificarLicencaPayloadReq
- VerificarLicencaPayloadResp
- StatusChavesResp

### Dependencias Adicionadas
- ed25519-dalek = { version = 2, features = [rand_core] }
- hex = 0.4

### Canonicalizacao
Campos em ordem fixa: empresa_id, licenca_id, plano_codigo, status, validade, terminal, tolerancia_offline_dias, emitido_em. Sem floats. SHA-256 e objeto assinado.

### Seguranca
- Chave privada NUNCA sai da Retaguarda
- Chave DEV efemera em modo sem env configurado (warning explicito)
- Chave de producao via AUREON_LICENSE_PRIVATE_KEY_B64

### Build
cargo check -p aureon-api-local: APROVADO (9 warnings pre-existentes, zero erros)

### Pendencias / Limitacoes
1. PDV ainda nao consome payload assinado (proximo bloco)
2. Chave publica ainda nao e distribuida ao PDV
3. Verificacao offline no PDV ainda nao implementada
4. Payload usa dados mock (aguarda banco real)

## Bloco 5 — Aplicação Local de Licença Assinada (PDV)
**Status**: APROVADO
**Data**: 2026-05-25

### Objetivo
Capacitar o PDV a receber, verificar usando criptografia Ed25519 offline e aplicar payloads de licença assinados.

### Entregas
- `licenca_crypto_local.rs`: Módulo de verificação Ed25519 no PDV.
- Integração de DTOs Rust compartilhados e classes C# (`PdvModels.cs`).
- UI `LicencaPdv.razor` atualizada com suporte a colagem de Payload e Assinatura com validação local.
- Build verificado com sucesso para `aureon-pdv` e `aureon-api-local`.

## Bloco 6 — Sincronização Manual/Online da Licença (PDV -> Retaguarda)
**Status**: CONCLUÍDO (Pronto para Homologação)
**Data**: 2026-05-25

### Objetivo
Permitir que o PDV sincronize de forma automática/manual consultando a URL da Retaguarda, enviando um check-in, obtendo o payload criptográfico assinado e aplicando localmente.

### Entregas
- Commands Tauri: `configurar_licenciamento_online`, `obter_config_licenciamento_online` e `sincronizar_licenca_online`.
- Tabela SQLite local `licenca_config` criada sob demanda para persistência das configurações de URL e chaves do licenciamento.
- Regra de tolerância a falhas de rede (Offline-First): se houver erro HTTP ou falha de conexão, a licença local nunca é apagada nem a operação bloqueada. É registrado o evento `LICENCA_SYNC_FALHOU` e exibida mensagem de contingência.
- UI do Blazor atualizada com campos de URL da retaguarda, Empresa ID e botões funcionais de "Salvar Configuração" e "Sincronizar com Retaguarda".

## Bloco 7 — Política de Bloqueio Suave e Alertas
**Status**: APROVADO
**Data**: 2026-05-25

### Objetivo
Construir a política operacional e alertas para a licença, calculando o nível atual baseado na tolerância offline e dias restantes, sem aplicar bloqueio duro de caixa/venda.

### Entregas
- Command `obter_politica_licenca` que calcula o nível (`OK`, `ALERTA_VENCIMENTO`, `TOLERANCIA_OFFLINE`, `EXPIRADA`, `BLOQUEADA`, `MODO_DEV`, `SEM_LICENCA`).
- UI `LicencaPdv.razor` atualizada com exibição clara do status, cores e recomendações operacionais.

## Bloco 8 — Guarda Operacional de Licença (Bloqueio Controlado)
**Status**: CONCLUÍDO
**Data**: 2026-05-25

### Objetivo
Implementar bloqueio operacional real para operações críticas com base na política de licença, de forma limitada, auditável e reversível. A regularização nunca pode ser travada.

### Entregas
- DTOs `VerificarOperacaoLicencaReq` e `VerificarOperacaoLicencaResp` adicionados em `aureon-core` e `PdvModels.cs`.
- Módulo e functions `avaliar_operacao_licenca` e `garantir_operacao_licenciada` criados no Tauri.
- Inserção da guarda em `commands_caixa` (`abrir_caixa`) e `commands_pagamento` (`finalizar_venda` e `registrar_pagamento`).
- Eventos de auditoria gravados: `LICENCA_OPERACAO_PERMITIDA`, `LICENCA_OPERACAO_BLOQUEADA`, `LICENCA_BLOQUEIO_SUAVE_APLICADO`.
- Nenhuma operação de leitura, backup ou sincronização foi afetada, garantindo total regularização.


