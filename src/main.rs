use std::sync::Arc;

use file_storage_service::{
    config::ConfiguracaoAplicacao,
    construir_estado_aplicacao,
    rotas,
    services::auth_admin_service::garantir_bootstrap_admin_conta,
};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    inicializar_logs();

    let configuracao = Arc::new(ConfiguracaoAplicacao::carregar());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuracao.database_url)
        .await
        .expect("Nao foi possivel conectar ao PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Falha ao executar migracoes SQL");

    garantir_bootstrap_admin_conta(&pool, configuracao.as_ref())
        .await
        .expect("Falha ao aplicar bootstrap da conta de administrador");

    let estado = construir_estado_aplicacao(configuracao.clone(), pool);
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
