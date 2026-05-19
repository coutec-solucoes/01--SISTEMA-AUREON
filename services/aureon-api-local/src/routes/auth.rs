use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;
use sha2::{Sha256, Digest};

use crate::{app::AppState, erros::ErroApi, crypto::{hash_senha, verificar_senha}, middleware::UsuarioLogado};
use aureon_core::RespostaBase;

#[derive(Deserialize)]
pub struct LoginDto {
    pub login: String,
    pub senha: String,
}

#[derive(Serialize)]
pub struct SessaoDto {
    pub token: String,
    pub usuario: UsuarioBaseDto,
}

#[derive(Serialize)]
pub struct UsuarioBaseDto {
    pub id: Uuid,
    pub nome: String,
    pub login: String,
    pub is_admin: bool,
    pub perfil_id: Uuid,
    pub perfil_nome: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(dados): Json<LoginDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco de dados não configurado.").into_response(),
    };

    if dados.login.trim().is_empty() || dados.senha.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Credenciais inválidas.", "ERRO_LOGIN_INVALIDO", "Login e senha são obrigatórios."))).into_response();
    }

    // Buscar usuário
    let record = match sqlx::query(
        "SELECT u.id, u.nome, u.senha_hash, u.status, u.bloqueado, u.is_admin, p.id as perfil_id, p.nome as perfil_nome 
         FROM usuarios u 
         JOIN usuarios_perfis up ON up.usuario_id = u.id 
         JOIN perfis p ON p.id = up.perfil_id 
         WHERE u.login = $1"
    )
    .bind(&dados.login)
    .fetch_optional(pool)
    .await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(format!("Erro ao buscar usuário: {}", e)).into_response(),
    };

    let row = match record {
        Some(r) => r,
        None => return (StatusCode::UNAUTHORIZED, Json(RespostaBase::<()>::falha_manual("Usuário ou senha inválidos.", "ERRO_LOGIN_INVALIDO", ""))).into_response(),
    };

    let status: String = row.get("status");
    if status != "ATIVO" {
        return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Usuário inativo. Acesso negado.", "ERRO_USUARIO_INATIVO", ""))).into_response();
    }

    let bloqueado: bool = row.get("bloqueado");
    if bloqueado {
        return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Usuário bloqueado. Acesso negado.", "ERRO_USUARIO_BLOQUEADO", ""))).into_response();
    }

    let senha_hash: String = row.get("senha_hash");
    let senha_ok = match verificar_senha(&dados.senha, &senha_hash) {
        Ok(v) => v,
        Err(e) => return e.into_response(),
    };

    if !senha_ok {
        // Registrar log de falha de login (ignorado erro de inserção aqui)
        let _ = sqlx::query("INSERT INTO logs_seguranca (tipo_evento, mensagem, severidade, usuario_id) VALUES ('LOGIN_FALHO', 'Falha de login (senha incorreta)', 'Warning', $1)")
            .bind(row.get::<Uuid, _>("id"))
            .execute(pool).await;

        return (StatusCode::UNAUTHORIZED, Json(RespostaBase::<()>::falha_manual("Usuário ou senha inválidos.", "ERRO_LOGIN_INVALIDO", ""))).into_response();
    }

    // Criar nova sessão
    let token = Uuid::new_v4().to_string();
    
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    if let Err(e) = sqlx::query("INSERT INTO sessoes_usuarios (usuario_id, token_hash, ip_dispositivo) VALUES ($1, $2, 'Local')")
        .bind(row.get::<Uuid, _>("id"))
        .bind(&token_hash)
        .execute(pool)
        .await
    {
        return ErroApi::interno(format!("Erro ao criar sessão: {}", e)).into_response();
    }

    // Atualizar ultimo login
    sqlx::query("UPDATE usuarios SET ultimo_login = NOW() WHERE id = $1")
        .bind(row.get::<Uuid, _>("id"))
        .execute(pool)
        .await
        .ok();

    // Log de sucesso
    sqlx::query("INSERT INTO logs_seguranca (tipo_evento, mensagem, severidade, usuario_id) VALUES ('LOGIN_SUCESSO', 'Login realizado com sucesso', 'Info', $1)")
        .bind(row.get::<Uuid, _>("id"))
        .execute(pool).await.ok();

    let sessao = SessaoDto {
        token,
        usuario: UsuarioBaseDto {
            id: row.get("id"),
            nome: row.get("nome"),
            login: row.get("login"),
            is_admin: row.get("is_admin"),
            perfil_id: row.get("perfil_id"),
            perfil_nome: row.get("perfil_nome"),
        }
    };

    (StatusCode::OK, Json(RespostaBase::ok("Login realizado com sucesso.", sessao))).into_response()
}

pub async fn logout(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco de dados não configurado.").into_response(),
    };

    let auth_header = req.headers().get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    if let Some(token) = auth_header {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

        sqlx::query("UPDATE sessoes_usuarios SET revogado_em = NOW() WHERE token_hash = $1")
            .bind(&token_hash)
            .execute(pool)
            .await
            .ok();

        // Log
        sqlx::query("INSERT INTO logs_seguranca (tipo_evento, mensagem, severidade, usuario_id) VALUES ('LOGOUT', 'Logout realizado com sucesso', 'Info', $1)")
            .bind(usuario.usuario_id)
            .execute(pool).await.ok();
    }

    (StatusCode::OK, Json(RespostaBase::ok("Logout realizado com sucesso.", ()))).into_response()
}

#[derive(Serialize)]
pub struct PermissaoDto {
    pub menu: String,
    pub acoes: Vec<String>,
}

#[derive(Serialize)]
pub struct AuthMeDto {
    pub usuario: UsuarioBaseDto,
    pub permissoes: Vec<PermissaoDto>,
}

pub async fn me(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco de dados não configurado.").into_response(),
    };

    let user_row = match sqlx::query(
        "SELECT u.id, u.nome, u.login, u.is_admin, p.id as perfil_id, p.nome as perfil_nome 
         FROM usuarios u 
         JOIN usuarios_perfis up ON up.usuario_id = u.id 
         JOIN perfis p ON p.id = up.perfil_id 
         WHERE u.id = $1"
    )
    .bind(usuario.usuario_id)
    .fetch_one(pool)
    .await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(format!("Erro ao carregar dados do usuário: {}", e)).into_response(),
    };

    let perm_rows = match sqlx::query("SELECT menu, acao FROM permissoes WHERE perfil_id = $1 AND permitido = true")
        .bind(usuario.perfil_id)
        .fetch_all(pool)
        .await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(format!("Erro ao buscar permissões: {}", e)).into_response(),
    };

    let mut permissoes_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for row in perm_rows {
        let menu: String = row.get("menu");
        let acao: String = row.get("acao");
        permissoes_map.entry(menu).or_default().push(acao);
    }

    let permissoes: Vec<PermissaoDto> = permissoes_map.into_iter().map(|(menu, acoes)| PermissaoDto { menu, acoes }).collect();

    let dados = AuthMeDto {
        usuario: UsuarioBaseDto {
            id: user_row.get("id"),
            nome: user_row.get("nome"),
            login: user_row.get("login"),
            is_admin: user_row.get("is_admin"),
            perfil_id: user_row.get("perfil_id"),
            perfil_nome: user_row.get("perfil_nome"),
        },
        permissoes,
    };

    (StatusCode::OK, Json(RespostaBase::ok("Dados do usuário logado", dados))).into_response()
}

pub async fn setup_admin(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco de dados não configurado.").into_response(),
    };

    // Verificar se já foi rodado
    let setup_status = match sqlx::query("SELECT valor FROM configuracao_setup WHERE chave = 'admin_inicial_criado'")
        .fetch_optional(pool)
        .await {
        Ok(s) => s,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    if let Some(row) = setup_status {
        let valor: String = row.get("valor");
        if valor == "true" {
            return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Setup já foi concluído.", "ERRO_SETUP_CONCLUIDO", "O administrador inicial já foi criado."))).into_response();
        }
    } else {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Banco de dados não preparado.", "ERRO_SETUP_FALHO", "A chave de controle não existe."))).into_response();
    }

    let senha_hash = match hash_senha("Aureon@2026") {
        Ok(h) => h,
        Err(e) => return e.into_response(),
    };
    
    // Obter perfil de admin
    let perfil = match sqlx::query("SELECT id FROM perfis WHERE nome = 'ADMINISTRADOR'")
        .fetch_one(pool)
        .await {
        Ok(p) => p,
        Err(e) => return ErroApi::interno(format!("Perfil ADMINISTRADOR não encontrado: {}", e)).into_response(),
    };
        
    let perfil_id: Uuid = perfil.get("id");

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let user_id = Uuid::new_v4();

    if let Err(e) = sqlx::query("INSERT INTO usuarios (id, login, nome, senha_hash, email, status, bloqueado, is_admin, acessa_retaguarda, acessa_pdv) VALUES ($1, $2, $3, $4, $5, 'ATIVO', false, true, true, false)")
        .bind(user_id)
        .bind("admin")
        .bind("Administrador do Sistema")
        .bind(&senha_hash)
        .bind("admin@aureon.local")
        .execute(&mut *tx)
        .await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    if let Err(e) = sqlx::query("INSERT INTO usuarios_perfis (usuario_id, perfil_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(perfil_id)
        .execute(&mut *tx)
        .await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    if let Err(e) = sqlx::query("UPDATE configuracao_setup SET valor = 'true' WHERE chave = 'admin_inicial_criado'")
        .execute(&mut *tx)
        .await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    if let Err(e) = tx.commit().await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    (StatusCode::OK, Json(RespostaBase::ok("Administrador inicial criado com sucesso. Login: admin / Senha: Aureon@2026", ()))).into_response()
}
