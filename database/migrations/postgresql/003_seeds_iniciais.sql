-- database/migrations/postgresql/003_seeds_iniciais.sql
-- Seeds idempotentes da Fase 1

-- Moedas (BRL, PYG, USD)
INSERT INTO moedas (codigo, nome, simbolo) VALUES
    ('BRL', 'Real Brasileiro', 'R$'),
    ('PYG', 'Guarani Paraguaio', '₲'),
    ('USD', 'Dólar Americano', 'US$')
ON CONFLICT (codigo) DO NOTHING;

-- Formas de Pagamento Básicas
INSERT INTO formas_pagamento (codigo, nome, tipo) VALUES
    ('DINHEIRO', 'Dinheiro', 'FISICO'),
    ('PIX_MANUAL', 'Pix Manual', 'DIGITAL'),
    ('CARTAO_DEBITO', 'Cartão de Débito', 'CARTAO'),
    ('CARTAO_CREDITO', 'Cartão de Crédito', 'CARTAO'),
    ('TRANSFERENCIA', 'Transferência Bancária', 'DIGITAL'),
    ('CREDIARIO', 'Crediário', 'PRAZO')
ON CONFLICT (codigo) DO NOTHING;

-- Perfis Básicos
INSERT INTO perfis (nome, descricao) VALUES
    ('ADMINISTRADOR', 'Acesso total ao sistema'),
    ('GERENTE', 'Acesso gerencial e financeiro avançado'),
    ('OPERADOR', 'Acesso restrito ao PDV e vendas básicas'),
    ('SUPORTE_TECNICO', 'Acesso para configurações e logs')
ON CONFLICT (nome) DO NOTHING;

-- A inserção da Tesouraria Central e do Usuário Admin inicial 
-- será feita via código (Rust) durante o Setup para garantir o Hash da senha,
-- e vincular corretamente a tesouraria com o ID da moeda (BRL default).
