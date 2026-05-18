use serde::{Deserialize, Serialize};
use crate::errors::AureonError;

/// Resposta padrão de todos os commands Tauri e endpoints da API
#[derive(Debug, Serialize, Deserialize)]
pub struct RespostaBase<T: Serialize> {
    pub sucesso:  bool,
    pub mensagem: String,
    pub dados:    Option<T>,
    pub erro:     Option<DetalhesErro>,
}

/// Detalhe estruturado do erro
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetalhesErro {
    pub codigo:  String,
    pub detalhe: String,
}

impl<T: Serialize> RespostaBase<T> {
    /// Cria resposta de sucesso
    pub fn ok(mensagem: impl Into<String>, dados: T) -> Self {
        Self {
            sucesso:  true,
            mensagem: mensagem.into(),
            dados:    Some(dados),
            erro:     None,
        }
    }

    /// Cria resposta de erro a partir de AureonError
    pub fn falha(mensagem: impl Into<String>, erro: &AureonError) -> Self {
        Self {
            sucesso:  false,
            mensagem: mensagem.into(),
            dados:    None,
            erro:     Some(DetalhesErro {
                codigo:  erro.codigo().to_string(),
                detalhe: erro.to_string(),
            }),
        }
    }

    /// Cria resposta de erro com código e detalhe manuais
    pub fn falha_manual(
        mensagem: impl Into<String>,
        codigo:   impl Into<String>,
        detalhe:  impl Into<String>,
    ) -> Self {
        Self {
            sucesso:  false,
            mensagem: mensagem.into(),
            dados:    None,
            erro:     Some(DetalhesErro {
                codigo:  codigo.into(),
                detalhe: detalhe.into(),
            }),
        }
    }
}
