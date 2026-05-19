use std::env;
use aureon_core::AureonError;

/// Configuração da API Local.
/// Valores sensíveis NUNCA são logados.
#[derive(Debug, Clone)]
pub struct Config {
    pub porta:       u16,
    pub database_url: Option<String>,
    pub ambiente:    String,
    pub versao:      String,
}

#[derive(serde::Deserialize)]
struct AppServerConfig {
    postgres_host: String,
    postgres_porta: u16,
    postgres_usuario: String,
    postgres_senha: String,
    postgres_banco: String,
}

impl Config {
    /// Carrega configuração de variáveis de ambiente.
    /// Tenta carregar .env se existir (apenas em desenvolvimento).
    pub fn carregar() -> Result<Self, AureonError> {
        let _ = dotenvy::dotenv(); // ignora erro se .env não existir

        let porta = env::var("AUREON_API_PORTA")
            .unwrap_or_else(|_| "7070".to_string())
            .parse::<u16>()
            .map_err(|_| AureonError::Configuracao("Porta inválida".to_string()))?;

        // DATABASE_URL nunca é logada. Tenta do ambiente, senão decripta do cofre técnico
        let mut database_url = env::var("DATABASE_URL").ok();

        if database_url.is_none() {
            let config_path = std::path::Path::new("C:/Aureon/config/server.config.enc");
            let keystore_path = std::path::Path::new("C:/Aureon/config/.keystore");
            if config_path.exists() && keystore_path.exists() {
                if let Ok(server_config) = aureon_shared::config_store::ler_config_criptografada::<AppServerConfig>(
                    config_path,
                    keystore_path,
                ) {
                    database_url = Some(format!(
                        "postgres://{}:{}@{}:{}/{}",
                        server_config.postgres_usuario,
                        server_config.postgres_senha,
                        server_config.postgres_host,
                        server_config.postgres_porta,
                        server_config.postgres_banco
                    ));
                }
            }
        }

        let ambiente = env::var("AUREON_AMBIENTE")
            .unwrap_or_else(|_| "desenvolvimento".to_string());

        Ok(Self {
            porta,
            database_url,
            ambiente,
            versao: "0.0.1".to_string(),
        })
    }

    /// Configuração padrão usada quando carregar() falha
    pub fn padrao() -> Self {
        Self {
            porta: 7070,
            database_url: None,
            ambiente: "desenvolvimento".to_string(),
            versao: "0.0.1".to_string(),
        }
    }
}
