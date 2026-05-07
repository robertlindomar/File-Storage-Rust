mod common;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde::Deserialize;
use tower::ServiceExt;

use common::{
    OpcoesContextoTeste, bearer, construir_contexto_teste, construir_contexto_teste_com,
    criar_projeto_teste,
};

#[derive(Debug, Deserialize)]
struct RespostaUploadTeste {
    id: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct FalhaLoteTeste {
    nome_arquivo: String,
    erro: String,
}

#[derive(Debug, Deserialize)]
struct RespostaLoteTeste {
    sucesso: Vec<RespostaUploadTeste>,
    falhas: Vec<FalhaLoteTeste>,
}

fn corpo_multipart_lote(arquivos: &[(&str, &[u8])]) -> (String, Vec<u8>) {
    let boundary = "boundaryLoteTeste987";
    let mut v = Vec::new();
    for (nome, bytes) in arquivos {
        v.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        v.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"arquivo\"; filename=\"{nome}\"\r\n\r\n")
                .as_bytes(),
        );
        v.extend_from_slice(bytes);
        v.extend_from_slice(b"\r\n");
    }
    v.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (boundary.to_string(), v)
}

#[tokio::test]
async fn lote_dois_arquivos_ok() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let (_projeto_id, api_key) = criar_projeto_teste(&contexto, "Lote A").await;
    let (boundary, corpo) = corpo_multipart_lote(&[("a.txt", b"aaa"), ("b.txt", b"bbbb")]);

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/arquivos/lote")
                .method("POST")
                .header(header::AUTHORIZATION, bearer(&api_key))
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(corpo))
                .expect("requisicao"),
        )
        .await
        .expect("exec");

    assert_eq!(resposta.status(), StatusCode::OK);
    let bytes = to_bytes(resposta.into_body(), usize::MAX)
        .await
        .expect("corpo");
    let json: RespostaLoteTeste = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json.sucesso.len(), 2);
    assert!(json.falhas.is_empty());
    assert!(!json.sucesso[0].id.is_empty());
    assert!(json.sucesso[0].url.contains("/api/v1/arquivos/"));
}

#[tokio::test]
async fn lote_arquivo_vazio_vai_para_falhas_outro_ok() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let (_projeto_id, api_key) = criar_projeto_teste(&contexto, "Lote B").await;
    let (boundary, corpo) = corpo_multipart_lote(&[("ok.txt", b"x"), ("vazio.txt", b"")]);

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/arquivos/lote")
                .method("POST")
                .header(header::AUTHORIZATION, bearer(&api_key))
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(corpo))
                .expect("requisicao"),
        )
        .await
        .expect("exec");

    assert_eq!(resposta.status(), StatusCode::OK);
    let bytes = to_bytes(resposta.into_body(), usize::MAX)
        .await
        .expect("corpo");
    let json: RespostaLoteTeste = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json.sucesso.len(), 1);
    assert_eq!(json.falhas.len(), 1);
    assert_eq!(json.falhas[0].nome_arquivo, "vazio.txt");
    assert!(!json.falhas[0].erro.is_empty());
}

#[tokio::test]
async fn lote_rejeita_acima_do_max_config() {
    let Some(contexto) = construir_contexto_teste_com(OpcoesContextoTeste {
        max_arquivos_por_lote: Some(2),
        ..Default::default()
    })
    .await
    else {
        return;
    };

    let (_projeto_id, api_key) = criar_projeto_teste(&contexto, "Lote C").await;
    let (boundary, corpo) =
        corpo_multipart_lote(&[("1.txt", b"a"), ("2.txt", b"b"), ("3.txt", b"c")]);

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/arquivos/lote")
                .method("POST")
                .header(header::AUTHORIZATION, bearer(&api_key))
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(corpo))
                .expect("requisicao"),
        )
        .await
        .expect("exec");

    assert_eq!(resposta.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn lote_sem_campo_arquivo_400() {
    let Some(contexto) = construir_contexto_teste().await else {
        return;
    };

    let (_projeto_id, api_key) = criar_projeto_teste(&contexto, "Lote D").await;
    let boundary = "boundarySolo";
    let corpo = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"outro\"; filename=\"x.txt\"\r\n\r\nhi\r\n--{boundary}--\r\n"
    );

    let resposta = contexto
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/arquivos/lote")
                .method("POST")
                .header(header::AUTHORIZATION, bearer(&api_key))
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(corpo.into_bytes()))
                .expect("requisicao"),
        )
        .await
        .expect("exec");

    assert_eq!(resposta.status(), StatusCode::BAD_REQUEST);
}
