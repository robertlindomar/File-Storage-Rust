use std::sync::Arc;

use mime::IMAGE;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{
    dtos::arquivo_dto::{AtividadeUploadDto, MetricasArmazenamentoDto, RespostaUploadDto},
    erros::ErroAplicacao,
    models::arquivo_modelo::ArquivoModelo,
    repositories::arquivo_repository::RepositorioArquivo,
};

/// Regras de negocio do dominio de arquivos (por projeto).
pub struct ServicoArquivo {
    repositorio: Arc<RepositorioArquivo>,
    porta_aplicacao: u16,
    base_url: Option<String>,
    tamanho_maximo_bytes: u64,
}

impl ServicoArquivo {
    pub fn novo(
        repositorio: Arc<RepositorioArquivo>,
        porta_aplicacao: u16,
        base_url: Option<String>,
        tamanho_maximo_bytes: u64,
    ) -> Self {
        Self {
            repositorio,
            porta_aplicacao,
            base_url,
            tamanho_maximo_bytes,
        }
    }

    pub fn tamanho_maximo_bytes(&self) -> u64 {
        self.tamanho_maximo_bytes
    }

    /// Caminho absoluto onde o ficheiro deve ser gravado (streaming).
    pub fn caminho_fisico_arquivo(&self, projeto_id: Uuid, id_arquivo: &str) -> String {
        self.repositorio
            .caminho_absoluto_arquivo(projeto_id, id_arquivo)
    }

    /// Apos gravar o ficheiro no disco (ex.: por chunks), regista metadados e devolve URL publica.
    pub async fn finalizar_upload(
        &self,
        projeto_id: Uuid,
        id: String,
        nome_arquivo: String,
        tipo_mime: String,
        tamanho_bytes: u64,
        segmento_url: &str,
    ) -> Result<RespostaUploadDto, ErroAplicacao> {
        self.validar_tamanho(tamanho_bytes)?;
        let caminho = self.caminho_fisico_arquivo(projeto_id, &id);
        let tamanho = tamanho_bytes as i64;
        self.repositorio
            .inserir_registro(
                projeto_id,
                id.clone(),
                nome_arquivo,
                caminho,
                tipo_mime,
                tamanho,
            )
            .await?;
        Ok(RespostaUploadDto {
            id: id.clone(),
            url: self.montar_url(segmento_url, &id),
        })
    }

    fn url_base(&self) -> String {
        self.base_url.clone().unwrap_or_else(|| {
            format!("http://localhost:{}", self.porta_aplicacao)
        })
    }

    /// Monta URL publica com prefixo `/api/v1`.
    fn montar_url(&self, segmento: &str, id: &str) -> String {
        let base_completo = self.url_base();
        let base = base_completo.trim_end_matches('/');
        format!("{base}/api/v1/{segmento}/{id}")
    }

    fn validar_tamanho(&self, tamanho: u64) -> Result<(), ErroAplicacao> {
        if tamanho > self.tamanho_maximo_bytes {
            return Err(ErroAplicacao::PayloadDemasiadoGrande(format!(
                "Arquivo excede o limite de {} bytes",
                self.tamanho_maximo_bytes
            )));
        }
        Ok(())
    }

    pub async fn buscar_arquivo(
        &self,
        projeto_id: Uuid,
        id: &str,
    ) -> Result<ArquivoModelo, ErroAplicacao> {
        self.repositorio
            .buscar_por_id_e_projeto(projeto_id, id)
            .await?
            .ok_or_else(|| {
                ErroAplicacao::NaoEncontrado(format!("Arquivo com id {id} nao foi encontrado"))
            })
    }

    /// Stream de leitura para download (evita carregar ficheiros grandes inteiros em RAM).
    pub async fn fluxo_download(
        &self,
        projeto_id: Uuid,
        id: &str,
    ) -> Result<(ArquivoModelo, ReaderStream<File>), ErroAplicacao> {
        let arquivo = self.buscar_arquivo(projeto_id, id).await?;
        let ficheiro = self
            .repositorio
            .abrir_ficheiro_leitura(&arquivo.caminho)
            .await?;
        let fluxo = ReaderStream::new(ficheiro);
        Ok((arquivo, fluxo))
    }

    pub async fn deletar_arquivo(
        &self,
        projeto_id: Uuid,
        id: &str,
    ) -> Result<(), ErroAplicacao> {
        self.repositorio
            .deletar_por_projeto_e_id(projeto_id, id)
            .await
    }

    pub async fn listar_arquivos(
        &self,
        projeto_id: Uuid,
    ) -> Result<Vec<ArquivoModelo>, ErroAplicacao> {
        self.repositorio.listar_por_projeto(projeto_id).await
    }

    /// Uploads recentes em todos os projetos (rota admin com JWT).
    pub async fn listar_uploads_recentes_globais(
        &self,
        limite: i64,
    ) -> Result<Vec<AtividadeUploadDto>, ErroAplicacao> {
        let linhas = self
            .repositorio
            .listar_uploads_recentes_globais(limite)
            .await?;
        Ok(linhas
            .into_iter()
            .map(|linha| AtividadeUploadDto {
                nome_arquivo: linha.nome_arquivo,
                projeto_id: linha.projeto_id,
                projeto_nome: linha.projeto_nome,
                tamanho: linha.tamanho,
                criado_em: linha.criado_em,
            })
            .collect())
    }

    /// Bytes totais registados e, se houver cota, percentagem usada (max 100).
    pub async fn metricas_armazenamento_globais(
        &self,
        bytes_cota: Option<u64>,
    ) -> Result<MetricasArmazenamentoDto, ErroAplicacao> {
        let bytes_usados = self.repositorio.somar_bytes_totais_admin().await?;
        let percentagem = bytes_cota.and_then(|cota| {
            if cota == 0 {
                return None;
            }
            let usados = bytes_usados.max(0) as f64;
            let pct = (usados / cota as f64) * 100.0;
            Some((pct.min(100.0) * 10.0).round() / 10.0)
        });
        Ok(MetricasArmazenamentoDto {
            bytes_usados,
            bytes_cota,
            percentagem,
        })
    }
}

/// Usado pelo controlador de imagens apos inferir MIME.
pub fn validar_imagem_publica(
    nome_arquivo: &str,
    content_type: Option<&str>,
    tipo_mime_inferido: &str,
) -> bool {
    validar_imagem_interna(nome_arquivo, content_type, tipo_mime_inferido)
}

fn validar_imagem_interna(
    nome_arquivo: &str,
    content_type: Option<&str>,
    tipo_mime_inferido: &str,
) -> bool {
    if tipo_mime_inferido.starts_with("image/") {
        return true;
    }
    if let Some(ct) = content_type
        && let Ok(mime_tipo) = ct.parse::<mime::Mime>()
        && mime_tipo.type_() == IMAGE
    {
        return true;
    }

    mime_guess::from_path(nome_arquivo)
        .first_raw()
        .map(|s| s.starts_with("image/"))
        .unwrap_or(false)
}

/// Deduz MIME a partir do cabecalho multipart e do nome do ficheiro.
pub fn inferir_tipo_mime(nome_arquivo: &str, content_type: Option<&str>) -> String {
    if let Some(ct) = content_type
        && let Ok(m) = ct.parse::<mime::Mime>()
    {
        return m.essence_str().to_string();
    }
    mime_guess::from_path(nome_arquivo)
        .first_or_octet_stream()
        .essence_str()
        .to_string()
}
