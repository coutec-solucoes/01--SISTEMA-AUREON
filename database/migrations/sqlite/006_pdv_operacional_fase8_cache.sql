-- Migration 006: Criação das tabelas de cache para Clientes e Supervisores no PDV local

CREATE TABLE IF NOT EXISTS clientes_cache (
    id TEXT PRIMARY KEY,
    nome TEXT NOT NULL,
    documento TEXT,
    ativo BOOLEAN NOT NULL DEFAULT 1,
    atualizado_em TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS supervisores_cache (
    id TEXT PRIMARY KEY,
    nome TEXT NOT NULL,
    pin_hash TEXT NOT NULL,
    ativo BOOLEAN NOT NULL DEFAULT 1,
    atualizado_em TEXT NOT NULL
);

-- Inserção de sementes para teste e homologação local

-- Clientes
INSERT OR REPLACE INTO clientes_cache (id, nome, documento, ativo, atualizado_em)
VALUES ('CLI-001', 'Consumidor Final', '000.000.000-00', 1, '2026-05-19T20:30:00Z');

INSERT OR REPLACE INTO clientes_cache (id, nome, documento, ativo, atualizado_em)
VALUES ('CLI-002', 'Aureon Corp S.A.', '12.345.678/0001-90', 1, '2026-05-19T20:30:00Z');

INSERT OR REPLACE INTO clientes_cache (id, nome, documento, ativo, atualizado_em)
VALUES ('CLI-003', 'Cliente Inativo Teste', '999.999.999-99', 0, '2026-05-19T20:30:00Z');

-- Supervisor: PIN default "1234"
-- Hash correspondente a "1234" gerado com bcrypt (4 rounds para resposta ultra-rápida no PDV local)
INSERT OR REPLACE INTO supervisores_cache (id, nome, pin_hash, ativo, atualizado_em)
VALUES ('SUP-001', 'Supervisor Geral (PDV)', '$2b$04$63.EoijIQHUR0y/6srSBAeYTTxgH3cFZfVCFRPZSMrwJi0wcuY9Bi', 1, '2026-05-19T20:30:00Z');
