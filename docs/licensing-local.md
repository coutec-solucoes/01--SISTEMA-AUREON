# Licenciamento Local (Offline-First)

A arquitetura do Aureon exige que o PDV não dependa exclusivamente de conexão constante com a internet para aprovar vendas.

## Conceitos

### Installation ID
Identificador global único gerado assim que o PDV é ativado. Garante que cópias de banco de dados não clonem a licença automaticamente.

### Terminal ID
Identificador lógico para o caixa. Útil para gestão no backend web futuro.

### Tolerância Offline
Tempo (ex: 10 dias) que o PDV pode operar normalmente após a última verificação cloud bem-sucedida.

### Modo DEV
Permite rodar o sistema e ignorar bloqueios comerciais temporariamente para desenvolvedores e testes.

### Status e Bloqueios
- `ATIVA`: Operação liberada.
- `EXPIRADA`: Prazo comercial expirou.
- `BLOQUEADA`: Ordem explícita de travamento do PDV.

## Riscos Atuais
No Bloco 1 (Fase 20), o licenciamento é puramente estrutural e ativável localmente via command Tauri. **Não existe** um servidor online verificando e não há criptografia forte bloqueando manipulação do banco local. Estes controles serão endurecidos posteriormente.
