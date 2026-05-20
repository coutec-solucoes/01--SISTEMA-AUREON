-- Migration 006: Criação das tabelas de cache para Clientes e Supervisores no PDV local

CREATE TABLE IF NOT EXISTS clientes_cache (
    id TEXT PRIMARY KEY,
    nome TEXT NOT NULL,
    documento TEXT,
    ativo BOOLEAN NOT NULL DEFAULT 1,
    atualizado_em TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS supervisores_cache (
    id TEXT PRIMARY KEY,
    nome TEXT NOT NULL,
    pin_hash TEXT NOT NULL,
    ativo BOOLEAN NOT NULL DEFAULT 1,
    atualizado_em TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_clientes_cache_nome ON clientes_cache(nome);
CREATE INDEX IF NOT EXISTS idx_clientes_cache_documento ON clientes_cache(documento);
