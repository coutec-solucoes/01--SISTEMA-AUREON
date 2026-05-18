# FASE 1 — INSTALAÇÃO, AUREON CONFIG E BANCO BASE

## Objetivo
Estabelecer a fundação de instalação, provisionamento automático do banco de dados (PostgreSQL) local e persistência segura das configurações técnicas.

## Status: APROVADA E IMPLEMENTADA

## Entregas Principais
1. **AUREON Config (App Isolado):** Aplicativo criado (`apps/aureon-config`) em Tauri 2.0 + Blazor WASM focado inteiramente na configuração inicial, expurgando o PDV de regras de setup perigosas e de alto privilégio.
2. **Cofre de Chaves e Configuração AES:** Implementação da geração segura do `.keystore` usando `rand` e persistência do arquivo codificado em Base64. Os dados sensíveis como conexão do banco são armazenados no `server.config.enc` através de AES-256-GCM. A senha em si nunca é revelada ou logada.
3. **Provisionamento PostgreSQL Dinâmico:** Validação do perfil `CREATEDB` ou superuser. Normalização de caracteres para a segurança contra injeção SQL, culminando no comando nativo `CREATE DATABASE`.
4. **Migrations e Seeds Idempotentes:** O processo implanta as estruturas SQL no banco zero da empresa. A tabela `schema_migrations` foi adotada e criada como tracker de versionamento. Seeds idempotentes via `ON CONFLICT DO NOTHING` preenchem:
   - Moedas: BRL, PYG, USD.
   - Perfis: Administrador, Gerente, Operador, Suporte Técnico.
   - Formas de Pagamento (6 tipos padrões).
5. **Administrador Inicial Seguro:** Cria a Tesouraria e o Usuário Raiz associando o perfil via Hash Bcrypt e integrando toda a rede do schema inicial de modo confiável.

## Notas Técnicas
- As senhas jamais navegam puras além dos *boundaries* DTO-Tauri. Elas são hasheadas via Rust ou criptografadas simetricamente no disco. 
- O banco local SQLite continua existindo e inicializa normalmente (herança da Fase 0), aguardando para atuar como cache *offline-first* nas próximas fases.
- O PDV é restrito a avisar "Terminal não configurado" se o `.keystore` ou `.enc` não existirem.

## Próximos Passos
- Na **Fase 2**, utilizaremos essa configuração para implementar o Cadastro Local de Produtos e Sincronização Piloto.
