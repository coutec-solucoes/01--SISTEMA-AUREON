use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use tracing::{info, error, warn};
use chrono::Utc;
use sha2::{Sha256, Digest};
use std::io::Read;

use aureon_core::RespostaBase;
use aureon_core::dtos::{
    CriarBackupReq, BackupResp, ValidarBackupReq, ValidarBackupResp, 
    RestaurarBackupReq, RestaurarBackupResp, DiagnosticoBancoResp, BackupInfoResp
};
use crate::estado::EstadoApp;

const BACKUP_DIR_PADRAO: &str = "C:/Aureon/backups";

fn obter_dir_backups() -> PathBuf {
    if let Ok(dir) = std::env::var("AUREON_BACKUP_DIR") {
        PathBuf::from(dir)
    } else {
        PathBuf::from(BACKUP_DIR_PADRAO)
    }
}

fn calcular_sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("Erro ao abrir arquivo para hash: {}", e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(|e| format!("Erro ao ler arquivo: {}", e))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[tauri::command]
pub async fn criar_backup_local(
    req: CriarBackupReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<BackupResp>, String> {
    info!("Chamada: criar_backup_local");

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    // Obter informacoes de identidade do banco para metadados
    let mut empresa_id = String::new();
    let mut installation_id = String::new();
    let mut terminal_id = String::new();

    let _ = conn.query_row(
        "SELECT empresa_id, id as installation_id, terminal_id FROM instalacao_local LIMIT 1",
        [],
        |row| {
            empresa_id = row.get(0).unwrap_or_default();
            installation_id = row.get(1).unwrap_or_default();
            terminal_id = row.get(2).unwrap_or_default();
            Ok(())
        }
    );

    // Se nao encontrar dados, pode ser banco vazio, mas ainda faremos backup
    let inst_suffix = if installation_id.len() >= 8 { &installation_id[..8] } else { "none" };
    let agora = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    
    let nome_arquivo = format!("aureon_bkp_{}_{}_{}.db", agora, terminal_id, inst_suffix);
    
    let dest_dir = if let Some(dir) = req.destino_dir {
        PathBuf::from(dir)
    } else {
        obter_dir_backups()
    };

    if !dest_dir.exists() {
        fs::create_dir_all(&dest_dir).map_err(|e| format!("Erro ao criar pasta de backup: {}", e))?;
    }

    let db_path = estado.dados_dir.join("aureon-local.db");
    let dest_path = dest_dir.join(&nome_arquivo);

    // SQLite backup via vacuum inside a transaction lock
    // For simplicity and safety we use the online backup API or just copy if no lock
    // Since we locked conn, we can use vacuum into.
    let sql = format!("VACUUM INTO '{}'", dest_path.display().to_string().replace("\\", "/"));
    conn.execute(&sql, []).map_err(|e| format!("Erro ao gerar backup via VACUUM INTO: {}", e))?;

    let meta = fs::metadata(&dest_path).map_err(|e| e.to_string())?;
    let tamanho = meta.len();
    let hash = calcular_sha256(&dest_path)?;

    let mut metadados_arquivo = None;
    if req.incluir_metadados {
        let meta_name = format!("{}.json", nome_arquivo);
        let meta_path = dest_dir.join(&meta_name);
        
        let mut warnings = vec![];
        if let Some(motivo) = req.motivo {
            warnings.push(format!("Motivo: {}", motivo));
        }

        let json = serde_json::json!({
            "arquivo": nome_arquivo,
            "tamanho_bytes": tamanho,
            "sha256": hash,
            "criado_em": Utc::now().to_rfc3339(),
            "empresa_id": empresa_id,
            "installation_id": installation_id,
            "terminal_id": terminal_id,
            "motivo": warnings
        });
        
        fs::write(&meta_path, serde_json::to_string_pretty(&json).unwrap())
            .map_err(|e| format!("Erro ao escrever metadados: {}", e))?;
        metadados_arquivo = Some(meta_name);
    }

    let resp = BackupResp {
        sucesso: true,
        backup_id: nome_arquivo.clone(),
        arquivo: nome_arquivo,
        metadados_arquivo,
        tamanho_bytes: tamanho,
        sha256: hash,
        criado_em: Utc::now().to_rfc3339(),
        mensagem: "Backup finalizado".to_string(),
        warnings: vec![],
    };

    Ok(RespostaBase {
        sucesso: true,
        mensagem: Some("Backup gerado com sucesso".to_string()),
        dados: Some(resp),
        erro: None,
    })
}

#[tauri::command]
pub async fn listar_backups_locais() -> Result<RespostaBase<Vec<BackupInfoResp>>, String> {
    let dir = obter_dir_backups();
    if !dir.exists() {
        return Ok(RespostaBase { sucesso: true, mensagem: Some("".to_string()), dados: Some(vec![]), erro: None });
    }

    let mut lista = vec![];
    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().unwrap_or_default() == "db" {
            let nome = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let meta = fs::metadata(&path).unwrap();
            
            // Tentar ler json se existir
            let json_path = path.with_extension("db.json");
            let mut empresa_id = None;
            let mut installation_id = None;
            let mut terminal_id = None;
            let mut metadados_arquivo = None;
            
            if json_path.exists() {
                if let Ok(conteudo) = fs::read_to_string(&json_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&conteudo) {
                        empresa_id = json["empresa_id"].as_str().map(|s| s.to_string());
                        installation_id = json["installation_id"].as_str().map(|s| s.to_string());
                        terminal_id = json["terminal_id"].as_str().map(|s| s.to_string());
                        metadados_arquivo = Some(json_path.file_name().unwrap().to_string_lossy().to_string());
                    }
                }
            }

            lista.push(BackupInfoResp {
                backup_id: nome.clone(),
                arquivo: nome,
                metadados_arquivo,
                tamanho_bytes: meta.len(),
                sha256: "".to_string(), // Omitido na listagem para rapidez, ou calcular se necessario
                criado_em: "".to_string(),
                empresa_id,
                installation_id,
                terminal_id,
                app_versao: None,
                valido: None,
                mensagem: None,
            });
        }
    }

    Ok(RespostaBase {
        sucesso: true,
        mensagem: Some("".to_string()),
        dados: Some(lista),
        erro: None,
    })
}

#[tauri::command]
pub async fn validar_backup_local(
    req: ValidarBackupReq,
) -> Result<RespostaBase<ValidarBackupResp>, String> {
    info!("Chamada: validar_backup_local");

    let dir = obter_dir_backups();
    let db_path = dir.join(&req.arquivo);

    if !db_path.exists() {
        return Err("Arquivo de backup nao encontrado.".into());
    }

    let meta = fs::metadata(&db_path).map_err(|e| e.to_string())?;
    let tamanho = meta.len();
    let hash = calcular_sha256(&db_path)?;

    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
    
    let mut sqlite_integrity_ok = false;
    let mut migrations_ok = false;
    let mut warnings = vec![];

    // PRAGMA integrity_check
    match conn.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0)) {
        Ok(res) if res == "ok" => sqlite_integrity_ok = true,
        Ok(res) => warnings.push(format!("Integrity falhou: {}", res)),
        Err(e) => warnings.push(format!("Erro ao checar integridade: {}", e)),
    }

    // Checar migrations
    let count: Result<i64, _> = conn.query_row("SELECT COUNT(*) FROM auth_roles", [], |row| row.get(0));
    if count.is_ok() {
        migrations_ok = true; // Simples teste de existencia de tabela comum
    } else {
        warnings.push("Tabela de migrations/estrutura base ausente.".to_string());
    }

    let valido = sqlite_integrity_ok && migrations_ok;
    let msg = if valido { "Backup valido." } else { "Backup corrompido ou invalido." };

    let resp = ValidarBackupResp {
        valido,
        arquivo: req.arquivo,
        tamanho_bytes: tamanho,
        sha256: hash,
        sqlite_integrity_ok,
        migrations_ok,
        mensagem: msg.to_string(),
        warnings,
    };

    Ok(RespostaBase {
        sucesso: true,
        mensagem: Some(msg.to_string()),
        dados: Some(resp),
        erro: None,
    })
}

#[tauri::command]
pub async fn restaurar_backup_local(
    req: RestaurarBackupReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<RestaurarBackupResp>, String> {
    info!("Chamada: restaurar_backup_local");

    if req.confirmacao_texto != "RESTAURAR" {
        return Err("Confirmacao invalida. Digite RESTAURAR para prosseguir.".into());
    }

    let dir = obter_dir_backups();
    let bkp_path = dir.join(&req.arquivo);

    if !bkp_path.exists() {
        return Err("Arquivo de backup nao encontrado.".into());
    }

    // Validar integridade primeiro
    let conn_teste = rusqlite::Connection::open(&bkp_path).map_err(|e| format!("Erro ao ler backup: {}", e))?;
    let integridade_teste: String = conn_teste.query_row("PRAGMA integrity_check", [], |row| row.get(0)).unwrap_or_default();
    if integridade_teste != "ok" {
        return Err("O backup selecionado esta corrompido e falhou no teste de integridade.".into());
    }
    drop(conn_teste);

    // Gerar backup automatico do estado atual
    let mut backup_pre = None;
    let req_bkp = CriarBackupReq {
        destino_dir: None,
        motivo: Some("Backup automatico pre-restauracao".to_string()),
        incluir_metadados: true,
    };
    if let Ok(res_bkp) = criar_backup_local(req_bkp, estado.clone()).await {
        if let Some(dados) = res_bkp.dados {
            backup_pre = Some(dados.arquivo);
        }
    }

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    // Close and overwrite... Actually we can't overwrite an open file in Windows easily, 
    // or SQLite might be locked. 
    // The safest way via active connection is RESTORE from SQLite Online Backup API,
    // but rusqlite has `backup` API.
    // However, the easiest trick without API is to close the app or use ATTACH.
    // In our case, `rusqlite::backup` from one to another:
    
    // We open a new connection to the backup file and push it to main DB.
    let source = rusqlite::Connection::open(&bkp_path).map_err(|e| format!("Erro ao abrir backup: {}", e))?;
    
    let mut b = rusqlite::backup::Backup::new(&source, &mut *conn).map_err(|e| e.to_string())?;
    b.step(-1).map_err(|e| format!("Erro ao restaurar dados: {}", e))?;
    drop(b);
    drop(source);

    // Registrar evento de auditoria
    let _ = conn.execute(
        "INSERT INTO licenca_eventos (licenca_id, tipo, detalhes, criado_em) VALUES ('SISTEMA', 'RESTORE_EXECUTADO', ?1, ?2)",
        rusqlite::params![&req.arquivo, Utc::now().to_rfc3339()]
    );

    let resp = RestaurarBackupResp {
        sucesso: true,
        backup_restaurado: req.arquivo,
        backup_pre_restauracao: backup_pre,
        mensagem: "Banco de dados restaurado. Reinicie o sistema se necessario.".to_string(),
        warnings: vec![],
    };

    Ok(RespostaBase {
        sucesso: true,
        mensagem: Some("Restauracao concluida com sucesso.".to_string()),
        erro: Some(e),
    })
}

#[tauri::command]
pub async fn diagnosticar_banco_local(
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<DiagnosticoBancoResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let db_path = estado.dados_dir.join("aureon-local.db");
    let mut warnings = vec![];

    let integridade: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0)).unwrap_or_default();
    let sqlite_integrity_ok = integridade == "ok";

    if !sqlite_integrity_ok {
        warnings.push(format!("Integrity falhou: {}", integridade));
    }

    let meta = fs::metadata(&db_path).map_err(|e| e.to_string())?;
    
    // Test migrations
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM auth_roles", [], |row| row.get(0)).unwrap_or(0);

    Ok(RespostaBase {
        sucesso: true,
        mensagem: None,
        dados: Some(DiagnosticoBancoResp {
            sqlite_integrity_ok,
            tamanho_bytes: meta.len(),
            caminho_banco: db_path.display().to_string(),
            migrations_count: count,
            ultima_migration: None,
            mensagem: "Diagnostico concluido".to_string(),
            warnings,
        }),
        erro: Some(e),
    })
}
