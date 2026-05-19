use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use sqlx::Row;
use uuid::Uuid;
use chrono::Utc;

use crate::{app::AppState, erros::ErroApi};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UsuarioLogado {
    pub usuario_id: Uuid,
    pub perfil_id: Uuid,
    pub is_admin: bool,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ErroApi> {
    let pool = match &state.pool {
        Some(p) => p,
        None => return Err(ErroApi::indisponivel("Banco de dados não configurado.")),
    };

    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => {
            return Err(ErroApi {
                status: StatusCode::UNAUTHORIZED,
                codigo: "ERRO_NAO_AUTORIZADO".to_string(),
                detalhe: "Token de autenticação não fornecido.".to_string(),
            });
        }
    };

    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Buscar sessão válida e obter usuário ativo
    let record = sqlx::query(
        r#"
        SELECT 
            s.id as sessao_id, 
            s.expira_em, 
            s.revogado_em,
            u.id as usuario_id, 
            u.status, 
            u.bloqueado, 
            u.is_admin,
            up.perfil_id
        FROM sessoes_usuarios s
        JOIN usuarios u ON u.id = s.usuario_id
        JOIN usuarios_perfis up ON up.usuario_id = u.id
        WHERE s.token_hash = $1
        "#
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .map_err(|e| ErroApi::interno(format!("Erro ao buscar sessão: {}", e)))?;

    let row = match record {
        Some(r) => r,
        None => {
            return Err(ErroApi {
                status: StatusCode::UNAUTHORIZED,
                codigo: "ERRO_SESSAO_INVALIDA".to_string(),
                detalhe: "Sessão inválida ou não encontrada.".to_string(),
            });
        }
    };

    let revogado_em: Option<chrono::DateTime<Utc>> = row.get("revogado_em");
    if revogado_em.is_some() {
        return Err(ErroApi {
            status: StatusCode::UNAUTHORIZED,
            codigo: "ERRO_SESSAO_REVOGADA".to_string(),
            detalhe: "Sessão revogada.".to_string(),
        });
    }

    let expira_em: chrono::DateTime<Utc> = row.get("expira_em");
    if expira_em < Utc::now() {
        return Err(ErroApi {
            status: StatusCode::UNAUTHORIZED,
            codigo: "ERRO_SESSAO_EXPIRADA".to_string(),
            detalhe: "Sessão expirada. Faça login novamente.".to_string(),
        });
    }

    let bloqueado: bool = row.get("bloqueado");
    if bloqueado {
        return Err(ErroApi {
            status: StatusCode::FORBIDDEN,
            codigo: "ERRO_USUARIO_BLOQUEADO".to_string(),
            detalhe: "Usuário bloqueado. Acesso negado.".to_string(),
        });
    }

    let status: String = row.get("status");
    if status != "ATIVO" {
        return Err(ErroApi {
            status: StatusCode::FORBIDDEN,
            codigo: "ERRO_USUARIO_INATIVO".to_string(),
            detalhe: "Usuário inativo. Acesso negado.".to_string(),
        });
    }

    let sessao_id: Uuid = row.get("sessao_id");

    // Renovar sessão (adiciona mais 1 hora de inatividade)
    sqlx::query("UPDATE sessoes_usuarios SET ultimo_acesso_em = NOW(), expira_em = NOW() + INTERVAL '1 hour' WHERE id = $1")
        .bind(sessao_id)
        .execute(pool)
        .await
        .map_err(|e| ErroApi::interno(format!("Erro ao renovar sessão: {}", e)))?;

    let usuario_logado = UsuarioLogado {
        usuario_id: row.get("usuario_id"),
        perfil_id: row.get("perfil_id"),
        is_admin: row.get("is_admin"),
    };

    req.extensions_mut().insert(usuario_logado);

    Ok(next.run(req).await)
}
