use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use crate::{app::AppState, erros::ErroApi, middleware::UsuarioLogado};
use aureon_core::RespostaBase;

#[derive(Deserialize)]
pub struct RegistroTerminalReq {
    pub codigo_terminal: String,
    pub nome_terminal: String,
    pub identificador_maquina: String,
}

pub async fn registrar_terminal(
    State(state): State<AppState>,
    Json(payload): Json<RegistroTerminalReq>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    let existente = sqlx::query(
        "SELECT id, status_sync, chave_terminal FROM terminais_pdv WHERE codigo_terminal = $1 OR identificador_maquina_futuro = $2"
    )
    .bind(&payload.codigo_terminal)
    .bind(&payload.identificador_maquina)
    .fetch_optional(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    if let Some(row) = existente {
        let term_id: Uuid = row.get("id");
        let term_chave: String = row.get("chave_terminal");
        let term_status: String = row.get("status_sync");

        let resp = RespostaBase::ok("Terminal já registrado. Aguardando autorização.", json!({
            "terminal_id": term_id,
            "chave_terminal": term_chave,
            "status": term_status
        }));
        return Ok((StatusCode::OK, Json(resp)));
    }

    let id = Uuid::new_v4();
    let chave_terminal = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO terminais_pdv (id, codigo_terminal, nome_terminal, identificador_maquina_futuro, chave_terminal, status_sync, ativo, autorizado)
         VALUES ($1, $2, $3, $4, $5, 'PENDENTE', TRUE, FALSE)"
    )
    .bind(id)
    .bind(&payload.codigo_terminal)
    .bind(&payload.nome_terminal)
    .bind(&payload.identificador_maquina)
    .bind(&chave_terminal)
    .execute(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    // Registrar log na tabela sync_logs
    sqlx::query(
        "INSERT INTO sync_logs (terminal_id, tipo_evento, status, mensagem, detalhe_json)
         VALUES ($1, 'TERMINAL_REGISTRADO', 'INFO', 'Terminal registrado via API. Aguardando autorização.', '{}')"
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    let resp = RespostaBase::ok("Terminal registrado com sucesso. Aguardando autorização.", json!({
        "terminal_id": id,
        "chave_terminal": chave_terminal,
        "status": "PENDENTE"
    }));

    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn status_terminal(
    State(state): State<AppState>,
    Path(codigo_terminal): Path<String>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    let term = sqlx::query(
        "SELECT id, ativo, autorizado, status_sync, primeiro_sync_concluido FROM terminais_pdv WHERE codigo_terminal = $1"
    )
    .bind(&codigo_terminal)
    .fetch_optional(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    match term {
        Some(row) => {
            let id: Uuid = row.get("id");
            let ativo: bool = row.get("ativo");
            let autorizado: bool = row.get("autorizado");
            let status_sync: String = row.get("status_sync");
            let primeiro_sync_concluido: bool = row.get("primeiro_sync_concluido");

            Ok((StatusCode::OK, Json(RespostaBase::ok("Status obtido", json!({
                "terminal_id": id,
                "ativo": ativo,
                "autorizado": autorizado,
                "status_sync": status_sync,
                "primeiro_sync_concluido": primeiro_sync_concluido
            })))))
        },
        None => Err(ErroApi::bad_request("Terminal não encontrado", "NAO_ENCONTRADO"))
    }
}

pub async fn confirmar_autorizacao(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    let term = sqlx::query(
        "SELECT id, autorizado FROM terminais_pdv WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    match term {
        Some(row) => {
            let autorizado: bool = row.get("autorizado");
            if autorizado {
                Ok((StatusCode::OK, Json(RespostaBase::ok("Terminal está autorizado", json!({"autorizado": true})))))
            } else {
                Ok((StatusCode::OK, Json(RespostaBase::ok("Terminal pendente de autorização", json!({"autorizado": false})))))
            }
        },
        None => Err(ErroApi::bad_request("Terminal não encontrado", "NAO_ENCONTRADO"))
    }
}

pub async fn status_geral_terminais(
    State(state): State<AppState>,
    axum::extract::Extension(_usuario): axum::extract::Extension<UsuarioLogado>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    let terminais = sqlx::query(
        r#"
        SELECT 
            t.id, t.codigo_terminal, t.nome_terminal, t.ativo, t.autorizado, 
            t.status_sync, t.ultima_sincronizacao, t.primeiro_sync_concluido,
            s.versoes_aplicadas, s.versoes_pendentes, s.erro_detalhe
        FROM terminais_pdv t
        LEFT JOIN sync_status_terminais s ON s.terminal_id = t.id
        ORDER BY t.nome_terminal
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    let json_terminais: Vec<_> = terminais.into_iter().map(|row| {
        let id: Uuid = row.get("id");
        let codigo_terminal: String = row.get("codigo_terminal");
        let nome_terminal: String = row.get("nome_terminal");
        let ativo: bool = row.get("ativo");
        let autorizado: bool = row.get("autorizado");
        let status_sync: String = row.get("status_sync");
        let ultima_sincronizacao: Option<chrono::DateTime<chrono::Utc>> = row.try_get("ultima_sincronizacao").unwrap_or(None);
        let primeiro_sync_concluido: bool = row.get("primeiro_sync_concluido");
        let versoes_aplicadas: Option<serde_json::Value> = row.try_get("versoes_aplicadas").unwrap_or(None);
        let versoes_pendentes: Option<serde_json::Value> = row.try_get("versoes_pendentes").unwrap_or(None);
        let erro_detalhe: Option<String> = row.try_get("erro_detalhe").unwrap_or(None);

        json!({
            "id": id,
            "codigo_terminal": codigo_terminal,
            "nome_terminal": nome_terminal,
            "ativo": ativo,
            "autorizado": autorizado,
            "status_sync": status_sync,
            "ultima_sincronizacao": ultima_sincronizacao,
            "primeiro_sync_concluido": primeiro_sync_concluido,
            "versoes_aplicadas": versoes_aplicadas.unwrap_or(json!({})),
            "versoes_pendentes": versoes_pendentes.unwrap_or(json!({})),
            "erro_detalhe": erro_detalhe
        })
    }).collect();

    Ok((StatusCode::OK, Json(RespostaBase::ok("Status dos terminais", json!(json_terminais)))))
}

pub async fn diagnostico_terminal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(_usuario): axum::extract::Extension<UsuarioLogado>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    let term = sqlx::query(
        r#"
        SELECT id, codigo_terminal, nome_terminal, ativo, autorizado, status_sync, ultima_sincronizacao, ultima_versao_recebida 
        FROM terminais_pdv 
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    match term {
        Some(row) => {
            let t_id: Uuid = row.get("id");
            let t_codigo: String = row.get("codigo_terminal");
            let t_nome: String = row.get("nome_terminal");
            let t_ativo: bool = row.get("ativo");
            let t_autorizado: bool = row.get("autorizado");
            let t_status: String = row.get("status_sync");
            let t_ultima_sync: Option<chrono::DateTime<chrono::Utc>> = row.try_get("ultima_sincronizacao").unwrap_or(None);
            let t_ultima_versao: Option<String> = row.try_get("ultima_versao_recebida").unwrap_or(None);

            let logs = sqlx::query(
                "SELECT tipo_evento, status, mensagem, criado_em FROM sync_logs WHERE terminal_id = $1 ORDER BY criado_em DESC LIMIT 10"
            )
            .bind(id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            let logs_json: Vec<_> = logs.into_iter().map(|l| {
                let tipo_evento: String = l.get("tipo_evento");
                let status: String = l.get("status");
                let mensagem: String = l.get("mensagem");
                let criado_em: chrono::DateTime<chrono::Utc> = l.get("criado_em");
                json!({
                    "tipo_evento": tipo_evento,
                    "status": status,
                    "mensagem": mensagem,
                    "criado_em": criado_em
                })
            }).collect();

            Ok((StatusCode::OK, Json(RespostaBase::ok("Diagnóstico do terminal", json!({
                "terminal": {
                    "id": t_id,
                    "codigo_terminal": t_codigo,
                    "nome_terminal": t_nome,
                    "ativo": t_ativo,
                    "autorizado": t_autorizado,
                    "status_sync": t_status,
                    "ultima_sincronizacao": t_ultima_sync,
                    "ultima_versao_recebida": t_ultima_versao
                },
                "ultimos_logs": logs_json
            })))))
        },
        None => Err(ErroApi::bad_request("Terminal não encontrado", "NAO_ENCONTRADO"))
    }
}
