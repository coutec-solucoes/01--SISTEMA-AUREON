-- ================================================================
-- MIGRATION SQLite 002 — Cache de Sincronização e Dados Mestres
-- Versão: 2
-- Projeto: Aureon Sistema Inteligente — Fase 6
-- REGRAS: nomes em português, snake_case, sem acentos
-- ================================================================
--
-- DECISÕES DE REAPROVEITAMENTO:
--
-- [REAPROVEITADO] configuracoes_locais  → já existe na migration 001
-- [REAPROVEITADO] terminais             → já existe na migration 001
-- [REAPROVEITADO] logs_locais           → já existe na migration 001
-- [REAPROVEITADO] sync_outbox           → já existe na migration 001
-- [REAPROVEITADO] sync_inbox            → já existe na migration 001
-- [REAPROVEITADO] sync_logs             → já existe na migration 001
--
-- Esta migration adiciona:
-- 1. Controle local de terminal (dados de registro/autorizacao)
-- 2. Controle de versoes aplicadas
-- 3. Idempotencia local
-- 4. Caches de dados mestres (empresa, moedas, seguranca, produtos, config op.)
--
-- ================================================================

-- ================================================================
-- PARTE 1: CONTROLE DE TERMINAL LOCAL
-- ================================================================

CREATE TABLE IF NOT EXISTS terminal_local (
    id                       INTEGER PRIMARY KEY CHECK (id = 1), -- apenas 1 registro
    codigo_terminal          TEXT    NOT NULL,
    nome_terminal            TEXT    NOT NULL,
    tipo_terminal            TEXT    NOT NULL DEFAULT 'PDV',
    identificador_maquina    TEXT,   -- hash do hardware local
    chave_terminal           TEXT    NOT NULL,   -- token opaco recebido da API (armazenado criptografado em producao)
    autorizado               INTEGER NOT NULL DEFAULT 0,  -- 0=nao, 1=sim
    primeiro_sync_concluido  INTEGER NOT NULL DEFAULT 0,
    host_api                 TEXT    NOT NULL DEFAULT 'http://localhost:7000',
    porta_api                INTEGER NOT NULL DEFAULT 7000,
    criado_em                TEXT    NOT NULL DEFAULT (datetime('now')),
    atualizado_em            TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ================================================================
-- PARTE 2: CONTROLE DE VERSÕES APLICADAS
-- Registra qual versão de cada grupo de dados foi recebida e aplicada
-- ================================================================

CREATE TABLE IF NOT EXISTS sync_versoes_aplicadas (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    tipo_dado     TEXT    NOT NULL UNIQUE,   -- ex: 'produtos_catalogo'
    versao        INTEGER NOT NULL DEFAULT 0,
    hash_conteudo TEXT    NOT NULL DEFAULT '',
    pacote_id     TEXT,   -- UUID do pacote que trouxe esta versao
    aplicado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Seeds iniciais de controle de versão (todas iniciam em versao 0 = sem sync)
INSERT OR IGNORE INTO sync_versoes_aplicadas (tipo_dado, versao, hash_conteudo)
VALUES
    ('empresa_config',            0, 'sem_sync'),
    ('moedas_cotacoes',           0, 'sem_sync'),
    ('usuarios_permissoes',       0, 'sem_sync'),
    ('produtos_catalogo',         0, 'sem_sync'),
    ('produtos_precos',           0, 'sem_sync'),
    ('produtos_fiscal',           0, 'sem_sync'),
    ('produtos_complementos',     0, 'sem_sync'),
    ('configuracoes_operacionais',0, 'sem_sync'),
    ('dispositivos_perifericos',  0, 'sem_sync');

-- ================================================================
-- PARTE 3: IDEMPOTÊNCIA LOCAL
-- Evita reprocessamento duplicado de pacotes
-- ================================================================

CREATE TABLE IF NOT EXISTS sync_idempotencia_local (
    idempotency_key  TEXT PRIMARY KEY,
    operacao         TEXT NOT NULL,
    processado_em    TEXT NOT NULL DEFAULT (datetime('now')),
    resultado        TEXT  -- 'SUCESSO' ou 'ERRO: mensagem'
);

-- ================================================================
-- PARTE 4: CACHE DE EMPRESA
-- ================================================================

CREATE TABLE IF NOT EXISTS empresa_cache (
    id               INTEGER PRIMARY KEY CHECK (id = 1),
    empresa_id       TEXT    NOT NULL,
    codigo           TEXT    NOT NULL,
    nome             TEXT    NOT NULL,
    documento        TEXT,
    pais             TEXT    NOT NULL DEFAULT 'BR',
    idioma           TEXT    NOT NULL DEFAULT 'pt_BR',
    logo_url         TEXT,
    versao_dados     INTEGER NOT NULL DEFAULT 0,
    atualizado_em    TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS parametros_operacionais_cache (
    id                                    INTEGER PRIMARY KEY CHECK (id = 1),
    permitir_venda_offline                INTEGER NOT NULL DEFAULT 1,
    dias_maximos_offline                  INTEGER NOT NULL DEFAULT 7,
    permitir_venda_sem_estoque            INTEGER NOT NULL DEFAULT 1,
    bloquear_produto_vencido              INTEGER NOT NULL DEFAULT 1,
    permitir_desconto_pdv                 INTEGER NOT NULL DEFAULT 1,
    desconto_maximo_padrao_percentual     REAL    NOT NULL DEFAULT 10.0,
    exigir_supervisor_desconto_acima      INTEGER NOT NULL DEFAULT 1,
    exigir_supervisor_cancelamento_item   INTEGER NOT NULL DEFAULT 1,
    exigir_supervisor_cancelamento_venda  INTEGER NOT NULL DEFAULT 1,
    permitir_alterar_preco_pdv            INTEGER NOT NULL DEFAULT 0,
    permitir_cliente_sem_cadastro         INTEGER NOT NULL DEFAULT 1,
    versao_dados                          INTEGER NOT NULL DEFAULT 0,
    atualizado_em                         TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ================================================================
-- PARTE 5: CACHE DE MOEDAS E COTAÇÕES
-- ================================================================

CREATE TABLE IF NOT EXISTS moedas_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    moeda_id        TEXT    NOT NULL UNIQUE,
    codigo          TEXT    NOT NULL,   -- ex: BRL, USD, PYG
    simbolo         TEXT    NOT NULL,   -- ex: R$, $, Gs
    nome            TEXT    NOT NULL,
    casa_decimal    INTEGER NOT NULL DEFAULT 2,
    principal       INTEGER NOT NULL DEFAULT 0,
    ativa           INTEGER NOT NULL DEFAULT 1,
    ordem_exibicao  INTEGER NOT NULL DEFAULT 0,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS cotacoes_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    cotacao_id      TEXT    NOT NULL UNIQUE,
    moeda_origem    TEXT    NOT NULL,
    moeda_destino   TEXT    NOT NULL,
    taxa            REAL    NOT NULL,
    data_cotacao    TEXT    NOT NULL,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ================================================================
-- PARTE 6: CACHE DE SEGURANÇA (USUÁRIOS, PERFIS, PERMISSÕES)
-- ================================================================

CREATE TABLE IF NOT EXISTS usuarios_cache (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    usuario_id     TEXT    NOT NULL UNIQUE,
    nome           TEXT    NOT NULL,
    email          TEXT,
    login          TEXT    NOT NULL,
    -- NUNCA armazenar senha plain-text; hash bcrypt ou token de sessão temporário
    hash_senha     TEXT    NOT NULL,
    perfil_id      TEXT,
    ativo          INTEGER NOT NULL DEFAULT 1,
    versao_dados   INTEGER NOT NULL DEFAULT 0,
    atualizado_em  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS perfis_cache (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    perfil_id      TEXT    NOT NULL UNIQUE,
    nome           TEXT    NOT NULL,
    descricao      TEXT,
    ativo          INTEGER NOT NULL DEFAULT 1,
    versao_dados   INTEGER NOT NULL DEFAULT 0,
    atualizado_em  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS permissoes_cache (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    permissao_id   TEXT    NOT NULL UNIQUE,
    perfil_id      TEXT    NOT NULL,
    recurso        TEXT    NOT NULL,
    acao           TEXT    NOT NULL,
    permitido      INTEGER NOT NULL DEFAULT 1,
    versao_dados   INTEGER NOT NULL DEFAULT 0,
    atualizado_em  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS supervisores_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    supervisor_id   TEXT    NOT NULL UNIQUE,
    usuario_id      TEXT    NOT NULL,
    nome            TEXT    NOT NULL,
    -- senha de supervisor: hash bcrypt apenas — NUNCA plain text
    hash_senha      TEXT    NOT NULL,
    nivel           INTEGER NOT NULL DEFAULT 1,
    ativo           INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ================================================================
-- PARTE 7: CACHE DE PRODUTOS — ESTRUTURA BASE
-- ================================================================

CREATE TABLE IF NOT EXISTS produtos_grupos_cache (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    grupo_id      TEXT    NOT NULL UNIQUE,
    codigo        TEXT    NOT NULL,
    nome          TEXT    NOT NULL,
    ativo         INTEGER NOT NULL DEFAULT 1,
    versao_dados  INTEGER NOT NULL DEFAULT 0,
    atualizado_em TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS produtos_subgrupos_cache (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    subgrupo_id   TEXT    NOT NULL UNIQUE,
    grupo_id      TEXT    NOT NULL,
    codigo        TEXT    NOT NULL,
    nome          TEXT    NOT NULL,
    ativo         INTEGER NOT NULL DEFAULT 1,
    versao_dados  INTEGER NOT NULL DEFAULT 0,
    atualizado_em TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS produtos_marcas_cache (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    marca_id      TEXT    NOT NULL UNIQUE,
    nome          TEXT    NOT NULL,
    ativo         INTEGER NOT NULL DEFAULT 1,
    versao_dados  INTEGER NOT NULL DEFAULT 0,
    atualizado_em TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS produtos_cache (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    produto_id            TEXT    NOT NULL UNIQUE,
    codigo                TEXT    NOT NULL,
    codigo_barras         TEXT,
    nome                  TEXT    NOT NULL,
    descricao             TEXT,
    grupo_id              TEXT,
    subgrupo_id           TEXT,
    marca_id              TEXT,
    unidade_medida        TEXT    NOT NULL DEFAULT 'UN',
    tipo                  TEXT    NOT NULL DEFAULT 'PRODUTO',
    vendido_por_peso      INTEGER NOT NULL DEFAULT 0,
    permite_adicional     INTEGER NOT NULL DEFAULT 0,
    pizza                 INTEGER NOT NULL DEFAULT 0,
    combo                 INTEGER NOT NULL DEFAULT 0,
    ativo                 INTEGER NOT NULL DEFAULT 1,
    imagem_url            TEXT,
    versao_dados          INTEGER NOT NULL DEFAULT 0,
    atualizado_em         TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS produtos_fiscal_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    produto_id      TEXT    NOT NULL UNIQUE,
    ncm             TEXT,
    cest            TEXT,
    cfop            TEXT,
    cst_icms        TEXT,
    aliquota_icms   REAL    NOT NULL DEFAULT 0.0,
    cst_pis         TEXT,
    aliquota_pis    REAL    NOT NULL DEFAULT 0.0,
    cst_cofins      TEXT,
    aliquota_cofins REAL    NOT NULL DEFAULT 0.0,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS produtos_precos_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    produto_id      TEXT    NOT NULL UNIQUE,
    preco_venda     REAL    NOT NULL DEFAULT 0.0,
    preco_custo     REAL    NOT NULL DEFAULT 0.0,
    margem          REAL    NOT NULL DEFAULT 0.0,
    vigente_desde   TEXT,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ================================================================
-- PARTE 8: CACHE DE COMPLEMENTOS DE PRODUTOS
-- ================================================================

CREATE TABLE IF NOT EXISTS adicionais_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    adicional_id    TEXT    NOT NULL UNIQUE,
    nome            TEXT    NOT NULL,
    preco           REAL    NOT NULL DEFAULT 0.0,
    ativo           INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS produtos_adicionais_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    produto_id      TEXT    NOT NULL,
    adicional_id    TEXT    NOT NULL,
    obrigatorio     INTEGER NOT NULL DEFAULT 0,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(produto_id, adicional_id)
);

CREATE TABLE IF NOT EXISTS pizza_sabores_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    sabor_id        TEXT    NOT NULL UNIQUE,
    nome            TEXT    NOT NULL,
    descricao       TEXT,
    disponivel      INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS pizza_sabores_precos_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    sabor_id        TEXT    NOT NULL,
    tamanho         TEXT    NOT NULL,
    preco           REAL    NOT NULL DEFAULT 0.0,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(sabor_id, tamanho)
);

CREATE TABLE IF NOT EXISTS produtos_combos_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    combo_id        TEXT    NOT NULL UNIQUE,
    nome            TEXT    NOT NULL,
    preco_combo     REAL    NOT NULL DEFAULT 0.0,
    ativo           INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS locais_producao_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    local_id        TEXT    NOT NULL UNIQUE,
    nome            TEXT    NOT NULL,
    impressora_id   TEXT,
    ativo           INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ================================================================
-- PARTE 9: CACHE DE CONFIGURAÇÕES OPERACIONAIS
-- ================================================================

CREATE TABLE IF NOT EXISTS configuracoes_pdv_cache (
    id                                    INTEGER PRIMARY KEY CHECK (id = 1),
    permitir_venda_offline                INTEGER NOT NULL DEFAULT 1,
    dias_maximos_offline                  INTEGER NOT NULL DEFAULT 7,
    permitir_venda_sem_estoque            INTEGER NOT NULL DEFAULT 1,
    bloquear_produto_vencido              INTEGER NOT NULL DEFAULT 1,
    permitir_desconto_pdv                 INTEGER NOT NULL DEFAULT 1,
    desconto_maximo_padrao_percentual     REAL    NOT NULL DEFAULT 10.0,
    exigir_supervisor_desconto_acima      INTEGER NOT NULL DEFAULT 1,
    exigir_supervisor_cancelamento_item   INTEGER NOT NULL DEFAULT 1,
    exigir_supervisor_cancelamento_venda  INTEGER NOT NULL DEFAULT 1,
    permitir_alterar_preco_pdv            INTEGER NOT NULL DEFAULT 0,
    versao_dados                          INTEGER NOT NULL DEFAULT 0,
    atualizado_em                         TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS registradoras_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    registradora_id TEXT    NOT NULL UNIQUE,
    codigo          TEXT    NOT NULL,
    nome            TEXT    NOT NULL,
    tipo            TEXT    NOT NULL DEFAULT 'Caixa PDV',
    permite_multi   INTEGER NOT NULL DEFAULT 0,
    ativo           INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS terminais_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    terminal_id     TEXT    NOT NULL UNIQUE,
    codigo          TEXT    NOT NULL,
    nome            TEXT    NOT NULL,
    tipo            TEXT    NOT NULL DEFAULT 'PDV',
    ip_rede         TEXT,
    autorizado      INTEGER NOT NULL DEFAULT 0,
    ativo           INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS impressoras_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    impressora_id   TEXT    NOT NULL UNIQUE,
    nome            TEXT    NOT NULL,
    tipo_conexao    TEXT    NOT NULL DEFAULT 'USB',
    ip_porta        TEXT,
    protocolo       TEXT    NOT NULL DEFAULT 'ESC_POS',
    colunas         INTEGER NOT NULL DEFAULT 48,
    ativo           INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS setores_producao_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    setor_id        TEXT    NOT NULL UNIQUE,
    nome            TEXT    NOT NULL,
    impressora_id   TEXT,
    ativo           INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS balancas_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    balanca_id      TEXT    NOT NULL UNIQUE,
    nome            TEXT    NOT NULL,
    tipo_conexao    TEXT    NOT NULL DEFAULT 'SERIAL',
    porta           TEXT    NOT NULL,
    protocolo       TEXT    NOT NULL DEFAULT 'TOLEDO',
    baud_rate       INTEGER,
    ativo           INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS etiquetas_balanca_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    etiqueta_id     TEXT    NOT NULL UNIQUE,
    nome_formato    TEXT    NOT NULL,
    prefixo         TEXT    NOT NULL DEFAULT '2',
    posicao_inicio  INTEGER NOT NULL DEFAULT 2,
    posicao_fim     INTEGER NOT NULL DEFAULT 7,
    tipo_info       TEXT    NOT NULL DEFAULT 'PESO',
    decimais        INTEGER NOT NULL DEFAULT 3,
    ativo           INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS perifericos_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    periferico_id   TEXT    NOT NULL UNIQUE,
    nome            TEXT    NOT NULL,
    tipo            TEXT    NOT NULL DEFAULT 'GAVETA_DINHEIRO',
    marca_modelo    TEXT,
    porta           TEXT,
    ativo           INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS regras_venda_cache (
    id                          INTEGER PRIMARY KEY CHECK (id = 1),
    valor_minimo_venda          REAL    NOT NULL DEFAULT 0.0,
    valor_maximo_sem_supervisor REAL    NOT NULL DEFAULT 1000.0,
    limite_itens_por_venda      INTEGER NOT NULL DEFAULT 100,
    exigir_motivo_cancelamento  INTEGER NOT NULL DEFAULT 1,
    versao_dados                INTEGER NOT NULL DEFAULT 0,
    atualizado_em               TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS series_numeracao_cache (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    serie_id        TEXT    NOT NULL UNIQUE,
    tipo_documento  TEXT    NOT NULL DEFAULT 'FATURA',
    serie           TEXT    NOT NULL,
    proximo_numero  INTEGER NOT NULL DEFAULT 1,
    numero_final    INTEGER,
    ativo           INTEGER NOT NULL DEFAULT 1,
    versao_dados    INTEGER NOT NULL DEFAULT 0,
    atualizado_em   TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ================================================================
-- PARTE 10: ÍNDICES DE PERFORMANCE NO SQLITE
-- ================================================================

CREATE INDEX IF NOT EXISTS idx_sync_versoes_tipo    ON sync_versoes_aplicadas(tipo_dado);
CREATE INDEX IF NOT EXISTS idx_produtos_cache_cod   ON produtos_cache(codigo);
CREATE INDEX IF NOT EXISTS idx_produtos_cache_barras ON produtos_cache(codigo_barras);
CREATE INDEX IF NOT EXISTS idx_produtos_cache_ativo ON produtos_cache(ativo);
CREATE INDEX IF NOT EXISTS idx_usuarios_cache_login ON usuarios_cache(login);
CREATE INDEX IF NOT EXISTS idx_permissoes_perfil    ON permissoes_cache(perfil_id);
CREATE INDEX IF NOT EXISTS idx_sync_idemp_key       ON sync_idempotencia_local(idempotency_key);
