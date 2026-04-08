use std::sync::Arc;

use axum::{
    Json,
    extract::{Extension, Path as CaminhoUrl},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    EstadoAplicacao,
    dtos::projeto_dto::{CriarProjetoDto, ProjetoApagadoDto, ProjetoCriadoDto, ProjetoListaDto},
    erros::ErroAplicacao,
};

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
