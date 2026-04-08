use std::path::PathBuf;

use sqlx::{PgPool, Row};
use tokio::fs;
use uuid::Uuid;

use crate::{erros::ErroAplicacao, models::arquivo_modelo::ArquivoModelo};

/// Camada de acesso ao disco local e metadados em PostgreSQL (isolado por projeto).
pub struct RepositorioArquivo {
    pool: PgPool,
    diretorio_base: PathBuf,
}

impl RepositorioArquivo {
    pub fn novo(pool: PgPool, diretorio_base: String) -> Self {
        Self {
            pool,
            diretorio_base: PathBuf::from(diretorio_base),
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Caminho absoluto `{base}/{projeto_id}/{id_arquivo}` (sem extensao no disco).
    pub fn caminho_absoluto_arquivo(&self, projeto_id: Uuid, id_arquivo: &str) -> String {
        self.diretorio_base
            .join(projeto_id.to_string())
            .join(id_arquivo)
            .to_string_lossy()
            .to_string()
    }

    /// Grava em `{base}/{projeto_id}/{arquivo_id}` (sem nome original no path).
    pub async fn salvar(
        &self,
        projeto_id: Uuid,
        id: String,
        nome_arquivo: String,
        tipo_mime: String,
        conteudo: Vec<u8>,
    ) -> Result<ArquivoModelo, ErroAplicacao> {
        let id_uuid = Uuid::parse_str(&id).map_err(|erro| {
            ErroAplicacao::Interno(format!("Id invalido para persistencia: {erro}"))
        })?;

        let tamanho = conteudo.len() as i64;
        let diretorio_projeto = self.diretorio_base.join(projeto_id.to_string());
        fs::create_dir_all(&diretorio_projeto)
            .await
            .map_err(|erro| {
                ErroAplicacao::Interno(format!(
                    "Falha ao preparar diretorio do projeto: {erro}"
                ))
            })?;

        let caminho_arquivo = diretorio_projeto.join(&id);
        let caminho_str = caminho_arquivo.to_string_lossy().to_string();

        fs::write(&caminho_arquivo, &conteudo)
            .await
            .map_err(|erro| {
                ErroAplicacao::Interno(format!("Falha ao salvar arquivo no disco: {erro}"))
            })?;

        let resultado = sqlx::query(
            r#"
            INSERT INTO arquivos (id, projeto_id, nome_arquivo, caminho, tipo_mime, tamanho)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id_uuid)
        .bind(projeto_id)
        .bind(&nome_arquivo)
        .bind(&caminho_str)
        .bind(&tipo_mime)
        .bind(tamanho)
        .execute(&self.pool)
        .await;

        if let Err(erro) = resultado {
            let _ = fs::remove_file(&caminho_arquivo).await;
            return Err(ErroAplicacao::Interno(format!(
                "Falha ao registrar arquivo na base de dados: {erro}"
            )));
        }

        Ok(ArquivoModelo {
            id,
            projeto_id,
            nome_arquivo,
            caminho: caminho_str,
            tipo_mime,
            tamanho,
        })
    }

    pub async fn buscar_por_id_e_projeto(
        &self,
        projeto_id: Uuid,
        id: &str,
    ) -> Result<Option<ArquivoModelo>, ErroAplicacao> {
        let id_uuid = match Uuid::parse_str(id) {
            Ok(valor) => valor,
            Err(_) => return Ok(None),
        };

        let linha = sqlx::query(
            r#"
            SELECT id::text, projeto_id, nome_arquivo, caminho, tipo_mime, tamanho
            FROM arquivos
            WHERE id = $1 AND projeto_id = $2
            "#,
        )
        .bind(id_uuid)
        .bind(projeto_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao consultar arquivo na base: {erro}"))
        })?;

        Ok(linha.map(|linha| ArquivoModelo {
            id: linha.get::<String, _>(0),
            projeto_id: linha.get(1),
            nome_arquivo: linha.get(2),
            caminho: linha.get(3),
            tipo_mime: linha.get(4),
            tamanho: linha.get(5),
        }))
    }

    pub async fn listar_por_projeto(
        &self,
        projeto_id: Uuid,
    ) -> Result<Vec<ArquivoModelo>, ErroAplicacao> {
        let linhas = sqlx::query(
            r#"
            SELECT id::text, projeto_id, nome_arquivo, caminho, tipo_mime, tamanho
            FROM arquivos
            WHERE projeto_id = $1
            ORDER BY criado_em DESC
            "#,
        )
        .bind(projeto_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao listar arquivos na base: {erro}"))
        })?;

        Ok(linhas
            .into_iter()
            .map(|linha| ArquivoModelo {
                id: linha.get(0),
                projeto_id: linha.get(1),
                nome_arquivo: linha.get(2),
                caminho: linha.get(3),
                tipo_mime: linha.get(4),
                tamanho: linha.get(5),
            })
            .collect())
    }

    /// Leitura completa em memoria; para ficheiros muito grandes prefira `abrir_ficheiro`.
    pub async fn ler_bytes(&self, caminho: &str) -> Result<Vec<u8>, ErroAplicacao> {
        fs::read(caminho).await.map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao ler arquivo armazenado: {erro}"))
        })
    }

    pub async fn abrir_ficheiro_leitura(
        &self,
        caminho: &str,
    ) -> Result<tokio::fs::File, ErroAplicacao> {
        tokio::fs::File::open(caminho).await.map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao abrir arquivo no disco: {erro}"))
        })
    }

    pub async fn deletar_por_projeto_e_id(
        &self,
        projeto_id: Uuid,
        id: &str,
    ) -> Result<(), ErroAplicacao> {
        let id_uuid = Uuid::parse_str(id).map_err(|_| {
            ErroAplicacao::NaoEncontrado("Identificador de arquivo invalido".to_string())
        })?;

        let caminho: Option<String> = sqlx::query_scalar(
            r#"
            SELECT caminho FROM arquivos WHERE id = $1 AND projeto_id = $2
            "#,
        )
        .bind(id_uuid)
        .bind(projeto_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao localizar arquivo na base: {erro}"))
        })?;

        let caminho = caminho.ok_or_else(|| {
            ErroAplicacao::NaoEncontrado("Arquivo nao encontrado para remocao".to_string())
        })?;

        sqlx::query(r#"DELETE FROM arquivos WHERE id = $1 AND projeto_id = $2"#)
            .bind(id_uuid)
            .bind(projeto_id)
            .execute(&self.pool)
            .await
            .map_err(|erro| {
                ErroAplicacao::Interno(format!("Falha ao remover registro do arquivo: {erro}"))
            })?;

        if let Err(erro) = fs::remove_file(&caminho).await {
            tracing::warn!(
                caminho = %caminho,
                erro = %erro,
                "Ficheiro no disco nao foi removido apos DELETE na base"
            );
        }

        Ok(())
    }

    /// Grava apenas metadados na base; o ficheiro ja deve existir em `caminho`.
    pub async fn inserir_registro(
        &self,
        projeto_id: Uuid,
        id: String,
        nome_arquivo: String,
        caminho: String,
        tipo_mime: String,
        tamanho: i64,
    ) -> Result<ArquivoModelo, ErroAplicacao> {
        let id_uuid = Uuid::parse_str(&id).map_err(|erro| {
            ErroAplicacao::Interno(format!("Id invalido para persistencia: {erro}"))
        })?;

        sqlx::query(
            r#"
            INSERT INTO arquivos (id, projeto_id, nome_arquivo, caminho, tipo_mime, tamanho)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id_uuid)
        .bind(projeto_id)
        .bind(&nome_arquivo)
        .bind(&caminho)
        .bind(&tipo_mime)
        .bind(tamanho)
        .execute(&self.pool)
        .await
        .map_err(|erro| {
            ErroAplicacao::Interno(format!(
                "Falha ao registrar arquivo na base de dados: {erro}"
            ))
        })?;

        Ok(ArquivoModelo {
            id,
            projeto_id,
            nome_arquivo,
            caminho,
            tipo_mime,
            tamanho,
        })
    }
}
