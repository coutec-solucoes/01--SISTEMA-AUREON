-- FASE 11: ESTOQUE OPERACIONAL (BLOCO 1)
-- Migration: 009_fase11_estoque.sql

-- 1. Alteração na tabela de produtos_cache
ALTER TABLE produtos_cache ADD COLUMN controla_estoque INTEGER NOT NULL DEFAULT 1;

CREATE INDEX IF NOT EXISTS idx_produtos_cache_controla_estoque ON produtos_cache(controla_estoque);

-- 2. Tabela de Saldo / Cache de Estoque
CREATE TABLE IF NOT EXISTS produtos_estoque_cache (
    produto_id         TEXT PRIMARY KEY,
    quantidade_escala3 INTEGER NOT NULL DEFAULT 0,
    atualizado_em      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_produtos_estoque_cache_produto ON produtos_estoque_cache(produto_id);

-- 3. Tabela Kardex (Movimentações de Estoque)
CREATE TABLE IF NOT EXISTS estoque_movimentacoes (
    id                 TEXT PRIMARY KEY,
    produto_id         TEXT NOT NULL,
    quantidade_escala3 INTEGER NOT NULL,
    saldo_apos_escala3 INTEGER NOT NULL,
    tipo_movimentacao  TEXT NOT NULL,
    origem_tipo        TEXT NOT NULL,
    origem_id          TEXT NOT NULL,
    motivo             TEXT,
    usuario_id         TEXT NOT NULL,
    criado_em          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_estoque_mov_produto ON estoque_movimentacoes(produto_id);
CREATE INDEX IF NOT EXISTS idx_estoque_mov_origem ON estoque_movimentacoes(origem_tipo, origem_id);
CREATE INDEX IF NOT EXISTS idx_estoque_mov_tipo ON estoque_movimentacoes(tipo_movimentacao);
CREATE INDEX IF NOT EXISTS idx_estoque_mov_criado_em ON estoque_movimentacoes(criado_em);

-- 4. Tabela de Lotes (Estrutural)
CREATE TABLE IF NOT EXISTS estoque_lotes (
    id                 TEXT PRIMARY KEY,
    produto_id         TEXT NOT NULL,
    numero_lote        TEXT NOT NULL,
    validade           TEXT NOT NULL,
    saldo_escala3      INTEGER NOT NULL DEFAULT 0,
    ativo              INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_estoque_lotes_produto ON estoque_lotes(produto_id);
CREATE INDEX IF NOT EXISTS idx_estoque_lotes_validade ON estoque_lotes(validade);
