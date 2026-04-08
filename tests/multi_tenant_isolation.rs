mod common;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde::Deserialize;
use sqlx::Row;
use tower::ServiceExt;

use common::{bearer, construir_contexto_teste, criar_projeto_teste, semear_arquivo};

#[derive(Debug, Deserialize)]
struct ArquivoDtoTeste {
    id: String,
    nome_arquivo: String,
    tipo_mime: String,
    tamanho: i64,
}

#[tokio::test]
async fn tenant_lista_apenas_os_proprios_arquivos() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let (projeto_a, api_key_a) = criar_projeto_teste(&contexto, "Projeto A").await;
    let (projeto_b, _) = criar_projeto_teste(&contexto, "Projeto B").await;

    let arquivo_a = semear_arquivo(&contexto, projeto_a, "a.txt", b"conteudo-a").await;
    semear_arquivo(&contexto, projeto_b, "b.txt", b"conteudo-b").await;

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/arquivos")
                .method("GET")
                .header(header::AUTHORIZATION, bearer(&api_key_a))
                .body(Body::empty())
                .expect("Nao foi possivel criar requisicao"),
        )
        .await
        .expect("Falha ao executar requisicao");

    assert_eq!(resposta.status(), StatusCode::OK);

    let corpo = to_bytes(resposta.into_body(), usize::MAX)
        .await
        .expect("Nao foi possivel ler corpo");
    let arquivos: Vec<ArquivoDtoTeste> =
        serde_json::from_slice(&corpo).expect("Resposta JSON invalida");

    assert_eq!(arquivos.len(), 1);
    assert_eq!(arquivos[0].id, arquivo_a);
    assert_eq!(arquivos[0].nome_arquivo, "a.txt");
    assert_eq!(arquivos[0].tipo_mime, "text/plain");
    assert_eq!(arquivos[0].tamanho, b"conteudo-a".len() as i64);
}

#[tokio::test]
async fn tenant_nao_baixa_arquivo_de_outro_tenant() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let (projeto_a, _) = criar_projeto_teste(&contexto, "Projeto A").await;
    let (_, api_key_b) = criar_projeto_teste(&contexto, "Projeto B").await;

    let arquivo_a =
        semear_arquivo(&contexto, projeto_a, "segredo.txt", b"segredo-do-tenant-a").await;

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/arquivos/{arquivo_a}"))
                .method("GET")
                .header(header::AUTHORIZATION, bearer(&api_key_b))
                .body(Body::empty())
                .expect("Nao foi possivel criar requisicao"),
        )
        .await
        .expect("Falha ao executar requisicao");

    assert_eq!(resposta.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn tenant_nao_remove_arquivo_de_outro_tenant() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let (projeto_a, _) = criar_projeto_teste(&contexto, "Projeto A").await;
    let (_, api_key_b) = criar_projeto_teste(&contexto, "Projeto B").await;

    let arquivo_a = semear_arquivo(&contexto, projeto_a, "persistente.txt", b"nao-apagar").await;

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/arquivos/{arquivo_a}"))
                .method("DELETE")
                .header(header::AUTHORIZATION, bearer(&api_key_b))
                .body(Body::empty())
                .expect("Nao foi possivel criar requisicao"),
        )
        .await
        .expect("Falha ao executar requisicao");

    assert_eq!(resposta.status(), StatusCode::NOT_FOUND);

    let registro = contexto
        .repo_arquivo
        .buscar_por_id_e_projeto(projeto_a, &arquivo_a)
        .await
        .expect("Falha ao consultar arquivo apos DELETE");
    assert!(registro.is_some());
}

#[tokio::test]
async fn api_key_invalida_recebe_401() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/arquivos")
                .method("GET")
                .header(header::AUTHORIZATION, bearer("chave-invalida"))
                .body(Body::empty())
                .expect("Nao foi possivel criar requisicao"),
        )
        .await
        .expect("Falha ao executar requisicao");

    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rls_filtra_arquivos_pelo_contexto_da_transacao() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let (projeto_a, _) = criar_projeto_teste(&contexto, "Projeto A").await;
    let (projeto_b, _) = criar_projeto_teste(&contexto, "Projeto B").await;

    semear_arquivo(&contexto, projeto_a, "a.txt", b"conteudo-a").await;
    semear_arquivo(&contexto, projeto_b, "b.txt", b"conteudo-b").await;

    let mut transacao = contexto
        .repo_arquivo
        .pool()
        .begin()
        .await
        .expect("Falha ao abrir transacao para teste de RLS");

    sqlx::query("SELECT set_config('app.current_project_id', $1, true)")
        .bind(projeto_a.to_string())
        .execute(transacao.as_mut())
        .await
        .expect("Falha ao configurar contexto do projeto no teste de RLS");

    let quantidade = sqlx::query("SELECT COUNT(*) AS total FROM arquivos")
        .fetch_one(transacao.as_mut())
        .await
        .expect("Falha ao consultar arquivos com RLS")
        .get::<i64, _>("total");

    assert_eq!(quantidade, 1);
}
