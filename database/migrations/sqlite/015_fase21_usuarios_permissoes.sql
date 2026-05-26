CREATE TABLE IF NOT EXISTS usuarios_local (
    id TEXT PRIMARY KEY,
    nome TEXT NOT NULL,
    login TEXT NOT NULL UNIQUE,
    senha_hash TEXT NOT NULL,
    senha_algoritmo TEXT NOT NULL DEFAULT 'ARGON2ID',
    pin_hash TEXT,
    ativo INTEGER NOT NULL DEFAULT 1,
    exige_troca_senha INTEGER NOT NULL DEFAULT 0,
    ultimo_login_em TEXT,
    criado_em TEXT NOT NULL,
    atualizado_em TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS perfis_local (
    id TEXT PRIMARY KEY,
    codigo TEXT NOT NULL UNIQUE,
    nome TEXT NOT NULL,
    descricao TEXT,
    sistema INTEGER NOT NULL DEFAULT 0,
    ativo INTEGER NOT NULL DEFAULT 1,
    criado_em TEXT NOT NULL,
    atualizado_em TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS permissoes_local (
    id TEXT PRIMARY KEY,
    codigo TEXT NOT NULL UNIQUE,
    modulo TEXT NOT NULL,
    acao TEXT NOT NULL,
    descricao TEXT,
    risco TEXT NOT NULL DEFAULT 'NORMAL',
    criado_em TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS perfil_permissoes_local (
    id TEXT PRIMARY KEY,
    perfil_id TEXT NOT NULL,
    permissao_id TEXT NOT NULL,
    permitido INTEGER NOT NULL DEFAULT 1,
    criado_em TEXT NOT NULL,
    UNIQUE(perfil_id, permissao_id),
    FOREIGN KEY(perfil_id) REFERENCES perfis_local(id),
    FOREIGN KEY(permissao_id) REFERENCES permissoes_local(id)
);

CREATE TABLE IF NOT EXISTS usuario_perfis_local (
    id TEXT PRIMARY KEY,
    usuario_id TEXT NOT NULL,
    perfil_id TEXT NOT NULL,
    criado_em TEXT NOT NULL,
    UNIQUE(usuario_id, perfil_id),
    FOREIGN KEY(usuario_id) REFERENCES usuarios_local(id),
    FOREIGN KEY(perfil_id) REFERENCES perfis_local(id)
);

CREATE TABLE IF NOT EXISTS sessoes_usuario_local (
    id TEXT PRIMARY KEY,
    usuario_id TEXT NOT NULL,
    login TEXT NOT NULL,
    nome_usuario TEXT NOT NULL,
    aberta_em TEXT NOT NULL,
    encerrada_em TEXT,
    ativa INTEGER NOT NULL DEFAULT 1,
    terminal_id TEXT,
    installation_id TEXT,
    FOREIGN KEY(usuario_id) REFERENCES usuarios_local(id)
);

CREATE TABLE IF NOT EXISTS auditoria_operacional_local (
    id TEXT PRIMARY KEY,
    usuario_id TEXT,
    login TEXT,
    tipo_evento TEXT NOT NULL,
    modulo TEXT,
    acao TEXT,
    entidade_tipo TEXT,
    entidade_id TEXT,
    sucesso INTEGER NOT NULL DEFAULT 1,
    mensagem TEXT,
    payload_preview TEXT,
    criado_em TEXT NOT NULL
);
