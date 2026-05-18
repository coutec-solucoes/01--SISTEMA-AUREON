// =============================================================
// ESTRATÉGIA DE CRIPTOGRAFIA — FASE 0
// =============================================================
// A chave AES-256 é gerada aleatoriamente na PRIMEIRA inicialização
// e armazenada em: C:/Aureon/config/.keystore (acesso restrito ao SO)
//
// NUNCA salvar:
//   - senhas em texto puro
//   - tokens em texto puro
//   - chave em código-fonte
//
// Evolução futura:
//   - Windows DPAPI para proteger o .keystore
//   - Chave derivada de hardware (TPM) em versão enterprise
// =============================================================

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use aureon_core::AureonError;

const NONCE_LEN: usize = 12; // 96 bits — padrão AES-GCM

/// Gera chave AES-256 aleatória (32 bytes).
/// Deve ser chamado apenas na PRIMEIRA inicialização.
pub fn gerar_chave() -> Vec<u8> {
    let mut chave = vec![0u8; 32];
    OsRng.fill_bytes(&mut chave);
    chave
}

/// Serializa chave para Base64 (armazenamento em arquivo)
pub fn chave_para_base64(chave: &[u8]) -> String {
    general_purpose::STANDARD.encode(chave)
}

/// Desserializa chave de Base64
pub fn chave_de_base64(b64: &str) -> Result<Vec<u8>, AureonError> {
    general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| AureonError::Criptografia(format!("Chave Base64 inválida: {e}")))
}

/// Criptografa `valor` com AES-256-GCM.
/// Retorna string no formato `<nonce_b64>:<ciphertext_b64>`.
pub fn criptografar(valor: &str, chave: &[u8]) -> Result<String, AureonError> {
    let cipher = Aes256Gcm::new_from_slice(chave)
        .map_err(|e| AureonError::Criptografia(e.to_string()))?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, valor.as_bytes())
        .map_err(|e| AureonError::Criptografia(e.to_string()))?;

    let resultado = format!(
        "{}:{}",
        general_purpose::STANDARD.encode(nonce_bytes),
        general_purpose::STANDARD.encode(ciphertext)
    );
    Ok(resultado)
}

/// Descriptografa string no formato `<nonce_b64>:<ciphertext_b64>`.
pub fn descriptografar(valor_enc: &str, chave: &[u8]) -> Result<String, AureonError> {
    let partes: Vec<&str> = valor_enc.splitn(2, ':').collect();
    if partes.len() != 2 {
        return Err(AureonError::Criptografia(
            "Formato de valor criptografado inválido".to_string(),
        ));
    }

    let nonce_bytes = general_purpose::STANDARD
        .decode(partes[0])
        .map_err(|e| AureonError::Criptografia(e.to_string()))?;

    let ciphertext = general_purpose::STANDARD
        .decode(partes[1])
        .map_err(|e| AureonError::Criptografia(e.to_string()))?;

    let cipher = Aes256Gcm::new_from_slice(chave)
        .map_err(|e| AureonError::Criptografia(e.to_string()))?;

    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| {
            AureonError::Criptografia(
                "Falha na descriptografia — chave incorreta ou dados corrompidos".to_string(),
            )
        })?;

    String::from_utf8(plaintext)
        .map_err(|e| AureonError::Criptografia(e.to_string()))
}
