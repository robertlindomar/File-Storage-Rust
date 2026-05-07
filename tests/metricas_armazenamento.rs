mod common;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde::Deserialize;
use tower::ServiceExt;

use common::{
    OpcoesContextoTeste, bearer, construir_contexto_teste, construir_contexto_teste_com,
    criar_projeto_teste, semear_arquivo,
};

#[derive(Debug, Deserialize)]
struct MetricasTeste {
    bytes_usados: i64,
    bytes_cota: Option<u64>,
    percentagem: Option<f64>,
}

#[tokio::test]
async fn metricas_soma_bytes_sem_arquivos() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/metricas/armazenamento")
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
    let m: MetricasTeste = serde_json::from_slice(&corpo).expect("json");
    assert_eq!(m.bytes_usados, 0);
    assert!(m.bytes_cota.is_none());
    assert!(m.percentagem.is_none());
}

#[tokio::test]
async fn metricas_soma_varios_projetos() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let (projeto_a, _) = criar_projeto_teste(&contexto, "A").await;
    let (projeto_b, _) = criar_projeto_teste(&contexto, "B").await;
    semear_arquivo(&contexto, projeto_a, "a.txt", b"12345").await;
    semear_arquivo(&contexto, projeto_b, "b.txt", b"ab").await;

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/metricas/armazenamento")
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
    let m: MetricasTeste = serde_json::from_slice(&corpo).expect("json");
    assert_eq!(m.bytes_usados, 7);
}

#[tokio::test]
async fn metricas_com_cota_devolve_percentagem() {
    let Some(contexto) = construir_contexto_teste_com(OpcoesContextoTeste {
        cota_armazenamento_bytes: Some(100),
        ..Default::default()
    })
    .await
    else {
        return;
    };

    let (pid, _) = criar_projeto_teste(&contexto, "C").await;
    semear_arquivo(&contexto, pid, "x.txt", &[0u8; 40]).await;

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/metricas/armazenamento")
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
    let m: MetricasTeste = serde_json::from_slice(&corpo).expect("json");
    assert_eq!(m.bytes_usados, 40);
    assert_eq!(m.bytes_cota, Some(100));
    assert!((m.percentagem.unwrap() - 40.0).abs() < 0.01);
}

#[tokio::test]
async fn metricas_sem_admin_401() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/metricas/armazenamento")
                .method("GET")
                .header(header::AUTHORIZATION, bearer("chave-invalida"))
                .body(Body::empty())
                .expect("requisicao"),
        )
        .await
        .expect("oneshot");

    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);
}
