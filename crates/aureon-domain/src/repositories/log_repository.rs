use aureon_core::Resultado;

/// Trait de repositório de logs locais.
/// REGRA: nunca registrar senhas ou tokens.
pub trait LogRepository: Send + Sync {
    fn gravar(&self, nivel: &str, componente: &str, mensagem: &str) -> Resultado<()>;
    fn listar_recentes(&self, limite: u32) -> Resultado<Vec<EntradaLog>>;
}

/// Entrada de log armazenada no SQLite
#[derive(Debug)]
pub struct EntradaLog {
    pub id:         i64,
    pub nivel:      String,
    pub componente: String,
    pub mensagem:   String,
    pub criado_em:  String,
}
