/// licenca_crypto.rs — Fase 20 Bloco 4
/// Assinatura Ed25519 de payloads de licença.
///
/// REGRAS DE SEGURANÇA:
/// - Chave privada NUNCA é logada, retornada ou enviada ao PDV.
/// - Em modo DEV a chave é gerada/persistida em memória estática.
/// - Em produção, a chave deve vir de variável de ambiente AUREON_LICENSE_PRIVATE_KEY_B64.
/// - A chave pública pode ser distribuída livremente ao PDV.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use std::sync::OnceLock;

/// Algoritmo registrado oficialmente neste bloco.
pub const ALGORITMO: &str = "Ed25519";

/// Estrutura interna de chave (mantida em memória na Retaguarda).
/// A chave privada é SOMENTE interna — nunca serializada para fora.
pub struct ChaveLicenca {
    /// signing_key contém internamente a chave privada (não pub).
    signing_key: SigningKey,
    /// Chave pública derivada — pode ser distribuída.
    pub verifying_key: VerifyingKey,
    /// Identificador desta instância de chave (rotação futura).
    pub key_id: String,
    /// true = chave gerada em runtime para DEV (sem persistência).
    pub is_dev: bool,
}

impl ChaveLicenca {
    /// Cria chave a partir de variável de ambiente (produção)
    /// ou gera chave efêmera DEV se não configurada.
    pub fn inicializar() -> Self {
        // Tenta ler chave privada do ambiente
        if let Ok(privkey_b64) = std::env::var("AUREON_LICENSE_PRIVATE_KEY_B64") {
            match B64.decode(privkey_b64.trim()) {
                Ok(bytes) if bytes.len() == 32 => {
                    let key_bytes: [u8; 32] = bytes.try_into().unwrap();
                    let signing_key = SigningKey::from_bytes(&key_bytes);
                    let verifying_key = signing_key.verifying_key();
                    return ChaveLicenca {
                        signing_key,
                        verifying_key,
                        key_id: "prod-v1".to_string(),
                        is_dev: false,
                    };
                }
                _ => {
                    tracing::warn!(
                        "[licenca_crypto] AUREON_LICENSE_PRIVATE_KEY_B64 inválida — usando chave DEV efêmera."
                    );
                }
            }
        }

        // Fallback: chave DEV efêmera gerada com OsRng
        tracing::warn!(
            "[licenca_crypto] Nenhuma chave de produção configurada. Gerando chave DEV efêmera. \
             NÃO USE EM PRODUÇÃO."
        );
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        ChaveLicenca {
            signing_key,
            verifying_key,
            key_id: "dev-efemero-v1".to_string(),
            is_dev: true,
        }
    }

    /// Retorna chave pública em base64 (seguro para distribuição ao PDV).
    pub fn chave_publica_b64(&self) -> String {
        B64.encode(self.verifying_key.as_bytes())
    }
}

/// Instância global — inicializada uma única vez no startup.
/// A chave privada fica apenas nesta estrutura em memória.
static CHAVE_GLOBAL: OnceLock<ChaveLicenca> = OnceLock::new();

/// Obtém ou inicializa a chave global de licença.
pub fn chave_global() -> &'static ChaveLicenca {
    CHAVE_GLOBAL.get_or_init(ChaveLicenca::inicializar)
}

// ─────────────────────────────────────────────────────────────────
// CANONICALIZAÇÃO DO PAYLOAD
// ─────────────────────────────────────────────────────────────────

/// Campos obrigatórios para assinatura (em ordem canônica fixa).
/// A ordem é SEMPRE a mesma — não depende de serialização JSON instável.
#[derive(Debug)]
pub struct PayloadCanonicoLicenca<'a> {
    pub empresa_id: &'a str,
    pub licenca_id: &'a str,
    pub plano_codigo: &'a str,
    pub status: &'a str,
    pub validade: &'a str,      // ISO-8601 ou "null"
    pub terminal: &'a str,
    pub tolerancia_offline_dias: i64,
    pub emitido_em: &'a str,    // ISO-8601
}

impl<'a> PayloadCanonicoLicenca<'a> {
    /// Serializa em JSON canônico deterministico com campos em ordem fixa.
    ///
    /// Regras de canonicalização:
    /// 1. Campos em ordem fixa declarada nesta struct.
    /// 2. Sem espaços extras, sem quebras de linha.
    /// 3. Sem campos ausentes — todos obrigatórios.
    /// 4. Sem float/double — apenas strings e inteiros.
    /// 5. Strings escapadas conforme JSON RFC 8259.
    pub fn to_canonical_json(&self) -> String {
        format!(
            "{{\"empresa_id\":\"{}\",\"licenca_id\":\"{}\",\"plano_codigo\":\"{}\",\
             \"status\":\"{}\",\"validade\":\"{}\",\"terminal\":\"{}\",\
             \"tolerancia_offline_dias\":{},\"emitido_em\":\"{}\"}}",
            escape_json(self.empresa_id),
            escape_json(self.licenca_id),
            escape_json(self.plano_codigo),
            escape_json(self.status),
            escape_json(self.validade),
            escape_json(self.terminal),
            self.tolerancia_offline_dias,
            escape_json(self.emitido_em),
        )
    }
}

/// Escapa caracteres especiais JSON de forma minimal e segura.
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

// ─────────────────────────────────────────────────────────────────
// OPERAÇÕES CRIPTOGRÁFICAS
// ─────────────────────────────────────────────────────────────────

/// Resultado de uma operação de assinatura.
pub struct ResultadoAssinatura {
    pub payload_json: String,
    pub payload_hash_hex: String,
    pub assinatura_b64: String,
    pub key_id: String,
    pub is_dev: bool,
}

/// Assina o payload canônico da licença.
///
/// Processo:
/// 1. Serializa payload em JSON canônico determinístico.
/// 2. Calcula SHA-256 do JSON → payload_hash (hex).
/// 3. Assina os bytes do hash com Ed25519 (chave privada fica na Retaguarda).
/// 4. Retorna assinatura em base64 + hash + payload.
pub fn assinar_payload(payload: &PayloadCanonicoLicenca) -> ResultadoAssinatura {
    let chave = chave_global();
    let json = payload.to_canonical_json();

    // SHA-256 do payload canônico
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let hash_bytes = hasher.finalize();
    let hash_hex = hex::encode(hash_bytes.as_slice());

    // Assinar os bytes do hash com Ed25519
    // NOTA: A chave privada nunca sai deste processo.
    let assinatura: Signature = chave.signing_key.sign(hash_bytes.as_slice());
    let assinatura_b64 = B64.encode(assinatura.to_bytes());

    ResultadoAssinatura {
        payload_json: json,
        payload_hash_hex: hash_hex,
        assinatura_b64,
        key_id: chave.key_id.clone(),
        is_dev: chave.is_dev,
    }
}

/// Resultado da verificação de um payload assinado.
pub struct ResultadoVerificacao {
    pub valido: bool,
    pub payload_hash_calculado: String,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

/// Verifica a assinatura de um payload de licença.
///
/// Processo:
/// 1. Recalcula SHA-256 do payload_json recebido.
/// 2. Decodifica assinatura de base64 → bytes.
/// 3. Verifica assinatura com chave pública Ed25519.
/// 4. Se payload_hash opcional foi fornecido, verifica consistência.
pub fn verificar_payload(
    payload_json: &str,
    assinatura_b64: &str,
    key_id: &str,
    payload_hash_informado: Option<&str>,
) -> ResultadoVerificacao {
    let chave = chave_global();
    let mut warnings = Vec::new();

    // Verificar key_id
    if key_id != chave.key_id {
        return ResultadoVerificacao {
            valido: false,
            payload_hash_calculado: String::new(),
            mensagem: format!(
                "key_id '{}' desconhecido. Chave ativa: '{}'.",
                key_id, chave.key_id
            ),
            warnings,
        };
    }

    // Calcular hash do payload recebido
    let mut hasher = Sha256::new();
    hasher.update(payload_json.as_bytes());
    let hash_bytes = hasher.finalize();
    let hash_hex = hex::encode(hash_bytes.as_slice());

    // Verificar consistência do hash opcional
    if let Some(hash_info) = payload_hash_informado {
        if hash_info != hash_hex {
            return ResultadoVerificacao {
                valido: false,
                payload_hash_calculado: hash_hex,
                mensagem: "payload_hash informado não corresponde ao hash calculado. Payload pode ter sido adulterado.".to_string(),
                warnings,
            };
        }
    }

    // Decodificar assinatura base64
    let sig_bytes = match B64.decode(assinatura_b64) {
        Ok(b) => b,
        Err(e) => {
            return ResultadoVerificacao {
                valido: false,
                payload_hash_calculado: hash_hex,
                mensagem: format!("Assinatura base64 inválida: {}", e),
                warnings,
            };
        }
    };

    // Converter para Signature Ed25519 (64 bytes)
    let sig_array: [u8; 64] = match sig_bytes.try_into() {
        Ok(a) => a,
        Err(_) => {
            return ResultadoVerificacao {
                valido: false,
                payload_hash_calculado: hash_hex,
                mensagem: "Assinatura Ed25519 deve ter exatamente 64 bytes.".to_string(),
                warnings,
            };
        }
    };
    let signature = Signature::from_bytes(&sig_array);

    // Verificar com chave pública
    match chave.verifying_key.verify(hash_bytes.as_slice(), &signature) {
        Ok(_) => {
            if chave.is_dev {
                warnings.push(
                    "Verificação realizada com chave DEV efêmera. Em produção use chave persistente."
                        .to_string(),
                );
            }
            ResultadoVerificacao {
                valido: true,
                payload_hash_calculado: hash_hex,
                mensagem: "Assinatura válida. Payload autêntico e não adulterado.".to_string(),
                warnings,
            }
        }
        Err(_) => ResultadoVerificacao {
            valido: false,
            payload_hash_calculado: hash_hex,
            mensagem: "Assinatura INVÁLIDA. Payload pode ter sido adulterado ou chave incorreta."
                .to_string(),
            warnings,
        },
    }
}
