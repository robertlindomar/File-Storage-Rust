use serde::Serialize;
use uuid::Uuid;

/// Modelo interno de um arquivo armazenado no servico.
#[derive(Clone, Debug, Serialize)]
pub struct ArquivoModelo {
    pub id: String,
    pub projeto_id: Uuid,
    pub nome_arquivo: String,
    pub caminho: String,
    pub tipo_mime: String,
    pub tamanho: i64,
}
