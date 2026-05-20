-- ================================================================
-- MIGRATION SQLite 011 — Financeiro Base
-- Versao: 11
-- Projeto: Aureon Sistema Inteligente — Fase 13
-- ================================================================

-- 1. TABELA: contas_pagar
CREATE TABLE IF NOT EXISTS contas_pagar (
    id                             TEXT    PRIMARY KEY, -- UUID v4
    fornecedor_id                  TEXT,                -- Nullable para despesas manuais
    fornecedor_nome_snapshot       TEXT,                -- Histórico de nome do fornecedor
    compra_id                      TEXT,                -- Nullable se for despesa manual
    descricao                      TEXT    NOT NULL,    -- Ex: "Aluguel", "Compra nota 123"
    moeda_codigo                   TEXT    NOT NULL,    -- Moeda original do titulo (ex: BRL, USD)
    valor_original_minor           INTEGER NOT NULL CHECK(valor_original_minor >= 0),
    taxa_cambio_escala6            INTEGER NOT NULL CHECK(taxa_cambio_escala6 > 0),
    valor_original_principal_minor INTEGER NOT NULL CHECK(valor_original_principal_minor >= 0), -- Convertido em BRL na emissao
    data_emissao                   TEXT    NOT NULL,    -- Formato yyyy-MM-dd HH:mm:ss
    data_vencimento                TEXT    NOT NULL,    -- Formato yyyy-MM-dd HH:mm:ss
    status                         TEXT    NOT NULL DEFAULT 'PENDENTE'
                                           CHECK(status IN ('PENDENTE', 'PAGO_PARCIAL', 'PAGO', 'CANCELADO')),
    saldo_pendente_minor           INTEGER NOT NULL CHECK(saldo_pendente_minor >= 0),
    criado_em                      TEXT    NOT NULL,
    atualizado_em                  TEXT    NOT NULL,
    usuario_id                     TEXT    NOT NULL,
    observacao                     TEXT,
    FOREIGN KEY (fornecedor_id) REFERENCES fornecedores_cache(id),
    FOREIGN KEY (compra_id) REFERENCES compras(id)
);

-- Indices contas_pagar
CREATE INDEX IF NOT EXISTS idx_contas_pagar_status ON contas_pagar(status);
CREATE INDEX IF NOT EXISTS idx_contas_pagar_fornecedor ON contas_pagar(fornecedor_id);
CREATE INDEX IF NOT EXISTS idx_contas_pagar_compra ON contas_pagar(compra_id);
CREATE INDEX IF NOT EXISTS idx_contas_pagar_vencimento ON contas_pagar(data_vencimento);

-- 2. TABELA: contas_receber
CREATE TABLE IF NOT EXISTS contas_receber (
    id                             TEXT    PRIMARY KEY, -- UUID v4
    cliente_id                     TEXT,                -- Obrigatório para crediários, Nullable estruturalmente
    cliente_nome_snapshot          TEXT,                -- Histórico de nome do cliente
    venda_id                       TEXT,                -- Link com a venda geradora
    descricao                      TEXT    NOT NULL,    -- Ex: "Venda crediário #1002"
    moeda_codigo                   TEXT    NOT NULL,    -- Moeda original (geralmente BRL)
    valor_original_minor           INTEGER NOT NULL CHECK(valor_original_minor >= 0),
    taxa_cambio_escala6            INTEGER NOT NULL CHECK(taxa_cambio_escala6 > 0),
    valor_original_principal_minor INTEGER NOT NULL CHECK(valor_original_principal_minor >= 0),
    data_emissao                   TEXT    NOT NULL,    -- Formato yyyy-MM-dd HH:mm:ss
    data_vencimento                TEXT    NOT NULL,    -- Formato yyyy-MM-dd HH:mm:ss
    status                         TEXT    NOT NULL DEFAULT 'PENDENTE'
                                           CHECK(status IN ('PENDENTE', 'PAGO_PARCIAL', 'PAGO', 'CANCELADO')),
    saldo_pendente_minor           INTEGER NOT NULL CHECK(saldo_pendente_minor >= 0),
    criado_em                      TEXT    NOT NULL,
    atualizado_em                  TEXT    NOT NULL,
    usuario_id                     TEXT    NOT NULL,
    observacao                     TEXT,
    FOREIGN KEY (cliente_id) REFERENCES clientes_cache(id),
    FOREIGN KEY (venda_id) REFERENCES vendas(id)
);

-- Indices contas_receber
CREATE INDEX IF NOT EXISTS idx_contas_receber_status ON contas_receber(status);
CREATE INDEX IF NOT EXISTS idx_contas_receber_cliente ON contas_receber(cliente_id);
CREATE INDEX IF NOT EXISTS idx_contas_receber_venda ON contas_receber(venda_id);
CREATE INDEX IF NOT EXISTS idx_contas_receber_vencimento ON contas_receber(data_vencimento);

-- 3. TABELA: financeiro_lancamentos
CREATE TABLE IF NOT EXISTS financeiro_lancamentos (
    id                     TEXT    PRIMARY KEY, -- UUID v4
    conta_pagar_id         TEXT,                -- Null se for recebimento
    conta_receber_id       TEXT,                -- Null se for pagamento
    sessao_caixa_id        TEXT,                -- Sessao de caixa ativa do PDV
    tipo_lancamento        TEXT    NOT NULL CHECK(tipo_lancamento IN ('RECEBIMENTO', 'PAGAMENTO')),
    forma_pagamento        TEXT    NOT NULL CHECK(forma_pagamento IN (
                                                'DINHEIRO','CARTAO_DEBITO','CARTAO_CREDITO',
                                                'PIX','MULTIMOEDA','VALE','CREDITO_CLIENTE'
                                            )),
    moeda_codigo           TEXT    NOT NULL,    -- Moeda usada na baixa
    valor_informado_minor  INTEGER NOT NULL CHECK(valor_informado_minor > 0),
    taxa_cambio_escala6    INTEGER NOT NULL CHECK(taxa_cambio_escala6 > 0),
    valor_principal_minor  INTEGER NOT NULL CHECK(valor_principal_minor > 0), -- Equivalente em BRL
    data_pagamento         TEXT    NOT NULL,    -- Formato yyyy-MM-dd HH:mm:ss
    usuario_id             TEXT    NOT NULL,
    observacao             TEXT,
    criado_em              TEXT    NOT NULL,
    FOREIGN KEY (conta_pagar_id) REFERENCES contas_pagar(id),
    FOREIGN KEY (conta_receber_id) REFERENCES contas_receber(id),
    FOREIGN KEY (sessao_caixa_id) REFERENCES sessoes_caixa(id),
    -- Regra: deve apontar para contas_pagar OU contas_receber, nunca ambos
    CHECK (
        (conta_pagar_id IS NOT NULL AND conta_receber_id IS NULL) OR
        (conta_pagar_id IS NULL AND conta_receber_id IS NOT NULL)
    )
);

-- Indices financeiro_lancamentos
CREATE INDEX IF NOT EXISTS idx_fin_lanc_pagar ON financeiro_lancamentos(conta_pagar_id);
CREATE INDEX IF NOT EXISTS idx_fin_lanc_receber ON financeiro_lancamentos(conta_receber_id);
CREATE INDEX IF NOT EXISTS idx_fin_lanc_sessao ON financeiro_lancamentos(sessao_caixa_id);
CREATE INDEX IF NOT EXISTS idx_fin_lanc_data ON financeiro_lancamentos(data_pagamento);
