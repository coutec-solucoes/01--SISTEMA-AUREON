use tauri::State;
use chrono::Utc;
use tracing::{info, error};

use aureon_core::{dtos::*, RespostaBase};
use aureon_infra::sqlite::repositories::{
    ConfiguracaoSqliteRepository, LogSqliteRepository,
};
use aureon_domain::{
    repositories::{ConfiguracaoRepository, LogRepository},
    services::{ConfiguracaoService, LogService},
};
use aureon_shared::crypto::{criptografar, chave_de_base64};
use crate::estado::EstadoApp;

// ================================================================
// COMMAND: obter_status_local
// ================================================================

#[tauri::command]
pub async fn obter_status_local(
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<StatusLocalDto>, String> {
    info!(componente = "aureon-pdv::commands", "Chamada: obter_status_local");

    let conn = estado.conn_sqlite.clone();

    let sqlite_status = {
        let c = conn.lock().map_err(|e| e.to_string())?;
        match c.execute_batch("SELECT 1") {
            Ok(_)  => "ok".to_string(),
            Err(e) => format!("erro: {e}"),
        }
    };

    // Tenta obter terminal_id do banco
    let terminal_id = {
        let c = conn.lock().map_err(|e| e.to_string())?;
        c.query_row(
            "SELECT terminal_id FROM terminais WHERE ativo = 1 LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "nao-registrado".to_string())
    };

    Ok(RespostaBase::ok(
        "Status local obtido com sucesso.",
        StatusLocalDto {
            app_versao:    "0.0.1".to_string(),
            sqlite_status,
            terminal_id,
            horario:       Utc::now(),
        },
    ))
}

// ================================================================
// COMMAND: testar_sqlite
// ================================================================

#[tauri::command]
pub async fn testar_sqlite(
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<TesteConexaoDto>, String> {
    info!(componente = "aureon-pdv::commands", "Chamada: testar_sqlite");

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    match conn.execute_batch("SELECT 1") {
        Ok(_) => {
            info!(componente = "aureon-pdv::commands", "SQLite: healthcheck OK");
            Ok(RespostaBase::ok(
                "SQLite operacional.",
                TesteConexaoDto { sqlite_ok: true, mensagem: "Conexão SQLite estabelecida com sucesso.".to_string() },
            ))
        }
        Err(e) => {
            error!(componente = "aureon-pdv::commands", erro = %e, "SQLite: falha no healthcheck");
            Ok(RespostaBase::ok(
                "SQLite com erro.",
                TesteConexaoDto { sqlite_ok: false, mensagem: format!("Falha: {e}") },
            ))
        }
    }
}

// ================================================================
// COMMAND: gravar_log_local
// ================================================================

#[tauri::command]
pub async fn gravar_log_local(
    dto:    GravarLogDto,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<()>, String> {
    info!(
        componente = "aureon-pdv::commands",
        nivel      = %dto.nivel,
        "Chamada: gravar_log_local"
    );

    let conn     = estado.conn_sqlite.clone();
    let repo     = LogSqliteRepository::novo(conn);
    let service  = LogService::novo(std::sync::Arc::new(repo));

    match service.gravar(&dto.nivel, &dto.componente, &dto.mensagem) {
        Ok(_)  => Ok(RespostaBase::ok("Log gravado com sucesso.", ())),
        Err(e) => Ok(RespostaBase::falha("Falha ao gravar log.", &e)),
    }
}

// ================================================================
// COMMAND: obter_configuracao_local
// ================================================================

#[tauri::command]
pub async fn obter_configuracao_local(
    chave:  String,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<Option<String>>, String> {
    info!(
        componente = "aureon-pdv::commands",
        chave      = %chave,
        "Chamada: obter_configuracao_local"
    );

    if chave.trim().is_empty() {
        return Ok(RespostaBase::falha_manual(
            "Chave inválida.",
            "ERRO_VALIDACAO",
            "A chave não pode ser vazia.",
        ));
    }

    let conn    = estado.conn_sqlite.clone();
    let repo    = ConfiguracaoSqliteRepository::novo(conn);
    let service = ConfiguracaoService::novo(std::sync::Arc::new(repo));

    // Retorna o valor criptografado — NUNCA o puro
    match service.obter_criptografado(&chave) {
        Ok(valor) => Ok(RespostaBase::ok("Configuração obtida.", valor)),
        Err(e)    => Ok(RespostaBase::falha("Falha ao obter configuração.", &e)),
    }
}
