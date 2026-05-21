-- Seed Fase 16 - Mínimo Fiscal Dev
-- Apenas para testes e homologação local.
-- NÃO CONTÉM TABELAS LEGAIS COMPLETAS, ELAS VIRÃO DO POSTGRESQL.

-- Mock do IVA Paraguaio
INSERT INTO fiscal_iva_cache (id, codigo, descricao, aliquota_escala6, ativo, atualizado_em) VALUES 
('iva-py-10', '10', 'IVA 10%', 100000, 1, datetime('now')),
('iva-py-05', '5', 'IVA 5%', 50000, 1, datetime('now')),
('iva-py-00', '0', 'IVA Isento', 0, 1, datetime('now'));

-- Mock Exemplo de CFOP (Brasil)
INSERT INTO fiscal_cfop_cache (id, codigo, descricao, tipo_operacao, ativo, atualizado_em) VALUES 
('cfop-5102', '5102', 'Venda de mercadoria de terceiros', 'VENDA_INTERNA', 1, datetime('now'));

-- Mock Exemplo de CSOSN (Brasil - Simples Nacional)
INSERT INTO fiscal_cst_csosn_cache (id, codigo, tipo, descricao, ativo, atualizado_em) VALUES 
('csosn-102', '102', 'CSOSN', 'Tributada pelo Simples Nacional sem permissão de crédito', 1, datetime('now'));

-- Mock Exemplo de NCM (Brasil)
INSERT INTO fiscal_ncm_cache (id, codigo, descricao, ativo, atualizado_em) VALUES 
('ncm-geral', '00000000', 'NCM Padrao / Nao Classificado', 1, datetime('now'));

-- Mock Configuração da Empresa Atual (assumindo a empresa do cache inicial)
INSERT INTO fiscal_empresa_cache (id, pais_fiscal, regime_fiscal, ambiente, forma_emissao, atualizado_em) VALUES 
('config-fiscal-padrao', 'BR', 'SIMPLES_NACIONAL', 'HOMOLOGACAO', 'NORMAL', datetime('now'));
