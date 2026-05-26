# Gestão Local de Usuários e Políticas de Acesso

O Aureon PDV implementa um modelo estritamente offline-first para autenticação e gestão de usuários, visando garantir resiliência operacional contínua, mesmo durante quedas de internet. A regra principal da Fase 21 determina que o PDV seja independente de serviços cloud como Supabase ou Auth0 para operações do dia a dia.

## 1. Modelo de Dados Local
Os dados de segurança residem localmente nas seguintes tabelas do SQLite (`aureon-local.db`):
- `usuarios_local`: Armazena login, nome, status, flag `exige_troca_senha`, `senha_hash` e `pin_hash`.
- `perfis_local` / `permissoes_local` / `perfil_permissoes_local` / `usuario_perfis_local`: Controle de Acesso Baseado em Perfis (RBAC).
- `sessoes_usuario_local`: Rastreamento de sessões ativas (login e logout).
- `auditoria_operacional_local`: Registro inalterável de eventos críticos (ex: redefinição de senhas, criação de usuários, vendas canceladas).

## 2. Política de Senhas e PIN
- **Argon2id Obrigatório**: Toda senha e todo PIN transacionado no PDV é hasheado utilizando `Argon2id` (v19) com Salt criptográfico gerado randomicamente por execução. 
- **Tamanho Mínimo**: Senhas devem conter obrigatoriamente, no mínimo, 8 caracteres.
- **PIN Opcional**: O PIN é um recurso auxiliar projetado para agilizar a autorização de supervisor (ex: 4 dígitos numéricos). É armazenado separadamente (`pin_hash`) e sua criação exige a confirmação da senha principal em tela.
- **Isolamento de Memória**: Senhas e PINs trafegam do Blazor para o Tauri via IPC, onde são validados. Após a submissão, a UI apaga as variáveis e o Rust nunca retorna credenciais puras para a tela, nem mesmo salva as senhas em log ou na auditoria operacional.

## 3. Gestão de Usuários
Qualquer ação de criação, edição ou redefinição de senha exige permissões administrativas (`USUARIOS_GERENCIAR`). 
O fluxo operacional suporta as seguintes ações de segurança:
- **Criação de Usuário**: Admin pode criar login com senha inicial e forçar a troca no primeiro acesso através da flag `exige_troca_senha`.
- **Inativação**: Um usuário inativado não consegue logar. Adicionalmente, o sistema previne a inativação ou remoção do último perfil `ADMIN` ativo do banco, prevenindo lock-out generalizado.
- **Redefinição de Senhas**: A redefinição feita por outro usuário (Supervisor/Admin) tem sua própria auditoria específica (`SENHA_REDEFINIDA`).
- **Troca Própria**: Quando o usuário troca a própria senha sabendo a antiga (`SENHA_TROCADA`).

## 4. O que NÂO implementamos
Em virtude do foco *offline-first* do PDV:
- **Não há Auth0 ou AWS Cognito.**
- **Não há "Esqueci minha senha" via SMS ou E-mail:** Como o caixa atua offline, esquecer a senha exige intervenção de um Gerente ou Administrador local.
- **Não há Autenticação Biométrica** nativa embutida na base (limitado a Senhas e PINs).
