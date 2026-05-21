-- Migration Fase 16 - Fiscal Base (Preview e Estrutura)
-- Criado em: 2026-05-20
-- NENHUMA OBRIGACAO FISCAL REAL AINDA, APENAS PREPARACAO/ESPELHO

CREATE TABLE IF NOT EXISTS fiscal_empresa_cache (
    id TEXT PRIMARY KEY,
    pais_fiscal TEXT NOT NULL CHECK (pais_fiscal IN ('BR', 'PY')),
    regime_fiscal TEXT,
    ambiente TEXT NOT NULL DEFAULT 'HOMOLOGACAO' CHECK (ambiente IN ('HOMOLOGACAO', 'PRODUCAO')),
    forma_emissao TEXT NOT NULL DEFAULT 'NORMAL' CHECK (forma_emissao IN ('NORMAL', 'CONTINGENCIA_OFFLINE')),
    certificado_alias TEXT,
    certificado_caminho TEXT,
    configuracao_json TEXT,
    atualizado_em TEXT
);
CREATE INDEX IF NOT EXISTS idx_fiscal_empresa_cache_pais ON fiscal_empresa_cache (pais_fiscal);

CREATE TABLE IF NOT EXISTS fiscal_numeracao_cache (
    id TEXT PRIMARY KEY,
    pais_fiscal TEXT NOT NULL CHECK (pais_fiscal IN ('BR', 'PY')),
    modelo_documento TEXT NOT NULL,
    serie TEXT NOT NULL,
    ultimo_numero INTEGER NOT NULL DEFAULT 0 CHECK (ultimo_numero >= 0),
    ambiente TEXT NOT NULL DEFAULT 'HOMOLOGACAO' CHECK (ambiente IN ('HOMOLOGACAO', 'PRODUCAO')),
    ativo INTEGER NOT NULL DEFAULT 1 CHECK (ativo IN (0, 1)),
    validade_inicio TEXT,
    validade_fim TEXT,
    timbrado TEXT,
    atualizado_em TEXT
);
CREATE INDEX IF NOT EXISTS idx_fiscal_numeracao_cache_busca ON fiscal_numeracao_cache (pais_fiscal, modelo_documento, serie, ambiente);

CREATE TABLE IF NOT EXISTS fiscal_ncm_cache (
    id TEXT PRIMARY KEY,
    codigo TEXT NOT NULL,
    descricao TEXT,
    ativo INTEGER NOT NULL DEFAULT 1 CHECK (ativo IN (0, 1)),
    atualizado_em TEXT
);
CREATE INDEX IF NOT EXISTS idx_fiscal_ncm_cache_codigo ON fiscal_ncm_cache (codigo);

CREATE TABLE IF NOT EXISTS fiscal_cfop_cache (
    id TEXT PRIMARY KEY,
    codigo TEXT NOT NULL,
    descricao TEXT,
    tipo_operacao TEXT,
    ativo INTEGER NOT NULL DEFAULT 1 CHECK (ativo IN (0, 1)),
    atualizado_em TEXT
);
CREATE INDEX IF NOT EXISTS idx_fiscal_cfop_cache_codigo ON fiscal_cfop_cache (codigo);

CREATE TABLE IF NOT EXISTS fiscal_cst_csosn_cache (
    id TEXT PRIMARY KEY,
    codigo TEXT NOT NULL,
    tipo TEXT NOT NULL,
    descricao TEXT,
    ativo INTEGER NOT NULL DEFAULT 1 CHECK (ativo IN (0, 1)),
    atualizado_em TEXT
);
CREATE INDEX IF NOT EXISTS idx_fiscal_cst_csosn_cache_busca ON fiscal_cst_csosn_cache (codigo, tipo);

CREATE TABLE IF NOT EXISTS fiscal_iva_cache (
    id TEXT PRIMARY KEY,
    codigo TEXT NOT NULL,
    descricao TEXT,
    aliquota_escala6 INTEGER NOT NULL DEFAULT 0 CHECK (aliquota_escala6 >= 0),
    ativo INTEGER NOT NULL DEFAULT 1 CHECK (ativo IN (0, 1)),
    atualizado_em TEXT
);
CREATE INDEX IF NOT EXISTS idx_fiscal_iva_cache_codigo ON fiscal_iva_cache (codigo);

CREATE TABLE IF NOT EXISTS fiscal_regras_tributarias_cache (
    id TEXT PRIMARY KEY,
    pais_fiscal TEXT NOT NULL CHECK (pais_fiscal IN ('BR', 'PY')),
    tipo_operacao TEXT NOT NULL,
    uf_origem TEXT,
    uf_destino TEXT,
    ncm_id TEXT,
    cfop_id TEXT,
    cst_csosn_id TEXT,
    iva_id TEXT,
    aliquota_icms_escala6 INTEGER NOT NULL DEFAULT 0 CHECK (aliquota_icms_escala6 >= 0),
    aliquota_pis_escala6 INTEGER NOT NULL DEFAULT 0 CHECK (aliquota_pis_escala6 >= 0),
    aliquota_cofins_escala6 INTEGER NOT NULL DEFAULT 0 CHECK (aliquota_cofins_escala6 >= 0),
    aliquota_iva_escala6 INTEGER NOT NULL DEFAULT 0 CHECK (aliquota_iva_escala6 >= 0),
    reducao_base_escala6 INTEGER NOT NULL DEFAULT 0 CHECK (reducao_base_escala6 >= 0),
    ativo INTEGER NOT NULL DEFAULT 1 CHECK (ativo IN (0, 1)),
    atualizado_em TEXT
);
CREATE INDEX IF NOT EXISTS idx_fiscal_regras_tributarias_cache_busca ON fiscal_regras_tributarias_cache (pais_fiscal, tipo_operacao);

CREATE TABLE IF NOT EXISTS fiscal_eventos_logs (
    id TEXT PRIMARY KEY,
    venda_id TEXT,
    tipo_evento TEXT NOT NULL,
    origem TEXT,
    payload_preview TEXT,
    mensagem TEXT,
    criado_em TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_fiscal_eventos_logs_venda ON fiscal_eventos_logs (venda_id);

-- Alterando a tabela de produtos (cache local) com campos fiscais
ALTER TABLE produtos_cache ADD COLUMN ncm_id TEXT;
ALTER TABLE produtos_cache ADD COLUMN iva_id TEXT;
ALTER TABLE produtos_cache ADD COLUMN cst_csosn_id TEXT;
ALTER TABLE produtos_cache ADD COLUMN cfop_padrao_id TEXT;
ALTER TABLE produtos_cache ADD COLUMN origem_mercadoria TEXT;
ALTER TABLE produtos_cache ADD COLUMN fiscal_atualizado_em TEXT;

CREATE INDEX IF NOT EXISTS idx_produtos_cache_ncm ON produtos_cache (ncm_id);
CREATE INDEX IF NOT EXISTS idx_produtos_cache_iva ON produtos_cache (iva_id);

-- Alterando a tabela de vendas (somente campos de preview, NENHUM DADO OFICIAL DE EMISSAO AINDA)
ALTER TABLE vendas ADD COLUMN fiscal_pais TEXT;
ALTER TABLE vendas ADD COLUMN fiscal_ambiente TEXT;
ALTER TABLE vendas ADD COLUMN fiscal_modelo_preview TEXT;
ALTER TABLE vendas ADD COLUMN fiscal_status_preparacao TEXT;
ALTER TABLE vendas ADD COLUMN fiscal_total_base_minor INTEGER NOT NULL DEFAULT 0 CHECK (fiscal_total_base_minor >= 0);
ALTER TABLE vendas ADD COLUMN fiscal_total_imposto_minor INTEGER NOT NULL DEFAULT 0 CHECK (fiscal_total_imposto_minor >= 0);
ALTER TABLE vendas ADD COLUMN fiscal_preview_json TEXT;
ALTER TABLE vendas ADD COLUMN fiscal_calculado_em TEXT;

CREATE INDEX IF NOT EXISTS idx_vendas_fiscal_status_preparacao ON vendas (fiscal_status_preparacao);

-- Alterando a tabela de itens da venda (somente estrutural e calculo)
ALTER TABLE venda_itens ADD COLUMN fiscal_cfop_id TEXT;
ALTER TABLE venda_itens ADD COLUMN fiscal_cst_csosn_id TEXT;
ALTER TABLE venda_itens ADD COLUMN fiscal_iva_id TEXT;
ALTER TABLE venda_itens ADD COLUMN fiscal_ncm_id TEXT;
ALTER TABLE venda_itens ADD COLUMN fiscal_base_minor INTEGER NOT NULL DEFAULT 0 CHECK (fiscal_base_minor >= 0);
ALTER TABLE venda_itens ADD COLUMN fiscal_aliquota_escala6 INTEGER NOT NULL DEFAULT 0 CHECK (fiscal_aliquota_escala6 >= 0);
ALTER TABLE venda_itens ADD COLUMN fiscal_imposto_minor INTEGER NOT NULL DEFAULT 0 CHECK (fiscal_imposto_minor >= 0);
ALTER TABLE venda_itens ADD COLUMN fiscal_preview_json TEXT;
