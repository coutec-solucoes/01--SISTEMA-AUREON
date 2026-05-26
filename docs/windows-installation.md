# Instalação Windows: Aureon PDV

Este documento descreve as etapas de preparação e infraestrutura para a instalação comercial do Aureon PDV no ambiente Windows (Fase 20, Bloco 10).

## 1. Diretórios Padrão
O sistema depende de uma estrutura de arquivos externa para manter a persistência de banco de dados, backups e logs, independentemente do binário ou atualizações futuras do software. 

A árvore de diretórios padrão é:
- `C:/Aureon/` (Base)
- `C:/Aureon/data/` (Armazena o SQLite ativo)
- `C:/Aureon/backups/` (Armazena as cópias de segurança locais e metadados)
- `C:/Aureon/logs/` (Armazena logs do sistema para auditoria e troubleshooting)
- `C:/Aureon/print-sim/` (Arquivos simulando as impressões de teste)
- `C:/Aureon/diagnostics/` (Relatórios de erros)

> [!CAUTION]
> Ao atualizar o binário principal (`.exe`), NUNCA sobrescreva ou apague a pasta `C:/Aureon/data/`. Ela contém todas as vendas e registros operacionais.

## 2. Permissões
O usuário Windows que executa o `aureon-pdv.exe` DEVE ter permissões de **leitura e gravação** recursivas sobre o diretório `C:/Aureon`. O sistema validará se é capaz de criar e apagar arquivos `.teste_escrita_aureon` dentro dos diretórios chaves (`data` e `backups`) durante o diagnóstico.

## 3. Preparando Instalação Manual
Caso esteja instalando o sistema manualmente:
1. Abra um terminal PowerShell como administrador.
2. Execute o script `scripts/create-aureon-dirs.ps1`.
3. Verifique a saúde do ambiente executando `scripts/check-pdv-environment.ps1`.
4. Copie o arquivo gerado em `target/release/aureon-pdv.exe` para a pasta desejada.

## 4. Build Local
Para gerar o executável final de uso local:
```powershell
.\scripts\build-pdv-windows.ps1
```
Esse script compilará a interface Blazor para os assets estáticos e depois invocará o `cargo build --release` no backend.

## 5. Limitações
- **Auto-Update**: O sistema atual ainda não contém rotinas de download de novas versões ou verificadores de servidor. Qualquer atualização requer intervenção técnica/manual (substituição do `.exe`).
- **Assinatura**: O executável construído via script não contém Assinatura de Código validada pela Microsoft. Usuários finais verão um aviso de SmartScreen. 
