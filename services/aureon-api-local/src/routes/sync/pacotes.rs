use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use crate::{app::AppState, erros::ErroApi};
use aureon_core::RespostaBase;

#[derive(Deserialize)]
pub struct PrimeiraSyncReq {
    pub idempotency_key: String,
    pub terminal_id: Uuid,
}

pub async fn primeira_sincronizacao(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PrimeiraSyncReq>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    // Validar token/chave terminal
    let auth_header = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));
    
    let chave = auth_header.ok_or(ErroApi::bad_request("Chave do terminal ausente", "NAO_AUTORIZADO"))?;

    // Validar se o terminal existe, está ativo, autorizado e se a chave bate
    let term = sqlx::query(
        "SELECT id, ativo, autorizado FROM terminais_pdv WHERE id = $1 AND chave_terminal = $2"
    )
    .bind(payload.terminal_id)
    .bind(chave)
    .fetch_optional(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    if let Some(row) = term {
        let ativo: bool = row.get("ativo");
        let autorizado: bool = row.get("autorizado");
        if !ativo {
            return Err(ErroApi::bad_request("Terminal inativo", "NAO_AUTORIZADO"));
        }
        if !autorizado {
            return Err(ErroApi::bad_request("Terminal não autorizado", "NAO_AUTORIZADO"));
        }
    } else {
        return Err(ErroApi::bad_request("Terminal inválido ou chave incorreta", "NAO_AUTORIZADO"));
    }

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
            // Retorna o resultado anterior
            return Ok((StatusCode::OK, Json(RespostaBase::ok("Pacote recuperado (idempotência)", res))));
        }
    }

    // Gerar pacote na tabela pacotes_sincronizacao
    let pacote_id = Uuid::new_v4();
    let hash_geral = "hash-placeholder".to_string(); // será calculado com base nos itens

    let mut tx = pool.begin().await.map_err(|e| ErroApi::interno(e.to_string()))?;

    // Registra o pacote
    sqlx::query(
        "INSERT INTO pacotes_sincronizacao (id, terminal_id, tipo_pacote, status, idempotency_key, versao_geral, total_itens)
         VALUES ($1, $2, 'PRIMEIRA_SYNC', 'GERADO', $3, $4, 0)"
    )
    .bind(pacote_id)
    .bind(payload.terminal_id)
    .bind(&payload.idempotency_key)
    .bind(&hash_geral)
    .execute(&mut *tx)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    // Placeholder do pacote (na implementação completa aqui leríamos as versões reais)
    let resposta_pacote = json!({
        "pacote_id": pacote_id,
        "tipo": "PRIMEIRA_SYNC",
        "grupos_dados": [
            "empresa_config", "moedas_cotacoes", "usuarios_permissoes",
            "produtos_catalogo", "produtos_precos", "produtos_fiscal",
            "produtos_complementos", "configuracoes_operacionais", "dispositivos_perifericos"
        ]
    });

    sqlx::query(
        "INSERT INTO sync_idempotencia (idempotency_key, event_type, resultado) VALUES ($1, 'PRIMEIRA_SYNC', $2)"
    )
    .bind(&payload.idempotency_key)
    .bind(serde_json::to_value(&resposta_pacote).unwrap())
    .execute(&mut *tx)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    tx.commit().await.map_err(|e| ErroApi::interno(e.to_string()))?;

    Ok((StatusCode::OK, Json(RespostaBase::ok("Pacote inicial gerado", resposta_pacote))))
}

#[derive(Deserialize)]
pub struct ConfirmacaoAplicacaoReq {
    pub pacote_id: Uuid,
    pub terminal_id: Uuid,
    pub sucesso: bool,
    pub erro_detalhes: Option<String>,
}

pub async fn confirmar_aplicacao(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ConfirmacaoAplicacaoReq>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    // Validar token/chave terminal
    let auth_header = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));
    let chave = auth_header.ok_or(ErroApi::bad_request("Chave do terminal ausente", "NAO_AUTORIZADO"))?;

    let term = sqlx::query(
        "SELECT id FROM terminais_pdv WHERE id = $1 AND chave_terminal = $2"
    )
    .bind(payload.terminal_id)
    .bind(chave)
    .fetch_optional(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    if term.is_none() {
        return Err(ErroApi::bad_request("Terminal inválido ou chave incorreta", "NAO_AUTORIZADO"));
    }

    // Logica de confirmação de aplicação
    let status_pacote = if payload.sucesso { "APLICADO" } else { "FALHOU" };

    sqlx::query(
        "UPDATE pacotes_sincronizacao SET status = $1, aplicado_em = NOW(), erro_detalhes = $2 WHERE id = $3"
    )
    .bind(status_pacote)
    .bind(&payload.erro_detalhes)
    .bind(payload.pacote_id)
    .execute(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    if payload.sucesso {
        sqlx::query(
            "UPDATE terminais_pdv SET status_sync = 'ATUALIZADO', ultima_sincronizacao = NOW(), primeiro_sync_concluido = TRUE WHERE id = $1"
        )
        .bind(payload.terminal_id)
        .execute(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

        sqlx::query(
            "INSERT INTO sync_logs (terminal_id, tipo_evento, status, mensagem, detalhe_json) VALUES ($1, 'PACOTE_APLICADO', 'SUCESSO', 'Pacote aplicado com sucesso.', '{}')"
        )
        .bind(payload.terminal_id)
        .execute(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;
    } else {
        sqlx::query(
            "UPDATE terminais_pdv SET status_sync = 'ERRO' WHERE id = $1"
        )
        .bind(payload.terminal_id)
        .execute(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

        sqlx::query(
            "INSERT INTO sync_logs (terminal_id, tipo_evento, status, mensagem, detalhe_json) VALUES ($1, 'ERRO_APLICACAO', 'ERRO', 'Falha ao aplicar pacote', $2)"
        )
        .bind(payload.terminal_id)
        .bind(json!({"erro": payload.erro_detalhes}))
        .execute(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;
    }

    Ok((StatusCode::OK, Json(RespostaBase::ok("Confirmação processada", json!({ "status": status_pacote })))))
}

pub async fn listar_versoes(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ErroApi> {
    let pool = state.pool.as_ref().ok_or(ErroApi::interno("Pool não configurado"))?;

    let versoes = sqlx::query("SELECT tipo_dado, versao, hash_conteudo, atualizado_em FROM sync_versoes_dados")
        .fetch_all(pool)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

    let result: Vec<_> = versoes.into_iter().map(|v| {
        let tipo_dado: String = v.get("tipo_dado");
        let versao: String = v.get("versao");
        let hash_conteudo: String = v.get("hash_conteudo");
        let atualizado_em: chrono::DateTime<chrono::Utc> = v.get("atualizado_em");
        json!({
            "tipo_dado": tipo_dado,
            "versao": versao,
            "hash_conteudo": hash_conteudo,
            "atualizado_em": atualizado_em
        })
    }).collect();

    Ok((StatusCode::OK, Json(RespostaBase::ok("Versões de dados", json!(result)))))
}
