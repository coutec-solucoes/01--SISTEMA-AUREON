-- ================================================================
-- MIGRATION PostgreSQL 005 — Segurança, Usuários, Permissões e Supervisores
-- Versão: 5
-- Projeto: Aureon Sistema Inteligente
-- Fase: 3
-- ================================================================
-- ESTRATÉGIA: Tabelas usuarios e perfis já existem da Fase 1.
-- Usamos ALTER TABLE com IF NOT EXISTS para adicionar colunas novas
-- sem destruir dados existentes.
-- ================================================================

-- ================================================================
-- PARTE 1: EXPANDIR TABELA perfis (Fase 1 tinha estrutura mínima)
-- ================================================================

ALTER TABLE perfis
    ADD COLUMN IF NOT EXISTS nivel_hierarquico    INTEGER     NOT NULL DEFAULT 99,
    ADD COLUMN IF NOT EXISTS is_admin             BOOLEAN     NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS perfil_administrativo BOOLEAN    NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS pode_autorizar       BOOLEAN     NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS ativo                BOOLEAN     NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS atualizado_em        TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- ================================================================
-- PARTE 2: EXPANDIR TABELA usuarios (Fase 1 tinha estrutura mínima)
-- ================================================================

-- Adicionar coluna login (único, não pode ser nulo — preenchemos com email temporário)
ALTER TABLE usuarios
    ADD COLUMN IF NOT EXISTS login               TEXT,
    ADD COLUMN IF NOT EXISTS status              TEXT        NOT NULL DEFAULT 'ATIVO',
    ADD COLUMN IF NOT EXISTS bloqueado           BOOLEAN     NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS is_admin            BOOLEAN     NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS is_supervisor       BOOLEAN     NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS acessa_retaguarda   BOOLEAN     NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS acessa_pdv          BOOLEAN     NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS telefone            TEXT,
    ADD COLUMN IF NOT EXISTS observacoes         TEXT,
    ADD COLUMN IF NOT EXISTS funcionario_id      UUID,
    ADD COLUMN IF NOT EXISTS ultimo_login        TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS atualizado_em       TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Preencher login com email para registros antigos (evitar NULL em UNIQUE)
UPDATE usuarios SET login = email WHERE login IS NULL;

-- Agora aplicamos NOT NULL e UNIQUE no login
ALTER TABLE usuarios
    ALTER COLUMN login SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_usuarios_login ON usuarios(login);

-- Constraint de status
ALTER TABLE usuarios DROP CONSTRAINT IF EXISTS usuarios_status_check;
ALTER TABLE usuarios
    ADD CONSTRAINT usuarios_status_check
    CHECK (status IN ('ATIVO', 'INATIVO', 'BLOQUEADO'));

-- ================================================================
-- PARTE 3: NOVA TABELA perfis_acesso (substitui perfis com estrutura completa)
-- ================================================================
-- A tabela perfis da Fase 1 é mantida para compatibilidade.
-- A Fase 3 trabalha com ela expandida (já feito acima).
-- Aqui criamos a tabela de relação usuario <-> perfil com mais controle.

CREATE TABLE IF NOT EXISTS usuarios_perfis (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    usuario_id      UUID        NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    perfil_id       UUID        NOT NULL REFERENCES perfis(id) ON DELETE RESTRICT,
    criado_em       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(usuario_id, perfil_id)
);

-- ================================================================
-- PARTE 4: PERMISSÕES POR MENU E AÇÃO
-- ================================================================

CREATE TABLE IF NOT EXISTS permissoes (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    perfil_id       UUID        NOT NULL REFERENCES perfis(id) ON DELETE CASCADE,
    menu            TEXT        NOT NULL,
    acao            TEXT        NOT NULL CHECK(acao IN (
                        'visualizar', 'incluir', 'editar', 'cancelar',
                        'imprimir', 'exportar', 'autorizar', 'configurar',
                        'sincronizar', 'reprocessar'
                    )),
    permitido       BOOLEAN     NOT NULL DEFAULT FALSE,
    criado_em       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(perfil_id, menu, acao)
);

CREATE INDEX IF NOT EXISTS idx_permissoes_perfil ON permissoes(perfil_id);
CREATE INDEX IF NOT EXISTS idx_permissoes_menu ON permissoes(menu);

-- ================================================================
-- PARTE 5: SESSÕES DE USUÁRIOS (TOKEN OPACO COM HASH)
-- ================================================================

CREATE TABLE IF NOT EXISTS sessoes_usuarios (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    usuario_id          UUID        NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    token_hash          TEXT        NOT NULL UNIQUE,
    criado_em           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ultimo_acesso_em    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expira_em           TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '1 hour'),
    revogado_em         TIMESTAMPTZ,
    ip_dispositivo      TEXT,
    user_agent          TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessoes_usuario ON sessoes_usuarios(usuario_id);
CREATE INDEX IF NOT EXISTS idx_sessoes_token_hash ON sessoes_usuarios(token_hash);
CREATE INDEX IF NOT EXISTS idx_sessoes_expira ON sessoes_usuarios(expira_em);

-- ================================================================
-- PARTE 6: SUPERVISORES
-- ================================================================

CREATE TABLE IF NOT EXISTS supervisores (
    id                              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    usuario_id                      UUID        NOT NULL UNIQUE REFERENCES usuarios(id) ON DELETE RESTRICT,
    pin_hash                        TEXT        NOT NULL,
    ativo                           BOOLEAN     NOT NULL DEFAULT TRUE,
    limite_desconto_percentual      NUMERIC(5,2) NOT NULL DEFAULT 10.00,
    limite_sangria                  NUMERIC(12,2) NOT NULL DEFAULT 500.00,
    autoriza_cancelamento_item      BOOLEAN     NOT NULL DEFAULT TRUE,
    autoriza_cancelamento_venda     BOOLEAN     NOT NULL DEFAULT FALSE,
    autoriza_exclusao_item          BOOLEAN     NOT NULL DEFAULT TRUE,
    autoriza_alteracao_preco        BOOLEAN     NOT NULL DEFAULT FALSE,
    autoriza_ajuste_estoque         BOOLEAN     NOT NULL DEFAULT FALSE,
    autoriza_estoque_negativo       BOOLEAN     NOT NULL DEFAULT FALSE,
    autoriza_fechamento_divergencia BOOLEAN     NOT NULL DEFAULT FALSE,
    criado_em                       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em                   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ================================================================
-- PARTE 7: AUTORIZAÇÕES DE OPERAÇÕES CRÍTICAS
-- ================================================================

CREATE TABLE IF NOT EXISTS autorizacoes (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    empresa_id          UUID        REFERENCES empresas(id),
    operacao            TEXT        NOT NULL,
    usuario_solicitante UUID        REFERENCES usuarios(id),
    supervisor_id       UUID        REFERENCES supervisores(id),
    motivo              TEXT        NOT NULL,
    valor_anterior      JSONB,
    valor_autorizado    JSONB,
    terminal_id         UUID,
    caixa_id            UUID,
    status              TEXT        NOT NULL DEFAULT 'SOLICITADA'
                        CHECK(status IN ('SOLICITADA', 'APROVADA', 'NEGADA', 'CANCELADA')),
    criado_em           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_autorizacoes_empresa ON autorizacoes(empresa_id);
CREATE INDEX IF NOT EXISTS idx_autorizacoes_supervisor ON autorizacoes(supervisor_id);
CREATE INDEX IF NOT EXISTS idx_autorizacoes_status ON autorizacoes(status);

-- ================================================================
-- PARTE 8: LOGS DE SEGURANÇA
-- ================================================================

CREATE TABLE IF NOT EXISTS logs_seguranca (
    id              BIGSERIAL   PRIMARY KEY,
    empresa_id      UUID        REFERENCES empresas(id),
    usuario_id      UUID        REFERENCES usuarios(id),
    tipo_evento     TEXT        NOT NULL,
    mensagem        TEXT        NOT NULL,
    severidade      TEXT        NOT NULL DEFAULT 'Info'
                    CHECK(severidade IN ('Info', 'Warning', 'Error', 'Critical')),
    ip_dispositivo  TEXT,
    user_agent      TEXT,
    criado_em       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_logs_seg_empresa ON logs_seguranca(empresa_id);
CREATE INDEX IF NOT EXISTS idx_logs_seg_usuario ON logs_seguranca(usuario_id);
CREATE INDEX IF NOT EXISTS idx_logs_seg_tipo ON logs_seguranca(tipo_evento);
CREATE INDEX IF NOT EXISTS idx_logs_seg_criado ON logs_seguranca(criado_em DESC);

-- ================================================================
-- PARTE 9: COMPLEMENTAR auditoria_eventos (adicionar supervisor_id)
-- ================================================================

ALTER TABLE auditoria_eventos
    ADD COLUMN IF NOT EXISTS supervisor_id  UUID REFERENCES usuarios(id),
    ADD COLUMN IF NOT EXISTS resultado      TEXT,
    ADD COLUMN IF NOT EXISTS ip_dispositivo TEXT;

-- ================================================================
-- PARTE 10: SEEDS IDEMPOTENTES — PERFIS COMPLETOS
-- ================================================================

-- Expande perfis existentes da Fase 1 e insere os novos
INSERT INTO perfis (nome, descricao, nivel_hierarquico, is_admin, perfil_administrativo, pode_autorizar, ativo) VALUES
    ('ADMINISTRADOR',       'Acesso total ao sistema',                      1,  TRUE,  TRUE,  TRUE,  TRUE),
    ('GERENTE',             'Acesso gerencial e financeiro avançado',       2,  FALSE, TRUE,  TRUE,  TRUE),
    ('SUPERVISOR',          'Supervisor de caixa e operações críticas',     3,  FALSE, FALSE, TRUE,  TRUE),
    ('OPERADOR_CAIXA',      'Operador do Ponto de Venda',                   5,  FALSE, FALSE, FALSE, TRUE),
    ('VENDEDOR',            'Vendedor sem acesso administrativo',           6,  FALSE, FALSE, FALSE, TRUE),
    ('ESTOQUISTA',          'Controle de estoque e entradas',               6,  FALSE, FALSE, FALSE, TRUE),
    ('FINANCEIRO',          'Acesso ao módulo financeiro',                  4,  FALSE, TRUE,  FALSE, TRUE),
    ('COMPRAS',             'Gestão de compras e fornecedores',             5,  FALSE, FALSE, FALSE, TRUE),
    ('GARCOM',              'Atendimento de mesas e comanda',               7,  FALSE, FALSE, FALSE, TRUE),
    ('PRODUCAO_COZINHA',    'Produção/Cozinha sem acesso financeiro',       8,  FALSE, FALSE, FALSE, TRUE),
    ('DELIVERY',            'Responsável por entregas',                     8,  FALSE, FALSE, FALSE, TRUE),
    ('DASHBOARD_PROPRIETARIO', 'Visualização de relatórios e KPIs',         3,  FALSE, TRUE,  FALSE, TRUE),
    ('SUPORTE_TECNICO',     'Acesso para configurações e logs',             2,  FALSE, TRUE,  FALSE, TRUE)
ON CONFLICT (nome) DO UPDATE SET
    nivel_hierarquico    = EXCLUDED.nivel_hierarquico,
    is_admin             = EXCLUDED.is_admin,
    perfil_administrativo = EXCLUDED.perfil_administrativo,
    pode_autorizar       = EXCLUDED.pode_autorizar,
    ativo                = EXCLUDED.ativo;

-- ================================================================
-- PARTE 11: PRIMEIRO ADMINISTRADOR REAL
-- Seed idempotente — só cria se não houver nenhum admin ativo.
-- Senha padrão: Aureon@2026 (DEVE ser alterada no primeiro acesso)
-- Hash Argon2 gerado em tempo de execução pela API — aqui usamos placeholder.
-- A API Local (endpoint POST /auth/setup) faz a criação real com hash correto.
-- Esta flag sinaliza que o setup precisa ser executado.
-- ================================================================

CREATE TABLE IF NOT EXISTS configuracao_setup (
    chave   TEXT PRIMARY KEY,
    valor   TEXT NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO configuracao_setup (chave, valor)
    VALUES ('admin_inicial_criado', 'false')
ON CONFLICT (chave) DO NOTHING;

-- ================================================================
-- PARTE 12: PERMISSÕES INICIAIS DO ADMINISTRADOR
-- Seeds de permissões completas para o perfil ADMINISTRADOR
-- ================================================================

-- Inserir permissões de administrador para todos os menus principais
INSERT INTO permissoes (perfil_id, menu, acao, permitido)
SELECT
    p.id,
    m.menu,
    a.acao,
    TRUE
FROM
    perfis p,
    (VALUES
        ('configuracoes.empresa'),
        ('seguranca.usuarios'),
        ('seguranca.perfis'),
        ('seguranca.permissoes'),
        ('seguranca.supervisores'),
        ('seguranca.autorizacoes'),
        ('seguranca.auditoria'),
        ('seguranca.logs')
    ) AS m(menu),
    (VALUES
        ('visualizar'), ('incluir'), ('editar'), ('cancelar'),
        ('imprimir'), ('exportar'), ('autorizar'), ('configurar'),
        ('sincronizar'), ('reprocessar')
    ) AS a(acao)
WHERE p.nome = 'ADMINISTRADOR'
ON CONFLICT (perfil_id, menu, acao) DO NOTHING;

-- ================================================================
-- FIM DA MIGRATION 005
-- ================================================================
