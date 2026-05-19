pub mod commands;
pub mod commands_sync;
pub mod commands_caixa;
pub mod commands_venda;
pub mod commands_pagamento;
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
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar o Aureon PDV");
}

