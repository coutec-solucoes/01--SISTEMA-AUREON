use std::sync::Arc;
use aureon_core::{AureonError, Resultado};
use crate::repositories::ConfiguracaoRepository;

/// Service de configuração local — camada de lógica entre command e repository.
/// Nunca expõe valor descriptografado ao chamador sem necessidade explícita.
pub struct ConfiguracaoService {
    repo: Arc<dyn ConfiguracaoRepository>,
}

impl ConfiguracaoService {
    pub fn novo(repo: Arc<dyn ConfiguracaoRepository>) -> Self {
        Self { repo }
    }

    /// Retorna o valor criptografado (nunca o puro) para exibição/auditoria
    pub fn obter_criptografado(&self, chave: &str) -> Resultado<Option<String>> {
        self.repo.obter(chave)
    }

    /// Valida e persiste valor já criptografado
    pub fn salvar_criptografado(&self, chave: &str, valor_criptografado: &str) -> Resultado<()> {
        if chave.trim().is_empty() {
            return Err(AureonError::Validacao("Chave não pode ser vazia".to_string()));
        }
        if valor_criptografado.trim().is_empty() {
            return Err(AureonError::Validacao(
                "Valor criptografado não pode ser vazio".to_string(),
            ));
        }
        self.repo.salvar(chave, valor_criptografado)
    }

    pub fn listar_chaves(&self) -> Resultado<Vec<String>> {
        self.repo.listar_chaves()
    }
}
