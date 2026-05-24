-- database/seeds/dev/postgresql/seed_fase17_fiscal_dev.sql
-- Seed de homologação (carga mínima) para não inflar a migration oficial.

-- Inserindo alguns dicionários de exemplo (apenas dev)
INSERT INTO fiscal_dicionario_ncm (id, codigo, descricao, ativo) VALUES 
('10000000-0000-0000-0000-000000000101', '00000000', 'NCM GENERICO DE HOMOLOGACAO', true)
ON CONFLICT DO NOTHING;

INSERT INTO fiscal_dicionario_cfop (id, codigo, descricao, tipo_operacao, ativo) VALUES 
('20000000-0000-0000-0000-000000000102', '5102', 'VENDA DE MERCADORIA DE TERCEIROS', 'SAIDA', true)
ON CONFLICT DO NOTHING;

INSERT INTO fiscal_dicionario_cst_csosn (id, codigo, tipo, descricao, ativo) VALUES 
('30000000-0000-0000-0000-000000000103', '102', 'CSOSN', 'TRIBUTADA PELO SIMPLES NACIONAL', true)
ON CONFLICT DO NOTHING;

INSERT INTO fiscal_dicionario_iva (id, codigo, descricao, pais_fiscal, aliquota_escala6, ativo) VALUES 
('40000000-0000-0000-0000-000000000104', '10', 'IVA 10%', 'PY', 100000, true)
ON CONFLICT DO NOTHING;
