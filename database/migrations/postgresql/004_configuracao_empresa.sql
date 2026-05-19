-- ================================================================
-- MIGRATION PostgreSQL 004 — Configuração da Empresa, Moedas e Fiscal Base
-- Versão: 4
-- Projeto: Aureon Sistema Inteligente
-- ================================================================

-- 1. Configurações gerais da Empresa
CREATE TABLE IF NOT EXISTS configuracoes_empresa (
    empresa_id               UUID        PRIMARY KEY REFERENCES empresas(id) ON DELETE CASCADE,
    razao_social             TEXT        NOT NULL,
    tipo_pessoa              TEXT        NOT NULL CHECK(tipo_pessoa IN ('FISICA', 'JURIDICA')),
    status_empresa           TEXT        NOT NULL DEFAULT 'EM_CONFIGURACAO' CHECK(status_empresa IN ('ATIVA', 'INATIVA', 'BLOQUEADA', 'EM_CONFIGURACAO')),
    observacoes              TEXT,
    idioma_padrao            TEXT        NOT NULL DEFAULT 'PT' CHECK(idioma_padrao IN ('PT', 'ES')),
    idioma_comprovantes      TEXT        NOT NULL DEFAULT 'PT' CHECK(idioma_comprovantes IN ('PT', 'ES')),
    permitir_idioma_usuario  BOOLEAN     NOT NULL DEFAULT TRUE,
    idioma_autoatendimento   TEXT        NOT NULL DEFAULT 'PT' CHECK(idioma_autoatendimento IN ('PT', 'ES')),
    ambiente_fiscal          TEXT        NOT NULL DEFAULT 'HOMOLOGACAO' CHECK(ambiente_fiscal IN ('HOMOLOGACAO', 'PRODUCAO')),
    atualizado_em            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. Identificação de Documentos por País
CREATE TABLE IF NOT EXISTS empresas_documentos (
    empresa_id               UUID        PRIMARY KEY REFERENCES empresas(id) ON DELETE CASCADE,
    cnpj                     TEXT,
    inscricao_estadual       TEXT,
    inscricao_municipal      TEXT,
    cpf                      TEXT,
    rg                       TEXT,
    ruc                      TEXT,
    ci                       TEXT
);

-- 3. Contatos da Empresa
CREATE TABLE IF NOT EXISTS empresas_contatos (
    empresa_id               UUID        PRIMARY KEY REFERENCES empresas(id) ON DELETE CASCADE,
    telefone_principal       TEXT        NOT NULL,
    whatsapp                 TEXT,
    telefone_secundario      TEXT,
    email                    TEXT,
    responsavel              TEXT        NOT NULL,
    site                     TEXT
);

-- 4. Endereço da Empresa
CREATE TABLE IF NOT EXISTS empresas_enderecos (
    empresa_id               UUID        PRIMARY KEY REFERENCES empresas(id) ON DELETE CASCADE,
    pais                     TEXT        NOT NULL CHECK(pais IN ('BR', 'PY')),
    estado                   TEXT        NOT NULL,
    cidade                   TEXT        NOT NULL,
    bairro                   TEXT,
    logradouro               TEXT        NOT NULL,
    numero                   TEXT,
    complemento              TEXT,
    cep                      TEXT,
    referencia               TEXT
);

-- 5. Armazenamento / Referência do Logo da Empresa
CREATE TABLE IF NOT EXISTS empresas_logos (
    empresa_id               UUID        PRIMARY KEY REFERENCES empresas(id) ON DELETE CASCADE,
    caminho_logo             TEXT,
    usar_comprovantes        BOOLEAN     NOT NULL DEFAULT TRUE,
    usar_relatorios          BOOLEAN     NOT NULL DEFAULT TRUE
);

-- 6. Configuração de Regras Multimoedas da Empresa
CREATE TABLE IF NOT EXISTS empresas_moedas (
    empresa_id               UUID        NOT NULL REFERENCES empresas(id) ON DELETE CASCADE,
    moeda_id                 UUID        NOT NULL REFERENCES moedas(id),
    tipo_moeda               TEXT        NOT NULL CHECK(tipo_moeda IN ('PRINCIPAL', 'SECUNDARIA', 'TERCEIRA')),
    ordem_exibicao           INTEGER     NOT NULL DEFAULT 1,
    permitir_pagamento_multiplo BOOLEAN  NOT NULL DEFAULT TRUE,
    permitir_troco_diferente BOOLEAN     NOT NULL DEFAULT TRUE,
    PRIMARY KEY(empresa_id, moeda_id),
    CONSTRAINT unica_tipo_moeda UNIQUE(empresa_id, tipo_moeda)
);

-- 7. Controle e Cotações Diárias
CREATE TABLE IF NOT EXISTS cotacoes_moedas (
    id                       UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    empresa_id               UUID        NOT NULL REFERENCES empresas(id) ON DELETE CASCADE,
    data_cotacao             DATE        NOT NULL DEFAULT CURRENT_DATE,
    moeda_origem_id          UUID        NOT NULL REFERENCES moedas(id),
    moeda_destino_id         UUID        NOT NULL REFERENCES moedas(id),
    taxa_direta              NUMERIC(18, 10) NOT NULL CHECK(taxa_direta > 0),
    taxa_inversa             NUMERIC(18, 10) NOT NULL CHECK(taxa_inversa > 0),
    usuario_id               UUID,       -- Relacionado ao admin criador
    observacao               TEXT,
    status                   TEXT        NOT NULL DEFAULT 'ATIVA' CHECK(status IN ('ATIVA', 'SUBSTITUIDA', 'CANCELADA')),
    criado_em                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índice parcial de cotação ativa (Somente uma ativa por par por dia por empresa)
CREATE UNIQUE INDEX IF NOT EXISTS idx_unica_cotacao_ativa 
ON cotacoes_moedas(empresa_id, data_cotacao, moeda_origem_id, moeda_destino_id) 
WHERE status = 'ATIVA';

-- 8. Parâmetros Operacionais da Empresa
CREATE TABLE IF NOT EXISTS parametros_operacionais_empresa (
    empresa_id                      UUID        PRIMARY KEY REFERENCES empresas(id) ON DELETE CASCADE,
    permitir_estoque_negativo       BOOLEAN     NOT NULL DEFAULT FALSE,
    bloquear_produto_vencido        BOOLEAN     NOT NULL DEFAULT TRUE,
    alertar_produto_vencendo        BOOLEAN     NOT NULL DEFAULT TRUE,
    dias_alerta_vencimento          INTEGER     NOT NULL DEFAULT 15 CHECK(dias_alerta_vencimento >= 0),
    permitir_alterar_preco_pdv      BOOLEAN     NOT NULL DEFAULT FALSE,
    permitir_desconto_pdv           BOOLEAN     NOT NULL DEFAULT TRUE,
    exigir_supervisor_desconto      BOOLEAN     NOT NULL DEFAULT TRUE,
    exigir_supervisor_cancelamento  BOOLEAN     NOT NULL DEFAULT TRUE,
    permitir_venda_prazo            BOOLEAN     NOT NULL DEFAULT TRUE,
    exigir_cliente_completo_crediario BOOLEAN   NOT NULL DEFAULT TRUE,
    permitir_venda_offline          BOOLEAN     NOT NULL DEFAULT TRUE,
    dias_maximos_offline            INTEGER     NOT NULL DEFAULT 10 CHECK(dias_maximos_offline > 0),
    atualizado_em                   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 9. Configurações Fiscais Base do Brasil
CREATE TABLE IF NOT EXISTS configuracoes_fiscais_brasil (
    empresa_id               UUID        PRIMARY KEY REFERENCES empresas(id) ON DELETE CASCADE,
    regime_tributario        TEXT        NOT NULL CHECK(regime_tributario IN ('SIMPLES_NACIONAL', 'REGIME_NORMAL')),
    preparar_nfce            BOOLEAN     NOT NULL DEFAULT TRUE,
    preparar_nfe             BOOLEAN     NOT NULL DEFAULT TRUE,
    preparar_nfse            BOOLEAN     NOT NULL DEFAULT TRUE,
    regra_tributaria_base    TEXT,
    ambiente                 TEXT        NOT NULL DEFAULT 'HOMOLOGACAO' CHECK(ambiente IN ('HOMOLOGACAO', 'PRODUCAO')),
    provedor_fiscal          TEXT,
    atualizado_em            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 10. Configurações Fiscais Base do Paraguai
CREATE TABLE IF NOT EXISTS configuracoes_fiscais_paraguai (
    empresa_id               UUID        PRIMARY KEY REFERENCES empresas(id) ON DELETE CASCADE,
    regime_tributario        TEXT        NOT NULL CHECK(regime_tributario IN ('SIMPLE', 'GENERAL')),
    preparar_sifen           BOOLEAN     NOT NULL DEFAULT TRUE,
    regra_tributaria_base    TEXT,
    ambiente                 TEXT        NOT NULL DEFAULT 'HOMOLOGACAO' CHECK(ambiente IN ('HOMOLOGACAO', 'PRODUCAO')),
    provedor_fiscal          TEXT,
    atualizado_em            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 11. Auditoria de Alterações Críticas (JSON)
CREATE TABLE IF NOT EXISTS auditoria_eventos (
    id                       BIGSERIAL   PRIMARY KEY,
    empresa_id               UUID        REFERENCES empresas(id) ON DELETE SET NULL,
    usuario_id               UUID,
    acao                     TEXT        NOT NULL,
    entidade                 TEXT        NOT NULL,
    entidade_id              TEXT,
    valor_anterior           JSONB,
    valor_novo               JSONB,
    motivo                   TEXT,
    criado_em                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 12. Fila / Logs de Eventos para Publicação nos PDVs (Offline Sync)
CREATE TABLE IF NOT EXISTS eventos_publicacao_configuracao (
    id                       BIGSERIAL   PRIMARY KEY,
    empresa_id               UUID        REFERENCES empresas(id) ON DELETE CASCADE,
    tipo_evento              TEXT        NOT NULL,
    schema_version           INTEGER     NOT NULL DEFAULT 1,
    payload                  JSONB       NOT NULL,
    criado_em                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
