# Guarda de Permissões Operacionais

O sistema PDV implementa um controle de acesso baseado em RBAC na camada `Tauri`. 

## Ordem de Verificação

A verificação para iniciar ou concluir ações críticas dentro do PDV obedece a uma hierarquia rigorosa que **coexiste** entre licenciamento e permissões do operador.
1. **Guarda de Licença:** O PDV verifica se a instalação tem uma licença válida e se a operação não está bloqueada comercialmente.
2. **Guarda de Permissão:** O PDV verifica se há uma sessão de usuário ativa e se o usuário detém a permissão específica requerida.
3. **Execução:** Ocorrendo tudo bem nas duas barreiras, a transação avança.

## Como funciona a Guarda de Permissão

- **Comando Interno:** `garantir_permissao_usuario`
- Esta função verifica a sessão (`sessoes_usuario_local`) ativa. Se não existir sessão, a operação é sumariamente negada ("Operação exige usuário logado.").
- Se existir sessão, ela cruza o usuário com as permissões permitidas daquele perfil na tabela `perfil_permissoes_local`.
- Se tiver sucesso, ela cria um evento `PERMISSAO_OPERACAO_PERMITIDA` em `auditoria_operacional_local`.
- Se falhar, registra `OPERACAO_BLOQUEADA_PERMISSAO`.

## Operações Protegidas no Bloco 2

As seguintes operações e comandos exigem sessão e as seguintes permissões:

- **Abrir Caixa** (`abrir_caixa`): Exige `CAIXA_ABRIR`.
- **Fechar Caixa** (`fechar_caixa`): Exige `CAIXA_FECHAR`.
- **Finalizar Venda** (`finalizar_venda`, `registrar_pagamento`): Exige `VENDA_FINALIZAR`.
- **Restaurar Backup** (`restaurar_backup_local`): Exige `BACKUP_RESTAURAR`.

*(Criar ou validar backups continua livre. Cancelamentos, estornos e descontos entrarão nos próximos blocos).*

## Tratamento de UI

Se uma requisição Tauri é negada pela guarda de permissões, a exception gerada no Rust transborda para o Blazor de forma limpa. A UI intercepta a falha e mostra na tela. A tela de Segurança possui um painel simples `Testar Permissão` para o desenvolvedor atestar os blocos em funcionamento sem ter que simular um fechamento de caixa real.
