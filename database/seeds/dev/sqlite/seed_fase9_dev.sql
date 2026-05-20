-- ============================================================================
-- Seed de Desenvolvimento — Fase 9 (PDV Gourmet)
-- ESTE SCRIPT É APENAS PARA AMBIENTE DE HOMOLOGAÇÃO E TESTES LOCAIS.
-- PROIBIDO RODAR DIRETAMENTE EM PRODUÇÃO.
-- ============================================================================

-- Popular mesas_cache com 10 mesas de teste
INSERT OR IGNORE INTO mesas_cache (id, numero, nome, ativo) VALUES
('mesa-id-1',  1,  'Mesa 01 - Salão Principal', 1),
('mesa-id-2',  2,  'Mesa 02 - Salão Principal', 1),
('mesa-id-3',  3,  'Mesa 03 - Salão Principal', 1),
('mesa-id-4',  4,  'Mesa 04 - Salão Principal', 1),
('mesa-id-5',  5,  'Mesa 05 - Área Externa',    1),
('mesa-id-6',  6,  'Mesa 06 - Área Externa',    1),
('mesa-id-7',  7,  'Mesa 07 - Área Externa',    1),
('mesa-id-8',  8,  'Mesa 08 - Varanda',         1),
('mesa-id-9',  9,  'Mesa 09 - Varanda',         1),
('mesa-id-10', 10, 'Mesa 10 - VIP',             1);

-- Popular comandas_cache com 10 comandas de teste
INSERT OR IGNORE INTO comandas_cache (id, numero, ativo) VALUES
('comanda-id-101', 101, 1),
('comanda-id-102', 102, 1),
('comanda-id-103', 103, 1),
('comanda-id-104', 104, 1),
('comanda-id-105', 105, 1),
('comanda-id-106', 106, 1),
('comanda-id-107', 107, 1),
('comanda-id-108', 108, 1),
('comanda-id-109', 109, 1),
('comanda-id-110', 110, 1);
