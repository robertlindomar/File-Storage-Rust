use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CriarProjetoDto {
    pub nome: String,
}

#[derive(Debug, Serialize)]
pub struct ProjetoCriadoDto {
    pub id: Uuid,
    pub nome: String,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
pub struct ProjetoListaDto {
    pub id: Uuid,
    pub nome: String,
    pub criado_em: chrono::DateTime<chrono::Utc>,
}

/// Copia do projeto removido (resposta de `DELETE /api/v1/admin/projetos/:id`).
#[derive(Debug, Serialize)]
pub struct ProjetoApagadoDto {
    pub id: Uuid,
    pub nome: String,
    pub api_key: String,
    pub criado_em: chrono::DateTime<chrono::Utc>,
}
