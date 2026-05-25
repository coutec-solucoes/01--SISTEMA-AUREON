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
