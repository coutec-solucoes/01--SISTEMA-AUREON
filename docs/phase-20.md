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
