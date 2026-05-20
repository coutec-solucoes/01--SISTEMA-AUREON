-- FASE 12: COMPRAS E ENTRADA MANUAL (BLOCO 1)
-- Migration: 010_fase12_compras.sql

-- 1. Ajustes em produtos_cache
ALTER TABLE produtos_cache ADD COLUMN ultimo_custo_minor INTEGER NOT NULL DEFAULT 0;
ALTER TABLE produtos_cache ADD COLUMN ultimo_custo_moeda_codigo TEXT;
ALTER TABLE produtos_cache ADD COLUMN ultimo_custo_taxa_cambio_escala6 INTEGER;
ALTER TABLE produtos_cache ADD COLUMN ultimo_custo_atualizado_em TEXT;

-- 2. Tabela fornecedores_cache
CREATE TABLE IF NOT EXISTS fornecedores_cache (
    id            TEXT PRIMARY KEY,
    nome          TEXT NOT NULL,
    documento     TEXT,
    ativo         INTEGER NOT NULL DEFAULT 1 CHECK (ativo IN (0, 1)),
    atualizado_em TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_fornecedores_cache_ativo ON fornecedores_cache(ativo);
CREATE INDEX IF NOT EXISTS idx_fornecedores_cache_nome ON fornecedores_cache(nome);

-- 3. Tabela compras
CREATE TABLE IF NOT EXISTS compras (
    id                       TEXT PRIMARY KEY,
    fornecedor_id            TEXT NOT NULL,
    fornecedor_nome_snapshot TEXT NOT NULL,
    numero_nota              TEXT,
    serie                    TEXT,
    chave_acesso_xml_fiscal  TEXT,
    data_emissao             TEXT,
    status                   TEXT NOT NULL CHECK (status IN ('EM_ANDAMENTO', 'FINALIZADA', 'CANCELADA')),
    moeda_codigo             TEXT NOT NULL,
    taxa_cambio_escala6      INTEGER NOT NULL CHECK (taxa_cambio_escala6 > 0),
    subtotal_itens_minor     INTEGER NOT NULL DEFAULT 0 CHECK (subtotal_itens_minor >= 0),
    desconto_total_minor     INTEGER NOT NULL DEFAULT 0 CHECK (desconto_total_minor >= 0),
    frete_total_minor        INTEGER NOT NULL DEFAULT 0 CHECK (frete_total_minor >= 0),
    outras_despesas_minor    INTEGER NOT NULL DEFAULT 0 CHECK (outras_despesas_minor >= 0),
    impostos_total_minor     INTEGER NOT NULL DEFAULT 0 CHECK (impostos_total_minor >= 0),
    total_compra_minor       INTEGER NOT NULL DEFAULT 0 CHECK (total_compra_minor >= 0),
    observacao               TEXT,
    criado_em                TEXT NOT NULL,
    atualizado_em            TEXT NOT NULL,
    finalizada_em            TEXT,
    cancelada_em             TEXT,
    motivo_cancelamento      TEXT,
    usuario_id               TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_compras_status ON compras(status);
CREATE INDEX IF NOT EXISTS idx_compras_fornecedor_id ON compras(fornecedor_id);
CREATE INDEX IF NOT EXISTS idx_compras_numero_nota ON compras(numero_nota);
CREATE INDEX IF NOT EXISTS idx_compras_data_emissao ON compras(data_emissao);

-- 4. Tabela compra_itens
CREATE TABLE IF NOT EXISTS compra_itens (
    id                         TEXT PRIMARY KEY,
    compra_id                  TEXT NOT NULL,
    produto_id                 TEXT NOT NULL,
    descricao_produto_snapshot TEXT NOT NULL,
    quantidade_escala3         INTEGER NOT NULL CHECK (quantidade_escala3 > 0),
    custo_unitario_minor       INTEGER NOT NULL CHECK (custo_unitario_minor >= 0),
    total_item_minor           INTEGER NOT NULL CHECK (total_item_minor >= 0),
    lote                       TEXT,
    validade                   TEXT,
    serial                     TEXT,
    imei                       TEXT,
    cancelado                  INTEGER NOT NULL DEFAULT 0 CHECK (cancelado IN (0, 1)),
    criado_em                  TEXT NOT NULL,
    FOREIGN KEY(compra_id) REFERENCES compras(id)
);

CREATE INDEX IF NOT EXISTS idx_compra_itens_compra_id ON compra_itens(compra_id);
CREATE INDEX IF NOT EXISTS idx_compra_itens_produto_id ON compra_itens(produto_id);
