use std::path::PathBuf;
use std::sync::Arc;

use tokio::fs;
use uuid::Uuid;

use crate::{
    contexto::ProjetoAutenticado,
    dtos::projeto_dto::{ProjetoApagadoDto, ProjetoCriadoDto, ProjetoListaDto},
    erros::ErroAplicacao,
    repositories::projeto_repository::RepositorioProjeto,
};

/// Regras de negocio para gestao de projetos (admin).
pub struct ServicoProjeto {
    repositorio: Arc<RepositorioProjeto>,
    diretorio_armazenamento: PathBuf,
}

impl ServicoProjeto {
    pub fn novo(repositorio: Arc<RepositorioProjeto>, diretorio_armazenamento: String) -> Self {
        Self {
            repositorio,
            diretorio_armazenamento: PathBuf::from(diretorio_armazenamento),
        }
    }

    /// Resolve projeto pela API key (middleware).
    pub async fn obter_por_api_key(
        &self,
        api_key: &str,
    ) -> Result<Option<ProjetoAutenticado>, ErroAplicacao> {
        match self.repositorio.buscar_por_api_key(api_key).await? {
            None => Ok(None),
            Some(modelo) => Ok(Some(ProjetoAutenticado {
                id: modelo.id,
                nome: modelo.nome,
            })),
        }
    }

    /// Cria projeto e gera uma API key aleatoria (UUID).
    pub async fn criar_projeto(&self, nome: String) -> Result<ProjetoCriadoDto, ErroAplicacao> {
        let nome = nome.trim().to_string();
        if nome.is_empty() {
            return Err(ErroAplicacao::RequisicaoInvalida(
                "Nome do projeto nao pode ser vazio".to_string(),
            ));
        }

        let id = Uuid::new_v4();
        let api_key = Uuid::new_v4().to_string();

        let modelo = self.repositorio.inserir(id, nome.clone(), api_key.clone()).await?;

        Ok(ProjetoCriadoDto {
            id: modelo.id,
            nome: modelo.nome,
            api_key: modelo.api_key,
        })
    }

    pub async fn listar_projetos(&self) -> Result<Vec<ProjetoListaDto>, ErroAplicacao> {
        let linhas = self.repositorio.listar_todos().await?;
        Ok(linhas
            .into_iter()
            .map(|(id, nome, criado_em)| ProjetoListaDto {
                id,
                nome,
                criado_em,
            })
            .collect())
    }

    /// Remove projeto na base (CASCADE em arquivos), apaga pasta `{base}/{projeto_id}` no disco
    /// e devolve os dados do projeto removido.
    pub async fn apagar_projeto(&self, id: Uuid) -> Result<ProjetoApagadoDto, ErroAplicacao> {
        let pasta_projeto = self.diretorio_armazenamento.join(id.to_string());

        let removido = self.repositorio.apagar_por_id_retornando(id).await?;
        let (id, nome, api_key, criado_em) = removido.ok_or_else(|| {
            ErroAplicacao::NaoEncontrado("Projeto nao encontrado".to_string())
        })?;

        if let Err(erro) = fs::remove_dir_all(&pasta_projeto).await {
            tracing::warn!(
                pasta = %pasta_projeto.display(),
                erro = %erro,
                "Nao foi possivel remover pasta do projeto no disco"
            );
        }

        Ok(ProjetoApagadoDto {
            id,
            nome,
            api_key,
            criado_em,
        })
    }
}
