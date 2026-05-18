-- ================================================================
-- MIGRATION SQLite 001 — Schema Inicial
-- Versão: 1
-- Projeto: Aureon Sistema Inteligente
-- REGRAS: nomes em português, snake_case, sem acentos
-- ================================================================

-- Controle de configurações locais (valores sempre criptografados)
CREATE TABLE IF NOT EXISTS configuracoes_locais (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    chave                TEXT    NOT NULL UNIQUE,
    valor_criptografado  TEXT    NOT NULL,
    criado_em            TEXT    NOT NULL DEFAULT (datetime('now')),
    atualizado_em        TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Terminais registrados nesta instalação
CREATE TABLE IF NOT EXISTS terminais (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    terminal_id  TEXT    NOT NULL UNIQUE,
    nome         TEXT    NOT NULL,
    ativo        INTEGER NOT NULL DEFAULT 1,
    criado_em    TEXT    NOT NULL DEFAULT (datetime('now')),
    atualizado_em TEXT   NOT NULL DEFAULT (datetime('now'))
);

-- Logs locais de operação (sem dados sensíveis)
CREATE TABLE IF NOT EXISTS logs_locais (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    nivel       TEXT    NOT NULL CHECK(nivel IN ('DEBUG','INFO','WARN','ERROR')),
    componente  TEXT    NOT NULL,
    mensagem    TEXT    NOT NULL,
    criado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Fila de eventos para sincronização (outbox pattern)
CREATE TABLE IF NOT EXISTS sync_outbox (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id         TEXT    NOT NULL UNIQUE,
    idempotency_key  TEXT    NOT NULL UNIQUE,
    event_type       TEXT    NOT NULL,
    schema_version   INTEGER NOT NULL DEFAULT 1,
    payload          TEXT    NOT NULL,         -- JSON serializado
    status           TEXT    NOT NULL DEFAULT 'PENDENTE'
                             CHECK(status IN ('PENDENTE','ENVIANDO','ENVIADO','ERRO','IGNORADO')),
    tentativas       INTEGER NOT NULL DEFAULT 0,
    ultimo_erro      TEXT,
    criado_em        TEXT    NOT NULL DEFAULT (datetime('now')),
    atualizado_em    TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Fila de eventos recebidos do servidor (inbox pattern)
CREATE TABLE IF NOT EXISTS sync_inbox (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id         TEXT    NOT NULL UNIQUE,
    idempotency_key  TEXT    NOT NULL UNIQUE,
    event_type       TEXT    NOT NULL,
    schema_version   INTEGER NOT NULL DEFAULT 1,
    payload          TEXT    NOT NULL,
    status           TEXT    NOT NULL DEFAULT 'PENDENTE'
                             CHECK(status IN ('PENDENTE','PROCESSANDO','PROCESSADO','ERRO','IGNORADO')),
    tentativas       INTEGER NOT NULL DEFAULT 0,
    ultimo_erro      TEXT,
    criado_em        TEXT    NOT NULL DEFAULT (datetime('now')),
    atualizado_em    TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Log de execuções de sincronização
CREATE TABLE IF NOT EXISTS sync_logs (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    operacao     TEXT    NOT NULL,
    status       TEXT    NOT NULL,
    detalhes     TEXT,
    criado_em    TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Índices de performance
CREATE INDEX IF NOT EXISTS idx_sync_outbox_status    ON sync_outbox(status);
CREATE INDEX IF NOT EXISTS idx_sync_inbox_status     ON sync_inbox(status);
CREATE INDEX IF NOT EXISTS idx_logs_locais_nivel     ON logs_locais(nivel);
CREATE INDEX IF NOT EXISTS idx_logs_locais_criado_em ON logs_locais(criado_em);
