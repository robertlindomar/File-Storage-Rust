use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginAdminPedidoDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginAdminRespostaDto {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
}
