# Pessoas & Papéis

## 👥 Conceito de Pessoas e Múltiplos Papéis
No Sistema Aureon, a entidade **Pessoa** (`pessoas`) é o elemento centralizador de qualquer agente físico ou jurídico que interaja com o sistema. Uma mesma pessoa pode assumir **múltiplos papéis** simultaneamente (ex: um indivíduo que é *Funcionário* pode também ser *Cliente* e cadastrado como *Vendedor*).

A arquitetura de banco de dados suporta isso mapeando a relação 1:N com tabelas filhas opcionais de configurações específicas. O controle de quais papéis estão ativos em tempo de execução para cada registro é feito na tabela de associação `pessoas_papeis`.

---

## 🏷️ Papéis Disponíveis
1.  **Clientes** (`clientes_configuracoes`)
    *   Gerencia parâmetros de concessão de crédito de confiança (crediário).
    *   Campos: `limite_credito` (decimal) e `bloqueado_a_prazo` (booleano).
2.  **Fornecedores** (`fornecedores_configuracoes`)
    *   Determina as preferências comerciais de compra da empresa.
    *   Campos: `moeda_padrao` (BRL / PYG) e `prazo_pagamento_padrao_dias`.
3.  **Funcionários** (`funcionarios_configuracoes`)
    *   Controla a admissão do colaborador na retaguarda operacional.
    *   Campos: `cargo`, `data_admissao` e `salario_base`.
4.  **Vendedores** (`vendedores_configuracoes`)
    *   Mapeia o comissionamento de vendas ativas de retaguarda ou balcão.
    *   Campos: `percentual_comissao` e `ativo_para_vendas`.
5.  **Entregadores** (`entregadores_configuracoes`)
    *   Parâmetros de entrega logística local.
    *   Campos: `tipo_veiculo` (MOTO, CARRO, OUTRO), `placa_veiculo` e `cnh`.
6.  **Transportadoras** (`transportadoras_configuracoes`)
    *   Entidades parceiras de fretes e entregas externas/intermunicipais.
    *   Campos: `registro_antt` e `placa_veiculo_padrao`.

---

## 📞 Contatos & Endereços
*   **Contatos (`pessoas_contatos`)**: Permite cadastrar múltiplos meios de comunicação vinculados, incluindo telefones fixos, WhatsApp direto, e-mail de faturamento e sites corporativos.
*   **Endereços (`pessoas_enderecos`)**: Suporta endereçamento completo nacional e internacional, preparado com campos dedicados para CEP/Código Postal, logradouro, número, bairro, cidade, estado e país (adequado para operações no Brasil ou Paraguai).

---

## 🔒 Documentos Únicos
Para assegurar a integridade contra fraudes ou duplicidades cadastrais, o sistema aplica restrições de unicidade (`UNIQUE`) em nível de banco de dados para os seguintes documentos:
*   **CPF / CNPJ** (Brasil)
*   **RUC / Cédula de Identidade** (Paraguai)

---

## 🛑 Regras de Inativação Lógica
O sistema **não realiza deleções físicas** de registros de pessoas para garantir a integridade referencial histórica de futuras transações, notas fiscais e relatórios. 
*   A inativação de uma pessoa é realizada de forma lógica atualizando a flag `ativo = false` na tabela `pessoas`.
*   Quando inativada, a pessoa é automaticamente omitida de buscas operacionais de vendas, mas permanece indexada na auditoria e em relatórios históricos.
