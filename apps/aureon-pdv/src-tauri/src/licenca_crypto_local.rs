/// licenca_crypto_local.rs — Fase 20 Bloco 5
/// Verificação Ed25519 de payload de licença no PDV (lado offline).
///
/// REGRAS DE SEGURANÇA:
/// - O PDV NUNCA armazena chave privada.
/// - Apenas a chave pública é usada aqui.
/// - A chave pública pode vir do banco local (licenca_chaves) ou ser informada em DEV.
/// - Payload com assinatura inválida NUNCA é aplicado.
/// - Modo DEV tem aviso explícito.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};

/// Algoritmo suportado nesta versão.
pub const ALGORITMO_SUPORTADO: &str = "Ed25519";

/// Resultado de uma verificação local de payload assinado.
pub struct ResultadoVerificacaoLocal {
    pub valido: bool,
    pub payload_hash_calculado: String,
    pub mensagem: String,
    pub warnings: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────
// HASHING
// ─────────────────────────────────────────────────────────────────

/// Calcula SHA-256 do payload_licenca_json e retorna em hex lowercase.
///
/// O hash é calculado sobre os bytes UTF-8 do JSON canônico exatamente
/// como feito pela Retaguarda em licenca_crypto.rs.
pub fn calcular_hash_payload(payload_json: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload_json.as_bytes());
    let hash_bytes = hasher.finalize();
    hex::encode(hash_bytes.as_slice())
}

// ─────────────────────────────────────────────────────────────────
// VERIFICAÇÃO ED25519
// ─────────────────────────────────────────────────────────────────

/// Verifica a assinatura Ed25519 de um payload de licença.
///
/// Processo:
/// 1. Valida algoritmo declarado.
/// 2. Decodifica chave pública de base64 (32 bytes).
/// 3. Calcula SHA-256 do payload_json.
/// 4. Opcionalmente compara com payload_hash informado.
/// 5. Decodifica assinatura de base64 (64 bytes).
/// 6. Verifica Ed25519: chave pública + hash → assinatura.
pub fn verificar_assinatura_local(
    payload_json: &str,
    algoritmo: &str,
    key_id: &str,
    assinatura_b64: &str,
    payload_hash_informado: Option<&str>,
    chave_publica_b64: &str,
) -> ResultadoVerificacaoLocal {
    let mut warnings = Vec::new();

    // 1. Verificar algoritmo
    if algoritmo != ALGORITMO_SUPORTADO {
        return ResultadoVerificacaoLocal {
            valido: false,
            payload_hash_calculado: String::new(),
            mensagem: format!(
                "Algoritmo '{}' não suportado. Apenas '{}' é aceito.",
                algoritmo, ALGORITMO_SUPORTADO
            ),
            warnings,
        };
    }

    // 2. Decodificar chave pública
    let chave_bytes = match B64.decode(chave_publica_b64.trim()) {
        Ok(b) => b,
        Err(e) => {
            return ResultadoVerificacaoLocal {
                valido: false,
                payload_hash_calculado: String::new(),
                mensagem: format!("Chave pública base64 inválida: {}", e),
                warnings,
            };
        }
    };
    let chave_array: [u8; 32] = match chave_bytes.try_into() {
        Ok(a) => a,
        Err(_) => {
            return ResultadoVerificacaoLocal {
                valido: false,
                payload_hash_calculado: String::new(),
                mensagem: "Chave pública Ed25519 deve ter exatamente 32 bytes.".to_string(),
                warnings,
            };
        }
    };
    let verifying_key = match VerifyingKey::from_bytes(&chave_array) {
        Ok(k) => k,
        Err(e) => {
            return ResultadoVerificacaoLocal {
                valido: false,
                payload_hash_calculado: String::new(),
                mensagem: format!("Chave pública Ed25519 inválida: {}", e),
                warnings,
            };
        }
    };

    // 3. Calcular hash do payload recebido
    let hash_hex = calcular_hash_payload(payload_json);
    let hash_bytes_local = hex::decode(&hash_hex).unwrap_or_default();

    // 4. Verificar hash opcional informado
    if let Some(hash_info) = payload_hash_informado {
        if !hash_info.is_empty() && hash_info != hash_hex {
            return ResultadoVerificacaoLocal {
                valido: false,
                payload_hash_calculado: hash_hex,
                mensagem: "payload_hash informado difere do calculado. Payload pode ter sido adulterado.".to_string(),
                warnings,
            };
        }
    }

    // 5. Decodificar assinatura
    let sig_bytes = match B64.decode(assinatura_b64.trim()) {
        Ok(b) => b,
        Err(e) => {
            return ResultadoVerificacaoLocal {
                valido: false,
                payload_hash_calculado: hash_hex,
                mensagem: format!("Assinatura base64 inválida: {}", e),
                warnings,
            };
        }
    };
    let sig_array: [u8; 64] = match sig_bytes.try_into() {
        Ok(a) => a,
        Err(_) => {
            return ResultadoVerificacaoLocal {
                valido: false,
                payload_hash_calculado: hash_hex,
                mensagem: "Assinatura Ed25519 deve ter 64 bytes.".to_string(),
                warnings,
            };
        }
    };
    let signature = Signature::from_bytes(&sig_array);

    // 6. Verificar assinatura com chave pública
    match verifying_key.verify(hash_bytes_local.as_slice(), &signature) {
        Ok(_) => {
            // Aviso se key_id sugere DEV
            if key_id.contains("dev") || key_id.contains("efemero") {
                warnings.push(
                    "Assinatura verificada com chave DEV. Em produção use chave persistente."
                        .to_string(),
                );
            }
            ResultadoVerificacaoLocal {
                valido: true,
                payload_hash_calculado: hash_hex,
                mensagem: "Assinatura Ed25519 válida. Payload autêntico.".to_string(),
                warnings,
            }
        }
        Err(_) => ResultadoVerificacaoLocal {
            valido: false,
            payload_hash_calculado: hash_hex,
            mensagem: "Assinatura Ed25519 INVÁLIDA. Payload rejeitado.".to_string(),
            warnings,
        },
    }
}

// ─────────────────────────────────────────────────────────────────
// EXTRAÇÃO DE CAMPOS DO PAYLOAD CANÔNICO
// ─────────────────────────────────────────────────────────────────

/// Campos extraídos do payload canônico da licença.
/// Todos são obrigatórios — se ausentes, payload é rejeitado.
pub struct CamposPayload {
    pub empresa_id: String,
    pub licenca_id: String,
    pub plano_codigo: String,
    pub status: String,
    pub validade: String,
    pub terminal: String,
    pub tolerancia_offline_dias: i64,
    pub emitido_em: String,
}

/// Extrai os campos obrigatórios do payload JSON da licença.
/// Retorna erro se qualquer campo obrigatório estiver ausente.
///
/// SEGURANÇA: Não aplica payload com campos críticos ausentes.
pub fn extrair_campos_payload(payload_json: &str) -> Result<CamposPayload, String> {
    let v: serde_json::Value = serde_json::from_str(payload_json)
        .map_err(|e| format!("JSON inválido: {}", e))?;

    let get_str = |campo: &str| -> Result<String, String> {
        v.get(campo)
            .and_then(|f| f.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("Campo obrigatório ausente: '{}'", campo))
    };

    let empresa_id = get_str("empresa_id")?;
    let licenca_id = get_str("licenca_id")?;
    let plano_codigo = get_str("plano_codigo")?;
    let status = get_str("status")?;
    let validade = get_str("validade").unwrap_or_else(|_| "null".to_string());
    let terminal = get_str("terminal")?;
    let emitido_em = get_str("emitido_em")?;

    let tolerancia_offline_dias = v
        .get("tolerancia_offline_dias")
        .and_then(|f| f.as_i64())
        .ok_or("Campo 'tolerancia_offline_dias' ausente ou inválido")?;

    // Validar campos críticos não vazios
    if empresa_id.is_empty() {
        return Err("empresa_id não pode ser vazio".to_string());
    }
    if licenca_id.is_empty() {
        return Err("licenca_id não pode ser vazio".to_string());
    }
    if plano_codigo.is_empty() {
        return Err("plano_codigo não pode ser vazio".to_string());
    }
    if status.is_empty() {
        return Err("status não pode ser vazio".to_string());
    }

    Ok(CamposPayload {
        empresa_id,
        licenca_id,
        plano_codigo,
        status,
        validade,
        terminal,
        tolerancia_offline_dias,
        emitido_em,
    })
}
