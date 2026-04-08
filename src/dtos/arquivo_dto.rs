use serde::Serialize;

use crate::models::arquivo_modelo::ArquivoModelo;

/// Resposta padrao apos upload (mini S3).
#[derive(Debug, Serialize)]
pub struct RespostaUploadDto {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ArquivoDto {
    pub id: String,
    pub nome_arquivo: String,
    pub tipo_mime: String,
    pub tamanho: i64,
}

impl From<ArquivoModelo> for ArquivoDto {
    fn from(modelo: ArquivoModelo) -> Self {
        Self {
            id: modelo.id,
            nome_arquivo: modelo.nome_arquivo,
            tipo_mime: modelo.tipo_mime,
            tamanho: modelo.tamanho,
        }
    }
}
