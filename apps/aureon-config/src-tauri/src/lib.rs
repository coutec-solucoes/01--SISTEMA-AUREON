pub mod commands;
pub mod estado;

use aureon_shared::logging::inicializar_logs;
use tracing::info;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    inicializar_logs("info");
    info!(componente = "aureon-config", "Iniciando AUREON Config v0.0.1");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(estado::inicializar_estado())
        .invoke_handler(tauri::generate_handler![
            commands::testar_postgres,
            commands::inicializar_keystore,
            commands::criar_banco_empresa,
            commands::finalizar_instalacao,
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar o Aureon Config");
}
