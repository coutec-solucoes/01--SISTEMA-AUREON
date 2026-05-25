use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
    response::IntoResponse,
};
use aureon_core::RespostaBase;
use aureon_core::dtos::*;
use crate::app::AppState;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        // Planos
        .route("/planos", get(listar_planos).post(criar_plano))
        .route("/planos/:id", put(atualizar_plano))
        .route("/planos/:id/inativar", put(inativar_plano))
        // Empresas
        .route("/empresas", get(listar_empresas).post(criar_empresa))
        .route("/empresas/:id", get(obter_empresa))
        .route("/empresas/:id/status", put(atualizar_status_empresa))
        // Licencas
        .route("/licencas", get(listar_licencas).post(criar_licenca))
        .route("/licencas/:id", get(obter_licenca))
        .route("/licencas/:id/bloquear", put(bloquear_licenca))
        .route("/licencas/:id/reativar", put(reativar_licenca))
        // Terminais
        .route("/licencas/:id/terminais", get(listar_terminais).post(autorizar_terminal))
        .route("/terminais/:id/bloquear", put(bloquear_terminal))
        .route("/terminais/:id/autorizar", put(reativar_terminal))
        // Eventos
        .route("/eventos", get(listar_eventos))
}

// ==========================================
// PLANOS
// ==========================================
async fn listar_planos(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(RespostaBase::ok("Planos", Vec::<LicPlanoDto>::new()))
}

async fn criar_plano(
    State(_state): State<AppState>,
    Json(payload): Json<CriarLicPlanoReq>,
) -> impl IntoResponse {
    let dto = LicPlanoDto {
        id: Uuid::new_v4().to_string(),
        codigo: payload.codigo,
        nome: payload.nome,
        descricao: payload.descricao,
        max_empresas: payload.max_empresas,
        max_terminais: std::cmp::max(1, payload.max_terminais),
        permite_pdv: payload.permite_pdv,
        permite_retaguarda: payload.permite_retaguarda,
        permite_delivery: payload.permite_delivery,
        permite_gourmet: payload.permite_gourmet,
        permite_fiscal: payload.permite_fiscal,
        ativo: true,
    };
    Json(RespostaBase::ok("Plano criado", dto))
}

async fn atualizar_plano(
    Path(id): Path<String>,
    State(_state): State<AppState>,
    Json(payload): Json<CriarLicPlanoReq>,
) -> impl IntoResponse {
    let dto = LicPlanoDto {
        id,
        codigo: payload.codigo,
        nome: payload.nome,
        descricao: payload.descricao,
        max_empresas: payload.max_empresas,
        max_terminais: std::cmp::max(1, payload.max_terminais),
        permite_pdv: payload.permite_pdv,
        permite_retaguarda: payload.permite_retaguarda,
        permite_delivery: payload.permite_delivery,
        permite_gourmet: payload.permite_gourmet,
        permite_fiscal: payload.permite_fiscal,
        ativo: true,
    };
    Json(RespostaBase::ok("Plano atualizado", dto))
}

async fn inativar_plano(
    Path(_id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(RespostaBase::ok("Plano inativado", true))
}

// ==========================================
// EMPRESAS
// ==========================================
async fn listar_empresas(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(RespostaBase::ok("Empresas", Vec::<LicEmpresaDto>::new()))
}

async fn obter_empresa(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let dto = LicEmpresaDto {
        id,
        empresa_id: "emp-123".to_string(),
        nome_empresa: "Empresa Mock".to_string(),
        documento: None,
        pais: None,
        status: "ATIVA".to_string(),
        plano_id: None,
    };
    Json(RespostaBase::ok("Empresa encontrada", dto))
}

async fn criar_empresa(
    State(_state): State<AppState>,
    Json(payload): Json<CriarLicEmpresaReq>,
) -> impl IntoResponse {
    let dto = LicEmpresaDto {
        id: Uuid::new_v4().to_string(),
        empresa_id: payload.empresa_id,
        nome_empresa: payload.nome_empresa,
        documento: payload.documento,
        pais: payload.pais,
        status: payload.status,
        plano_id: payload.plano_id,
    };
    Json(RespostaBase::ok("Empresa criada", dto))
}

async fn atualizar_status_empresa(
    Path(id): Path<String>,
    State(_state): State<AppState>,
    Json(payload): Json<AtualizarStatusReq>,
) -> impl IntoResponse {
    Json(RespostaBase::ok(&format!("Empresa {} atualizada para {}", id, payload.status), true))
}

// ==========================================
// LICENCAS
// ==========================================
async fn listar_licencas(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(RespostaBase::ok("Licencas", Vec::<LicLicencaDto>::new()))
}

async fn obter_licenca(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let dto = LicLicencaDto {
        id,
        empresa_licenciada_id: Uuid::new_v4().to_string(),
        plano_id: Uuid::new_v4().to_string(),
        status: "ATIVA".to_string(),
        modo: "DEV".to_string(),
        validade_inicio: None,
        validade_fim: None,
        tolerancia_offline_dias: 10,
        bloqueio_total: false,
        motivo_bloqueio: None,
    };
    Json(RespostaBase::ok("Licenca encontrada", dto))
}

async fn criar_licenca(
    State(_state): State<AppState>,
    Json(payload): Json<CriarLicLicencaReq>,
) -> impl IntoResponse {
    let dto = LicLicencaDto {
        id: Uuid::new_v4().to_string(),
        empresa_licenciada_id: payload.empresa_licenciada_id,
        plano_id: payload.plano_id,
        status: "ATIVA".to_string(),
        modo: payload.modo,
        validade_inicio: None,
        validade_fim: payload.validade_fim,
        tolerancia_offline_dias: payload.tolerancia_offline_dias,
        bloqueio_total: false,
        motivo_bloqueio: None,
    };
    Json(RespostaBase::ok("Licenca criada", dto))
}

async fn bloquear_licenca(
    Path(_id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(RespostaBase::ok("Licenca bloqueada", true))
}

async fn reativar_licenca(
    Path(_id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(RespostaBase::ok("Licenca reativada", true))
}

// ==========================================
// TERMINAIS
// ==========================================
async fn listar_terminais(
    Path(_id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(RespostaBase::ok("Terminais", Vec::<LicTerminalDto>::new()))
}

async fn autorizar_terminal(
    Path(id): Path<String>,
    State(_state): State<AppState>,
    Json(payload): Json<AutorizarTerminalReq>,
) -> impl IntoResponse {
    let dto = LicTerminalDto {
        id: Uuid::new_v4().to_string(),
        licenca_id: id,
        installation_id: payload.installation_id,
        terminal_id: payload.terminal_id,
        terminal_nome: payload.terminal_nome,
        status: "AUTORIZADO".to_string(),
    };
    Json(RespostaBase::ok("Terminal autorizado", dto))
}

async fn bloquear_terminal(
    Path(_id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(RespostaBase::ok("Terminal bloqueado", true))
}

async fn reativar_terminal(
    Path(_id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(RespostaBase::ok("Terminal reativado", true))
}

// ==========================================
// EVENTOS
// ==========================================
async fn listar_eventos(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(RespostaBase::ok("Eventos", Vec::<LicEventoDto>::new()))
}
