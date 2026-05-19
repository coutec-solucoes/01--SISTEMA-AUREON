use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use crate::{app::AppState, erros::ErroApi, middleware::UsuarioLogado};
use aureon_core::RespostaBase;

pub async fn diagnostico_geral(
    State(state): State<AppState>,
    axum::extract::Extension(_usuario): axum::extract::Extension<UsuarioLogado>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    // Totais de terminais
    let terminais = sqlx::query("SELECT status_sync FROM terminais_pdv")
        .fetch_all(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

    let totais = terminais.len();
    
    let mut pendentes = 0;
    let mut ativos = 0;
    let mut com_erro = 0;

    for term in &terminais {
        let status: String = term.get("status_sync");
        match status.as_str() {
            "PENDENTE" => pendentes += 1,
            "ATUALIZADO" => ativos += 1,
            "ERRO" => com_erro += 1,
            _ => {}
        }
    }

    // Última publicação
    let ultima_pub = sqlx::query("SELECT id, status, criado_em FROM sync_publicacoes ORDER BY criado_em DESC LIMIT 1")
        .fetch_optional(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

    let pub_json = ultima_pub.map(|p| {
        let id: Uuid = p.get("id");
        let status: String = p.get("status");
        let criado_em: chrono::DateTime<chrono::Utc> = p.get("criado_em");
        json!({
            "id": id,
            "status": status,
            "criado_em": criado_em
        })
    });

    // Versões atuais
    let versoes = sqlx::query("SELECT tipo_dado, versao FROM sync_versoes_dados")
        .fetch_all(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

    let versoes_json: Vec<_> = versoes.into_iter().map(|v| {
        let tipo_dado: String = v.get("tipo_dado");
        let versao: String = v.get("versao");
        json!({
            "tipo_dado": tipo_dado,
            "versao": versao
        })
    }).collect();

    // Logs recentes
    let logs = sqlx::query("SELECT id, terminal_id, tipo_evento, status, mensagem, criado_em FROM sync_logs ORDER BY criado_em DESC LIMIT 20")
        .fetch_all(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

    let logs_json: Vec<_> = logs.into_iter().map(|l| {
        let id: i32 = l.get("id");
        let terminal_id: Option<Uuid> = l.try_get("terminal_id").unwrap_or(None);
        let tipo_evento: String = l.get("tipo_evento");
        let status: String = l.get("status");
        let mensagem: String = l.get("mensagem");
        let criado_em: chrono::DateTime<chrono::Utc> = l.get("criado_em");

        json!({
            "id": id,
            "terminal_id": terminal_id,
            "tipo_evento": tipo_evento,
            "status": status,
            "mensagem": mensagem,
            "criado_em": criado_em
        })
    }).collect();

    Ok((StatusCode::OK, Json(RespostaBase::ok("Diagnóstico de Sincronização", json!({
        "terminais": {
            "total": totais,
            "ativos": ativos,
            "pendentes": pendentes,
            "erros": com_erro
        },
        "ultima_publicacao": pub_json,
        "versoes": versoes_json,
        "ultimos_logs": logs_json
    })))))
}

pub async fn listar_logs(
    State(state): State<AppState>,
    axum::extract::Extension(_usuario): axum::extract::Extension<UsuarioLogado>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    let logs = sqlx::query("SELECT id, terminal_id, tipo_evento, status, mensagem, detalhe_json, criado_em FROM sync_logs ORDER BY criado_em DESC LIMIT 100")
        .fetch_all(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

    let logs_json: Vec<_> = logs.into_iter().map(|l| {
        let id: i32 = l.get("id");
        let terminal_id: Option<Uuid> = l.try_get("terminal_id").unwrap_or(None);
        let tipo_evento: String = l.get("tipo_evento");
        let status: String = l.get("status");
        let mensagem: String = l.get("mensagem");
        let detalhe: Option<serde_json::Value> = l.try_get("detalhe_json").unwrap_or(None);
        let criado_em: chrono::DateTime<chrono::Utc> = l.get("criado_em");

        json!({
            "id": id,
            "terminal_id": terminal_id,
            "tipo_evento": tipo_evento,
            "status": status,
            "mensagem": mensagem,
            "detalhe": detalhe,
            "criado_em": criado_em
        })
    }).collect();

    Ok((StatusCode::OK, Json(RespostaBase::ok("Logs de sincronização", json!(logs_json)))))
}
