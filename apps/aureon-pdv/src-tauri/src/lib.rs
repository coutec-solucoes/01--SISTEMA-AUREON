pub mod commands;
pub mod commands_sync;
pub mod commands_caixa;
pub mod commands_venda;
pub mod commands_pagamento;
pub mod commands_operacional;
pub mod commands_gourmet;
pub mod estado;

use aureon_shared::logging::inicializar_logs;
use tracing::info;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    inicializar_logs("info");
    info!(componente = "aureon-pdv", "Iniciando Aureon PDV v0.0.1");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(estado::inicializar_estado())
        .invoke_handler(tauri::generate_handler![
            // Commands base (Fase 3)
            commands::obter_status_local,
            commands::testar_sqlite,
            commands::gravar_log_local,
            commands::obter_configuracao_local,
            // Commands de sincronizacao (Fase 6)
            commands_sync::configurar_servidor_sync,
            commands_sync::testar_conexao_sync,
            commands_sync::registrar_terminal,
            commands_sync::verificar_status_terminal,
            commands_sync::executar_primeira_sincronizacao,
            commands_sync::aplicar_pacote_sincronizacao,
            commands_sync::obter_status_sync_local,
            commands_sync::listar_versoes_aplicadas,
            commands_sync::limpar_cache_sync_dev,
            // Commands de caixa (Fase 7)
            commands_caixa::abrir_caixa,
            commands_caixa::fechar_caixa,
            commands_caixa::obter_sessao_ativa,
            commands_caixa::listar_sessoes,
            // Commands de venda (Fase 7)
            commands_venda::iniciar_venda,
            commands_venda::buscar_produto_pdv,
            commands_venda::adicionar_item_venda,
            commands_venda::cancelar_item_venda,
            commands_venda::cancelar_venda,
            commands_venda::obter_venda,
            // Commands de pagamento (Fase 7)
            commands_pagamento::registrar_pagamento,
            commands_pagamento::calcular_troco,
            commands_pagamento::finalizar_venda,
            commands_pagamento::listar_pagamentos_venda,
            // Commands operacional (Fase 8)
            commands_operacional::registrar_suprimento,
            commands_operacional::registrar_sangria,
            commands_operacional::registrar_vale_funcionario,
            commands_operacional::cancelar_movimentacao_caixa,
            commands_operacional::listar_movimentacoes_caixa,
            commands_operacional::obter_resumo_caixa,
            commands_operacional::solicitar_autorizacao_supervisor,
            commands_operacional::validar_autorizacao_supervisor,
            commands_operacional::listar_autorizacoes_local,
            commands_operacional::listar_vendas_pdv,
            commands_operacional::buscar_venda_por_numero,
            commands_operacional::gerar_comprovante_nao_fiscal,
            commands_operacional::registrar_reimpressao_comprovante,
            commands_operacional::listar_pre_vendas_pdv,
            commands_operacional::obter_pre_venda_pdv,
            commands_operacional::converter_pre_venda_em_venda,
            commands_operacional::listar_orcamentos_pdv,
            commands_operacional::obter_orcamento_pdv,
            commands_operacional::converter_orcamento_em_venda,
            commands_operacional::buscar_clientes_pdv,
            commands_operacional::associar_cliente_venda,
            // Commands de PDV Gourmet (Fase 9 Bloco 2)
            commands_gourmet::listar_mesas_pdv,
            commands_gourmet::abrir_mesa,
            commands_gourmet::reservar_mesa,
            commands_gourmet::bloquear_mesa,
            commands_gourmet::cancelar_mesa,
            commands_gourmet::obter_mesa,
            commands_gourmet::adicionar_item_mesa,
            commands_gourmet::cancelar_item_mesa,
            commands_gourmet::listar_comandas_pdv,
            commands_gourmet::abrir_comanda,
            commands_gourmet::bloquear_comanda,
            commands_gourmet::cancelar_comanda,
            commands_gourmet::obter_comanda,
            commands_gourmet::adicionar_item_comanda,
            commands_gourmet::cancelar_item_comanda,
            // Commands de Transferência Gourmet (Fase 9 Bloco 3)
            commands_gourmet::transferir_mesa_total,
            commands_gourmet::transferir_itens_mesa,
            commands_gourmet::transferir_comanda_total,
            commands_gourmet::transferir_itens_comanda,
            // Commands de Produção Gourmet (Fase 9 Bloco 3)
            commands_gourmet::enviar_itens_producao,
            commands_gourmet::gerar_texto_producao,
            commands_gourmet::reimprimir_envio_producao,
            commands_gourmet::listar_envios_producao,
            // Commands de Fechamento em Venda (Fase 9 Bloco 3)
            commands_gourmet::fechar_mesa_em_venda,
            commands_gourmet::fechar_comanda_em_venda,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
