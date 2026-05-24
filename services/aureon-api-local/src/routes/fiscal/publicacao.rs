use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;
use crate::app::AppState;

// ----------------------------------------------------------------
// DTOs
// ----------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicarVersaoFiscalReq {
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicacaoFiscalResp {
    pub versao_id: String,
    pub versao: String,
    pub status: String,
    pub total_registros: i64,
    pub payload_hash: String,
    pub publicado_em: String,
}

// ----------------------------------------------------------------
// Helpers internos
// ----------------------------------------------------------------

/// Monta o payload fiscal versionado a partir dos dados da Retaguarda.
/// Filtra por pais_fiscal, empresa_id e filial_id quando disponíveis.
async fn montar_payload_fiscal(
    pool: &sqlx::PgPool,
    pais_fiscal: &str,
    empresa_id: Option<Uuid>,
    filial_id: Option<Uuid>,
) -> Result<(serde_json::Value, i64), sqlx::Error> {
    // --- fiscal_empresa_config ---
    let configs = sqlx::query(
        "SELECT id, empresa_id, filial_id, pais_fiscal, regime_fiscal, ambiente, forma_emissao, ativo, atualizado_em
         FROM fiscal_empresas_config
         WHERE pais_fiscal = $1
           AND ($2::uuid IS NULL OR empresa_id = $2)
           AND ($3::uuid IS NULL OR filial_id = $3)
         ORDER BY criado_em DESC LIMIT 50"
    )
    .bind(pais_fiscal)
    .bind(empresa_id)
    .bind(filial_id)
    .fetch_all(pool)
    .await?;

    let fiscal_empresa_config: Vec<serde_json::Value> = configs.iter().map(|r| json!({
        "id": r.get::<Uuid, _>("id").to_string(),
        "empresa_id": r.get::<Option<Uuid>, _>("empresa_id").map(|u| u.to_string()),
        "filial_id": r.get::<Option<Uuid>, _>("filial_id").map(|u| u.to_string()),
        "pais_fiscal": r.get::<String, _>("pais_fiscal"),
        "regime_fiscal": r.get::<Option<String>, _>("regime_fiscal"),
        "ambiente": r.get::<String, _>("ambiente"),
        "forma_emissao": r.get::<String, _>("forma_emissao"),
        "ativo": r.get::<bool, _>("ativo"),
        "atualizado_em": r.get::<chrono::DateTime<chrono::Utc>, _>("atualizado_em").to_rfc3339(),
        "operacao": "UPSERT"
    })).collect();

    // --- fiscal_numeracao_mestre ---
    let numeracoes = sqlx::query(
        "SELECT id, empresa_id, filial_id, pais_fiscal, tipo_documento, serie, proximo_numero, ativo, atualizado_em
         FROM fiscal_numeracao_mestre
         WHERE pais_fiscal = $1
           AND ($2::uuid IS NULL OR empresa_id = $2)
           AND ($3::uuid IS NULL OR filial_id = $3)
         LIMIT 200"
    )
    .bind(pais_fiscal)
    .bind(empresa_id)
    .bind(filial_id)
    .fetch_all(pool)
    .await?;

    let fiscal_numeracao: Vec<serde_json::Value> = numeracoes.iter().map(|r| json!({
        "id": r.get::<Uuid, _>("id").to_string(),
        "empresa_id": r.get::<Option<Uuid>, _>("empresa_id").map(|u| u.to_string()),
        "filial_id": r.get::<Option<Uuid>, _>("filial_id").map(|u| u.to_string()),
        "pais_fiscal": r.get::<String, _>("pais_fiscal"),
        "tipo_documento": r.get::<String, _>("tipo_documento"),
        "serie": r.get::<String, _>("serie"),
        "proximo_numero": r.get::<i64, _>("proximo_numero"),
        "ativo": r.get::<bool, _>("ativo"),
        "atualizado_em": r.get::<chrono::DateTime<chrono::Utc>, _>("atualizado_em").to_rfc3339(),
        "operacao": "UPSERT"
    })).collect();

    // --- fiscal_dicionario_ncm ---
    let ncms = sqlx::query(
        "SELECT id, codigo, descricao, ativo, atualizado_em
         FROM fiscal_dicionario_ncm
         ORDER BY codigo ASC LIMIT 5000"
    )
    .fetch_all(pool)
    .await?;

    let fiscal_ncm: Vec<serde_json::Value> = ncms.iter().map(|r| json!({
        "id": r.get::<Uuid, _>("id").to_string(),
        "codigo": r.get::<String, _>("codigo"),
        "descricao": r.get::<String, _>("descricao"),
        "ativo": r.get::<bool, _>("ativo"),
        "atualizado_em": r.get::<chrono::DateTime<chrono::Utc>, _>("atualizado_em").to_rfc3339(),
        "operacao": if r.get::<bool, _>("ativo") { "UPSERT" } else { "DELETE_LOGICO" }
    })).collect();

    // --- fiscal_dicionario_cfop ---
    let cfops = sqlx::query(
        "SELECT id, codigo, descricao, tipo_operacao, ativo, atualizado_em
         FROM fiscal_dicionario_cfop
         ORDER BY codigo ASC LIMIT 2000"
    )
    .fetch_all(pool)
    .await?;

    let fiscal_cfop: Vec<serde_json::Value> = cfops.iter().map(|r| json!({
        "id": r.get::<Uuid, _>("id").to_string(),
        "codigo": r.get::<String, _>("codigo"),
        "descricao": r.get::<String, _>("descricao"),
        "tipo_operacao": r.get::<Option<String>, _>("tipo_operacao"),
        "ativo": r.get::<bool, _>("ativo"),
        "atualizado_em": r.get::<chrono::DateTime<chrono::Utc>, _>("atualizado_em").to_rfc3339(),
        "operacao": if r.get::<bool, _>("ativo") { "UPSERT" } else { "DELETE_LOGICO" }
    })).collect();

    // --- fiscal_dicionario_cst_csosn ---
    let cst_csosns = sqlx::query(
        "SELECT id, codigo, tipo, descricao, ativo, atualizado_em
         FROM fiscal_dicionario_cst_csosn
         ORDER BY codigo ASC LIMIT 200"
    )
    .fetch_all(pool)
    .await?;

    let fiscal_cst_csosn: Vec<serde_json::Value> = cst_csosns.iter().map(|r| json!({
        "id": r.get::<Uuid, _>("id").to_string(),
        "codigo": r.get::<String, _>("codigo"),
        "tipo": r.get::<String, _>("tipo"),
        "descricao": r.get::<String, _>("descricao"),
        "ativo": r.get::<bool, _>("ativo"),
        "atualizado_em": r.get::<chrono::DateTime<chrono::Utc>, _>("atualizado_em").to_rfc3339(),
        "operacao": if r.get::<bool, _>("ativo") { "UPSERT" } else { "DELETE_LOGICO" }
    })).collect();

    // --- fiscal_dicionario_iva (filtrado por país) ---
    let ivas = sqlx::query(
        "SELECT id, codigo, descricao, pais_fiscal, aliquota_escala6, ativo, atualizado_em
         FROM fiscal_dicionario_iva
         WHERE pais_fiscal = $1
         ORDER BY codigo ASC LIMIT 200"
    )
    .bind(pais_fiscal)
    .fetch_all(pool)
    .await?;

    let fiscal_iva: Vec<serde_json::Value> = ivas.iter().map(|r| json!({
        "id": r.get::<Uuid, _>("id").to_string(),
        "codigo": r.get::<String, _>("codigo"),
        "descricao": r.get::<String, _>("descricao"),
        "pais_fiscal": r.get::<String, _>("pais_fiscal"),
        "aliquota_escala6": r.get::<i64, _>("aliquota_escala6"),
        "ativo": r.get::<bool, _>("ativo"),
        "atualizado_em": r.get::<chrono::DateTime<chrono::Utc>, _>("atualizado_em").to_rfc3339(),
        "operacao": if r.get::<bool, _>("ativo") { "UPSERT" } else { "DELETE_LOGICO" }
    })).collect();

    // --- fiscal_regras_tributarias_mestre (filtrado por país/empresa/filial) ---
    let regras = sqlx::query(
        "SELECT id, empresa_id, filial_id, pais_fiscal, tipo_operacao, uf_origem, uf_destino,
                ncm_id, cfop_id, cst_csosn_id, iva_id,
                aliquota_icms_escala6, aliquota_pis_escala6, aliquota_cofins_escala6,
                aliquota_iva_escala6, reducao_base_escala6, prioridade, ativo, atualizado_em
         FROM fiscal_regras_tributarias_mestre
         WHERE pais_fiscal = $1
           AND ($2::uuid IS NULL OR empresa_id = $2)
           AND ($3::uuid IS NULL OR filial_id = $3)
         ORDER BY COALESCE(prioridade, 0) DESC LIMIT 500"
    )
    .bind(pais_fiscal)
    .bind(empresa_id)
    .bind(filial_id)
    .fetch_all(pool)
    .await?;

    let fiscal_regras_tributarias: Vec<serde_json::Value> = regras.iter().map(|r| json!({
        "id": r.get::<Uuid, _>("id").to_string(),
        "empresa_id": r.get::<Option<Uuid>, _>("empresa_id").map(|u| u.to_string()),
        "filial_id": r.get::<Option<Uuid>, _>("filial_id").map(|u| u.to_string()),
        "pais_fiscal": r.get::<String, _>("pais_fiscal"),
        "tipo_operacao": r.get::<Option<String>, _>("tipo_operacao"),
        "uf_origem": r.get::<Option<String>, _>("uf_origem"),
        "uf_destino": r.get::<Option<String>, _>("uf_destino"),
        "ncm_id": r.get::<Option<Uuid>, _>("ncm_id").map(|u| u.to_string()),
        "cfop_id": r.get::<Option<Uuid>, _>("cfop_id").map(|u| u.to_string()),
        "cst_csosn_id": r.get::<Option<Uuid>, _>("cst_csosn_id").map(|u| u.to_string()),
        "iva_id": r.get::<Option<Uuid>, _>("iva_id").map(|u| u.to_string()),
        "aliquota_icms_escala6": r.get::<Option<i64>, _>("aliquota_icms_escala6"),
        "aliquota_pis_escala6": r.get::<Option<i64>, _>("aliquota_pis_escala6"),
        "aliquota_cofins_escala6": r.get::<Option<i64>, _>("aliquota_cofins_escala6"),
        "aliquota_iva_escala6": r.get::<Option<i64>, _>("aliquota_iva_escala6"),
        "reducao_base_escala6": r.get::<Option<i64>, _>("reducao_base_escala6"),
        "prioridade": r.get::<Option<i32>, _>("prioridade"),
        "ativo": r.get::<bool, _>("ativo"),
        "atualizado_em": r.get::<chrono::DateTime<chrono::Utc>, _>("atualizado_em").to_rfc3339(),
        "operacao": if r.get::<bool, _>("ativo") { "UPSERT" } else { "DELETE_LOGICO" }
    })).collect();

    // Conta total de registros no payload
    let total: i64 = (fiscal_empresa_config.len()
        + fiscal_numeracao.len()
        + fiscal_ncm.len()
        + fiscal_cfop.len()
        + fiscal_cst_csosn.len()
        + fiscal_iva.len()
        + fiscal_regras_tributarias.len()) as i64;

    let payload = json!({
        "versao_schema": "1.0",
        "pais_fiscal": pais_fiscal,
        "blocos": {
            "fiscal_empresa_config": fiscal_empresa_config,
            "fiscal_numeracao": fiscal_numeracao,
            "fiscal_ncm": fiscal_ncm,
            "fiscal_cfop": fiscal_cfop,
            "fiscal_cst_csosn": fiscal_cst_csosn,
            "fiscal_iva": fiscal_iva,
            "fiscal_regras_tributarias": fiscal_regras_tributarias
        },
        "total_registros": total
    });

    Ok((payload, total))
}

/// Calcula hash SHA-256 simples usando UUID determinístico do conteúdo serializado.
/// RISCO CONTROLADO: hash real com SHA-256 pode ser adicionado como crate futuramente.
fn calcular_hash_payload(payload: &serde_json::Value) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let s = payload.to_string();
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("fnv1a-{:x}", hasher.finish())
}

// ----------------------------------------------------------------
// Handlers de publicação
// ----------------------------------------------------------------

/// POST /fiscal/versoes/:id/publicar
pub async fn publicar_versao(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<PublicarVersaoFiscalReq>,
) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };

    let versao_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response(),
    };

    // --- Idempotência: se já existir chave, retorna resultado anterior ---
    if let Some(ref idem_key) = payload.idempotency_key {
        match sqlx::query(
            "SELECT resultado FROM sync_idempotencia WHERE idempotency_key = $1"
        )
        .bind(idem_key)
        .fetch_optional(pool)
        .await
        {
            Ok(Some(row)) => {
                let resultado: Option<serde_json::Value> = row.try_get("resultado").unwrap_or(None);
                if let Some(res) = resultado {
                    return (StatusCode::OK, Json(json!({"idempotente": true, "resultado": res}))).into_response();
                }
            }
            _ => {}
        }
    }

    // --- Validar que a versão existe e está em RASCUNHO ---
    let versao_row = match sqlx::query(
        "SELECT id, versao, pais_fiscal, empresa_id, filial_id, status
         FROM fiscal_versoes_publicacao
         WHERE id = $1"
    )
    .bind(versao_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"erro": "Versão não encontrada"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    };

    let status_atual: String = versao_row.get("status");
    if status_atual != "RASCUNHO" {
        return (StatusCode::CONFLICT, Json(json!({
            "erro": "Somente versões com status RASCUNHO podem ser publicadas",
            "status_atual": status_atual
        }))).into_response();
    }

    let versao_str: String = versao_row.get("versao");
    let pais_fiscal: String = versao_row.get("pais_fiscal");
    let empresa_uuid: Option<Uuid> = versao_row.get::<Option<Uuid>, _>("empresa_id");
    let filial_uuid: Option<Uuid> = versao_row.get::<Option<Uuid>, _>("filial_id");

    // --- Montar payload fiscal ---
    let (payload_fiscal, total_registros) = match montar_payload_fiscal(pool, &pais_fiscal, empresa_uuid, filial_uuid).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    };

    let payload_hash = calcular_hash_payload(&payload_fiscal);
    let agora = chrono::Utc::now();

    // --- Transação: publicar versão + itens + auditoria + sync ---
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    };

    // 1. Atualizar status da versão para PUBLICADA
    if let Err(e) = sqlx::query(
        "UPDATE fiscal_versoes_publicacao
         SET status = 'PUBLICADA', publicado_em = $2, payload_hash = $3, total_registros = $4, atualizado_em = now()
         WHERE id = $1"
    )
    .bind(versao_id)
    .bind(agora)
    .bind(&payload_hash)
    .bind(total_registros as i32)
    .execute(&mut *tx)
    .await
    {
        let _ = tx.rollback().await;
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response();
    }

    // 2. Limpar itens anteriores (idempotência de reprocessamento)
    let _ = sqlx::query(
        "DELETE FROM fiscal_versoes_publicacao_itens WHERE versao_id = $1"
    )
    .bind(versao_id)
    .execute(&mut *tx)
    .await;

    // 3. Inserir itens da versão por tipo de dado
    let tipos_dados = [
        ("FISCAL_EMPRESA_CONFIG", &payload_fiscal["blocos"]["fiscal_empresa_config"]),
        ("FISCAL_NUMERACAO", &payload_fiscal["blocos"]["fiscal_numeracao"]),
        ("FISCAL_NCM", &payload_fiscal["blocos"]["fiscal_ncm"]),
        ("FISCAL_CFOP", &payload_fiscal["blocos"]["fiscal_cfop"]),
        ("FISCAL_CST_CSOSN", &payload_fiscal["blocos"]["fiscal_cst_csosn"]),
        ("FISCAL_IVA", &payload_fiscal["blocos"]["fiscal_iva"]),
        ("FISCAL_REGRAS_TRIBUTARIAS", &payload_fiscal["blocos"]["fiscal_regras_tributarias"]),
    ];

    for (tipo_dado, bloco) in &tipos_dados {
        if let Some(arr) = bloco.as_array() {
            for item in arr {
                let registro_id_str = item["id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
                let operacao = item["operacao"].as_str().unwrap_or("UPSERT");

                if let Err(e) = sqlx::query(
                    "INSERT INTO fiscal_versoes_publicacao_itens (id, versao_id, tipo_dado, registro_id, operacao)
                     VALUES ($1, $2, $3, $4, $5)
                     ON CONFLICT DO NOTHING"
                )
                .bind(Uuid::new_v4())
                .bind(versao_id)
                .bind(tipo_dado)
                .bind(registro_id_str)
                .bind(operacao)
                .execute(&mut *tx)
                .await
                {
                    let _ = tx.rollback().await;
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response();
                }
            }
        }
    }

    // 4. Inserir entrada de sync (pacotes_sincronizacao) para os PDVs consumirem
    let pacote_sync_id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO pacotes_sincronizacao
         (id, terminal_id, tipo_pacote, status, idempotency_key, versao_geral, total_itens)
         VALUES ($1, NULL, 'SYNC_FISCAL', 'GERADO', $2, $3, $4)"
    )
    .bind(pacote_sync_id)
    .bind(format!("fiscal-{}-{}", versao_id, payload_hash))
    .bind(&versao_str)
    .bind(total_registros as i32)
    .execute(&mut *tx)
    .await
    {
        // Log sem falhar — sync é melhor esforço neste bloco
        tracing::warn!("Não foi possível registrar pacote sync fiscal: {}", e);
    } else {
        // Registrar item do pacote com payload completo
        let _ = sqlx::query(
            "INSERT INTO pacotes_sincronizacao_itens
             (pacote_id, tipo_dado, versao, hash_conteudo, payload_json)
             VALUES ($1, 'SYNC_FISCAL', $2, $3, $4)"
        )
        .bind(pacote_sync_id)
        .bind(&versao_str)
        .bind(&payload_hash)
        .bind(&payload_fiscal)
        .execute(&mut *tx)
        .await;
    }

    // 5. Auditoria fiscal
    let _ = sqlx::query(
        "INSERT INTO fiscal_auditoria_mestre
         (id, entidade, entidade_id, acao, payload_novo, usuario_id)
         VALUES ($1, 'FISCAL_VERSAO', $2, 'PUBLICAR_VERSAO_FISCAL', $3, NULL)"
    )
    .bind(Uuid::new_v4())
    .bind(versao_id)
    .bind(json!({
        "versao": versao_str,
        "pais_fiscal": pais_fiscal,
        "total_registros": total_registros,
        "payload_hash": payload_hash,
        "pacote_sync_id": pacote_sync_id
    }))
    .execute(&mut *tx)
    .await;

    // 6. Idempotência
    if let Some(ref idem_key) = payload.idempotency_key {
        let resultado = json!({
            "versao_id": versao_id,
            "versao": versao_str,
            "payload_hash": payload_hash,
            "total_registros": total_registros,
            "status": "PUBLICADA"
        });
        let _ = sqlx::query(
            "INSERT INTO sync_idempotencia (idempotency_key, event_type, resultado)
             VALUES ($1, 'FISCAL_VERSAO_PUBLICADA', $2)
             ON CONFLICT (idempotency_key) DO NOTHING"
        )
        .bind(idem_key)
        .bind(resultado)
        .execute(&mut *tx)
        .await;
    }

    if let Err(e) = tx.commit().await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response();
    }

    (StatusCode::OK, Json(json!({
        "versao_id": versao_id,
        "versao": versao_str,
        "status": "PUBLICADA",
        "total_registros": total_registros,
        "payload_hash": payload_hash,
        "publicado_em": agora.to_rfc3339(),
        "pacote_sync_id": pacote_sync_id,
        "aviso": "Pacote fiscal gerado. Aplicação no PDV ocorre no Bloco 4."
    }))).into_response()
}

/// POST /fiscal/versoes/:id/reprocessar
pub async fn reprocessar_versao(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };

    let versao_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response(),
    };

    // Validar que a versão existe e está PUBLICADA ou REPROCESSADA
    let versao_row = match sqlx::query(
        "SELECT id, versao, pais_fiscal, empresa_id, filial_id, status
         FROM fiscal_versoes_publicacao
         WHERE id = $1"
    )
    .bind(versao_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"erro": "Versão não encontrada"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    };

    let status_atual: String = versao_row.get("status");
    if status_atual != "PUBLICADA" && status_atual != "REPROCESSADA" {
        return (StatusCode::CONFLICT, Json(json!({
            "erro": "Somente versões PUBLICADA ou REPROCESSADA podem ser reprocessadas",
            "status_atual": status_atual
        }))).into_response();
    }

    let versao_str: String = versao_row.get("versao");
    let pais_fiscal: String = versao_row.get("pais_fiscal");
    let empresa_uuid: Option<Uuid> = versao_row.get::<Option<Uuid>, _>("empresa_id");
    let filial_uuid: Option<Uuid> = versao_row.get::<Option<Uuid>, _>("filial_id");

    // Remontar payload
    let (payload_fiscal, total_registros) = match montar_payload_fiscal(pool, &pais_fiscal, empresa_uuid, filial_uuid).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    };

    let payload_hash = calcular_hash_payload(&payload_fiscal);
    let agora = chrono::Utc::now();

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    };

    // Atualizar status para REPROCESSADA
    if let Err(e) = sqlx::query(
        "UPDATE fiscal_versoes_publicacao
         SET status = 'REPROCESSADA', publicado_em = $2, payload_hash = $3, total_registros = $4, atualizado_em = now()
         WHERE id = $1"
    )
    .bind(versao_id)
    .bind(agora)
    .bind(&payload_hash)
    .bind(total_registros as i32)
    .execute(&mut *tx)
    .await
    {
        let _ = tx.rollback().await;
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response();
    }

    // Auditoria
    let _ = sqlx::query(
        "INSERT INTO fiscal_auditoria_mestre
         (id, entidade, entidade_id, acao, payload_novo, usuario_id)
         VALUES ($1, 'FISCAL_VERSAO', $2, 'REPROCESSAR_VERSAO_FISCAL', $3, NULL)"
    )
    .bind(Uuid::new_v4())
    .bind(versao_id)
    .bind(json!({
        "versao": versao_str,
        "pais_fiscal": pais_fiscal,
        "total_registros": total_registros,
        "payload_hash": payload_hash
    }))
    .execute(&mut *tx)
    .await;

    if let Err(e) = tx.commit().await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response();
    }

    (StatusCode::OK, Json(json!({
        "versao_id": versao_id,
        "versao": versao_str,
        "status": "REPROCESSADA",
        "total_registros": total_registros,
        "payload_hash": payload_hash,
        "publicado_em": agora.to_rfc3339()
    }))).into_response()
}

/// GET /fiscal/versoes/:id/payload
pub async fn obter_payload_versao(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };

    let versao_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response(),
    };

    // Busca versão e valida que está publicada
    let versao_row = match sqlx::query(
        "SELECT id, versao, pais_fiscal, empresa_id, filial_id, status, payload_hash, total_registros, publicado_em
         FROM fiscal_versoes_publicacao
         WHERE id = $1"
    )
    .bind(versao_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"erro": "Versão não encontrada"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    };

    let status_atual: String = versao_row.get("status");
    if status_atual == "RASCUNHO" || status_atual == "CANCELADA" {
        return (StatusCode::CONFLICT, Json(json!({
            "erro": "Payload disponível somente para versões PUBLICADA ou REPROCESSADA",
            "status_atual": status_atual
        }))).into_response();
    }

    let pais_fiscal: String = versao_row.get("pais_fiscal");
    let empresa_uuid: Option<Uuid> = versao_row.get::<Option<Uuid>, _>("empresa_id");
    let filial_uuid: Option<Uuid> = versao_row.get::<Option<Uuid>, _>("filial_id");
    let versao_str: String = versao_row.get("versao");
    let payload_hash: Option<String> = versao_row.get("payload_hash");
    let total_registros: Option<i32> = versao_row.get("total_registros");
    let publicado_em: Option<chrono::DateTime<chrono::Utc>> = versao_row.get("publicado_em");

    // Buscar do pacote_sincronizacao se existir, senão regenerar on-demand
    let payload_from_sync = sqlx::query(
        "SELECT psi.payload_json
         FROM pacotes_sincronizacao ps
         JOIN pacotes_sincronizacao_itens psi ON psi.pacote_id = ps.id
         WHERE ps.tipo_pacote = 'SYNC_FISCAL'
           AND ps.versao_geral = $1
         ORDER BY ps.criado_em DESC LIMIT 1"
    )
    .bind(&versao_str)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let payload_fiscal = if let Some(row) = payload_from_sync {
        row.try_get::<serde_json::Value, _>("payload_json").unwrap_or(json!({}))
    } else {
        // Regenera on-demand sem gravar (somente leitura)
        match montar_payload_fiscal(pool, &pais_fiscal, empresa_uuid, filial_uuid).await {
            Ok((p, _)) => p,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
        }
    };

    // Auditoria de acesso ao payload
    let _ = sqlx::query(
        "INSERT INTO fiscal_auditoria_mestre
         (id, entidade, entidade_id, acao, payload_novo, usuario_id)
         VALUES ($1, 'FISCAL_VERSAO', $2, 'GERAR_PAYLOAD_FISCAL', $3, NULL)"
    )
    .bind(Uuid::new_v4())
    .bind(versao_id)
    .bind(json!({"versao": versao_str, "acesso": "leitura"}))
    .execute(pool)
    .await;

    (StatusCode::OK, Json(json!({
        "versao_id": versao_id,
        "versao": versao_str,
        "status": status_atual,
        "pais_fiscal": pais_fiscal,
        "payload_hash": payload_hash,
        "total_registros": total_registros,
        "publicado_em": publicado_em.map(|d| d.to_rfc3339()),
        "payload": payload_fiscal,
        "aviso": "Este payload é somente para consulta. Aplicação no PDV ocorre no Bloco 4."
    }))).into_response()
}

/// GET /fiscal/publicacoes
pub async fn listar_publicacoes_fiscais(State(state): State<AppState>) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };

    match sqlx::query(
        "SELECT id, tipo_pacote, status, versao_geral, total_itens, criado_em
         FROM pacotes_sincronizacao
         WHERE tipo_pacote = 'SYNC_FISCAL'
         ORDER BY criado_em DESC LIMIT 100"
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => {
            let resp: Vec<serde_json::Value> = rows.iter().map(|r| json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "tipo_pacote": r.get::<String, _>("tipo_pacote"),
                "status": r.get::<String, _>("status"),
                "versao_geral": r.get::<String, _>("versao_geral"),
                "total_itens": r.get::<i32, _>("total_itens"),
                "criado_em": r.get::<chrono::DateTime<chrono::Utc>, _>("criado_em").to_rfc3339()
            })).collect();
            (StatusCode::OK, Json(json!(resp))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    }
}

/// GET /fiscal/publicacoes/:id
pub async fn obter_publicacao_fiscal(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = match state.pool.as_ref() {
        Some(p) => p,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"erro": "Sem PostgreSQL"}))).into_response(),
    };

    let pacote_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"erro": "ID inválido"}))).into_response(),
    };

    let row = match sqlx::query(
        "SELECT id, tipo_pacote, status, versao_geral, total_itens, idempotency_key, criado_em
         FROM pacotes_sincronizacao
         WHERE id = $1 AND tipo_pacote = 'SYNC_FISCAL'"
    )
    .bind(pacote_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"erro": "Publicação fiscal não encontrada"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": e.to_string()}))).into_response(),
    };

    (StatusCode::OK, Json(json!({
        "id": row.get::<Uuid, _>("id").to_string(),
        "tipo_pacote": row.get::<String, _>("tipo_pacote"),
        "status": row.get::<String, _>("status"),
        "versao_geral": row.get::<String, _>("versao_geral"),
        "total_itens": row.get::<i32, _>("total_itens"),
        "idempotency_key": row.get::<String, _>("idempotency_key"),
        "criado_em": row.get::<chrono::DateTime<chrono::Utc>, _>("criado_em").to_rfc3339()
    }))).into_response()
}
