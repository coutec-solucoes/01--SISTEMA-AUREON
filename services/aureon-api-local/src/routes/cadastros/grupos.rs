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
use chrono::Utc;

use crate::{app::AppState, erros::ErroApi, middleware::UsuarioLogado};
use aureon_core::RespostaBase;
use crate::routes::seguranca::tem_permissao;
use super::pessoas::{auditar, publicar_evento};

// ================================================================
// DTOs de Grupos
// ================================================================

#[derive(Serialize)]
pub struct GrupoDto {
    pub id: Uuid,
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: bool,
    pub ordem: i32,
    pub criado_em: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct GrupoInputDto {
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: Option<bool>,
    pub ordem: Option<i32>,
}

// ================================================================
// DTOs de Subgrupos
// ================================================================

#[derive(Serialize)]
pub struct SubgrupoDto {
    pub id: Uuid,
    pub grupo_id: Uuid,
    pub grupo_nome: String,
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: bool,
    pub ordem: i32,
    pub criado_em: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct SubgrupoInputDto {
    pub grupo_id: Uuid,
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: Option<bool>,
    pub ordem: Option<i32>,
}

// ================================================================
// DTOs de Marcas
// ================================================================

#[derive(Serialize)]
pub struct MarcaDto {
    pub id: Uuid,
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: bool,
    pub criado_em: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct MarcaInputDto {
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: Option<bool>,
}

// ================================================================
// Handlers de Grupos
// ================================================================

pub async fn listar_grupos(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PRODUTOS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query(
        "SELECT id, nome, descricao, ativo, ordem, criado_em FROM produtos_grupos ORDER BY ordem, nome"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let lista: Vec<GrupoDto> = records.into_iter().map(|row| GrupoDto {
        id: row.get("id"),
        nome: row.get("nome"),
        descricao: row.get("descricao"),
        ativo: row.get("ativo"),
        ordem: row.get("ordem"),
        criado_em: row.get("criado_em"),
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok("Grupos obtidos com sucesso", lista))).into_response()
}

pub async fn criar_grupo(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<GrupoInputDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PRODUTOS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.nome.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do grupo é obrigatório.", "ERRO_GRUPO_NOME_OBRIGATORIO", ""))).into_response();
    }

    let id = Uuid::new_v4();
    let ativo = dados.ativo.unwrap_or(true);
    let ordem = dados.ordem.unwrap_or(0);

    if let Err(e) = sqlx::query(
        "INSERT INTO produtos_grupos (id, nome, descricao, ativo, ordem) VALUES ($1, $2, $3, $4, $5)"
    ).bind(id).bind(&dados.nome).bind(&dados.descricao).bind(ativo).bind(ordem)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("uq_produtos_grupos_nome") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Grupo com este nome já existe.", "ERRO_GRUPO_DUPLICADO", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    auditar(pool, "GRUPO", Some(id), "CRIAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "GRUPO_PRODUTO_CRIADO", "GRUPO", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Grupo criado com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_grupo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<GrupoInputDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PRODUTOS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.nome.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do grupo é obrigatório.", "ERRO_GRUPO_NOME_OBRIGATORIO", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "UPDATE produtos_grupos SET nome = $1, descricao = $2, ativo = $3, ordem = $4, atualizado_em = NOW() WHERE id = $5"
    ).bind(&dados.nome).bind(&dados.descricao).bind(dados.ativo.unwrap_or(true)).bind(dados.ordem.unwrap_or(0)).bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("uq_produtos_grupos_nome") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Grupo com este nome já existe.", "ERRO_GRUPO_DUPLICADO", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    auditar(pool, "GRUPO", Some(id), "EDITAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "GRUPO_PRODUTO_ALTERADO", "GRUPO", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Grupo atualizado com sucesso", ()))).into_response()
}

// ================================================================
// Handlers de Subgrupos
// ================================================================

pub async fn listar_subgrupos(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PRODUTOS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query(
        "SELECT s.id, s.grupo_id, g.nome as grupo_nome, s.nome, s.descricao, s.ativo, s.ordem, s.criado_em
         FROM produtos_subgrupos s JOIN produtos_grupos g ON g.id = s.grupo_id ORDER BY g.nome, s.nome"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let lista: Vec<SubgrupoDto> = records.into_iter().map(|row| SubgrupoDto {
        id: row.get("id"),
        grupo_id: row.get("grupo_id"),
        grupo_nome: row.get("grupo_nome"),
        nome: row.get("nome"),
        descricao: row.get("descricao"),
        ativo: row.get("ativo"),
        ordem: row.get("ordem"),
        criado_em: row.get("criado_em"),
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok("Subgrupos obtidos com sucesso", lista))).into_response()
}

pub async fn criar_subgrupo(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<SubgrupoInputDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PRODUTOS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.nome.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do subgrupo é obrigatório.", "ERRO_SUBGRUPO_NOME_OBRIGATORIO", ""))).into_response();
    }

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO produtos_subgrupos (id, grupo_id, nome, descricao, ativo, ordem) VALUES ($1, $2, $3, $4, $5, $6)"
    ).bind(id).bind(dados.grupo_id).bind(&dados.nome).bind(&dados.descricao)
    .bind(dados.ativo.unwrap_or(true)).bind(dados.ordem.unwrap_or(0))
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("uq_produtos_subgrupos_nome_grupo") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Subgrupo com este nome já existe no grupo.", "ERRO_SUBGRUPO_DUPLICADO", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    auditar(pool, "SUBGRUPO", Some(id), "CRIAR", None, None, Some(json!({"nome": &dados.nome, "grupo_id": dados.grupo_id})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "SUBGRUPO_PRODUTO_CRIADO", "SUBGRUPO", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Subgrupo criado com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_subgrupo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<SubgrupoInputDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PRODUTOS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.nome.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do subgrupo é obrigatório.", "ERRO_SUBGRUPO_NOME_OBRIGATORIO", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "UPDATE produtos_subgrupos SET grupo_id = $1, nome = $2, descricao = $3, ativo = $4, ordem = $5, atualizado_em = NOW() WHERE id = $6"
    ).bind(dados.grupo_id).bind(&dados.nome).bind(&dados.descricao)
    .bind(dados.ativo.unwrap_or(true)).bind(dados.ordem.unwrap_or(0)).bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("uq_produtos_subgrupos_nome_grupo") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Subgrupo com este nome já existe no grupo.", "ERRO_SUBGRUPO_DUPLICADO", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    auditar(pool, "SUBGRUPO", Some(id), "EDITAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "SUBGRUPO_PRODUTO_ALTERADO", "SUBGRUPO", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Subgrupo atualizado com sucesso", ()))).into_response()
}

// ================================================================
// Handlers de Marcas
// ================================================================

pub async fn listar_marcas(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PRODUTOS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query(
        "SELECT id, nome, descricao, ativo, criado_em FROM produtos_marcas ORDER BY nome"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let lista: Vec<MarcaDto> = records.into_iter().map(|row| MarcaDto {
        id: row.get("id"),
        nome: row.get("nome"),
        descricao: row.get("descricao"),
        ativo: row.get("ativo"),
        criado_em: row.get("criado_em"),
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok("Marcas obtidas com sucesso", lista))).into_response()
}

pub async fn criar_marca(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<MarcaInputDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PRODUTOS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.nome.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome da marca é obrigatório.", "ERRO_MARCA_NOME_OBRIGATORIO", ""))).into_response();
    }

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO produtos_marcas (id, nome, descricao, ativo) VALUES ($1, $2, $3, $4)"
    ).bind(id).bind(&dados.nome).bind(&dados.descricao).bind(dados.ativo.unwrap_or(true))
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("uq_produtos_marcas_nome") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Marca com este nome já existe.", "ERRO_MARCA_DUPLICADA", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    auditar(pool, "MARCA", Some(id), "CRIAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "MARCA_CRIADA", "MARCA", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Marca criada com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_marca(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<MarcaInputDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PRODUTOS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.nome.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome da marca é obrigatório.", "ERRO_MARCA_NOME_OBRIGATORIO", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "UPDATE produtos_marcas SET nome = $1, descricao = $2, ativo = $3, atualizado_em = NOW() WHERE id = $4"
    ).bind(&dados.nome).bind(&dados.descricao).bind(dados.ativo.unwrap_or(true)).bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("uq_produtos_marcas_nome") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Marca com este nome já existe.", "ERRO_MARCA_DUPLICADA", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    auditar(pool, "MARCA", Some(id), "EDITAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "MARCA_ALTERADA", "MARCA", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Marca atualizada com sucesso", ()))).into_response()
}
