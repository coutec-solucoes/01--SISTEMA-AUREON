-- ================================================================
-- MIGRATION SQLite 004 — Correcao Financeira do Nucleo de Venda
-- Versao: 4
-- Projeto: Aureon Sistema Inteligente — Fase 7 (correcao obrigatoria)
-- ================================================================
--
-- MOTIVO: Migration 003 usou REAL/f64 para valores monetarios.
--         Valores monetarios NUNCA devem usar ponto flutuante.
--
-- CONVENCAO DE UNIDADE MENOR (_minor):
--   BRL: centavos   (R$ 10,50 -> 1050)
--   USD: cents      ($10.50   -> 1050)
--   PYG: guaranis   (Gs 10500 -> 10500)
--   taxa de cambio: escala 1_000_000
--     (ex: 1 USD = 5.52 BRL -> 5_520_000)
--
-- ESTRATEGIA: DROP das tabelas da migration 003 e recriacao correta.
--   Seguro pois o sistema ainda nao entrou em producao.
-- ================================================================

-- ================================================================
-- PASSO 1: REMOVER TABELAS COM TIPOS INCORRETOS (migration 003)
-- ================================================================

DROP TABLE IF EXISTS venda_pagamentos;
DROP TABLE IF EXISTS venda_itens;
DROP TABLE IF EXISTS vendas;
DROP TABLE IF EXISTS sessoes_caixa;
DROP TABLE IF EXISTS controle_numeracao;

-- ================================================================
-- PASSO 2: CONTROLE DE NUMERACAO (sem alteracao)
-- ================================================================

CREATE TABLE IF NOT EXISTS controle_numeracao (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    proximo_numero  INTEGER NOT NULL DEFAULT 1,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO controle_numeracao (id, proximo_numero) VALUES (1, 1);

-- ================================================================
-- PASSO 3: SESSOES DE CAIXA (sem valores monetarios diretos)
-- Valores ficam em sessoes_caixa_moedas para suporte multimoeda
-- ================================================================

CREATE TABLE IF NOT EXISTS sessoes_caixa (
    id               TEXT    PRIMARY KEY,  -- UUID v4
    registradora_id  TEXT    NOT NULL,
    usuario_id       TEXT    NOT NULL,
    status           TEXT    NOT NULL DEFAULT 'ABERTO'
                             CHECK (status IN ('ABERTO','FECHADO','SUSPENSO')),
    aberto_em        TEXT    NOT NULL DEFAULT (datetime('now')),
    fechado_em       TEXT,
    observacao       TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessoes_caixa_status
    ON sessoes_caixa(status);

CREATE INDEX IF NOT EXISTS idx_sessoes_caixa_registradora
    ON sessoes_caixa(registradora_id, status);

-- ================================================================
-- PASSO 4: SALDO DE CAIXA POR MOEDA
-- Uma sessao pode ter saldo em multiplas moedas
-- ================================================================

CREATE TABLE IF NOT EXISTS sessoes_caixa_moedas (
    id                             TEXT    PRIMARY KEY,  -- UUID v4
    sessao_id                      TEXT    NOT NULL REFERENCES sessoes_caixa(id),
    moeda_codigo                   TEXT    NOT NULL,
    valor_abertura_minor           INTEGER NOT NULL DEFAULT 0,
    valor_fechamento_informado_minor INTEGER,           -- NULL se nao fechado
    valor_esperado_minor           INTEGER,             -- apurado dos pagamentos
    diferenca_minor                INTEGER,             -- informado - esperado
    UNIQUE(sessao_id, moeda_codigo)
);

CREATE INDEX IF NOT EXISTS idx_sessoes_caixa_moedas_sessao
    ON sessoes_caixa_moedas(sessao_id);

-- ================================================================
-- PASSO 5: VENDAS
-- numero_venda NULL ate finalizacao; valores em INTEGER (minor unit)
-- ================================================================

CREATE TABLE IF NOT EXISTS vendas (
    id                       TEXT    PRIMARY KEY,  -- UUID v4
    numero_venda             INTEGER,              -- NULL ate finalizar; preenchido na finalizacao
    sessao_caixa_id          TEXT    NOT NULL REFERENCES sessoes_caixa(id),
    usuario_id               TEXT    NOT NULL,
    status                   TEXT    NOT NULL DEFAULT 'EM_ANDAMENTO'
                                     CHECK (status IN ('EM_ANDAMENTO','FINALIZADA','CANCELADA')),
    tipo_venda               TEXT    NOT NULL DEFAULT 'BALCAO'
                                     CHECK (tipo_venda IN ('BALCAO','PRE_VENDA','MESA','COMANDA','DELIVERY')),
    cliente_id               TEXT,
    observacao               TEXT,
    -- Valores em minor unit (centavos BRL, etc.)
    subtotal_minor           INTEGER NOT NULL DEFAULT 0,
    desconto_total_minor     INTEGER NOT NULL DEFAULT 0,
    acrescimo_total_minor    INTEGER NOT NULL DEFAULT 0,
    total_minor              INTEGER NOT NULL DEFAULT 0,
    -- Timestamps
    criado_em                TEXT    NOT NULL DEFAULT (datetime('now')),
    atualizado_em            TEXT    NOT NULL DEFAULT (datetime('now')),
    finalizado_em            TEXT,
    -- Rastreamento de cancelamento
    cancelado_em             TEXT,
    usuario_cancelamento_id  TEXT,
    motivo_cancelamento      TEXT,
    supervisor_id            TEXT,
    autorizacao_id           TEXT
);

-- numero_venda unico somente quando nao-nulo
CREATE UNIQUE INDEX IF NOT EXISTS uq_vendas_numero
    ON vendas(numero_venda) WHERE numero_venda IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_vendas_status      ON vendas(status);
CREATE INDEX IF NOT EXISTS idx_vendas_sessao      ON vendas(sessao_caixa_id);
CREATE INDEX IF NOT EXISTS idx_vendas_criado_em   ON vendas(criado_em DESC);

-- ================================================================
-- PASSO 6: ITENS DE VENDA (valores em minor unit)
-- ================================================================

CREATE TABLE IF NOT EXISTS venda_itens (
    id                       TEXT    PRIMARY KEY,  -- UUID v4
    venda_id                 TEXT    NOT NULL REFERENCES vendas(id),
    produto_id               TEXT    NOT NULL,
    descricao_produto        TEXT    NOT NULL,
    codigo_produto           TEXT,
    codigo_barras            TEXT,
    -- Quantidade: escala 3 casas decimais armazenada como INTEGER * 1000
    -- (ex: 1.500 kg -> 1500)
    quantidade_escala3       INTEGER NOT NULL DEFAULT 1000
                                     CHECK (quantidade_escala3 > 0),
    -- Valores em minor unit
    preco_unitario_minor     INTEGER NOT NULL CHECK (preco_unitario_minor >= 0),
    desconto_item_minor      INTEGER NOT NULL DEFAULT 0 CHECK (desconto_item_minor >= 0),
    acrescimo_item_minor     INTEGER NOT NULL DEFAULT 0 CHECK (acrescimo_item_minor >= 0),
    total_item_minor         INTEGER NOT NULL CHECK (total_item_minor >= 0),
    -- Cancelamento
    cancelado                INTEGER NOT NULL DEFAULT 0 CHECK (cancelado IN (0, 1)),
    cancelado_em             TEXT,
    usuario_cancelamento_id  TEXT,
    motivo_cancelamento      TEXT,
    supervisor_id            TEXT,
    autorizacao_id           TEXT,
    -- Auditoria
    criado_em                TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_venda_itens_venda    ON venda_itens(venda_id, cancelado);
CREATE INDEX IF NOT EXISTS idx_venda_itens_produto  ON venda_itens(produto_id);

-- ================================================================
-- PASSO 7: PAGAMENTOS DA VENDA (valores em minor unit, taxa escalada)
-- ================================================================

CREATE TABLE IF NOT EXISTS venda_pagamentos (
    id                                TEXT    PRIMARY KEY,  -- UUID v4
    venda_id                          TEXT    NOT NULL REFERENCES vendas(id),
    forma_pagamento                   TEXT    NOT NULL
                                              CHECK (forma_pagamento IN (
                                                  'DINHEIRO','CARTAO_DEBITO','CARTAO_CREDITO',
                                                  'PIX','MULTIMOEDA','VALE','CREDITO_CLIENTE'
                                              )),
    -- Moeda informada pelo operador
    moeda_codigo                      TEXT    NOT NULL DEFAULT 'BRL',
    valor_informado_minor             INTEGER NOT NULL CHECK (valor_informado_minor > 0),
    -- Conversao para moeda principal
    moeda_principal_codigo            TEXT    NOT NULL,
    valor_convertido_minor            INTEGER NOT NULL CHECK (valor_convertido_minor > 0),
    -- Taxa travada no momento do pagamento (escala 1_000_000)
    -- Ex: 1 USD = 5.52 BRL -> taxa_cambio_escala6 = 5_520_000
    taxa_cambio_escala6               INTEGER NOT NULL DEFAULT 1000000,
    data_cotacao_usada                TEXT    NOT NULL,   -- datetime da cotacao usada
    -- Troco
    troco_minor                       INTEGER NOT NULL DEFAULT 0 CHECK (troco_minor >= 0),
    moeda_troco_codigo                TEXT,
    -- Auditoria
    criado_em                         TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_venda_pagamentos_venda
    ON venda_pagamentos(venda_id);

-- ================================================================
-- PASSO 8: INDICES ADICIONAIS
-- ================================================================

CREATE INDEX IF NOT EXISTS idx_sessoes_caixa_usuario
    ON sessoes_caixa(usuario_id, status);
