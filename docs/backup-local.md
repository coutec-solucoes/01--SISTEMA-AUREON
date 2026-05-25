# Backup e Restauração Local

Este documento descreve a implementação da rotina de **Backup Local, Restauração Controlada e Diagnóstico de Integridade** do PDV Aureon (Fase 20, Bloco 9).

## 1. Objetivo
Permitir a criação segura de cópias de segurança do banco de dados SQLite local, auditar a integridade desses arquivos e restaurar o sistema caso necessário. 
**Regra de Ouro**: A rotina de backup e restauração NÃO depende de licença ativa, garantindo o direito do usuário à recuperação de dados independente de seu status comercial com a Aureon.

## 2. Estrutura e Localização
- **Diretório Padrão:** `C:/Aureon/backups/` (ou configurável via `AUREON_BACKUP_DIR`).
- **Nomenclatura do Arquivo:** `aureon_bkp_YYYYMMDD_HHMMSS_{terminal_id}_{installation_id}.db`.
- **Metadados:** Arquivo `.json` gerado na mesma pasta, contendo hash SHA-256, origem e informações do ambiente.

## 3. Segurança e Integridade
### Geração de Hash
A integridade no nível de arquivo é protegida via `SHA-256`. O hash é gerado utilizando leitura em stream diretamente do binário copiado e salvo no arquivo de metadados.

### Validação `PRAGMA integrity_check`
Além da integridade do arquivo, a saúde dos dados dentro do backup é checada de forma nativa. O PDV realiza uma conexão temporária com o arquivo `.db` gerado e dispara a query `PRAGMA integrity_check`. Apenas se o retorno for `"ok"`, o backup é considerado válido no nível estrutural de tabelas e índices.
Uma validação secundária verifica se a tabela de `auth_roles` existe, garantindo que seja um banco do sistema Aureon.

## 4. Como Restaurar
A restauração foi construída com máxima precaução contra erros operacionais:
1. O usuário seleciona o backup na interface `/backup`.
2. O botão de restaurar exibirá um alerta vermelho exigindo digitação exata da palavra "RESTAURAR".
3. O Tauri então:
   - Valida a integridade do arquivo de backup (se falhar, aborta).
   - Realiza automaticamente um backup pré-restauração do estado **atual** do banco.
   - Aplica a restauração usando as APIs do `rusqlite`.
   - Adiciona um evento em `licenca_eventos` de que uma restauração foi concluída.

## 5. Limitações e Escopo Futuro
Neste bloco as seguintes funções NÃO foram implementadas propositalmente:
- **Cloud Backup:** Nenhum dado é enviado para a AWS/Supabase. Tudo permanece na pasta local.
- **Criptografia Simétrica de Backup:** O banco copiado está aberto (SQLite padrão). Futuramente será implementada encriptação em repouso.
- **Agendamento:** Os backups por enquanto são manuais ou gerados automaticamente apenas pré-restauração.

## 6. Riscos Conhecidos
- Caso ocorra falha de energia durante a restauração (gravação `restore`), o banco principal pode corromper. É por isso que um backup de segurança prévio é gerado antes do comando.
