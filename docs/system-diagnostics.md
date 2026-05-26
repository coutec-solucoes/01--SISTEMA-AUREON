# Diagnóstico do Sistema

O **Diagnóstico do Sistema** (acessível via Menu > Diagnóstico) foi projetado para fornecer uma visão clara dos parâmetros operacionais vitais do Aureon PDV, ajudando no suporte técnico e identificação precoce de falhas de permissão.

## 1. Campos Analisados
- **Sistema Operacional & Arquitetura**: Ajuda o suporte a replicar falhas ou distribuir atualizações compatíveis (ex: windows/x86_64).
- **Versão do PDV**: Versão binária atual do App, mapeada através do pacote Cargo.
- **Caminho Base e Pastas**: Confirma se os caminhos `C:/Aureon/*` estão criados corretamente e em seus locais preditos.
- **Banco de Dados Local**: Confirma fisicamente a existência do arquivo `aureon-local.db`.
- **Permissão de Escrita (Base / Backups)**: Garante que o aplicativo do cliente e os comandos do Tauri possuem as permissões necessárias pelo Windows para modificar as pastas (o teste escreve fisicamente e apaga um pequeno arquivo de validação).

## 2. Solucionando Alertas

### "Faltam Pastas" ou "Diretório não encontrado"
Pode ocorrer em instalações incompletas ou deleção acidental pelo usuário.
**Solução**: Na própria tela de Diagnóstico, clique no botão secundário **Garantir Pastas Padrão**. O sistema recriará automaticamente todas as árvores necessárias dentro de `C:/Aureon/`. Nenhuma informação será sobrescrita.

### "Sem Permissão de Escrita"
Pode ocorrer devido a restrições de Anti-Vírus (ex: Windows Defender Controlled Folder Access) ou regras estritas da rede Active Directory.
**Solução**:
1. Execute o PDV como Administrador.
2. Nas Propriedades do diretório `C:/Aureon`, dê permissões *Total* ou *Leitura e Execução + Modificação* para o usuário do Windows que utiliza o PDV.

## 3. Riscos de Não Tratamento
Ignorar as mensagens de alerta da tela de diagnóstico implicará em:
1. Impossibilidade de guardar novos backups na pasta de arquivos de recuperação.
2. Vendas e cadastros não salvos (banco impossibilitado de atualizar seu `.wal`).
3. Falhas ao gerar relatórios ou registros fiscais (`logs` / `print-sim`).
