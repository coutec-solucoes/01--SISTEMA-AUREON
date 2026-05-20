-- Migration 008: Delivery Operacional e Entregadores

-- 1. Tabela de Cache de Entregadores (Master Data)
CREATE TABLE IF NOT EXISTS entregadores_cache (
    id TEXT PRIMARY KEY,
    nome TEXT NOT NULL,
    documento TEXT,
    ativo INTEGER NOT NULL DEFAULT 1 CHECK (ativo IN (0, 1)),
    criado_em TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_entregadores_ativo ON entregadores_cache(ativo);

-- 2. Tabela de Delivery Operacional
CREATE TABLE IF NOT EXISTS delivery_operacional (
    id TEXT PRIMARY KEY,
    numero_pedido INTEGER NOT NULL,
    cliente_id TEXT,
    nome_cliente_informal TEXT NOT NULL,
    telefone TEXT NOT NULL,
    endereco_completo TEXT,
    tipo_pedido TEXT NOT NULL CHECK (tipo_pedido IN ('RETIRADA', 'ENTREGA')),
    status TEXT NOT NULL CHECK (status IN ('NOVO', 'ACEITO', 'PREPARANDO', 'PRONTO', 'DESPACHADO', 'FECHADO', 'CANCELADO')),
    origem TEXT NOT NULL CHECK (origem IN ('LOCAL', 'ONLINE')),
    entregador_id TEXT,
    taxa_entrega_minor INTEGER NOT NULL DEFAULT 0 CHECK (taxa_entrega_minor >= 0),
    total_consumo_minor INTEGER NOT NULL DEFAULT 0 CHECK (total_consumo_minor >= 0),
    sessao_caixa_id TEXT,
    observacao TEXT,
    previsao_entrega TEXT,
    aberto_em TEXT NOT NULL,
    fechado_em TEXT
);

CREATE INDEX IF NOT EXISTS idx_delivery_status ON delivery_operacional(status);
CREATE INDEX IF NOT EXISTS idx_delivery_origem ON delivery_operacional(origem);
CREATE INDEX IF NOT EXISTS idx_delivery_tipo ON delivery_operacional(tipo_pedido);
CREATE INDEX IF NOT EXISTS idx_delivery_entregador ON delivery_operacional(entregador_id);
CREATE INDEX IF NOT EXISTS idx_delivery_numero ON delivery_operacional(numero_pedido);

-- 3. Tabela de Itens do Delivery
CREATE TABLE IF NOT EXISTS delivery_itens (
    id TEXT PRIMARY KEY,
    delivery_id TEXT NOT NULL,
    produto_id TEXT NOT NULL,
    descricao_produto TEXT NOT NULL,
    codigo_produto TEXT,
    quantidade_escala3 INTEGER NOT NULL CHECK (quantidade_escala3 > 0),
    preco_unitario_minor INTEGER NOT NULL CHECK (preco_unitario_minor >= 0),
    desconto_item_minor INTEGER NOT NULL DEFAULT 0 CHECK (desconto_item_minor >= 0),
    acrescimo_item_minor INTEGER NOT NULL DEFAULT 0 CHECK (acrescimo_item_minor >= 0),
    total_item_minor INTEGER NOT NULL CHECK (total_item_minor >= 0),
    observacao_producao TEXT,
    local_producao_id TEXT,
    status TEXT NOT NULL DEFAULT 'PENDENTE',
    enviado_producao INTEGER NOT NULL DEFAULT 0 CHECK (enviado_producao IN (0, 1)),
    enviado_producao_em TEXT,
    cancelado INTEGER NOT NULL DEFAULT 0 CHECK (cancelado IN (0, 1)),
    cancelado_em TEXT,
    motivo_cancelamento TEXT,
    usuario_cancelamento_id TEXT,
    supervisor_id TEXT,
    autorizacao_id TEXT,
    criado_em TEXT NOT NULL,
    FOREIGN KEY (delivery_id) REFERENCES delivery_operacional(id)
);

CREATE INDEX IF NOT EXISTS idx_delivery_itens_delivery ON delivery_itens(delivery_id);
CREATE INDEX IF NOT EXISTS idx_delivery_itens_status ON delivery_itens(status);

-- 4. Alteração na tabela Vendas (Adicionando taxa_entrega_minor se não existir)
-- O SQLite tem suporte a ADD COLUMN via ALTER TABLE
-- A aplicação pode executar isso de forma segura se a coluna não existir, mas o `rusqlite_migration`
-- já trata a execução sequencial. Vamos adicionar a coluna.
-- Como é uma tabela já existente, é seguro adicionar uma coluna DEFAULT 0.
ALTER TABLE vendas ADD COLUMN taxa_entrega_minor INTEGER NOT NULL DEFAULT 0 CHECK (taxa_entrega_minor >= 0);
