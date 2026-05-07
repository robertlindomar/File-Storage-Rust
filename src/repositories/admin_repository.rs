use sqlx::{PgPool, Row};

use crate::erros::ErroAplicacao;

/// Leitura e escrita da conta de administrador unica (`admin_conta`).
pub struct RepositorioAdmin;

impl RepositorioAdmin {
    pub async fn contar_linhas(pool: &PgPool) -> Result<i64, ErroAplicacao> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::bigint FROM admin_conta")
            .fetch_one(pool)
            .await
            .map_err(|erro| {
                ErroAplicacao::Interno(format!("Falha ao contar admin_conta: {erro}"))
            })
    }

    pub async fn obter_email_e_hash(pool: &PgPool) -> Result<Option<(String, String)>, ErroAplicacao> {
        let linha = sqlx::query("SELECT email, senha_hash FROM admin_conta WHERE id = 1")
            .fetch_optional(pool)
            .await
            .map_err(|erro| {
                ErroAplicacao::Interno(format!("Falha ao ler admin_conta: {erro}"))
            })?;

        Ok(linha.map(|linha| {
            (
                linha.get::<String, _>(0),
                linha.get::<String, _>(1),
            )
        }))
    }

    pub async fn inserir_conta(
        pool: &PgPool,
        email: &str,
        senha_hash: &str,
    ) -> Result<(), ErroAplicacao> {
        sqlx::query(
            r#"
            INSERT INTO admin_conta (id, email, senha_hash)
            VALUES (1, $1, $2)
            "#,
        )
        .bind(email)
        .bind(senha_hash)
        .execute(pool)
        .await
        .map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao inserir admin_conta: {erro}"))
        })?;
        Ok(())
    }
}
