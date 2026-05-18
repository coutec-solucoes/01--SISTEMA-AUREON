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

impl Config {
    /// Carrega configuração de variáveis de ambiente.
    /// Tenta carregar .env se existir (apenas em desenvolvimento).
    pub fn carregar() -> Result<Self, AureonError> {
        let _ = dotenvy::dotenv(); // ignora erro se .env não existir

        let porta = env::var("AUREON_API_PORTA")
            .unwrap_or_else(|_| "7070".to_string())
            .parse::<u16>()
            .map_err(|_| AureonError::Configuracao("Porta inválida".to_string()))?;

        // DATABASE_URL nunca é logada
        let database_url = env::var("DATABASE_URL").ok();

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
