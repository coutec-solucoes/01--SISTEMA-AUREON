use aureon_core::AureonError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::crypto::{criptografar, descriptografar, carregar_keystore};

/// Salva uma estrutura configurável em formato JSON criptografado num arquivo `.enc`
pub fn salvar_config_criptografada<T: Serialize>(
    dados: &T,
    caminho_arquivo: &Path,
    caminho_keystore: &Path,
) -> Result<(), AureonError> {
    // 1. Carrega a chave
    let chave = carregar_keystore(caminho_keystore)?;

    // 2. Serializa para JSON
    let json_str = serde_json::to_string(dados)
        .map_err(|e| AureonError::Criptografia(format!("Falha ao serializar config: {}", e)))?;

    // 3. Criptografa
    let texto_criptografado = criptografar(&json_str, &chave)?;

    // 4. Garante que o diretório existe
    if let Some(parent) = caminho_arquivo.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AureonError::Criptografia(format!("Falha ao criar diretório de config: {}", e)))?;
    }

    // 5. Salva no disco
    fs::write(caminho_arquivo, texto_criptografado)
        .map_err(|e| AureonError::Criptografia(format!("Falha ao salvar arquivo .enc: {}", e)))?;

    Ok(())
}

/// Lê uma estrutura configurável de um arquivo `.enc` e a descriptografa
pub fn ler_config_criptografada<T: for<'a> Deserialize<'a>>(
    caminho_arquivo: &Path,
    caminho_keystore: &Path,
) -> Result<T, AureonError> {
    if !caminho_arquivo.exists() {
        return Err(AureonError::Criptografia("Arquivo de configuração não encontrado".into()));
    }

    // 1. Carrega a chave
    let chave = carregar_keystore(caminho_keystore)?;

    // 2. Lê o arquivo
    let texto_criptografado = fs::read_to_string(caminho_arquivo)
        .map_err(|e| AureonError::Criptografia(format!("Falha ao ler arquivo .enc: {}", e)))?;

    // 3. Descriptografa
    let json_str = descriptografar(&texto_criptografado, &chave)?;

    // 4. Deserializa
    let dados: T = serde_json::from_str(&json_str)
        .map_err(|e| AureonError::Criptografia(format!("Falha ao deserializar config: {}", e)))?;

    Ok(dados)
}
