-- ============================================================================
-- Migration SQLite 007 — PDV Gourmet
-- ============================================================================

-- Cache de Mesas Configurados
CREATE TABLE IF NOT EXISTS mesas_cache (
    id TEXT PRIMARY KEY,
    numero INTEGER NOT NULL UNIQUE,
    nome TEXT NOT NULL,
    ativo BOOLEAN NOT NULL DEFAULT 1 CHECK(ativo IN (0, 1))
);

-- Cache de Comandas Configurados
CREATE TABLE IF NOT EXISTS comandas_cache (
    id TEXT PRIMARY KEY,
    numero INTEGER NOT NULL UNIQUE,
    ativo BOOLEAN NOT NULL DEFAULT 1 CHECK(ativo IN (0, 1))
);

-- Mesas Operacionais
CREATE TABLE IF NOT EXISTS mesas_operacionais (
    id TEXT PRIMARY KEY,
    mesa_numero INTEGER NOT NULL UNIQUE,
    nome_exibicao TEXT NOT NULL,
    cliente_nome_informal TEXT,
    cliente_id TEXT,
    status TEXT NOT NULL CHECK(status IN ('ABERTA', 'RESERVADA', 'BLOQUEADA', 'FECHADA', 'CANCELADA')),
    usuario_abertura_id TEXT NOT NULL,
    sessao_caixa_id TEXT NOT NULL,
    observacao TEXT,
    aberta_em TEXT NOT NULL,
    fechada_em TEXT,
    cancelada_em TEXT,
    usuario_cancelamento_id TEXT,
    motivo_cancelamento TEXT,
    supervisor_id TEXT,
    autorizacao_id TEXT
);

-- Comandas Operacionais
CREATE TABLE IF NOT EXISTS comandas_operacionais (
    id TEXT PRIMARY KEY,
    numero_comanda INTEGER NOT NULL UNIQUE,
    codigo_barras_qr TEXT,
    cliente_nome_informal TEXT,
    cliente_id TEXT,
    status TEXT NOT NULL CHECK(status IN ('ABERTA', 'BLOQUEADA', 'FECHADA', 'CANCELADA')),
    usuario_abertura_id TEXT NOT NULL,
    sessao_caixa_id TEXT NOT NULL,
    observacao TEXT,
    aberta_em TEXT NOT NULL,
    fechada_em TEXT,
    cancelada_em TEXT,
    usuario_cancelamento_id TEXT,
    motivo_cancelamento TEXT,
    supervisor_id TEXT,
    autorizacao_id TEXT
);

-- Itens Gourmet (Consumo de Mesa/Comanda)
CREATE TABLE IF NOT EXISTS gourmet_itens (
    id TEXT PRIMARY KEY,
    origem_tipo TEXT NOT NULL CHECK(origem_tipo IN ('MESA', 'COMANDA')),
    origem_id TEXT NOT NULL,
    produto_id TEXT NOT NULL,
    descricao_produto TEXT NOT NULL,
    codigo_produto TEXT NOT NULL,
    quantidade_escala3 INTEGER NOT NULL CHECK(quantidade_escala3 > 0),
    preco_unitario_minor INTEGER NOT NULL CHECK(preco_unitario_minor >= 0),
    desconto_item_minor INTEGER NOT NULL DEFAULT 0 CHECK(desconto_item_minor >= 0),
    acrescimo_item_minor INTEGER NOT NULL DEFAULT 0 CHECK(acrescimo_item_minor >= 0),
    total_item_minor INTEGER NOT NULL CHECK(total_item_minor >= 0),
    observacao_producao TEXT,
    local_producao_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('PENDENTE', 'ENVIADO_PRODUCAO', 'CANCELADO', 'TRANSFERIDO', 'FECHADO')),
    enviado_producao INTEGER NOT NULL DEFAULT 0 CHECK(enviado_producao IN (0, 1)),
    enviado_producao_em TEXT,
    cancelado INTEGER NOT NULL DEFAULT 0 CHECK(cancelado IN (0, 1)),
    cancelado_em TEXT,
    usuario_cancelamento_id TEXT,
    motivo_cancelamento TEXT,
    supervisor_id TEXT,
    autorizacao_id TEXT,
    criado_em TEXT NOT NULL
);

-- Transferências
CREATE TABLE IF NOT EXISTS gourmet_transferencias (
    id TEXT PRIMARY KEY,
    origem_tipo TEXT NOT NULL CHECK(origem_tipo IN ('MESA', 'COMANDA')),
    origem_id TEXT NOT NULL,
    destino_tipo TEXT NOT NULL CHECK(destino_tipo IN ('MESA', 'COMANDA')),
    destino_id TEXT NOT NULL,
    usuario_id TEXT NOT NULL,
    motivo TEXT NOT NULL,
    transferencia_total INTEGER NOT NULL DEFAULT 0 CHECK(transferencia_total IN (0, 1)),
    criado_em TEXT NOT NULL
);

-- Detalhes da Transferência de Itens
CREATE TABLE IF NOT EXISTS gourmet_transferencias_itens (
    transferencia_id TEXT NOT NULL,
    item_origem_id TEXT NOT NULL,
    quantidade_transferida_escala3 INTEGER NOT NULL CHECK(quantidade_transferida_escala3 > 0),
    item_destino_id TEXT NOT NULL,
    PRIMARY KEY (transferencia_id, item_origem_id),
    FOREIGN KEY (transferencia_id) REFERENCES gourmet_transferencias(id)
);

-- Envios de Produção Setorial
CREATE TABLE IF NOT EXISTS producao_envios (
    id TEXT PRIMARY KEY,
    origem_tipo TEXT NOT NULL CHECK(origem_tipo IN ('MESA', 'COMANDA')),
    origem_id TEXT NOT NULL,
    setor_producao_id TEXT NOT NULL,
    usuario_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('PENDENTE', 'GERADO', 'IMPRESSO_PREPARADO', 'CANCELADO')),
    criado_em TEXT NOT NULL,
    impresso_em TEXT,
    reimpresso_em TEXT
);

-- Itens Enviados para Produção
CREATE TABLE IF NOT EXISTS producao_envios_itens (
    envio_id TEXT NOT NULL,
    item_id TEXT NOT NULL,
    produto_id TEXT NOT NULL,
    descricao_produto TEXT NOT NULL,
    quantidade_escala3 INTEGER NOT NULL CHECK(quantidade_escala3 > 0),
    observacao_producao TEXT,
    cancelamento INTEGER NOT NULL DEFAULT 0 CHECK(cancelamento IN (0, 1)),
    criado_em TEXT NOT NULL,
    PRIMARY KEY (envio_id, item_id),
    FOREIGN KEY (envio_id) REFERENCES producao_envios(id)
);

-- Índices Recomendados
CREATE INDEX IF NOT EXISTS idx_mesas_operacionais_status ON mesas_operacionais(status);
CREATE INDEX IF NOT EXISTS idx_mesas_operacionais_numero ON mesas_operacionais(mesa_numero);

CREATE INDEX IF NOT EXISTS idx_comandas_operacionais_status ON comandas_operacionais(status);
CREATE INDEX IF NOT EXISTS idx_comandas_operacionais_numero ON comandas_operacionais(numero_comanda);

CREATE INDEX IF NOT EXISTS idx_gourmet_itens_origem ON gourmet_itens(origem_tipo, origem_id);
CREATE INDEX IF NOT EXISTS idx_gourmet_itens_status ON gourmet_itens(status);
CREATE INDEX IF NOT EXISTS idx_gourmet_itens_produto ON gourmet_itens(produto_id);

CREATE INDEX IF NOT EXISTS idx_gourmet_transferencias_origem ON gourmet_transferencias(origem_tipo, origem_id);
CREATE INDEX IF NOT EXISTS idx_gourmet_transferencias_destino ON gourmet_transferencias(destino_tipo, destino_id);

CREATE INDEX IF NOT EXISTS idx_producao_envios_origem ON producao_envios(origem_tipo, origem_id);
CREATE INDEX IF NOT EXISTS idx_producao_envios_setor ON producao_envios(setor_producao_id);
CREATE INDEX IF NOT EXISTS idx_producao_envios_criado ON producao_envios(criado_em);
