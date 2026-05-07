use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models::arquivo_modelo::ArquivoModelo;

/// Resposta padrao apos upload (mini S3).
#[derive(Debug, Clone, Serialize)]
pub struct RespostaUploadDto {
    pub id: String,
    pub url: String,
}

/// Entrada em `falhas` apos `POST /arquivos/lote`.
#[derive(Debug, Serialize)]
pub struct FalhaUploadLoteDto {
    pub nome_arquivo: String,
    pub erro: String,
}

/// Resposta do upload em lote (melhor esforco; falhas parciais nao revertem sucessos).
#[derive(Debug, Serialize)]
pub struct RespostaLoteUploadDto {
    pub sucesso: Vec<RespostaUploadDto>,
    pub falhas: Vec<FalhaUploadLoteDto>,
}

#[derive(Debug, Serialize)]
pub struct ArquivoDto {
    pub id: String,
    pub nome_arquivo: String,
    pub tipo_mime: String,
    pub tamanho: i64,
    pub criado_em: DateTime<Utc>,
    pub projeto_id: Uuid,
}

impl From<ArquivoModelo> for ArquivoDto {
    fn from(modelo: ArquivoModelo) -> Self {
        Self {
            id: modelo.id,
            nome_arquivo: modelo.nome_arquivo,
            tipo_mime: modelo.tipo_mime,
            tamanho: modelo.tamanho,
            criado_em: modelo.criado_em,
            projeto_id: modelo.projeto_id,
        }
    }
}

/// Metricas agregadas para o dashboard admin (`GET /admin/metricas/armazenamento`).
#[derive(Debug, Serialize)]
pub struct MetricasArmazenamentoDto {
    /// Soma dos campos `tamanho` na tabela `arquivos`.
    pub bytes_usados: i64,
    /// Cota opcional (`COTA_ARMAZENAMENTO_BYTES`); se ausente, `percentagem` e null.
    pub bytes_cota: Option<u64>,
    /// Percentagem da cota usada (0–100), arredondada a 1 casa decimal.
    pub percentagem: Option<f64>,
}

/// Upload recente para o painel admin (atividade global).
#[derive(Debug, Serialize)]
pub struct AtividadeUploadDto {
    pub nome_arquivo: String,
    pub projeto_id: Uuid,
    pub projeto_nome: String,
    pub tamanho: i64,
    pub criado_em: DateTime<Utc>,
}
