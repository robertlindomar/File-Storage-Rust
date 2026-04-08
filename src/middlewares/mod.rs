//! Middlewares HTTP: autenticacao por projeto (API key) e por admin.

mod admin;
mod projeto;

pub use admin::camada_admin;
pub use projeto::camada_projeto;
