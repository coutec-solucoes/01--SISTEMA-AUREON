# Manual de Instalação e Configuração Técnica — AUREON

Este documento descreve o processo de instalação e o uso do assistente técnico (`AUREON Config`) responsável por provisionar o ambiente offline-first do PDV.

## 1. Requisitos Prévios
- Sistema Operacional: Windows 10/11.
- PostgreSQL 14+ instalado e rodando na máquina ou servidor de retaguarda local.
- Usuário do PostgreSQL (`postgres` ou equivalente) com privilégio `CREATEDB` ou `superuser`.

## 2. Visão Geral da Arquitetura de Instalação
O AUREON separa a configuração técnica do aplicativo PDV comercial. A configuração é feita exclusivamente pelo aplicativo **AUREON Config**.

- **App de Configuração:** `apps/aureon-config`
- **App de Vendas:** `apps/aureon-pdv`

O PDV não pode ser inicializado sem que a configuração técnica tenha sido concluída com sucesso.

## 3. Passo a Passo do Setup

### Passo 1: Conexão PostgreSQL
Ao abrir o **AUREON Config**, você deverá informar:
- Host (ex: localhost)
- Porta (ex: 5432)
- Usuário (com privilégio CREATEDB)
- Senha

O sistema fará uma conexão genérica e verificará no catálogo `pg_roles` se as permissões necessárias existem. A senha plana não é gravada nos logs em nenhum momento.

### Passo 2: Criação de Banco de Dados
Você informará o nome fantasia da empresa (Ex: "Padaria São João").
O AUREON Config irá:
- Normalizar o nome removendo acentos e convertendo para ASCII minúsculo (`padaria_sao_joao_bd`).
- Executar `CREATE DATABASE padaria_sao_joao_bd` de forma segura.
- Garantir que não sobrescreva um banco já existente.

### Passo 3: Criação do Administrador
O sistema exige a criação do usuário Root inicial (Administrador).
A senha inserida aqui sofrerá hash utilizando **bcrypt**.

### Passo 4: Cofre de Chaves (.keystore)
O sistema vai gerar uma chave AES-256 única e randômica (`C:/Aureon/config/.keystore`).
Este arquivo é vital: se ele for apagado ou alterado, a criptografia dos arquivos de configuração locais e os acessos locais serão corrompidos.

### Passo 5: Finalização (Migrations e Seeds)
A última etapa realiza:
1. Conexão definitiva com o banco da empresa.
2. Criação da tabela `schema_migrations`.
3. Execução das migrations (Tabelas estruturais).
4. Execução de Seeds (Moedas, Perfis, Formas de Pagamento, Tesouraria).
5. Inserção do Administrador.
6. Gravação das credenciais de conexão seguras em `C:/Aureon/config/server.config.enc`.

## 4. O que o PDV Consome?
Ao iniciar o `aureon-pdv`, ele simplesmente checa se `C:/Aureon/config/.keystore` e `server.config.enc` existem e estão decifráveis. Caso não, ele entra em modo bloqueio e pede ao usuário que rode o **AUREON Config**.
