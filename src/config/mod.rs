use std::env;

/// Configuracoes da aplicacao carregadas de variaveis de ambiente.
#[derive(Clone, Debug)]
pub struct ConfiguracaoAplicacao {
    pub porta: u16,
    pub diretorio_armazenamento: String,
    /// Chave para rotas administrativas (`/api/v1/admin/*`).
    pub api_key_admin: String,
    /// URL JDBC/Postgres, ex.: `postgresql://user:pass@localhost:5432/file_storage`
    pub database_url: String,
    /// Base publica para URLs devolvidas no upload (ex.: `http://10.0.0.1:3000`). Opcional.
    pub base_url: Option<String>,
    /// Limite maximo de bytes por upload (corpo multipart).
    pub tamanho_maximo_arquivo_bytes: u64,
}

impl ConfiguracaoAplicacao {
    pub fn carregar() -> Self {
        let porta = env::var("PORTA")
            .ok()
            .and_then(|valor| valor.parse::<u16>().ok())
            .unwrap_or(3000);

        let diretorio_armazenamento =
            env::var("DIRETORIO_ARMAZENAMENTO").unwrap_or_else(|_| "./armazenamento".to_string());

        let api_key_admin = env::var("API_KEY_ADMIN")
            .unwrap_or_else(|_| "admin-chave-local-trocar".to_string());

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:postgres@localhost:5432/file_storage".to_string()
        });

        let base_url = env::var("BASE_URL").ok().filter(|valor| !valor.trim().is_empty());

        // Default: 100 MiB
        let tamanho_maximo_arquivo_bytes = env::var("TAMANHO_MAXIMO_ARQUIVO_BYTES")
            .ok()
            .and_then(|valor| valor.parse::<u64>().ok())
            .unwrap_or(100 * 1024 * 1024);

        Self {
            porta,
            diretorio_armazenamento,
            api_key_admin,
            database_url,
            base_url,
            tamanho_maximo_arquivo_bytes,
        }
    }
}
