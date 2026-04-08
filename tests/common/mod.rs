use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::Router;
use file_storage_service::{
    config::ConfiguracaoAplicacao,
    construir_estado_aplicacao,
    repositories::{
        arquivo_repository::RepositorioArquivo, projeto_repository::RepositorioProjeto,
    },
    rotas,
};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

pub struct DiretoriaTemporaria {
    caminho: PathBuf,
}

impl DiretoriaTemporaria {
    fn nova(prefixo: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duracao| duracao.as_nanos())
            .unwrap_or_default();
        let caminho = std::env::temp_dir().join(format!("{prefixo}-{nanos}-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&caminho)
            .expect("Nao foi possivel criar diretorio temporario do teste");
        Self { caminho }
    }

    fn caminho(&self) -> &Path {
        &self.caminho
    }
}

impl Drop for DiretoriaTemporaria {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.caminho);
    }
}

pub struct ContextoTeste {
    pub app: Router,
    pub repo_arquivo: Arc<RepositorioArquivo>,
    pub repo_projeto: Arc<RepositorioProjeto>,
    _diretorio: DiretoriaTemporaria,
}

pub async fn construir_contexto_teste() -> Option<ContextoTeste> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:postgres@localhost:5432/file_storage".to_string()
    });

    let pool = match PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(erro) => {
            eprintln!("Ignorando teste de isolamento: PostgreSQL indisponivel ({erro})");
            return None;
        }
    };

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Falha ao executar migracoes SQL para os testes");

    let diretorio = DiretoriaTemporaria::nova("file-storage-service-testes");
    let configuracao = Arc::new(ConfiguracaoAplicacao {
        porta: 3001,
        diretorio_armazenamento: diretorio.caminho().to_string_lossy().to_string(),
        api_key_admin: "admin-chave-teste".to_string(),
        database_url,
        base_url: Some("http://localhost:3001".to_string()),
        tamanho_maximo_arquivo_bytes: 1024 * 1024,
    });

    let estado = construir_estado_aplicacao(configuracao, pool.clone());
    let repo_projeto = Arc::new(RepositorioProjeto::novo(pool.clone()));
    let repo_arquivo = Arc::new(RepositorioArquivo::novo(
        pool,
        estado.configuracao.diretorio_armazenamento.clone(),
    ));

    Some(ContextoTeste {
        app: rotas::criar_rotas(estado.clone()),
        repo_arquivo,
        repo_projeto,
        _diretorio: diretorio,
    })
}

pub async fn criar_projeto_teste(contexto: &ContextoTeste, nome: &str) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let api_key = format!("test-key-{}", Uuid::new_v4());
    contexto
        .repo_projeto
        .inserir(id, nome.to_string(), api_key.clone())
        .await
        .expect("Nao foi possivel criar projeto de teste");
    (id, api_key)
}

pub async fn semear_arquivo(
    contexto: &ContextoTeste,
    projeto_id: Uuid,
    nome_arquivo: &str,
    conteudo: &[u8],
) -> String {
    let id = Uuid::new_v4().to_string();
    let caminho = contexto
        .repo_arquivo
        .caminho_absoluto_arquivo(projeto_id, &id);

    if let Some(pasta) = Path::new(&caminho).parent() {
        tokio::fs::create_dir_all(pasta)
            .await
            .expect("Nao foi possivel preparar pasta do projeto");
    }

    tokio::fs::write(&caminho, conteudo)
        .await
        .expect("Nao foi possivel escrever arquivo de teste");

    contexto
        .repo_arquivo
        .inserir_registro(
            projeto_id,
            id.clone(),
            nome_arquivo.to_string(),
            caminho,
            "text/plain".to_string(),
            conteudo.len() as i64,
        )
        .await
        .expect("Nao foi possivel registrar arquivo de teste");

    id
}

pub fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}
