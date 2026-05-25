-- Fase 20 — Bloco 1
-- Licenciamento Local, Identidade da Instalação e Estado Comercial da Empresa

-- 1. instalacao_local
CREATE TABLE IF NOT EXISTS instalacao_local (
    id TEXT PRIMARY KEY,
    installation_id TEXT NOT NULL UNIQUE,
    empresa_id TEXT,
    terminal_id TEXT,
    terminal_nome TEXT,
    dispositivo_hash TEXT,
    sistema_operacional TEXT,
    criado_em TEXT NOT NULL,
    atualizado_em TEXT NOT NULL
);

-- 2. licenca_local
CREATE TABLE IF NOT EXISTS licenca_local (
    id TEXT PRIMARY KEY,
    installation_id TEXT NOT NULL,
    empresa_id TEXT,
    plano_codigo TEXT NOT NULL,
    status TEXT NOT NULL, -- ATIVA, EXPIRADA, BLOQUEADA, PENDENTE_ATIVACAO, MODO_DEV
    modo TEXT NOT NULL, -- DEV, MANUAL, ONLINE_FUTURO
    validade_inicio TEXT,
    validade_fim TEXT,
    ultimo_check_em TEXT,
    tolerancia_offline_dias INTEGER NOT NULL DEFAULT 10,
    bloqueio_total INTEGER NOT NULL DEFAULT 0,
    motivo_bloqueio TEXT,
    assinatura_licenca TEXT,
    payload_licenca_json TEXT,
    criado_em TEXT NOT NULL,
    atualizado_em TEXT NOT NULL,
    FOREIGN KEY(installation_id) REFERENCES instalacao_local(installation_id)
);

-- 3. licenca_eventos
CREATE TABLE IF NOT EXISTS licenca_eventos (
    id TEXT PRIMARY KEY,
    installation_id TEXT,
    licenca_id TEXT,
    tipo_evento TEXT NOT NULL, -- LICENCA_CRIADA, LICENCA_ATIVADA_MANUALMENTE, LICENCA_CONSULTADA, LICENCA_EXPIRADA_DETECTADA, LICENCA_BLOQUEADA, LICENCA_MODO_DEV_ATIVO, LICENCA_TOLERANCIA_OFFLINE
    status_anterior TEXT,
    status_novo TEXT,
    mensagem TEXT,
    payload_preview TEXT,
    criado_em TEXT NOT NULL
);
