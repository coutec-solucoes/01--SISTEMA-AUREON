# 🏢 Configuração da Empresa — Guia Técnico

Este guia detalha a estrutura de persistência, as regras cadastrais e os fluxos de alteração dos dados gerais da empresa no sistema Aureon.

---

## 1. Persistência de Dados (PostgreSQL)

Os dados da empresa são descentralizados fisicamente em tabelas separadas para normalização ideal no PostgreSQL:

*   `empresas`
*   `configuracoes_empresa`
*   `empresas_documentos`
*   `empresas_contatos`
*   `empresas_enderecos`
*   `empresas_logos`

Cada tabela contém uma restrição de chave estrangeira (`FOREIGN KEY`) referenciando `empresas(id)` com remoção em cascata (`ON DELETE CASCADE`).

---

## 2. Fluxo de Validação & Salvamento

Ao alterar as informações na tela de **Identificação Geral** ou **Contatos & Endereço**, o backend Axum valida:
1.  **Nome Fantasia**: Obrigatório, não pode ser nulo ou vazio.
2.  **Razão Social**: Obrigatório, não pode ser nulo ou vazio.
3.  **Tipo de Pessoa**: Valida contra o enum de bancos (`FISICA` ou `JURIDICA`).

Caso ocorra alguma divergência, o backend retorna um payload estruturado em `RespostaBase` com código HTTP `400 Bad Request`.

---

## 3. Idioma Reativo

O Aureon suporta localização multilíngue completa:
*   **Idioma Principal**: O idioma padrão das telas e do painel gestor (ex: `pt-BR`, `es-PY`).
*   **Idioma dos Comprovantes**: O idioma das impressões térmicas emitidas pelo PDV.
*   **Controle de Permissão**: Parâmetro `permitir_idioma_usuario` controla se operadores podem alterar individualmente seu idioma.
