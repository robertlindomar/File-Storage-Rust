use serde::Serialize;
use uuid::Uuid;

/// Modelo de um projeto (tenant) no servico de armazenamento.
#[derive(Clone, Debug, Serialize)]
pub struct ProjetoModelo {
    pub id: Uuid,
    pub nome: String,
    pub api_key: String,
}
