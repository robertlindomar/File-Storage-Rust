use std::sync::Arc;

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

use crate::{
    EstadoAplicacao,
    erros::ErroAplicacao,
    services::auth_admin_service::autorizacao_admin_valida,
};

/// JWT de admin valido ou `API_KEY_ADMIN` (compatibilidade com scripts).
pub async fn camada_admin(
    requisicao: Request,
    seguinte: Next,
) -> Result<Response, ErroAplicacao> {
    let estado = requisicao
        .extensions()
        .get::<Arc<EstadoAplicacao>>()
        .cloned()
        .ok_or_else(|| {
            ErroAplicacao::Interno("Estado da aplicacao nao injetado (extension)".to_string())
        })?;

    let chave = extrair_token(&requisicao)?;
    autorizacao_admin_valida(estado.configuracao.as_ref(), &chave)?;
    Ok(seguinte.run(requisicao).await)
}

fn extrair_token(requisicao: &Request) -> Result<String, ErroAplicacao> {
    let cabecalho = requisicao
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|valor| valor.to_str().ok())
        .ok_or(ErroAplicacao::NaoAutorizado)?;

    if let Some(resto) = cabecalho.strip_prefix("Bearer ") {
        return Ok(resto.trim().to_string());
    }

    Ok(cabecalho.trim().to_string())
}
