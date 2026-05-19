use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, FromRow};
use uuid::Uuid;
use chrono::Utc;
use rust_decimal::Decimal;

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

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct PessoaContatoDto {
    pub telefone_principal: Option<String>,
    pub whatsapp: Option<String>,
    pub telefone_secundario: Option<String>,
    pub email: Option<String>,
    pub site: Option<String>,
    pub responsavel: Option<String>,
    pub observacao: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct PessoaEnderecoDto {
    pub tipo_endereco: String,
    pub pais: String,
    pub estado_departamento: Option<String>,
    pub cidade: Option<String>,
    pub bairro: Option<String>,
    pub logradouro: Option<String>,
    pub numero: Option<String>,
    pub complemento: Option<String>,
    pub cep_codigo_postal: Option<String>,
    pub referencia: Option<String>,
    pub principal: bool,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct ClienteConfigDto {
    pub limite_credito: Decimal,
    pub permitir_crediario: bool,
    pub bloquear_venda_prazo: bool,
    pub observacao_credito: Option<String>,
    pub status_cliente: String,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct FornecedorConfigDto {
    pub prazo_pagamento_padrao: Option<i32>,
    pub moeda_padrao_compra: String,
    pub observacao_comercial: Option<String>,
    pub status_fornecedor: String,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct FuncionarioConfigDto {
    pub cargo: Option<String>,
    pub data_admissao: Option<chrono::NaiveDate>,
    pub data_demissao: Option<chrono::NaiveDate>,
    pub salario_base: Option<Decimal>,
    pub ativo_funcionario: bool,
    pub observacao_funcionario: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct VendedorConfigDto {
    pub codigo_vendedor: Option<String>,
    pub tipo_comissao: String,
    pub percentual_comissao: Decimal,
    pub valor_comissao_fixa: Decimal,
    pub comissao_ativa: bool,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct EntregadorConfigDto {
    pub tipo_entregador: Option<String>,
    pub veiculo: Option<String>,
    pub placa: Option<String>,
    pub ativo_entregador: bool,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct TransportadoraConfigDto {
    pub contato_logistica: Option<String>,
    pub observacao_logistica: Option<String>,
    pub ativa_transportadora: bool,
}

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
    
    // Nested
    pub contato: Option<PessoaContatoDto>,
    pub endereco: Option<PessoaEnderecoDto>,
    pub cliente_config: Option<ClienteConfigDto>,
    pub fornecedor_config: Option<FornecedorConfigDto>,
    pub funcionario_config: Option<FuncionarioConfigDto>,
    pub vendedor_config: Option<VendedorConfigDto>,
    pub entregador_config: Option<EntregadorConfigDto>,
    pub transportadora_config: Option<TransportadoraConfigDto>,
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
    
    // Nested
    pub contato: Option<PessoaContatoDto>,
    pub endereco: Option<PessoaEnderecoDto>,
    pub cliente_config: Option<ClienteConfigDto>,
    pub fornecedor_config: Option<FornecedorConfigDto>,
    pub funcionario_config: Option<FuncionarioConfigDto>,
    pub vendedor_config: Option<VendedorConfigDto>,
    pub entregador_config: Option<EntregadorConfigDto>,
    pub transportadora_config: Option<TransportadoraConfigDto>,
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
    
    // Nested
    pub contato: Option<PessoaContatoDto>,
    pub endereco: Option<PessoaEnderecoDto>,
    pub cliente_config: Option<ClienteConfigDto>,
    pub fornecedor_config: Option<FornecedorConfigDto>,
    pub funcionario_config: Option<FuncionarioConfigDto>,
    pub vendedor_config: Option<VendedorConfigDto>,
    pub entregador_config: Option<EntregadorConfigDto>,
    pub transportadora_config: Option<TransportadoraConfigDto>,
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

    // Nested Contato
    let contato = sqlx::query_as::<_, PessoaContatoDto>(
        "SELECT telefone_principal, whatsapp, telefone_secundario, email, site, responsavel, observacao FROM pessoas_contatos WHERE pessoa_id = $1"
    ).bind(id).fetch_optional(pool).await.unwrap_or(None);

    // Nested Endereço
    let endereco = sqlx::query_as::<_, PessoaEnderecoDto>(
        "SELECT tipo_endereco, pais, estado_departamento, cidade, bairro, logradouro, numero, complemento, cep_codigo_postal, referencia, principal FROM pessoas_enderecos WHERE pessoa_id = $1 AND principal = true"
    ).bind(id).fetch_optional(pool).await.unwrap_or(None);

    // Nested Configs
    let cliente_config = sqlx::query_as::<_, ClienteConfigDto>(
        "SELECT limite_credito, permitir_crediario, bloquear_venda_prazo, observacao_credito, status_cliente FROM clientes_configuracoes WHERE pessoa_id = $1"
    ).bind(id).fetch_optional(pool).await.unwrap_or(None);

    let fornecedor_config = sqlx::query_as::<_, FornecedorConfigDto>(
        "SELECT prazo_pagamento_padrao, moeda_padrao_compra, observacao_comercial, status_fornecedor FROM fornecedores_configuracoes WHERE pessoa_id = $1"
    ).bind(id).fetch_optional(pool).await.unwrap_or(None);

    let funcionario_config = sqlx::query_as::<_, FuncionarioConfigDto>(
        "SELECT cargo, data_admissao, data_demissao, salario_base, ativo_funcionario, observacao_funcionario FROM funcionarios_configuracoes WHERE pessoa_id = $1"
    ).bind(id).fetch_optional(pool).await.unwrap_or(None);

    let vendedor_config = sqlx::query_as::<_, VendedorConfigDto>(
        "SELECT codigo_vendedor, tipo_comissao, percentual_comissao, valor_comissao_fixa, comissao_ativa FROM vendedores_configuracoes WHERE pessoa_id = $1"
    ).bind(id).fetch_optional(pool).await.unwrap_or(None);

    let entregador_config = sqlx::query_as::<_, EntregadorConfigDto>(
        "SELECT tipo_entregador, veiculo, placa, ativo_entregador FROM entregadores_configuracoes WHERE pessoa_id = $1"
    ).bind(id).fetch_optional(pool).await.unwrap_or(None);

    let transportadora_config = sqlx::query_as::<_, TransportadoraConfigDto>(
        "SELECT contato_logistica, observacao_logistica, ativa_transportadora FROM transportadoras_configuracoes WHERE pessoa_id = $1"
    ).bind(id).fetch_optional(pool).await.unwrap_or(None);

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
        
        contato,
        endereco,
        cliente_config,
        fornecedor_config,
        funcionario_config,
        vendedor_config,
        entregador_config,
        transportadora_config,
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

    // Inserir Contato
    if let Some(c) = &dados.contato {
        let _ = sqlx::query(
            "INSERT INTO pessoas_contatos (pessoa_id, telefone_principal, whatsapp, telefone_secundario, email, site, responsavel, observacao)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(id).bind(&c.telefone_principal).bind(&c.whatsapp).bind(&c.telefone_secundario)
        .bind(&c.email).bind(&c.site).bind(&c.responsavel).bind(&c.observacao)
        .execute(&mut *tx).await;
    }

    // Inserir Endereço
    if let Some(e) = &dados.endereco {
        let _ = sqlx::query(
            "INSERT INTO pessoas_enderecos (pessoa_id, tipo_endereco, pais, estado_departamento, cidade, bairro, logradouro, numero, complemento, cep_codigo_postal, referencia, principal)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
        )
        .bind(id).bind(&e.tipo_endereco).bind(&e.pais).bind(&e.estado_departamento)
        .bind(&e.cidade).bind(&e.bairro).bind(&e.logradouro).bind(&e.numero)
        .bind(&e.complemento).bind(&e.cep_codigo_postal).bind(&e.referencia).bind(e.principal)
        .execute(&mut *tx).await;
    }

    // Inserir Configurações de Cliente
    if dados.papeis.contains(&"CLIENTE".to_string()) {
        let cc = dados.cliente_config.clone().unwrap_or(ClienteConfigDto {
            limite_credito: Decimal::ZERO,
            permitir_crediario: false,
            bloquear_venda_prazo: false,
            observacao_credito: None,
            status_cliente: "ATIVO".to_string(),
        });
        let _ = sqlx::query(
            "INSERT INTO clientes_configuracoes (pessoa_id, limite_credito, permitir_crediario, bloquear_venda_prazo, observacao_credito, status_cliente)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(id).bind(cc.limite_credito).bind(cc.permitir_crediario).bind(cc.bloquear_venda_prazo)
        .bind(&cc.observacao_credito).bind(&cc.status_cliente)
        .execute(&mut *tx).await;
    }

    // Inserir Configurações de Fornecedor
    if dados.papeis.contains(&"FORNECEDOR".to_string()) {
        let fc = dados.fornecedor_config.clone().unwrap_or(FornecedorConfigDto {
            prazo_pagamento_padrao: Some(30),
            moeda_padrao_compra: "BRL".to_string(),
            observacao_comercial: None,
            status_fornecedor: "ATIVO".to_string(),
        });
        let _ = sqlx::query(
            "INSERT INTO fornecedores_configuracoes (pessoa_id, prazo_pagamento_padrao, moeda_padrao_compra, observacao_comercial, status_fornecedor)
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(id).bind(fc.prazo_pagamento_padrao).bind(&fc.moeda_padrao_compra).bind(&fc.observacao_comercial).bind(&fc.status_fornecedor)
        .execute(&mut *tx).await;
    }

    // Inserir Configurações de Funcionário
    if dados.papeis.contains(&"FUNCIONARIO".to_string()) {
        if let Some(fc) = &dados.funcionario_config {
            let _ = sqlx::query(
                "INSERT INTO funcionarios_configuracoes (pessoa_id, cargo, data_admissao, data_demissao, salario_base, ativo_funcionario, observacao_funcionario)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)"
            )
            .bind(id).bind(&fc.cargo).bind(fc.data_admissao).bind(fc.data_demissao)
            .bind(fc.salario_base.unwrap_or(Decimal::ZERO)).bind(fc.ativo_funcionario).bind(&fc.observacao_funcionario)
            .execute(&mut *tx).await;
        }
    }

    // Inserir Configurações de Vendedor
    if dados.papeis.contains(&"VENDEDOR".to_string()) {
        let vc = dados.vendedor_config.clone().unwrap_or(VendedorConfigDto {
            codigo_vendedor: None,
            tipo_comissao: "SEM_COMISSAO".to_string(),
            percentual_comissao: Decimal::ZERO,
            valor_comissao_fixa: Decimal::ZERO,
            comissao_ativa: true,
        });
        let _ = sqlx::query(
            "INSERT INTO vendedores_configuracoes (pessoa_id, codigo_vendedor, tipo_comissao, percentual_comissao, valor_comissao_fixa, comissao_ativa)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(id).bind(&vc.codigo_vendedor).bind(&vc.tipo_comissao).bind(vc.percentual_comissao).bind(vc.valor_comissao_fixa).bind(vc.comissao_ativa)
        .execute(&mut *tx).await;
    }

    // Inserir Configurações de Entregador
    if dados.papeis.contains(&"ENTREGADOR".to_string()) {
        let ec = dados.entregador_config.clone().unwrap_or(EntregadorConfigDto {
            tipo_entregador: Some("PROPRIO".to_string()),
            veiculo: None,
            placa: None,
            ativo_entregador: true,
        });
        let _ = sqlx::query(
            "INSERT INTO entregadores_configuracoes (pessoa_id, tipo_entregador, veiculo, placa, ativo_entregador)
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(id).bind(&ec.tipo_entregador).bind(&ec.veiculo).bind(&ec.placa).bind(ec.ativo_entregador)
        .execute(&mut *tx).await;
    }

    // Inserir Configurações de Transportadora
    if dados.papeis.contains(&"TRANSPORTADORA".to_string()) {
        let tc = dados.transportadora_config.clone().unwrap_or(TransportadoraConfigDto {
            contato_logistica: None,
            observacao_logistica: None,
            ativa_transportadora: true,
        });
        let _ = sqlx::query(
            "INSERT INTO transportadoras_configuracoes (pessoa_id, contato_logistica, observacao_logistica, ativa_transportadora)
             VALUES ($1, $2, $3, $4)"
        )
        .bind(id).bind(&tc.contato_logistica).bind(&tc.observacao_logistica).bind(tc.ativa_transportadora)
        .execute(&mut *tx).await;
    }

    if let Err(e) = tx.commit().await {
        return ErroApi::interno(e.to_string()).into_response();
    }

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

    // Atualizar/Inserir Contato
    if let Some(c) = &dados.contato {
        let _ = sqlx::query(
            "INSERT INTO pessoas_contatos (pessoa_id, telefone_principal, whatsapp, telefone_secundario, email, site, responsavel, observacao)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (pessoa_id) DO UPDATE
             SET telefone_principal = EXCLUDED.telefone_principal, whatsapp = EXCLUDED.whatsapp,
                 telefone_secundario = EXCLUDED.telefone_secundario, email = EXCLUDED.email,
                 site = EXCLUDED.site, responsavel = EXCLUDED.responsavel, observacao = EXCLUDED.observacao, atualizado_em = NOW()"
        )
        .bind(id).bind(&c.telefone_principal).bind(&c.whatsapp).bind(&c.telefone_secundario)
        .bind(&c.email).bind(&c.site).bind(&c.responsavel).bind(&c.observacao)
        .execute(&mut *tx).await;
    }

    // Atualizar/Inserir Endereço
    if let Some(e) = &dados.endereco {
        let _ = sqlx::query(
            "INSERT INTO pessoas_enderecos (pessoa_id, tipo_endereco, pais, estado_departamento, cidade, bairro, logradouro, numero, complemento, cep_codigo_postal, referencia, principal)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             ON CONFLICT (pessoa_id) DO UPDATE
             SET tipo_endereco = EXCLUDED.tipo_endereco, pais = EXCLUDED.pais,
                 estado_departamento = EXCLUDED.estado_departamento, cidade = EXCLUDED.cidade,
                 bairro = EXCLUDED.bairro, logradouro = EXCLUDED.logradouro, numero = EXCLUDED.numero,
                 complemento = EXCLUDED.complemento, cep_codigo_postal = EXCLUDED.cep_codigo_postal,
                 referencia = EXCLUDED.referencia, atualizado_em = NOW()"
        )
        .bind(id).bind(&e.tipo_endereco).bind(&e.pais).bind(&e.estado_departamento)
        .bind(&e.cidade).bind(&e.bairro).bind(&e.logradouro).bind(&e.numero)
        .bind(&e.complemento).bind(&e.cep_codigo_postal).bind(&e.referencia).bind(e.principal)
        .execute(&mut *tx).await;
    }

    // Atualizar Configurações de Cliente
    if dados.papeis.contains(&"CLIENTE".to_string()) {
        if let Some(cc) = &dados.cliente_config {
            let _ = sqlx::query(
                "INSERT INTO clientes_configuracoes (pessoa_id, limite_credito, permitir_crediario, bloquear_venda_prazo, observacao_credito, status_cliente)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (pessoa_id) DO UPDATE
                 SET limite_credito = EXCLUDED.limite_credito, permitir_crediario = EXCLUDED.permitir_crediario,
                     bloquear_venda_prazo = EXCLUDED.bloquear_venda_prazo, observacao_credito = EXCLUDED.observacao_credito,
                     status_cliente = EXCLUDED.status_cliente, atualizado_em = NOW()"
            )
            .bind(id).bind(cc.limite_credito).bind(cc.permitir_crediario).bind(cc.bloquear_venda_prazo)
            .bind(&cc.observacao_credito).bind(&cc.status_cliente)
            .execute(&mut *tx).await;
        }
    }

    // Atualizar Configurações de Fornecedor
    if dados.papeis.contains(&"FORNECEDOR".to_string()) {
        if let Some(fc) = &dados.fornecedor_config {
            let _ = sqlx::query(
                "INSERT INTO fornecedores_configuracoes (pessoa_id, prazo_pagamento_padrao, moeda_padrao_compra, observacao_comercial, status_fornecedor)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (pessoa_id) DO UPDATE
                 SET prazo_pagamento_padrao = EXCLUDED.prazo_pagamento_padrao, moeda_padrao_compra = EXCLUDED.moeda_padrao_compra,
                     observacao_comercial = EXCLUDED.observacao_comercial, status_fornecedor = EXCLUDED.status_fornecedor, atualizado_em = NOW()"
            )
            .bind(id).bind(fc.prazo_pagamento_padrao).bind(&fc.moeda_padrao_compra).bind(&fc.observacao_comercial).bind(&fc.status_fornecedor)
            .execute(&mut *tx).await;
        }
    }

    // Atualizar Configurações de Funcionário
    if dados.papeis.contains(&"FUNCIONARIO".to_string()) {
        if let Some(fc) = &dados.funcionario_config {
            let _ = sqlx::query(
                "INSERT INTO funcionarios_configuracoes (pessoa_id, cargo, data_admissao, data_demissao, salario_base, ativo_funcionario, observacao_funcionario)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)
                 ON CONFLICT (pessoa_id) DO UPDATE
                 SET cargo = EXCLUDED.cargo, data_admissao = EXCLUDED.data_admissao, data_demissao = EXCLUDED.data_demissao,
                     salario_base = EXCLUDED.salario_base, ativo_funcionario = EXCLUDED.ativo_funcionario,
                     observacao_funcionario = EXCLUDED.observacao_funcionario, atualizado_em = NOW()"
            )
            .bind(id).bind(&fc.cargo).bind(fc.data_admissao).bind(fc.data_demissao)
            .bind(fc.salario_base.unwrap_or(Decimal::ZERO)).bind(fc.ativo_funcionario).bind(&fc.observacao_funcionario)
            .execute(&mut *tx).await;
        }
    }

    // Atualizar Configurações de Vendedor
    if dados.papeis.contains(&"VENDEDOR".to_string()) {
        if let Some(vc) = &dados.vendedor_config {
            let _ = sqlx::query(
                "INSERT INTO vendedores_configuracoes (pessoa_id, codigo_vendedor, tipo_comissao, percentual_comissao, valor_comissao_fixa, comissao_ativa)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (pessoa_id) DO UPDATE
                 SET codigo_vendedor = EXCLUDED.codigo_vendedor, tipo_comissao = EXCLUDED.tipo_comissao,
                     percentual_comissao = EXCLUDED.percentual_comissao, valor_comissao_fixa = EXCLUDED.valor_comissao_fixa,
                     comissao_ativa = EXCLUDED.comissao_ativa, atualizado_em = NOW()"
            )
            .bind(id).bind(&vc.codigo_vendedor).bind(&vc.tipo_comissao).bind(vc.percentual_comissao).bind(vc.valor_comissao_fixa).bind(vc.comissao_ativa)
            .execute(&mut *tx).await;
        }
    }

    // Atualizar Configurações de Entregador
    if dados.papeis.contains(&"ENTREGADOR".to_string()) {
        if let Some(ec) = &dados.entregador_config {
            let _ = sqlx::query(
                "INSERT INTO entregadores_configuracoes (pessoa_id, tipo_entregador, veiculo, placa, ativo_entregador)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (pessoa_id) DO UPDATE
                 SET tipo_entregador = EXCLUDED.tipo_entregador, veiculo = EXCLUDED.veiculo,
                     placa = EXCLUDED.placa, ativo_entregador = EXCLUDED.ativo_entregador, atualizado_em = NOW()"
            )
            .bind(id).bind(&ec.tipo_entregador).bind(&ec.veiculo).bind(&ec.placa).bind(ec.ativo_entregador)
            .execute(&mut *tx).await;
        }
    }

    // Atualizar Configurações de Transportadora
    if dados.papeis.contains(&"TRANSPORTADORA".to_string()) {
        if let Some(tc) = &dados.transportadora_config {
            let _ = sqlx::query(
                "INSERT INTO transportadoras_configuracoes (pessoa_id, contato_logistica, observacao_logistica, ativa_transportadora)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (pessoa_id) DO UPDATE
                 SET contato_logistica = EXCLUDED.contato_logistica, observacao_logistica = EXCLUDED.observacao_logistica,
                     ativa_transportadora = EXCLUDED.ativa_transportadora, atualizado_em = NOW()"
            )
            .bind(id).bind(&tc.contato_logistica).bind(&tc.observacao_logistica).bind(tc.ativa_transportadora)
            .execute(&mut *tx).await;
        }
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
