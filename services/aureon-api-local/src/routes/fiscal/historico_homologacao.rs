/// historico_homologacao.rs
/// Fase 19 — Bloco 5
/// Histórico de Homologação Fiscal, Tentativas Técnicas e Auditoria Local
///
/// ATENÇÃO CRÍTICA:
/// - Este módulo registra SOMENTE auditoria técnica interna.
/// - NÃO representa autorização, protocolo ou status fiscal real.
/// - NÃO tem validade jurídica perante SEFAZ/DNIT/SET.
/// - NÃO contém senha, chave privada ou certificado PFX.
/// - Ambiente sempre restrito a 'HOMOLOGACAO'.
/// - Histórico de homologação NÃO é autorização fiscal.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::Row;
use crate::app::AppState;

// ─────────────────────────────────────────────
// CONSTANTES
// ─────────────────────────────────────────────

/// Tamanho máximo do payload_preview em bytes (4KB).
const PAYLOAD_PREVIEW_MAX_BYTES: usize = 4096;

/// Tipos de evento controlados — espelham o ENUM da migration.
pub const TIPOS_EVENTO_VALIDOS: &[&str] = &[
    "CERTIFICADO_VALIDADO",
    "ASSINATURA_PREVIEW_EXECUTADA",
    "XMLDSIG_HOMOLOGACAO_TENTADO",
    "NFCE_PREVIEW_GERADO",
    "SIFEN_PREVIEW_GERADO",
    "PREVIEW_VALIDADO_LOCALMENTE",
    "QRCODE_PREVIEW_GERADO",
    "CONECTIVIDADE_HOMOLOGACAO_TESTADA",
    "PRODUCAO_BLOQUEADA",
    "ERRO_HOMOLOGACAO_TECNICA",
];

// ─────────────────────────────────────────────
// HELPER PÚBLICO: Registrar Evento
// ─────────────────────────────────────────────

/// Parâmetros para registrar um evento técnico de homologação.
/// Usado internamente por outros módulos fiscais.
///
/// ATENÇÃO: nunca incluir senha, chave privada, certificado PFX ou XML completo.
pub struct RegistrarEventoHomologacaoParams {
    pub tipo_evento: &'static str,
    pub pais: Option<String>,
    pub modelo: Option<String>,
    pub venda_id: Option<Uuid>,
    pub chave_preview: Option<String>,
    pub cdc_preview: Option<String>,
    pub sucesso: bool,
    pub mensagem: String,
    pub payload_hash: Option<String>,
    pub erro_codigo: Option<String>,
    /// Metadados técnicos resumidos — será truncado a PAYLOAD_PREVIEW_MAX_BYTES
    pub payload_preview: Option<JsonValue>,
}

/// Registra um evento técnico de homologação no banco de dados PostgreSQL.
/// Retorna o UUID gerado ou None se não houver pool disponível.
///
/// Segurança garantida:
/// - ambiente sempre forçado para 'HOMOLOGACAO'
/// - payload_preview serializado e truncado a 4KB
/// - nunca grava senha/chave/certificado
pub async fn registrar_evento_homologacao(
    pool: &sqlx::PgPool,
    params: RegistrarEventoHomologacaoParams,
) -> Option<Uuid> {
    let id = Uuid::new_v4();

    // Trunca payload_preview para segurança
    let payload_json: Option<JsonValue> = params.payload_preview.map(|p| {
        let serializado = serde_json::to_string(&p).unwrap_or_default();
        if serializado.len() > PAYLOAD_PREVIEW_MAX_BYTES {
            json!({
                "truncado": true,
                "aviso": "payload_preview excedeu 4KB e foi truncado por segurança",
                "tamanho_original_bytes": serializado.len()
            })
        } else {
            p
        }
    });

    let resultado = sqlx::query(
        r#"
        INSERT INTO fiscal_homologacao_historico
            (id, tipo_evento, pais, modelo, ambiente,
             venda_id, chave_preview, cdc_preview,
             sucesso, mensagem, payload_hash, erro_codigo, payload_preview)
        VALUES
            ($1, $2::fiscal_tipo_evento_hom, $3, $4, 'HOMOLOGACAO',
             $5, $6, $7,
             $8, $9, $10, $11, $12)
        "#,
    )
    .bind(id)
    .bind(params.tipo_evento)
    .bind(params.pais)
    .bind(params.modelo)
    .bind(params.venda_id)
    .bind(params.chave_preview)
    .bind(params.cdc_preview)
    .bind(params.sucesso)
    .bind(params.mensagem)
    .bind(params.payload_hash)
    .bind(params.erro_codigo)
    .bind(payload_json)
    .execute(pool)
    .await;

    match resultado {
        Ok(_) => Some(id),
        Err(e) => {
            tracing::warn!(
                "historico_homologacao: falha ao registrar evento {}: {}",
                params.tipo_evento, e
            );
            None
        }
    }
}

// ─────────────────────────────────────────────
// DTOs
// ─────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct HistoricoHomologacaoFiscalResp {
    pub id: String,
    pub tipo_evento: String,
    pub pais: Option<String>,
    pub modelo: Option<String>,
    pub ambiente: String,
    pub venda_id: Option<String>,
    pub chave_preview: Option<String>,
    pub cdc_preview: Option<String>,
    pub sucesso: bool,
    pub mensagem: Option<String>,
    pub payload_hash: Option<String>,
    pub erro_codigo: Option<String>,
    pub criado_em: String,
    pub aviso: String,
}

#[derive(Debug, Deserialize)]
pub struct ListarHistoricoQuery {
    pub tipo_evento: Option<String>,
    pub pais: Option<String>,
    pub modelo: Option<String>,
    pub sucesso: Option<bool>,
    pub venda_id: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ListarHistoricoResp {
    pub total: usize,
    pub limit: i64,
    pub historico: Vec<HistoricoHomologacaoFiscalResp>,
    pub aviso: String,
}

// ─────────────────────────────────────────────
// HANDLERS
// ─────────────────────────────────────────────

/// GET /fiscal/homologacao/historico
/// Lista o histórico técnico de operações de homologação.
/// NÃO retorna autorizações fiscais — apenas auditoria técnica interna.
pub async fn listar_historico(
    State(state): State<AppState>,
    Query(params): Query<ListarHistoricoQuery>,
) -> Result<Json<ListarHistoricoResp>, (axum::http::StatusCode, String)> {
    let pool = match &state.pool {
        Some(p) => p,
        None => {
            return Ok(Json(ListarHistoricoResp {
                total: 0,
                limit: 0,
                historico: vec![],
                aviso: "PostgreSQL não configurado. O histórico técnico requer banco de dados.".into(),
            }));
        }
    };

    let limit = params.limit.unwrap_or(50).min(200);

    // Converte venda_id string para Uuid opcional
    let venda_uuid: Option<Uuid> = params.venda_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let registros = sqlx::query(
        r#"
        SELECT
            id, tipo_evento::TEXT as tipo_evento, pais, modelo, ambiente,
            venda_id, chave_preview, cdc_preview,
            sucesso, mensagem, payload_hash, erro_codigo, criado_em
        FROM fiscal_homologacao_historico
        WHERE
            ($1::TEXT IS NULL OR tipo_evento::TEXT = $1)
            AND ($2::TEXT IS NULL OR pais = $2)
            AND ($3::TEXT IS NULL OR modelo = $3)
            AND ($4::BOOLEAN IS NULL OR sucesso = $4)
            AND ($5::UUID IS NULL OR venda_id = $5)
        ORDER BY criado_em DESC
        LIMIT $6
        "#,
    )
    .bind(params.tipo_evento)
    .bind(params.pais)
    .bind(params.modelo)
    .bind(params.sucesso)
    .bind(venda_uuid)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        format!("Erro ao consultar histórico: {}", e),
    ))?;

    let historico: Vec<HistoricoHomologacaoFiscalResp> = registros
        .iter()
        .map(|r| HistoricoHomologacaoFiscalResp {
            id: r.get::<Uuid, _>("id").to_string(),
            tipo_evento: r.get::<String, _>("tipo_evento"),
            pais: r.get::<Option<String>, _>("pais"),
            modelo: r.get::<Option<String>, _>("modelo"),
            ambiente: r.get::<String, _>("ambiente"),
            venda_id: r.get::<Option<Uuid>, _>("venda_id").map(|u| u.to_string()),
            chave_preview: r.get::<Option<String>, _>("chave_preview"),
            cdc_preview: r.get::<Option<String>, _>("cdc_preview"),
            sucesso: r.get::<bool, _>("sucesso"),
            mensagem: r.get::<Option<String>, _>("mensagem"),
            payload_hash: r.get::<Option<String>, _>("payload_hash"),
            erro_codigo: r.get::<Option<String>, _>("erro_codigo"),
            criado_em: r.get::<DateTime<Utc>, _>("criado_em").to_rfc3339(),
            aviso: "Este registro é auditoria técnica interna. Não representa autorização fiscal.".into(),
        })
        .collect();

    let total = historico.len();

    Ok(Json(ListarHistoricoResp {
        total,
        limit,
        historico,
        aviso: "Histórico de homologação técnica. Não representa autorização, protocolo ou documento fiscal com validade jurídica.".into(),
    }))
}

/// GET /fiscal/homologacao/historico/{id}
/// Retorna um registro específico do histórico técnico de homologação.
pub async fn obter_historico(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<HistoricoHomologacaoFiscalResp>, (axum::http::StatusCode, String)> {
    let pool = match &state.pool {
        Some(p) => p,
        None => {
            return Err((
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "PostgreSQL não configurado.".into(),
            ));
        }
    };

    let registro = sqlx::query(
        r#"
        SELECT
            id, tipo_evento::TEXT as tipo_evento, pais, modelo, ambiente,
            venda_id, chave_preview, cdc_preview,
            sucesso, mensagem, payload_hash, erro_codigo, criado_em
        FROM fiscal_homologacao_historico
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        format!("Erro ao consultar registro: {}", e),
    ))?;

    match registro {
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            format!("Registro de histórico {} não encontrado.", id),
        )),
        Some(r) => Ok(Json(HistoricoHomologacaoFiscalResp {
            id: r.get::<Uuid, _>("id").to_string(),
            tipo_evento: r.get::<String, _>("tipo_evento"),
            pais: r.get::<Option<String>, _>("pais"),
            modelo: r.get::<Option<String>, _>("modelo"),
            ambiente: r.get::<String, _>("ambiente"),
            venda_id: r.get::<Option<Uuid>, _>("venda_id").map(|u| u.to_string()),
            chave_preview: r.get::<Option<String>, _>("chave_preview"),
            cdc_preview: r.get::<Option<String>, _>("cdc_preview"),
            sucesso: r.get::<bool, _>("sucesso"),
            mensagem: r.get::<Option<String>, _>("mensagem"),
            payload_hash: r.get::<Option<String>, _>("payload_hash"),
            erro_codigo: r.get::<Option<String>, _>("erro_codigo"),
            criado_em: r.get::<DateTime<Utc>, _>("criado_em").to_rfc3339(),
            aviso: "Este registro é auditoria técnica interna. Não representa autorização fiscal.".into(),
        })),
    }
}

// ─────────────────────────────────────────────
// ENDPOINT: Listar tipos de evento válidos
// ─────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TiposEventoResp {
    pub tipos: Vec<&'static str>,
    pub aviso: String,
}

/// GET /fiscal/homologacao/historico/tipos
/// Lista os tipos de evento técnico permitidos na tabela de histórico.
pub async fn listar_tipos_evento(
    State(_state): State<AppState>,
) -> Json<TiposEventoResp> {
    Json(TiposEventoResp {
        tipos: TIPOS_EVENTO_VALIDOS.to_vec(),
        aviso: "Estes são os tipos de evento técnico de homologação registráveis. Nenhum representa autorização fiscal real.".into(),
    })
}
