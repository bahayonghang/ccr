#![allow(deprecated)]

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng, Payload, rand_core::RngCore},
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ccr_core::core::error::{CcrError, Result};
use serde::{Deserialize, Serialize};

const ENVELOPE_MAGIC: &str = "ccr-sync-encrypted-asset";
const ENVELOPE_VERSION: u8 = 2;
const KEY_BYTES: usize = 32;
const SALT_BYTES: usize = 32;
const NONCE_BYTES: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvelopeKdfParams {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl Default for EnvelopeKdfParams {
    fn default() -> Self {
        Self {
            m_cost: 65_536,
            t_cost: 3,
            p_cost: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvelopeMetadata {
    pub asset_id: String,
    pub relative_path: String,
    pub schema: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptedEnvelopeV2 {
    pub magic: String,
    pub version: u8,
    pub algorithm: String,
    pub kdf: String,
    pub kdf_params: EnvelopeKdfParams,
    pub salt: String,
    pub nonce: String,
    pub metadata: EnvelopeMetadata,
    pub ciphertext: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeRead {
    EncryptedV2(Vec<u8>),
    PlaintextV1(Vec<u8>),
}

pub fn encrypt_asset_bytes(
    plaintext: &[u8],
    passphrase: &str,
    asset_id: &str,
    relative_path: &str,
) -> Result<Vec<u8>> {
    validate_passphrase(passphrase)?;
    validate_metadata(asset_id, relative_path)?;

    let kdf_params = EnvelopeKdfParams::default();
    let mut salt = [0_u8; SALT_BYTES];
    let mut nonce_bytes = [0_u8; NONCE_BYTES];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce_bytes);

    let mut key = derive_key(passphrase, &salt, &kdf_params)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| envelope_error("cipher", "无法初始化 AES-256-GCM"))?;
    let metadata = EnvelopeMetadata {
        asset_id: asset_id.to_string(),
        relative_path: relative_path.to_string(),
        schema: "ccr-sync-asset-v2".to_string(),
    };
    let aad = build_aad(&metadata);
    let encrypt_result = cipher.encrypt(
        Nonce::from_slice(&nonce_bytes),
        Payload {
            msg: plaintext,
            aad: &aad,
        },
    );
    key.fill(0);
    let ciphertext =
        encrypt_result.map_err(|_| envelope_error("encrypt", "敏感同步资产加密失败"))?;

    serde_json::to_vec(&EncryptedEnvelopeV2 {
        magic: ENVELOPE_MAGIC.to_string(),
        version: ENVELOPE_VERSION,
        algorithm: "aes-256-gcm".to_string(),
        kdf: "argon2id".to_string(),
        kdf_params,
        salt: BASE64.encode(salt),
        nonce: BASE64.encode(nonce_bytes),
        metadata,
        ciphertext: BASE64.encode(ciphertext),
    })
    .map_err(|error| envelope_error("serialize", &format!("加密信封序列化失败: {error}")))
}

pub fn decrypt_asset_bytes(
    encoded: &[u8],
    passphrase: &str,
    expected_asset_id: &str,
    expected_relative_path: &str,
    allow_plaintext_v1: bool,
) -> Result<EnvelopeRead> {
    validate_passphrase(passphrase)?;
    validate_metadata(expected_asset_id, expected_relative_path)?;

    let value = match serde_json::from_slice::<serde_json::Value>(encoded) {
        Ok(value) => value,
        Err(_) if allow_plaintext_v1 => {
            return Ok(EnvelopeRead::PlaintextV1(encoded.to_vec()));
        }
        Err(_) => return Err(plaintext_migration_required()),
    };

    if value.get("magic").and_then(serde_json::Value::as_str) != Some(ENVELOPE_MAGIC) {
        if allow_plaintext_v1 {
            return Ok(EnvelopeRead::PlaintextV1(encoded.to_vec()));
        }
        return Err(plaintext_migration_required());
    }

    let envelope: EncryptedEnvelopeV2 = serde_json::from_value(value)
        .map_err(|error| envelope_error("invalid", &format!("v2 加密信封结构无效: {error}")))?;
    validate_envelope_header(&envelope)?;
    if envelope.metadata.asset_id != expected_asset_id
        || envelope.metadata.relative_path != expected_relative_path
    {
        return Err(envelope_error("metadata", "加密信封资产元数据与请求不匹配"));
    }

    let salt = decode_exact(&envelope.salt, SALT_BYTES, "salt")?;
    let nonce = decode_exact(&envelope.nonce, NONCE_BYTES, "nonce")?;
    let ciphertext = BASE64
        .decode(&envelope.ciphertext)
        .map_err(|_| envelope_error("ciphertext", "加密信封密文不是有效 Base64"))?;
    let mut key = derive_key(passphrase, &salt, &envelope.kdf_params)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| envelope_error("cipher", "无法初始化 AES-256-GCM"))?;
    let aad = build_aad(&envelope.metadata);
    let decrypt_result = cipher.decrypt(
        Nonce::from_slice(&nonce),
        Payload {
            msg: &ciphertext,
            aad: &aad,
        },
    );
    key.fill(0);
    let plaintext =
        decrypt_result.map_err(|_| envelope_error("authentication", "口令错误或加密资产已损坏"))?;
    Ok(EnvelopeRead::EncryptedV2(plaintext))
}

fn validate_envelope_header(envelope: &EncryptedEnvelopeV2) -> Result<()> {
    if envelope.version != ENVELOPE_VERSION
        || envelope.algorithm != "aes-256-gcm"
        || envelope.kdf != "argon2id"
        || envelope.kdf_params != EnvelopeKdfParams::default()
        || envelope.metadata.schema != "ccr-sync-asset-v2"
    {
        return Err(envelope_error(
            "unsupported",
            "不支持的敏感资产加密信封版本或算法",
        ));
    }
    Ok(())
}

fn validate_passphrase(passphrase: &str) -> Result<()> {
    if passphrase.is_empty() {
        return Err(envelope_error(
            "passphrase_required",
            "敏感同步资产需要独立口令",
        ));
    }
    Ok(())
}

fn validate_metadata(asset_id: &str, relative_path: &str) -> Result<()> {
    if asset_id.trim().is_empty() || relative_path.contains('\0') {
        return Err(envelope_error("metadata", "敏感同步资产元数据无效"));
    }
    Ok(())
}

fn derive_key(
    passphrase: &str,
    salt: &[u8],
    params: &EnvelopeKdfParams,
) -> Result<[u8; KEY_BYTES]> {
    let params = Params::new(params.m_cost, params.t_cost, params.p_cost, Some(KEY_BYTES))
        .map_err(|error| envelope_error("kdf_params", &format!("Argon2id 参数无效: {error}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0_u8; KEY_BYTES];
    if let Err(error) = argon2.hash_password_into(passphrase.as_bytes(), salt, &mut key) {
        key.fill(0);
        return Err(envelope_error(
            "kdf",
            &format!("Argon2id 密钥派生失败: {error}"),
        ));
    }
    Ok(key)
}

fn build_aad(metadata: &EnvelopeMetadata) -> Vec<u8> {
    format!(
        "{ENVELOPE_MAGIC}|{ENVELOPE_VERSION}|aes-256-gcm|argon2id|{}|{}|{}",
        metadata.asset_id, metadata.relative_path, metadata.schema
    )
    .into_bytes()
}

fn decode_exact(encoded: &str, expected: usize, field: &str) -> Result<Vec<u8>> {
    let decoded = BASE64
        .decode(encoded)
        .map_err(|_| envelope_error(field, "加密信封字段不是有效 Base64"))?;
    if decoded.len() != expected {
        return Err(envelope_error(field, "加密信封字段长度无效"));
    }
    Ok(decoded)
}

fn plaintext_migration_required() -> CcrError {
    envelope_error(
        "plaintext_v1_requires_migration",
        "远端是明文 v1；必须显式选择明文迁移后才能读取",
    )
}

fn envelope_error(code: &str, message: &str) -> CcrError {
    CcrError::SyncError(format!("sync_envelope_{code}: {message}"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn v2_round_trip_binds_asset_metadata() {
        let encoded =
            encrypt_asset_bytes(b"api_key=secret", "correct horse", "codex-config", ".").unwrap();
        assert!(!String::from_utf8_lossy(&encoded).contains("api_key=secret"));

        let read =
            decrypt_asset_bytes(&encoded, "correct horse", "codex-config", ".", false).unwrap();
        assert_eq!(read, EnvelopeRead::EncryptedV2(b"api_key=secret".to_vec()));
        assert!(decrypt_asset_bytes(&encoded, "wrong", "codex-config", ".", false).is_err());
        assert!(decrypt_asset_bytes(&encoded, "correct horse", "other", ".", false).is_err());
    }

    #[test]
    fn random_salt_and_nonce_change_ciphertext() {
        let first = encrypt_asset_bytes(b"same", "pass", "asset", ".").unwrap();
        let second = encrypt_asset_bytes(b"same", "pass", "asset", ".").unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn plaintext_v1_requires_explicit_migration() {
        let plaintext = br#"{"token":"secret"}"#;
        assert!(
            decrypt_asset_bytes(plaintext, "pass", "asset", ".", false)
                .unwrap_err()
                .to_string()
                .contains("plaintext_v1_requires_migration")
        );
        assert_eq!(
            decrypt_asset_bytes(plaintext, "pass", "asset", ".", true).unwrap(),
            EnvelopeRead::PlaintextV1(plaintext.to_vec())
        );
    }

    #[test]
    fn tampering_with_authenticated_metadata_fails() {
        let encoded = encrypt_asset_bytes(b"secret", "pass", "asset", ".").unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
        value["metadata"]["relative_path"] = serde_json::Value::String("other".to_string());
        let tampered = serde_json::to_vec(&value).unwrap();
        assert!(decrypt_asset_bytes(&tampered, "pass", "asset", "other", false).is_err());
    }

    #[test]
    fn rejects_unbounded_remote_kdf_parameters_before_derivation() {
        let encoded = encrypt_asset_bytes(b"secret", "pass", "asset", ".").unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
        value["kdf_params"]["m_cost"] = serde_json::Value::from(u32::MAX);
        let hostile = serde_json::to_vec(&value).unwrap();

        let error = decrypt_asset_bytes(&hostile, "pass", "asset", ".", false).unwrap_err();
        assert!(error.to_string().contains("sync_envelope_unsupported"));
    }
}
