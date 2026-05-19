-- ================================================================
-- MIGRATION PostgreSQL 007 — Cadastros Base: Produtos
-- Versão: 7
-- Projeto: Aureon Sistema Inteligente
-- Fase: 4
-- ================================================================

-- ================================================================
-- PARTE 1: GRUPOS DE PRODUTOS
-- ================================================================

CREATE TABLE IF NOT EXISTS produtos_grupos (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    nome        TEXT        NOT NULL,
    descricao   TEXT,
    ativo       BOOLEAN     NOT NULL DEFAULT TRUE,
    ordem       INTEGER     NOT NULL DEFAULT 0,
    criado_em   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_produtos_grupos_nome ON produtos_grupos (UPPER(nome));

-- ================================================================
-- PARTE 2: SUBGRUPOS DE PRODUTOS
-- ================================================================

CREATE TABLE IF NOT EXISTS produtos_subgrupos (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    grupo_id    UUID        NOT NULL REFERENCES produtos_grupos(id),
    nome        TEXT        NOT NULL,
    descricao   TEXT,
    ativo       BOOLEAN     NOT NULL DEFAULT TRUE,
    ordem       INTEGER     NOT NULL DEFAULT 0,
    criado_em   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_produtos_subgrupos_nome_grupo
    ON produtos_subgrupos (grupo_id, UPPER(nome));

-- ================================================================
-- PARTE 3: MARCAS
-- ================================================================

CREATE TABLE IF NOT EXISTS produtos_marcas (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    nome        TEXT        NOT NULL,
    descricao   TEXT,
    ativo       BOOLEAN     NOT NULL DEFAULT TRUE,
    criado_em   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_produtos_marcas_nome ON produtos_marcas (UPPER(nome));

-- ================================================================
-- PARTE 4: LOCAIS DE PRODUÇÃO
-- ================================================================

CREATE TABLE IF NOT EXISTS locais_producao (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    nome        TEXT        NOT NULL,
    descricao   TEXT,
    ativo       BOOLEAN     NOT NULL DEFAULT TRUE,
    criado_em   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ================================================================
-- PARTE 5: TABELA PRINCIPAL DE PRODUTOS
-- ================================================================

CREATE TABLE IF NOT EXISTS produtos (
    id                              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    codigo_interno                  TEXT,
    codigo_barras                   TEXT,
    descricao                       TEXT            NOT NULL,
    descricao_detalhada             TEXT,
    referencia                      TEXT,
    grupo_id                        UUID            NOT NULL REFERENCES produtos_grupos(id),
    subgrupo_id                     UUID            REFERENCES produtos_subgrupos(id),
    marca_id                        UUID            REFERENCES produtos_marcas(id),
    unidade_medida                  TEXT            NOT NULL DEFAULT 'UN',
    -- Preço e custo (moeda principal da empresa)
    preco_custo                     NUMERIC(15,4)   NOT NULL DEFAULT 0
                                        CHECK (preco_custo >= 0),
    margem_lucro_percentual         NUMERIC(7,4)    NOT NULL DEFAULT 0
                                        CHECK (margem_lucro_percentual >= 0),
    preco_venda                     NUMERIC(15,4)   NOT NULL DEFAULT 0
                                        CHECK (preco_venda >= 0),
    -- Estoque base (cadastral — operacional fica para fase posterior)
    estoque_atual                   NUMERIC(15,4)   NOT NULL DEFAULT 0,
    estoque_minimo                  NUMERIC(15,4)   NOT NULL DEFAULT 0
                                        CHECK (estoque_minimo >= 0),
    controla_estoque                BOOLEAN         NOT NULL DEFAULT TRUE,
    -- Flags de características
    controla_validade               BOOLEAN         NOT NULL DEFAULT FALSE,
    produto_balanca                 BOOLEAN         NOT NULL DEFAULT FALSE,
    reconfirmar_pesagem_pdv         BOOLEAN         NOT NULL DEFAULT FALSE,
    leitura_etiqueta_balanca        BOOLEAN         NOT NULL DEFAULT FALSE,
    -- Flags de tipo
    produto_pizza                   BOOLEAN         NOT NULL DEFAULT FALSE,
    produto_combo                   BOOLEAN         NOT NULL DEFAULT FALSE,
    permite_adicionais              BOOLEAN         NOT NULL DEFAULT FALSE,
    -- Regras de venda
    desconto_maximo_percentual      NUMERIC(5,2)    NOT NULL DEFAULT 100
                                        CHECK (desconto_maximo_percentual >= 0 AND desconto_maximo_percentual <= 100),
    exibir_catalogo                 BOOLEAN         NOT NULL DEFAULT TRUE,
    -- Localização
    local_producao_id               UUID            REFERENCES locais_producao(id),
    -- Status
    ativo                           BOOLEAN         NOT NULL DEFAULT TRUE,
    criado_em                       TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    atualizado_em                   TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

-- Unicidade de código de barras e referência (índice parcial para permitir nulo)
CREATE UNIQUE INDEX IF NOT EXISTS uq_produtos_codigo_barras
    ON produtos (codigo_barras) WHERE codigo_barras IS NOT NULL AND codigo_barras <> '';

CREATE UNIQUE INDEX IF NOT EXISTS uq_produtos_codigo_interno
    ON produtos (codigo_interno) WHERE codigo_interno IS NOT NULL AND codigo_interno <> '';

CREATE UNIQUE INDEX IF NOT EXISTS uq_produtos_referencia
    ON produtos (referencia) WHERE referencia IS NOT NULL AND referencia <> '';

CREATE INDEX IF NOT EXISTS idx_produtos_grupo ON produtos (grupo_id);
CREATE INDEX IF NOT EXISTS idx_produtos_ativo ON produtos (ativo);

-- ================================================================
-- PARTE 6: INFORMAÇÕES FISCAIS DO PRODUTO
-- ================================================================

CREATE TABLE IF NOT EXISTS produtos_fiscal (
    produto_id                  UUID    PRIMARY KEY REFERENCES produtos(id) ON DELETE CASCADE,
    -- Brasil
    ncm                         TEXT,
    regra_tributaria_brasil     TEXT,   -- referência para fase fiscal futura
    cest                        TEXT,
    -- Paraguai
    iva_tipo                    TEXT    DEFAULT 'IVA_10'
                                    CHECK (iva_tipo IN ('ISENTO','IVA_5','IVA_10')),
    regra_tributaria_paraguai   TEXT,   -- referência para fase fiscal futura
    -- Geral
    pais_fiscal_referencia      TEXT    NOT NULL DEFAULT 'BR'
                                    CHECK (pais_fiscal_referencia IN ('BR','PY','AMBOS')),
    criado_em                   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em               TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ================================================================
-- PARTE 7: HISTÓRICO DE PREÇOS
-- ================================================================

CREATE TABLE IF NOT EXISTS produtos_historico_precos (
    id                      BIGSERIAL       PRIMARY KEY,
    produto_id              UUID            NOT NULL REFERENCES produtos(id) ON DELETE CASCADE,
    preco_custo_anterior    NUMERIC(15,4),
    preco_custo_novo        NUMERIC(15,4),
    preco_venda_anterior    NUMERIC(15,4),
    preco_venda_novo        NUMERIC(15,4),
    margem_anterior         NUMERIC(7,4),
    margem_nova             NUMERIC(7,4),
    usuario_id              UUID            REFERENCES usuarios(id),
    motivo                  TEXT,
    criado_em               TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_historico_precos_produto
    ON produtos_historico_precos (produto_id, criado_em DESC);

-- ================================================================
-- PARTE 8: PIZZA — SABORES
-- ================================================================

CREATE TABLE IF NOT EXISTS pizza_sabores (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    nome        TEXT        NOT NULL,
    descricao   TEXT,
    ativo       BOOLEAN     NOT NULL DEFAULT TRUE,
    criado_em   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Preços por sabor + produto pizza (cada tamanho é um produto distinto)
CREATE TABLE IF NOT EXISTS pizza_sabores_precos (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    sabor_id            UUID            NOT NULL REFERENCES pizza_sabores(id) ON DELETE CASCADE,
    produto_pizza_id    UUID            NOT NULL REFERENCES produtos(id) ON DELETE CASCADE,
    preco_venda         NUMERIC(15,4)   NOT NULL DEFAULT 0 CHECK (preco_venda >= 0),
    ativo               BOOLEAN         NOT NULL DEFAULT TRUE,
    criado_em           TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    UNIQUE (sabor_id, produto_pizza_id)
);

-- ================================================================
-- PARTE 9: COMBOS (composição de produtos)
-- ================================================================

CREATE TABLE IF NOT EXISTS produtos_combos (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    combo_id            UUID            NOT NULL REFERENCES produtos(id) ON DELETE CASCADE,
    produto_item_id     UUID            NOT NULL REFERENCES produtos(id),
    quantidade          NUMERIC(10,4)   NOT NULL DEFAULT 1 CHECK (quantidade > 0),
    valor_original_item NUMERIC(15,4),
    valor_combo_item    NUMERIC(15,4)   NOT NULL DEFAULT 0 CHECK (valor_combo_item >= 0),
    ativo               BOOLEAN         NOT NULL DEFAULT TRUE,
    criado_em           TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    UNIQUE (combo_id, produto_item_id)
);

-- ================================================================
-- PARTE 10: ADICIONAIS
-- ================================================================

CREATE TABLE IF NOT EXISTS adicionais (
    id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    nome            TEXT            NOT NULL,
    descricao       TEXT,
    preco_adicional NUMERIC(15,4)   NOT NULL DEFAULT 0 CHECK (preco_adicional >= 0),
    ativo           BOOLEAN         NOT NULL DEFAULT TRUE,
    criado_em       TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    atualizado_em   TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

-- Vínculo produto <-> adicionais permitidos
CREATE TABLE IF NOT EXISTS produtos_adicionais (
    produto_id      UUID    NOT NULL REFERENCES produtos(id) ON DELETE CASCADE,
    adicional_id    UUID    NOT NULL REFERENCES adicionais(id) ON DELETE CASCADE,
    obrigatorio     BOOLEAN NOT NULL DEFAULT FALSE,
    ativo           BOOLEAN NOT NULL DEFAULT TRUE,
    criado_em       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (produto_id, adicional_id)
);
