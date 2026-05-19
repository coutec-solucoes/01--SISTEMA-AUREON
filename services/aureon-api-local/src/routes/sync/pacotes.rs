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
    
    let mut tx = pool.begin().await.map_err(|e| ErroApi::interno(e.to_string()))?;

    // Consulta versoes de dados reais do PostgreSQL
    let versoes = sqlx::query("SELECT tipo_dado, versao, hash_conteudo FROM sync_versoes_dados")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

    let mut grupos_dados = Vec::new();
    let mut itens_payload = Vec::new();

    for row in &versoes {
        let tipo_dado: String = row.try_get("tipo_dado").unwrap_or_default();
        let versao: i32 = row.try_get("versao").unwrap_or(1);
        let hash_conteudo: String = row.try_get("hash_conteudo").unwrap_or_default();

        // Consulta real de dados baseados no grupo
        let payload_json: serde_json::Value = match tipo_dado.as_str() {
            "empresa_config" => {
                let empresas: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(row_to_json(t)), '[]') FROM empresas t").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                json!({ "empresas": empresas })
            },
            "moedas_cotacoes" => {
                let moedas: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(row_to_json(t)), '[]') FROM moedas t").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                json!({ "moedas": moedas })
            },
            "usuarios_permissoes" => {
                // Removido senha e hash sensível por regra
                let usuarios: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(json_build_object('id', t.id, 'nome', t.nome, 'ativo', t.ativo)), '[]') FROM usuarios t").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                let perfis: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(row_to_json(t)), '[]') FROM perfis t").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                json!({ "usuarios": usuarios, "perfis": perfis })
            },
            "produtos_catalogo" => {
                let produtos: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(row_to_json(t)), '[]') FROM produtos t").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                json!({ "produtos": produtos })
            },
            "produtos_precos" => {
                let precos: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(json_build_object('produto_id', id, 'preco_custo', preco_custo, 'preco_venda', preco_venda)), '[]') FROM produtos").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                json!({ "precos": precos })
            },
            "produtos_fiscal" => {
                let fiscais: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(row_to_json(t)), '[]') FROM produtos_fiscal t").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                json!({ "fiscais": fiscais })
            },
            "produtos_complementos" => {
                let adicionais: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(row_to_json(t)), '[]') FROM adicionais t").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                let prod_adicionais: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(row_to_json(t)), '[]') FROM produtos_adicionais t").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                json!({ "adicionais": adicionais, "produtos_adicionais": prod_adicionais })
            },
            "configuracoes_operacionais" => {
                let terminais: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(json_build_object('id', t.id, 'status_sync', t.status_sync, 'ativo', t.ativo)), '[]') FROM terminais_pdv t").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                let configs_pdv: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(row_to_json(t)), '[]') FROM configuracoes_pdv t").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                let regras: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(row_to_json(t)), '[]') FROM regras_venda t").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                json!({ "terminais_pdv": terminais, "configuracoes_pdv": configs_pdv, "regras_venda": regras })
            },
            "dispositivos_perifericos" => {
                let perifericos: serde_json::Value = sqlx::query_scalar("SELECT COALESCE(json_agg(row_to_json(t)), '[]') FROM perifericos t").fetch_one(&mut *tx).await.unwrap_or(json!([]));
                json!({ "perifericos": perifericos })
            },
            _ => json!({}),
        };

        // Insere o item do pacote
        sqlx::query(
            "INSERT INTO pacotes_sincronizacao_itens (pacote_id, tipo_dado, versao, hash_conteudo, payload_json)
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(pacote_id)
        .bind(&tipo_dado)
        .bind(versao)
        .bind(&hash_conteudo)
        .bind(&payload_json)
        .execute(&mut *tx)
        .await
        .map_err(|e| ErroApi::interno(e.to_string()))?;

        grupos_dados.push(tipo_dado.clone());
        itens_payload.push(json!({
            "tipo_dado": tipo_dado,
            "versao": versao,
            "hash_conteudo": hash_conteudo,
            "payload": payload_json
        }));
    }

    let hash_geral = format!("hash-{}", Uuid::new_v4()); // Hash real seria digest SHA256 do pacote
    let total_itens = grupos_dados.len() as i32;

    // Registra o pacote pai
    sqlx::query(
        "INSERT INTO pacotes_sincronizacao (id, terminal_id, tipo_pacote, status, idempotency_key, versao_geral, total_itens)
         VALUES ($1, $2, 'PRIMEIRA_SYNC', 'GERADO', $3, $4, $5)"
    )
    .bind(pacote_id)
    .bind(payload.terminal_id)
    .bind(&payload.idempotency_key)
    .bind(&hash_geral)
    .bind(total_itens)
    .execute(&mut *tx)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    let resposta_pacote = json!({
        "pacote_id": pacote_id,
        "tipo": "PRIMEIRA_SYNC",
        "grupos_dados": itens_payload
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
