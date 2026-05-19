-- Migration 005 - Fase 8 - PDV Operacional

-- Tabela de Movimentações Extras de Caixa (Sangria, Suprimento, Vale, etc)
CREATE TABLE IF NOT EXISTS caixa_movimentacoes (
    id TEXT PRIMARY KEY,
    sessao_caixa_id TEXT NOT NULL,
    usuario_id TEXT NOT NULL,
    tipo_movimentacao TEXT NOT NULL, -- SUPRIMENTO, SANGRIA, VALE_FUNCIONARIO, AJUSTE_ENTRADA, AJUSTE_SAIDA
    moeda_codigo TEXT NOT NULL,
    valor_minor INTEGER NOT NULL,
    motivo TEXT,
    funcionario_id TEXT,
    supervisor_id TEXT,
    autorizacao_id TEXT,
    cancelado BOOLEAN NOT NULL DEFAULT 0,
    cancelado_em TEXT,
    usuario_cancelamento_id TEXT,
    motivo_cancelamento TEXT,
    criado_em TEXT NOT NULL,
    FOREIGN KEY (sessao_caixa_id) REFERENCES sessoes_caixa (id)
);
CREATE INDEX IF NOT EXISTS idx_caixa_movimentacoes_sessao ON caixa_movimentacoes(sessao_caixa_id);
CREATE INDEX IF NOT EXISTS idx_caixa_movimentacoes_tipo ON caixa_movimentacoes(tipo_movimentacao);

-- Tabela de Autorizações Locais de Supervisor
CREATE TABLE IF NOT EXISTS supervisor_autorizacoes_local (
    id TEXT PRIMARY KEY,
    operacao TEXT NOT NULL, -- CANCELAMENTO_VENDA, SANGRIA, DESCONTO_LIMITE, etc.
    usuario_solicitante_id TEXT NOT NULL,
    supervisor_id TEXT NOT NULL,
    motivo TEXT,
    aprovado BOOLEAN NOT NULL,
    criado_em TEXT NOT NULL,
    terminal_id TEXT NOT NULL,
    sessao_caixa_id TEXT,
    entidade_tipo TEXT, -- VENDA, ITEM, CAIXA_MOVIMENTACAO
    entidade_id TEXT
);
CREATE INDEX IF NOT EXISTS idx_sup_auth_supervisor ON supervisor_autorizacoes_local(supervisor_id);
CREATE INDEX IF NOT EXISTS idx_sup_auth_entidade ON supervisor_autorizacoes_local(entidade_tipo, entidade_id);

-- Tabela de Auditoria de Reimpressões
CREATE TABLE IF NOT EXISTS vendas_reimpressoes (
    id TEXT PRIMARY KEY,
    venda_id TEXT NOT NULL,
    usuario_id TEXT NOT NULL,
    motivo TEXT,
    supervisor_id TEXT,
    criado_em TEXT NOT NULL,
    FOREIGN KEY (venda_id) REFERENCES vendas (id)
);
CREATE INDEX IF NOT EXISTS idx_vendas_reimpressoes_venda ON vendas_reimpressoes(venda_id);

-- Tabelas de Cache de Pré-Vendas
CREATE TABLE IF NOT EXISTS pre_vendas_cache (
    id TEXT PRIMARY KEY,
    codigo TEXT NOT NULL,
    cliente_id TEXT,
    status TEXT NOT NULL, -- ABERTA, ENVIADA_PDV, CONVERTIDA, CANCELADA, EXPIRADA
    total_minor INTEGER NOT NULL,
    moeda_codigo TEXT NOT NULL,
    observacao TEXT,
    validade_em TEXT,
    criado_em TEXT NOT NULL,
    atualizado_em TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_pre_vendas_cache_codigo ON pre_vendas_cache(codigo);
CREATE INDEX IF NOT EXISTS idx_pre_vendas_cache_status ON pre_vendas_cache(status);

CREATE TABLE IF NOT EXISTS pre_vendas_itens_cache (
    id TEXT PRIMARY KEY,
    pre_venda_id TEXT NOT NULL,
    produto_id TEXT NOT NULL,
    descricao_produto TEXT NOT NULL,
    quantidade_escala3 INTEGER NOT NULL,
    preco_unitario_minor INTEGER NOT NULL,
    desconto_item_minor INTEGER NOT NULL DEFAULT 0,
    total_item_minor INTEGER NOT NULL,
    FOREIGN KEY (pre_venda_id) REFERENCES pre_vendas_cache (id)
);
CREATE INDEX IF NOT EXISTS idx_pre_vendas_itens_venda ON pre_vendas_itens_cache(pre_venda_id);

-- Tabelas de Cache de Orçamentos
CREATE TABLE IF NOT EXISTS orcamentos_cache (
    id TEXT PRIMARY KEY,
    codigo TEXT NOT NULL,
    cliente_id TEXT,
    status TEXT NOT NULL, -- ABERTO, ENVIADO_PDV, CONVERTIDO, CANCELADO, EXPIRADO
    total_minor INTEGER NOT NULL,
    moeda_codigo TEXT NOT NULL,
    validade_em TEXT,
    observacao TEXT,
    criado_em TEXT NOT NULL,
    atualizado_em TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_orcamentos_cache_codigo ON orcamentos_cache(codigo);
CREATE INDEX IF NOT EXISTS idx_orcamentos_cache_status ON orcamentos_cache(status);

CREATE TABLE IF NOT EXISTS orcamentos_itens_cache (
    id TEXT PRIMARY KEY,
    orcamento_id TEXT NOT NULL,
    produto_id TEXT NOT NULL,
    descricao_produto TEXT NOT NULL,
    quantidade_escala3 INTEGER NOT NULL,
    preco_unitario_minor INTEGER NOT NULL,
    desconto_item_minor INTEGER NOT NULL DEFAULT 0,
    total_item_minor INTEGER NOT NULL,
    FOREIGN KEY (orcamento_id) REFERENCES orcamentos_cache (id)
);
CREATE INDEX IF NOT EXISTS idx_orcamentos_itens_orcamento ON orcamentos_itens_cache(orcamento_id);

-- Alterações na tabela vendas
ALTER TABLE vendas ADD COLUMN cliente_id TEXT;
ALTER TABLE vendas ADD COLUMN origem_tipo TEXT;
ALTER TABLE vendas ADD COLUMN origem_id TEXT;
ALTER TABLE vendas ADD COLUMN comprovante_gerado_em TEXT;

CREATE INDEX IF NOT EXISTS idx_vendas_cliente ON vendas(cliente_id);
CREATE INDEX IF NOT EXISTS idx_vendas_origem ON vendas(origem_tipo, origem_id);

-- Alterações na tabela venda_itens
ALTER TABLE venda_itens ADD COLUMN origem_item_id TEXT;
