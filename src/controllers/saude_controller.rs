use std::sync::Arc;

use axum::{
    Json,
    extract::Extension,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde::Serialize;

use crate::EstadoAplicacao;

#[derive(Serialize)]
struct CorpoSaude {
    status: &'static str,
}

const PAGINA_INICIO: &str = r#"<!DOCTYPE html>
<html lang="pt">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>file_storage_service</title>
  <style>
    body { font-family: system-ui, sans-serif; max-width: 40rem; margin: 2rem auto; padding: 0 1rem; line-height: 1.5; }
    code { background: #f0f0f0; padding: 0.15rem 0.4rem; border-radius: 4px; }
    ul { padding-left: 1.2rem; }
    a { color: #0b57d0; }
  </style>
</head>
<body>
  <h1>file_storage_service</h1>
  <p>API REST — não há interface web completa; a raiz só mostra esta página de ajuda.</p>
  <ul>
    <li><a href="/health"><code>/health</code></a> — processo OK</li>
    <li><a href="/ready"><code>/ready</code></a> — base de dados OK</li>
    <li><code>/api/v1/…</code> — upload e gestão (com <code>Authorization</code>)</li>
  </ul>
</body>
</html>"#;

/// Página simples em <code>/</code> para quem abre o URL no browser.
pub async fn raiz() -> Html<&'static str> {
    Html(PAGINA_INICIO)
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
