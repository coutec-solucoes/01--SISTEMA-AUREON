-- ================================================================
-- MIGRATION PostgreSQL 001 — Schema Inicial
-- Versão: 1
-- Projeto: Aureon Sistema Inteligente
-- REGRAS: nomes em português, snake_case, sem acentos
-- ================================================================

-- Controle de migrations aplicadas
CREATE TABLE IF NOT EXISTS schema_migrations (
    versao       INTEGER PRIMARY KEY,
    nome         TEXT        NOT NULL,
    aplicado_em  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Cadastro de empresas (multi-empresa)
CREATE TABLE IF NOT EXISTS empresas (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    codigo       TEXT        NOT NULL UNIQUE,
    nome         TEXT        NOT NULL,
    documento    TEXT,
    pais         TEXT        NOT NULL DEFAULT 'BR' CHECK(pais IN ('BR','PY')),
    ativo        BOOLEAN     NOT NULL DEFAULT TRUE,
    criado_em    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Terminais registrados na empresa
CREATE TABLE IF NOT EXISTS terminais (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    empresa_id    UUID        NOT NULL REFERENCES empresas(id),
    terminal_id   TEXT        NOT NULL UNIQUE,
    nome          TEXT        NOT NULL,
    ativo         BOOLEAN     NOT NULL DEFAULT TRUE,
    ultimo_acesso TIMESTAMPTZ,
    criado_em     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Logs centralizados do sistema (sem dados sensíveis)
CREATE TABLE IF NOT EXISTS logs_sistema (
    id          BIGSERIAL   PRIMARY KEY,
    empresa_id  UUID        REFERENCES empresas(id),
    terminal_id TEXT,
    nivel       TEXT        NOT NULL CHECK(nivel IN ('DEBUG','INFO','WARN','ERROR')),
    componente  TEXT        NOT NULL,
    mensagem    TEXT        NOT NULL,
    criado_em   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Eventos de sincronização recebidos de terminais
CREATE TABLE IF NOT EXISTS sync_eventos_recebidos (
    id               BIGSERIAL   PRIMARY KEY,
    event_id         TEXT        NOT NULL UNIQUE,
    idempotency_key  TEXT        NOT NULL UNIQUE,
    event_type       TEXT        NOT NULL,
    schema_version   INTEGER     NOT NULL DEFAULT 1,
    empresa_id       UUID        REFERENCES empresas(id),
    terminal_id      TEXT,
    payload          JSONB       NOT NULL,
    status           TEXT        NOT NULL DEFAULT 'RECEBIDO'
                                 CHECK(status IN ('RECEBIDO','PROCESSANDO','PROCESSADO','ERRO','IGNORADO')),
    tentativas       INTEGER     NOT NULL DEFAULT 0,
    ultimo_erro      TEXT,
    criado_em        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Controle de idempotência global
CREATE TABLE IF NOT EXISTS sync_idempotencia (
    idempotency_key  TEXT        PRIMARY KEY,
    event_type       TEXT        NOT NULL,
    processado_em    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resultado        TEXT
);

-- Índices de performance
CREATE INDEX IF NOT EXISTS idx_terminais_empresa    ON terminais(empresa_id);
CREATE INDEX IF NOT EXISTS idx_logs_sistema_nivel   ON logs_sistema(nivel);
CREATE INDEX IF NOT EXISTS idx_logs_sistema_ts      ON logs_sistema(criado_em DESC);
CREATE INDEX IF NOT EXISTS idx_sync_eventos_status  ON sync_eventos_recebidos(status);
CREATE INDEX IF NOT EXISTS idx_sync_eventos_ts      ON sync_eventos_recebidos(criado_em DESC);

-- Registrar esta migration
INSERT INTO schema_migrations (versao, nome)
VALUES (1, 'schema_inicial')
ON CONFLICT (versao) DO NOTHING;
