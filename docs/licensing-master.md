# Licenciamento Mestre (Retaguarda)

A Retaguarda do Aureon atua como o servidor Mestre de Licenças para todos os terminais PDV ativados.
No Bloco 2 da Fase 20, foi construída a base estrutural para este controle.

## Entidades Principais

### Planos (`lic_planos`)
Define limites comerciais (número de empresas, número de terminais) e permissões de acesso aos módulos (PDV, Retaguarda, Fiscal, etc.). Um plano possui um código único e nunca deve ser apagado fisicamente.

### Empresas Licenciadas (`lic_empresas`)
Representa o cliente pagante. O status da empresa dita se as licenças vinculadas a ela podem operar (ex: SUSPENSA, BLOQUEADA).

### Licenças Emitidas (`lic_licencas`)
Unidade comercial ativada para uma empresa baseada num plano.
Gera regras sobre:
- Validade (`validade_inicio`, `validade_fim`)
- Modo de operação (`MANUAL`, `DEV`, `ONLINE_FUTURO`)
- Tolerância offline permitida no terminal

### Terminais Autorizados (`lic_terminais`)
Controle físico/lógico dos PDVs. Cada PDV recebe uma licença.
O número de terminais ativos não pode ultrapassar `max_terminais` do plano.
Terminal bloqueado perde o direito de registrar vendas e fechar caixa (quando o sync for implementado).

### Auditoria e Eventos (`lic_eventos`)
Rastreabilidade obrigatória para criação de planos, autorizações e ações restritivas (bloqueio/suspensão). Não deve armazenar segredos, apenas contexto (JSONB).

## Futuro (Próximos Blocos)
- A infraestrutura mestre será sincronizada com os PDVs via conexão segura.
- A coluna `assinatura_licenca` passará a conter a assinatura criptográfica real (AES/RSA) do payload JSON.
- A retaguarda passará a se comunicar com gateways de pagamento se aplicável (fora do escopo deste bloco).
