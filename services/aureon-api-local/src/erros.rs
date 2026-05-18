use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use aureon_core::RespostaBase;

/// Erro tipado da API para conversão automática em resposta HTTP
#[derive(Debug)]
pub struct ErroApi {
    pub status:  StatusCode,
    pub codigo:  String,
    pub detalhe: String,
}

impl ErroApi {
    pub fn interno(detalhe: impl Into<String>) -> Self {
        Self {
            status:  StatusCode::INTERNAL_SERVER_ERROR,
            codigo:  "ERRO_INTERNO".to_string(),
            detalhe: detalhe.into(),
        }
    }

    pub fn indisponivel(detalhe: impl Into<String>) -> Self {
        Self {
            status:  StatusCode::SERVICE_UNAVAILABLE,
            codigo:  "SERVICO_INDISPONIVEL".to_string(),
            detalhe: detalhe.into(),
        }
    }
}

impl IntoResponse for ErroApi {
    fn into_response(self) -> Response {
        let corpo: RespostaBase<serde_json::Value> = RespostaBase::falha_manual(
            "Não foi possível executar a operação.",
            &self.codigo,
            &self.detalhe,
        );
        (self.status, Json(corpo)).into_response()
    }
}
