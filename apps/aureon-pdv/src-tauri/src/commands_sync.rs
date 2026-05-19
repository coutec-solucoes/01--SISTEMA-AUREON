use tauri::State;
use tracing::{info, error, warn};
use serde_json::json;
use reqwest::Client;
use std::time::Duration;
use uuid::Uuid;

use aureon_core::{dtos::*, RespostaBase};
use crate::estado::EstadoApp;

#[derive(serde::Deserialize)]
pub struct ConfigurarServidorDto {
    pub host_api: String,
    pub porta_api: u16,
}

#[tauri::command]
pub async fn configurar_servidor_sync(
    dto: ConfigurarServidorDto,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<()>, String> {
    info!(componente = "aureon-pdv::commands_sync", host = %dto.host_api, porta = dto.porta_api, "Chamada: configurar_servidor_sync");
    
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO terminal_local (id, codigo_terminal, nome_terminal, chave_terminal, host_api, porta_api)
         VALUES (1, 'PENDENTE', 'PENDENTE', '', ?1, ?2)
         ON CONFLICT(id) DO UPDATE SET host_api=excluded.host_api, porta_api=excluded.porta_api, atualizado_em=datetime('now')",
        rusqlite::params![dto.host_api, dto.porta_api],
    ).map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Servidor configurado localmente", ()))
}

#[tauri::command]
pub async fn testar_conexao_sync(
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<bool>, String> {
    info!(componente = "aureon-pdv::commands_sync", "Chamada: testar_conexao_sync");

    let host = {
        let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT host_api, porta_api FROM terminal_local WHERE id = 1").map_err(|e| e.to_string())?;
        stmt.query_row([], |row| {
            let h: String = row.get(0)?;
            Ok(h)
        }).unwrap_or("http://localhost:7000".to_string())
    };

    let url = format!("{}/sync/diagnostico", host);
    let client = Client::builder().timeout(Duration::from_secs(5)).build().map_err(|e| e.to_string())?;

    match client.get(&url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok(RespostaBase::ok("Conexão bem sucedida", true))
            } else {
                Ok(RespostaBase::ok("Erro na API de Sincronização", false))
            }
        },
        Err(e) => {
            error!(componente = "aureon-pdv::commands_sync", erro = %e, "Erro ao testar conexão");
            Ok(RespostaBase::ok("Servidor inacessível", false))
        }
    }
}

#[tauri::command]
pub async fn registrar_terminal(
    req: RegistroTerminalReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<RegistroTerminalResp>, String> {
    info!(componente = "aureon-pdv::commands_sync", codigo = %req.codigo_terminal, "Chamada: registrar_terminal");

    let host = {
        let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT host_api, porta_api FROM terminal_local WHERE id = 1").map_err(|e| e.to_string())?;
        stmt.query_row([], |row| {
            let h: String = row.get(0)?;
            Ok(h)
        }).map_err(|e| e.to_string())?
    };

    let url = format!("{}/sync/terminais/registrar", host);
    let client = Client::builder().timeout(Duration::from_secs(10)).build().map_err(|e| e.to_string())?;

    let response = client.post(&url).json(&req).send().await.map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let r: RespostaBase<RegistroTerminalResp> = response.json().await.map_err(|e| e.to_string())?;
        let dados = r.dados.clone().unwrap();

        let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE terminal_local SET 
                codigo_terminal = ?1,
                nome_terminal = ?2,
                identificador_maquina = ?3,
                chave_terminal = ?4,
                atualizado_em = datetime('now')
             WHERE id = 1",
            rusqlite::params![req.codigo_terminal, req.nome_terminal, req.identificador_maquina, dados.chave_terminal],
        ).map_err(|e| e.to_string())?;

        Ok(RespostaBase::ok("Terminal registrado/atualizado", dados))
    } else {
        Err(format!("Erro ao registrar terminal na API local: {}", response.status()))
    }
}

#[tauri::command]
pub async fn verificar_status_terminal(
    codigo_terminal: String,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<StatusTerminalResp>, String> {
    info!(componente = "aureon-pdv::commands_sync", codigo = %codigo_terminal, "Chamada: verificar_status_terminal");

    let host = {
        let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT host_api FROM terminal_local WHERE id = 1").map_err(|e| e.to_string())?;
        stmt.query_row([], |row| row.get::<_, String>(0)).unwrap_or("http://localhost:7000".to_string())
    };

    let url = format!("{}/sync/terminais/{}/status", host, codigo_terminal);
    let client = Client::builder().timeout(Duration::from_secs(10)).build().map_err(|e| e.to_string())?;

    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let r: RespostaBase<StatusTerminalResp> = response.json().await.map_err(|e| e.to_string())?;
        let dados = r.dados.clone().unwrap();

        let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE terminal_local SET 
                autorizado = ?1,
                primeiro_sync_concluido = ?2,
                atualizado_em = datetime('now')
             WHERE id = 1",
            rusqlite::params![if dados.autorizado { 1 } else { 0 }, if dados.primeiro_sync_concluido { 1 } else { 0 }],
        ).map_err(|e| e.to_string())?;

        Ok(RespostaBase::ok("Status do terminal obtido", dados))
    } else {
        Err(format!("Erro ao verificar status na API local: {}", response.status()))
    }
}

#[tauri::command]
pub async fn executar_primeira_sincronizacao(
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<PacoteSyncResp>, String> {
    info!(componente = "aureon-pdv::commands_sync", "Chamada: executar_primeira_sincronizacao");

    let (host, chave_terminal) = {
        let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT host_api, chave_terminal FROM terminal_local WHERE id = 1").map_err(|e| e.to_string())?;
        stmt.query_row([], |row| {
            let h: String = row.get(0)?;
            let c: String = row.get(1)?;
            Ok((h, c))
        }).map_err(|e| e.to_string())?
    };

    let req = PrimeiraSyncReq {
        terminal_id: "ignorado-na-req".into(),
        idempotency_key: Uuid::new_v4().to_string(),
    };

    let url = format!("{}/sync/primeira-sincronizacao", host);
    let client = Client::builder().timeout(Duration::from_secs(30)).build().map_err(|e| e.to_string())?;

    let response = client.post(&url)
        .header("Authorization", format!("Bearer {}", chave_terminal))
        .json(&req)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let r: RespostaBase<PacoteSyncResp> = response.json().await.map_err(|e| e.to_string())?;
        let dados = r.dados.clone().unwrap();

        Ok(RespostaBase::ok("Pacote recebido", dados))
    } else {
        Err(format!("Erro ao executar primeira sync: {}", response.status()))
    }
}

#[tauri::command]
pub async fn aplicar_pacote_sincronizacao(
    pacote_id: String,
    idempotency_key: String,
    payload: serde_json::Value,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<bool>, String> {
    info!(componente = "aureon-pdv::commands_sync", pacote_id = %pacote_id, "Chamada: aplicar_pacote_sincronizacao");

    let (host_chave_0, host_chave_1) = {
        let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
        
        // Validar idempotencia
        let ja_processado: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sync_idempotencia_local WHERE idempotency_key = ?1",
            rusqlite::params![&idempotency_key],
            |row| row.get(0)
        ).unwrap_or(false);

        if ja_processado {
            warn!("Pacote com idempotency_key {} já processado, ignorando.", idempotency_key);
            return Ok(RespostaBase::ok("Pacote já foi aplicado anteriormente", true));
        }

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // Exemplo de extração do payload de empresa_config
        if let Some(emp_config) = payload.get("empresa_config") {
            if let Some(arr) = emp_config.as_array() {
                if let Some(empresa) = arr.first() {
                    // Inserir na empresa_cache
                    let emp_id = empresa.get("empresa_id").and_then(|v| v.as_str()).unwrap_or("");
                    let cod = empresa.get("codigo").and_then(|v| v.as_str()).unwrap_or("");
                    let nome = empresa.get("nome").and_then(|v| v.as_str()).unwrap_or("");
                    
                    tx.execute(
                        "INSERT INTO empresa_cache (id, empresa_id, codigo, nome) VALUES (1, ?1, ?2, ?3)
                         ON CONFLICT(id) DO UPDATE SET empresa_id=excluded.empresa_id, codigo=excluded.codigo, nome=excluded.nome, atualizado_em=datetime('now')",
                        rusqlite::params![emp_id, cod, nome],
                    ).map_err(|e| e.to_string())?;
                    
                    // Registrar versao aplicadas (mockado na Fase 6 bloco 3)
                    tx.execute(
                        "INSERT INTO sync_versoes_aplicadas (tipo_dado, versao, hash_conteudo, pacote_id) VALUES ('empresa_config', 1, 'hash', ?1)
                         ON CONFLICT(tipo_dado) DO UPDATE SET versao=1, pacote_id=excluded.pacote_id, aplicado_em=datetime('now')",
                        rusqlite::params![&pacote_id],
                    ).map_err(|e| e.to_string())?;
                }
            }
        }

        // Registra na idempotencia
        tx.execute(
            "INSERT INTO sync_idempotencia_local (idempotency_key, operacao, resultado) VALUES (?1, 'APLICAR_PACOTE', 'SUCESSO')",
            rusqlite::params![&idempotency_key],
        ).map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;

        // Extrai host e chave para enviar confirmacao para a API
        let mut stmt = conn.prepare("SELECT host_api, chave_terminal FROM terminal_local WHERE id = 1").map_err(|e| e.to_string())?;
        stmt.query_row([], |row| {
            let h: String = row.get(0)?;
            let c: String = row.get(1)?;
            Ok((h, c))
        }).unwrap_or(("http://localhost:7000".to_string(), "".to_string()))
    };

    let req = ConfirmacaoAplicacaoReq {
        pacote_id: pacote_id.clone(),
        terminal_id: "terminal_id_aqui".into(),
        idempotency_key: Uuid::new_v4().to_string(),
        sucesso: true,
        erro_detalhes: None,
    };

    let url = format!("{}/sync/confirmar-aplicacao", host_chave_0);
    let client = Client::new();
    let _ = client.post(&url)
        .header("Authorization", format!("Bearer {}", host_chave_1))
        .json(&req)
        .send()
        .await; // ignoramos o erro de confirmacao no bloco 3 por simplicidade, ou logamos

    Ok(RespostaBase::ok("Pacote aplicado com sucesso e transacionado localmente.", true))
}

#[tauri::command]
pub async fn obter_status_sync_local(
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<serde_json::Value>, String> {
    info!(componente = "aureon-pdv::commands_sync", "Chamada: obter_status_sync_local");

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, codigo_terminal, nome_terminal, autorizado, primeiro_sync_concluido, host_api, atualizado_em FROM terminal_local WHERE id = 1").map_err(|e| e.to_string())?;
    let terminal = stmt.query_row([], |row| {
        let id: i32 = row.get(0)?;
        let codigo: String = row.get(1)?;
        let nome: String = row.get(2)?;
        let autorizado: bool = row.get::<_, i32>(3)? == 1;
        let psc: bool = row.get::<_, i32>(4)? == 1;
        let host: String = row.get(5)?;
        let atualizado: String = row.get(6)?;
        
        Ok(json!({
            "id": id,
            "codigo_terminal": codigo,
            "nome_terminal": nome,
            "autorizado": autorizado,
            "primeiro_sync_concluido": psc,
            "host_api": host,
            "atualizado_em": atualizado
        }))
    }).unwrap_or_else(|_| json!({ "status": "Nenhum terminal configurado" }));

    Ok(RespostaBase::ok("Status Local", terminal))
}

#[tauri::command]
pub async fn listar_versoes_aplicadas(
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<serde_json::Value>, String> {
    info!(componente = "aureon-pdv::commands_sync", "Chamada: listar_versoes_aplicadas");

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT tipo_dado, versao, hash_conteudo, aplicado_em FROM sync_versoes_aplicadas").map_err(|e| e.to_string())?;
    let iter = stmt.query_map([], |row| {
        let tipo: String = row.get(0)?;
        let versao: i32 = row.get(1)?;
        let hash: String = row.get(2)?;
        let aplicado: String = row.get(3)?;
        Ok(json!({
            "tipo_dado": tipo,
            "versao": versao,
            "hash_conteudo": hash,
            "aplicado_em": aplicado
        }))
    }).map_err(|e| e.to_string())?;

    let mut versoes = Vec::new();
    for v in iter {
        if let Ok(val) = v { versoes.push(val); }
    }

    Ok(RespostaBase::ok("Versões aplicadas localmente", json!(versoes)))
}

#[tauri::command]
pub async fn limpar_cache_sync_dev(
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<bool>, String> {
    info!(componente = "aureon-pdv::commands_sync", "Chamada: limpar_cache_sync_dev");

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // Limpar cache de dev
    tx.execute_batch("
        DELETE FROM sync_versoes_aplicadas;
        DELETE FROM sync_idempotencia_local;
        DELETE FROM empresa_cache;
        DELETE FROM terminal_local;
    ").map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("Cache limpo (DEV)", true))
}
