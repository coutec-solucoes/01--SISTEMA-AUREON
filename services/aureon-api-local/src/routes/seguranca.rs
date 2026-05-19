use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;

use crate::{app::AppState, erros::ErroApi, crypto::hash_senha, middleware::UsuarioLogado};
use aureon_core::RespostaBase;

// ================================================================
// Verificador de Permissões
// ================================================================
pub async fn tem_permissao(pool: &PgPool, usuario: &UsuarioLogado, menu: &str, acao: &str) -> Result<bool, ErroApi> {
    if usuario.is_admin {
        return Ok(true);
    }
    
    let result = sqlx::query(
        "SELECT permitido FROM permissoes WHERE perfil_id = $1 AND menu = $2 AND acao = $3"
    )
    .bind(usuario.perfil_id)
    .bind(menu)
    .bind(acao)
    .fetch_optional(pool)
    .await
    .map_err(|e| ErroApi::interno(e.to_string()))?;

    if let Some(row) = result {
        let permitido: bool = row.get("permitido");
        Ok(permitido)
    } else {
        Ok(false)
    }
}

// ================================================================
// DTOs
// ================================================================

#[derive(Serialize)]
pub struct UsuarioListaDto {
    pub id: Uuid,
    pub nome: String,
    pub login: String,
    pub email: Option<String>,
    pub status: String,
    pub bloqueado: bool,
    pub is_admin: bool,
    pub acessa_retaguarda: bool,
    pub acessa_pdv: bool,
    pub perfil_id: Uuid,
    pub perfil_nome: String,
    pub ultimo_login: Option<chrono::DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct UsuarioCreateDto {
    pub nome: String,
    pub login: String,
    pub senha_plana: String,
    pub email: Option<String>,
    pub status: String,
    pub is_admin: bool,
    pub acessa_retaguarda: bool,
    pub acessa_pdv: bool,
    pub perfil_id: Uuid,
}

#[derive(Deserialize)]
pub struct UsuarioUpdateDto {
    pub nome: String,
    pub login: String,
    pub email: Option<String>,
    pub status: String,
    pub is_admin: bool,
    pub bloqueado: bool,
    pub acessa_retaguarda: bool,
    pub acessa_pdv: bool,
    pub perfil_id: Uuid,
}

#[derive(Deserialize)]
pub struct ResetSenhaDto {
    pub nova_senha: String,
}

// ================================================================
// Handlers de Usuários
// ================================================================

pub async fn listar_usuarios(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "SEGURANCA_USUARIOS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query(
        "SELECT u.id, u.nome, u.login, u.email, u.status, u.bloqueado, u.is_admin, u.acessa_retaguarda, u.acessa_pdv, u.ultimo_login, p.id as perfil_id, p.nome as perfil_nome 
         FROM usuarios u 
         JOIN usuarios_perfis up ON up.usuario_id = u.id 
         JOIN perfis p ON p.id = up.perfil_id 
         ORDER BY u.nome"
    )
    .fetch_all(pool)
    .await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let mut usuarios = Vec::new();
    for row in records {
        usuarios.push(UsuarioListaDto {
            id: row.get("id"),
            nome: row.get("nome"),
            login: row.get("login"),
            email: row.get("email"),
            status: row.get("status"),
            bloqueado: row.get("bloqueado"),
            is_admin: row.get("is_admin"),
            acessa_retaguarda: row.get("acessa_retaguarda"),
            acessa_pdv: row.get("acessa_pdv"),
            perfil_id: row.get("perfil_id"),
            perfil_nome: row.get("perfil_nome"),
            ultimo_login: row.get("ultimo_login"),
        });
    }

    (StatusCode::OK, Json(RespostaBase::ok("Usuários obtidos com sucesso", usuarios))).into_response()
}

pub async fn criar_usuario(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<UsuarioCreateDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "SEGURANCA_USUARIOS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };
    
    if dados.senha_plana.len() < 8 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Senha muito fraca.", "ERRO_SENHA_FRACA", "A senha deve ter pelo menos 8 caracteres."))).into_response();
    }

    // Valida unicidade de login
    let login_exists = sqlx::query("SELECT id FROM usuarios WHERE login = $1").bind(&dados.login).fetch_optional(pool).await;
    if let Ok(Some(_)) = login_exists {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Login já existe.", "ERRO_LOGIN_DUPLICADO", "Este login já está em uso."))).into_response();
    }

    let senha_hash = match hash_senha(&dados.senha_plana) {
        Ok(h) => h,
        Err(e) => return e.into_response(),
    };

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let user_id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO usuarios (id, login, nome, senha_hash, email, status, bloqueado, is_admin, acessa_retaguarda, acessa_pdv) 
         VALUES ($1, $2, $3, $4, $5, $6, false, $7, $8, $9)"
    )
    .bind(user_id).bind(&dados.login).bind(&dados.nome).bind(&senha_hash)
    .bind(&dados.email).bind(&dados.status).bind(dados.is_admin)
    .bind(dados.acessa_retaguarda).bind(dados.acessa_pdv)
    .execute(&mut *tx).await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    if let Err(e) = sqlx::query("INSERT INTO usuarios_perfis (usuario_id, perfil_id) VALUES ($1, $2)")
        .bind(user_id).bind(dados.perfil_id)
        .execute(&mut *tx).await {
        return ErroApi::interno(e.to_string()).into_response();
    }
    
    // Log
    let log_msg = format!("Usuário '{}' (Login: {}) criado", dados.nome, dados.login);
    let _ = sqlx::query("INSERT INTO logs_seguranca (tipo_evento, mensagem, severidade, usuario_id) VALUES ('USUARIO_CRIADO', $1, 'Info', $2)")
        .bind(log_msg).bind(usuario.usuario_id)
        .execute(&mut *tx).await;

    if let Err(e) = tx.commit().await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    (StatusCode::CREATED, Json(RespostaBase::ok("Usuário criado com sucesso", json!({"id": user_id})))).into_response()
}

pub async fn atualizar_usuario(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<UsuarioUpdateDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "SEGURANCA_USUARIOS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    // Valida unicidade de login (ignorando o proprio ID)
    let login_exists = sqlx::query("SELECT id FROM usuarios WHERE login = $1 AND id != $2").bind(&dados.login).bind(id).fetch_optional(pool).await;
    if let Ok(Some(_)) = login_exists {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Login já existe.", "ERRO_LOGIN_DUPLICADO", "Este login já está em uso."))).into_response();
    }

    // Regra de segurança: O último administrador não pode perder o privilégio de admin, ser inativado ou bloqueado
    if !dados.is_admin || dados.status != "ATIVO" || dados.bloqueado {
        let admins = sqlx::query("SELECT count(*) as total FROM usuarios WHERE is_admin = true AND id != $1 AND status = 'ATIVO' AND bloqueado = false")
            .bind(id)
            .fetch_one(pool).await;
            
        if let Ok(r) = admins {
            let total: i64 = r.get("total");
            if total == 0 {
                // Descobrindo se o usuário alvo era admin antes
                if let Ok(Some(u)) = sqlx::query("SELECT is_admin FROM usuarios WHERE id = $1").bind(id).fetch_optional(pool).await {
                    let era_admin: bool = u.get("is_admin");
                    if era_admin {
                        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Proteção de Sistema.", "ERRO_ULTIMO_ADMIN", "O último administrador ativo não pode perder os privilégios ou ser inativado/bloqueado."))).into_response();
                    }
                }
            }
        }
    }

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    if let Err(e) = sqlx::query(
        "UPDATE usuarios SET login = $1, nome = $2, email = $3, status = $4, is_admin = $5, bloqueado = $6, acessa_retaguarda = $7, acessa_pdv = $8, atualizado_em = NOW() WHERE id = $9"
    )
    .bind(&dados.login).bind(&dados.nome).bind(&dados.email).bind(&dados.status)
    .bind(dados.is_admin).bind(dados.bloqueado).bind(dados.acessa_retaguarda).bind(dados.acessa_pdv)
    .bind(id)
    .execute(&mut *tx).await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    if let Err(e) = sqlx::query("UPDATE usuarios_perfis SET perfil_id = $1 WHERE usuario_id = $2")
        .bind(dados.perfil_id).bind(id)
        .execute(&mut *tx).await {
        return ErroApi::interno(e.to_string()).into_response();
    }
    
    // Log
    let log_msg = format!("Usuário (ID: {}) atualizado", id);
    let _ = sqlx::query("INSERT INTO logs_seguranca (tipo_evento, mensagem, severidade, usuario_id) VALUES ('USUARIO_ATUALIZADO', $1, 'Info', $2)")
        .bind(log_msg).bind(usuario.usuario_id)
        .execute(&mut *tx).await;

    if let Err(e) = tx.commit().await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    (StatusCode::OK, Json(RespostaBase::ok("Usuário atualizado com sucesso", ()))).into_response()
}

pub async fn redefinir_senha(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ResetSenhaDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "SEGURANCA_USUARIOS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };
    
    if dados.nova_senha.len() < 8 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Senha muito fraca.", "ERRO_SENHA_FRACA", "A senha deve ter pelo menos 8 caracteres."))).into_response();
    }

    let senha_hash = match hash_senha(&dados.nova_senha) {
        Ok(h) => h,
        Err(e) => return e.into_response(),
    };
    
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    // Atualiza a senha e força revogação das sessoes para que o usuário logue novamente
    if let Err(e) = sqlx::query("UPDATE usuarios SET senha_hash = $1, atualizado_em = NOW() WHERE id = $2")
        .bind(&senha_hash)
        .bind(id)
        .execute(&mut *tx).await {
        return ErroApi::interno(e.to_string()).into_response();
    }
    
    // Revoga sessoes ativas do usuário alvo
    let _ = sqlx::query("UPDATE sessoes_usuarios SET revogado_em = NOW() WHERE usuario_id = $1 AND revogado_em IS NULL")
        .bind(id)
        .execute(&mut *tx).await;
        
    // Log
    let log_msg = format!("Senha do usuário (ID: {}) redefinida por admin", id);
    let _ = sqlx::query("INSERT INTO logs_seguranca (tipo_evento, mensagem, severidade, usuario_id) VALUES ('SENHA_REDEFINIDA', $1, 'Warning', $2)")
        .bind(log_msg).bind(usuario.usuario_id)
        .execute(&mut *tx).await;

    if let Err(e) = tx.commit().await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    (StatusCode::OK, Json(RespostaBase::ok("Senha redefinida com sucesso", ()))).into_response()
}

// ================================================================
// Handlers de Perfis
// ================================================================

#[derive(Serialize)]
pub struct PerfilDto {
    pub id: Uuid,
    pub nome: String,
    pub descricao: Option<String>,
    pub criado_em: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct PerfilInputDto {
    pub nome: String,
    pub descricao: Option<String>,
}

pub async fn listar_perfis(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "SEGURANCA_PERFIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query("SELECT id, nome, descricao, criado_em FROM perfis ORDER BY nome")
        .fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let perfis: Vec<PerfilDto> = records.into_iter().map(|row| PerfilDto {
        id: row.get("id"),
        nome: row.get("nome"),
        descricao: row.get("descricao"),
        criado_em: row.get("criado_em"),
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok("Perfis obtidos com sucesso", perfis))).into_response()
}

pub async fn criar_perfil(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<PerfilInputDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "SEGURANCA_PERFIS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let id = Uuid::new_v4();
    if let Err(e) = sqlx::query("INSERT INTO perfis (id, nome, descricao) VALUES ($1, $2, $3)")
        .bind(id).bind(&dados.nome).bind(&dados.descricao)
        .execute(pool).await {
        return ErroApi::interno(e.to_string()).into_response();
    }
    
    // Log
    let log_msg = format!("Perfil '{}' criado", dados.nome);
    let _ = sqlx::query("INSERT INTO logs_seguranca (tipo_evento, mensagem, severidade, usuario_id) VALUES ('PERFIL_CRIADO', $1, 'Info', $2)")
        .bind(log_msg).bind(usuario.usuario_id)
        .execute(pool).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Perfil criado com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_perfil(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<PerfilInputDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "SEGURANCA_PERFIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };
    
    // Regra: Perfil ADMINISTRADOR não deve ter o nome alterado, é reservado do sistema.
    if let Ok(Some(row)) = sqlx::query("SELECT nome FROM perfis WHERE id = $1").bind(id).fetch_optional(pool).await {
        let nome_atual: String = row.get("nome");
        if nome_atual == "ADMINISTRADOR" && dados.nome != "ADMINISTRADOR" {
            return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_PERFIL_RESERVADO", "O perfil ADMINISTRADOR não pode ser renomeado."))).into_response();
        }
    }

    if let Err(e) = sqlx::query("UPDATE perfis SET nome = $1, descricao = $2, atualizado_em = NOW() WHERE id = $3")
        .bind(&dados.nome).bind(&dados.descricao).bind(id)
        .execute(pool).await {
        return ErroApi::interno(e.to_string()).into_response();
    }
    
    // Log
    let log_msg = format!("Perfil '{}' (ID: {}) atualizado", dados.nome, id);
    let _ = sqlx::query("INSERT INTO logs_seguranca (tipo_evento, mensagem, severidade, usuario_id) VALUES ('PERFIL_ATUALIZADO', $1, 'Info', $2)")
        .bind(log_msg).bind(usuario.usuario_id)
        .execute(pool).await;

    (StatusCode::OK, Json(RespostaBase::ok("Perfil atualizado com sucesso", ()))).into_response()
}

// ================================================================
// Handlers de Permissões
// ================================================================

#[derive(Serialize, Deserialize)]
pub struct PermissaoItemDto {
    pub menu: String,
    pub acao: String,
    pub permitido: bool,
}

#[derive(Deserialize)]
pub struct SalvarPermissoesDto {
    pub permissoes: Vec<PermissaoItemDto>,
}

pub async fn obter_permissoes(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "SEGURANCA_PERFIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query("SELECT menu, acao, permitido FROM permissoes WHERE perfil_id = $1 ORDER BY menu, acao")
        .bind(id)
        .fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let permissoes: Vec<PermissaoItemDto> = records.into_iter().map(|row| PermissaoItemDto {
        menu: row.get("menu"),
        acao: row.get("acao"),
        permitido: row.get("permitido"),
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok("Permissões obtidas com sucesso", permissoes))).into_response()
}

pub async fn salvar_permissoes(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<SalvarPermissoesDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "SEGURANCA_PERFIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };
    
    // Regra: Não permitir retirar permissões totais do perfil ADMINISTRADOR.
    if let Ok(Some(row)) = sqlx::query("SELECT nome FROM perfis WHERE id = $1").bind(id).fetch_optional(pool).await {
        let nome_atual: String = row.get("nome");
        if nome_atual == "ADMINISTRADOR" {
            // Verifica se está tentando colocar permitido=false em algo
            for p in &dados.permissoes {
                if !p.permitido {
                    return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_PERFIL_RESERVADO", "O perfil ADMINISTRADOR não pode ter restrições de permissões."))).into_response();
                }
            }
        }
    }

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    // Deleta permissões antigas
    if let Err(e) = sqlx::query("DELETE FROM permissoes WHERE perfil_id = $1").bind(id).execute(&mut *tx).await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    // Insere novas permissões
    for p in dados.permissoes {
        if let Err(e) = sqlx::query("INSERT INTO permissoes (perfil_id, menu, acao, permitido) VALUES ($1, $2, $3, $4)")
            .bind(id).bind(p.menu).bind(p.acao).bind(p.permitido)
            .execute(&mut *tx).await {
            return ErroApi::interno(e.to_string()).into_response();
        }
    }
    
    // Log
    let log_msg = format!("Permissões do perfil ID: {} atualizadas", id);
    let _ = sqlx::query("INSERT INTO logs_seguranca (tipo_evento, mensagem, severidade, usuario_id) VALUES ('PERMISSOES_ATUALIZADAS', $1, 'Warning', $2)")
        .bind(log_msg).bind(usuario.usuario_id)
        .execute(&mut *tx).await;

    if let Err(e) = tx.commit().await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    (StatusCode::OK, Json(RespostaBase::ok("Permissões atualizadas com sucesso", ()))).into_response()
}
