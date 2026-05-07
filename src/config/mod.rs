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
    /// Limite maximo do corpo HTTP para `POST /arquivos/lote` (varios arquivos no mesmo pedido).
    pub tamanho_maximo_corpo_lote_bytes: u64,
    /// Quantidade maxima de partes `arquivo` aceites num unico lote.
    pub max_arquivos_por_lote: usize,
    /// Cota opcional para metricas do painel (percentagem usada = bytes_usados / cota).
    pub cota_armazenamento_bytes: Option<u64>,
    /// Segredo HMAC para assinar JWT de admin (`JWT_SECRET`).
    pub jwt_secret: String,
    /// Validade do access token em segundos (`JWT_EXPIRACAO_SEGS`, ex.: 28800).
    pub jwt_expiracao_secs: u64,
    /// Se a tabela `admin_conta` estiver vazia apos migrates, criar conta com estes valores.
    pub admin_bootstrap_email: Option<String>,
    pub admin_bootstrap_password: Option<String>,
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

        let max_arquivos_por_lote = env::var("MAX_ARQUIVOS_POR_LOTE")
            .ok()
            .and_then(|valor| valor.parse::<usize>().ok())
            .unwrap_or(50)
            .clamp(1, 500);

        let tamanho_maximo_corpo_lote_bytes = env::var("TAMANHO_MAXIMO_CORPO_LOTE_BYTES")
            .ok()
            .and_then(|valor| valor.parse::<u64>().ok())
            .unwrap_or_else(|| {
                tamanho_maximo_arquivo_bytes
                    .saturating_mul(50)
                    .max(50 * 1024 * 1024)
            })
            .max(tamanho_maximo_arquivo_bytes);

        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            tracing::warn!(
                "JWT_SECRET nao definida; usando valor inseguro apenas para desenvolvimento"
            );
            "jwt-desenvolvimento-inseguro-minimo-32-caracteres-!!!!".to_string()
        });

        let jwt_expiracao_secs = env::var("JWT_EXPIRACAO_SEGS")
            .ok()
            .and_then(|valor| valor.parse::<u64>().ok())
            .unwrap_or(28_800);

        let admin_bootstrap_email = env::var("ADMIN_BOOTSTRAP_EMAIL")
            .ok()
            .map(|valor| valor.trim().to_string())
            .filter(|valor| !valor.is_empty());

        let admin_bootstrap_password = env::var("ADMIN_BOOTSTRAP_PASSWORD")
            .ok()
            .filter(|valor| !valor.is_empty());

        let cota_armazenamento_bytes = env::var("COTA_ARMAZENAMENTO_BYTES")
            .ok()
            .and_then(|valor| valor.parse::<u64>().ok())
            .filter(|&v| v > 0);

        Self {
            porta,
            diretorio_armazenamento,
            api_key_admin,
            database_url,
            base_url,
            tamanho_maximo_arquivo_bytes,
            tamanho_maximo_corpo_lote_bytes,
            max_arquivos_por_lote,
            cota_armazenamento_bytes,
            jwt_secret,
            jwt_expiracao_secs,
            admin_bootstrap_email,
            admin_bootstrap_password,
        }
    }
}
