use std::sync::Arc;

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

use crate::{EstadoAplicacao, erros::ErroAplicacao};

/// Valida `Authorization` (Bearer ou valor cru), resolve projeto na base e injeta [`ProjetoAutenticado`].
/// Requer [`AddExtensionLayer`](tower_http::add_extension::AddExtensionLayer) com `Arc<EstadoAplicacao>` antes desta camada.
pub async fn camada_projeto(
    mut requisicao: Request,
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
    let projeto = estado
        .servico_projeto
        .obter_por_api_key(&chave)
        .await?
        .ok_or(ErroAplicacao::NaoAutorizado)?;

    requisicao.extensions_mut().insert(projeto);
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
