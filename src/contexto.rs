use uuid::Uuid;

/// Dados do projeto autenticado (injectados pelo middleware apos validar API Key).
#[derive(Clone, Debug)]
pub struct ProjetoAutenticado {
    pub id: Uuid,
    pub nome: String,
}
