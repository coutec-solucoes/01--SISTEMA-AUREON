use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use uuid::Uuid;
use chrono::Utc;
use rust_decimal::Decimal;

use crate::{app::AppState, erros::ErroApi, middleware::UsuarioLogado};
use aureon_core::RespostaBase;
use crate::routes::seguranca::tem_permissao;
use crate::routes::cadastros::pessoas::{auditar, publicar_evento};

// ================================================================
// DTOs gerais da Fase 5
// ================================================================

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct ConfiguracoesPdvDto {
    pub empresa_id: Uuid,
    pub permitir_venda_offline: bool,
    pub dias_maximos_offline: i32,
    pub exigir_cotacao_ao_abrir_caixa: bool,
    pub permitir_venda_sem_estoque: bool,
    pub bloquear_produto_vencido: bool,
    pub alertar_produto_proximo_vencer: bool,
    pub dias_alerta_vencimento: i32,
    pub permitir_desconto_pdv: bool,
    pub desconto_maximo_padrao_percentual: Decimal,
    pub exigir_supervisor_desconto_acima_limite: bool,
    pub exigir_supervisor_cancelamento_item: bool,
    pub exigir_supervisor_cancelamento_venda: bool,
    pub permitir_alterar_preco_pdv: bool,
    pub permitir_cliente_sem_cadastro: bool,
    pub exigir_cliente_completo_crediario: bool,
    pub permitir_reimpressao_comprovante: bool,
    pub exigir_supervisor_reimpressao: bool,
    pub permitir_cadastro_cliente_pdv: bool,
    pub ativo: bool,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct TerminalPdvDto {
    pub id: Uuid,
    pub codigo_terminal: String,
    pub nome_terminal: String,
    pub descricao: Option<String>,
    pub tipo_terminal: String,
    pub ip_rede_local: Option<String>,
    pub identificador_maquina_futuro: Option<String>,
    pub registradora_id: Option<Uuid>,
    pub registradora_nome: Option<String>,
    pub ativo: bool,
    pub autorizado: bool,
    pub data_autorizacao: Option<chrono::DateTime<Utc>>,
    pub ultimo_status_futuro: Option<String>,
    pub ultima_sincronizacao_futura: Option<chrono::DateTime<Utc>>,
    pub criado_em: chrono::DateTime<Utc>,
    pub atualizado_em: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct TerminalPdvInput {
    pub codigo_terminal: String,
    pub nome_terminal: String,
    pub descricao: Option<String>,
    pub tipo_terminal: String,
    pub ip_rede_local: Option<String>,
    pub identificador_maquina_futuro: Option<String>,
    pub registradora_id: Option<Uuid>,
    pub ativo: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct RegistradoraDto {
    pub id: Uuid,
    pub codigo: String,
    pub nome: String,
    pub descricao: Option<String>,
    pub tipo: String,
    pub tesouraria_id: Option<Uuid>,
    pub terminal_padrao_id: Option<Uuid>,
    pub terminal_padrao_nome: Option<String>,
    pub usuario_responsavel_id: Option<Uuid>,
    pub usuario_responsavel_nome: Option<String>,
    pub permite_multimoeda: bool,
    pub ativo: bool,
    pub criado_em: chrono::DateTime<Utc>,
    pub atualizado_em: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct RegistradoraInput {
    pub codigo: String,
    pub nome: String,
    pub descricao: Option<String>,
    pub tipo: String,
    pub tesouraria_id: Option<Uuid>,
    pub terminal_padrao_id: Option<Uuid>,
    pub usuario_responsavel_id: Option<Uuid>,
    pub permite_multimoeda: Option<bool>,
    pub ativo: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct ConfiguracoesMesasDto {
    pub id: Uuid,
    pub mesas_ativas: bool,
    pub quantidade_mesas: i32,
    pub prefixo_mesa: String,
    pub permitir_nome_informal: bool,
    pub permitir_reserva_mesa: bool,
    pub permitir_transferencia_mesa: bool,
    pub permitir_transferencia_parcial_itens: bool,
    pub imprimir_producao_por_mesa: bool,
    pub bloquear_mesa_com_pendencia: bool,
    pub ativo: bool,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct MesaDto {
    pub id: Uuid,
    pub numero: i32,
    pub nome_exibicao: Option<String>,
    pub setor: Option<String>,
    pub capacidade: i32,
    pub ativo: bool,
    pub observacao: Option<String>,
}

#[derive(Deserialize)]
pub struct MesaInput {
    pub numero: i32,
    pub nome_exibicao: Option<String>,
    pub setor: Option<String>,
    pub capacidade: Option<i32>,
    pub ativo: Option<bool>,
    pub observacao: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct ConfiguracoesComandasDto {
    pub id: Uuid,
    pub comandas_ativas: bool,
    pub faixa_inicial: i32,
    pub faixa_final: i32,
    pub permitir_nome_informal: bool,
    pub permitir_transferencia_comanda: bool,
    pub permitir_transferencia_parcial_itens: bool,
    pub imprimir_producao_por_comanda: bool,
    pub bloquear_comanda_com_pendencia: bool,
    pub ativo: bool,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct ComandaDto {
    pub id: Uuid,
    pub numero: i32,
    pub codigo_barras_qr_futuro: Option<String>,
    pub ativo: bool,
    pub observacao: Option<String>,
}

#[derive(Deserialize)]
pub struct ComandaInput {
    pub numero: i32,
    pub codigo_barras_qr_futuro: Option<String>,
    pub ativo: Option<bool>,
    pub observacao: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct ConfiguracoesPreVendasDto {
    pub id: Uuid,
    pub receber_pre_venda_pdv: bool,
    pub permitir_buscar_pre_venda_por_codigo: bool,
    pub permitir_buscar_pre_venda_por_cliente: bool,
    pub ativo: bool,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct ConfiguracoesOrcamentosDto {
    pub id: Uuid,
    pub permitir_transformar_orcamento_em_venda: bool,
    pub validade_padrao_orcamento_dias: i32,
    pub exigir_cliente_orcamento: bool,
    pub permitir_desconto_orcamento: bool,
    pub exigir_supervisor_desconto_orcamento: bool,
    pub ativo: bool,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct RegrasVendaDto {
    pub id: Uuid,
    pub permitir_venda_produto_inativo: bool,
    pub permitir_venda_estoque_negativo: bool,
    pub bloquear_venda_produto_vencido: bool,
    pub alertar_venda_produto_proximo_vencer: bool,
    pub permitir_desconto_item: bool,
    pub permitir_desconto_total: bool,
    pub desconto_maximo_item_percentual: Decimal,
    pub desconto_maximo_total_percentual: Decimal,
    pub exigir_supervisor_desconto_item: bool,
    pub exigir_supervisor_desconto_total: bool,
    pub permitir_cancelamento_item: bool,
    pub exigir_supervisor_cancelamento_item: bool,
    pub permitir_cancelamento_venda: bool,
    pub exigir_supervisor_cancelamento_venda: bool,
    pub permitir_reimpressao: bool,
    pub exigir_supervisor_reimpressao: bool,
    pub ativo: bool,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct SerieNumeracaoDto {
    pub id: Uuid,
    pub tipo_documento: String,
    pub serie: String,
    pub proximo_numero: i32,
    pub reiniciar_diariamente: bool,
    pub ativo: bool,
}

#[derive(Deserialize)]
pub struct SerieNumeracaoInput {
    pub tipo_documento: String,
    pub serie: String,
    pub proximo_numero: i32,
    pub reiniciar_diariamente: Option<bool>,
    pub ativo: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct ImpressoraDto {
    pub id: Uuid,
    pub nome: String,
    pub tipo: String,
    pub conexao: String,
    pub endereco: Option<String>,
    pub porta: Option<i32>,
    pub largura_colunas: i32,
    pub modelo_driver: Option<String>,
    pub cortar_papel: bool,
    pub abrir_gaveta: bool,
    pub ativo: bool,
    pub observacao: Option<String>,
}

#[derive(Deserialize)]
pub struct ImpressoraInput {
    pub nome: String,
    pub tipo: String,
    pub conexao: String,
    pub endereco: Option<String>,
    pub porta: Option<i32>,
    pub largura_colunas: Option<i32>,
    pub modelo_driver: Option<String>,
    pub cortar_papel: Option<bool>,
    pub abrir_gaveta: Option<bool>,
    pub ativo: Option<bool>,
    pub observacao: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct SetorProducaoDto {
    pub id: Uuid,
    pub nome: String,
    pub descricao: Option<String>,
    pub impressora_id: Option<Uuid>,
    pub impressora_nome: Option<String>,
    pub tipo_producao: String,
    pub ativo: bool,
}

#[derive(Deserialize)]
pub struct SetorProducaoInput {
    pub nome: String,
    pub descricao: Option<String>,
    pub impressora_id: Option<Uuid>,
    pub tipo_producao: String,
    pub ativo: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct BalancaDto {
    pub id: Uuid,
    pub nome: String,
    pub marca: Option<String>,
    pub modelo: Option<String>,
    pub tipo_comunicacao: String,
    pub porta_serial: Option<String>,
    pub ip: Option<String>,
    pub porta_tcp: Option<i32>,
    pub protocolo: Option<String>,
    pub ativo: bool,
    pub observacao: Option<String>,
}

#[derive(Deserialize)]
pub struct BalancaInput {
    pub nome: String,
    pub marca: Option<String>,
    pub modelo: Option<String>,
    pub tipo_comunicacao: String,
    pub porta_serial: Option<String>,
    pub ip: Option<String>,
    pub porta_tcp: Option<i32>,
    pub protocolo: Option<String>,
    pub ativo: Option<bool>,
    pub observacao: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct EtiquetaBalancaDto {
    pub id: Uuid,
    pub nome: String,
    pub prefixo: Option<String>,
    pub tamanho_codigo: i32,
    pub posicao_codigo_inicio: i32,
    pub posicao_codigo_fim: i32,
    pub posicao_peso_inicio: i32,
    pub posicao_peso_fim: i32,
    pub posicao_valor_inicio: i32,
    pub posicao_valor_fim: i32,
    pub tipo_leitura: String,
    pub casas_decimais: i32,
    pub ativo: bool,
}

#[derive(Deserialize)]
pub struct EtiquetaBalancaInput {
    pub nome: String,
    pub prefixo: Option<String>,
    pub tamanho_codigo: i32,
    pub posicao_codigo_inicio: i32,
    pub posicao_codigo_fim: i32,
    pub posicao_peso_inicio: i32,
    pub posicao_peso_fim: i32,
    pub posicao_valor_inicio: i32,
    pub posicao_valor_fim: i32,
    pub tipo_leitura: String,
    pub casas_decimais: i32,
    pub ativo: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct PerifericoDto {
    pub id: Uuid,
    pub nome: String,
    pub tipo: String,
    pub conexao: String,
    pub endereco: Option<String>,
    pub porta: Option<i32>,
    pub ativo: bool,
    pub observacao: Option<String>,
}

#[derive(Deserialize)]
pub struct PerifericoInput {
    pub nome: String,
    pub tipo: String,
    pub conexao: String,
    pub endereco: Option<String>,
    pub porta: Option<i32>,
    pub ativo: Option<bool>,
    pub observacao: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct ConfiguracoesSenhasChamadasDto {
    pub id: Uuid,
    pub senhas_ativas: bool,
    pub prefixo_senha: String,
    pub proximo_numero: i32,
    pub reiniciar_diariamente: bool,
    pub permitir_chamada_painel: bool,
    pub zerar_senha_dia_seguinte: bool,
    pub ativo: bool,
}

// ================================================================
// Handlers: Configuração PDV
// ================================================================

pub async fn obter_configuracao_pdv(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let record = match sqlx::query_as::<_, ConfiguracoesPdvDto>(
        "SELECT empresa_id, permitir_venda_offline, dias_maximos_offline, exigir_cotacao_ao_abrir_caixa, permitir_venda_sem_estoque, bloquear_produto_vencido, alertar_produto_proximo_vencer, dias_alerta_vencimento, permitir_desconto_pdv, desconto_maximo_padrao_percentual, exigir_supervisor_desconto_acima_limite, exigir_supervisor_cancelamento_item, exigir_supervisor_cancelamento_venda, permitir_alterar_preco_pdv, permitir_cliente_sem_cadastro, exigir_cliente_completo_crediario, permitir_reimpressao_comprovante, exigir_supervisor_reimpressao, permitir_cadastro_cliente_pdv, ativo 
         FROM configuracoes_pdv LIMIT 1"
    ).fetch_optional(pool).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            // Se vazio, tenta inicializar com a primeira empresa cadastrada
            let emp_id = match sqlx::query_scalar::<_, Uuid>("SELECT id FROM empresas LIMIT 1").fetch_one(pool).await {
                Ok(id) => id,
                Err(_) => return ErroApi::interno("Nenhuma empresa cadastrada para associar as configurações.").into_response(),
            };

            let _ = sqlx::query(
                "INSERT INTO configuracoes_pdv (empresa_id) VALUES ($1) ON CONFLICT DO NOTHING"
            ).bind(emp_id).execute(pool).await;

            sqlx::query_as::<_, ConfiguracoesPdvDto>(
                "SELECT * FROM configuracoes_pdv WHERE empresa_id = $1"
            ).bind(emp_id).fetch_one(pool).await.unwrap()
        },
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Configurações do PDV obtidas com sucesso", record))).into_response()
}

pub async fn salvar_configuracao_pdv(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ConfiguracoesPdvDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.dias_maximos_offline <= 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Dias máximos offline deve ser maior que zero.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    if dados.desconto_maximo_padrao_percentual < Decimal::ZERO || dados.desconto_maximo_padrao_percentual > Decimal::from(100) {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Desconto padrão deve estar entre 0% e 100%.", "ERRO_DESCONTO_INVALIDO", ""))).into_response();
    }

    let query = "
        INSERT INTO configuracoes_pdv (
            empresa_id, permitir_venda_offline, dias_maximos_offline, exigir_cotacao_ao_abrir_caixa,
            permitir_venda_sem_estoque, bloquear_produto_vencido, alertar_produto_proximo_vencer,
            dias_alerta_vencimento, permitir_desconto_pdv, desconto_maximo_padrao_percentual,
            exigir_supervisor_desconto_acima_limite, exigir_supervisor_cancelamento_item,
            exigir_supervisor_cancelamento_venda, permitir_alterar_preco_pdv, permitir_cliente_sem_cadastro,
            exigir_cliente_completo_crediario, permitir_reimpressao_comprovante, exigir_supervisor_reimpressao,
            permitir_cadastro_cliente_pdv, ativo
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
        ON CONFLICT (empresa_id) DO UPDATE SET
            permitir_venda_offline = EXCLUDED.permitir_venda_offline,
            dias_maximos_offline = EXCLUDED.dias_maximos_offline,
            exigir_cotacao_ao_abrir_caixa = EXCLUDED.exigir_cotacao_ao_abrir_caixa,
            permitir_venda_sem_estoque = EXCLUDED.permitir_venda_sem_estoque,
            bloquear_produto_vencido = EXCLUDED.bloquear_produto_vencido,
            alertar_produto_proximo_vencer = EXCLUDED.alertar_produto_proximo_vencer,
            dias_alerta_vencimento = EXCLUDED.dias_alerta_vencimento,
            permitir_desconto_pdv = EXCLUDED.permitir_desconto_pdv,
            desconto_maximo_padrao_percentual = EXCLUDED.desconto_maximo_padrao_percentual,
            exigir_supervisor_desconto_acima_limite = EXCLUDED.exigir_supervisor_desconto_acima_limite,
            exigir_supervisor_cancelamento_item = EXCLUDED.exigir_supervisor_cancelamento_item,
            exigir_supervisor_cancelamento_venda = EXCLUDED.exigir_supervisor_cancelamento_venda,
            permitir_alterar_preco_pdv = EXCLUDED.permitir_alterar_preco_pdv,
            permitir_cliente_sem_cadastro = EXCLUDED.permitir_cliente_sem_cadastro,
            exigir_cliente_completo_crediario = EXCLUDED.exigir_cliente_completo_crediario,
            permitir_reimpressao_comprovante = EXCLUDED.permitir_reimpressao_comprovante,
            exigir_supervisor_reimpressao = EXCLUDED.exigir_supervisor_reimpressao,
            permitir_cadastro_cliente_pdv = EXCLUDED.permitir_cadastro_cliente_pdv,
            ativo = EXCLUDED.ativo
    ";

    if let Err(e) = sqlx::query(query)
        .bind(dados.empresa_id)
        .bind(dados.permitir_venda_offline)
        .bind(dados.dias_maximos_offline)
        .bind(dados.exigir_cotacao_ao_abrir_caixa)
        .bind(dados.permitir_venda_sem_estoque)
        .bind(dados.bloquear_produto_vencido)
        .bind(dados.alertar_produto_proximo_vencer)
        .bind(dados.dias_alerta_vencimento)
        .bind(dados.permitir_desconto_pdv)
        .bind(dados.desconto_maximo_padrao_percentual)
        .bind(dados.exigir_supervisor_desconto_acima_limite)
        .bind(dados.exigir_supervisor_cancelamento_item)
        .bind(dados.exigir_supervisor_cancelamento_venda)
        .bind(dados.permitir_alterar_preco_pdv)
        .bind(dados.permitir_cliente_sem_cadastro)
        .bind(dados.exigir_cliente_completo_crediario)
        .bind(dados.permitir_reimpressao_comprovante)
        .bind(dados.exigir_supervisor_reimpressao)
        .bind(dados.permitir_cadastro_cliente_pdv)
        .bind(dados.ativo)
        .execute(pool).await {
            return ErroApi::interno(e.to_string()).into_response();
    }

    auditar(pool, "CONFIG_PDV", Some(dados.empresa_id), "EDITAR", None, None, Some(json!({"permitir_venda_offline": dados.permitir_venda_offline})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "CONFIG_PDV_ALTERADA", "CONFIG_PDV", Some(dados.empresa_id), json!(dados)).await;

    (StatusCode::OK, Json(RespostaBase::ok("Configurações salvas com sucesso", ()))).into_response()
}

// ================================================================
// Handlers: Terminais PDV
// ================================================================

pub async fn listar_terminais(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query_as::<_, TerminalPdvDto>(
        "SELECT t.id, t.codigo_terminal, t.nome_terminal, t.descricao, t.tipo_terminal, t.ip_rede_local,
                t.identificador_maquina_futuro, t.registradora_id, r.nome as registradora_nome, t.ativo,
                t.autorizado, t.data_autorizacao, t.ultimo_status_futuro, t.ultima_sincronizacao_futura,
                t.criado_em, t.atualizado_em
         FROM terminais_pdv t
         LEFT JOIN registradoras r ON r.id = t.registradora_id
         ORDER BY t.codigo_terminal"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Terminais obtidos com sucesso", records))).into_response()
}

pub async fn criar_terminal(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<TerminalPdvInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.codigo_terminal.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Código do terminal é obrigatório.", "ERRO_TERMINAL_CODIGO_OBRIGATORIO", ""))).into_response();
    }
    if dados.nome_terminal.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do terminal é obrigatório.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    let id = Uuid::new_v4();
    let ativo = dados.ativo.unwrap_or(true);

    if let Err(e) = sqlx::query(
        "INSERT INTO terminais_pdv (id, codigo_terminal, nome_terminal, descricao, tipo_terminal, ip_rede_local, identificador_maquina_futuro, registradora_id, ativo)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(id)
    .bind(&dados.codigo_terminal)
    .bind(&dados.nome_terminal)
    .bind(&dados.descricao)
    .bind(&dados.tipo_terminal)
    .bind(&dados.ip_rede_local)
    .bind(&dados.identificador_maquina_futuro)
    .bind(dados.registradora_id)
    .bind(ativo)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("uq_terminais_pdv_codigo") || msg.contains("codigo_terminal") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Código de terminal já cadastrado.", "ERRO_TERMINAL_DUPLICADO", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    auditar(pool, "TERMINAL", Some(id), "CRIAR", None, None, Some(json!({"nome": &dados.nome_terminal})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "TERMINAL_CRIADO", "TERMINAL", Some(id), json!({"id": id, "codigo": &dados.codigo_terminal})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Terminal cadastrado com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_terminal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<TerminalPdvInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.codigo_terminal.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Código do terminal é obrigatório.", "ERRO_TERMINAL_CODIGO_OBRIGATORIO", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "UPDATE terminais_pdv SET codigo_terminal = $1, nome_terminal = $2, descricao = $3, tipo_terminal = $4, ip_rede_local = $5, identificador_maquina_futuro = $6, registradora_id = $7, ativo = $8, atualizado_em = NOW() WHERE id = $9"
    )
    .bind(&dados.codigo_terminal)
    .bind(&dados.nome_terminal)
    .bind(&dados.descricao)
    .bind(&dados.tipo_terminal)
    .bind(&dados.ip_rede_local)
    .bind(&dados.identificador_maquina_futuro)
    .bind(dados.registradora_id)
    .bind(dados.ativo.unwrap_or(true))
    .bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("uq_terminais_pdv_codigo") || msg.contains("codigo_terminal") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Código de terminal já cadastrado.", "ERRO_TERMINAL_DUPLICADO", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    auditar(pool, "TERMINAL", Some(id), "EDITAR", None, None, Some(json!({"nome": &dados.nome_terminal})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "TERMINAL_ALTERADO", "TERMINAL", Some(id), json!({"id": id, "codigo": &dados.codigo_terminal})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Terminal atualizado com sucesso", ()))).into_response()
}

pub async fn inativar_terminal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let _ = sqlx::query("UPDATE terminais_pdv SET ativo = FALSE, atualizado_em = NOW() WHERE id = $1").bind(id).execute(pool).await;
    publicar_evento(pool, "TERMINAL_INATIVADO", "TERMINAL", Some(id), json!({"id": id})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Terminal inativado com sucesso", ()))).into_response()
}

pub async fn autorizar_terminal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let _ = sqlx::query("UPDATE terminais_pdv SET autorizado = TRUE, data_autorizacao = NOW(), atualizado_em = NOW() WHERE id = $1").bind(id).execute(pool).await;
    publicar_evento(pool, "TERMINAL_AUTORIZADO", "TERMINAL", Some(id), json!({"id": id})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Terminal autorizado com sucesso", ()))).into_response()
}

// ================================================================
// Handlers: Registradoras / Caixas
// ================================================================

pub async fn listar_registradoras(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query_as::<_, RegistradoraDto>(
        "SELECT r.id, r.codigo, r.nome, r.descricao, r.tipo, r.tesouraria_id, r.terminal_padrao_id,
                t.nome_terminal as terminal_padrao_nome, r.usuario_responsavel_id, p.nome as usuario_responsavel_nome,
                r.permite_multimoeda, r.ativo, r.criado_em, r.atualizado_em
         FROM registradoras r
         LEFT JOIN terminais_pdv t ON t.id = r.terminal_padrao_id
         LEFT JOIN pessoas p ON p.id = r.usuario_responsavel_id
         ORDER BY r.codigo"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Registradoras obtidas com sucesso", records))).into_response()
}

pub async fn criar_registradora(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<RegistradoraInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.codigo.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Código da registradora é obrigatório.", "ERRO_REGISTRADORA_CODIGO_OBRIGATORIO", ""))).into_response();
    }
    if dados.nome.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome da registradora é obrigatório.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO registradoras (id, codigo, nome, descricao, tipo, tesouraria_id, terminal_padrao_id, usuario_responsavel_id, permite_multimoeda, ativo)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    )
    .bind(id)
    .bind(&dados.codigo)
    .bind(&dados.nome)
    .bind(&dados.descricao)
    .bind(&dados.tipo)
    .bind(dados.tesouraria_id)
    .bind(dados.terminal_padrao_id)
    .bind(dados.usuario_responsavel_id)
    .bind(dados.permite_multimoeda.unwrap_or(false))
    .bind(dados.ativo.unwrap_or(true))
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("codigo") || msg.contains("registradoras_codigo_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Código de registradora duplicado.", "ERRO_REGISTRADORA_DUPLICADA", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    auditar(pool, "REGISTRADORA", Some(id), "CRIAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "REGISTRADORA_CRIADA", "REGISTRADORA", Some(id), json!({"id": id, "codigo": &dados.codigo})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Registradora cadastrada com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_registradora(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<RegistradoraInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.codigo.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Código é obrigatório.", "ERRO_REGISTRADORA_CODIGO_OBRIGATORIO", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "UPDATE registradoras SET codigo = $1, nome = $2, descricao = $3, tipo = $4, tesouraria_id = $5, terminal_padrao_id = $6, usuario_responsavel_id = $7, permite_multimoeda = $8, ativo = $9, atualizado_em = NOW() WHERE id = $10"
    )
    .bind(&dados.codigo)
    .bind(&dados.nome)
    .bind(&dados.descricao)
    .bind(&dados.tipo)
    .bind(dados.tesouraria_id)
    .bind(dados.terminal_padrao_id)
    .bind(dados.usuario_responsavel_id)
    .bind(dados.permite_multimoeda.unwrap_or(false))
    .bind(dados.ativo.unwrap_or(true))
    .bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("codigo") || msg.contains("registradoras_codigo_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Código de registradora duplicado.", "ERRO_REGISTRADORA_DUPLICADA", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    auditar(pool, "REGISTRADORA", Some(id), "EDITAR", None, None, Some(json!({"nome": &dados.nome})), Some(usuario.usuario_id)).await;
    publicar_evento(pool, "REGISTRADORA_ALTERADA", "REGISTRADORA", Some(id), json!({"id": id, "codigo": &dados.codigo})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Registradora atualizada com sucesso", ()))).into_response()
}

pub async fn inativar_registradora(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let _ = sqlx::query("UPDATE registradoras SET ativo = FALSE, atualizado_em = NOW() WHERE id = $1").bind(id).execute(pool).await;
    publicar_evento(pool, "REGISTRADORA_INATIVADA", "REGISTRADORA", Some(id), json!({"id": id})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Registradora inativada com sucesso", ()))).into_response()
}

// ================================================================
// Handlers: Mesas
// ================================================================

pub async fn obter_mesas_configuracao(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let record = match sqlx::query_as::<_, ConfiguracoesMesasDto>(
        "SELECT * FROM configuracoes_mesas LIMIT 1"
    ).fetch_optional(pool).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            let id = Uuid::new_v4();
            let _ = sqlx::query("INSERT INTO configuracoes_mesas (id) VALUES ($1)").bind(id).execute(pool).await;
            sqlx::query_as::<_, ConfiguracoesMesasDto>("SELECT * FROM configuracoes_mesas LIMIT 1").fetch_one(pool).await.unwrap()
        },
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Configuração de mesas obtida", record))).into_response()
}

pub async fn salvar_mesas_configuracao(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ConfiguracoesMesasDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.quantidade_mesas < 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Quantidade de mesas não pode ser negativa.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    let _ = sqlx::query(
        "UPDATE configuracoes_mesas SET mesas_ativas = $1, quantidade_mesas = $2, prefixo_mesa = $3, permitir_nome_informal = $4, permitir_reserva_mesa = $5, permitir_transferencia_mesa = $6, permitir_transferencia_parcial_itens = $7, imprimir_producao_por_mesa = $8, bloquear_mesa_com_pendencia = $9, ativo = $10 WHERE id = $11"
    )
    .bind(dados.mesas_ativas)
    .bind(dados.quantidade_mesas)
    .bind(&dados.prefixo_mesa)
    .bind(dados.permitir_nome_informal)
    .bind(dados.permitir_reserva_mesa)
    .bind(dados.permitir_transferencia_mesa)
    .bind(dados.permitir_transferencia_parcial_itens)
    .bind(dados.imprimir_producao_por_mesa)
    .bind(dados.bloquear_mesa_com_pendencia)
    .bind(dados.ativo)
    .bind(dados.id)
    .execute(pool).await;

    publicar_evento(pool, "MESA_CONFIG_ALTERADA", "MESA_CONFIG", Some(dados.id), json!(dados)).await;

    (StatusCode::OK, Json(RespostaBase::ok("Configurações salvas com sucesso", ()))).into_response()
}

pub async fn listar_mesas(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query_as::<_, MesaDto>(
        "SELECT * FROM mesas ORDER BY numero"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Mesas obtidas com sucesso", records))).into_response()
}

pub async fn criar_mesa(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<MesaInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.numero <= 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Número da mesa deve ser maior que zero.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    let id = Uuid::new_v4();
    let cap = dados.capacidade.unwrap_or(4);
    if cap < 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Capacidade não pode ser negativa.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "INSERT INTO mesas (id, numero, nome_exibicao, setor, capacidade, ativo, observacao)
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(id)
    .bind(dados.numero)
    .bind(&dados.nome_exibicao)
    .bind(&dados.setor)
    .bind(cap)
    .bind(dados.ativo.unwrap_or(true))
    .bind(&dados.observacao)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("numero") || msg.contains("mesas_numero_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Mesa com este número já existe.", "ERRO_MESA_DUPLICADA", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "MESA_CRIADA", "MESA", Some(id), json!({"id": id, "numero": dados.numero})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Mesa criada com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_mesa(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<MesaInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let cap = dados.capacidade.unwrap_or(4);
    if cap < 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Capacidade não pode ser negativa.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "UPDATE mesas SET numero = $1, nome_exibicao = $2, setor = $3, capacidade = $4, ativo = $5, observacao = $6 WHERE id = $7"
    )
    .bind(dados.numero)
    .bind(&dados.nome_exibicao)
    .bind(&dados.setor)
    .bind(cap)
    .bind(dados.ativo.unwrap_or(true))
    .bind(&dados.observacao)
    .bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("numero") || msg.contains("mesas_numero_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Mesa com este número já existe.", "ERRO_MESA_DUPLICADA", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "MESA_ALTERADA", "MESA", Some(id), json!({"id": id, "numero": dados.numero})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Mesa atualizada com sucesso", ()))).into_response()
}

// ================================================================
// Handlers: Comandas
// ================================================================

pub async fn obter_comandas_configuracao(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let record = match sqlx::query_as::<_, ConfiguracoesComandasDto>(
        "SELECT * FROM configuracoes_comandas LIMIT 1"
    ).fetch_optional(pool).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            let id = Uuid::new_v4();
            let _ = sqlx::query("INSERT INTO configuracoes_comandas (id) VALUES ($1)").bind(id).execute(pool).await;
            sqlx::query_as::<_, ConfiguracoesComandasDto>("SELECT * FROM configuracoes_comandas LIMIT 1").fetch_one(pool).await.unwrap()
        },
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Configuração de comandas obtida", record))).into_response()
}

pub async fn salvar_comandas_configuracao(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ConfiguracoesComandasDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.faixa_inicial > dados.faixa_final {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("A faixa inicial deve ser menor ou igual à faixa final.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    let _ = sqlx::query(
        "UPDATE configuracoes_comandas SET comandas_ativas = $1, faixa_inicial = $2, faixa_final = $3, permitir_nome_informal = $4, permitir_transferencia_comanda = $5, permitir_transferencia_parcial_itens = $6, imprimir_producao_por_comanda = $7, bloquear_comanda_com_pendencia = $8, ativo = $9 WHERE id = $10"
    )
    .bind(dados.comandas_ativas)
    .bind(dados.faixa_inicial)
    .bind(dados.faixa_final)
    .bind(dados.permitir_nome_informal)
    .bind(dados.permitir_transferencia_comanda)
    .bind(dados.permitir_transferencia_parcial_itens)
    .bind(dados.imprimir_producao_por_comanda)
    .bind(dados.bloquear_comanda_com_pendencia)
    .bind(dados.ativo)
    .bind(dados.id)
    .execute(pool).await;

    publicar_evento(pool, "COMANDA_CONFIG_ALTERADA", "COMANDA_CONFIG", Some(dados.id), json!(dados)).await;

    (StatusCode::OK, Json(RespostaBase::ok("Configurações salvas com sucesso", ()))).into_response()
}

pub async fn listar_comandas(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query_as::<_, ComandaDto>(
        "SELECT * FROM comandas ORDER BY numero"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Comandas obtidas com sucesso", records))).into_response()
}

pub async fn criar_comanda(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ComandaInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.numero <= 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Número da comanda deve ser maior que zero.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO comandas (id, numero, codigo_barras_qr_futuro, ativo, observacao)
         VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(id)
    .bind(dados.numero)
    .bind(&dados.codigo_barras_qr_futuro)
    .bind(dados.ativo.unwrap_or(true))
    .bind(&dados.observacao)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("numero") || msg.contains("comandas_numero_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Comanda com este número já existe.", "ERRO_COMANDA_DUPLICADA", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "COMANDA_CRIADA", "COMANDA", Some(id), json!({"id": id, "numero": dados.numero})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Comanda criada com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_comanda(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ComandaInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if let Err(e) = sqlx::query(
        "UPDATE comandas SET numero = $1, codigo_barras_qr_futuro = $2, ativo = $3, observacao = $4 WHERE id = $5"
    )
    .bind(dados.numero)
    .bind(&dados.codigo_barras_qr_futuro)
    .bind(dados.ativo.unwrap_or(true))
    .bind(&dados.observacao)
    .bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("numero") || msg.contains("comandas_numero_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Comanda com este número já existe.", "ERRO_COMANDA_DUPLICADA", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "COMANDA_ALTERADA", "COMANDA", Some(id), json!({"id": id, "numero": dados.numero})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Comanda atualizada com sucesso", ()))).into_response()
}

// ================================================================
// Handlers: Pré-vendas / Orçamentos
// ================================================================

pub async fn obter_prevendas(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let record = match sqlx::query_as::<_, ConfiguracoesPreVendasDto>(
        "SELECT * FROM configuracoes_pre_vendas LIMIT 1"
    ).fetch_optional(pool).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            let id = Uuid::new_v4();
            let _ = sqlx::query("INSERT INTO configuracoes_pre_vendas (id) VALUES ($1)").bind(id).execute(pool).await;
            sqlx::query_as::<_, ConfiguracoesPreVendasDto>("SELECT * FROM configuracoes_pre_vendas LIMIT 1").fetch_one(pool).await.unwrap()
        },
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Configuração de pré-vendas obtida", record))).into_response()
}

pub async fn salvar_prevendas(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ConfiguracoesPreVendasDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let _ = sqlx::query(
        "UPDATE configuracoes_pre_vendas SET receber_pre_venda_pdv = $1, permitir_buscar_pre_venda_por_codigo = $2, permitir_buscar_pre_venda_por_cliente = $3, ativo = $4 WHERE id = $5"
    )
    .bind(dados.receber_pre_venda_pdv)
    .bind(dados.permitir_buscar_pre_venda_por_codigo)
    .bind(dados.permitir_buscar_pre_venda_por_cliente)
    .bind(dados.ativo)
    .bind(dados.id)
    .execute(pool).await;

    publicar_evento(pool, "PRE_VENDA_CONFIG_ALTERADA", "PRE_VENDA_CONFIG", Some(dados.id), json!(dados)).await;

    (StatusCode::OK, Json(RespostaBase::ok("Configurações salvas com sucesso", ()))).into_response()
}

pub async fn obter_orcamentos(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let record = match sqlx::query_as::<_, ConfiguracoesOrcamentosDto>(
        "SELECT * FROM configuracoes_orcamentos LIMIT 1"
    ).fetch_optional(pool).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            let id = Uuid::new_v4();
            let _ = sqlx::query("INSERT INTO configuracoes_orcamentos (id) VALUES ($1)").bind(id).execute(pool).await;
            sqlx::query_as::<_, ConfiguracoesOrcamentosDto>("SELECT * FROM configuracoes_orcamentos LIMIT 1").fetch_one(pool).await.unwrap()
        },
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Configuração de orçamentos obtida", record))).into_response()
}

pub async fn salvar_orcamentos(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ConfiguracoesOrcamentosDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.validade_padrao_orcamento_dias <= 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Validade do orçamento deve ser maior que zero.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    let _ = sqlx::query(
        "UPDATE configuracoes_orcamentos SET permitir_transformar_orcamento_em_venda = $1, validade_padrao_orcamento_dias = $2, exigir_cliente_orcamento = $3, permitir_desconto_orcamento = $4, exigir_supervisor_desconto_orcamento = $5, ativo = $6 WHERE id = $7"
    )
    .bind(dados.permitir_transformar_orcamento_em_venda)
    .bind(dados.validade_padrao_orcamento_dias)
    .bind(dados.exigir_cliente_orcamento)
    .bind(dados.permitir_desconto_orcamento)
    .bind(dados.exigir_supervisor_desconto_orcamento)
    .bind(dados.ativo)
    .bind(dados.id)
    .execute(pool).await;

    publicar_evento(pool, "ORCAMENTO_CONFIG_ALTERADA", "ORCAMENTO_CONFIG", Some(dados.id), json!(dados)).await;

    (StatusCode::OK, Json(RespostaBase::ok("Configurações salvas com sucesso", ()))).into_response()
}

// ================================================================
// Handlers: Regras de Venda
// ================================================================

pub async fn obter_regras_venda(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let record = match sqlx::query_as::<_, RegrasVendaDto>(
        "SELECT * FROM regras_venda LIMIT 1"
    ).fetch_optional(pool).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            let id = Uuid::new_v4();
            let _ = sqlx::query("INSERT INTO regras_venda (id) VALUES ($1)").bind(id).execute(pool).await;
            sqlx::query_as::<_, RegrasVendaDto>("SELECT * FROM regras_venda LIMIT 1").fetch_one(pool).await.unwrap()
        },
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Regras de venda obtidas", record))).into_response()
}

pub async fn salvar_regras_venda(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<RegrasVendaDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.desconto_maximo_item_percentual < Decimal::ZERO || dados.desconto_maximo_item_percentual > Decimal::from(100) ||
       dados.desconto_maximo_total_percentual < Decimal::ZERO || dados.desconto_maximo_total_percentual > Decimal::from(100) {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Percentuais de desconto devem estar entre 0% e 100%.", "ERRO_DESCONTO_INVALIDO", ""))).into_response();
    }

    let _ = sqlx::query(
        "UPDATE regras_venda SET permitir_venda_produto_inativo = $1, permitir_venda_estoque_negativo = $2, bloquear_venda_produto_vencido = $3, alertar_venda_produto_proximo_vencer = $4, permitir_desconto_item = $5, permitir_desconto_total = $6, desconto_maximo_item_percentual = $7, desconto_maximo_total_percentual = $8, exigir_supervisor_desconto_item = $9, exigir_supervisor_desconto_total = $10, permitir_cancelamento_item = $11, exigir_supervisor_cancelamento_item = $12, permitir_cancelamento_venda = $13, exigir_supervisor_cancelamento_venda = $14, permitir_reimpressao = $15, exigir_supervisor_reimpressao = $16, ativo = $17 WHERE id = $18"
    )
    .bind(dados.permitir_venda_produto_inativo)
    .bind(dados.permitir_venda_estoque_negativo)
    .bind(dados.bloquear_venda_produto_vencido)
    .bind(dados.alertar_venda_produto_proximo_vencer)
    .bind(dados.permitir_desconto_item)
    .bind(dados.permitir_desconto_total)
    .bind(dados.desconto_maximo_item_percentual)
    .bind(dados.desconto_maximo_total_percentual)
    .bind(dados.exigir_supervisor_desconto_item)
    .bind(dados.exigir_supervisor_desconto_total)
    .bind(dados.permitir_cancelamento_item)
    .bind(dados.exigir_supervisor_cancelamento_item)
    .bind(dados.permitir_cancelamento_venda)
    .bind(dados.exigir_supervisor_cancelamento_venda)
    .bind(dados.permitir_reimpressao)
    .bind(dados.exigir_supervisor_reimpressao)
    .bind(dados.ativo)
    .bind(dados.id)
    .execute(pool).await;

    publicar_evento(pool, "REGRAS_VENDA_ALTERADAS", "REGRAS_VENDA", Some(dados.id), json!(dados)).await;

    (StatusCode::OK, Json(RespostaBase::ok("Regras de venda salvas com sucesso", ()))).into_response()
}

// ================================================================
// Handlers: Séries e Numeração
// ================================================================

pub async fn listar_series_numeracao(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query_as::<_, SerieNumeracaoDto>(
        "SELECT * FROM series_numeracao ORDER BY tipo_documento, serie"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Séries obtidas com sucesso", records))).into_response()
}

pub async fn criar_serie_numeracao(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<SerieNumeracaoInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.tipo_documento.trim().is_empty() || dados.serie.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Tipo de documento e série são obrigatórios.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }
    if dados.proximo_numero <= 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("O próximo número deve ser maior que zero.", "ERRO_NUMERACAO_INVALIDA", ""))).into_response();
    }

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO series_numeracao (id, tipo_documento, serie, proximo_numero, reiniciar_diariamente, ativo)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(id)
    .bind(&dados.tipo_documento)
    .bind(&dados.serie)
    .bind(dados.proximo_numero)
    .bind(dados.reiniciar_diariamente.unwrap_or(false))
    .bind(dados.ativo.unwrap_or(true))
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("uq_documento_serie") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Esta combinação de tipo de documento e série já existe.", "ERRO_NUMERACAO_INVALIDA", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "SERIE_NUMERACAO_CRIADA", "SERIE_NUMERACAO", Some(id), json!({"id": id, "serie": &dados.serie})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Série cadastrada com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_serie_numeracao(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<SerieNumeracaoInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.proximo_numero <= 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("O próximo número deve ser maior que zero.", "ERRO_NUMERACAO_INVALIDA", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "UPDATE series_numeracao SET tipo_documento = $1, serie = $2, proximo_numero = $3, reiniciar_diariamente = $4, ativo = $5 WHERE id = $6"
    )
    .bind(&dados.tipo_documento)
    .bind(&dados.serie)
    .bind(dados.proximo_numero)
    .bind(dados.reiniciar_diariamente.unwrap_or(false))
    .bind(dados.ativo.unwrap_or(true))
    .bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("uq_documento_serie") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Esta combinação de tipo de documento e série já existe.", "ERRO_NUMERACAO_INVALIDA", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "SERIE_NUMERACAO_ALTERADA", "SERIE_NUMERACAO", Some(id), json!({"id": id, "serie": &dados.serie})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Série atualizada com sucesso", ()))).into_response()
}

// ================================================================
// Handlers: Impressoras
// ================================================================

pub async fn listar_impressoras(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query_as::<_, ImpressoraDto>(
        "SELECT * FROM impressoras ORDER BY nome"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Impressoras obtidas com sucesso", records))).into_response()
}

pub async fn criar_impressora(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ImpressoraInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.nome.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome da impressora é obrigatório.", "ERRO_IMPRESSORA_NOME_OBRIGATORIO", ""))).into_response();
    }

    let id = Uuid::new_v4();
    let colunas = dados.largura_colunas.unwrap_or(48);
    if colunas <= 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Largura das colunas deve ser maior que zero.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "INSERT INTO impressoras (id, nome, tipo, conexao, endereco, porta, largura_colunas, modelo_driver, cortar_papel, abrir_gaveta, ativo, observacao)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
    )
    .bind(id)
    .bind(&dados.nome)
    .bind(&dados.tipo)
    .bind(&dados.conexao)
    .bind(&dados.endereco)
    .bind(dados.porta)
    .bind(colunas)
    .bind(&dados.modelo_driver)
    .bind(dados.cortar_papel.unwrap_or(true))
    .bind(dados.abrir_gaveta.unwrap_or(false))
    .bind(dados.ativo.unwrap_or(true))
    .bind(&dados.observacao)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("nome") || msg.contains("impressoras_nome_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Já existe uma impressora cadastrada com este nome.", "ERRO_IMPRESSORA_NOME_OBRIGATORIO", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "IMPRESSORA_CRIADA", "IMPRESSORA", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Impressora cadastrada com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_impressora(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ImpressoraInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let colunas = dados.largura_colunas.unwrap_or(48);
    if colunas <= 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Largura das colunas deve ser maior que zero.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "UPDATE impressoras SET nome = $1, tipo = $2, conexao = $3, endereco = $4, porta = $5, largura_colunas = $6, modelo_driver = $7, cortar_papel = $8, abrir_gaveta = $9, ativo = $10, observacao = $11 WHERE id = $12"
    )
    .bind(&dados.nome)
    .bind(&dados.tipo)
    .bind(&dados.conexao)
    .bind(&dados.endereco)
    .bind(dados.porta)
    .bind(colunas)
    .bind(&dados.modelo_driver)
    .bind(dados.cortar_papel.unwrap_or(true))
    .bind(dados.abrir_gaveta.unwrap_or(false))
    .bind(dados.ativo.unwrap_or(true))
    .bind(&dados.observacao)
    .bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("nome") || msg.contains("impressoras_nome_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Já existe uma impressora cadastrada com este nome.", "ERRO_IMPRESSORA_NOME_OBRIGATORIO", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "IMPRESSORA_ALTERADA", "IMPRESSORA", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Impressora atualizada com sucesso", ()))).into_response()
}

// ================================================================
// Handlers: Setores de Produção
// ================================================================

pub async fn listar_setores_producao(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query_as::<_, SetorProducaoDto>(
        "SELECT s.id, s.nome, s.descricao, s.impressora_id, i.nome as impressora_nome, s.tipo_producao, s.ativo
         FROM setores_producao s
         LEFT JOIN impressoras i ON i.id = s.impressora_id
         ORDER BY s.nome"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Setores obtidos com sucesso", records))).into_response()
}

pub async fn criar_setor_producao(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<SetorProducaoInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.nome.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do setor é obrigatório.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO setores_producao (id, nome, descricao, impressora_id, tipo_producao, ativo)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(id)
    .bind(&dados.nome)
    .bind(&dados.descricao)
    .bind(dados.impressora_id)
    .bind(&dados.tipo_producao)
    .bind(dados.ativo.unwrap_or(true))
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("nome") || msg.contains("setores_producao_nome_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Já existe um setor de produção com este nome.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "SETOR_PRODUCAO_CRIADO", "SETOR_PRODUCAO", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Setor de produção cadastrado", json!({"id": id})))).into_response()
}

pub async fn atualizar_setor_producao(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<SetorProducaoInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if let Err(e) = sqlx::query(
        "UPDATE setores_producao SET nome = $1, descricao = $2, impressora_id = $3, tipo_producao = $4, ativo = $5 WHERE id = $6"
    )
    .bind(&dados.nome)
    .bind(&dados.descricao)
    .bind(dados.impressora_id)
    .bind(&dados.tipo_producao)
    .bind(dados.ativo.unwrap_or(true))
    .bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("nome") || msg.contains("setores_producao_nome_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Já existe um setor de produção com este nome.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "SETOR_PRODUCAO_ALTERADO", "SETOR_PRODUCAO", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Setor de produção atualizado", ()))).into_response()
}

// ================================================================
// Handlers: Balanças
// ================================================================

pub async fn listar_balancas(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query_as::<_, BalancaDto>(
        "SELECT * FROM balancas ORDER BY nome"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Balanças obtidas com sucesso", records))).into_response()
}

pub async fn criar_balanca(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<BalancaInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.nome.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome da balança é obrigatório.", "ERRO_BALANCA_NOME_OBRIGATORIO", ""))).into_response();
    }

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO balancas (id, nome, marca, modelo, tipo_comunicacao, porta_serial, ip, porta_tcp, protocolo, ativo, observacao)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
    )
    .bind(id)
    .bind(&dados.nome)
    .bind(&dados.marca)
    .bind(&dados.modelo)
    .bind(&dados.tipo_comunicacao)
    .bind(&dados.porta_serial)
    .bind(&dados.ip)
    .bind(dados.porta_tcp)
    .bind(&dados.protocolo)
    .bind(dados.ativo.unwrap_or(true))
    .bind(&dados.observacao)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("nome") || msg.contains("balancas_nome_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Já existe uma balança cadastrada com este nome.", "ERRO_BALANCA_NOME_OBRIGATORIO", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "BALANCA_CRIADA", "BALANCA", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Balança cadastrada com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_balanca(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<BalancaInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if let Err(e) = sqlx::query(
        "UPDATE balancas SET nome = $1, marca = $2, modelo = $3, tipo_comunicacao = $4, porta_serial = $5, ip = $6, porta_tcp = $7, protocolo = $8, ativo = $9, observacao = $10 WHERE id = $11"
    )
    .bind(&dados.nome)
    .bind(&dados.marca)
    .bind(&dados.modelo)
    .bind(&dados.tipo_comunicacao)
    .bind(&dados.porta_serial)
    .bind(&dados.ip)
    .bind(dados.porta_tcp)
    .bind(&dados.protocolo)
    .bind(dados.ativo.unwrap_or(true))
    .bind(&dados.observacao)
    .bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("nome") || msg.contains("balancas_nome_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Já existe uma balança cadastrada com este nome.", "ERRO_BALANCA_NOME_OBRIGATORIO", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "BALANCA_ALTERADA", "BALANCA", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Balança atualizada com sucesso", ()))).into_response()
}

// ================================================================
// Handlers: Etiquetas de Balança
// ================================================================

pub async fn listar_etiquetas_balanca(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query_as::<_, EtiquetaBalancaDto>(
        "SELECT * FROM etiquetas_balanca ORDER BY nome"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Layouts obtidos com sucesso", records))).into_response()
}

pub async fn criar_etiqueta_balanca(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<EtiquetaBalancaInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.nome.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome do layout é obrigatório.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }
    if dados.posicao_codigo_fim < dados.posicao_codigo_inicio ||
       dados.posicao_peso_fim < dados.posicao_peso_inicio ||
       dados.posicao_valor_fim < dados.posicao_valor_inicio {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Posições finais devem ser maiores ou iguais às posições iniciais.", "ERRO_ETIQUETA_POSICAO_INVALIDA", ""))).into_response();
    }

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO etiquetas_balanca (id, nome, prefixo, tamanho_codigo, posicao_codigo_inicio, posicao_codigo_fim, posicao_peso_inicio, posicao_peso_fim, posicao_valor_inicio, posicao_valor_fim, tipo_leitura, casas_decimais, ativo)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"
    )
    .bind(id)
    .bind(&dados.nome)
    .bind(&dados.prefixo)
    .bind(dados.tamanho_codigo)
    .bind(dados.posicao_codigo_inicio)
    .bind(dados.posicao_codigo_fim)
    .bind(dados.posicao_peso_inicio)
    .bind(dados.posicao_peso_fim)
    .bind(dados.posicao_valor_inicio)
    .bind(dados.posicao_valor_fim)
    .bind(&dados.tipo_leitura)
    .bind(dados.casas_decimais)
    .bind(dados.ativo.unwrap_or(true))
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("nome") || msg.contains("etiquetas_balanca_nome_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Já existe um layout com este nome.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "ETIQUETA_BALANCA_CRIADA", "ETIQUETA_BALANCA", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Layout de etiqueta cadastrado", json!({"id": id})))).into_response()
}

pub async fn atualizar_etiqueta_balanca(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<EtiquetaBalancaInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.posicao_codigo_fim < dados.posicao_codigo_inicio ||
       dados.posicao_peso_fim < dados.posicao_peso_inicio ||
       dados.posicao_valor_fim < dados.posicao_valor_inicio {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Posições inválidas no layout.", "ERRO_ETIQUETA_POSICAO_INVALIDA", ""))).into_response();
    }

    if let Err(e) = sqlx::query(
        "UPDATE etiquetas_balanca SET nome = $1, prefixo = $2, tamanho_codigo = $3, posicao_codigo_inicio = $4, posicao_codigo_fim = $5, posicao_peso_inicio = $6, posicao_peso_fim = $7, posicao_valor_inicio = $8, posicao_valor_fim = $9, tipo_leitura = $10, casas_decimais = $11, ativo = $12 WHERE id = $13"
    )
    .bind(&dados.nome)
    .bind(&dados.prefixo)
    .bind(dados.tamanho_codigo)
    .bind(dados.posicao_codigo_inicio)
    .bind(dados.posicao_codigo_fim)
    .bind(dados.posicao_peso_inicio)
    .bind(dados.posicao_peso_fim)
    .bind(dados.posicao_valor_inicio)
    .bind(dados.posicao_valor_fim)
    .bind(&dados.tipo_leitura)
    .bind(dados.casas_decimais)
    .bind(dados.ativo.unwrap_or(true))
    .bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("nome") || msg.contains("etiquetas_balanca_nome_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Já existe um layout com este nome.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "ETIQUETA_BALANCA_ALTERADA", "ETIQUETA_BALANCA", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Layout de etiqueta atualizado", ()))).into_response()
}

// ================================================================
// Handlers: Periféricos
// ================================================================

pub async fn listar_perifericos(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let records = match sqlx::query_as::<_, PerifericoDto>(
        "SELECT * FROM perifericos ORDER BY nome"
    ).fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Periféricos obtidos com sucesso", records))).into_response()
}

pub async fn criar_periferico(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<PerifericoInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "CRIAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.nome.trim().is_empty() || dados.tipo.trim().is_empty() || dados.conexao.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome, tipo e conexão são obrigatórios.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    let id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        "INSERT INTO perifericos (id, nome, tipo, conexao, endereco, porta, ativo, observacao)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(id)
    .bind(&dados.nome)
    .bind(&dados.tipo)
    .bind(&dados.conexao)
    .bind(&dados.endereco)
    .bind(dados.porta)
    .bind(dados.ativo.unwrap_or(true))
    .bind(&dados.observacao)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("nome") || msg.contains("perifericos_nome_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Já existe um periférico cadastrado com este nome.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "PERIFERICO_CRIADO", "PERIFERICO", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::CREATED, Json(RespostaBase::ok("Periférico cadastrado com sucesso", json!({"id": id})))).into_response()
}

pub async fn atualizar_periferico(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<PerifericoInput>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if let Err(e) = sqlx::query(
        "UPDATE perifericos SET nome = $1, tipo = $2, conexao = $3, endereco = $4, porta = $5, ativo = $6, observacao = $7 WHERE id = $8"
    )
    .bind(&dados.nome)
    .bind(&dados.tipo)
    .bind(&dados.conexao)
    .bind(&dados.endereco)
    .bind(dados.porta)
    .bind(dados.ativo.unwrap_or(true))
    .bind(&dados.observacao)
    .bind(id)
    .execute(pool).await {
        let msg = e.to_string();
        if msg.contains("nome") || msg.contains("perifericos_nome_key") {
            return (StatusCode::CONFLICT, Json(RespostaBase::<()>::falha_manual("Já existe um periférico cadastrado com este nome.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
        }
        return ErroApi::interno(msg).into_response();
    }

    publicar_evento(pool, "PERIFERICO_ALTERADO", "PERIFERICO", Some(id), json!({"id": id, "nome": &dados.nome})).await;

    (StatusCode::OK, Json(RespostaBase::ok("Periférico atualizado com sucesso", ()))).into_response()
}

// ================================================================
// Handlers: Senhas / Chamadas
// ================================================================

pub async fn obter_senhas_chamadas(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "LER").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    let record = match sqlx::query_as::<_, ConfiguracoesSenhasChamadasDto>(
        "SELECT * FROM configuracoes_senhas_chamadas LIMIT 1"
    ).fetch_optional(pool).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            let id = Uuid::new_v4();
            let _ = sqlx::query("INSERT INTO configuracoes_senhas_chamadas (id) VALUES ($1)").bind(id).execute(pool).await;
            sqlx::query_as::<_, ConfiguracoesSenhasChamadasDto>("SELECT * FROM configuracoes_senhas_chamadas LIMIT 1").fetch_one(pool).await.unwrap()
        },
        Err(e) => return ErroApi::interno(e.to_string()).into_response(),
    };

    (StatusCode::OK, Json(RespostaBase::ok("Configurações de senhas obtidas", record))).into_response()
}

pub async fn salvar_senhas_chamadas(
    State(state): State<AppState>,
    axum::extract::Extension(usuario): axum::extract::Extension<UsuarioLogado>,
    Json(dados): Json<ConfiguracoesSenhasChamadasDto>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => return ErroApi::indisponivel("Banco não configurado.").into_response(),
    };

    match tem_permissao(pool, &usuario, "CONFIGURACOES_OPERACIONAIS", "EDITAR").await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, Json(RespostaBase::<()>::falha_manual("Acesso negado.", "ERRO_SEM_PERMISSAO", ""))).into_response(),
        Err(e) => return e.into_response(),
    };

    if dados.proximo_numero <= 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Próximo número deve ser maior que zero.", "ERRO_CONFIGURACAO_OPERACIONAL", ""))).into_response();
    }

    let _ = sqlx::query(
        "UPDATE configuracoes_senhas_chamadas SET senhas_ativas = $1, prefixo_senha = $2, proximo_numero = $3, reiniciar_diariamente = $4, permitir_chamada_painel = $5, zerar_senha_dia_seguinte = $6, ativo = $7 WHERE id = $8"
    )
    .bind(dados.senhas_ativas)
    .bind(&dados.prefixo_senha)
    .bind(dados.proximo_numero)
    .bind(dados.reiniciar_diariamente)
    .bind(dados.permitir_chamada_painel)
    .bind(dados.zerar_senha_dia_seguinte)
    .bind(dados.ativo)
    .bind(dados.id)
    .execute(pool).await;

    publicar_evento(pool, "SENHA_CHAMADA_CONFIG_ALTERADA", "SENHAS_CONFIG", Some(dados.id), json!(dados)).await;

    (StatusCode::OK, Json(RespostaBase::ok("Configurações de senhas salvas", ()))).into_response()
}
