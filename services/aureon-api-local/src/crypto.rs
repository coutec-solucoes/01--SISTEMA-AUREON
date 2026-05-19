use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use crate::erros::ErroApi;

pub fn hash_senha(senha: &str) -> Result<String, ErroApi> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let hash = argon2
        .hash_password(senha.as_bytes(), &salt)
        .map_err(|e| ErroApi::interno(format!("Erro ao gerar hash: {}", e)))?
        .to_string();
        
    Ok(hash)
}

pub fn verificar_senha(senha_informada: &str, hash_salvo: &str) -> Result<bool, ErroApi> {
    let parsed_hash = PasswordHash::new(hash_salvo)
        .map_err(|e| ErroApi::interno(format!("Hash inválido no banco: {}", e)))?;
        
    let verificado = Argon2::default()
        .verify_password(senha_informada.as_bytes(), &parsed_hash)
        .is_ok();
        
    Ok(verificado)
}
