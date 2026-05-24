use std::sync::Mutex;
use tauri::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rusqlite::{params, OptionalExtension};
use crate::estado::EstadoApp;

// -----------------------------------------------------------------------------
// DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct AplicarPacoteFiscalReq {
    pub pacote_id: Option<String>,
    pub versao: String,
    pub payload_hash: String,
    pub payload_json: String,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusVersaoFiscalResp {
    pub versao_atual: Option<String>,
    pub pacote_id: Option<String>,
    pub payload_hash: Option<String>,
    pub status: Option<String>,
    pub total_registros: i64,
    pub aplicado_em: Option<String>,
    pub ultimo_erro: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogSyncFiscalResp {
    pub id: String,
    pub pacote_id: Option<String>,
    pub versao: Option<String>,
    pub tipo_evento: String,
    pub mensagem: Option<String>,
    pub criado_em: String,
}

// -----------------------------------------------------------------------------
// Helpers de Log
// -----------------------------------------------------------------------------

fn log_sync_fiscal(
    tx: &rusqlite::Transaction,
    pacote_id: Option<&str>,
    versao: Option<&str>,
    tipo_evento: &str,
    mensagem: &str,
    payload_preview: Option<&str>,
) -> Result<(), String> {
    let id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO fiscal_sync_logs (id, pacote_id, versao, tipo_evento, mensagem, payload_preview)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, pacote_id, versao, tipo_evento, mensagem, payload_preview],
    ).map_err(|e| format!("Erro ao gravar log fiscal: {}", e))?;
    Ok(())
}

fn log_sync_fiscal_db(
    conn: &rusqlite::Connection,
    pacote_id: Option<&str>,
    versao: Option<&str>,
    tipo_evento: &str,
    mensagem: &str,
    payload_preview: Option<&str>,
) {
    let id = Uuid::new_v4().to_string();
    let _ = conn.execute(
        "INSERT INTO fiscal_sync_logs (id, pacote_id, versao, tipo_evento, mensagem, payload_preview)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, pacote_id, versao, tipo_evento, mensagem, payload_preview],
    );
}

// -----------------------------------------------------------------------------
// Commands
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn validar_pacote_fiscal_local(req: AplicarPacoteFiscalReq) -> Result<bool, String> {
    // Apenas tenta fazer o parse do JSON para validar a estrutura e blocos
    let payload: serde_json::Value = serde_json::from_str(&req.payload_json)
        .map_err(|e| format!("JSON inválido: {}", e))?;

    let blocos = payload.get("blocos").ok_or("Blocos ausentes no payload fiscal")?;
    
    // Validar presença de blocos mínimos esperados
    if blocos.get("fiscal_empresa_config").is_none() { return Err("Bloco fiscal_empresa_config ausente".into()); }
    if blocos.get("fiscal_ncm").is_none() { return Err("Bloco fiscal_ncm ausente".into()); }
    if blocos.get("fiscal_cfop").is_none() { return Err("Bloco fiscal_cfop ausente".into()); }
    if blocos.get("fiscal_cst_csosn").is_none() { return Err("Bloco fiscal_cst_csosn ausente".into()); }
    if blocos.get("fiscal_iva").is_none() { return Err("Bloco fiscal_iva ausente".into()); }
    if blocos.get("fiscal_regras_tributarias").is_none() { return Err("Bloco fiscal_regras_tributarias ausente".into()); }

    Ok(true)
}

#[tauri::command]
pub fn obter_status_versao_fiscal_local(estado: State<'_, EstadoApp>) -> Result<StatusVersaoFiscalResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let row = conn.query_row(
        "SELECT versao, pacote_id, payload_hash, status, total_registros, aplicado_em, erro
         FROM fiscal_versoes_aplicadas_cache
         ORDER BY atualizado_em DESC LIMIT 1",
        [],
        |row| {
            Ok(StatusVersaoFiscalResp {
                versao_atual: row.get(0)?,
                pacote_id: row.get(1)?,
                payload_hash: row.get(2)?,
                status: row.get(3)?,
                total_registros: row.get(4)?,
                aplicado_em: row.get(5)?,
                ultimo_erro: row.get(6)?,
            })
        }
    ).optional().map_err(|e| e.to_string())?;

    match row {
        Some(s) => Ok(s),
        None => Ok(StatusVersaoFiscalResp {
            versao_atual: None,
            pacote_id: None,
            payload_hash: None,
            status: None,
            total_registros: 0,
            aplicado_em: None,
            ultimo_erro: None,
        })
    }
}

#[tauri::command]
pub fn listar_logs_sync_fiscal(estado: State<'_, EstadoApp>) -> Result<Vec<LogSyncFiscalResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT id, pacote_id, versao, tipo_evento, mensagem, criado_em
         FROM fiscal_sync_logs
         ORDER BY criado_em DESC LIMIT 100"
    ).map_err(|e| e.to_string())?;

    let iter = stmt.query_map([], |row| {
        Ok(LogSyncFiscalResp {
            id: row.get(0)?,
            pacote_id: row.get(1)?,
            versao: row.get(2)?,
            tipo_evento: row.get(3)?,
            mensagem: row.get(4)?,
            criado_em: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut logs = Vec::new();
    for l in iter {
        if let Ok(l) = l { logs.push(l); }
    }

    Ok(logs)
}

#[tauri::command]
pub fn aplicar_pacote_fiscal(estado: State<'_, EstadoApp>, req: AplicarPacoteFiscalReq) -> Result<String, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    log_sync_fiscal_db(&conn, req.pacote_id.as_deref(), Some(&req.versao), "FISCAL_PACOTE_RECEBIDO", "Pacote recebido para aplicação no PDV", None);

    // Validação Idempotência (Hash e Versão)
    let applied: Option<String> = conn.query_row(
        "SELECT status FROM fiscal_versoes_aplicadas_cache WHERE payload_hash = ?1 AND status = 'APLICADO'",
        params![req.payload_hash],
        |row| row.get(0)
    ).optional().map_err(|e| e.to_string())?;

    if applied.is_some() {
        log_sync_fiscal_db(&conn, req.pacote_id.as_deref(), Some(&req.versao), "FISCAL_PACOTE_IGNORADO_IDEMPOTENTE", "Pacote com mesmo hash já aplicado", None);
        return Ok("Pacote ignorado: Idempotência garantida (já aplicado com sucesso).".to_string());
    }

    // Parse do JSON
    let payload: serde_json::Value = match serde_json::from_str(&req.payload_json) {
        Ok(v) => v,
        Err(e) => {
            log_sync_fiscal_db(&conn, req.pacote_id.as_deref(), Some(&req.versao), "FISCAL_PACOTE_ERRO", &format!("JSON inválido: {}", e), None);
            return Err(format!("JSON inválido: {}", e));
        }
    };

    let blocos = payload.get("blocos").ok_or_else(|| {
        log_sync_fiscal_db(&conn, req.pacote_id.as_deref(), Some(&req.versao), "FISCAL_PACOTE_ERRO", "Blocos não encontrados no payload", None);
        "Blocos não encontrados no payload".to_string()
    })?;

    log_sync_fiscal_db(&conn, req.pacote_id.as_deref(), Some(&req.versao), "FISCAL_PACOTE_VALIDADO", "Estrutura do pacote validada com sucesso", None);

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let versao_apl_id = Uuid::new_v4().to_string();

    // Registra tentativa de aplicação
    tx.execute(
        "INSERT INTO fiscal_versoes_aplicadas_cache (id, versao, pacote_id, payload_hash, status, total_registros)
         VALUES (?1, ?2, ?3, ?4, 'PENDENTE', ?5)",
        params![versao_apl_id, req.versao, req.pacote_id, req.payload_hash, payload.get("total_registros").and_then(|v| v.as_i64()).unwrap_or(0)],
    ).map_err(|e| e.to_string())?;

    // --- Aplicar Blocos (UPSERT/DELETE_LOGICO) ---

    // Empresa Config
    if let Some(arr) = blocos.get("fiscal_empresa_config").and_then(|v| v.as_array()) {
        for item in arr {
            let op = item.get("operacao").and_then(|v| v.as_str()).unwrap_or("UPSERT");
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() { continue; }

            if op == "UPSERT" {
                tx.execute(
                    "INSERT INTO fiscal_empresa_cache (id, empresa_id, filial_id, pais_fiscal, regime_fiscal, ambiente, forma_emissao, ativo)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     ON CONFLICT(id) DO UPDATE SET
                     empresa_id=excluded.empresa_id, filial_id=excluded.filial_id, pais_fiscal=excluded.pais_fiscal,
                     regime_fiscal=excluded.regime_fiscal, ambiente=excluded.ambiente, forma_emissao=excluded.forma_emissao, ativo=excluded.ativo, atualizado_em=CURRENT_TIMESTAMP",
                    params![
                        id,
                        item.get("empresa_id").and_then(|v| v.as_str()),
                        item.get("filial_id").and_then(|v| v.as_str()),
                        item.get("pais_fiscal").and_then(|v| v.as_str()).unwrap_or("BR"),
                        item.get("regime_fiscal").and_then(|v| v.as_str()),
                        item.get("ambiente").and_then(|v| v.as_str()).unwrap_or("HOMOLOGACAO"),
                        item.get("forma_emissao").and_then(|v| v.as_str()).unwrap_or("NORMAL"),
                        item.get("ativo").and_then(|v| v.as_bool()).unwrap_or(true)
                    ],
                ).map_err(|e| { format!("Erro empresa_config: {}", e) })?;
            } else if op == "DELETE_LOGICO" {
                tx.execute("UPDATE fiscal_empresa_cache SET ativo = 0, atualizado_em = CURRENT_TIMESTAMP WHERE id = ?1", params![id])
                  .map_err(|e| { e.to_string() })?;
            }
        }
    }

    // Numeração
    if let Some(arr) = blocos.get("fiscal_numeracao").and_then(|v| v.as_array()) {
        for item in arr {
            let op = item.get("operacao").and_then(|v| v.as_str()).unwrap_or("UPSERT");
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() { continue; }

            if op == "UPSERT" {
                tx.execute(
                    "INSERT INTO fiscal_numeracao_cache (id, empresa_id, filial_id, pais_fiscal, tipo_documento, serie, proximo_numero, ativo)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     ON CONFLICT(id) DO UPDATE SET
                     tipo_documento=excluded.tipo_documento, serie=excluded.serie, proximo_numero=excluded.proximo_numero, ativo=excluded.ativo, atualizado_em=CURRENT_TIMESTAMP",
                    params![
                        id,
                        item.get("empresa_id").and_then(|v| v.as_str()),
                        item.get("filial_id").and_then(|v| v.as_str()),
                        item.get("pais_fiscal").and_then(|v| v.as_str()).unwrap_or("BR"),
                        item.get("tipo_documento").and_then(|v| v.as_str()),
                        item.get("serie").and_then(|v| v.as_str()),
                        item.get("proximo_numero").and_then(|v| v.as_i64()).unwrap_or(1),
                        item.get("ativo").and_then(|v| v.as_bool()).unwrap_or(true)
                    ],
                ).map_err(|e| { format!("Erro numeracao: {}", e) })?;
            } else if op == "DELETE_LOGICO" {
                tx.execute("UPDATE fiscal_numeracao_cache SET ativo = 0, atualizado_em = CURRENT_TIMESTAMP WHERE id = ?1", params![id])
                  .map_err(|e| { e.to_string() })?;
            }
        }
    }

    // NCM
    if let Some(arr) = blocos.get("fiscal_ncm").and_then(|v| v.as_array()) {
        for item in arr {
            let op = item.get("operacao").and_then(|v| v.as_str()).unwrap_or("UPSERT");
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() { continue; }

            if op == "UPSERT" {
                tx.execute(
                    "INSERT INTO fiscal_ncm_cache (id, codigo, descricao, ativo)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(id) DO UPDATE SET
                     codigo=excluded.codigo, descricao=excluded.descricao, ativo=excluded.ativo, atualizado_em=CURRENT_TIMESTAMP",
                    params![
                        id,
                        item.get("codigo").and_then(|v| v.as_str()).unwrap_or(""),
                        item.get("descricao").and_then(|v| v.as_str()).unwrap_or(""),
                        item.get("ativo").and_then(|v| v.as_bool()).unwrap_or(true)
                    ],
                ).map_err(|e| { format!("Erro ncm: {}", e) })?;
            } else if op == "DELETE_LOGICO" {
                tx.execute("UPDATE fiscal_ncm_cache SET ativo = 0, atualizado_em = CURRENT_TIMESTAMP WHERE id = ?1", params![id])
                  .map_err(|e| { e.to_string() })?;
            }
        }
    }

    // CFOP
    if let Some(arr) = blocos.get("fiscal_cfop").and_then(|v| v.as_array()) {
        for item in arr {
            let op = item.get("operacao").and_then(|v| v.as_str()).unwrap_or("UPSERT");
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() { continue; }

            if op == "UPSERT" {
                tx.execute(
                    "INSERT INTO fiscal_cfop_cache (id, codigo, descricao, tipo_operacao, ativo)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(id) DO UPDATE SET
                     codigo=excluded.codigo, descricao=excluded.descricao, tipo_operacao=excluded.tipo_operacao, ativo=excluded.ativo, atualizado_em=CURRENT_TIMESTAMP",
                    params![
                        id,
                        item.get("codigo").and_then(|v| v.as_str()).unwrap_or(""),
                        item.get("descricao").and_then(|v| v.as_str()).unwrap_or(""),
                        item.get("tipo_operacao").and_then(|v| v.as_str()),
                        item.get("ativo").and_then(|v| v.as_bool()).unwrap_or(true)
                    ],
                ).map_err(|e| { format!("Erro cfop: {}", e) })?;
            } else if op == "DELETE_LOGICO" {
                tx.execute("UPDATE fiscal_cfop_cache SET ativo = 0, atualizado_em = CURRENT_TIMESTAMP WHERE id = ?1", params![id])
                  .map_err(|e| { e.to_string() })?;
            }
        }
    }

    // CST_CSOSN
    if let Some(arr) = blocos.get("fiscal_cst_csosn").and_then(|v| v.as_array()) {
        for item in arr {
            let op = item.get("operacao").and_then(|v| v.as_str()).unwrap_or("UPSERT");
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() { continue; }

            if op == "UPSERT" {
                tx.execute(
                    "INSERT INTO fiscal_cst_csosn_cache (id, codigo, tipo, descricao, ativo)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(id) DO UPDATE SET
                     codigo=excluded.codigo, tipo=excluded.tipo, descricao=excluded.descricao, ativo=excluded.ativo, atualizado_em=CURRENT_TIMESTAMP",
                    params![
                        id,
                        item.get("codigo").and_then(|v| v.as_str()).unwrap_or(""),
                        item.get("tipo").and_then(|v| v.as_str()).unwrap_or("CST"),
                        item.get("descricao").and_then(|v| v.as_str()).unwrap_or(""),
                        item.get("ativo").and_then(|v| v.as_bool()).unwrap_or(true)
                    ],
                ).map_err(|e| { format!("Erro cst_csosn: {}", e) })?;
            } else if op == "DELETE_LOGICO" {
                tx.execute("UPDATE fiscal_cst_csosn_cache SET ativo = 0, atualizado_em = CURRENT_TIMESTAMP WHERE id = ?1", params![id])
                  .map_err(|e| { e.to_string() })?;
            }
        }
    }

    // IVA
    if let Some(arr) = blocos.get("fiscal_iva").and_then(|v| v.as_array()) {
        for item in arr {
            let op = item.get("operacao").and_then(|v| v.as_str()).unwrap_or("UPSERT");
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() { continue; }

            if op == "UPSERT" {
                tx.execute(
                    "INSERT INTO fiscal_iva_cache (id, codigo, descricao, pais_fiscal, aliquota_escala6, ativo)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                     ON CONFLICT(id) DO UPDATE SET
                     codigo=excluded.codigo, descricao=excluded.descricao, pais_fiscal=excluded.pais_fiscal, aliquota_escala6=excluded.aliquota_escala6, ativo=excluded.ativo, atualizado_em=CURRENT_TIMESTAMP",
                    params![
                        id,
                        item.get("codigo").and_then(|v| v.as_str()).unwrap_or(""),
                        item.get("descricao").and_then(|v| v.as_str()).unwrap_or(""),
                        item.get("pais_fiscal").and_then(|v| v.as_str()).unwrap_or("PY"),
                        item.get("aliquota_escala6").and_then(|v| v.as_i64()).unwrap_or(0),
                        item.get("ativo").and_then(|v| v.as_bool()).unwrap_or(true)
                    ],
                ).map_err(|e| { format!("Erro iva: {}", e) })?;
            } else if op == "DELETE_LOGICO" {
                tx.execute("UPDATE fiscal_iva_cache SET ativo = 0, atualizado_em = CURRENT_TIMESTAMP WHERE id = ?1", params![id])
                  .map_err(|e| { e.to_string() })?;
            }
        }
    }

    // REGRAS TRIBUTARIAS
    if let Some(arr) = blocos.get("fiscal_regras_tributarias").and_then(|v| v.as_array()) {
        for item in arr {
            let op = item.get("operacao").and_then(|v| v.as_str()).unwrap_or("UPSERT");
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() { continue; }

            if op == "UPSERT" {
                tx.execute(
                    "INSERT INTO fiscal_regras_tributarias_cache
                     (id, empresa_id, filial_id, pais_fiscal, tipo_operacao, uf_origem, uf_destino,
                      ncm_id, cfop_id, cst_csosn_id, iva_id,
                      aliquota_icms_escala6, aliquota_pis_escala6, aliquota_cofins_escala6, aliquota_iva_escala6, reducao_base_escala6,
                      prioridade, ativo)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)
                     ON CONFLICT(id) DO UPDATE SET
                     tipo_operacao=excluded.tipo_operacao, uf_origem=excluded.uf_origem, uf_destino=excluded.uf_destino,
                     ncm_id=excluded.ncm_id, cfop_id=excluded.cfop_id, cst_csosn_id=excluded.cst_csosn_id, iva_id=excluded.iva_id,
                     aliquota_icms_escala6=excluded.aliquota_icms_escala6, aliquota_pis_escala6=excluded.aliquota_pis_escala6,
                     aliquota_cofins_escala6=excluded.aliquota_cofins_escala6, aliquota_iva_escala6=excluded.aliquota_iva_escala6,
                     reducao_base_escala6=excluded.reducao_base_escala6, prioridade=excluded.prioridade, ativo=excluded.ativo, atualizado_em=CURRENT_TIMESTAMP",
                    params![
                        id,
                        item.get("empresa_id").and_then(|v| v.as_str()),
                        item.get("filial_id").and_then(|v| v.as_str()),
                        item.get("pais_fiscal").and_then(|v| v.as_str()).unwrap_or("BR"),
                        item.get("tipo_operacao").and_then(|v| v.as_str()),
                        item.get("uf_origem").and_then(|v| v.as_str()),
                        item.get("uf_destino").and_then(|v| v.as_str()),
                        item.get("ncm_id").and_then(|v| v.as_str()),
                        item.get("cfop_id").and_then(|v| v.as_str()),
                        item.get("cst_csosn_id").and_then(|v| v.as_str()),
                        item.get("iva_id").and_then(|v| v.as_str()),
                        item.get("aliquota_icms_escala6").and_then(|v| v.as_i64()),
                        item.get("aliquota_pis_escala6").and_then(|v| v.as_i64()),
                        item.get("aliquota_cofins_escala6").and_then(|v| v.as_i64()),
                        item.get("aliquota_iva_escala6").and_then(|v| v.as_i64()),
                        item.get("reducao_base_escala6").and_then(|v| v.as_i64()),
                        item.get("prioridade").and_then(|v| v.as_i64()).unwrap_or(0),
                        item.get("ativo").and_then(|v| v.as_bool()).unwrap_or(true)
                    ],
                ).map_err(|e| { format!("Erro regras: {}", e) })?;
            } else if op == "DELETE_LOGICO" {
                tx.execute("UPDATE fiscal_regras_tributarias_cache SET ativo = 0, atualizado_em = CURRENT_TIMESTAMP WHERE id = ?1", params![id])
                  .map_err(|e| { e.to_string() })?;
            }
        }
    }

    tx.execute(
        "UPDATE fiscal_versoes_aplicadas_cache SET status = 'APLICADO', aplicado_em = CURRENT_TIMESTAMP WHERE id = ?1",
        params![versao_apl_id],
    ).map_err(|e| { e.to_string() })?;

    log_sync_fiscal(&tx, req.pacote_id.as_deref(), Some(&req.versao), "FISCAL_PACOTE_APLICADO", "Pacote aplicado com sucesso no SQLite (Cache PDV)", None)
        .map_err(|e| { e })?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok("Pacote fiscal aplicado com sucesso".to_string())
}
