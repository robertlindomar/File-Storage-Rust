use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng,
    },
};
use chrono::Duration;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    config::ConfiguracaoAplicacao,
    dtos::auth_dto::LoginAdminRespostaDto,
    erros::ErroAplicacao,
    repositories::admin_repository::RepositorioAdmin,
};

const PAPEL_ADMIN: &str = "admin";

#[derive(Debug, Serialize, Deserialize)]
struct ClaimsAdmin {
    sub: String,
    role: String,
    exp: i64,
}

fn hash_senha(plain: &str) -> Result<String, ErroAplicacao> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|erro| ErroAplicacao::Interno(format!("Falha ao gerar hash de senha: {erro}")))?
        .to_string())
}

fn verificar_senha(hash_armazenado: &str, plain: &str) -> bool {
    let Ok(analisado) = PasswordHash::new(hash_armazenado) else {
        return false;
    };
    Argon2::default()
        .verify_password(plain.as_bytes(), &analisado)
        .is_ok()
}

/// Aceita JWT de admin valido ou, em alternativa, o valor de `API_KEY_ADMIN` (compat. scripts).
pub fn autorizacao_admin_valida(
    configuracao: &ConfiguracaoAplicacao,
    token: &str,
) -> Result<(), ErroAplicacao> {
    if token == configuracao.api_key_admin {
        return Ok(());
    }
    let dados = decodificar_jwt_admin(token, configuracao).map_err(|_| ErroAplicacao::NaoAutorizado)?;
    if dados.role != PAPEL_ADMIN {
        return Err(ErroAplicacao::NaoAutorizado);
    }
    Ok(())
}

fn decodificar_jwt_admin(
    token: &str,
    configuracao: &ConfiguracaoAplicacao,
) -> Result<ClaimsAdmin, jsonwebtoken::errors::Error> {
    let chave = DecodingKey::from_secret(configuracao.jwt_secret.as_bytes());
    let mut validacao = Validation::new(Algorithm::HS256);
    validacao.validate_exp = true;
    let dados = decode::<ClaimsAdmin>(token, &chave, &validacao)?;
    Ok(dados.claims)
}

fn emitir_jwt_admin(email: &str, configuracao: &ConfiguracaoAplicacao) -> Result<String, ErroAplicacao> {
    let expiracao = chrono::Utc::now()
        + Duration::seconds(configuracao.jwt_expiracao_secs as i64);
    let claims = ClaimsAdmin {
        sub: email.to_string(),
        role: PAPEL_ADMIN.to_string(),
        exp: expiracao.timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(configuracao.jwt_secret.as_bytes()),
    )
    .map_err(|erro| ErroAplicacao::Interno(format!("Falha ao emitir JWT: {erro}")))
}

pub async fn login_admin(
    pool: &PgPool,
    configuracao: &ConfiguracaoAplicacao,
    email: &str,
    password: &str,
) -> Result<LoginAdminRespostaDto, ErroAplicacao> {
    let email = email.trim();
    if email.is_empty() || password.is_empty() {
        return Err(ErroAplicacao::NaoAutorizado);
    }

    let credenciais = RepositorioAdmin::obter_email_e_hash(pool).await?;
    let Some((email_db, hash_db)) = credenciais else {
        return Err(ErroAplicacao::NaoAutorizado);
    };

    if email != email_db.trim() || !verificar_senha(&hash_db, password) {
        return Err(ErroAplicacao::NaoAutorizado);
    }

    let access_token = emitir_jwt_admin(&email_db, configuracao)?;
    Ok(LoginAdminRespostaDto {
        access_token,
        token_type: "Bearer",
        expires_in: configuracao.jwt_expiracao_secs,
    })
}

/// Se `admin_conta` estiver vazia e existirem `ADMIN_BOOTSTRAP_*`, cria a conta inicial.
pub async fn garantir_bootstrap_admin_conta(
    pool: &PgPool,
    configuracao: &ConfiguracaoAplicacao,
) -> Result<(), ErroAplicacao> {
    let n = RepositorioAdmin::contar_linhas(pool).await?;
    if n > 0 {
        return Ok(());
    }

    let (Some(ref email), Some(ref password)) = (
        configuracao.admin_bootstrap_email.as_ref(),
        configuracao.admin_bootstrap_password.as_ref(),
    ) else {
        tracing::info!(
            "Tabela admin_conta vazia; defina ADMIN_BOOTSTRAP_EMAIL e ADMIN_BOOTSTRAP_PASSWORD para criar o primeiro administrador, ou insira manualmente na base."
        );
        return Ok(());
    };

    let hash = hash_senha(password)?;
    RepositorioAdmin::inserir_conta(pool, email.trim(), &hash).await?;
    tracing::info!(
        email = %email.trim(),
        "Conta de administrador criada a partir de ADMIN_BOOTSTRAP_*"
    );
    Ok(())
}
