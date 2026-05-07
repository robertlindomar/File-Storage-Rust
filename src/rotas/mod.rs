use std::sync::Arc;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware::from_fn,
    routing::{delete, get, post},
};
use tower_http::{
    add_extension::AddExtensionLayer,
    cors::{Any, CorsLayer},
};

use crate::{
    EstadoAplicacao,
    controllers::{admin_controller, arquivo_controller, auth_controller, saude_controller},
    middlewares::{camada_admin, camada_projeto},
};

/// Constroi todas as rotas HTTP. Estado global via [`AddExtensionLayer`]
/// para compatibilidade com `axum::serve` (`Router<()>`).
pub fn criar_rotas(estado: Arc<EstadoAplicacao>) -> Router {
    let limite_arquivo = estado
        .configuracao
        .tamanho_maximo_arquivo_bytes
        .min(usize::MAX as u64) as usize;

    let limite_lote = estado
        .configuracao
        .tamanho_maximo_corpo_lote_bytes
        .min(usize::MAX as u64) as usize;

    let rotas_tenant_lote = Router::new()
        .route("/arquivos/lote", post(arquivo_controller::enviar_arquivos_lote))
        .layer(DefaultBodyLimit::max(limite_lote))
        .layer(from_fn(camada_projeto));

    let rotas_tenant_resto = Router::new()
        .route("/arquivos", post(arquivo_controller::enviar_arquivo))
        .route("/arquivos", get(arquivo_controller::listar_arquivos))
        .route("/arquivos/:id", get(arquivo_controller::baixar_arquivo))
        .route("/arquivos/:id", delete(arquivo_controller::deletar_arquivo))
        .route("/imagens", post(arquivo_controller::enviar_imagem))
        .route("/imagens/:id", get(arquivo_controller::baixar_arquivo))
        .layer(DefaultBodyLimit::max(limite_arquivo))
        .layer(from_fn(camada_projeto));

    let rotas_tenant = Router::new()
        .merge(rotas_tenant_resto)
        .merge(rotas_tenant_lote);

    let rotas_admin = Router::new()
        .route("/admin/projetos", post(admin_controller::criar_projeto))
        .route("/admin/projetos", get(admin_controller::listar_projetos))
        .route("/admin/projetos/:id", delete(admin_controller::apagar_projeto))
        .route(
            "/admin/atividade/uploads-recentes",
            get(admin_controller::listar_uploads_recentes),
        )
        .route(
            "/admin/metricas/armazenamento",
            get(admin_controller::metricas_armazenamento),
        )
        .layer(from_fn(camada_admin));

    let rotas_auth = Router::new().route(
        "/auth/admin/login",
        post(auth_controller::login_admin),
    );

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(saude_controller::raiz))
        .route("/estado", get(saude_controller::estado))
        .route("/health", get(saude_controller::saude))
        .route("/ready", get(saude_controller::pronto))
        .nest(
            "/api/v1",
            Router::new()
                .merge(rotas_auth)
                .merge(rotas_tenant)
                .merge(rotas_admin),
        )
        .layer(AddExtensionLayer::new(estado))
        .layer(cors)
}
