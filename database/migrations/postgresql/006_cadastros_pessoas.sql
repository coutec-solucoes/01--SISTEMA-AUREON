-- ================================================================
-- MIGRATION PostgreSQL 006 — Cadastros Base: Pessoas e Papéis
-- Versão: 6
-- Projeto: Aureon Sistema Inteligente
-- Fase: 4
-- ================================================================
-- ESTRATÉGIA: Criar tabelas novas com IF NOT EXISTS.
-- Documentos únicos usam índices parciais (WHERE campo IS NOT NULL).
-- ================================================================

-- ================================================================
-- PARTE 1: TABELA PRINCIPAL DE PESSOAS
-- ================================================================

CREATE TABLE IF NOT EXISTS pessoas (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tipo_pessoa             TEXT        NOT NULL CHECK (tipo_pessoa IN ('FISICA', 'JURIDICA')),
    nome_razao_social       TEXT        NOT NULL,
    nome_fantasia           TEXT,
    -- Documentos por país/tipo
    cpf                     TEXT,
    cnpj                    TEXT,
    ci                      TEXT,       -- Cédula de Identidade (Paraguai)
    ruc                     TEXT,       -- Registro Único de Contribuyente (Paraguai)
    rg                      TEXT,
    inscricao_estadual      TEXT,
    inscricao_municipal     TEXT,
    data_nascimento         DATE,
    observacao              TEXT,
    ativo                   BOOLEAN     NOT NULL DEFAULT TRUE,
    criado_em               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índices parciais para unicidade de documentos quando preenchidos
CREATE UNIQUE INDEX IF NOT EXISTS uq_pessoas_cpf   ON pessoas (cpf)   WHERE cpf  IS NOT NULL AND cpf  <> '';
CREATE UNIQUE INDEX IF NOT EXISTS uq_pessoas_cnpj  ON pessoas (cnpj)  WHERE cnpj IS NOT NULL AND cnpj <> '';
CREATE UNIQUE INDEX IF NOT EXISTS uq_pessoas_ci    ON pessoas (ci)    WHERE ci   IS NOT NULL AND ci   <> '';
CREATE UNIQUE INDEX IF NOT EXISTS uq_pessoas_ruc   ON pessoas (ruc)   WHERE ruc  IS NOT NULL AND ruc  <> '';

-- ================================================================
-- PARTE 2: PAPÉIS DA PESSOA
-- ================================================================
-- Permite que uma pessoa seja cliente E fornecedor ao mesmo tempo.

CREATE TABLE IF NOT EXISTS pessoas_papeis (
    pessoa_id   UUID    NOT NULL REFERENCES pessoas(id) ON DELETE CASCADE,
    papel       TEXT    NOT NULL CHECK (papel IN (
                    'CLIENTE', 'FORNECEDOR', 'FUNCIONARIO',
                    'VENDEDOR', 'ENTREGADOR', 'TRANSPORTADORA'
                )),
    ativo       BOOLEAN NOT NULL DEFAULT TRUE,
    criado_em   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (pessoa_id, papel)
);

-- ================================================================
-- PARTE 3: CONTATOS
-- ================================================================

CREATE TABLE IF NOT EXISTS pessoas_contatos (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    pessoa_id           UUID        NOT NULL REFERENCES pessoas(id) ON DELETE CASCADE,
    telefone_principal  TEXT,
    whatsapp            TEXT,
    telefone_secundario TEXT,
    email               TEXT,
    site                TEXT,
    responsavel         TEXT,
    observacao          TEXT,
    criado_em           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ================================================================
-- PARTE 4: ENDEREÇOS
-- ================================================================

CREATE TABLE IF NOT EXISTS pessoas_enderecos (
    id                   UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    pessoa_id            UUID        NOT NULL REFERENCES pessoas(id) ON DELETE CASCADE,
    tipo_endereco        TEXT        NOT NULL DEFAULT 'PRINCIPAL'
                             CHECK (tipo_endereco IN ('PRINCIPAL','COBRANCA','ENTREGA','COMERCIAL','RESIDENCIAL')),
    pais                 TEXT        NOT NULL DEFAULT 'BR',
    estado_departamento  TEXT,
    cidade               TEXT,
    bairro               TEXT,
    logradouro           TEXT,
    numero               TEXT,
    complemento          TEXT,
    cep_codigo_postal    TEXT,
    referencia           TEXT,
    principal            BOOLEAN     NOT NULL DEFAULT FALSE,
    criado_em            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Garante no máximo um endereço principal por pessoa
CREATE UNIQUE INDEX IF NOT EXISTS uq_pessoa_endereco_principal
    ON pessoas_enderecos (pessoa_id)
    WHERE principal = TRUE;

-- ================================================================
-- PARTE 5: CONFIGURAÇÕES DE CLIENTE
-- ================================================================

CREATE TABLE IF NOT EXISTS clientes_configuracoes (
    pessoa_id               UUID            PRIMARY KEY REFERENCES pessoas(id) ON DELETE CASCADE,
    limite_credito          NUMERIC(15,2)   NOT NULL DEFAULT 0,
    permitir_crediario      BOOLEAN         NOT NULL DEFAULT FALSE,
    bloquear_venda_prazo    BOOLEAN         NOT NULL DEFAULT FALSE,
    observacao_credito      TEXT,
    status_cliente          TEXT            NOT NULL DEFAULT 'ATIVO'
                                CHECK (status_cliente IN ('ATIVO','INATIVO','BLOQUEADO')),
    criado_em               TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    atualizado_em           TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

-- ================================================================
-- PARTE 6: CONFIGURAÇÕES DE FORNECEDOR
-- ================================================================

CREATE TABLE IF NOT EXISTS fornecedores_configuracoes (
    pessoa_id               UUID        PRIMARY KEY REFERENCES pessoas(id) ON DELETE CASCADE,
    prazo_pagamento_padrao  INTEGER     DEFAULT 30,  -- em dias
    moeda_padrao_compra     TEXT        NOT NULL DEFAULT 'BRL',
    observacao_comercial    TEXT,
    status_fornecedor       TEXT        NOT NULL DEFAULT 'ATIVO'
                                CHECK (status_fornecedor IN ('ATIVO','INATIVO')),
    criado_em               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ================================================================
-- PARTE 7: CONFIGURAÇÕES DE FUNCIONÁRIO
-- ================================================================

CREATE TABLE IF NOT EXISTS funcionarios_configuracoes (
    pessoa_id               UUID            PRIMARY KEY REFERENCES pessoas(id) ON DELETE CASCADE,
    cargo                   TEXT,
    data_admissao           DATE,
    data_demissao           DATE,
    salario_base            NUMERIC(15,2)   DEFAULT 0,
    ativo_funcionario       BOOLEAN         NOT NULL DEFAULT TRUE,
    observacao_funcionario  TEXT,
    criado_em               TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    atualizado_em           TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

-- ================================================================
-- PARTE 8: CONFIGURAÇÕES DE VENDEDOR
-- ================================================================

CREATE TABLE IF NOT EXISTS vendedores_configuracoes (
    pessoa_id                   UUID            PRIMARY KEY REFERENCES pessoas(id) ON DELETE CASCADE,
    codigo_vendedor             TEXT,
    tipo_comissao               TEXT            NOT NULL DEFAULT 'SEM_COMISSAO'
                                    CHECK (tipo_comissao IN ('SEM_COMISSAO','PERCENTUAL','VALOR_FIXO')),
    percentual_comissao         NUMERIC(5,2)    NOT NULL DEFAULT 0
                                    CHECK (percentual_comissao >= 0 AND percentual_comissao <= 100),
    valor_comissao_fixa         NUMERIC(15,2)   NOT NULL DEFAULT 0,
    comissao_ativa              BOOLEAN         NOT NULL DEFAULT TRUE,
    criado_em                   TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    atualizado_em               TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

-- ================================================================
-- PARTE 9: CONFIGURAÇÕES DE ENTREGADOR
-- ================================================================

CREATE TABLE IF NOT EXISTS entregadores_configuracoes (
    pessoa_id           UUID        PRIMARY KEY REFERENCES pessoas(id) ON DELETE CASCADE,
    tipo_entregador     TEXT        DEFAULT 'PROPRIO',
    veiculo             TEXT,
    placa               TEXT,
    ativo_entregador    BOOLEAN     NOT NULL DEFAULT TRUE,
    criado_em           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ================================================================
-- PARTE 10: CONFIGURAÇÕES DE TRANSPORTADORA
-- ================================================================

CREATE TABLE IF NOT EXISTS transportadoras_configuracoes (
    pessoa_id                   UUID        PRIMARY KEY REFERENCES pessoas(id) ON DELETE CASCADE,
    contato_logistica           TEXT,
    observacao_logistica        TEXT,
    ativa_transportadora        BOOLEAN     NOT NULL DEFAULT TRUE,
    criado_em                   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em               TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ================================================================
-- PARTE 11: AUDITORIA DE CADASTROS (separada de logs_seguranca)
-- ================================================================

CREATE TABLE IF NOT EXISTS auditoria_cadastros (
    id              BIGSERIAL   PRIMARY KEY,
    entidade        TEXT        NOT NULL,   -- ex: 'PESSOA', 'PRODUTO', 'GRUPO'
    entidade_id     UUID,
    acao            TEXT        NOT NULL,   -- ex: 'CRIAR', 'EDITAR', 'INATIVAR'
    campo_alterado  TEXT,
    valor_anterior  JSONB,
    valor_novo      JSONB,
    usuario_id      UUID        REFERENCES usuarios(id),
    observacao      TEXT,
    criado_em       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_auditoria_cadastros_entidade
    ON auditoria_cadastros (entidade, entidade_id);

-- ================================================================
-- PARTE 12: EVENTOS DE PUBLICAÇÃO PARA PDVS (genérico, reutilizável)
-- ================================================================
-- Diferente de eventos_publicacao_configuracao (que é para empresa/config),
-- esta tabela registra eventos de negócio (pessoas, produtos) para sincronização futura com PDVs.

CREATE TABLE IF NOT EXISTS eventos_publicacao (
    id              BIGSERIAL   PRIMARY KEY,
    tipo_evento     TEXT        NOT NULL,   -- ex: 'PRODUTO_CRIADO', 'PESSOA_ALTERADA'
    entidade        TEXT        NOT NULL,   -- ex: 'PRODUTO', 'PESSOA'
    entidade_id     UUID,
    payload         JSONB       NOT NULL DEFAULT '{}',
    schema_version  INTEGER     NOT NULL DEFAULT 1,
    processado      BOOLEAN     NOT NULL DEFAULT FALSE,
    processado_em   TIMESTAMPTZ,
    criado_em       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_eventos_publicacao_tipo
    ON eventos_publicacao (tipo_evento, processado);
