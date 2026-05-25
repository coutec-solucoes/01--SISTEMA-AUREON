use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
    response::IntoResponse,
};
use aureon_core::RespostaBase;
use aureon_core::dtos::*;
use crate::app::AppState;
use crate::licenca_crypto::{
    assinar_payload, verificar_payload, chave_global,
    PayloadCanonicoLicenca, ALGORITMO,
};
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
        // Check-in (Fase 20 Bloco 3)
        .route("/check-in", post(checkin_licenca))
        .route("/validar-terminal", post(validar_terminal))
        .route("/licencas/:id/payload", get(obter_licenca_payload))
        // Assinatura Criptográfica (Fase 20 Bloco 4)
        .route("/licencas/:id/payload-assinado", get(obter_payload_assinado))
        .route("/licencas/verificar-payload", post(verificar_licenca_payload))
        .route("/chaves/status", get(status_chaves))
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

// ==========================================
// CHECK-IN E PAYLOAD (BLOCO 3)
// ==========================================
async fn checkin_licenca(
    State(_state): State<AppState>,
    Json(payload): Json<LicencaCheckInReq>,
) -> impl IntoResponse {
    // 1. Validar Empresa
    if payload.empresa_id.is_empty() {
        return Json(RespostaBase::ok("Empresa inválida", LicencaPayloadResp {
            sucesso: false,
            pode_operar: false,
            status: "BLOQUEADA".to_string(),
            modo: "MANUAL".to_string(),
            empresa_id: payload.empresa_id,
            licenca_id: None,
            plano_codigo: None,
            terminal_id: None,
            terminal_status: None,
            validade_inicio: None,
            validade_fim: None,
            tolerancia_offline_dias: 0,
            bloqueio_total: true,
            motivo_bloqueio: Some("Empresa não identificada".to_string()),
            ultimo_check_em: Some(chrono::Utc::now().to_rfc3339()),
            assinatura_licenca: None,
            payload_licenca_json: None,
            mensagem: Some("Empresa inválida".to_string()),
            warnings: vec![],
        }));
    }

    // Mocking check-in logic
    let mut warnings = vec![];
    warnings.push("Assinatura futura não implementada".to_string());

    let payload_json = serde_json::json!({
        "empresa_id": payload.empresa_id,
        "licenca_id": Uuid::new_v4().to_string(),
        "plano_codigo": "ESSENCIAL",
        "permissoes": ["PDV", "FISCAL"],
        "validade": null,
        "terminal": payload.terminal_id.clone().unwrap_or_else(|| "PENDENTE".to_string()),
        "tolerancia_offline_dias": 10,
        "emitido_em": chrono::Utc::now().to_rfc3339()
    });

    let resp = LicencaPayloadResp {
        sucesso: true,
        pode_operar: true,
        status: "ATIVA".to_string(),
        modo: "DEV".to_string(),
        empresa_id: payload.empresa_id,
        licenca_id: Some(Uuid::new_v4().to_string()),
        plano_codigo: Some("ESSENCIAL".to_string()),
        terminal_id: payload.terminal_id,
        terminal_status: Some("AUTORIZADO".to_string()),
        validade_inicio: Some(chrono::Utc::now().to_rfc3339()),
        validade_fim: None, // DEV mode
        tolerancia_offline_dias: 10,
        bloqueio_total: false,
        motivo_bloqueio: None,
        ultimo_check_em: Some(chrono::Utc::now().to_rfc3339()),
        assinatura_licenca: Some("ASSINATURA_FUTURA_NAO_IMPLEMENTADA".to_string()),
        payload_licenca_json: Some(payload_json),
        mensagem: Some("Check-in efetuado com sucesso".to_string()),
        warnings,
    };

    Json(RespostaBase::ok("Check-in aprovado", resp))
}

async fn validar_terminal(
    State(_state): State<AppState>,
    Json(payload): Json<ValidarTerminalReq>,
) -> impl IntoResponse {
    let mut warnings = vec![];
    if payload.terminal_id.is_none() {
        warnings.push("Terminal_id não fornecido, criando como PENDENTE".to_string());
    }

    let resp = ValidarTerminalResp {
        sucesso: true,
        terminal_id: Some(payload.terminal_id.unwrap_or_else(|| Uuid::new_v4().to_string())),
        status: "AUTORIZADO".to_string(),
        autorizado: true,
        mensagem: Some("Terminal autorizado no modo DEV".to_string()),
        warnings,
    };

    Json(RespostaBase::ok("Terminal validado", resp))
}

async fn obter_licenca_payload(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let payload_json = serde_json::json!({
        "empresa_id": "MOCK-EMP",
        "licenca_id": id,
        "plano_codigo": "ESSENCIAL",
        "permissoes": ["PDV", "FISCAL"],
        "validade": null,
        "terminal": "PENDENTE",
        "tolerancia_offline_dias": 10,
        "emitido_em": chrono::Utc::now().to_rfc3339()
    });

    let resp = LicencaPayloadResp {
        sucesso: true,
        pode_operar: true,
        status: "ATIVA".to_string(),
        modo: "DEV".to_string(),
        empresa_id: "MOCK-EMP".to_string(),
        licenca_id: Some(id),
        plano_codigo: Some("ESSENCIAL".to_string()),
        terminal_id: None,
        terminal_status: None,
        validade_inicio: Some(chrono::Utc::now().to_rfc3339()),
        validade_fim: None,
        tolerancia_offline_dias: 10,
        bloqueio_total: false,
        motivo_bloqueio: None,
        ultimo_check_em: Some(chrono::Utc::now().to_rfc3339()),
        assinatura_licenca: Some("ASSINATURA_FUTURA_NAO_IMPLEMENTADA".to_string()),
        payload_licenca_json: Some(payload_json),
        mensagem: Some("Payload gerado com sucesso".to_string()),
        warnings: vec!["Assinatura futura não implementada".to_string()],
    };

    Json(RespostaBase::ok("Payload obtido", resp))
}

// ==========================================
// ASSINATURA CRIPTOGRÁFICA (BLOCO 4)
// ==========================================

/// GET /licenciamento/licencas/{id}/payload-assinado
///
/// Gera um payload de licença assinado com Ed25519 real.
/// A assinatura cobre o SHA-256 do payload canônico determinístico.
/// NUNCA expõe a chave privada.
async fn obter_payload_assinado(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let emitido_em = chrono::Utc::now().to_rfc3339();

    // Construir payload canônico com campos obrigatórios fixos.
    // Ordem dos campos é determinística — não depende de serde_json.
    let payload = PayloadCanonicoLicenca {
        empresa_id: "MOCK-EMP",
        licenca_id: &id,
        plano_codigo: "ESSENCIAL",
        status: "ATIVA",
        validade: "null",
        terminal: "PENDENTE",
        tolerancia_offline_dias: 10,
        emitido_em: &emitido_em,
    };

    // Assinar com Ed25519 (chave privada fica na Retaguarda)
    let resultado = assinar_payload(&payload);

    let mut warnings = Vec::new();
    // Avisar se estiver usando chave DEV
    if resultado.is_dev {
        warnings.push(
            "ATENÇÃO: Assinatura gerada com chave DEV efêmera. \
             Configure AUREON_LICENSE_PRIVATE_KEY_B64 para produção."
                .to_string(),
        );
    }

    let resp = LicencaPayloadAssinadoResp {
        sucesso: true,
        payload_licenca_json: resultado.payload_json,
        algoritmo_assinatura: ALGORITMO.to_string(),
        key_id: resultado.key_id,
        assinatura_licenca: resultado.assinatura_b64,
        payload_hash: resultado.payload_hash_hex,
        emitido_em,
        mensagem: Some("Payload assinado com Ed25519 gerado com sucesso.".to_string()),
        warnings,
    };

    Json(RespostaBase::ok("Payload assinado gerado", resp))
}

/// POST /licenciamento/licencas/verificar-payload
///
/// Verifica se um payload de licença foi assinado com a chave desta Retaguarda.
/// Retorna valido=false se o payload foi adulterado ou a assinatura é inválida.
async fn verificar_licenca_payload(
    State(_state): State<AppState>,
    Json(req): Json<VerificarLicencaPayloadReq>,
) -> impl IntoResponse {
    // Verificar algoritmo declarado
    if req.algoritmo_assinatura != ALGORITMO {
        let algo = req.algoritmo_assinatura.clone();
        let resp = VerificarLicencaPayloadResp {
            valido: false,
            algoritmo_assinatura: req.algoritmo_assinatura,
            key_id: req.key_id,
            payload_hash: String::new(),
            mensagem: format!(
                "Algoritmo '{}' não suportado. Algoritmo esperado: '{}'.",
                algo, ALGORITMO
            ),
            warnings: vec![],
        };
        return Json(RespostaBase::ok("Verificação concluída", resp));
    }

    // Verificar assinatura com chave pública
    let resultado = verificar_payload(
        &req.payload_licenca_json,
        &req.assinatura_licenca,
        &req.key_id,
        req.payload_hash.as_deref(),
    );

    let resp = VerificarLicencaPayloadResp {
        valido: resultado.valido,
        algoritmo_assinatura: ALGORITMO.to_string(),
        key_id: req.key_id,
        payload_hash: resultado.payload_hash_calculado,
        mensagem: resultado.mensagem,
        warnings: resultado.warnings,
    };

    Json(RespostaBase::ok("Verificação concluída", resp))
}

/// GET /licenciamento/chaves/status
///
/// Retorna informações da chave pública ativa.
/// NUNCA retorna a chave privada — apenas dados públicos.
async fn status_chaves(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let chave = chave_global();
    let mut warnings = vec![];

    if chave.is_dev {
        warnings.push(
            "Modo DEV: chave efêmera gerada em runtime. Configure \
             AUREON_LICENSE_PRIVATE_KEY_B64 para produção persistente."
                .to_string(),
        );
        warnings.push(
            "Em produção, distribua a chave pública ao PDV para validação offline."
                .to_string(),
        );
    }

    let resp = StatusChavesResp {
        modo_chave: if chave.is_dev {
            "DEV".to_string()
        } else {
            "PRODUCAO".to_string()
        },
        key_id: chave.key_id.clone(),
        chave_publica_base64: chave.chave_publica_b64(),
        algoritmo: ALGORITMO.to_string(),
        warnings,
    };

    Json(RespostaBase::ok("Status das chaves de licença", resp))
}
