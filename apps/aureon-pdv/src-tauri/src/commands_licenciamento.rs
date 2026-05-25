use tauri::State;
use aureon_core::AureonError;
use aureon_core::dtos::{LicencaStatusResp, AtivarLicencaReq};
use crate::estado::AppState;
use tracing::{info, warn, error};
use rusqlite::OptionalExtension;
use uuid::Uuid;

#[tauri::command]
pub fn obter_status_licenca(state: State<AppState>) -> Result<LicencaStatusResp, AureonError> {
    info!(componente = "commands_licenciamento", "Consultando status da licenca local");
    let conn_guard = state.db.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
    let conn = conn_guard.as_ref().ok_or_else(|| AureonError::Interno("ConexaoFechada".to_string()))?;

    // Busca instalação
    let mut stmt = conn.prepare("SELECT installation_id, empresa_id, terminal_id, terminal_nome FROM instalacao_local LIMIT 1").map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    
    let instalacao = stmt.query_row([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?
        ))
    }).optional().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    if let Some((installation_id, empresa_id, terminal_id, terminal_nome)) = instalacao {
        // Busca licença
        let mut stmt_lic = conn.prepare("SELECT plano_codigo, status, modo, validade_fim, tolerancia_offline_dias, bloqueio_total, motivo_bloqueio FROM licenca_local WHERE installation_id = ?1").map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
        let licenca = stmt_lic.query_row([&installation_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, Option<String>>(6)?
            ))
        }).optional().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

        if let Some((plano_codigo, status, modo, validade_fim, tolerancia_offline_dias, bloqueio_total, motivo_bloqueio)) = licenca {
            let mut pode_operar = true;
            let mut mensagem = Some("Licença válida.".to_string());

            if modo == "DEV" {
                pode_operar = true;
                mensagem = Some("Modo desenvolvedor ativado.".to_string());
            } else if status == "BLOQUEADA" && bloqueio_total == 1 {
                pode_operar = false;
                mensagem = Some(motivo_bloqueio.clone().unwrap_or("Terminal bloqueado.".to_string()));
            } else if status == "EXPIRADA" {
                pode_operar = false;
                mensagem = Some("Licença expirada.".to_string());
            }

            // atualiza o ultimo_check
            let _ = conn.execute("UPDATE licenca_local SET ultimo_check_em = datetime('now') WHERE installation_id = ?1", rusqlite::params![installation_id]);

            return Ok(LicencaStatusResp {
                installation_id,
                empresa_id,
                terminal_id,
                terminal_nome,
                plano_codigo,
                status,
                modo,
                validade_fim,
                dias_restantes: None, // a implementar calc datas
                tolerancia_offline_dias,
                bloqueio_total,
                motivo_bloqueio,
                pode_operar,
                mensagem,
            });
        }
    }

    Ok(LicencaStatusResp {
        installation_id: "".to_string(),
        empresa_id: None,
        terminal_id: None,
        terminal_nome: None,
        plano_codigo: "".to_string(),
        status: "PENDENTE_ATIVACAO".to_string(),
        modo: "".to_string(),
        validade_fim: None,
        dias_restantes: None,
        tolerancia_offline_dias: 10,
        bloqueio_total: 1,
        motivo_bloqueio: Some("Instalação não ativada.".to_string()),
        pode_operar: false,
        mensagem: Some("Nenhuma licença local encontrada.".to_string()),
    })
}

#[tauri::command]
pub fn ativar_licenca_dev(req: AtivarLicencaReq, state: State<AppState>) -> Result<LicencaStatusResp, AureonError> {
    info!(componente = "commands_licenciamento", "Ativando licenca: modo={}", req.modo);
    let conn_guard = state.db.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
    let conn = conn_guard.as_ref().ok_or_else(|| AureonError::Interno("ConexaoFechada".to_string()))?;

    let instalacao_id = Uuid::new_v4().to_string();
    let terminal_id = Uuid::new_v4().to_string();
    let lic_id = Uuid::new_v4().to_string();
    
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let status_lic = if req.modo == "DEV" { "MODO_DEV" } else { "ATIVA" };

    // limpa antes
    conn.execute("DELETE FROM licenca_local", []).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    conn.execute("DELETE FROM instalacao_local", []).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    conn.execute(
        "INSERT INTO instalacao_local (id, installation_id, empresa_id, terminal_id, terminal_nome, dispositivo_hash, sistema_operacional, criado_em, atualizado_em)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        rusqlite::params![Uuid::new_v4().to_string(), instalacao_id, req.empresa_id, terminal_id, req.terminal_nome, "hash_dummy", "Windows", now],
    ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    conn.execute(
        "INSERT INTO licenca_local (id, installation_id, empresa_id, plano_codigo, status, modo, validade_inicio, validade_fim, ultimo_check_em, tolerancia_offline_dias, bloqueio_total, criado_em, atualizado_em)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 10, 0, ?10, ?10)",
        rusqlite::params![lic_id, instalacao_id, req.empresa_id, "PLANO_BASICO", status_lic, req.modo, now, "2099-12-31 23:59:59", now, now],
    ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    // evento
    conn.execute(
        "INSERT INTO licenca_eventos (id, installation_id, licenca_id, tipo_evento, status_novo, criado_em)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![Uuid::new_v4().to_string(), instalacao_id, lic_id, "LICENCA_CRIADA", status_lic, now],
    ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    drop(conn_guard);
    obter_status_licenca(state)
}

#[tauri::command]
pub fn registrar_evento_licenca(tipo_evento: String, msg: String, state: State<AppState>) -> Result<(), AureonError> {
    let conn_guard = state.db.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
    let conn = conn_guard.as_ref().ok_or_else(|| AureonError::Interno("ConexaoFechada".to_string()))?;
    
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO licenca_eventos (id, tipo_evento, mensagem, criado_em) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![Uuid::new_v4().to_string(), tipo_evento, msg, now],
    ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    
    Ok(())
}

#[tauri::command]
pub fn obter_identidade_instalacao(state: State<AppState>) -> Result<String, AureonError> {
    let conn_guard = state.db.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
    let conn = conn_guard.as_ref().ok_or_else(|| AureonError::Interno("ConexaoFechada".to_string()))?;
    
    let mut stmt = conn.prepare("SELECT installation_id FROM instalacao_local LIMIT 1").map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    let instalacao = stmt.query_row([], |row| row.get::<_, String>(0)).optional().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    
    Ok(instalacao.unwrap_or("".to_string()))
}
