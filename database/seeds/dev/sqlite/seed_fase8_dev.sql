-- ============================================================================
-- SCRIPT DE SEED / DADOS DE TESTE E HOMOLOGAÇÃO LOCAL (FASE 8)
-- 
-- AMBIENTE: SOMENTE DESENVOLVIMENTO (DEV) E HOMOLOGAÇÃO LOCAL
-- AVISO: PROIBIDO EXECUTAR ESTE SCRIPT EM BANCOS DE PRODUÇÃO.
-- 
-- Este arquivo insere clientes de teste e um supervisor padrão com PIN "1234".
-- ============================================================================

-- Limpeza preventiva de dados de desenvolvimento anteriores nas tabelas de cache
DELETE FROM clientes_cache WHERE id IN ('CLI-001', 'CLI-002', 'CLI-003');
DELETE FROM supervisores_cache WHERE id = 'SUP-001';

-- Inserção de Clientes de Teste
INSERT INTO clientes_cache (id, nome, documento, ativo, atualizado_em)
VALUES ('CLI-001', 'Consumidor Final', '000.000.000-00', 1, '2026-05-19T20:30:00Z');

INSERT INTO clientes_cache (id, nome, documento, ativo, atualizado_em)
VALUES ('CLI-002', 'Aureon Corp S.A.', '12.345.678/0001-90', 1, '2026-05-19T20:30:00Z');

INSERT INTO clientes_cache (id, nome, documento, ativo, atualizado_em)
VALUES ('CLI-003', 'Cliente Inativo Teste', '999.999.999-99', 0, '2026-05-19T20:30:00Z');

-- Inserção de Supervisor de Teste (PIN default "1234" com hash bcrypt de 4 rounds)
INSERT INTO supervisores_cache (id, nome, pin_hash, ativo, atualizado_em)
VALUES ('SUP-001', 'Supervisor Geral (PDV)', '$2b$04$63.EoijIQHUR0y/6srSBAeYTTxgH3cFZfVCFRPZSMrwJi0wcuY9Bi', 1, '2026-05-19T20:30:00Z');
