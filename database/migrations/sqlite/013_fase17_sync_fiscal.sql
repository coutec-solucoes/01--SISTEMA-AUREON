-- Migration Fase 17 - Sync Fiscal Local (Tabelas de Cache do Pacote)
-- SQLite Local PDV

-- Guarda as versões dos pacotes fiscais recebidos e aplicados
CREATE TABLE IF NOT EXISTS fiscal_versoes_aplicadas_cache (
    id TEXT PRIMARY KEY,
    versao TEXT NOT NULL,
    pacote_id TEXT,
    payload_hash TEXT,
    status TEXT NOT NULL, -- PENDENTE, APLICADO, ERRO
    total_registros INTEGER NOT NULL DEFAULT 0,
    aplicado_em TEXT,
    erro TEXT,
    criado_em TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    atualizado_em TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_fiscal_vers_apl_hash ON fiscal_versoes_aplicadas_cache(payload_hash);
CREATE INDEX IF NOT EXISTS idx_fiscal_vers_apl_vers ON fiscal_versoes_aplicadas_cache(versao);

-- Histórico de logs de aplicação dos pacotes fiscais (eventos técnicos)
CREATE TABLE IF NOT EXISTS fiscal_sync_logs (
    id TEXT PRIMARY KEY,
    pacote_id TEXT,
    versao TEXT,
    tipo_evento TEXT NOT NULL, -- RECEBIDO, VALIDADO, APLICADO, ERRO, IGNORADO_IDEMPOTENTE
    mensagem TEXT,
    payload_preview TEXT,
    criado_em TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_fiscal_sync_logs_pacote ON fiscal_sync_logs(pacote_id);
