# FASE 20 — Instalação, Licenciamento, Backup e Atualização Base

**Status Final Esperado**: APROVADA E ENCERRADA COM RESSALVAS CONTROLADAS
**Data de Encerramento**: 2026-05-25

## Resumo Geral da Fase
A Fase 20 teve como foco garantir que o sistema AUREON pudesse ser executado localmente de forma segura, com restrições comerciais auditáveis (licenciamento) operando no modelo "offline-first". A fase cobriu a arquitetura do licenciamento (da retaguarda à verificação local criptográfica), a guarda operacional, a segurança dos dados por meio de backup integrado no PDV e o preparo do ambiente físico de distribuição para Windows, formando a base de instalação do sistema.

Nenhuma mudança no escopo de negócio fim (finanças, faturamento online) foi feita nesta fase, respeitando as restrições impostas.

## Lista de Blocos Implementados

- **Bloco 1**: Entidades e Fluxo na Retaguarda Mestre
- **Bloco 2**: Endpoints de Gestão e Check-in na Retaguarda Mestre
- **Bloco 3**: Identidade, Base Local e Tolerância Offline (PDV)
- **Bloco 4**: Assinatura Criptográfica de Licença (Ed25519)
- **Bloco 5**: Aplicação Local de Licença Assinada (PDV)
- **Bloco 6**: Sincronização Manual/Online da Licença (PDV -> Retaguarda)
- **Bloco 7**: Política de Bloqueio Suave e Alertas
- **Bloco 8**: Guarda Operacional de Licença (Bloqueio Controlado)
- **Bloco 9**: Backup Local e Restauração Controlada
- **Bloco 10**: Empacotamento Windows, Estrutura de Instalação e Diagnóstico de Ambiente
- **Bloco 11**: Documentação Final, Checklist de Homologação e Encerramento da Fase 20

## Commits por Bloco

- Bloco 1: `b5fa15e`
- Bloco 2: `ec9d06e`
- Bloco 3: `1bb3cd4`
- Bloco 4: `7211251`
- Bloco 5: `04fecc6`
- Bloco 6: `17dc500`
- Bloco 7: `d69be40`
- Bloco 8: `5422f25`
- Bloco 9: `128749d`
- Bloco 10: `12287c7`
- Bloco 11 (este): `[PENDENTE APLICAÇÃO]`

## Tabelas PostgreSQL Criadas (Retaguarda Mestre)
- `planos`: Definição comercial de capacidades.
- `empresas_licenciadas`: Entidade cliente vinculada à empresa do ecossistema.
- `licencas`: Registro de instâncias de uso geradas.
- `terminais`: Pontos físicos/lógicos que utilizam a licença.
- `eventos_licenciamento`: Registro inalterável de ciclo de vida.

## Tabelas SQLite Criadas (PDV Local)
- `instalacao_local`: Identificação do dispositivo local e empresa.
- `licenca_local`: Dados de validade, chave e restrição aplicáveis localmente.
- `licenca_eventos`: Histórico de ações com a licença no terminal.
- `licenca_config`: URL de check-in e chaves de conexão à retaguarda.

## Commands Tauri Criados
- `obter_status_licenca`
- `ativar_licenca_dev`
- `registrar_evento_licenca`
- `obter_identidade_instalacao`
- `configurar_licenciamento_online`
- `obter_config_licenciamento_online`
- `sincronizar_licenca_online`
- `obter_politica_licenca`
- `verificar_operacao_permitida_licenca`
- `criar_backup_local`
- `listar_backups_locais`
- `validar_backup_local`
- `restaurar_backup_local`
- `diagnosticar_banco_local`
- `diagnosticar_instalacao_sistema`
- `garantir_pastas_sistema`
- `obter_versao_app`

## Endpoints API Criados (aureon-api-local simulando Retaguarda)
- `POST /licenciamento/check-in`
- `GET /licenciamento/licencas/{id}/payload-assinado`
- `POST /licenciamento/licencas/verificar-payload`
- `GET /licenciamento/chaves/status`
*(Endpoints CRUD via services/aureon-api-local/src/routes/master_licenciamento.rs)*

## Telas Blazor Criadas / Atualizadas
- `LicencaPdv.razor` (Renovada, suporte a colagem, sync e alertas de bloqueio)
- `BackupPdv.razor` (Criada do zero)
- `DiagnosticoSistemaPdv.razor` (Criada do zero)
- `MainLayout.razor` (Links de menu atualizados)

## Scripts Criados
- `scripts/build-pdv-windows.ps1`
- `scripts/check-pdv-environment.ps1`
- `scripts/create-aureon-dirs.ps1`

## Regras Implementadas

### Licenciamento Offline-First
O PDV utiliza as propriedades em `licenca_local` para tomar decisões sobre disponibilidade comercial. Ao expirar a `validade`, o sistema acrescenta a `tolerancia_offline_dias`. Até esse limite estendido ser alcançado, o sistema funciona plenamente (Status = `TOLERANCIA_OFFLINE`), absorvendo falhas de conexão à internet ou ausência de sync. O PDV nâo deleta a licença caso a checagem falhe.

### Assinatura Ed25519
Toda licença é submetida a uma função de hash SHA-256 de forma canônica determinística no Backend Mestre (retaguarda). Com esse Hash, a Retaguarda assina o payload com a Chave Privada.
O PDV apenas guarda a Chave Pública, validando a assinatura criptograficamente a cada sync ou aplicação de payload manual. Payload forjado ou inconsistente com a assinatura é imediatamente rejeitado.

### Sincronização Online / Manual
O usuário (ou o sistema futuramente via cron) pode engatilhar uma sincronização manual chamando o check-in na retaguarda. Em caso de sucesso, o novo payload assinado é salvo. Em caso de falha (timeout, 500, no route to host), a exceção é engolida silenciosamente em termos de interrupção operacional, mantendo a licença local exatamente como estava (offline-first).

### Guarda Operacional
Aplicada restrição explícita (bloqueio rígido) somente em:
- `ABRIR_CAIXA`
- `FINALIZAR_VENDA`
- `FECHAR_VENDA_PAGAMENTO`

Caso a licença esteja `BLOQUEADA` ou `EXPIRADA` (fora da tolerância), a ação retorna Erro/Bloqueio e é auditada. A tela de licença, de backup e leitura seguem funcionando.

### Backup / Restauração
Isentos de bloqueio por licença, operam com permissões do Host base.
O SQLite Backup API nativo é usado (VACUUM INTO/Backup struct).
A restauração exige checagem de hash SHA-256 e `PRAGMA integrity_check`, cria um backup prévio obrigatório de rollback, e o usuário deve digitar "RESTAURAR" na interface para confirmar a exclusão e sobreposição da base de dados local atual.

### Diagnóstico de Instalação
O app obrigatoriamente espera (ou cria, caso autorizado) a estrutura `C:/Aureon/data`, `C:/Aureon/backups`, `C:/Aureon/logs` etc.
Scripts PS1 preparam o ambiente, e o PDV (no `/diagnostico-sistema`) checa se a leitura/escrita na pasta está operacional e mapeia propriedades base do host, como arquitetura, SO e tamanho disponível, ajudando o suporte técnico.

## Escopo Proibido (O que NÃO foi feito)
- Sem gateway de pagamento / cobrança de clientes.
- Sem recorrência automatizada de pagamentos.
- Sem auto-update remoto do executável do Tauri no momento.
- Sem assinatura de código (Code Signing Certificate) nativa do Windows no release final (requer infra futura).
- Nenhuma operação diagnóstica apaga dados operacionais de vendas.
- Rotinas de Backup e Atualização Licença são ilesas às validações de Guarda Operacional (nunca são bloqueadas).
- Nenhuma alteração no fluxo fiscal real com Sefaz.

## Pendências Futuras
1. Automação de auto-update (Tauri Updater ou serviço de Windows).
2. Agendamento em background da sincronização da licença (Check-in diário silencioso).
3. Integração com Gateway de Pagamento para emissão de cobranças na retaguarda.
4. Assinatura do instalador via certificado EV.
5. Upload de Backups para ambiente Cloud.

---

## Checklist Final de Homologação

### 1. Licenciamento local:
- [x] instalação local criada;
- [x] licença local criada;
- [x] eventos locais criados;
- [x] modo DEV funcional;
- [x] status de licença consultável offline.

### 2. Retaguarda mestre:
- [x] planos;
- [x] empresas licenciadas;
- [x] licenças;
- [x] terminais;
- [x] eventos.

### 3. Check-in:
- [x] endpoint de check-in;
- [x] payload para PDV;
- [x] validação conceitual de empresa/licença/terminal.

### 4. Assinatura:
- [x] Ed25519 implementado;
- [x] SHA-256 payload_hash;
- [x] key_id;
- [x] endpoint de payload assinado;
- [x] endpoint de verificação;
- [x] chave privada restrita à Retaguarda.

### 5. PDV offline:
- [x] verificação local de assinatura;
- [x] aplicação de payload assinado;
- [x] chave pública no PDV;
- [x] payload inválido não aplicado.

### 6. Sync licença:
- [x] URL Retaguarda;
- [x] sincronizar com Retaguarda;
- [x] falha de rede sem apagar licença local;
- [x] offline-first preservado.

### 7. Política:
- [x] OK;
- [x] ALERTA_VENCIMENTO;
- [x] TOLERANCIA_OFFLINE;
- [x] EXPIRADA;
- [x] BLOQUEADA;
- [x] MODO_DEV;
- [x] SEM_LICENCA.

### 8. Guarda operacional:
- [x] ABRIR_CAIXA protegido;
- [x] FINALIZAR_VENDA protegido;
- [x] FECHAR_VENDA_PAGAMENTO protegido;
- [x] tela Licença não bloqueada;
- [x] backup não bloqueado;
- [x] sincronização não bloqueada;
- [x] consulta não bloqueada.

### 9. Backup:
- [x] criar backup;
- [x] listar backup;
- [x] validar backup;
- [x] restaurar com confirmação RESTAURAR;
- [x] backup preventivo antes de restauração;
- [x] SHA-256;
- [x] PRAGMA integrity_check.

### 10. Instalação/diagnóstico:
- [x] C:/Aureon;
- [x] data;
- [x] backups;
- [x] logs;
- [x] print-sim;
- [x] diagnostics;
- [x] scripts PowerShell;
- [x] tela de diagnóstico;
- [x] diagnóstico de permissão e pastas.

### 11. Escopo proibido confirmado:
- [x] sem gateway de pagamento;
- [x] sem cobrança recorrente;
- [x] sem auto-update remoto;
- [x] sem assinatura de código oficial;
- [x] sem apagar dados operacionais;
- [x] sem bloquear backup/licença/sync;
- [x] sem alteração fiscal real;
- [x] sem alteração de estoque/financeiro/delivery/gourmet fora dos pontos autorizados.
