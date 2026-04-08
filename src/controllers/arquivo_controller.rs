use std::path::Path as CaminhoFicheiro;
use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Extension, Multipart, Path as CaminhoUrl},
    http::{HeaderValue, StatusCode, header},
    response::Response,
};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    EstadoAplicacao,
    contexto::ProjetoAutenticado,
    dtos::arquivo_dto::ArquivoDto,
    erros::ErroAplicacao,
    services::arquivo_service::inferir_tipo_mime,
};

/// Grava o campo multipart em disco por chunks (nao carrega o ficheiro inteiro em RAM).
async fn gravar_campo_em_ficheiro(
    campo: &mut axum::extract::multipart::Field<'_>,
    caminho: &CaminhoFicheiro,
    limite: u64,
) -> Result<u64, ErroAplicacao> {
    if let Some(pasta) = caminho.parent() {
        tokio::fs::create_dir_all(pasta).await.map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao criar diretorio de destino: {erro}"))
        })?;
    }

    let mut ficheiro = tokio::fs::File::create(caminho).await.map_err(|erro| {
        ErroAplicacao::Interno(format!("Falha ao criar ficheiro no disco: {erro}"))
    })?;

    let mut total = 0u64;
    loop {
        let chunk = match campo.chunk().await.map_err(|erro| {
            ErroAplicacao::RequisicaoInvalida(format!("Falha ao ler chunk do upload: {erro}"))
        })? {
            Some(c) => c,
            None => break,
        };

        let adicao = chunk.len() as u64;
        if total.saturating_add(adicao) > limite {
            let _ = tokio::fs::remove_file(caminho).await;
            return Err(ErroAplicacao::PayloadDemasiadoGrande(format!(
                "Arquivo excede o limite de {limite} bytes"
            )));
        }

        ficheiro.write_all(&chunk).await.map_err(|erro| {
            ErroAplicacao::Interno(format!("Falha ao escrever no disco: {erro}"))
        })?;
        total += adicao;
    }

    Ok(total)
}

/// Upload multipart (campo `arquivo`). Tipos MIME inferidos do nome e do Content-Type.
pub async fn enviar_arquivo(
    Extension(projeto): Extension<ProjetoAutenticado>,
    Extension(estado): Extension<Arc<EstadoAplicacao>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<crate::dtos::arquivo_dto::RespostaUploadDto>), ErroAplicacao> {
    let limite = estado.servico_arquivo.tamanho_maximo_bytes();

    let (nome_arquivo, mime_campo, mut campo) = loop {
        let campo = match multipart.next_field().await.map_err(|erro| {
            ErroAplicacao::RequisicaoInvalida(format!("Falha ao processar multipart: {erro}"))
        })? {
            None => {
                return Err(ErroAplicacao::RequisicaoInvalida(
                    "Campo 'arquivo' obrigatorio e nao encontrado no multipart".to_string(),
                ));
            }
            Some(c) => c,
        };
        if campo.name() != Some("arquivo") {
            continue;
        }
        let nome_arquivo = campo.file_name().map(ToString::to_string);
        let mime_campo = campo.content_type().map(ToString::to_string);
        break (nome_arquivo, mime_campo, campo);
    };

    let nome_arquivo = nome_arquivo.ok_or_else(|| {
        ErroAplicacao::RequisicaoInvalida("Nome do arquivo e obrigatorio".to_string())
    })?;

    if nome_arquivo.trim().is_empty() {
        return Err(ErroAplicacao::RequisicaoInvalida(
            "Nome do arquivo nao pode ser vazio".to_string(),
        ));
    }

    let tipo_mime = inferir_tipo_mime(&nome_arquivo, mime_campo.as_deref());
    let id = Uuid::new_v4().to_string();
    let caminho = estado
        .servico_arquivo
        .caminho_fisico_arquivo(projeto.id, &id);
    let caminho_path = CaminhoFicheiro::new(&caminho);

    let tamanho = gravar_campo_em_ficheiro(&mut campo, caminho_path, limite).await?;

    if tamanho == 0 {
        let _ = tokio::fs::remove_file(caminho_path).await;
        return Err(ErroAplicacao::RequisicaoInvalida(
            "Arquivo enviado esta vazio".to_string(),
        ));
    }

    let resposta = estado
        .servico_arquivo
        .finalizar_upload(
            projeto.id,
            id,
            nome_arquivo,
            tipo_mime,
            tamanho,
            "arquivos",
        )
        .await?;

    Ok((StatusCode::CREATED, Json(resposta)))
}

/// Download com stream (adequado a ficheiros grandes).
pub async fn baixar_arquivo(
    Extension(projeto): Extension<ProjetoAutenticado>,
    Extension(estado): Extension<Arc<EstadoAplicacao>>,
    CaminhoUrl(id): CaminhoUrl<String>,
) -> Result<Response, ErroAplicacao> {
    let (arquivo, fluxo) = estado
        .servico_arquivo
        .fluxo_download(projeto.id, &id)
        .await?;

    let valor_mime = HeaderValue::from_str(arquivo.tipo_mime.trim()).map_err(|erro| {
        ErroAplicacao::Interno(format!(
            "Nao foi possivel definir content-type da resposta: {erro}"
        ))
    })?;
    let disposition = format!("inline; filename=\"{}\"", arquivo.nome_arquivo);
    let valor_disposition = HeaderValue::from_str(&disposition).map_err(|erro| {
        ErroAplicacao::Interno(format!(
            "Nao foi possivel definir content-disposition da resposta: {erro}"
        ))
    })?;

    let mut resposta = Response::new(Body::from_stream(fluxo));
    *resposta.status_mut() = StatusCode::OK;
    resposta
        .headers_mut()
        .insert(header::CONTENT_TYPE, valor_mime);
    resposta
        .headers_mut()
        .insert(header::CONTENT_DISPOSITION, valor_disposition);

    Ok(resposta)
}

pub async fn deletar_arquivo(
    Extension(projeto): Extension<ProjetoAutenticado>,
    Extension(estado): Extension<Arc<EstadoAplicacao>>,
    CaminhoUrl(id): CaminhoUrl<String>,
) -> Result<StatusCode, ErroAplicacao> {
    estado
        .servico_arquivo
        .deletar_arquivo(projeto.id, &id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn listar_arquivos(
    Extension(projeto): Extension<ProjetoAutenticado>,
    Extension(estado): Extension<Arc<EstadoAplicacao>>,
) -> Result<Json<Vec<ArquivoDto>>, ErroAplicacao> {
    let arquivos = estado
        .servico_arquivo
        .listar_arquivos(projeto.id)
        .await?
        .into_iter()
        .map(ArquivoDto::from)
        .collect::<Vec<_>>();

    Ok(Json(arquivos))
}

pub async fn enviar_imagem(
    Extension(projeto): Extension<ProjetoAutenticado>,
    Extension(estado): Extension<Arc<EstadoAplicacao>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<crate::dtos::arquivo_dto::RespostaUploadDto>), ErroAplicacao> {
    let limite = estado.servico_arquivo.tamanho_maximo_bytes();

    let (nome_arquivo, mime_campo, mut campo) = loop {
        let campo = match multipart.next_field().await.map_err(|erro| {
            ErroAplicacao::RequisicaoInvalida(format!("Falha ao processar multipart: {erro}"))
        })? {
            None => {
                return Err(ErroAplicacao::RequisicaoInvalida(
                    "Campo 'arquivo' obrigatorio e nao encontrado no multipart".to_string(),
                ));
            }
            Some(c) => c,
        };
        if campo.name() != Some("arquivo") {
            continue;
        }
        let nome_arquivo = campo.file_name().map(ToString::to_string);
        let mime_campo = campo.content_type().map(ToString::to_string);
        break (nome_arquivo, mime_campo, campo);
    };

    let nome_arquivo = nome_arquivo.ok_or_else(|| {
        ErroAplicacao::RequisicaoInvalida("Nome do arquivo e obrigatorio".to_string())
    })?;

    let tipo_mime = inferir_tipo_mime(&nome_arquivo, mime_campo.as_deref());

    if !crate::services::arquivo_service::validar_imagem_publica(
        &nome_arquivo,
        mime_campo.as_deref(),
        &tipo_mime,
    ) {
        return Err(ErroAplicacao::RequisicaoInvalida(
            "O envio nao corresponde a uma imagem permitida (tipo ou extensao)".to_string(),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let caminho = estado
        .servico_arquivo
        .caminho_fisico_arquivo(projeto.id, &id);
    let caminho_path = CaminhoFicheiro::new(&caminho);

    let tamanho = gravar_campo_em_ficheiro(&mut campo, caminho_path, limite).await?;

    if tamanho == 0 {
        let _ = tokio::fs::remove_file(caminho_path).await;
        return Err(ErroAplicacao::RequisicaoInvalida(
            "Arquivo enviado esta vazio".to_string(),
        ));
    }

    let resposta = estado
        .servico_arquivo
        .finalizar_upload(
            projeto.id,
            id,
            nome_arquivo,
            tipo_mime,
            tamanho,
            "imagens",
        )
        .await?;

    Ok((StatusCode::CREATED, Json(resposta)))
}
