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

// ================================================================
// Utilitário: Auditar e publicar evento de cadastro
// ================================================================

pub async fn auditar(
    pool: &sqlx::PgPool,
    entidade: &str,
    entidade_id: Option<Uuid>,
    acao: &str,
    campo: Option<&str>,
    anterior: Option<serde_json::Value>,
    novo: Option<serde_json::Value>,
    usuario_id: Option<Uuid>,
) {
    let _ = sqlx::query(
        "INSERT INTO auditoria_cadastros (entidade, entidade_id, acao, campo_alterado, valor_anterior, valor_novo, usuario_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(entidade)
    .bind(entidade_id)
    .bind(acao)
    .bind(campo)
    .bind(anterior)
    .bind(novo)
    .bind(usuario_id)
    .execute(pool)
    .await;
}

pub async fn publicar_evento(
    pool: &sqlx::PgPool,
    tipo_evento: &str,
    entidade: &str,
    entidade_id: Option<Uuid>,
    payload: serde_json::Value,
) {
    let _ = sqlx::query(
        "INSERT INTO eventos_publicacao (tipo_evento, entidade, entidade_id, payload)
         VALUES ($1, $2, $3, $4)"
    )
    .bind(tipo_evento)
    .bind(entidade)
    .bind(entidade_id)
    .bind(payload)
    .execute(pool)
    .await;
}

// ================================================================
// DTOs de Pessoas
// ================================================================

#[derive(Serialize)]
pub struct PessoaListaDto {
    pub id: Uuid,
    pub tipo_pessoa: String,
    pub nome_razao_social: String,
    pub nome_fantasia: Option<String>,
    pub cpf: Option<String>,
    pub cnpj: Option<String>,
    pub ci: Option<String>,
    pub ruc: Option<String>,
    pub ativo: bool,
    pub papeis: Vec<String>,
    pub criado_em: chrono::DateTime<Utc>,
}

#[derive(Serialize)]
pub struct PessoaDetalheDto {
    pub id: Uuid,
    pub tipo_pessoa: String,
    pub nome_razao_social: String,
    pub nome_fantasia: Option<String>,
    pub cpf: Option<String>,
    pub cnpj: Option<String>,
    pub ci: Option<String>,
    pub ruc: Option<String>,
    pub rg: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub inscricao_municipal: Option<String>,
    pub data_nascimento: Option<chrono::NaiveDate>,
    pub observacao: Option<String>,
    pub ativo: bool,
    pub papeis: Vec<String>,
    pub criado_em: chrono::DateTime<Utc>,
    pub atualizado_em: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct PessoaCreateDto {
    pub tipo_pessoa: String,
    pub nome_razao_social: String,
    pub nome_fantasia: Option<String>,
    pub cpf: Option<String>,
    pub cnpj: Option<String>,
    pub ci: Option<String>,
    pub ruc: Option<String>,
    pub rg: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub inscricao_municipal: Option<String>,
    pub data_nascimento: Option<chrono::NaiveDate>,
    pub observacao: Option<String>,
    pub papeis: Vec<String>,
}

#[derive(Deserialize)]
pub struct PessoaUpdateDto {
    pub tipo_pessoa: String,
    pub nome_razao_social: String,
    pub nome_fantasia: Option<String>,
    pub cpf: Option<String>,
    pub cnpj: Option<String>,
    pub ci: Option<String>,
    pub ruc: Option<String>,
    pub rg: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub inscricao_municipal: Option<String>,
    pub data_nascimento: Option<chrono::NaiveDate>,
    pub observacao: Option<String>,
    pub papeis: Vec<String>,
}

// ================================================================
// Validações
// ================================================================

fn normalizar_doc(doc: &Option<String>) -> Option<String> {
    doc.as_ref()
        .map(|d| d.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>())
        .filter(|d| !d.is_empty())
}

// ================================================================
// Handlers de Pessoas
// ================================================================

pub async fn listar_pessoas(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PESSOAS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query(
        "SELECT p.id, p.tipo_pessoa, p.nome_razao_social, p.nome_fantasia, p.cpf, p.cnpj, p.ci, p.ruc, p.ativo, p.criado_em,
         COALESCE(array_agg(pp.papel) FILTER (WHERE pp.papel IS NOT NULL), '{}') as papeis
         FROM pessoas p
         LEFT JOIN pessoas_papeis pp ON pp.pessoa_id = p.id AND pp.ativo = true
         GROUP BY p.id ORDER BY p.nome_razao_social"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let lista: Vec<PessoaListaDto> = records.into_iter().map(|row| {
        let papeis: Vec<String> = row.try_get::<Vec<String>, _>("papeis").unwrap_or_default();
        PessoaListaDto {
            id: row.get("id"),
            tipo_pessoa: row.get("tipo_pessoa"),
            nome_razao_social: row.get("nome_razao_social"),
            nome_fantasia: row.get("nome_fantasia"),
            cpf: row.get("cpf"),
            cnpj: row.get("cnpj"),
            ci: row.get("ci"),
            ruc: row.get("ruc"),
            ativo: row.get("ativo"),
            papeis,
            criado_em: row.get("criado_em"),
        }
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok("Pessoas obtidas com sucesso", lista))).into_response()
}

pub async fn listar_pessoas_por_papel(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Path(papel): Path<String>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PESSOAS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let papel_upper = papel.to_uppercase();

    let records = match sqlx::query(
        "SELECT p.id, p.tipo_pessoa, p.nome_razao_social, p.nome_fantasia, p.cpf, p.cnpj, p.ci, p.ruc, p.ativo, p.criado_em,
         COALESCE(array_agg(pp2.papel) FILTER (WHERE pp2.papel IS NOT NULL), '{}') as papeis
         FROM pessoas p
         INNER JOIN pessoas_papeis pp ON pp.pessoa_id = p.id AND pp.papel = $1 AND pp.ativo = true
         LEFT JOIN pessoas_papeis pp2 ON pp2.pessoa_id = p.id AND pp2.ativo = true
         WHERE p.ativo = true
         GROUP BY p.id ORDER BY p.nome_razao_social"
    ).bind(&papel_upper).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let lista: Vec<PessoaListaDto> = records.into_iter().map(|row| {
        let papeis: Vec<String> = row.try_get::<Vec<String>, _>("papeis").unwrap_or_default();
        PessoaListaDto {
            id: row.get("id"),
            tipo_pessoa: row.get("tipo_pessoa"),
            nome_razao_social: row.get("nome_razao_social"),
            nome_fantasia: row.get("nome_fantasia"),
            cpf: row.get("cpf"),
            cnpj: row.get("cnpj"),
            ci: row.get("ci"),
            ruc: row.get("ruc"),
            ativo: row.get("ativo"),
            papeis,
            criado_em: row.get("criado_em"),
        }
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok(format!("{} obtidos com sucesso", papel_upper), lista))).into_response()
}

pub async fn obter_pessoa(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PESSOAS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let row = match sqlx::query(
        "SELECT p.*, COALESCE(array_agg(pp.papel) FILTER (WHERE pp.papel IS NOT NULL), '{}') as papeis
         FROM pessoas p LEFT JOIN pessoas_papeis pp ON pp.pessoa_id = p.id AND pp.ativo = true
         WHERE p.id = $1 GROUP BY p.id"
    ).bind(id).fetch_optional(pool).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(RespostaBase::<()>::falha_manual("Pessoa não encontrada.", "ERRO_NAO_ENCONTRADO", ""))).into_response(),
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let papeis: Vec<String> = row.try_get::<Vec<String>, _>("papeis").unwrap_or_default();

    let dto = PessoaDetalheDto {
        id: row.get("id"),
        tipo_pessoa: row.get("tipo_pessoa"),
        nome_razao_social: row.get("nome_razao_social"),
        nome_fantasia: row.get("nome_fantasia"),
        cpf: row.get("cpf"),
        cnpj: row.get("cnpj"),
        ci: row.get("ci"),
        ruc: row.get("ruc"),
        rg: row.get("rg"),
        inscricao_estadual: row.get("inscricao_estadual"),
        inscricao_municipal: row.get("inscricao_municipal"),
        data_nascimento: row.get("data_nascimento"),
        observacao: row.get("observacao"),
        ativo: row.get("ativo"),
        papeis,
        criado_em: row.get("criado_em"),
        atualizado_em: row.get("atualizado_em"),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Pessoa obtida", dto))).into_response()
}

pub async fn criar_pessoa(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<PessoaCreateDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PESSOAS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    // Validações
    if dados.nome_razao_social.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome/Razão Social é obrigatório.", "ERRO_PESSOA_NOME_OBRIGATORIO", ""))).into_response();
    }
    if dados.tipo_pessoa != "FISICA" && dados.tipo_pessoa != "JURIDICA" {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Tipo de pessoa inválido.", "ERRO_TIPO_PESSOA_INVALIDO", "Use FISICA ou JURIDICA."))).into_response();
    }

    let cpf = normalizar_doc(&dados.cpf);
    let cnpj = normalizar_doc(&dados.cnpj);
    let ci = normalizar_doc(&dados.ci);
    let ruc = normalizar_doc(&dados.ruc);

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO pessoas (id, tipo_pessoa, nome_razao_social, nome_fantasia, cpf, cnpj, ci, ruc, rg, inscricao_estadual, inscricao_municipal, data_nascimento, observacao)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"
    )
    .bind(id).bind(&dados.tipo_pessoa).bind(&dados.nome_razao_social).bind(&dados.nome_fantasia)
    .bind(&cpf).bind(&cnpj).bind(&ci).bind(&ruc)
    .bind(&dados.rg).bind(&dados.inscricao_estadual).bind(&dados.inscricao_municipal)
    .bind(&dados.data_nascimento).bind(&dados.observacao)
    .execute(&mut *tx).await {
        let msg = e.to_string();
        if msg.contains("uq_pessoas_") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Documento já cadastrado.", "ERRO_DOCUMENTO_DUPLICADO", "CPF, CNPJ, CI ou RUC já existe."))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    // Inserir papéis
    for papel in &dados.papeis {
        if let Err(e) = sqlx::query(
            "INSERT INTO pessoas_papeis (pessoa_id, papel) VALUES ($1, $2) ON CONFLICT (pessoa_id, papel) DO UPDATE SET ativo = true"
        ).bind(id).bind(papel.to_uppercase()).execute(&mut *tx).await {
            return ErroApi::interno(e.to_string()).into_response();
        }
    }

    if let Err(e) = tx.commit().await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    // Auditoria e evento (fora da tx para não bloquear em caso de falha de log)
    auditar(pool, "PESSOA", Some(id), "CRIAR", None, None, Some(json!({"nome": &dados.nome_razao_social, "papeis": &dados.papeis})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "PESSOA_CRIADA", "PESSOA", Some(id), json!({"id": id, "nome": &dados.nome_razao_social})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Pessoa criada com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_pessoa(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<PessoaUpdateDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PESSOAS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.nome_razao_social.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome/Razão Social é obrigatório.", "ERRO_PESSOA_NOME_OBRIGATORIO", ""))).into_response();
    }

    let cpf = normalizar_doc(&dados.cpf);
    let cnpj = normalizar_doc(&dados.cnpj);
    let ci = normalizar_doc(&dados.ci);
    let ruc = normalizar_doc(&dados.ruc);

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    if let Err(e) = sqlx::query(
        "UPDATE pessoas SET tipo_pessoa = $1, nome_razao_social = $2, nome_fantasia = $3,
         cpf = $4, cnpj = $5, ci = $6, ruc = $7, rg = $8, inscricao_estadual = $9,
         inscricao_municipal = $10, data_nascimento = $11, observacao = $12, atualizado_em = NOW()
         WHERE id = $13"
    )
    .bind(&dados.tipo_pessoa).bind(&dados.nome_razao_social).bind(&dados.nome_fantasia)
    .bind(&cpf).bind(&cnpj).bind(&ci).bind(&ruc)
    .bind(&dados.rg).bind(&dados.inscricao_estadual).bind(&dados.inscricao_municipal)
    .bind(&dados.data_nascimento).bind(&dados.observacao).bind(id)
    .execute(&mut *tx).await {
        let msg = e.to_string();
        if msg.contains("uq_pessoas_") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Documento já cadastrado para outra pessoa.", "ERRO_DOCUMENTO_DUPLICADO", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    // Sincronizar papéis: inativar todos e reativar os que vieram
    let _ = sqlx::query("UPDATE pessoas_papeis SET ativo = false WHERE pessoa_id = $1").bind(id).execute(&mut *tx).await;
    for papel in &dados.papeis {
        let _ = sqlx::query(
            "INSERT INTO pessoas_papeis (pessoa_id, papel, ativo) VALUES ($1, $2, true) ON CONFLICT (pessoa_id, papel) DO UPDATE SET ativo = true"
        ).bind(id).bind(papel.to_uppercase()).execute(&mut *tx).await;
    }

    if let Err(e) = tx.commit().await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "PESSOA", Some(id), "EDITAR", None, None, Some(json!({"nome": &dados.nome_razao_social})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "PESSOA_ALTERADA", "PESSOA", Some(id), json!({"id": id, "nome": &dados.nome_razao_social})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Pessoa atualizada com sucesso", ()))).into_response()
}

pub async fn inativar_pessoa(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CADASTROS_PESSOAS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if let Err(e) = sqlx::query("UPDATE pessoas SET ativo = false, atualizado_em = NOW() WHERE id = $1")
        .bind(id).execute(pool).await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "PESSOA", Some(id), "INATIVAR", None, None, None, Some(usuario.usuario_id)).await;
    publicar_evento(pool, "PESSOA_INATIVADA", "PESSOA", Some(id), json!({"id": id})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Pessoa inativada com sucesso", ()))).into_response()
}

pub async fn listar_clientes(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    listar_pessoas_por_papel(State(state), axum::extract::Extension(usuario), Path("CLIENTE".to_string())).await
}

pub async fn listar_fornecedores(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    listar_pessoas_por_papel(State(state), axum::extract::Extension(usuario), Path("FORNECEDOR".to_string())).await
}

pub async fn listar_funcionarios(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    listar_pessoas_por_papel(State(state), axum::extract::Extension(usuario), Path("FUNCIONARIO".to_string())).await
}

pub async fn listar_vendedores(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    listar_pessoas_por_papel(State(state), axum::extract::Extension(usuario), Path("VENDEDOR".to_string())).await
}

pub async fn listar_entregadores(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    listar_pessoas_por_papel(State(state), axum::extract::Extension(usuario), Path("ENTREGADOR".to_string())).await
}

pub async fn listar_transportadoras(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    listar_pessoas_por_papel(State(state), axum::extract::Extension(usuario), Path("TRANSPORTADORA".to_string())).await
}

