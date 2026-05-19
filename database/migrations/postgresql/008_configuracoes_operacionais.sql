-- ==========================================
-- FASE 5 — CONFIGURAÇÕES OPERACIONAIS
-- Migration 008: Tabelas Físicas de Dispositivos e Comportamentos Operacionais do PDV
-- ==========================================

-- 1. CONFIGURAÇÕES GERAIS DO PDV
CREATE TABLE IF NOT EXISTS configuracoes_pdv (
    empresa_id UUID PRIMARY KEY REFERENCES empresas(id) ON DELETE CASCADE,
    permitir_venda_offline BOOLEAN NOT NULL DEFAULT TRUE,
    dias_maximos_offline INT NOT NULL DEFAULT 7 CONSTRAINT chk_dias_maximos_offline CHECK (dias_maximos_offline > 0),
    exigir_cotacao_ao_abrir_caixa BOOLEAN NOT NULL DEFAULT FALSE,
    permitir_venda_sem_estoque BOOLEAN NOT NULL DEFAULT TRUE,
    bloquear_produto_vencido BOOLEAN NOT NULL DEFAULT TRUE,
    alertar_produto_proximo_vencer BOOLEAN NOT NULL DEFAULT TRUE,
    dias_alerta_vencimento INT NOT NULL DEFAULT 30 CONSTRAINT chk_dias_alerta_vencimento CHECK (dias_alerta_vencimento >= 0),
    permitir_desconto_pdv BOOLEAN NOT NULL DEFAULT TRUE,
    desconto_maximo_padrao_percentual DECIMAL(5,2) NOT NULL DEFAULT 10.00 CONSTRAINT chk_desconto_maximo_padrao CHECK (desconto_maximo_padrao_percentual >= 0 AND desconto_maximo_padrao_percentual <= 100),
    exigir_supervisor_desconto_acima_limite BOOLEAN NOT NULL DEFAULT TRUE,
    exigir_supervisor_cancelamento_item BOOLEAN NOT NULL DEFAULT TRUE,
    exigir_supervisor_cancelamento_venda BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_alterar_preco_pdv BOOLEAN NOT NULL DEFAULT FALSE,
    permitir_cliente_sem_cadastro BOOLEAN NOT NULL DEFAULT TRUE,
    exigir_cliente_completo_crediario BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_reimpressao_comprovante BOOLEAN NOT NULL DEFAULT TRUE,
    exigir_supervisor_reimpressao BOOLEAN NOT NULL DEFAULT FALSE,
    permitir_cadastro_cliente_pdv BOOLEAN NOT NULL DEFAULT TRUE,
    ativo BOOLEAN NOT NULL DEFAULT TRUE
);

-- 2. REGISTRADORAS / GAVETAS DE CAIXA
CREATE TABLE IF NOT EXISTS registradoras (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    codigo VARCHAR(50) NOT NULL UNIQUE,
    nome VARCHAR(100) NOT NULL,
    descricao TEXT,
    tipo VARCHAR(50) NOT NULL CONSTRAINT chk_tipo_registradora CHECK (tipo IN ('Caixa PDV', 'Caixa Balcao', 'Caixa Delivery', 'Tesouraria Auxiliar')),
    tesouraria_id UUID,
    terminal_padrao_id UUID, -- Chave estrangeira inserida por ALTER TABLE após criação da tabela de terminais
    usuario_responsavel_id UUID REFERENCES pessoas(id) ON DELETE SET NULL,
    permite_multimoeda BOOLEAN NOT NULL DEFAULT FALSE,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. TERMINAIS PDV
CREATE TABLE IF NOT EXISTS terminais_pdv (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    codigo_terminal VARCHAR(50) NOT NULL UNIQUE,
    nome_terminal VARCHAR(100) NOT NULL,
    descricao TEXT,
    tipo_terminal VARCHAR(50) NOT NULL CONSTRAINT chk_tipo_terminal CHECK (tipo_terminal IN ('PDV', 'Pre-venda', 'Autoatendimento', 'Tablet', 'Garcom', 'Producao')),
    ip_rede_local VARCHAR(45),
    identificador_maquina_futuro VARCHAR(255),
    registradora_id UUID REFERENCES registradoras(id) ON DELETE SET NULL,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    autorizado BOOLEAN NOT NULL DEFAULT FALSE,
    data_autorizacao TIMESTAMPTZ,
    ultimo_status_futuro VARCHAR(50),
    ultima_sincronizacao_futura TIMESTAMPTZ,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Adiciona a restrição circular segura entre Registradoras e Terminais
ALTER TABLE registradoras ADD CONSTRAINT fk_registradora_terminal FOREIGN KEY (terminal_padrao_id) REFERENCES terminais_pdv(id) ON DELETE SET NULL;

-- 4. CONFIGURAÇÃO DE MESAS (GOURMET)
CREATE TABLE IF NOT EXISTS configuracoes_mesas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mesas_ativas BOOLEAN NOT NULL DEFAULT FALSE,
    quantidade_mesas INT NOT NULL DEFAULT 0 CONSTRAINT chk_quantidade_mesas CHECK (quantidade_mesas >= 0),
    prefixo_mesa VARCHAR(20) NOT NULL DEFAULT 'Mesa',
    permitir_nome_informal BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_reserva_mesa BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_transferencia_mesa BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_transferencia_parcial_itens BOOLEAN NOT NULL DEFAULT TRUE,
    imprimir_producao_por_mesa BOOLEAN NOT NULL DEFAULT TRUE,
    bloquear_mesa_com_pendencia BOOLEAN NOT NULL DEFAULT FALSE,
    ativo BOOLEAN NOT NULL DEFAULT TRUE
);

-- 5. CADASTRO DE MESAS
CREATE TABLE IF NOT EXISTS mesas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    numero INT NOT NULL UNIQUE CONSTRAINT chk_numero_mesa CHECK (numero > 0),
    nome_exibicao VARCHAR(50),
    setor VARCHAR(50),
    capacidade INT NOT NULL DEFAULT 4 CONSTRAINT chk_capacidade_mesa CHECK (capacidade >= 0),
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    observacao TEXT
);

-- 6. CONFIGURAÇÃO DE COMANDAS
CREATE TABLE IF NOT EXISTS configuracoes_comandas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    comandas_ativas BOOLEAN NOT NULL DEFAULT FALSE,
    faixa_inicial INT NOT NULL DEFAULT 1 CONSTRAINT chk_faixa_inicial CHECK (faixa_inicial >= 0),
    faixa_final INT NOT NULL DEFAULT 100 CONSTRAINT chk_faixa_final CHECK (faixa_final >= 0),
    permitir_nome_informal BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_transferencia_comanda BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_transferencia_parcial_itens BOOLEAN NOT NULL DEFAULT TRUE,
    imprimir_producao_por_comanda BOOLEAN NOT NULL DEFAULT TRUE,
    bloquear_comanda_com_pendencia BOOLEAN NOT NULL DEFAULT FALSE,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    CONSTRAINT chk_comandas_faixas CHECK (faixa_inicial <= faixa_final)
);

-- 7. CADASTRO DE COMANDAS
CREATE TABLE IF NOT EXISTS comandas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    numero INT NOT NULL UNIQUE CONSTRAINT chk_numero_comanda CHECK (numero > 0),
    codigo_barras_qr_futuro VARCHAR(100),
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    observacao TEXT
);

-- 8. CONFIGURAÇÃO DE PRÉ-VENDAS
CREATE TABLE IF NOT EXISTS configuracoes_pre_vendas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    receber_pre_venda_pdv BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_buscar_pre_venda_por_codigo BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_buscar_pre_venda_por_cliente BOOLEAN NOT NULL DEFAULT TRUE,
    ativo BOOLEAN NOT NULL DEFAULT TRUE
);

-- 9. CONFIGURAÇÃO DE ORÇAMENTOS
CREATE TABLE IF NOT EXISTS configuracoes_orcamentos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    permitir_transformar_orcamento_em_venda BOOLEAN NOT NULL DEFAULT TRUE,
    validade_padrao_orcamento_dias INT NOT NULL DEFAULT 15 CONSTRAINT chk_validade_orcamento CHECK (validade_padrao_orcamento_dias > 0),
    exigir_cliente_orcamento BOOLEAN NOT NULL DEFAULT FALSE,
    permitir_desconto_orcamento BOOLEAN NOT NULL DEFAULT TRUE,
    exigir_supervisor_desconto_orcamento BOOLEAN NOT NULL DEFAULT FALSE,
    ativo BOOLEAN NOT NULL DEFAULT TRUE
);

-- 10. REGRAS DE VENDA
CREATE TABLE IF NOT EXISTS regras_venda (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    permitir_venda_produto_inativo BOOLEAN NOT NULL DEFAULT FALSE,
    permitir_venda_estoque_negativo BOOLEAN NOT NULL DEFAULT FALSE,
    bloquear_venda_produto_vencido BOOLEAN NOT NULL DEFAULT TRUE,
    alertar_venda_produto_proximo_vencer BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_desconto_item BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_desconto_total BOOLEAN NOT NULL DEFAULT TRUE,
    desconto_maximo_item_percentual DECIMAL(5,2) NOT NULL DEFAULT 100.00 CONSTRAINT chk_desconto_maximo_item CHECK (desconto_maximo_item_percentual >= 0 AND desconto_maximo_item_percentual <= 100),
    desconto_maximo_total_percentual DECIMAL(5,2) NOT NULL DEFAULT 100.00 CONSTRAINT chk_desconto_maximo_total CHECK (desconto_maximo_total_percentual >= 0 AND desconto_maximo_total_percentual <= 100),
    exigir_supervisor_desconto_item BOOLEAN NOT NULL DEFAULT TRUE,
    exigir_supervisor_desconto_total BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_cancelamento_item BOOLEAN NOT NULL DEFAULT TRUE,
    exigir_supervisor_cancelamento_item BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_cancelamento_venda BOOLEAN NOT NULL DEFAULT TRUE,
    exigir_supervisor_cancelamento_venda BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_reimpressao BOOLEAN NOT NULL DEFAULT TRUE,
    exigir_supervisor_reimpressao BOOLEAN NOT NULL DEFAULT FALSE,
    ativo BOOLEAN NOT NULL DEFAULT TRUE
);

-- 11. SÉRIES E NUMERAÇÃO
CREATE TABLE IF NOT EXISTS series_numeracao (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tipo_documento VARCHAR(50) NOT NULL CONSTRAINT chk_tipo_documento CHECK (tipo_documento IN ('Venda', 'Pre-venda', 'Orcamento', 'Mesa', 'Comanda', 'Delivery', 'Senha atendimento')),
    serie VARCHAR(10) NOT NULL,
    proximo_numero INT NOT NULL DEFAULT 1 CONSTRAINT chk_proximo_numero CHECK (proximo_numero > 0),
    reiniciar_diariamente BOOLEAN NOT NULL DEFAULT FALSE,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    CONSTRAINT uq_documento_serie UNIQUE (tipo_documento, serie)
);

-- 12. IMPRESSORAS
CREATE TABLE IF NOT EXISTS impressoras (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nome VARCHAR(100) NOT NULL UNIQUE,
    tipo VARCHAR(50) NOT NULL CONSTRAINT chk_tipo_impressora CHECK (tipo IN ('Termica 80mm', 'Termica 58mm', 'A4', 'Etiqueta', 'Cozinha/Producao', 'Fiscal futura')),
    conexao VARCHAR(50) NOT NULL CONSTRAINT chk_conexao_impressora CHECK (conexao IN ('USB', 'Rede/IP', 'Compartilhada Windows', 'Bluetooth futuro', 'Serial futuro')),
    endereco VARCHAR(255),
    porta INT,
    largura_colunas INT NOT NULL DEFAULT 48 CONSTRAINT chk_largura_colunas CHECK (largura_colunas > 0),
    modelo_driver VARCHAR(100),
    cortar_papel BOOLEAN NOT NULL DEFAULT TRUE,
    abrir_gaveta BOOLEAN NOT NULL DEFAULT FALSE,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    observacao TEXT
);

-- 13. SETORES DE PRODUÇÃO
CREATE TABLE IF NOT EXISTS setores_producao (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nome VARCHAR(100) NOT NULL UNIQUE,
    descricao TEXT,
    impressora_id UUID REFERENCES impressoras(id) ON DELETE SET NULL,
    tipo_producao VARCHAR(50) NOT NULL CONSTRAINT chk_tipo_producao CHECK (tipo_producao IN ('Cozinha', 'Bar', 'Pizzaria', 'Chapa', 'Expedicao', 'Balcao', 'Producao Geral')),
    ativo BOOLEAN NOT NULL DEFAULT TRUE
);

-- 14. BALANÇAS
CREATE TABLE IF NOT EXISTS balancas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nome VARCHAR(100) NOT NULL UNIQUE,
    marca VARCHAR(50),
    modelo VARCHAR(50),
    tipo_comunicacao VARCHAR(50) NOT NULL CONSTRAINT chk_tipo_comunicacao CHECK (tipo_comunicacao IN ('Serial', 'USB', 'TCP/IP', 'Etiqueta', 'Manual')),
    porta_serial VARCHAR(20),
    ip VARCHAR(45),
    porta_tcp INT,
    protocolo VARCHAR(50),
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    observacao TEXT
);

-- 15. ETIQUETAS DE BALANÇA
CREATE TABLE IF NOT EXISTS etiquetas_balanca (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nome VARCHAR(100) NOT NULL UNIQUE,
    prefixo VARCHAR(10),
    tamanho_codigo INT NOT NULL DEFAULT 5 CONSTRAINT chk_tamanho_codigo CHECK (tamanho_codigo > 0),
    posicao_codigo_inicio INT NOT NULL CONSTRAINT chk_posicao_codigo_inicio CHECK (posicao_codigo_inicio >= 0),
    posicao_codigo_fim INT NOT NULL CONSTRAINT chk_posicao_codigo_fim CHECK (posicao_codigo_fim >= posicao_codigo_inicio),
    posicao_peso_inicio INT NOT NULL CONSTRAINT chk_posicao_peso_inicio CHECK (posicao_peso_inicio >= 0),
    posicao_peso_fim INT NOT NULL CONSTRAINT chk_posicao_peso_fim CHECK (posicao_peso_fim >= posicao_peso_inicio),
    posicao_valor_inicio INT NOT NULL CONSTRAINT chk_posicao_valor_inicio CHECK (posicao_valor_inicio >= 0),
    posicao_valor_fim INT NOT NULL CONSTRAINT chk_posicao_valor_fim CHECK (posicao_valor_fim >= posicao_valor_inicio),
    tipo_leitura VARCHAR(50) NOT NULL CONSTRAINT chk_tipo_leitura_etiqueta CHECK (tipo_leitura IN ('Codigo + peso', 'Codigo + valor', 'Codigo fixo', 'Peso manual')),
    casas_decimais INT NOT NULL DEFAULT 3 CONSTRAINT chk_casas_decimais CHECK (casas_decimais >= 0),
    ativo BOOLEAN NOT NULL DEFAULT TRUE
);

-- 16. PERIFÉRICOS
CREATE TABLE IF NOT EXISTS perifericos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nome VARCHAR(100) NOT NULL UNIQUE,
    tipo VARCHAR(50) NOT NULL CONSTRAINT chk_tipo_periferico CHECK (tipo IN ('Gaveta', 'Leitor de codigo de barras', 'Display cliente', 'Pinpad futuro', 'SAT/fiscal futuro', 'Scanner', 'Teclado programavel')),
    conexao VARCHAR(50) NOT NULL,
    endereco VARCHAR(255),
    porta INT,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    observacao TEXT
);

-- 17. CONFIGURAÇÕES DE SENHAS E CHAMADAS
CREATE TABLE IF NOT EXISTS configuracoes_senhas_chamadas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    senhas_ativas BOOLEAN NOT NULL DEFAULT FALSE,
    prefixo_senha VARCHAR(10) NOT NULL DEFAULT 'A',
    proximo_numero INT NOT NULL DEFAULT 1 CONSTRAINT chk_proximo_senha CHECK (proximo_numero > 0),
    reiniciar_diariamente BOOLEAN NOT NULL DEFAULT TRUE,
    permitir_chamada_painel BOOLEAN NOT NULL DEFAULT TRUE,
    zerar_senha_dia_seguinte BOOLEAN NOT NULL DEFAULT TRUE,
    ativo BOOLEAN NOT NULL DEFAULT TRUE
);

-- SEEDS OPERACIONAIS PADRÃO (Para garantir registros iniciais não destrutivos de teste)
INSERT INTO configuracoes_pdv (empresa_id, permitir_venda_offline, dias_maximos_offline, exigir_cotacao_ao_abrir_caixa, permitir_venda_sem_estoque, bloquear_produto_vencido, alertar_produto_proximo_vencer, dias_alerta_vencimento, permitir_desconto_pdv, desconto_maximo_padrao_percentual, exigir_supervisor_desconto_acima_limite, exigir_supervisor_cancelamento_item, exigir_supervisor_cancelamento_venda, permitir_alterar_preco_pdv, permitir_cliente_sem_cadastro, exigir_cliente_completo_crediario, permitir_reimpressao_comprovante, exigir_supervisor_reimpressao, permitir_cadastro_cliente_pdv, ativo)
SELECT id, TRUE, 7, FALSE, TRUE, TRUE, TRUE, 30, TRUE, 10.00, TRUE, TRUE, TRUE, FALSE, TRUE, TRUE, TRUE, FALSE, TRUE, TRUE 
FROM empresas 
ON CONFLICT (empresa_id) DO NOTHING;

INSERT INTO configuracoes_mesas (mesas_ativas, quantidade_mesas, prefixo_mesa, permitir_nome_informal, permitir_reserva_mesa, permitir_transferencia_mesa, permitir_transferencia_parcial_itens, imprimir_producao_por_mesa, bloquear_mesa_com_pendencia, ativo)
VALUES (FALSE, 20, 'Mesa', TRUE, TRUE, TRUE, TRUE, TRUE, FALSE, TRUE)
ON CONFLICT DO NOTHING;

INSERT INTO configuracoes_comandas (comandas_ativas, faixa_inicial, faixa_final, permitir_nome_informal, permitir_transferencia_comanda, permitir_transferencia_parcial_itens, imprimir_producao_por_comanda, bloquear_comanda_com_pendencia, ativo)
VALUES (FALSE, 1, 100, TRUE, TRUE, TRUE, TRUE, FALSE, TRUE)
ON CONFLICT DO NOTHING;

INSERT INTO configuracoes_pre_vendas (receber_pre_venda_pdv, permitir_buscar_pre_venda_por_codigo, permitir_buscar_pre_venda_por_cliente, ativo)
VALUES (TRUE, TRUE, TRUE, TRUE)
ON CONFLICT DO NOTHING;

INSERT INTO configuracoes_orcamentos (permitir_transformar_orcamento_em_venda, validade_padrao_orcamento_dias, exigir_cliente_orcamento, permitir_desconto_orcamento, exigir_supervisor_desconto_orcamento, ativo)
VALUES (TRUE, 15, FALSE, TRUE, FALSE, TRUE)
ON CONFLICT DO NOTHING;

INSERT INTO regras_venda (permitir_venda_produto_inativo, permitir_venda_estoque_negativo, bloquear_venda_produto_vencido, alertar_venda_produto_proximo_vencer, permitir_desconto_item, permitir_desconto_total, desconto_maximo_item_percentual, desconto_maximo_total_percentual, exigir_supervisor_desconto_item, exigir_supervisor_desconto_total, permitir_cancelamento_item, exigir_supervisor_cancelamento_item, permitir_cancelamento_venda, exigir_supervisor_cancelamento_venda, permitir_reimpressao, exigir_supervisor_reimpressao, ativo)
VALUES (FALSE, FALSE, TRUE, TRUE, TRUE, TRUE, 100.00, 100.00, TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, FALSE, TRUE)
ON CONFLICT DO NOTHING;

INSERT INTO configuracoes_senhas_chamadas (senhas_ativas, prefixo_senha, proximo_numero, reiniciar_diariamente, permitir_chamada_painel, zerar_senha_dia_seguinte, ativo)
VALUES (FALSE, 'A', 1, TRUE, TRUE, TRUE, TRUE)
ON CONFLICT DO NOTHING;
