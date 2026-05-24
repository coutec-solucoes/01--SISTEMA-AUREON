-- database/migrations/postgresql/013_fase17_fiscal_mestre.sql
-- Fase 17 - Bloco 1: Retaguarda Fiscal Mestre e Versionamento

-- 1. fiscal_dicionario_ncm
CREATE TABLE IF NOT EXISTS fiscal_dicionario_ncm (
    id UUID PRIMARY KEY,
    codigo TEXT NOT NULL,
    descricao TEXT,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    vigencia_inicio DATE,
    vigencia_fim DATE,
    versao_origem TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 2. fiscal_dicionario_cfop
CREATE TABLE IF NOT EXISTS fiscal_dicionario_cfop (
    id UUID PRIMARY KEY,
    codigo TEXT NOT NULL,
    descricao TEXT,
    tipo_operacao TEXT,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    vigencia_inicio DATE,
    vigencia_fim DATE,
    versao_origem TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 3. fiscal_dicionario_cst_csosn
CREATE TABLE IF NOT EXISTS fiscal_dicionario_cst_csosn (
    id UUID PRIMARY KEY,
    codigo TEXT NOT NULL,
    tipo TEXT NOT NULL CHECK (tipo IN ('CST', 'CSOSN', 'PIS', 'COFINS')),
    descricao TEXT,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    vigencia_inicio DATE,
    vigencia_fim DATE,
    versao_origem TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 4. fiscal_dicionario_iva
CREATE TABLE IF NOT EXISTS fiscal_dicionario_iva (
    id UUID PRIMARY KEY,
    codigo TEXT NOT NULL,
    descricao TEXT,
    pais_fiscal TEXT NOT NULL DEFAULT 'PY' CHECK (pais_fiscal IN ('BR', 'PY')),
    aliquota_escala6 BIGINT NOT NULL DEFAULT 0 CHECK (aliquota_escala6 >= 0),
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    vigencia_inicio DATE,
    vigencia_fim DATE,
    versao_origem TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 5. fiscal_empresas_config
CREATE TABLE IF NOT EXISTS fiscal_empresas_config (
    id UUID PRIMARY KEY,
    empresa_id UUID,
    filial_id UUID,
    pais_fiscal TEXT NOT NULL CHECK (pais_fiscal IN ('BR', 'PY')),
    regime_fiscal TEXT,
    ambiente TEXT NOT NULL DEFAULT 'HOMOLOGACAO' CHECK (ambiente IN ('HOMOLOGACAO', 'PRODUCAO')),
    forma_emissao TEXT NOT NULL DEFAULT 'NORMAL' CHECK (forma_emissao IN ('NORMAL', 'CONTINGENCIA_OFFLINE')),
    certificado_alias TEXT,
    certificado_caminho TEXT,
    configuracao_json JSONB,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 6. fiscal_numeracao_mestre
CREATE TABLE IF NOT EXISTS fiscal_numeracao_mestre (
    id UUID PRIMARY KEY,
    empresa_id UUID,
    filial_id UUID,
    pais_fiscal TEXT NOT NULL CHECK (pais_fiscal IN ('BR', 'PY')),
    modelo_documento TEXT NOT NULL,
    serie TEXT NOT NULL,
    ultimo_numero BIGINT NOT NULL DEFAULT 0 CHECK (ultimo_numero >= 0),
    ambiente TEXT NOT NULL DEFAULT 'HOMOLOGACAO' CHECK (ambiente IN ('HOMOLOGACAO', 'PRODUCAO')),
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    validade_inicio DATE,
    validade_fim DATE,
    timbrado TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 7. fiscal_regras_tributarias_mestre
CREATE TABLE IF NOT EXISTS fiscal_regras_tributarias_mestre (
    id UUID PRIMARY KEY,
    empresa_id UUID,
    filial_id UUID,
    pais_fiscal TEXT NOT NULL CHECK (pais_fiscal IN ('BR', 'PY')),
    tipo_operacao TEXT NOT NULL,
    uf_origem TEXT,
    uf_destino TEXT,
    ncm_id UUID,
    cfop_id UUID,
    cst_csosn_id UUID,
    iva_id UUID,
    aliquota_icms_escala6 BIGINT NOT NULL DEFAULT 0 CHECK (aliquota_icms_escala6 >= 0),
    aliquota_pis_escala6 BIGINT NOT NULL DEFAULT 0 CHECK (aliquota_pis_escala6 >= 0),
    aliquota_cofins_escala6 BIGINT NOT NULL DEFAULT 0 CHECK (aliquota_cofins_escala6 >= 0),
    aliquota_iva_escala6 BIGINT NOT NULL DEFAULT 0 CHECK (aliquota_iva_escala6 >= 0),
    reducao_base_escala6 BIGINT NOT NULL DEFAULT 0 CHECK (reducao_base_escala6 >= 0),
    prioridade INTEGER NOT NULL DEFAULT 100 CHECK (prioridade >= 0),
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    vigencia_inicio DATE,
    vigencia_fim DATE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 8. fiscal_versoes_publicacao
CREATE TABLE IF NOT EXISTS fiscal_versoes_publicacao (
    id UUID PRIMARY KEY,
    versao TEXT NOT NULL,
    pais_fiscal TEXT CHECK (pais_fiscal IN ('BR', 'PY')),
    empresa_id UUID,
    filial_id UUID,
    status TEXT NOT NULL DEFAULT 'RASCUNHO' CHECK (status IN ('RASCUNHO', 'PUBLICADA', 'CANCELADA', 'REPROCESSADA')),
    tipo_pacote TEXT NOT NULL DEFAULT 'SYNC_FISCAL',
    payload_hash TEXT,
    total_registros INTEGER NOT NULL DEFAULT 0 CHECK (total_registros >= 0),
    publicado_em TIMESTAMPTZ,
    usuario_id UUID,
    observacao TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 9. fiscal_versoes_publicacao_itens
CREATE TABLE IF NOT EXISTS fiscal_versoes_publicacao_itens (
    id UUID PRIMARY KEY,
    versao_id UUID NOT NULL REFERENCES fiscal_versoes_publicacao(id) ON DELETE CASCADE,
    tipo_dado TEXT NOT NULL CHECK (tipo_dado IN ('NCM', 'CFOP', 'CST_CSOSN', 'IVA', 'REGRA_TRIBUTARIA', 'EMPRESA_CONFIG', 'NUMERACAO')),
    registro_id UUID,
    operacao TEXT NOT NULL CHECK (operacao IN ('UPSERT', 'DELETE_LOGICO')),
    payload JSONB NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 10. fiscal_auditoria_mestre
CREATE TABLE IF NOT EXISTS fiscal_auditoria_mestre (
    id UUID PRIMARY KEY,
    entidade TEXT NOT NULL,
    entidade_id UUID,
    acao TEXT NOT NULL,
    payload_anterior JSONB,
    payload_novo JSONB,
    usuario_id UUID,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Índices obrigatórios
CREATE INDEX IF NOT EXISTS idx_fiscal_dicionario_ncm_codigo ON fiscal_dicionario_ncm(codigo);
CREATE INDEX IF NOT EXISTS idx_fiscal_dicionario_cfop_codigo ON fiscal_dicionario_cfop(codigo);
CREATE INDEX IF NOT EXISTS idx_fiscal_dicionario_cst_csosn_codigo_tipo ON fiscal_dicionario_cst_csosn(codigo, tipo);
CREATE INDEX IF NOT EXISTS idx_fiscal_dicionario_iva_pais_codigo ON fiscal_dicionario_iva(pais_fiscal, codigo);
CREATE INDEX IF NOT EXISTS idx_fiscal_empresas_config_emp_fil_pais ON fiscal_empresas_config(empresa_id, filial_id, pais_fiscal);
CREATE INDEX IF NOT EXISTS idx_fiscal_numeracao_mestre_emp_fil_mod_ser_amb ON fiscal_numeracao_mestre(empresa_id, filial_id, modelo_documento, serie, ambiente);
CREATE INDEX IF NOT EXISTS idx_fiscal_regras_tributarias_mestre_pais_tipo ON fiscal_regras_tributarias_mestre(pais_fiscal, tipo_operacao);
CREATE INDEX IF NOT EXISTS idx_fiscal_regras_tributarias_mestre_emp_fil ON fiscal_regras_tributarias_mestre(empresa_id, filial_id);
CREATE INDEX IF NOT EXISTS idx_fiscal_versoes_publicacao_versao ON fiscal_versoes_publicacao(versao);
CREATE INDEX IF NOT EXISTS idx_fiscal_versoes_publicacao_status ON fiscal_versoes_publicacao(status);
CREATE INDEX IF NOT EXISTS idx_fiscal_versoes_publicacao_itens_versao_id ON fiscal_versoes_publicacao_itens(versao_id);
CREATE INDEX IF NOT EXISTS idx_fiscal_auditoria_mestre_entidade_id ON fiscal_auditoria_mestre(entidade, entidade_id);
