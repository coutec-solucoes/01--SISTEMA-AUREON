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
    SincronizarLicencaReq, SincronizarLicencaResp,
    ConfigLicenciamentoReq, ConfigLicenciamentoResp,
    LicencaPoliticaResp, VerificarOperacaoLicencaReq, VerificarOperacaoLicencaResp,
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

// ================================================================
// BLOCO 6: COMMANDS DE SINCRONIZACAO ONLINE (Retaguarda -> PDV)
// ================================================================

/// Garante que a tabela licenca_config existe para armazenar a URL da Retaguarda.
fn garantir_tabela_config(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS licenca_config (
            id          TEXT PRIMARY KEY,
            url_retaguarda TEXT NOT NULL,
            key_id      TEXT,
            chave_publica_base64 TEXT,
            criado_em   TEXT NOT NULL,
            atualizado_em TEXT NOT NULL
        );"
    ).map_err(|e| format!("Erro ao criar tabela licenca_config: {}", e))
}

#[tauri::command]
pub fn configurar_licenciamento_online(
    req: ConfigLicenciamentoReq,
    estado: State<EstadoApp>,
) -> Result<ConfigLicenciamentoResp, AureonError> {
    info!(
        componente = "commands_licenciamento",
        url = %req.url_retaguarda,
        "configurar_licenciamento_online chamado"
    );

    let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
    garantir_tabela_config(&conn).map_err(|e| AureonError::Interno(e))?;

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    // Sempre usa a mesma linha ID fixo 'config_unica' para garantir registro único
    conn.execute(
        "INSERT INTO licenca_config (id, url_retaguarda, key_id, chave_publica_base64, criado_em, atualizado_em)
         VALUES ('config_unica', ?1, ?2, ?3, ?4, ?4)
         ON CONFLICT(id) DO UPDATE SET 
            url_retaguarda = excluded.url_retaguarda,
            key_id = excluded.key_id,
            chave_publica_base64 = excluded.chave_publica_base64,
            atualizado_em = excluded.atualizado_em",
        rusqlite::params![
            req.url_retaguarda.trim(),
            req.key_id,
            req.chave_publica_base64,
            now
        ],
    ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    Ok(ConfigLicenciamentoResp {
        sucesso: true,
        url_retaguarda: req.url_retaguarda,
        key_id: req.key_id.unwrap_or_default(),
        mensagem: "Configuração de licenciamento salva com sucesso.".to_string(),
        warnings: vec![],
    })
}

#[tauri::command]
pub fn obter_config_licenciamento_online(
    estado: State<EstadoApp>,
) -> Result<ConfigLicenciamentoResp, AureonError> {
    let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
    garantir_tabela_config(&conn).map_err(|e| AureonError::Interno(e))?;

    let row = conn.query_row(
        "SELECT url_retaguarda, key_id, chave_publica_base64 FROM licenca_config WHERE id = 'config_unica'",
        [],
        |r| Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, Option<String>>(2)?,
        ))
    ).optional().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    match row {
        Some((url, kid, _cpub)) => {
            Ok(ConfigLicenciamentoResp {
                sucesso: true,
                url_retaguarda: url,
                key_id: kid.unwrap_or_default(),
                mensagem: "Configuração carregada com sucesso.".to_string(),
                warnings: vec![],
            })
        }
        None => {
            // Default dev url
            Ok(ConfigLicenciamentoResp {
                sucesso: false,
                url_retaguarda: "http://localhost:5053".to_string(),
                key_id: "".to_string(),
                mensagem: "Nenhuma configuração encontrada localmente. Usando padrões.".to_string(),
                warnings: vec![],
            })
        }
    }
}

#[tauri::command]
pub async fn sincronizar_licenca_online(
    req: SincronizarLicencaReq,
    estado: State<'_, EstadoApp>,
) -> Result<SincronizarLicencaResp, AureonError> {
    info!(
        componente = "commands_licenciamento",
        url = %req.url_retaguarda,
        "sincronizar_licenca_online iniciado"
    );

    // 1. Obter identidade local da instalação
    let (installation_id, emp_id_db, term_id, term_nome) = {
        let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
        let res = conn.query_row(
            "SELECT installation_id, empresa_id, terminal_id, terminal_nome FROM instalacao_local LIMIT 1",
            [],
            |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
            ))
        ).optional().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

        match res {
            Some(data) => data,
            None => {
                return Ok(SincronizarLicencaResp {
                    sucesso: false,
                    online: false,
                    checkin_realizado: false,
                    assinatura_valida: false,
                    aplicado_localmente: false,
                    status: "PENDENTE_ATIVACAO".to_string(),
                    modo: "".to_string(),
                    empresa_id: "".to_string(),
                    licenca_id: "".to_string(),
                    plano_codigo: "".to_string(),
                    terminal_id: None,
                    validade_fim: None,
                    pode_operar: false,
                    mensagem: "Erro: Instalação local não identificada. Ative localmente primeiro.".to_string(),
                    warnings: vec![],
                });
            }
        }
    };

    let empresa_id = req.empresa_id.unwrap_or(emp_id_db.unwrap_or_default());
    if empresa_id.is_empty() {
        return Ok(SincronizarLicencaResp {
            sucesso: false,
            online: false,
            checkin_realizado: false,
            assinatura_valida: false,
            aplicado_localmente: false,
            status: "PENDENTE_ATIVACAO".to_string(),
            modo: "".to_string(),
            empresa_id: "".to_string(),
            licenca_id: "".to_string(),
            plano_codigo: "".to_string(),
            terminal_id: None,
            validade_fim: None,
            pode_operar: false,
            mensagem: "Erro: Empresa ID é obrigatório para check-in.".to_string(),
            warnings: vec![],
        });
    }

    // 2. Chamar o endpoint da Retaguarda: POST /licenciamento/check-in
    let checkin_url = format!("{}/licenciamento/check-in", req.url_retaguarda.trim_end_matches('/'));
    
    // Obter hash/detalhes fictícios ou seguros do dispositivo
    let dispositivo_hash = format!("DISP-{}", &installation_id[..8]);
    let app_versao = "0.0.1".to_string();
    let os_name = std::env::consts::OS.to_string();

    let checkin_payload = serde_json::json!({
        "installation_id": installation_id,
        "empresa_id": empresa_id,
        "terminal_id": term_id,
        "terminal_nome": term_nome,
        "dispositivo_hash": dispositivo_hash,
        "app_versao": app_versao,
        "sistema_operacional": os_name
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| AureonError::Interno(format!("Erro ao criar cliente HTTP: {}", e)))?;

    info!(url = %checkin_url, "Enviando POST de check-in para retaguarda");
    let response_res = client.post(&checkin_url)
        .json(&checkin_payload)
        .send()
        .await;

    let response = match response_res {
        Ok(r) => r,
        Err(e) => {
            // Regra Offline-First: Tratar falha de rede sem apagar licença local
            warn!(erro = ?e, "Falha de comunicação com a Retaguarda. Mantendo status local.");
            
            // Registrar evento local de falha
            let conn_lock = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
            let _ = registrar_evento_interno(
                &conn_lock,
                &installation_id,
                "",
                "LICENCA_SYNC_FALHOU",
                "FALHA_CONEXAO",
                Some(&format!("Falha na sincronização online: {}", e)),
            );

            return Ok(SincronizarLicencaResp {
                sucesso: false,
                online: false,
                checkin_realizado: false,
                assinatura_valida: false,
                aplicado_localmente: false,
                status: "OFFLINE".to_string(),
                modo: "".to_string(),
                empresa_id,
                licenca_id: "".to_string(),
                plano_codigo: "".to_string(),
                terminal_id: term_id,
                validade_fim: None,
                pode_operar: true, // assume que mantém a capacidade de operar
                mensagem: format!("Falha de conexão com a retaguarda: {}. O PDV continua funcionando offline.", e),
                warnings: vec!["Operando em modo de contingência offline.".to_string()],
            });
        }
    };

    if !response.status().is_success() {
        let status_code = response.status();
        let err_body = response.text().await.unwrap_or_default();
        return Ok(SincronizarLicencaResp {
            sucesso: false,
            online: true,
            checkin_realizado: false,
            assinatura_valida: false,
            aplicado_localmente: false,
            status: "ERRO_HTTP".to_string(),
            modo: "".to_string(),
            empresa_id,
            licenca_id: "".to_string(),
            plano_codigo: "".to_string(),
            terminal_id: term_id,
            validade_fim: None,
            pode_operar: true,
            mensagem: format!("Retaguarda respondeu com status {}: {}", status_code, err_body),
            warnings: vec![],
        });
    }


    // Receber os dados do check-in
    let checkin_resp: serde_json::Value = response.json().await
        .map_err(|e| AureonError::Interno(format!("Erro ao ler JSON da resposta: {}", e)))?;

    // Verificar se a resposta veio com payload assinado
    let payload_licenca_json_val = checkin_resp.get("payload_licenca_json");
    let assinatura_licenca_val = checkin_resp.get("assinatura_licenca");

    if payload_licenca_json_val.is_none() || assinatura_licenca_val.is_none() {
        return Ok(SincronizarLicencaResp {
            sucesso: false,
            online: true,
            checkin_realizado: true,
            assinatura_valida: false,
            aplicado_localmente: false,
            status: "SEM_ASSINATURA".to_string(),
            modo: "".to_string(),
            empresa_id,
            licenca_id: "".to_string(),
            plano_codigo: "".to_string(),
            terminal_id: term_id,
            validade_fim: None,
            pode_operar: true,
            mensagem: "Check-in realizado, mas a Retaguarda não forneceu um payload assinado.".to_string(),
            warnings: vec![],
        });
    }

    let payload_str = match payload_licenca_json_val.unwrap().as_str() {
        Some(s) => s.to_string(),
        None => payload_licenca_json_val.unwrap().to_string(),
    };
    let assinatura_hex = assinatura_licenca_val.unwrap().as_str().unwrap_or_default().to_string();

    // Buscar chave pública registrada localmente para validar
    let cpub_hex = {
        let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
        garantir_tabela_chaves(&conn).map_err(|e| AureonError::Interno(e))?;
        // Tenta obter alguma chave pública cadastrada na tabela
        conn.query_row(
            "SELECT chave_publica_base64 FROM licenca_chaves LIMIT 1",
            [],
            |r| r.get::<_, String>(0)
        ).optional().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?
    };

    let cpub_hex = match cpub_hex {
        Some(k) => k,
        None => {
            // Permite fallback dev ou avisa que precisa de chave pública local
            return Ok(SincronizarLicencaResp {
                sucesso: false,
                online: true,
                checkin_realizado: true,
                assinatura_valida: false,
                aplicado_localmente: false,
                status: "SEM_CHAVE_PUBLICA".to_string(),
                modo: "".to_string(),
                empresa_id,
                licenca_id: "".to_string(),
                plano_codigo: "".to_string(),
                terminal_id: term_id,
                validade_fim: None,
                pode_operar: true,
                mensagem: "Retaguarda respondeu com licença, mas o PDV não possui nenhuma Chave Pública registrada localmente para validar.".to_string(),
                warnings: vec!["Cadastre a Chave Pública nas configurações do PDV.".to_string()],
            });
        }
    };

    // Validar assinatura Ed25519 localmente
    let verificacao = verificar_assinatura_local(
        &payload_str,
        ALGORITMO_SUPORTADO,
        "", // key_id opcional
        &assinatura_hex,
        None, // payload_hash_informado opcional
        &cpub_hex,
    );

    if !verificacao.valido {
        return Ok(SincronizarLicencaResp {
            sucesso: false,
            online: true,
            checkin_realizado: true,
            assinatura_valida: false,
            aplicado_localmente: false,
            status: "ASSINATURA_INVALIDA".to_string(),
            modo: "".to_string(),
            empresa_id,
            licenca_id: "".to_string(),
            plano_codigo: "".to_string(),
            terminal_id: term_id,
            validade_fim: None,
            pode_operar: true,
            mensagem: format!("Assinatura do payload recebido da Retaguarda é INVÁLIDA: {}", verificacao.mensagem),
            warnings: vec![],
        });
    }


    // Extrair os campos
    let campos = match extrair_campos_payload(&payload_str) {
        Ok(c) => c,
        Err(e) => {
            return Ok(SincronizarLicencaResp {
                sucesso: false,
                online: true,
                checkin_realizado: true,
                assinatura_valida: true,
                aplicado_localmente: false,
                status: "PAYLOAD_CORROMPIDO".to_string(),
                modo: "".to_string(),
                empresa_id,
                licenca_id: "".to_string(),
                plano_codigo: "".to_string(),
                terminal_id: term_id,
                validade_fim: None,
                pode_operar: true,
                mensagem: format!("Erro ao decodificar campos da licença recebida: {}", e),
                warnings: vec![],
            });
        }
    };

    // Aplicar no SQLite
    {
        let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let validade_fim_db = if campos.validade == "null" { None } else { Some(campos.validade.clone()) };

        conn.execute("DELETE FROM licenca_local", []).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
        conn.execute(
            "INSERT INTO licenca_local (id, installation_id, empresa_id, plano_codigo, status, modo, validade_fim, ultimo_check_em, tolerancia_offline_dias, bloqueio_total, criado_em, atualizado_em)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?8, ?8)",
            rusqlite::params![
                campos.licenca_id,
                installation_id,
                campos.empresa_id,
                campos.plano_codigo,
                campos.status,
                "PRODUCAO", // Assume produção para licenças online validadas
                validade_fim_db,
                now,
                campos.tolerancia_offline_dias,
            ],
        ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

        // Registrar evento de sucesso
        let _ = registrar_evento_interno(
            &conn,
            &installation_id,
            &campos.licenca_id,
            "LICENCA_SYNC_ONLINE_SUCESSO",
            &campos.status,
            Some("Check-in online realizado e licença assinada aplicada."),
        );
    }

    Ok(SincronizarLicencaResp {
        sucesso: true,
        online: true,
        checkin_realizado: true,
        assinatura_valida: true,
        aplicado_localmente: true,
        status: campos.status.clone(),
        modo: "PRODUCAO".to_string(),
        empresa_id: campos.empresa_id,
        licenca_id: campos.licenca_id,
        plano_codigo: campos.plano_codigo,
        terminal_id: Some(campos.terminal),
        validade_fim: if campos.validade == "null" { None } else { Some(campos.validade) },
        pode_operar: campos.status == "ATIVA" || campos.status == "MODO_DEV",
        mensagem: "Licença sincronizada online e atualizada localmente com sucesso!".to_string(),
        warnings: vec![],
    })
}

// ================================================================
// BLOCO 7: COMMANDS DE POLITICA OPERACIONAL DE LICENCA (OFFLINE-FIRST)
// ================================================================

#[tauri::command]
pub fn obter_politica_licenca(estado: State<EstadoApp>) -> Result<LicencaPoliticaResp, AureonError> {
    info!(componente = "commands_licenciamento", "Calculando politica operacional de licenca local");
    let conn = estado.conn_sqlite.lock().map_err(|_| AureonError::Interno("LockError".to_string()))?;

    // 1. Busca licença local ativa
    let lic_opt = conn.query_row(
        "SELECT id, status, modo, validade_fim, ultimo_check_em, tolerancia_offline_dias, bloqueio_total, motivo_bloqueio
         FROM licenca_local LIMIT 1",
        [],
        |r| Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, i64>(5)?,
            r.get::<_, i32>(6)?,
            r.get::<_, Option<String>>(7)?,
        ))
    ).optional().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    let now_dt = chrono::Utc::now();
    let mut warnings = vec![];
    let mut acoes = vec![];

    let (lic_id, status, modo, validade_fim, ultimo_check_em, tolerancia_dias, bloqueio_total, motivo_bloqueio) = match lic_opt {
        Some(data) => data,
        None => {
            // SEM LICENÇA LOCAL
            return Ok(LicencaPoliticaResp {
                nivel: "SEM_LICENCA".to_string(),
                pode_operar: true, // Decisão conservadora (true com aviso para não travar implantação)
                deve_exibir_alerta: true,
                deve_sincronizar: true,
                dias_restantes: None,
                dias_desde_ultimo_check: 0,
                tolerancia_offline_dias: 10,
                status: "PENDENTE_ATIVACAO".to_string(),
                modo: "".to_string(),
                bloqueio_total: 0,
                motivo_bloqueio: None,
                mensagem: "Terminal sem licença ativa. Por favor, ative online ou via payload offline.".to_string(),
                acoes_recomendadas: vec!["Sincronizar online com a Retaguarda".to_string(), "Inserir payload assinado manualmente".to_string()],
                warnings: vec!["Nenhuma licença local encontrada no SQLite.".to_string()],
            });
        }
    };

    // Calcular dias desde último check-in
    let dias_desde_ultimo_check = match &ultimo_check_em {
        Some(check_str) => {
            if let Ok(check_dt) = chrono::NaiveDateTime::parse_from_str(check_str, "%Y-%m-%d %H:%M:%S") {
                let check_dt_utc = chrono::DateTime::<chrono::Utc>::from_utc(check_dt, chrono::Utc);
                now_dt.signed_duration_since(check_dt_utc).num_days()
            } else {
                0
            }
        }
        None => 0,
    };

    // Calcular dias restantes da validade
    let dias_restantes = match &validade_fim {
        Some(val_str) => {
            if let Ok(val_dt) = chrono::NaiveDateTime::parse_from_str(val_str, "%Y-%m-%d %H:%M:%S") {
                let val_dt_utc = chrono::DateTime::<chrono::Utc>::from_utc(val_dt, chrono::Utc);
                Some(val_dt_utc.signed_duration_since(now_dt).num_days())
            } else {
                None
            }
        }
        None => None,
    };


    let mut nivel = "OK".to_string();
    let mut pode_operar = true;
    let mut deve_exibir_alerta = false;
    let mut deve_sincronizar = dias_desde_ultimo_check >= tolerancia_dias;
    let mut mensagem = "Licença regular e terminal autorizado.".to_string();

    // Regras de cálculo da política
    if bloqueio_total == 1 || status == "BLOQUEADA" {
        nivel = "BLOQUEADA".to_string();
        pode_operar = false;
        deve_exibir_alerta = true;
        mensagem = format!("Licença bloqueada pela retaguarda. Motivo: {}", motivo_bloqueio.clone().unwrap_or_else(|| "Bloqueio administrativo".to_string()));
        acoes.push("Contatar a administração central para regularização".to_string());
        
        let _ = registrar_evento_interno(&conn, "", &lic_id, "LICENCA_BLOQUEADA", &status, motivo_bloqueio.as_deref());
    } else if modo == "DEV" {
        nivel = "MODO_DEV".to_string();
        pode_operar = true;
        deve_exibir_alerta = true;
        mensagem = "Terminal operando em modo de Desenvolvimento/Demonstração.".to_string();
        acoes.push("Utilizar apenas para testes. Não utilizar em produção.".to_string());
        warnings.push("Licença de demonstração não comercial ativa.".to_string());
    } else {
        // Validação de datas
        if let Some(restantes) = dias_restantes {
            if restantes < 0 {
                // Licença expirada
                if dias_desde_ultimo_check < tolerancia_dias {
                    // Expirada mas dentro da tolerância offline
                    nivel = "TOLERANCIA_OFFLINE".to_string();
                    pode_operar = true;
                    deve_exibir_alerta = true;
                    mensagem = format!("Sua licença expirou há {} dias, mas a operação offline temporária está ativa (tolerância: {} dias).", restantes.abs(), tolerancia_dias);
                    acoes.push("Conectar à internet e sincronizar com a Retaguarda imediatamente".to_string());
                    
                    let _ = registrar_evento_interno(&conn, "", &lic_id, "LICENCA_TOLERANCIA_OFFLINE", &status, Some("Tolerância offline ativa"));
                } else {
                    // Completamente expirada
                    nivel = "EXPIRADA".to_string();
                    pode_operar = false;
                    deve_exibir_alerta = true;
                    mensagem = "Licença expirada e fora do período de tolerância offline.".to_string();
                    acoes.push("Sincronizar licença online ou aplicar novo payload".to_string());
                    
                    let _ = registrar_evento_interno(&conn, "", &lic_id, "LICENCA_EXPIRADA_DETECTADA", &status, Some("Licença completamente expirada"));
                }
            } else if restantes <= 7 {
                // Alerta de vencimento próximo
                nivel = "ALERTA_VENCIMENTO".to_string();
                pode_operar = true;
                deve_exibir_alerta = true;
                mensagem = format!("Sua licença vencerá em {} dias. Programe a renovação.", restantes);
                acoes.push("Providenciar a renovação da licença na central".to_string());
            }
        }
    }

    if deve_sincronizar {
        warnings.push(format!("Terminal está sem sincronizar há {} dias. Recomenda-se realizar check-in.", dias_desde_ultimo_check));
    }

    Ok(LicencaPoliticaResp {
        nivel,
        pode_operar,
        deve_exibir_alerta,
        deve_sincronizar,
        dias_restantes,
        dias_desde_ultimo_check,
        tolerancia_offline_dias: tolerancia_dias,
        status,
        modo,
        bloqueio_total,
        motivo_bloqueio,
        mensagem,
        acoes_recomendadas: acoes,
        warnings,
    })
}

// ================================================================
// BLOCO 8: GUARDA OPERACIONAL DE LICENCA (BLOQUEIO)
// ================================================================

pub fn avaliar_operacao_licenca(
    conn: &rusqlite::Connection,
    operacao: &str,
    _contexto_id: Option<&str>,
) -> Result<VerificarOperacaoLicencaResp, String> {
    let lic_opt = conn.query_row(
        "SELECT id, status, modo, validade_fim, ultimo_check_em, tolerancia_offline_dias, bloqueio_total, motivo_bloqueio
         FROM licenca_local LIMIT 1",
        [],
        |r| Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, i64>(5)?,
            r.get::<_, i32>(6)?,
            r.get::<_, Option<String>>(7)?,
        ))
    ).optional().map_err(|e| e.to_string())?;

    let now_dt = chrono::Utc::now();

    let (_, status, modo, validade_fim, ultimo_check_em, tolerancia_dias, bloqueio_total, motivo_bloqueio) = match lic_opt {
        Some(data) => data,
        None => {
            return Ok(VerificarOperacaoLicencaResp {
                permitido: true,
                nivel: "SEM_LICENCA".to_string(),
                status: "PENDENTE_ATIVACAO".to_string(),
                modo: "".to_string(),
                operacao: operacao.to_string(),
                mensagem: "Operação permitida (sem licença), mas regularização pendente.".to_string(),
                motivo_bloqueio: None,
                acoes_recomendadas: vec!["Sincronizar online com a Retaguarda".to_string()],
                warnings: vec!["Terminal sem licença operando em período de graça provisório.".to_string()],
            });
        }
    };

    let dias_desde_ultimo_check = match &ultimo_check_em {
        Some(check_str) => {
            if let Ok(check_dt) = chrono::NaiveDateTime::parse_from_str(check_str, "%Y-%m-%d %H:%M:%S") {
                let check_dt_utc = chrono::DateTime::<chrono::Utc>::from_utc(check_dt, chrono::Utc);
                now_dt.signed_duration_since(check_dt_utc).num_days()
            } else {
                0
            }
        }
        None => 0,
    };

    let dias_restantes = match &validade_fim {
        Some(val_str) => {
            if let Ok(val_dt) = chrono::NaiveDateTime::parse_from_str(val_str, "%Y-%m-%d %H:%M:%S") {
                let val_dt_utc = chrono::DateTime::<chrono::Utc>::from_utc(val_dt, chrono::Utc);
                Some(val_dt_utc.signed_duration_since(now_dt).num_days())
            } else {
                None
            }
        }
        None => None,
    };

    let mut nivel = "OK".to_string();
    let mut permitido = true;
    let mut mensagem = "Operação permitida.".to_string();
    let mut warnings = vec![];

    if bloqueio_total == 1 || status == "BLOQUEADA" {
        nivel = "BLOQUEADA".to_string();
        permitido = false;
        mensagem = "Operação bloqueada pela política de licença local. Acesse Sistema > Licença para sincronizar ou regularizar.".to_string();
    } else if modo == "DEV" {
        nivel = "MODO_DEV".to_string();
        warnings.push("Operação permitida em modo DEV.".to_string());
    } else {
        if let Some(restantes) = dias_restantes {
            if restantes < 0 {
                if dias_desde_ultimo_check < tolerancia_dias {
                    nivel = "TOLERANCIA_OFFLINE".to_string();
                    warnings.push("Operação permitida em tolerância offline.".to_string());
                } else {
                    nivel = "EXPIRADA".to_string();
                    permitido = false;
                    mensagem = "Operação bloqueada pela política de licença local. Acesse Sistema > Licença para sincronizar ou regularizar.".to_string();
                }
            } else if restantes <= 7 {
                nivel = "ALERTA_VENCIMENTO".to_string();
                warnings.push("Licença próxima do vencimento.".to_string());
            }
        }
    }

    Ok(VerificarOperacaoLicencaResp {
        permitido,
        nivel,
        status,
        modo,
        operacao: operacao.to_string(),
        mensagem,
        motivo_bloqueio: if !permitido { motivo_bloqueio } else { None },
        acoes_recomendadas: vec![],
        warnings,
    })
}

pub fn garantir_operacao_licenciada(
    conn: &rusqlite::Connection,
    operacao: &str,
    contexto_id: Option<&str>,
    origem: Option<&str>,
) -> Result<(), String> {
    let result = avaliar_operacao_licenca(conn, operacao, contexto_id)?;

    // Buscar licenca_id
    let lic_id: String = conn.query_row(
        "SELECT id FROM licenca_local LIMIT 1",
        [],
        |r| r.get(0)
    ).unwrap_or_default();

    let evt_tipo = if result.permitido { "LICENCA_OPERACAO_PERMITIDA" } else { "LICENCA_OPERACAO_BLOQUEADA" };
    let payload = serde_json::json!({
        "operacao": operacao,
        "nivel": &result.nivel,
        "status": &result.status,
        "contexto_id": contexto_id,
        "origem": origem,
        "motivo": &result.motivo_bloqueio
    });

    let _ = conn.execute(
        "INSERT INTO licenca_eventos (id, installation_id, licenca_id, tipo_evento, status_novo, mensagem, criado_em)
         VALUES (?1, '', ?2, ?3, ?4, ?5, datetime('now'))",
        rusqlite::params![
            Uuid::new_v4().to_string(),
            &lic_id,
            evt_tipo,
            &result.nivel,
            &payload.to_string(),
        ],
    );

    if result.permitido {
        if result.nivel != "OK" && result.nivel != "SEM_LICENCA" {
            let _ = conn.execute(
                "INSERT INTO licenca_eventos (id, installation_id, licenca_id, tipo_evento, status_novo, mensagem, criado_em)
                 VALUES (?1, '', ?2, 'LICENCA_BLOQUEIO_SUAVE_APLICADO', ?3, ?4, datetime('now'))",
                rusqlite::params![
                    Uuid::new_v4().to_string(),
                    &lic_id,
                    &result.nivel,
                    &format!("Operação permitida com alerta. Operação: {}", operacao),
                ],
            );
        }
        Ok(())
    } else {
        Err(result.mensagem)
    }
}

#[tauri::command]
pub async fn verificar_operacao_permitida_licenca(
    req: VerificarOperacaoLicencaReq,
    estado: State<'_, EstadoApp>,
) -> Result<aureon_core::RespostaBase<VerificarOperacaoLicencaResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|_| "LockError".to_string())?;
    let result = avaliar_operacao_licenca(&conn, &req.operacao, req.contexto_id.as_deref())?;
    Ok(aureon_core::RespostaBase::ok("Consulta finalizada", result))
}
