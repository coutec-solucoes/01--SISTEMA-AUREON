# Registro de Decisões de Projeto (ADR) — Fase 5

Este documento compila as decisões de arquitetura e padrões técnicos adotados durante o desenvolvimento da Fase 5.

---

## 🛡️ ADR 01: Segurança via Token Opaco UUID
- **Contexto**: A API necessita validar sessões de usuário e chaves de segurança da empresa para cada transação e alteração de parâmetros.
- **Decisão**: Rejeitado o uso de JWT (Json Web Tokens) para manter o alinhamento estrito com o padrão estabelecido na Fase 3. As requisições locais enviam o cabeçalho `Authorization: Bearer <token_uuid>`, validado diretamente na tabela `sessoes_usuarios` com hash SHA-256 no banco de dados local.
- **Consequência**: Garantia de revogabilidade imediata de chaves e sessões e menor sobrecarga computacional em hardware modesto local, mantendo a arquitetura offline simples e robusta.

---

## 🔌 ADR 02: Padronização Rígida de Rotas Operacionais
- **Contexto**: Diversos endpoints operacionais e cadastros de hardware foram propostos sob diferentes nomenclaturas em fases anteriores.
- **Decisão**: Padronizar rigidamente o prefixo `/configuracoes/operacionais` para todos os 17 endpoints operacionais. Foi banido completamente o uso do termo `/configuracoes/operacoes/`.
- **Consequência**: Uniformidade no roteamento Axum, facilidade de auditoria centralizada nas rotas locais de rede e consistência absoluta no consumo de APIs na retaguarda Blazor.

---

## ⚡ ADR 03: Separação de Parâmetros e Funcionamento Operacional Real
- **Contexto**: A Fase 5 foca em configurações e preparação física do ecossistema. Funcionalidades como transações financeiras, fechamentos, escuta real de balanças ou chamadores ativos de senhas eletrônicas exigiriam bibliotecas nativas de sistema operacional (Tauri/APS) que não pertencem ao escopo da retaguarda web.
- **Decisão**: Todos os endpoints de testes físicos (`/impressoras/{id}/testar`, `/perifericos/{id}/testar`, `/senhas-chamadas/{id}/testar` e `/balancas/{id}/ler-peso`) funcionam de forma simulada/mockada em ambiente web. O banco de dados armazena os parâmetros reais que serão consumidos futuramente pelo executável do PDV offline nativo na Fase 6.
- **Consequência**: Agilidade na homologação da retaguarda administrativa WebAssembly, isolando os drivers de hardware para o escopo nativo apropriado.
