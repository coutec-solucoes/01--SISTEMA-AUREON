-- ================================================================
-- MIGRATION SQLite 003 — Nucleo Operacional de Venda
-- Versão: 3
-- Projeto: Aureon Sistema Inteligente — Fase 7
-- REGRAS: nomes em portugues, snake_case, sem acentos
-- ================================================================
--
-- Esta migration adiciona:
-- 1. Controle de numeracao sequencial de vendas
-- 2. Sessoes de caixa (turno de trabalho)
-- 3. Vendas e itens de venda
-- 4. Pagamentos multimoeda
--
-- ================================================================

-- ================================================================
-- PARTE 1: CONTROLE DE NUMERACAO LOCAL
-- Garante numero sequencial unico por terminal, independente de rede
-- ================================================================

CREATE TABLE IF NOT EXISTS controle_numeracao (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    proximo_numero  INTEGER NOT NULL DEFAULT 1,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO controle_numeracao (id, proximo_numero) VALUES (1, 1);

-- ================================================================
-- PARTE 2: SESSOES DE CAIXA
-- Representa um turno de trabalho: abertura ate fechamento
-- ================================================================

CREATE TABLE IF NOT EXISTS sessoes_caixa (
    id               TEXT    PRIMARY KEY,  -- UUID v4
    registradora_id  TEXT    NOT NULL,
    usuario_id       TEXT    NOT NULL,
    status           TEXT    NOT NULL DEFAULT 'ABERTO'
                             CHECK (status IN ('ABERTO','FECHADO','SUSPENSO')),
    valor_abertura   REAL    NOT NULL DEFAULT 0.0,
    valor_fechamento REAL,
    aberto_em        TEXT    NOT NULL DEFAULT (datetime('now')),
    fechado_em       TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessoes_caixa_status
    ON sessoes_caixa(status);

CREATE INDEX IF NOT EXISTS idx_sessoes_caixa_registradora
    ON sessoes_caixa(registradora_id, status);

-- ================================================================
-- PARTE 3: VENDAS
-- Cabecalho da venda — cada linha eh uma transacao de venda completa
-- ================================================================

CREATE TABLE IF NOT EXISTS vendas (
    id               TEXT    PRIMARY KEY,  -- UUID v4
    numero_venda     INTEGER NOT NULL,
    sessao_caixa_id  TEXT    NOT NULL REFERENCES sessoes_caixa(id),
    usuario_id       TEXT    NOT NULL,
    status           TEXT    NOT NULL DEFAULT 'EM_ANDAMENTO'
                             CHECK (status IN ('EM_ANDAMENTO','FINALIZADA','CANCELADA')),
    tipo_venda       TEXT    NOT NULL DEFAULT 'BALCAO'
                             CHECK (tipo_venda IN ('BALCAO','PRE_VENDA','MESA','COMANDA','DELIVERY')),
    cliente_id       TEXT,
    observacao       TEXT,
    subtotal         REAL    NOT NULL DEFAULT 0.0,
    desconto_total   REAL    NOT NULL DEFAULT 0.0,
    acrescimo_total  REAL    NOT NULL DEFAULT 0.0,
    total            REAL    NOT NULL DEFAULT 0.0,
    criado_em        TEXT    NOT NULL DEFAULT (datetime('now')),
    atualizado_em    TEXT    NOT NULL DEFAULT (datetime('now')),
    finalizado_em    TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_vendas_numero
    ON vendas(numero_venda);

CREATE INDEX IF NOT EXISTS idx_vendas_status
    ON vendas(status);

CREATE INDEX IF NOT EXISTS idx_vendas_sessao
    ON vendas(sessao_caixa_id);

CREATE INDEX IF NOT EXISTS idx_vendas_criado_em
    ON vendas(criado_em DESC);

-- ================================================================
-- PARTE 4: ITENS DE VENDA
-- Cada produto adicionado a uma venda gera um item
-- ================================================================

CREATE TABLE IF NOT EXISTS venda_itens (
    id               TEXT    PRIMARY KEY,  -- UUID v4
    venda_id         TEXT    NOT NULL REFERENCES vendas(id),
    produto_id       TEXT    NOT NULL,
    descricao_produto TEXT   NOT NULL,
    codigo_produto   TEXT,
    codigo_barras    TEXT,
    quantidade       REAL    NOT NULL DEFAULT 1.0
                             CHECK (quantidade > 0),
    preco_unitario   REAL    NOT NULL
                             CHECK (preco_unitario >= 0),
    desconto_item    REAL    NOT NULL DEFAULT 0.0
                             CHECK (desconto_item >= 0),
    acrescimo_item   REAL    NOT NULL DEFAULT 0.0
                             CHECK (acrescimo_item >= 0),
    total_item       REAL    NOT NULL
                             CHECK (total_item >= 0),
    cancelado        INTEGER NOT NULL DEFAULT 0
                             CHECK (cancelado IN (0, 1)),
    cancelado_em     TEXT,
    criado_em        TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_venda_itens_venda
    ON venda_itens(venda_id, cancelado);

CREATE INDEX IF NOT EXISTS idx_venda_itens_produto
    ON venda_itens(produto_id);

-- ================================================================
-- PARTE 5: PAGAMENTOS DA VENDA
-- Suporta multiplos pagamentos e multiplas moedas por venda
-- ================================================================

CREATE TABLE IF NOT EXISTS venda_pagamentos (
    id               TEXT    PRIMARY KEY,  -- UUID v4
    venda_id         TEXT    NOT NULL REFERENCES vendas(id),
    forma_pagamento  TEXT    NOT NULL
                             CHECK (forma_pagamento IN (
                                 'DINHEIRO','CARTAO_DEBITO','CARTAO_CREDITO',
                                 'PIX','MULTIMOEDA','VALE','CREDITO_CLIENTE'
                             )),
    moeda_codigo     TEXT    NOT NULL DEFAULT 'BRL',
    valor_informado  REAL    NOT NULL CHECK (valor_informado > 0),
    valor_convertido REAL    NOT NULL CHECK (valor_convertido > 0),
    taxa_cambio      REAL    NOT NULL DEFAULT 1.0 CHECK (taxa_cambio > 0),
    troco            REAL    NOT NULL DEFAULT 0.0 CHECK (troco >= 0),
    criado_em        TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_venda_pagamentos_venda
    ON venda_pagamentos(venda_id);
