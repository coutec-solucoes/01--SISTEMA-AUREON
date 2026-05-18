use std::sync::Arc;
use aureon_core::{AureonError, Resultado};
use crate::repositories::LogRepository;

/// Service de log local
pub struct LogService {
    repo: Arc<dyn LogRepository>,
}

impl LogService {
    pub fn novo(repo: Arc<dyn LogRepository>) -> Self {
        Self { repo }
    }

    /// Grava log validando campos obrigatórios
    pub fn gravar(&self, nivel: &str, componente: &str, mensagem: &str) -> Resultado<()> {
        let nivel = nivel.to_uppercase();
        let niveis_validos = ["DEBUG", "INFO", "WARN", "ERROR"];
        if !niveis_validos.contains(&nivel.as_str()) {
            return Err(AureonError::Validacao(format!(
                "Nível de log inválido: {}. Use: DEBUG, INFO, WARN, ERROR",
                nivel
            )));
        }
        if componente.trim().is_empty() {
            return Err(AureonError::Validacao(
                "Componente do log não pode ser vazio".to_string(),
            ));
        }
        // REGRA DE SEGURANÇA: nunca registrar senhas ou tokens
        let mensagem_segura = sanitizar_log(mensagem);
        self.repo.gravar(&nivel, componente.trim(), &mensagem_segura)
    }
}

/// Remove padrões sensíveis da mensagem de log
fn sanitizar_log(msg: &str) -> String {
    // Remove padrões comuns de senha/token (básico para Fase 0)
    msg.replace("password=", "password=[REDACTED]")
        .replace("senha=", "senha=[REDACTED]")
        .replace("token=", "token=[REDACTED]")
        .replace("secret=", "secret=[REDACTED]")
}
