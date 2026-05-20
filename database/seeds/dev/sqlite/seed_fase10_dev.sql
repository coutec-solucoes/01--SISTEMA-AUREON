-- Script para inserção de dados de teste (DEV ONLY)
-- Fase 10 - Delivery Operacional

-- Entregadores Cache (Mock)
INSERT OR IGNORE INTO entregadores_cache (id, nome, documento, ativo, criado_em)
VALUES 
('entregador-mock-1', 'Zé Entregas', '111.111.111-11', 1, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
('entregador-mock-2', 'Maria Moto', '222.222.222-22', 1, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
('entregador-mock-3', 'João (Inativo)', '333.333.333-33', 0, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));

-- Delivery Operacional (Mock) - Pedido Novo / Retirada
INSERT OR IGNORE INTO delivery_operacional (id, numero_pedido, cliente_id, nome_cliente_informal, telefone, endereco_completo, tipo_pedido, status, origem, entregador_id, taxa_entrega_minor, total_consumo_minor, sessao_caixa_id, observacao, previsao_entrega, aberto_em, fechado_em)
VALUES 
('delivery-mock-1', 1001, NULL, 'Cliente Teste Retirada', '11999990000', '', 'RETIRADA', 'NOVO', 'LOCAL', NULL, 0, 0, 'sessao-dev-default', 'Ligar quando estiver pronto', strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '+30 minutes'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), NULL);

-- Delivery Operacional (Mock) - Pedido Online Novo / Entrega
INSERT OR IGNORE INTO delivery_operacional (id, numero_pedido, cliente_id, nome_cliente_informal, telefone, endereco_completo, tipo_pedido, status, origem, entregador_id, taxa_entrega_minor, total_consumo_minor, sessao_caixa_id, observacao, previsao_entrega, aberto_em, fechado_em)
VALUES 
('delivery-mock-2', 1002, NULL, 'Cliente Teste Online', '11999990001', 'Rua das Flores, 123, Centro', 'ENTREGA', 'NOVO', 'ONLINE', NULL, 500, 0, NULL, 'Entregar no portão azul', strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '+60 minutes'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), NULL);
