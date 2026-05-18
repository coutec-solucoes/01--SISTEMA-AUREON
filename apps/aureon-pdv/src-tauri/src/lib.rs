pub mod commands;
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
            commands::obter_status_local,
            commands::testar_sqlite,
            commands::gravar_log_local,
            commands::obter_configuracao_local,
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar o Aureon PDV");
}
