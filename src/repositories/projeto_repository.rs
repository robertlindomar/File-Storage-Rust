use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{erros::ErroAplicacao, models::projeto_modelo::ProjetoModelo};

/// Persistencia de projetos (tenants) e respetivas API keys.
pub struct RepositorioProjeto {
    pool: PgPool,
}

impl RepositorioProjeto {
    pub fn novo(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn inserir(
        &self,
        id: Uuid,
        nome: String,
        api_key: String,
    ) -> Result<ProjetoModelo, ErroAplicacao> {
        sqlx::query(
            r#"
            INSERT INTO projetos (id, nome, api_key)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(id)
        .bind(&nome)
        .bind(&api_key)
        .execute(&self.pool)
        .await
        .map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao criar projeto: {erro}"))
        })?;

        Ok(ProjetoModelo {
            id,
            nome,
            api_key,
        })
    }

    pub async fn buscar_por_api_key(&self, api_key: &str) -> Result<Option<ProjetoModelo>, ErroAplicacao> {
        let linha = sqlx::query(
            r#"
            SELECT id, nome, api_key
            FROM projetos
            WHERE api_key = $1
            "#,
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao resolver projeto pela API key: {erro}"))
        })?;

        Ok(linha.map(|linha| ProjetoModelo {
            id: linha.get(0),
            nome: linha.get(1),
            api_key: linha.get(2),
        }))
    }

    pub async fn buscar_por_id(&self, id: Uuid) -> Result<Option<ProjetoModelo>, ErroAplicacao> {
        let linha = sqlx::query(
            r#"
            SELECT id, nome, api_key
            FROM projetos
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao buscar projeto: {erro}"))
        })?;

        Ok(linha.map(|linha| ProjetoModelo {
            id: linha.get(0),
            nome: linha.get(1),
            api_key: linha.get(2),
        }))
    }

    pub async fn listar_todos(
        &self,
    ) -> Result<Vec<(Uuid, String, chrono::DateTime<chrono::Utc>)>, ErroAplicacao> {
        let linhas = sqlx::query(
            r#"
            SELECT id, nome, criado_em
            FROM projetos
            ORDER BY criado_em DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao listar projetos: {erro}"))
        })?;

        Ok(linhas
            .into_iter()
            .map(|linha| {
                (
                    linha.get(0),
                    linha.get(1),
                    linha.get(2),
                )
            })
            .collect())
    }

    /// Apaga o projeto e devolve os metadados da linha removida (uma unica ida a base).
    pub async fn apagar_por_id_retornando(
        &self,
        id: Uuid,
    ) -> Result<
        Option<(
            Uuid,
            String,
            String,
            chrono::DateTime<chrono::Utc>,
        )>,
        ErroAplicacao,
    > {
        let linha = sqlx::query(
            r#"
            DELETE FROM projetos
            WHERE id = $1
            RETURNING id, nome, api_key, criado_em
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao apagar projeto: {erro}"))
        })?;

        Ok(linha.map(|linha| {
            (
                linha.get(0),
                linha.get(1),
                linha.get(2),
                linha.get(3),
            )
        }))
    }
}
