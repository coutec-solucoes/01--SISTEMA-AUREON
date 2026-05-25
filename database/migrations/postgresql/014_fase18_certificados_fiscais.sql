-- Fase 18 - Bloco 1: Gestão de Certificados Digitais A1 na Retaguarda

ALTER TABLE fiscal_empresas_config
ADD COLUMN certificado_cn VARCHAR(255),
ADD COLUMN certificado_cnpj VARCHAR(14),
ADD COLUMN certificado_numero_serie VARCHAR(100),
ADD COLUMN certificado_validade_inicio TIMESTAMP,
ADD COLUMN certificado_validade_fim TIMESTAMP,
ADD COLUMN certificado_atualizado_em TIMESTAMP;
