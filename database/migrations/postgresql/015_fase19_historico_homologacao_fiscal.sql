-- Fase 19 — Bloco 5
-- Histórico de Homologação Fiscal, Tentativas Técnicas e Auditoria Local
--
-- ATENÇÃO:
-- Esta tabela registra SOMENTE metadados técnicos de operações de homologação.
-- NÃO representa autorização fiscal real.
-- NÃO representa protocolo fiscal real.
-- NÃO tem validade jurídica perante SEFAZ/DNIT/SET.
-- NÃO contém senha, chave privada ou certificado PFX.
-- Ambiente sempre restrito a 'HOMOLOGACAO'.

-- ─────────────────────────────────────────────────────────────────────────────
-- TIPO DE EVENTO: lista controlada
-- ─────────────────────────────────────────────────────────────────────────────
CREATE TYPE fiscal_tipo_evento_hom AS ENUM (
    'CERTIFICADO_VALIDADO',
    'ASSINATURA_PREVIEW_EXECUTADA',
    'XMLDSIG_HOMOLOGACAO_TENTADO',
    'NFCE_PREVIEW_GERADO',
    'SIFEN_PREVIEW_GERADO',
    'PREVIEW_VALIDADO_LOCALMENTE',
    'QRCODE_PREVIEW_GERADO',
    'CONECTIVIDADE_HOMOLOGACAO_TESTADA',
    'PRODUCAO_BLOQUEADA',
    'ERRO_HOMOLOGACAO_TECNICA'
);

-- ─────────────────────────────────────────────────────────────────────────────
-- TABELA PRINCIPAL
-- ─────────────────────────────────────────────────────────────────────────────
CREATE TABLE fiscal_homologacao_historico (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tipo_evento      fiscal_tipo_evento_hom NOT NULL,
    pais             TEXT        CHECK (pais IN ('BR', 'PY')),
    modelo           TEXT        CHECK (modelo IN ('NFE', 'NFCE', 'SIFEN', 'CTE')),

    -- Ambiente: apenas HOMOLOGACAO permitido nesta tabela
    ambiente         TEXT        NOT NULL DEFAULT 'HOMOLOGACAO'
                                 CHECK (ambiente = 'HOMOLOGACAO'),

    -- Referências opcionais (somente metadado, sem chave real autorizada)
    venda_id         UUID,
    chave_preview    TEXT,       -- Chave de acesso de preview (não é chave SEFAZ real)
    cdc_preview      TEXT,       -- CDC de preview SIFEN (não é CDC DNIT real)

    -- Resultado
    sucesso          BOOLEAN     NOT NULL DEFAULT false,
    mensagem         TEXT,
    payload_hash     TEXT,       -- SHA-256 do payload (sem expor o payload completo)
    erro_codigo      TEXT,

    -- Payload técnico resumido (truncado a 4KB por segurança)
    -- Nunca conterá: senha, chave privada, certificado PFX, XML completo de produção
    payload_preview  JSONB,

    -- Auditoria
    criado_em        TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE fiscal_homologacao_historico IS
    'Auditoria técnica interna de operações de homologação fiscal. '
    'Não representa autorização, protocolo ou documento fiscal com validade jurídica. '
    'Ambiente sempre HOMOLOGACAO. Nunca contém senha, chave ou certificado PFX.';

COMMENT ON COLUMN fiscal_homologacao_historico.chave_preview IS
    'Chave de acesso de preview técnico. Não é chave autorizada pela SEFAZ.';

COMMENT ON COLUMN fiscal_homologacao_historico.cdc_preview IS
    'CDC de preview SIFEN técnico. Não é CDC autorizado pela DNIT/SET.';

COMMENT ON COLUMN fiscal_homologacao_historico.payload_preview IS
    'Metadados técnicos resumidos (máx 4KB). Nunca contém senha, chave ou certificado.';

-- ─────────────────────────────────────────────────────────────────────────────
-- ÍNDICES
-- ─────────────────────────────────────────────────────────────────────────────
CREATE INDEX idx_fhh_tipo_evento   ON fiscal_homologacao_historico (tipo_evento);
CREATE INDEX idx_fhh_pais_modelo   ON fiscal_homologacao_historico (pais, modelo);
CREATE INDEX idx_fhh_venda_id      ON fiscal_homologacao_historico (venda_id) WHERE venda_id IS NOT NULL;
CREATE INDEX idx_fhh_criado_em     ON fiscal_homologacao_historico (criado_em DESC);
CREATE INDEX idx_fhh_sucesso       ON fiscal_homologacao_historico (sucesso);
CREATE INDEX idx_fhh_ambiente      ON fiscal_homologacao_historico (ambiente);
