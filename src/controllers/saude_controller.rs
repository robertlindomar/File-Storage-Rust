use std::sync::Arc;

use axum::{Json, extract::Extension, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::EstadoAplicacao;

#[derive(Serialize)]
struct CorpoSaude {
    status: &'static str,
}

pub async fn saude() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(CorpoSaude { status: "ok" }),
    )
}

pub async fn pronto(Extension(estado): Extension<Arc<EstadoAplicacao>>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").fetch_one(&estado.pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(CorpoSaude { status: "ready" }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(CorpoSaude {
                status: "not_ready",
            }),
        )
            .into_response(),
    }
}
