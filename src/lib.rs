use std::sync::Arc;

use config::ConfiguracaoAplicacao;
use repositories::{
    arquivo_repository::RepositorioArquivo, projeto_repository::RepositorioProjeto,
};
use services::{arquivo_service::ServicoArquivo, projeto_service::ServicoProjeto};

pub mod config;
pub mod contexto;
pub mod controllers;
pub mod dtos;
pub mod erros;
pub mod middlewares;
pub mod models;
pub mod repositories;
pub mod rotas;
pub mod services;

/// Estado partilhado pelo Axum (pool, configuracao, servicos).
#[derive(Clone)]
pub struct EstadoAplicacao {
    pub configuracao: Arc<ConfiguracaoAplicacao>,
    pub pool: sqlx::PgPool,
    pub servico_arquivo: Arc<ServicoArquivo>,
    pub servico_projeto: Arc<ServicoProjeto>,
}

pub fn construir_estado_aplicacao(
    configuracao: Arc<ConfiguracaoAplicacao>,
    pool: sqlx::PgPool,
) -> Arc<EstadoAplicacao> {
    let repositorio_projeto = Arc::new(RepositorioProjeto::novo(pool.clone()));
    let repositorio_arquivo = Arc::new(RepositorioArquivo::novo(
        pool.clone(),
        configuracao.diretorio_armazenamento.clone(),
    ));

    let servico_projeto = Arc::new(ServicoProjeto::novo(
        repositorio_projeto,
        configuracao.diretorio_armazenamento.clone(),
    ));

    let servico_arquivo = Arc::new(ServicoArquivo::novo(
        repositorio_arquivo,
        configuracao.porta,
        configuracao.base_url.clone(),
        configuracao.tamanho_maximo_arquivo_bytes,
    ));

    Arc::new(EstadoAplicacao {
        configuracao,
        pool,
        servico_arquivo,
        servico_projeto,
    })
}
