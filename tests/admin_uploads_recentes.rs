mod common;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde::Deserialize;
use tower::ServiceExt;
use uuid::Uuid;

use common::{bearer, construir_contexto_teste, criar_projeto_teste, semear_arquivo};

#[derive(Debug, Deserialize)]
struct AtividadeUploadTeste {
    nome_arquivo: String,
    projeto_id: Uuid,
    projeto_nome: String,
    tamanho: i64,
    criado_em: chrono::DateTime<chrono::Utc>,
}

#[tokio::test]
async fn admin_lista_uploads_recentes_entre_projetos() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let (projeto_a, _) = criar_projeto_teste(&contexto, "Projeto-A").await;
    let (projeto_b, _) = criar_projeto_teste(&contexto, "Projeto-B").await;

    semear_arquivo(&contexto, projeto_a, "older.txt", b"x").await;
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    semear_arquivo(&contexto, projeto_b, "newer.txt", b"yy").await;

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/atividade/uploads-recentes?limite=10")
                .method("GET")
                .header(header::AUTHORIZATION, bearer("admin-chave-teste"))
                .body(Body::empty())
                .expect("requisicao"),
        )
        .await
        .expect("oneshot");

    assert_eq!(resposta.status(), StatusCode::OK);

    let corpo = to_bytes(resposta.into_body(), usize::MAX)
        .await
        .expect("corpo");
    let lista: Vec<AtividadeUploadTeste> =
        serde_json::from_slice(&corpo).expect("json");

    assert_eq!(lista.len(), 2);
    assert_eq!(lista[0].nome_arquivo, "newer.txt");
    assert_eq!(lista[0].projeto_id, projeto_b);
    assert_eq!(lista[0].projeto_nome, "Projeto-B");
    assert_eq!(lista[0].tamanho, 2);
    assert!(lista[0].criado_em <= chrono::Utc::now());
    assert_eq!(lista[1].nome_arquivo, "older.txt");
    assert_eq!(lista[1].projeto_id, projeto_a);
    assert_eq!(lista[1].projeto_nome, "Projeto-A");
}

#[tokio::test]
async fn uploads_recentes_sem_admin_recebe_401() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/atividade/uploads-recentes")
                .method("GET")
                .body(Body::empty())
                .expect("requisicao"),
        )
        .await
        .expect("oneshot");

    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);
}
