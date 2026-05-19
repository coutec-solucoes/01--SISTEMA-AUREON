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
use rust_decimal::Decimal;

use crate::{app::AppState, erros::ErroApi, middleware::UsuarioLogado};
use aureon_core::RespostaBase;
use crate::routes::seguranca::tem_permissao;
use super::pessoas::{auditar, publicar_evento};

// ================================================================
// DTOs de Produtos
// ================================================================

#[derive(Serialize)]
pub struct ProdutoListaDto {
    pub id: Uuid,
    pub codigo_interno: Option<String>,
    pub codigo_barras: Option<String>,
    pub descricao: String,
    pub referencia: Option<String>,
    pub grupo_id: Uuid,
    pub grupo_nome: String,
    pub unidade_medida: String,
    pub preco_custo: Decimal,
    pub margem_lucro_percentual: Decimal,
    pub preco_venda: Decimal,
    pub estoque_atual: Decimal,
    pub ativo: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProdutoFiscalDto {
    pub ncm: Option<String>,
    pub regra_tributaria_brasil: Option<String>,
    pub cest: Option<String>,
    pub iva_tipo: String,
    pub regra_tributaria_paraguai: Option<String>,
    pub pais_fiscal_referencia: String,
}

#[derive(Serialize)]
pub struct ProdutoDetalheDto {
    pub id: Uuid,
    pub codigo_interno: Option<String>,
    pub codigo_barras: Option<String>,
    pub descricao: String,
    pub descricao_detalhada: Option<String>,
    pub referencia: Option<String>,
    pub grupo_id: Uuid,
    pub subgrupo_id: Option<Uuid>,
    pub marca_id: Option<Uuid>,
    pub unidade_medida: String,
    pub preco_custo: Decimal,
    pub margem_lucro_percentual: Decimal,
    pub preco_venda: Decimal,
    pub estoque_atual: Decimal,
    pub estoque_minimo: Decimal,
    pub controla_estoque: bool,
    pub controla_validade: bool,
    pub produto_balanca: bool,
    pub reconfirmar_pesagem_pdv: bool,
    pub leitura_etiqueta_balanca: bool,
    pub produto_pizza: bool,
    pub produto_combo: bool,
    pub permite_adicionais: bool,
    pub desconto_maximo_percentual: Decimal,
    pub exibir_catalogo: bool,
    pub local_producao_id: Option<Uuid>,
    pub ativo: bool,
    pub fiscal: Option<ProdutoFiscalDto>,
}

#[derive(Deserialize)]
pub struct ProdutoCreateDto {
    pub codigo_interno: Option<String>,
    pub codigo_barras: Option<String>,
    pub descricao: String,
    pub descricao_detalhada: Option<String>,
    pub referencia: Option<String>,
    pub grupo_id: Uuid,
    pub subgrupo_id: Option<Uuid>,
    pub marca_id: Option<Uuid>,
    pub unidade_medida: String,
    pub preco_custo: Decimal,
    pub margem_lucro_percentual: Decimal,
    pub preco_venda: Decimal,
    pub estoque_atual: Option<Decimal>,
    pub estoque_minimo: Decimal,
    pub controla_estoque: bool,
    pub controla_validade: bool,
    pub produto_balanca: bool,
    pub reconfirmar_pesagem_pdv: bool,
    pub leitura_etiqueta_balanca: bool,
    pub produto_pizza: bool,
    pub produto_combo: bool,
    pub permite_adicionais: bool,
    pub desconto_maximo_percentual: Decimal,
    pub exibir_catalogo: bool,
    pub local_producao_id: Option<Uuid>,
    pub fiscal: Option<ProdutoFiscalDto>,
}

#[derive(Deserialize)]
pub struct ProdutoUpdateDto {
    pub codigo_interno: Option<String>,
    pub codigo_barras: Option<String>,
    pub descricao: String,
    pub descricao_detalhada: Option<String>,
    pub referencia: Option<String>,
    pub grupo_id: Uuid,
    pub subgrupo_id: Option<Uuid>,
    pub marca_id: Option<Uuid>,
    pub unidade_medida: String,
    pub preco_custo: Decimal,
    pub margem_lucro_percentual: Decimal,
    pub preco_venda: Decimal,
    pub estoque_atual: Option<Decimal>,
    pub estoque_minimo: Decimal,
    pub controla_estoque: bool,
    pub controla_validade: bool,
    pub produto_balanca: bool,
    pub reconfirmar_pesagem_pdv: bool,
    pub leitura_etiqueta_balanca: bool,
    pub produto_pizza: bool,
    pub produto_combo: bool,
    pub permite_adicionais: bool,
    pub desconto_maximo_percentual: Decimal,
    pub exibir_catalogo: bool,
    pub local_producao_id: Option<Uuid>,
    pub fiscal: Option<ProdutoFiscalDto>,
}

#[derive(Serialize)]
pub struct HistoricoPrecoDto {
    pub id: i64,
    pub preco_custo_anterior: Option<Decimal>,
    pub preco_custo_novo: Option<Decimal>,
    pub preco_venda_anterior: Option<Decimal>,
    pub preco_venda_novo: Option<Decimal>,
    pub margem_anterior: Option<Decimal>,
    pub margem_nova: Option<Decimal>,
    pub usuario_nome: Option<String>,
    pub motivo: Option<String>,
    pub criado_em: chrono::DateTime<Utc>,
}

// ================================================================
// DTOs de Sabores de Pizza
// ================================================================

#[derive(Serialize)]
pub struct SaborPizzaListaDto {
    pub id: Uuid,
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: bool,
}

#[derive(Deserialize)]
pub struct SaborPizzaInputDto {
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: Option<bool>,
}

// ================================================================
// DTOs de Combos
// ================================================================

#[derive(Serialize)]
pub struct ComboItemDto {
    pub id: Uuid,
    pub combo_id: Uuid,
    pub produto_item_id: Uuid,
    pub produto_item_nome: String,
    pub quantidade: Decimal,
    pub valor_original_item: Option<Decimal>,
    pub valor_combo_item: Decimal,
    pub ativo: bool,
}

#[derive(Deserialize)]
pub struct ComboInputDto {
    pub combo_id: Uuid,
    pub produto_item_id: Uuid,
    pub quantidade: Decimal,
    pub valor_original_item: Option<Decimal>,
    pub valor_combo_item: Decimal,
    pub ativo: Option<bool>,
}

// ================================================================
// DTOs de Adicionais
// ================================================================

#[derive(Serialize)]
pub struct AdicionalListaDto {
    pub id: Uuid,
    pub nome: String,
    pub descricao: Option<String>,
    pub preco_adicional: Decimal,
    pub ativo: bool,
}

#[derive(Deserialize)]
pub struct AdicionalInputDto {
    pub nome: String,
    pub descricao: Option<String>,
    pub preco_adicional: Decimal,
    pub ativo: Option<bool>,
}

// ================================================================
// DTOs de Locais de Produção
// ================================================================

#[derive(Serialize)]
pub struct LocalProducaoListaDto {
    pub id: Uuid,
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: bool,
}

#[derive(Deserialize)]
pub struct LocalProducaoInputDto {
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: Option<bool>,
}

// ================================================================
// Validações Auxiliares
// ================================================================

fn validar_valores_produto(
    descricao: &str,
    preco_custo: Decimal,
    margem: Decimal,
    preco_venda: Decimal,
    estoque_minimo: Decimal,
    desconto_maximo: Decimal,
) -> Result<(), ErroApi> {
    if descricao.trim().is_empty() {
        return Err(ErroApi::bad_request("A descrição do produto é obrigatória.", "ERRO_PRODUTO_DESCRICAO_OBRIGATORIA"));
    }
    if preco_custo < Decimal::ZERO {
        return Err(ErroApi::bad_request("O preço de custo não pode ser negativo.", "ERRO_PRECO_INVALIDO"));
    }
    if margem < Decimal::ZERO {
        return Err(ErroApi::bad_request("A margem de lucro não pode ser negativa.", "ERRO_MARGEM_INVALIDA"));
    }
    if preco_venda < Decimal::ZERO {
        return Err(ErroApi::bad_request("O preço de venda não pode ser negativo.", "ERRO_PRECO_INVALIDO"));
    }
    if estoque_minimo < Decimal::ZERO {
        return Err(ErroApi::bad_request("O estoque mínimo não pode ser negativo.", "ERRO_PRECO_INVALIDO"));
    }
    if desconto_maximo < Decimal::ZERO || desconto_maximo > Decimal::from(100) {
        return Err(ErroApi::bad_request("O desconto máximo deve estar entre 0% e 100%.", "ERRO_DESCONTO_INVALIDO"));
    }
    Ok(())
}

// ================================================================
// Handlers de Produtos
// ================================================================

pub async fn listar_produtos(
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
        "SELECT p.id, p.codigo_interno, p.codigo_barras, p.descricao, p.referencia, p.grupo_id, g.nome as grupo_nome,
                p.unidade_medida, p.preco_custo, p.margem_lucro_percentual, p.preco_venda, p.estoque_atual, p.ativo
         FROM produtos p
         JOIN produtos_grupos g ON g.id = p.grupo_id
         ORDER BY p.descricao"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let lista: Vec<ProdutoListaDto> = records.into_iter().map(|row| ProdutoListaDto {
        id: row.get("id"),
        codigo_interno: row.get("codigo_interno"),
        codigo_barras: row.get("codigo_barras"),
        descricao: row.get("descricao"),
        referencia: row.get("referencia"),
        grupo_id: row.get("grupo_id"),
        grupo_nome: row.get("grupo_nome"),
        unidade_medida: row.get("unidade_medida"),
        preco_custo: row.get("preco_custo"),
        margem_lucro_percentual: row.get("margem_lucro_percentual"),
        preco_venda: row.get("preco_venda"),
        estoque_atual: row.get("estoque_atual"),
        ativo: row.get("ativo"),
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok("Produtos obtidos com sucesso", lista))).into_response()
}

pub async fn obter_produto(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
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

    let p_row = match sqlx::query(
        "SELECT * FROM produtos WHERE id = $1"
    ).bind(id).fetch_optional(pool).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(RespostaBase::<()>::falha_manual("Produto não encontrado.", "ERRO_NAO_ENCONTRADO", ""))).into_response(),
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let f_row = sqlx::query(
        "SELECT * FROM produtos_fiscal WHERE produto_id = $1"
    ).bind(id).fetch_optional(pool).await.unwrap_or(None);

    let fiscal = f_row.map(|row| ProdutoFiscalDto {
        ncm: row.get("ncm"),
        regra_tributaria_brasil: row.get("regra_tributaria_brasil"),
        cest: row.get("cest"),
        iva_tipo: row.get("iva_tipo"),
        regra_tributaria_paraguai: row.get("regra_tributaria_paraguai"),
        pais_fiscal_referencia: row.get("pais_fiscal_referencia"),
    });

    let dto = ProdutoDetalheDto {
        id: p_row.get("id"),
        codigo_interno: p_row.get("codigo_interno"),
        codigo_barras: p_row.get("codigo_barras"),
        descricao: p_row.get("descricao"),
        descricao_detalhada: p_row.get("descricao_detalhada"),
        referencia: p_row.get("referencia"),
        grupo_id: p_row.get("grupo_id"),
        subgrupo_id: p_row.get("subgrupo_id"),
        marca_id: p_row.get("marca_id"),
        unidade_medida: p_row.get("unidade_medida"),
        preco_custo: p_row.get("preco_custo"),
        margem_lucro_percentual: p_row.get("margem_lucro_percentual"),
        preco_venda: p_row.get("preco_venda"),
        estoque_atual: p_row.get("estoque_atual"),
        estoque_minimo: p_row.get("estoque_minimo"),
        controla_estoque: p_row.get("controla_estoque"),
        controla_validade: p_row.get("controla_validade"),
        produto_balanca: p_row.get("produto_balanca"),
        reconfirmar_pesagem_pdv: p_row.get("reconfirmar_pesagem_pdv"),
        leitura_etiqueta_balanca: p_row.get("leitura_etiqueta_balanca"),
        produto_pizza: p_row.get("produto_pizza"),
        produto_combo: p_row.get("produto_combo"),
        permite_adicionais: p_row.get("permite_adicionais"),
        desconto_maximo_percentual: p_row.get("desconto_maximo_percentual"),
        exibir_catalogo: p_row.get("exibir_catalogo"),
        local_producao_id: p_row.get("local_producao_id"),
        ativo: p_row.get("ativo"),
        fiscal,
    };

    (StatusCode::OK, Json(RespostaBase::ok("Produto obtido com sucesso", dto))).into_response()
}

pub async fn criar_produto(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ProdutoCreateDto>,
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

    if let Err(e) = validar_valores_produto(
        &dados.descricao,
        dados.preco_custo,
        dados.margem_lucro_percentual,
        dados.preco_venda,
        dados.estoque_minimo,
        dados.desconto_maximo_percentual,
    ) {
        return e.into_response();
    }

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let id = Uuid::new_v4();
    let est_atual = dados.estoque_atual.unwrap_or(Decimal::ZERO);

    if let Err(e) = sqlx::query(
        "INSERT INTO produtos (id, codigo_interno, codigo_barras, descricao, descricao_detalhada, referencia,
                             grupo_id, subgrupo_id, marca_id, unidade_medida, preco_custo, margem_lucro_percentual,
                             preco_venda, estoque_atual, estoque_minimo, controla_estoque, controla_validade,
                             produto_balanca, reconfirmar_pesagem_pdv, leitura_etiqueta_balanca, produto_pizza,
                             produto_combo, permite_adicionais, desconto_maximo_percentual, exibir_catalogo, local_producao_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26)"
    )
    .bind(id).bind(&dados.codigo_interno).bind(&dados.codigo_barras).bind(&dados.descricao)
    .bind(&dados.descricao_detalhada).bind(&dados.referencia).bind(dados.grupo_id).bind(dados.subgrupo_id)
    .bind(dados.marca_id).bind(&dados.unidade_medida).bind(dados.preco_custo).bind(dados.margem_lucro_percentual)
    .bind(dados.preco_venda).bind(est_atual).bind(dados.estoque_minimo).bind(dados.controla_estoque)
    .bind(dados.controla_validade).bind(dados.produto_balanca).bind(dados.reconfirmar_pesagem_pdv)
    .bind(dados.leitura_etiqueta_balanca).bind(dados.produto_pizza).bind(dados.produto_combo)
    .bind(dados.permite_adicionais).bind(dados.desconto_maximo_percentual).bind(dados.exibir_catalogo)
    .bind(dados.local_producao_id)
    .execute(&mut *tx).await {
        let msg = e.to_string();
        if msg.contains("uq_produtos_codigo_barras") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Código de barras já cadastrado.", "ERRO_CODIGO_BARRAS_DUPLICADO", ""))).into_response();
        }
        if msg.contains("uq_produtos_codigo_interno") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Código interno já cadastrado.", "ERRO_CODIGO_INTERNO_DUPLICADO", ""))).into_response();
        }
        if msg.contains("uq_produtos_referencia") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Referência já cadastrada.", "ERRO_REFERENCIA_DUPLICADA", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    // Inserir Fiscal Base
    if let Some(fisc) = &dados.fiscal {
        let _ = sqlx::query(
            "INSERT INTO produtos_fiscal (produto_id, ncm, regra_tributaria_brasil, cest, iva_tipo, regra_tributaria_paraguai, pais_fiscal_referencia)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(id).bind(&fisc.ncm).bind(&fisc.regra_tributaria_brasil).bind(&fisc.cest)
        .bind(&fisc.iva_tipo).bind(&fisc.regra_tributaria_paraguai).bind(&fisc.pais_fiscal_referencia)
        .execute(&mut *tx).await;
    } else {
        // Padrão vazio
        let _ = sqlx::query(
            "INSERT INTO produtos_fiscal (produto_id, pais_fiscal_referencia) VALUES ($1, 'BR')"
        ).bind(id).execute(&mut *tx).await;
    }

    // Gravar histórico inicial de preço
    let _ = sqlx::query(
        "INSERT INTO produtos_historico_precos (produto_id, preco_custo_novo, preco_venda_novo, margem_nova, usuario_id, motivo)
         VALUES ($1, $2, $3, $4, $5, 'Cadastro Inicial do Produto')"
    )
    .bind(id).bind(dados.preco_custo).bind(dados.preco_venda).bind(dados.margem_lucro_percentual)
    .bind(usuario.usuario_id)
    .execute(&mut *tx).await;

    if let Err(e) = tx.commit().await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "PRODUTO", Some(id), "CRIAR", None, None, Some(json!({"descricao": &dados.descricao, "preco_venda": dados.preco_venda})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "PRODUTO_CRIADO", "PRODUTO", Some(id), json!({"id": id, "descricao": &dados.descricao})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Produto criado com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_produto(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ProdutoUpdateDto>,
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

    if let Err(e) = validar_valores_produto(
        &dados.descricao,
        dados.preco_custo,
        dados.margem_lucro_percentual,
        dados.preco_venda,
        dados.estoque_minimo,
        dados.desconto_maximo_percentual,
    ) {
        return e.into_response();
    }

    // Obter dados antigos para histórico de preço
    let old_data = match sqlx::query(
        "SELECT preco_custo, preco_venda, margem_lucro_percentual FROM produtos WHERE id = $1"
    ).bind(id).fetch_optional(pool).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(RespostaBase::<()>::falha_manual("Produto não encontrado.", "ERRO_NAO_ENCONTRADO", ""))).into_response(),
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let old_custo: Decimal = old_data.get("preco_custo");
    let old_venda: Decimal = old_data.get("preco_venda");
    let old_margem: Decimal = old_data.get("margem_lucro_percentual");

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let est_atual = dados.estoque_atual.unwrap_or(Decimal::ZERO);

    if let Err(e) = sqlx::query(
        "UPDATE produtos SET codigo_interno = $1, codigo_barras = $2, descricao = $3, descricao_detalhada = $4,
                            referencia = $5, grupo_id = $6, subgrupo_id = $7, marca_id = $8, unidade_medida = $9,
                            preco_custo = $10, margem_lucro_percentual = $11, preco_venda = $12, estoque_atual = $13,
                            estoque_minimo = $14, controla_estoque = $15, controla_validade = $16, produto_balanca = $17,
                            reconfirmar_pesagem_pdv = $18, leitura_etiqueta_balanca = $19, produto_pizza = $20,
                            produto_combo = $21, permite_adicionais = $22, desconto_maximo_percentual = $23,
                            exibir_catalogo = $24, local_producao_id = $25, atualizado_em = NOW()
         WHERE id = $26"
    )
    .bind(&dados.codigo_interno).bind(&dados.codigo_barras).bind(&dados.descricao)
    .bind(&dados.descricao_detalhada).bind(&dados.referencia).bind(dados.grupo_id).bind(dados.subgrupo_id)
    .bind(dados.marca_id).bind(&dados.unidade_medida).bind(dados.preco_custo).bind(dados.margem_lucro_percentual)
    .bind(dados.preco_venda).bind(est_atual).bind(dados.estoque_minimo).bind(dados.controla_estoque)
    .bind(dados.controla_validade).bind(dados.produto_balanca).bind(dados.reconfirmar_pesagem_pdv)
    .bind(dados.leitura_etiqueta_balanca).bind(dados.produto_pizza).bind(dados.produto_combo)
    .bind(dados.permite_adicionais).bind(dados.desconto_maximo_percentual).bind(dados.exibir_catalogo)
    .bind(dados.local_producao_id).bind(id)
    .execute(&mut *tx).await {
        let msg = e.to_string();
        if msg.contains("uq_produtos_codigo_barras") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Código de barras já cadastrado para outro produto.", "ERRO_CODIGO_BARRAS_DUPLICADO", ""))).into_response();
        }
        if msg.contains("uq_produtos_codigo_interno") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Código interno já cadastrado para outro produto.", "ERRO_CODIGO_INTERNO_DUPLICADO", ""))).into_response();
        }
        if msg.contains("uq_produtos_referencia") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Referência já cadastrada para outro produto.", "ERRO_REFERENCIA_DUPLICADA", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    // Atualizar Fiscal
    if let Some(fisc) = &dados.fiscal {
        let _ = sqlx::query(
            "INSERT INTO produtos_fiscal (produto_id, ncm, regra_tributaria_brasil, cest, iva_tipo, regra_tributaria_paraguai, pais_fiscal_referencia)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (produto_id) DO UPDATE
             SET ncm = EXCLUDED.ncm, regra_tributaria_brasil = EXCLUDED.regra_tributaria_brasil, cest = EXCLUDED.cest,
                 iva_tipo = EXCLUDED.iva_tipo, regra_tributaria_paraguai = EXCLUDED.regra_tributaria_paraguai,
                 pais_fiscal_referencia = EXCLUDED.pais_fiscal_referencia, atualizado_em = NOW()"
        )
        .bind(id).bind(&fisc.ncm).bind(&fisc.regra_tributaria_brasil).bind(&fisc.cest)
        .bind(&fisc.iva_tipo).bind(&fisc.regra_tributaria_paraguai).bind(&fisc.pais_fiscal_referencia)
        .execute(&mut *tx).await;
    }

    // Se houve alteração de custo/venda/margem, registrar histórico de preços
    if old_custo != dados.preco_custo || old_venda != dados.preco_venda || old_margem != dados.margem_lucro_percentual {
        let _ = sqlx::query(
            "INSERT INTO produtos_historico_precos (produto_id, preco_custo_anterior, preco_custo_novo,
                                                 preco_venda_anterior, preco_venda_novo, margem_anterior, margem_nova,
                                                 usuario_id, motivo)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'Atualização Manual do Produto')"
        )
        .bind(id)
        .bind(old_custo).bind(dados.preco_custo)
        .bind(old_venda).bind(dados.preco_venda)
        .bind(old_margem).bind(dados.margem_lucro_percentual)
        .bind(usuario.usuario_id)
        .execute(&mut *tx).await;
    }

    if let Err(e) = tx.commit().await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "PRODUTO", Some(id), "EDITAR", None, None, Some(json!({"descricao": &dados.descricao, "preco_venda": dados.preco_venda})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "PRODUTO_ALTERADO", "PRODUTO", Some(id), json!({"id": id, "descricao": &dados.descricao})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Produto atualizado com sucesso", ()))).into_response()
}

pub async fn inativar_produto(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
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

    if let Err(e) = sqlx::query("UPDATE produtos SET ativo = false, atualizado_em = NOW() WHERE id = $1")
        .bind(id).execute(pool).await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "PRODUTO", Some(id), "INATIVAR", None, None, None, Some(usuario.usuario_id)).await;
    publicar_evento(pool, "PRODUTO_INATIVADO", "PRODUTO", Some(id), json!({"id": id})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Produto inativado com sucesso", ()))).into_response()
}

pub async fn listar_historico_precos(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
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
        "SELECT h.*, u.nome as usuario_nome
         FROM produtos_historico_precos h
         LEFT JOIN usuarios u ON u.id = h.usuario_id
         WHERE h.produto_id = $1
         ORDER BY h.criado_em DESC"
    ).bind(id).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let lista: Vec<HistoricoPrecoDto> = records.into_iter().map(|row| HistoricoPrecoDto {
        id: row.get("id"),
        preco_custo_anterior: row.get("preco_custo_anterior"),
        preco_custo_novo: row.get("preco_custo_novo"),
        preco_venda_anterior: row.get("preco_venda_anterior"),
        preco_venda_novo: row.get("preco_venda_novo"),
        margem_anterior: row.get("margem_anterior"),
        margem_nova: row.get("margem_nova"),
        usuario_nome: row.get("usuario_nome"),
        motivo: row.get("motivo"),
        criado_em: row.get("criado_em"),
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok("Histórico de preços obtido", lista))).into_response()
}

// ================================================================
// Handlers de Sabores de Pizza
// ================================================================

pub async fn listar_sabores_pizza(
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
        "SELECT id, nome, descricao, ativo FROM pizza_sabores ORDER BY nome"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let lista: Vec<SaborPizzaListaDto> = records.into_iter().map(|row| SaborPizzaListaDto {
        id: row.get("id"),
        nome: row.get("nome"),
        descricao: row.get("descricao"),
        ativo: row.get("ativo"),
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok("Sabores de pizza obtidos", lista))).into_response()
}

pub async fn criar_sabor_pizza(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<SaborPizzaInputDto>,
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
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do sabor é obrigatório.", "ERRO_PRODUTO_DESCRICAO_OBRIGATORIA", ""))).into_response();
    }

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO pizza_sabores (id, nome, descricao, ativo) VALUES ($1, $2, $3, $4)"
    ).bind(id).bind(&dados.nome).bind(&dados.descricao).bind(dados.ativo.unwrap_or(true))
    .execute(pool).await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "SABOR_PIZZA", Some(id), "CRIAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "SABOR_PIZZA_ALTERADO", "SABOR_PIZZA", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Sabor de pizza criado", json!({"id": id})))).into_response()
}

pub async fn atualizar_sabor_pizza(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<SaborPizzaInputDto>,
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
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do sabor é obrigatório.", "ERRO_PRODUTO_DESCRICAO_OBRIGATORIA", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "UPDATE pizza_sabores SET nome = $1, descricao = $2, ativo = $3, atualizado_em = NOW() WHERE id = $4"
    ).bind(&dados.nome).bind(&dados.descricao).bind(dados.ativo.unwrap_or(true)).bind(id)
    .execute(pool).await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "SABOR_PIZZA", Some(id), "EDITAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "SABOR_PIZZA_ALTERADO", "SABOR_PIZZA", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Sabor de pizza atualizado", ()))).into_response()
}

// ================================================================
// Handlers de Combos
// ================================================================

pub async fn listar_combos(
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
        "SELECT c.*, p.descricao as produto_item_nome
         FROM produtos_combos c
         JOIN produtos p ON p.id = c.produto_item_id
         ORDER BY c.combo_id"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let lista: Vec<ComboItemDto> = records.into_iter().map(|row| ComboItemDto {
        id: row.get("id"),
        combo_id: row.get("combo_id"),
        produto_item_id: row.get("produto_item_id"),
        produto_item_nome: row.get("produto_item_nome"),
        quantidade: row.get("quantidade"),
        valor_original_item: row.get("valor_original_item"),
        valor_combo_item: row.get("valor_combo_item"),
        ativo: row.get("ativo"),
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok("Itens de combo obtidos", lista))).into_response()
}

pub async fn criar_combo_item(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ComboInputDto>,
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

    if dados.quantidade <= Decimal::ZERO {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Quantidade deve ser maior que zero.", "ERRO_PRECO_INVALIDO", ""))).into_response();
    }

    let id = Uuid::new_v4();
    let ativo = dados.ativo.unwrap_or(true);

    if let Err(e) = sqlx::query(
        "INSERT INTO produtos_combos (id, combo_id, produto_item_id, quantidade, valor_original_item, valor_combo_item, ativo)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (combo_id, produto_item_id) DO UPDATE SET quantidade = EXCLUDED.quantidade, valor_combo_item = EXCLUDED.valor_combo_item, ativo = EXCLUDED.ativo"
    )
    .bind(id).bind(dados.combo_id).bind(dados.produto_item_id).bind(dados.quantidade)
    .bind(dados.valor_original_item).bind(dados.valor_combo_item).bind(ativo)
    .execute(pool).await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "COMBO", Some(dados.combo_id), "EDITAR", Some("ITENS"), None, Some(json!({"item_id": dados.produto_item_id})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "COMBO_ALTERADO", "COMBO", Some(dados.combo_id), json!({"combo_id": dados.combo_id})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Item de combo adicionado/atualizado", ()))).into_response()
}

// ================================================================
// Handlers de Adicionais
// ================================================================

pub async fn listar_adicionais(
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
        "SELECT id, nome, descricao, preco_adicional, ativo FROM adicionais ORDER BY nome"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let lista: Vec<AdicionalListaDto> = records.into_iter().map(|row| AdicionalListaDto {
        id: row.get("id"),
        nome: row.get("nome"),
        descricao: row.get("descricao"),
        preco_adicional: row.get("preco_adicional"),
        ativo: row.get("ativo"),
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok("Adicionais obtidos", lista))).into_response()
}

pub async fn criar_adicional(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<AdicionalInputDto>,
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
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do adicional é obrigatório.", "ERRO_PRODUTO_DESCRICAO_OBRIGATORIA", ""))).into_response();
    }
    if dados.preco_adicional < Decimal::ZERO {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Preço do adicional não pode ser negativo.", "ERRO_PRECO_INVALIDO", ""))).into_response();
    }

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO adicionais (id, nome, descricao, preco_adicional, ativo) VALUES ($1, $2, $3, $4, $5)"
    ).bind(id).bind(&dados.nome).bind(&dados.descricao).bind(dados.preco_adicional).bind(dados.ativo.unwrap_or(true))
    .execute(pool).await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "ADICIONAL", Some(id), "CRIAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "ADICIONAL_ALTERADO", "ADICIONAL", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Adicional criado", json!({"id": id})))).into_response()
}

pub async fn atualizar_adicional(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<AdicionalInputDto>,
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
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do adicional é obrigatório.", "ERRO_PRODUTO_DESCRICAO_OBRIGATORIA", ""))).into_response();
    }
    if dados.preco_adicional < Decimal::ZERO {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Preço do adicional não pode ser negativo.", "ERRO_PRECO_INVALIDO", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "UPDATE adicionais SET nome = $1, descricao = $2, preco_adicional = $3, ativo = $4, atualizado_em = NOW() WHERE id = $5"
    ).bind(&dados.nome).bind(&dados.descricao).bind(dados.preco_adicional).bind(dados.ativo.unwrap_or(true)).bind(id)
    .execute(pool).await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "ADICIONAL", Some(id), "EDITAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "ADICIONAL_ALTERADO", "ADICIONAL", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Adicional atualizado", ()))).into_response()
}

// ================================================================
// Handlers de Locais de Produção
// ================================================================

pub async fn listar_locais_producao(
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
        "SELECT id, nome, descricao, ativo FROM locais_producao ORDER BY nome"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    let lista: Vec<LocalProducaoListaDto> = records.into_iter().map(|row| LocalProducaoListaDto {
        id: row.get("id"),
        nome: row.get("nome"),
        descricao: row.get("descricao"),
        ativo: row.get("ativo"),
    }).collect();

    (StatusCode::OK, Json(RespostaBase::ok("Locais de produção obtidos", lista))).into_response()
}

pub async fn criar_local_producao(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<LocalProducaoInputDto>,
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
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do local de produção é obrigatório.", "ERRO_PRODUTO_DESCRICAO_OBRIGATORIA", ""))).into_response();
    }

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO locais_producao (id, nome, descricao, ativo) VALUES ($1, $2, $3, $4)"
    ).bind(id).bind(&dados.nome).bind(&dados.descricao).bind(dados.ativo.unwrap_or(true))
    .execute(pool).await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "LOCAL_PRODUCAO", Some(id), "CRIAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "LOCAL_PRODUCAO_ALTERADO", "LOCAL_PRODUCAO", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Local de produção criado", json!({"id": id})))).into_response()
}

pub async fn atualizar_local_producao(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<LocalProducaoInputDto>,
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
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do local é obrigatório.", "ERRO_PRODUTO_DESCRICAO_OBRIGATORIA", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "UPDATE locais_producao SET nome = $1, descricao = $2, ativo = $3, atualizado_em = NOW() WHERE id = $4"
    ).bind(&dados.nome).bind(&dados.descricao).bind(dados.ativo.unwrap_or(true)).bind(id)
    .execute(pool).await {
        return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "LOCAL_PRODUCAO", Some(id), "EDITAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "LOCAL_PRODUCAO_ALTERADO", "LOCAL_PRODUCAO", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Local de produção atualizado", ()))).into_response()
}
