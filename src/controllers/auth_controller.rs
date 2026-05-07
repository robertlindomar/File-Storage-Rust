use std::sync::Arc;

use axum::{Json, extract::Extension, http::StatusCode};

use crate::{
    EstadoAplicacao,
    dtos::auth_dto::{LoginAdminPedidoDto, LoginAdminRespostaDto},
    erros::ErroAplicacao,
    services::auth_admin_service,
};

pub async fn login_admin(
    Extension(estado): Extension<Arc<EstadoAplicacao>>,
    Json(corpo): Json<LoginAdminPedidoDto>,
) -> Result<(StatusCode, Json<LoginAdminRespostaDto>), ErroAplicacao> {
    let resposta = auth_admin_service::login_admin(
        &estado.pool,
        estado.configuracao.as_ref(),
        &corpo.email,
        &corpo.password,
    )
    .await?;
    Ok((StatusCode::OK, Json(resposta)))
}
