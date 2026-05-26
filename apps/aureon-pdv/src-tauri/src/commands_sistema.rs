use tauri::State;
use std::fs;
use std::path::{Path, PathBuf};
use aureon_core::dtos::DiagnosticoSistemaResp;
use crate::estado::EstadoApp;

fn testar_escrita(caminho: &Path) -> bool {
    if !caminho.exists() {
        return false;
    }
    let arquivo_teste = caminho.join(".teste_escrita_aureon");
    match fs::write(&arquivo_teste, b"teste") {
        Ok(_) => {
            let _ = fs::remove_file(arquivo_teste);
            true
        }
        Err(_) => false,
    }
}

// Em Windows, uma estimativa simples do espaço livre da partição
// Para simplicidade, podemos obter isso do fs2, ou omitir.
fn obter_espaco_livre(caminho: &Path) -> Option<u64> {
    #[cfg(target_os = "windows")]
    {
        // Alternativamente, no Windows podemos usar APIs nativas ou omitir se for complexo,
        // mas vamos retornar None temporariamente ou implementar um mock de sistema
        None
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[tauri::command]
pub async fn diagnosticar_instalacao_sistema(
    estado: State<'_, EstadoApp>,
) -> Result<DiagnosticoSistemaResp, String> {
    
    let sys_os = std::env::consts::OS.to_string();
    let sys_arch = std::env::consts::ARCH.to_string();
    let app_version = env!("CARGO_PKG_VERSION").to_string();
    
    let base_dir = std::path::PathBuf::from("C:/Aureon");
    let caminho_banco = base_dir.join("data/aureon-local.db");
    let caminho_backups = base_dir.join("backups");
    let caminho_logs = base_dir.join("logs");
    let caminho_print_sim = base_dir.join("print-sim");
    let caminho_diagnostics = base_dir.join("diagnostics");

    let mut warnings = vec![];

    let mut pastas_ok = true;
    if !base_dir.exists() { pastas_ok = false; warnings.push("Pasta base (C:/Aureon) não encontrada.".to_string()); }
    if !base_dir.join("data").exists() { pastas_ok = false; warnings.push("Pasta data não encontrada.".to_string()); }
    if !caminho_backups.exists() { pastas_ok = false; warnings.push("Pasta backups não encontrada.".to_string()); }
    if !caminho_logs.exists() { pastas_ok = false; warnings.push("Pasta logs não encontrada.".to_string()); }
    if !caminho_print_sim.exists() { pastas_ok = false; warnings.push("Pasta print-sim não encontrada.".to_string()); }
    if !caminho_diagnostics.exists() { pastas_ok = false; warnings.push("Pasta diagnostics não encontrada.".to_string()); }

    let pode_escrever_base = if base_dir.exists() { testar_escrita(&base_dir) } else { false };
    if !pode_escrever_base {
        warnings.push("Sem permissão de escrita na pasta base.".to_string());
    }

    let pode_escrever_backups = if caminho_backups.exists() { testar_escrita(&caminho_backups) } else { false };
    if pastas_ok && !pode_escrever_backups {
        warnings.push("Sem permissão de escrita na pasta de backups.".to_string());
    }

    let banco_existe = caminho_banco.exists();
    if !banco_existe {
        warnings.push("Banco de dados local não encontrado!".to_string());
    }

    let espaco_livre = obter_espaco_livre(&base_dir);

    Ok(DiagnosticoSistemaResp {
        sucesso: true,
        sistema_operacional: sys_os,
        arquitetura: sys_arch,
        app_versao: app_version,
        caminho_base: base_dir.to_string_lossy().to_string(),
        caminho_banco: caminho_banco.to_string_lossy().to_string(),
        caminho_backups: caminho_backups.to_string_lossy().to_string(),
        caminho_logs: caminho_logs.to_string_lossy().to_string(),
        caminho_print_sim: caminho_print_sim.to_string_lossy().to_string(),
        espaco_livre_bytes: espaco_livre,
        pode_escrever_base,
        pode_escrever_backups,
        banco_existe,
        pastas_ok,
        mensagem: if warnings.is_empty() { "Diagnóstico concluído sem alertas.".to_string() } else { "Diagnóstico concluído com alertas.".to_string() },
        warnings,
    })
}

#[tauri::command]
pub async fn garantir_pastas_sistema() -> Result<bool, String> {
    let base_dir = std::path::PathBuf::from("C:/Aureon");
    
    let dirs = vec![
        base_dir.clone(),
        base_dir.join("data"),
        base_dir.join("backups"),
        base_dir.join("logs"),
        base_dir.join("print-sim"),
        base_dir.join("diagnostics"),
    ];

    for dir in dirs {
        if !dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&dir) {
                return Err(format!("Falha ao criar diretório {}: {}", dir.display(), e));
            }
        }
    }

    Ok(true)
}

#[tauri::command]
pub async fn obter_versao_app() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}
