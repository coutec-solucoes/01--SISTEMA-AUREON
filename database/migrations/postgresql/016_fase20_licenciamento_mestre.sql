-- FASE 20 BLOCO 2: Licenciamento Mestre (Retaguarda)

CREATE TABLE lic_planos (
    id UUID PRIMARY KEY,
    codigo TEXT NOT NULL UNIQUE,
    nome TEXT NOT NULL,
    descricao TEXT,
    max_empresas INTEGER NOT NULL DEFAULT 1,
    max_terminais INTEGER NOT NULL DEFAULT 1,
    permite_pdv BOOLEAN NOT NULL DEFAULT true,
    permite_retaguarda BOOLEAN NOT NULL DEFAULT true,
    permite_delivery BOOLEAN NOT NULL DEFAULT false,
    permite_gourmet BOOLEAN NOT NULL DEFAULT false,
    permite_fiscal BOOLEAN NOT NULL DEFAULT false,
    ativo BOOLEAN NOT NULL DEFAULT true,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE lic_empresas (
    id UUID PRIMARY KEY,
    empresa_id TEXT NOT NULL,
    nome_empresa TEXT NOT NULL,
    documento TEXT,
    pais TEXT,
    status TEXT NOT NULL,
    plano_id UUID REFERENCES lic_planos(id),
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE lic_licencas (
    id UUID PRIMARY KEY,
    empresa_licenciada_id UUID NOT NULL REFERENCES lic_empresas(id),
    plano_id UUID NOT NULL REFERENCES lic_planos(id),
    status TEXT NOT NULL,
    modo TEXT NOT NULL,
    validade_inicio TIMESTAMPTZ,
    validade_fim TIMESTAMPTZ,
    tolerancia_offline_dias INTEGER NOT NULL DEFAULT 10,
    bloqueio_total BOOLEAN NOT NULL DEFAULT false,
    motivo_bloqueio TEXT,
    payload_licenca_json JSONB,
    assinatura_licenca TEXT,
    emitida_em TIMESTAMPTZ,
    ultimo_check_em TIMESTAMPTZ,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE lic_terminais (
    id UUID PRIMARY KEY,
    licenca_id UUID NOT NULL REFERENCES lic_licencas(id),
    installation_id TEXT,
    terminal_id TEXT,
    terminal_nome TEXT,
    dispositivo_hash TEXT,
    status TEXT NOT NULL,
    ultimo_check_em TIMESTAMPTZ,
    autorizado_em TIMESTAMPTZ,
    bloqueado_em TIMESTAMPTZ,
    motivo_bloqueio TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE lic_eventos (
    id UUID PRIMARY KEY,
    empresa_licenciada_id UUID REFERENCES lic_empresas(id),
    licenca_id UUID REFERENCES lic_licencas(id),
    terminal_id UUID REFERENCES lic_terminais(id),
    tipo_evento TEXT NOT NULL,
    mensagem TEXT,
    payload_preview JSONB,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Índices úteis
CREATE INDEX idx_lic_empresas_status ON lic_empresas(status);
CREATE INDEX idx_lic_licencas_status ON lic_licencas(status);
CREATE INDEX idx_lic_terminais_licenca ON lic_terminais(licenca_id);
