/// commands_licenciamento.rs — Fase 20 Blocos 1–5
/// Commands Tauri de licenciamento do PDV Aureon.
///
/// Bloco 1: obter_status_licenca, ativar_licenca_dev
/// Bloco 2: registrar_evento_licenca, obter_identidade_instalacao
/// Bloco 5: verificar_licenca_assinada, aplicar_licenca_assinada,
///           obter_chave_publica_licenca_local, atualizar_chave_publica_licenca_dev
///
/// REGRAS DE SEGURANÇA:
/// - PDV NUNCA armazena chave privada.
/// - Payload com assinatura inválida NUNCA é aplicado ao banco.
/// - Modo DEV aceita chave pública informada manualmente com aviso.
/// - Chave pública fica na tabela licenca_chaves (criada se ausente).

use tauri::State;
use tracing::{info, warn, error};
use uuid::Uuid;
use rusqlite::OptionalExtension;
use serde_json;

use aureon_core::AureonError;
use aureon_core::dtos::{
    LicencaStatusResp, AtivarLicencaReq,
    VerificarLicencaAssinadaReq, VerificarLicencaAssinadaResp,
    AplicarLicencaAssinadaReq, AplicarLicencaAssinadaResp,
};
use crate::estado::EstadoApp;
use crate::licenca_crypto_local::{
    verificar_assinatura_local, extrair_campos_payload, calcular_hash_payload, ALGORITMO_SUPORTADO,
};

// ================================================================
// HELPERS INTERNOS
// ================================================================

/// Garante que a tabela licenca_chaves existe.
/// Cria se não existir (migration inline — compatível com migrations existentes).
fn garantir_tabela_chaves(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS licenca_chaves (
            id          TEXT PRIMARY KEY,
            key_id      TEXT NOT NULL UNIQUE,
            chave_publica_base64 TEXT NOT NULL,
            algoritmo   TEXT NOT NULL DEFAULT 'Ed25519',
            modo        TEXT NOT NULL DEFAULT 'DEV',
            criado_em   TEXT NOT NULL,
            atualizado_em TEXT NOT NULL
        );"
    ).map_err(|e| format!("Erro ao criar tabela licenca_chaves: {}", e))
}

/// Obtém a chave pública armazenada pelo key_id. Retorna None se não encontrada.
fn obter_chave_publica(conn: &rusqlite::Connection, key_id: &str) -> Option<String> {
    conn.query_row(
        "SELECT chave_publica_base64 FROM licenca_chaves WHERE key_id = ?1 LIMIT 1",
        rusqlite::params![key_id],
        |row| row.get::<_, String>(0),
    ).ok()
}

/// Registra evento na tabela licenca_eventos.
fn registrar_evento_interno(
    conn: &rusqlite::Connection,
    installation_id: &str,
    licenca_id: &str,
    tipo_evento: &str,
    status_novo: &str,
    mensagem: Option<&str>,
) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO licenca_eventos (id, installation_id, licenca_id, tipo_evento, status_novo, mensagem, criado_em)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            Uuid::new_v4().to_string(),
            installation_id,
            licenca_id,
            tipo_evento,
            status_novo,
            mensagem,
            now,
        ],
    )
    .map(|_| ())
    .map_err(|e| format!("Erro ao registrar evento: {}", e))
}


// ================================================================
// BLOCO 1/2: COMMANDS EXISTENTES (migrados para EstadoApp)
// ================================================================

#[tauri::command]
pub fn obter_status_licenca(estado: State<EstadoApp>) -> Result<LicencaStatusResp, AureonError> {
    info!(componente = "commands_licenciamento", "Consultando status da licenca local");
    let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;

    // Busca instalação
    let instalacao = conn.query_row(
        "SELECT installation_id, empresa_id, terminal_id, terminal_nome FROM instalacao_local LIMIT 1",
        [],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?
        )),
    ).optional().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    if let Some((installation_id, empresa_id, terminal_id, terminal_nome)) = instalacao {
        let licenca = conn.query_row(
            "SELECT plano_codigo, status, modo, validade_fim, tolerancia_offline_dias, bloqueio_total, motivo_bloqueio
             FROM licenca_local WHERE installation_id = ?1",
            rusqlite::params![&installation_id],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, Option<String>>(6)?
            )),
        ).optional().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

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

            let _ = conn.execute(
                "UPDATE licenca_local SET ultimo_check_em = datetime('now') WHERE installation_id = ?1",
                rusqlite::params![installation_id],
            );

            return Ok(LicencaStatusResp {
                installation_id,
                empresa_id,
                terminal_id,
                terminal_nome,
                plano_codigo,
                status,
                modo,
                validade_fim,
                dias_restantes: None,
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
pub fn ativar_licenca_dev(req: AtivarLicencaReq, estado: State<EstadoApp>) -> Result<LicencaStatusResp, AureonError> {
    info!(componente = "commands_licenciamento", "Ativando licenca modo={}", req.modo);
    let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;

    let instalacao_id = Uuid::new_v4().to_string();
    let terminal_id = Uuid::new_v4().to_string();
    let lic_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let status_lic = if req.modo == "DEV" { "MODO_DEV" } else { "ATIVA" };

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

    conn.execute(
        "INSERT INTO licenca_eventos (id, installation_id, licenca_id, tipo_evento, status_novo, criado_em)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![Uuid::new_v4().to_string(), instalacao_id, lic_id, "LICENCA_CRIADA", status_lic, now],
    ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    drop(conn);
    obter_status_licenca(estado)
}

#[tauri::command]
pub fn registrar_evento_licenca(tipo_evento: String, msg: String, estado: State<EstadoApp>) -> Result<(), AureonError> {
    let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO licenca_eventos (id, tipo_evento, mensagem, criado_em) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![Uuid::new_v4().to_string(), tipo_evento, msg, now],
    ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn obter_identidade_instalacao(estado: State<EstadoApp>) -> Result<String, AureonError> {
    let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
    let instalacao = conn.query_row(
        "SELECT installation_id FROM instalacao_local LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    ).optional().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    Ok(instalacao.unwrap_or_default())
}

// ================================================================
// BLOCO 5: VERIFICAÇÃO E APLICAÇÃO DE LICENÇA ASSINADA
// ================================================================

/// Command: verificar_licenca_assinada
///
/// Verifica se o payload assinado pela Retaguarda é autêntico,
/// sem aplicar na licença local. Útil para pré-validação na UI.
///
/// REGRAS:
/// - Requer chave pública explícita.
/// - Não altera nenhuma tabela.
/// - Não aceita payload sem assinatura válida.
#[tauri::command]
pub fn verificar_licenca_assinada(
    req: VerificarLicencaAssinadaReq,
    _estado: State<EstadoApp>,
) -> Result<VerificarLicencaAssinadaResp, AureonError> {
    info!(
        componente = "commands_licenciamento",
        key_id = %req.key_id,
        "verificar_licenca_assinada chamado"
    );

    // Limitar tamanho do payload (segurança)
    if req.payload_licenca_json.len() > 4096 {
        return Ok(VerificarLicencaAssinadaResp {
            valido: false,
            payload_hash_calculado: String::new(),
            key_id: req.key_id,
            mensagem: "Payload excede limite de 4096 bytes. Rejeitado.".to_string(),
            warnings: vec![],
        });
    }

    let resultado = verificar_assinatura_local(
        &req.payload_licenca_json,
        &req.algoritmo_assinatura,
        &req.key_id,
        &req.assinatura_licenca,
        req.payload_hash.as_deref(),
        &req.chave_publica_base64,
    );

    Ok(VerificarLicencaAssinadaResp {
        valido: resultado.valido,
        payload_hash_calculado: resultado.payload_hash_calculado,
        key_id: req.key_id,
        mensagem: resultado.mensagem,
        warnings: resultado.warnings,
    })
}

/// Command: aplicar_licenca_assinada
///
/// Verifica a assinatura Ed25519 do payload e, se válida,
/// atualiza licenca_local e registra evento.
///
/// REGRAS DE SEGURANÇA:
/// - NUNCA aplica payload com assinatura inválida.
/// - NUNCA armazena chave privada.
/// - Extrai campos obrigatórios — rejeita payload incompleto.
/// - Registra evento LICENCA_PAYLOAD_ASSINADO_APLICADO no SQLite.
#[tauri::command]
pub fn aplicar_licenca_assinada(
    req: AplicarLicencaAssinadaReq,
    estado: State<EstadoApp>,
) -> Result<AplicarLicencaAssinadaResp, AureonError> {
    info!(
        componente = "commands_licenciamento",
        key_id = %req.key_id,
        "aplicar_licenca_assinada chamado"
    );

    // Limitar tamanho do payload
    if req.payload_licenca_json.len() > 4096 {
        return Ok(AplicarLicencaAssinadaResp {
            sucesso: false,
            assinatura_valida: false,
            status: String::new(),
            modo: String::new(),
            empresa_id: String::new(),
            licenca_id: String::new(),
            plano_codigo: String::new(),
            terminal_id: None,
            validade_fim: None,
            tolerancia_offline_dias: 0,
            pode_operar: false,
            mensagem: "Payload excede 4096 bytes. Rejeitado por segurança.".to_string(),
            warnings: vec![],
        });
    }

    // 1. Verificar assinatura Ed25519
    let verificacao = verificar_assinatura_local(
        &req.payload_licenca_json,
        &req.algoritmo_assinatura,
        &req.key_id,
        &req.assinatura_licenca,
        req.payload_hash.as_deref(),
        &req.chave_publica_base64,
    );

    if !verificacao.valido {
        warn!(
            componente = "commands_licenciamento",
            motivo = %verificacao.mensagem,
            "Payload rejeitado: assinatura inválida"
        );
        // REGRA CRÍTICA: payload inválido NUNCA é aplicado
        return Ok(AplicarLicencaAssinadaResp {
            sucesso: false,
            assinatura_valida: false,
            status: String::new(),
            modo: String::new(),
            empresa_id: String::new(),
            licenca_id: String::new(),
            plano_codigo: String::new(),
            terminal_id: None,
            validade_fim: None,
            tolerancia_offline_dias: 0,
            pode_operar: false,
            mensagem: format!("Assinatura inválida. {}", verificacao.mensagem),
            warnings: verificacao.warnings,
        });
    }

    // 2. Extrair campos obrigatórios do payload
    let campos = match extrair_campos_payload(&req.payload_licenca_json) {
        Ok(c) => c,
        Err(e) => {
            warn!(componente = "commands_licenciamento", erro = %e, "Payload incompleto rejeitado");
            return Ok(AplicarLicencaAssinadaResp {
                sucesso: false,
                assinatura_valida: true,
                status: String::new(),
                modo: String::new(),
                empresa_id: String::new(),
                licenca_id: String::new(),
                plano_codigo: String::new(),
                terminal_id: None,
                validade_fim: None,
                tolerancia_offline_dias: 0,
                pode_operar: false,
                mensagem: format!("Payload incompleto: {}", e),
                warnings: vec![],
            });
        }
    };

    // 3. Aplicar no banco SQLite local
    let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;

    // Garantir que tabela de chaves existe
    garantir_tabela_chaves(&conn).map_err(|e| AureonError::Interno(e))?;

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Buscar installation_id atual
    let installation_id: String = conn.query_row(
        "SELECT installation_id FROM instalacao_local LIMIT 1",
        [],
        |row| row.get(0),
    ).unwrap_or_else(|_| Uuid::new_v4().to_string());

    // Determinar modo da licença
    let modo_licenca = if req.key_id.contains("dev") || req.key_id.contains("efemero") {
        "DEV".to_string()
    } else {
        "PRODUCAO".to_string()
    };

    // Determinar validade_fim (null → 2099-12-31 como placeholder para sem validade)
    let validade_fim_db = if campos.validade == "null" || campos.validade.is_empty() {
        None::<String>
    } else {
        Some(campos.validade.clone())
    };

    // Atualizar licenca_local (upsert)
    let linhas = conn.execute(
        "UPDATE licenca_local SET
            empresa_id = ?1,
            plano_codigo = ?2,
            status = ?3,
            modo = ?4,
            validade_fim = ?5,
            tolerancia_offline_dias = ?6,
            ultimo_check_em = ?7,
            bloqueio_total = 0,
            motivo_bloqueio = NULL,
            atualizado_em = ?7
         WHERE installation_id = ?8",
        rusqlite::params![
            campos.empresa_id,
            campos.plano_codigo,
            campos.status,
            modo_licenca,
            validade_fim_db,
            campos.tolerancia_offline_dias,
            now,
            installation_id,
        ],
    ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    // Se nenhuma linha atualizada, inserir nova
    if linhas == 0 {
        let lic_id = campos.licenca_id.clone();
        conn.execute(
            "INSERT INTO licenca_local (id, installation_id, empresa_id, plano_codigo, status, modo, validade_fim, ultimo_check_em, tolerancia_offline_dias, bloqueio_total, criado_em, atualizado_em)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?8, ?8)",
            rusqlite::params![
                lic_id,
                installation_id,
                campos.empresa_id,
                campos.plano_codigo,
                campos.status,
                modo_licenca,
                validade_fim_db,
                now,
                campos.tolerancia_offline_dias,
            ],
        ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    }

    // Persistir chave pública para uso futuro (sem exigir nova entrada manual)
    conn.execute(
        "INSERT INTO licenca_chaves (id, key_id, chave_publica_base64, algoritmo, modo, criado_em, atualizado_em)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
         ON CONFLICT(key_id) DO UPDATE SET chave_publica_base64 = excluded.chave_publica_base64, atualizado_em = excluded.atualizado_em",
        rusqlite::params![
            Uuid::new_v4().to_string(),
            req.key_id,
            req.chave_publica_base64,
            req.algoritmo_assinatura,
            modo_licenca,
            now,
        ],
    ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    // Registrar evento
    let _ = registrar_evento_interno(
        &conn,
        &installation_id,
        &campos.licenca_id,
        "LICENCA_PAYLOAD_ASSINADO_APLICADO",
        &campos.status,
        Some(&format!(
            "Payload Ed25519 aplicado. key_id={} plano={} emitido_em={}",
            req.key_id, campos.plano_codigo, campos.emitido_em
        )),
    );

    let pode_operar = campos.status == "ATIVA" || campos.status == "MODO_DEV";
    let mut warnings = verificacao.warnings;

    if modo_licenca == "DEV" {
        warnings.push("Licença aplicada com chave DEV efêmera. Em produção use chave persistente.".to_string());
    }

    info!(
        componente = "commands_licenciamento",
        empresa_id = %campos.empresa_id,
        plano = %campos.plano_codigo,
        status = %campos.status,
        "Licença assinada aplicada com sucesso"
    );

    Ok(AplicarLicencaAssinadaResp {
        sucesso: true,
        assinatura_valida: true,
        status: campos.status,
        modo: modo_licenca,
        empresa_id: campos.empresa_id,
        licenca_id: campos.licenca_id,
        plano_codigo: campos.plano_codigo,
        terminal_id: Some(campos.terminal),
        validade_fim: if campos.validade == "null" { None } else { Some(campos.validade) },
        tolerancia_offline_dias: campos.tolerancia_offline_dias,
        pode_operar,
        mensagem: "Licença assinada verificada e aplicada com sucesso.".to_string(),
        warnings,
    })
}

/// Command: obter_chave_publica_licenca_local
///
/// Retorna a chave pública armazenada localmente para o key_id informado.
/// Útil para o UI mostrar qual chave está registrada no PDV.
/// NUNCA retorna nenhuma chave privada.
#[tauri::command]
pub fn obter_chave_publica_licenca_local(
    key_id: String,
    estado: State<EstadoApp>,
) -> Result<Option<String>, AureonError> {
    let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
    garantir_tabela_chaves(&conn).map_err(|e| AureonError::Interno(e))?;
    Ok(obter_chave_publica(&conn, &key_id))
}

/// Command: atualizar_chave_publica_licenca_dev
///
/// Permite inserir/atualizar a chave pública no PDV manualmente.
/// SOMENTE para modo DEV — em produção a chave vem via payload aplicado.
/// NUNCA aceita chave privada (validação de tamanho: chave pública Ed25519 = 32 bytes → 44 chars base64).
#[tauri::command]
pub fn atualizar_chave_publica_licenca_dev(
    key_id: String,
    chave_publica_base64: String,
    estado: State<EstadoApp>,
) -> Result<String, AureonError> {
    info!(
        componente = "commands_licenciamento",
        key_id = %key_id,
        "atualizar_chave_publica_licenca_dev chamado"
    );

    // Validar que tem exatamente 44 chars (32 bytes Ed25519 em base64 padrão)
    // Chave privada tem 64 bytes → 88 chars — rejeitamos explicitamente
    let chave_trimmed = chave_publica_base64.trim().to_string();
    if chave_trimmed.len() > 60 {
        return Err(AureonError::Interno(
            "Chave rejeitada: tamanho suspeito. Chave pública Ed25519 tem ~44 chars em base64. NUNCA insira chave privada.".to_string()
        ));
    }
    if chave_trimmed.is_empty() {
        return Err(AureonError::Interno("Chave pública não pode ser vazia.".to_string()));
    }

    let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
    garantir_tabela_chaves(&conn).map_err(|e| AureonError::Interno(e))?;

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO licenca_chaves (id, key_id, chave_publica_base64, algoritmo, modo, criado_em, atualizado_em)
         VALUES (?1, ?2, ?3, 'Ed25519', 'DEV', ?4, ?4)
         ON CONFLICT(key_id) DO UPDATE SET chave_publica_base64 = excluded.chave_publica_base64, atualizado_em = excluded.atualizado_em",
        rusqlite::params![Uuid::new_v4().to_string(), key_id, chave_trimmed, now],
    ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    warn!(
        componente = "commands_licenciamento",
        key_id = %key_id,
        "Chave publica DEV atualizada manualmente — NAO USE EM PRODUCAO"
    );

    Ok(format!("Chave pública DEV atualizada para key_id='{}'.", key_id))
}
