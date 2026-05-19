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
pub struct PublicarReq {
    pub idempotency_key: String,
    pub observacao: Option<String>,
}

pub async fn publicar(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(payload): Json<PublicarReq>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    // Validar idempotência
    let idempotencia_existente = sqlx::query(
        "SELECT resultado FROM sync_idempotencia WHERE idempotency_key = $1"
    )
    .bind(&payload.idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    if let Some(idem) = idempotencia_existente {
        let resultado_json: Option<serde_json::Value> = idem.try_get("resultado").unwrap_or(None);
        if let Some(res) = resultado_json {
            return Ok((StatusCode::OK, Json(RespostaBase::ok("Publicação já iniciada (idempotência)", res))));
        }
    }

    let publicacao_id = Uuid::new_v4();

    let mut tx = pool.begin().await.map_err(|e| ErroApi::interno(e.to_string()))?;

    // Criar publicação
    sqlx::query(
        "INSERT INTO sync_publicacoes (id, tipo_publicacao, status, idempotency_key, criado_por, observacao)
         VALUES ($1, 'GERAL', 'EM_ANDAMENTO', $2, $3, $4)"
    )
    .bind(publicacao_id)
    .bind(&payload.idempotency_key)
    .bind(usuario.usuario_id)
    .bind(&payload.observacao)
    .execute(&mut *tx)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    // Buscar terminais ativos e autorizados
    let terminais = sqlx::query("SELECT id FROM terminais_pdv WHERE ativo = TRUE AND autorizado = TRUE")
        .fetch_all(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

    let total_terminais = terminais.len() as i32;

    for term in terminais {
        let term_id: Uuid = term.get("id");
        sqlx::query(
            "INSERT INTO sync_publicacoes_itens (publicacao_id, terminal_id, status) VALUES ($1, $2, 'PENDENTE')"
        )
        .bind(publicacao_id)
        .bind(term_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;
    }

    sqlx::query(
        "UPDATE sync_publicacoes SET total_terminais = $1 WHERE id = $2"
    )
    .bind(total_terminais)
    .bind(publicacao_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    let resposta = json!({
        "publicacao_id": publicacao_id,
        "total_terminais_alvo": total_terminais,
        "status": "EM_ANDAMENTO"
    });

    sqlx::query(
        "INSERT INTO sync_idempotencia (idempotency_key, event_type, resultado) VALUES ($1, 'PUBLICACAO_MANUAL', $2)"
    )
    .bind(&payload.idempotency_key)
    .bind(serde_json::to_value(&resposta).unwrap())
    .execute(&mut *tx)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    tx.commit().await.map_err(|e| ErroApi::interno(e.to_string()))?;

    Ok((StatusCode::OK, Json(RespostaBase::ok("Publicação iniciada", resposta))))
}

pub async fn listar_publicacoes(
    State(state): State<AppState>,
    axum::extract::Extension(_usuario): axum::extract::Extension<UsuarioLogado>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    let publicacoes = sqlx::query("SELECT id, tipo_publicacao, status, observacao, total_terminais, terminais_ok, terminais_erro, criado_em FROM sync_publicacoes ORDER BY criado_em DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

    let json_pubs: Vec<_> = publicacoes.into_iter().map(|p| {
        let id: Uuid = p.get("id");
        let tipo_publicacao: String = p.get("tipo_publicacao");
        let status: String = p.get("status");
        let observacao: Option<String> = p.try_get("observacao").unwrap_or(None);
        let total_terminais: i32 = p.get("total_terminais");
        let terminais_ok: i32 = p.get("terminais_ok");
        let terminais_erro: i32 = p.get("terminais_erro");
        let criado_em: chrono::DateTime<chrono::Utc> = p.get("criado_em");
        json!({
            "id": id,
            "tipo_publicacao": tipo_publicacao,
            "status": status,
            "observacao": observacao,
            "total_terminais": total_terminais,
            "terminais_ok": terminais_ok,
            "terminais_erro": terminais_erro,
            "criado_em": criado_em
        })
    }).collect();

    Ok((StatusCode::OK, Json(RespostaBase::ok("Publicações", json!(json_pubs)))))
}

pub async fn obter_publicacao(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(_usuario): axum::extract::Extension<UsuarioLogado>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    let publ = sqlx::query("SELECT id, tipo_publicacao, status, observacao, total_terminais, terminais_ok, terminais_erro, criado_em FROM sync_publicacoes WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

    match publ {
        Some(p) => {
            let p_id: Uuid = p.get("id");
            let p_tipo: String = p.get("tipo_publicacao");
            let p_status: String = p.get("status");
            let p_obs: Option<String> = p.try_get("observacao").unwrap_or(None);
            let p_total: i32 = p.get("total_terminais");
            let p_ok: i32 = p.get("terminais_ok");
            let p_erro: i32 = p.get("terminais_erro");
            let p_criado_em: chrono::DateTime<chrono::Utc> = p.get("criado_em");

            let itens = sqlx::query("SELECT terminal_id, status, ultimo_erro FROM sync_publicacoes_itens WHERE publicacao_id = $1")
                .bind(id)
                .fetch_all(pool)
                .await
                .map_err(|e| ErroApi::interno(e.to_string()))?;
            
            let itens_json: Vec<_> = itens.into_iter().map(|i| {
                let term_id: Uuid = i.get("terminal_id");
                let status: String = i.get("status");
                let ultimo_erro: Option<String> = i.try_get("ultimo_erro").unwrap_or(None);
                json!({
                    "terminal_id": term_id,
                    "status": status,
                    "ultimo_erro": ultimo_erro
                })
            }).collect();

            Ok((StatusCode::OK, Json(RespostaBase::ok("Publicação", json!({
                "id": p_id,
                "tipo_publicacao": p_tipo,
                "status": p_status,
                "observacao": p_obs,
                "total_terminais": p_total,
                "terminais_ok": p_ok,
                "terminais_erro": p_erro,
                "criado_em": p_criado_em,
                "itens": itens_json
            })))))
        },
        None => Err(ErroApi::bad_request("Publicação não encontrada", "NAO_ENCONTRADO"))
    }
}

pub async fn reprocessar_publicacao(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(_usuario): axum::extract::Extension<UsuarioLogado>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    sqlx::query(
        "UPDATE sync_publicacoes_itens SET status = 'PENDENTE', ultimo_erro = NULL WHERE publicacao_id = $1 AND status = 'ERRO'"
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    sqlx::query(
        "UPDATE sync_publicacoes SET status = 'EM_ANDAMENTO' WHERE id = $1"
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    Ok((StatusCode::OK, Json(RespostaBase::ok("Publicação reprocessada", json!({"status": "reprocessando"})))))
}

pub async fn eventos_pendentes(
    State(state): State<AppState>,
    Path(terminal_id): Path<Uuid>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    let eventos = sqlx::query(
        "SELECT id, publicacao_id, status FROM sync_publicacoes_itens WHERE terminal_id = $1 AND status = 'PENDENTE'"
    )
    .bind(terminal_id)
    .fetch_all(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    let json_eventos: Vec<_> = eventos.into_iter().map(|e| {
        let item_id: Uuid = e.get("id");
        let publ_id: Uuid = e.get("publicacao_id");
        let status: String = e.get("status");
        json!({
            "item_id": item_id,
            "publicacao_id": publ_id,
            "status": status
        })
    }).collect();

    Ok((StatusCode::OK, Json(RespostaBase::ok("Eventos pendentes", json!(json_eventos)))))
}

#[derive(Deserialize)]
pub struct ConfirmacaoEventoReq {
    pub item_id: Uuid,
    pub sucesso: bool,
    pub erro: Option<String>,
}

pub async fn confirmar_evento(
    State(state): State<AppState>,
    Json(payload): Json<ConfirmacaoEventoReq>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    let novo_status = if payload.sucesso { "APLICADO" } else { "ERRO" };

    sqlx::query(
        "UPDATE sync_publicacoes_itens SET status = $1, ultimo_erro = $2, atualizado_em = NOW() WHERE id = $3"
    )
    .bind(novo_status)
    .bind(&payload.erro)
    .bind(payload.item_id)
    .execute(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    Ok((StatusCode::OK, Json(RespostaBase::ok("Evento confirmado", json!({"status": novo_status})))))
}
