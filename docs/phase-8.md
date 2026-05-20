# Fase 8 — PDV Operacional: Sangria, Suprimento, Reimpressão, Histórico, Pré-vendas e Orçamentos

Este documento consolida as especificações arquiteturais, modelo de dados, commands Tauri/Rust e fluxos de interface desenvolvidos para a **Fase 8** do Aureon Sistema Inteligente.

---

## 🎯 Escopo do PDV Operacional

A Fase 8 implementa recursos críticos para o dia a dia do operador de caixa e segurança gerencial no PDV offline-first:

1. **Movimentações de Caixa**: Suprimento (entrada de moedas para troco), Sangria (retirada física de cédulas para segurança) e Vale Funcionário (adiantamentos direto do caixa).
2. **Autorização de Supervisor**: Controle de ações de segurança por PIN criptografado com validação em cache local.
3. **Histórico de Caixa & Estornos**: Visualização das ações do caixa do turno e cancelamentos com auditoria atrelada.
4. **Histórico de Vendas & Reimpressões**: Emissão não fiscal em formato texto (TXT) de comprovantes com controle de justificativa do operador e autorização gerencial.
5. **Pré-Vendas & Orçamentos**: Conversão de documentos de pré-venda e orçamento pré-cadastrados (sincronizados no cache) em vendas ativas do PDV local.
6. **Associação de Clientes**: Vinculação de clientes à venda com verificação e bloqueio de clientes inativos no cache local.

---

## 🛠️ Arquitetura e Estrutura de Banco de Dados

Foram aplicadas duas migrations no SQLite local (`005_pdv_operacional_fase8.sql` e `006_pdv_operacional_fase8_cache.sql`) para criar a seguinte estrutura de dados:

- `caixa_movimentacoes`: Registra suprimentos, sangrias e vales.
- `supervisor_autorizacoes_local`: Log de auditoria de todas as tentativas (aprovadas e negadas) de ações supervisionadas.
- `vendas_reimpressoes`: Controle de reimpressões de cupons com operador, supervisor, data e justificativa.
- `pre_vendas_cache` & `pre_vendas_itens_cache`: Caches de pré-vendas recebidos do retaguarda.
- `orcamentos_cache` & `orcamentos_itens_cache`: Caches de orçamentos recebidos do retaguarda.
- `clientes_cache`: Nome, documento e status de ativação de clientes locais.
- `supervisores_cache`: Tabela de segurança contendo PINs criptografados (`pin_hash` via Bcrypt) para autorização offline.

---

## 🔒 Segurança Local (Bcrypt)

- **Sem PIN em Texto Puro**: O PIN do supervisor inserido na UI é verificado em memória pelo comando Rust através do hash Bcrypt e nunca é exposto nos arquivos de banco, logs ou payloads de sync.
- **Auditoria Rigorosa**: Toda tentativa de supervisor gera uma linha com UUID próprio em `supervisor_autorizacoes_local` e alimenta o `sync_outbox` com os eventos `SUPERVISOR_AUTORIZACAO_APROVADA` ou `SUPERVISOR_AUTORIZACAO_NEGADA` para auditoria centralizada.

---

## 🚫 Restrições de Escopo Proibido

Conforme regras rígidas de arquitetura do projeto:
- **Zero Fiscal**: Nenhuma lógica de emissão fiscal real (NFC-e, NF-e ou SIFEN) foi implementada.
- **Zero TEF / Pix Real**: Integrações físicas com maquininhas de cartão não foram feitas. Os pagamentos são declaratórios.
- **Zero Kardex / Estoque**: O PDV local não calcula baixa de estoque complexa ou Kardex.
- **Zero Produção / Comandas**: Recursos para cozinha, mesas e comandas continuam fora do escopo do PDV operacional.
