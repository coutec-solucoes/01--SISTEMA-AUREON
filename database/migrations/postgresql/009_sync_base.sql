-- ================================================================
-- MIGRATION PostgreSQL 009 — Sincronização Base e Publicação para Terminais
-- Versão: 9
-- Projeto: Aureon Sistema Inteligente — Fase 6
-- REGRAS: nomes em português, snake_case, sem acentos
-- ================================================================
--
-- DECISÕES DE REAPROVEITAMENTO (documentadas conforme regra da Fase 6):
--
-- [REAPROVEITADO] sync_idempotencia        → já criada em 001_schema_inicial.sql
--                                            NÃO RECRIADA. Campos existentes:
--                                            idempotency_key, event_type, processado_em, resultado
--
-- [REAPROVEITADO] sync_eventos_recebidos   → já criada em 001_schema_inicial.sql
--                                            NÃO RECRIADA.
--
-- [REAPROVEITADO] eventos_publicacao       → já criada em 006_cadastros_pessoas.sql
--                                            NÃO RECRIADA. Campos:
--                                            id, tipo_evento, entidade, entidade_id,
--                                            payload, schema_version, processado, processado_em, criado_em
--
-- [ALTERADO]      terminais_pdv            → criada em 008_configuracoes_operacionais.sql
--                                            ALTER TABLE para adicionar colunas de sync.
--
-- ================================================================

-- ================================================================
-- PASSO 1: ALTER TABLE terminais_pdv — adicionar colunas de sync
-- Usa DO $$ para ser idempotente (não falha se coluna já existir)
-- ================================================================

DO $$
BEGIN
    -- chave_terminal: token opaco UUID para autenticação do terminal
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'terminais_pdv' AND column_name = 'chave_terminal'
    ) THEN
        ALTER TABLE terminais_pdv ADD COLUMN chave_terminal TEXT;
    END IF;

    -- status_sync: estado atual da sincronização do terminal
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'terminais_pdv' AND column_name = 'status_sync'
    ) THEN
        ALTER TABLE terminais_pdv ADD COLUMN status_sync TEXT NOT NULL DEFAULT 'PENDENTE'
            CHECK(status_sync IN ('PENDENTE','ATUALIZADO','ERRO','SEM_SYNC'));
    END IF;

    -- ultima_versao_recebida: hash/versão do último pacote aplicado
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'terminais_pdv' AND column_name = 'ultima_versao_recebida'
    ) THEN
        ALTER TABLE terminais_pdv ADD COLUMN ultima_versao_recebida TEXT;
    END IF;

    -- ultima_sincronizacao: timestamp real da última sync confirmada
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'terminais_pdv' AND column_name = 'ultima_sincronizacao'
    ) THEN
        ALTER TABLE terminais_pdv ADD COLUMN ultima_sincronizacao TIMESTAMPTZ;
    END IF;

    -- primeiro_sync_concluido: flag de primeira sync bem-sucedida
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'terminais_pdv' AND column_name = 'primeiro_sync_concluido'
    ) THEN
        ALTER TABLE terminais_pdv ADD COLUMN primeiro_sync_concluido BOOLEAN NOT NULL DEFAULT FALSE;
    END IF;
END $$;

-- ================================================================
-- PASSO 2: CONTROLE DE VERSÕES DE DADOS MESTRES
-- Cada grupo de dados tem um número de versão + hash de conteúdo
-- ================================================================

CREATE TABLE IF NOT EXISTS sync_versoes_dados (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tipo_dado      TEXT        NOT NULL UNIQUE,  -- ex: 'produtos_catalogo', 'usuarios_permissoes'
    versao         INTEGER     NOT NULL DEFAULT 1,
    descricao      TEXT,
    hash_conteudo  TEXT        NOT NULL,         -- SHA-256 do payload serializado
    criado_em      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_por UUID        REFERENCES pessoas(id) ON DELETE SET NULL
);

COMMENT ON TABLE sync_versoes_dados IS 'Controle de versão por grupo de dados mestres. Versão incrementa a cada alteração relevante.';

-- Seeds de versão inicial para todos os grupos de dados mestres
INSERT INTO sync_versoes_dados (tipo_dado, versao, descricao, hash_conteudo)
VALUES
    ('empresa_config',           1, 'Dados da empresa e parametros operacionais',  'initial'),
    ('moedas_cotacoes',          1, 'Moedas, cotacoes e configuracoes de cambio',  'initial'),
    ('usuarios_permissoes',      1, 'Usuarios, perfis, permissoes e supervisores', 'initial'),
    ('produtos_catalogo',        1, 'Grupos, subgrupos, marcas e produtos ativos', 'initial'),
    ('produtos_precos',          1, 'Precos vigentes por produto',                 'initial'),
    ('produtos_fiscal',          1, 'Dados fiscais base por produto',              'initial'),
    ('produtos_complementos',    1, 'Adicionais, sabores, combos e locais',        'initial'),
    ('configuracoes_operacionais',1,'PDV, terminais, impressoras, balancas etc.',  'initial'),
    ('dispositivos_perifericos', 1, 'Perifericos, setores e etiquetas',            'initial')
ON CONFLICT (tipo_dado) DO NOTHING;

-- ================================================================
-- PASSO 3: PACOTES DE SINCRONIZAÇÃO
-- Um pacote contém todos os dados para atualizar um terminal
-- ================================================================

CREATE TABLE IF NOT EXISTS pacotes_sincronizacao (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    terminal_id      UUID        NOT NULL REFERENCES terminais_pdv(id) ON DELETE CASCADE,
    tipo_pacote      TEXT        NOT NULL DEFAULT 'PRIMEIRA_SYNC'
                                 CHECK(tipo_pacote IN ('PRIMEIRA_SYNC','INCREMENTAL','FORCADO')),
    status           TEXT        NOT NULL DEFAULT 'GERADO'
                                 CHECK(status IN ('GERADO','ENVIADO','APLICADO','FALHOU','EXPIRADO')),
    idempotency_key  TEXT        NOT NULL UNIQUE,
    versao_geral     TEXT        NOT NULL,   -- hash SHA-256 do pacote completo
    total_itens      INTEGER     NOT NULL DEFAULT 0,
    gerado_em        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    enviado_em       TIMESTAMPTZ,
    aplicado_em      TIMESTAMPTZ,
    expira_em        TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '72 hours'),
    erro_detalhes    TEXT
);

COMMENT ON TABLE pacotes_sincronizacao IS 'Pacotes de dados gerados para envio aos terminais PDV. Cada pacote é idempotente.';

-- ================================================================
-- PASSO 4: ITENS DOS PACOTES DE SINCRONIZAÇÃO
-- Cada grupo de dados é um item separado dentro do pacote
-- ================================================================

CREATE TABLE IF NOT EXISTS pacotes_sincronizacao_itens (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    pacote_id      UUID        NOT NULL REFERENCES pacotes_sincronizacao(id) ON DELETE CASCADE,
    tipo_dado      TEXT        NOT NULL,      -- ex: 'produtos_catalogo'
    versao         INTEGER     NOT NULL,
    hash_conteudo  TEXT        NOT NULL,
    payload_json   JSONB       NOT NULL DEFAULT '{}',
    total_registros INTEGER    NOT NULL DEFAULT 0,
    criado_em      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE pacotes_sincronizacao_itens IS 'Itens individuais de cada pacote. Um item por grupo de dados mestres.';

-- ================================================================
-- PASSO 5: PUBLICAÇÕES MANUAIS
-- Controle de publicações disparadas pela Retaguarda
-- ================================================================

CREATE TABLE IF NOT EXISTS sync_publicacoes (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tipo_publicacao  TEXT        NOT NULL DEFAULT 'GERAL'
                                 CHECK(tipo_publicacao IN ('GERAL','DIRECIONADA','FORCADA')),
    status           TEXT        NOT NULL DEFAULT 'PENDENTE'
                                 CHECK(status IN ('PENDENTE','EM_ANDAMENTO','CONCLUIDA','PARCIAL','ERRO')),
    idempotency_key  TEXT        NOT NULL UNIQUE,
    criado_por       UUID        REFERENCES pessoas(id) ON DELETE SET NULL,
    observacao       TEXT,
    total_terminais  INTEGER     NOT NULL DEFAULT 0,
    terminais_ok     INTEGER     NOT NULL DEFAULT 0,
    terminais_erro   INTEGER     NOT NULL DEFAULT 0,
    criado_em        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    concluido_em     TIMESTAMPTZ
);

COMMENT ON TABLE sync_publicacoes IS 'Registro de publicações manuais de dados mestres disparadas pela Retaguarda.';

-- ================================================================
-- PASSO 6: ITENS DE PUBLICAÇÃO (por terminal)
-- ================================================================

CREATE TABLE IF NOT EXISTS sync_publicacoes_itens (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    publicacao_id   UUID        NOT NULL REFERENCES sync_publicacoes(id) ON DELETE CASCADE,
    terminal_id     UUID        NOT NULL REFERENCES terminais_pdv(id) ON DELETE CASCADE,
    pacote_id       UUID        REFERENCES pacotes_sincronizacao(id) ON DELETE SET NULL,
    status          TEXT        NOT NULL DEFAULT 'PENDENTE'
                                CHECK(status IN ('PENDENTE','ENVIADO','APLICADO','ERRO')),
    tentativas      INTEGER     NOT NULL DEFAULT 0,
    ultimo_erro     TEXT,
    criado_em       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ================================================================
-- PASSO 7: STATUS DE SINCRONIZAÇÃO DOS TERMINAIS
-- Resumo atualizado por terminal — para exibir na UI da Retaguarda
-- ================================================================

CREATE TABLE IF NOT EXISTS sync_status_terminais (
    terminal_id          UUID        PRIMARY KEY REFERENCES terminais_pdv(id) ON DELETE CASCADE,
    status               TEXT        NOT NULL DEFAULT 'SEM_SYNC'
                                     CHECK(status IN ('SEM_SYNC','SINCRONIZANDO','ATUALIZADO','PENDENTE','ERRO')),
    ultima_sincronizacao TIMESTAMPTZ,
    ultima_tentativa     TIMESTAMPTZ,
    versoes_aplicadas    JSONB       NOT NULL DEFAULT '{}',  -- { "tipo_dado": versao_aplicada }
    versoes_pendentes    JSONB       NOT NULL DEFAULT '{}',  -- { "tipo_dado": versao_disponivel }
    erro_detalhe         TEXT,
    criado_em            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE sync_status_terminais IS 'Resumo de sincronização por terminal. Atualizado a cada confirmação ou falha.';

-- ================================================================
-- PASSO 8: LOGS DE SINCRONIZAÇÃO
-- Histórico detalhado de eventos de sync (sem dados sensíveis)
-- ================================================================

CREATE TABLE IF NOT EXISTS sync_logs (
    id            BIGSERIAL   PRIMARY KEY,
    terminal_id   UUID        REFERENCES terminais_pdv(id) ON DELETE SET NULL,
    tipo_evento   TEXT        NOT NULL,     -- ex: 'TERMINAL_REGISTRADO', 'PRIMEIRA_SYNC_APLICADA'
    status        TEXT        NOT NULL DEFAULT 'INFO'
                              CHECK(status IN ('INFO','SUCESSO','AVISO','ERRO')),
    mensagem      TEXT        NOT NULL,
    detalhe_json  JSONB       NOT NULL DEFAULT '{}',  -- metadados sem dados sensíveis
    criado_em     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE sync_logs IS 'Log de todos os eventos de sincronizacao. Chaves e tokens nunca aparecem aqui.';

-- ================================================================
-- PASSO 9: ÍNDICES DE PERFORMANCE
-- ================================================================

CREATE INDEX IF NOT EXISTS idx_pacotes_sync_terminal    ON pacotes_sincronizacao(terminal_id, status);
CREATE INDEX IF NOT EXISTS idx_pacotes_sync_status      ON pacotes_sincronizacao(status, gerado_em DESC);
CREATE INDEX IF NOT EXISTS idx_pacotes_itens_pacote     ON pacotes_sincronizacao_itens(pacote_id);
CREATE INDEX IF NOT EXISTS idx_pacotes_itens_tipo       ON pacotes_sincronizacao_itens(tipo_dado);
CREATE INDEX IF NOT EXISTS idx_sync_pub_itens_terminal  ON sync_publicacoes_itens(terminal_id, status);
CREATE INDEX IF NOT EXISTS idx_sync_pub_itens_pub       ON sync_publicacoes_itens(publicacao_id);
CREATE INDEX IF NOT EXISTS idx_sync_logs_terminal       ON sync_logs(terminal_id, criado_em DESC);
CREATE INDEX IF NOT EXISTS idx_sync_logs_tipo           ON sync_logs(tipo_evento, criado_em DESC);
CREATE INDEX IF NOT EXISTS idx_sync_versoes_tipo        ON sync_versoes_dados(tipo_dado);
CREATE INDEX IF NOT EXISTS idx_terminais_pdv_status     ON terminais_pdv(status_sync, autorizado);

-- ================================================================
-- PASSO 10: Registrar migration
-- ================================================================

INSERT INTO schema_migrations (versao, nome)
VALUES (9, 'sync_base_fase6')
ON CONFLICT (versao) DO NOTHING;
