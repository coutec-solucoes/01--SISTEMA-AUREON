# Fase 6 — Sincronização e Operação Offline do PDV

Este documento descreve as realizações, entregas e validações da **Fase 6** do Sistema Aureon, focada na arquitetura de sincronização robusta e bidirecional entre o servidor da Retaguarda (PostgreSQL) e os Terminais PDV locais (SQLite).

## Objetivos Alcançados

1. **Arquitetura de Sincronização Resiliente**: Estabelecemos o modelo Hub-Spoke com banco centralizado na Retaguarda e bancos SQLite isolados para cada terminal de vendas.
2. **Registro Seguro de Terminais**: Criamos o protocolo de registro, autorização e geração de chaves criptográficas (`chave_terminal`) para os PDVs.
3. **Publicação Transacional de Dados Mestres**: Fluxo controlado via API para preparar pacotes de dados reais e gerenciar as versões dos dados mestres.
4. **Aplicação Local no SQLite do PDV**: Carga e aplicação em lote transacional e idempotente das tabelas do Postgres diretamente no arquivo local de banco SQLite do PDV.
5. **Idempotência de Sync de Ponta a Ponta**: Uso de chaves de idempotência únicas e rastreamento local para evitar duplicação ou inconsistência.

## Estrutura de Documentos Técnicos da Fase 6

Consulte os seguintes guias para detalhes específicos de cada componente:

- **[Arquitetura de Sync](file:///e:/01-%20SISTEMA%20AUREON/docs/sync-architecture.md)**: Fluxo geral, topologia de rede e tabelas de controle.
- **[Registro de Terminal](file:///e:/01-%20SISTEMA%20AUREON/docs/terminal-registration.md)**: Passos e chamadas para autenticação e setup do terminal.
- **[Publicação de Dados Mestres](file:///e:/01-%20SISTEMA%20AUREON/docs/master-data-publication.md)**: Como os pacotes de dados reais são gerados na retaguarda.
- **[Cache Local SQLite](file:///e:/01-%20SISTEMA%20AUREON/docs/sqlite-cache.md)**: Mecanismo de persistência e suporte offline no PDV.
- **[Idempotência de Sincronização](file:///e:/01-%20SISTEMA%20AUREON/docs/sync-idempotency.md)**: Confiabilidade, repetição e logs de sincronização.
