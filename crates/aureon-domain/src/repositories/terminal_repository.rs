use aureon_core::Resultado;

/// Trait de repositório de terminais
pub trait TerminalRepository: Send + Sync {
    fn obter_id_terminal(&self) -> Resultado<Option<String>>;
    fn registrar_terminal(&self, terminal_id: &str, nome: &str) -> Resultado<()>;
}
