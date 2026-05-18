use aureon_core::Resultado;

/// Trait de acesso a configurações locais criptografadas.
/// REGRA: valor NUNCA trafega em texto puro após salvo.
pub trait ConfiguracaoRepository: Send + Sync {
    fn obter(&self, chave: &str) -> Resultado<Option<String>>;
    fn salvar(&self, chave: &str, valor_criptografado: &str) -> Resultado<()>;
    fn remover(&self, chave: &str) -> Resultado<()>;
    fn listar_chaves(&self) -> Resultado<Vec<String>>;
}
