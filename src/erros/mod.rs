use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug)]
pub enum ErroAplicacao {
    RequisicaoInvalida(String),
    NaoAutorizado,
    Proibido,
    NaoEncontrado(String),
    PayloadDemasiadoGrande(String),
    Interno(String),
}

#[derive(Serialize)]
struct CorpoErro {
    erro: String,
}

impl IntoResponse for ErroAplicacao {
    fn into_response(self) -> Response {
        let (status, mensagem) = match self {
            Self::RequisicaoInvalida(mensagem) => (StatusCode::BAD_REQUEST, mensagem),
            Self::NaoAutorizado => (
                StatusCode::UNAUTHORIZED,
                "Acesso negado: credencial invalida".to_string(),
            ),
            Self::Proibido => (StatusCode::FORBIDDEN, "Operacao nao permitida".to_string()),
            Self::NaoEncontrado(mensagem) => (StatusCode::NOT_FOUND, mensagem),
            Self::PayloadDemasiadoGrande(mensagem) => (StatusCode::PAYLOAD_TOO_LARGE, mensagem),
            Self::Interno(mensagem) => (StatusCode::INTERNAL_SERVER_ERROR, mensagem),
        };

        (status, Json(CorpoErro { erro: mensagem })).into_response()
    }
}
