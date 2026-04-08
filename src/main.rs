use std::sync::Arc;

use config::ConfiguracaoAplicacao;
use repositories::{arquivo_repository::RepositorioArquivo, projeto_repository::RepositorioProjeto};
use services::{arquivo_service::ServicoArquivo, projeto_service::ServicoProjeto};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{EnvFilter, fmt};

mod config;
mod contexto;
mod controllers;
mod dtos;
mod erros;
mod middlewares;
mod models;
mod repositories;
mod rotas;
mod services;

/// Estado partilhado pelo Axum (pool, configuracao, servicos).
#[derive(Clone)]
pub struct EstadoAplicacao {
    pub configuracao: Arc<ConfiguracaoAplicacao>,
    pub pool: sqlx::PgPool,
    pub servico_arquivo: Arc<ServicoArquivo>,
    pub servico_projeto: Arc<ServicoProjeto>,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    inicializar_logs();

    let configuracao = Arc::new(ConfiguracaoAplicacao::carregar());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuracao.database_url)
        .await
        .expect("Nao foi possivel ligar ao PostgreSQL; verifique DATABASE_URL");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Falha ao executar migracoes SQL");

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

    let estado = Arc::new(EstadoAplicacao {
        configuracao: configuracao.clone(),
        pool,
        servico_arquivo,
        servico_projeto,
    });

    let aplicativo = rotas::criar_rotas(estado);
    let endereco = format!("0.0.0.0:{}", configuracao.porta);

    tracing::info!(
        "Servidor iniciado em http://localhost:{}",
        configuracao.porta
    );
    let listener = tokio::net::TcpListener::bind(&endereco)
        .await
        .expect("Nao foi possivel abrir a porta configurada");

    axum::serve(listener, aplicativo)
        .await
        .expect("Falha ao executar o servidor HTTP");
}

fn inicializar_logs() {
    let filtro = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filtro).init();
}
