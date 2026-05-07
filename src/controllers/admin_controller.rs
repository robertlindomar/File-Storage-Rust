use std::sync::Arc;

use axum::{
    Json,
    extract::{Extension, Path as CaminhoUrl, Query},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    EstadoAplicacao,
    dtos::{
        arquivo_dto::{AtividadeUploadDto, MetricasArmazenamentoDto},
        projeto_dto::{CriarProjetoDto, ProjetoApagadoDto, ProjetoCriadoDto, ProjetoListaDto},
    },
    erros::ErroAplicacao,
};

#[derive(Debug, Deserialize)]
pub struct LimiteUploadsRecentes {
    limite: Option<i64>,
}

pub async fn criar_projeto(
    Extension(estado): Extension<Arc<EstadoAplicacao>>,
    Json(corpo): Json<CriarProjetoDto>,
) -> Result<(StatusCode, Json<ProjetoCriadoDto>), ErroAplicacao> {
    let criado = estado.servico_projeto.criar_projeto(corpo.nome).await?;
    Ok((StatusCode::CREATED, Json(criado)))
}

pub async fn listar_projetos(
    Extension(estado): Extension<Arc<EstadoAplicacao>>,
) -> Result<Json<Vec<ProjetoListaDto>>, ErroAplicacao> {
    let lista = estado.servico_projeto.listar_projetos().await?;
    Ok(Json(lista))
}

pub async fn apagar_projeto(
    Extension(estado): Extension<Arc<EstadoAplicacao>>,
    CaminhoUrl(id): CaminhoUrl<Uuid>,
) -> Result<(StatusCode, Json<ProjetoApagadoDto>), ErroAplicacao> {
    let apagado = estado.servico_projeto.apagar_projeto(id).await?;
    Ok((StatusCode::OK, Json(apagado)))
}

pub async fn listar_uploads_recentes(
    Extension(estado): Extension<Arc<EstadoAplicacao>>,
    Query(params): Query<LimiteUploadsRecentes>,
) -> Result<Json<Vec<AtividadeUploadDto>>, ErroAplicacao> {
    let limite = params.limite.unwrap_or(20).clamp(1, 100);
    let lista = estado
        .servico_arquivo
        .listar_uploads_recentes_globais(limite)
        .await?;
    Ok(Json(lista))
}

pub async fn metricas_armazenamento(
    Extension(estado): Extension<Arc<EstadoAplicacao>>,
) -> Result<Json<MetricasArmazenamentoDto>, ErroAplicacao> {
    let metricas = estado
        .servico_arquivo
        .metricas_armazenamento_globais(estado.configuracao.cota_armazenamento_bytes)
        .await?;
    Ok(Json(metricas))
}
