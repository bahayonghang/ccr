// Codex Auth 导出加密模块 - 基于密码的 AES-256-GCM 加密
//
// 用于跨设备安全传输 Codex 账号数据：
// - Argon2id 从用户密码派生 256-bit 密钥
// - AES-256-GCM 加密 accounts payload
// - 信封头字段绑定为 AAD，防止元数据篡改
#![allow(deprecated)]

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
};
use argon2::Argon2;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ccr_core::core::error::{CcrError, Result};
use chrono::{DateTime, Utc};

use crate::models::codex_auth::{CodexAuthEncryptedExport, EncryptionHeader, KdfParams};

/// 盐长度 (256 bits = 32 bytes)
const SALT_SIZE: usize = 32;
/// Nonce 长度 (96 bits = 12 bytes for GCM)
const NONCE_SIZE: usize = 12;
/// 密钥长度 (256 bits = 32 bytes)
const KEY_SIZE: usize = 32;

/// 导出加密工具
///
/// 提供基于密码的加密/解密功能，用于 codex auth 导出文件的跨设备安全传输。
pub struct ExportCrypto;

impl ExportCrypto {
    /// 使用 Argon2id 从密码派生 AES-256 密钥
    fn derive_key(password: &str, salt: &[u8], params: &KdfParams) -> Result<Key<Aes256Gcm>> {
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(params.m_cost, params.t_cost, params.p_cost, Some(KEY_SIZE))
                .map_err(|e| CcrError::ConfigError(format!("Argon2 参数无效: {}", e)))?,
        );

        let mut key_bytes = [0u8; KEY_SIZE];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key_bytes)
            .map_err(|e| CcrError::ConfigError(format!("密钥派生失败: {}", e)))?;

        Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes))
    }

    /// 构建 AAD (Additional Authenticated Data)
    ///
    /// 将信封头字段序列化为确定性字符串，绑定到 GCM 认证标签，
    /// 防止攻击者篡改可读的元数据部分。
    fn build_aad(
        version: &str,
        format: &str,
        exported_at: &DateTime<Utc>,
        account_count: usize,
    ) -> Vec<u8> {
        format!(
            "{}|{}|{}|{}",
            version,
            format,
            exported_at.to_rfc3339(),
            account_count
        )
        .into_bytes()
    }

    /// 加密导出数据
    ///
    /// 将 accounts JSON 字符串加密为信封格式。
    ///
    /// # Arguments
    /// * `accounts_json` - 序列化后的账号数据 JSON
    /// * `password` - 用户设置的导出密码
    /// * `exported_at` - 导出时间戳
    /// * `account_count` - 账号数量
    pub fn encrypt_export(
        accounts_json: &str,
        password: &str,
        exported_at: DateTime<Utc>,
        account_count: usize,
    ) -> Result<CodexAuthEncryptedExport> {
        let kdf_params = KdfParams::default();

        // 生成随机盐和 nonce
        let mut salt = [0u8; SALT_SIZE];
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce_bytes);

        // 派生密钥
        let key = Self::derive_key(password, &salt, &kdf_params)?;

        // 构建 AAD
        let version = "2.0".to_string();
        let format_str = "encrypted".to_string();
        let aad = Self::build_aad(&version, &format_str, &exported_at, account_count);

        // AES-256-GCM 加密
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(
                nonce,
                aes_gcm::aead::Payload {
                    msg: accounts_json.as_bytes(),
                    aad: &aad,
                },
            )
            .map_err(|e| CcrError::ConfigError(format!("加密失败: {}", e)))?;

        Ok(CodexAuthEncryptedExport {
            version,
            format: format_str,
            exported_at,
            account_count,
            encryption: EncryptionHeader {
                algorithm: "aes-256-gcm".to_string(),
                kdf: "argon2id".to_string(),
                kdf_params,
                salt: BASE64.encode(salt),
                nonce: BASE64.encode(nonce_bytes),
            },
            encrypted_payload: BASE64.encode(&ciphertext),
        })
    }

    /// 解密导出数据
    ///
    /// 从加密信封中恢复 accounts JSON 字符串。
    ///
    /// # Arguments
    /// * `encrypted` - 加密的导出信封
    /// * `password` - 用户输入的密码
    pub fn decrypt_export(encrypted: &CodexAuthEncryptedExport, password: &str) -> Result<String> {
        // 解码盐和 nonce
        let salt = BASE64
            .decode(&encrypted.encryption.salt)
            .map_err(|e| CcrError::ConfigError(format!("盐值解码失败: {}", e)))?;

        let nonce_bytes = BASE64
            .decode(&encrypted.encryption.nonce)
            .map_err(|e| CcrError::ConfigError(format!("Nonce 解码失败: {}", e)))?;

        if nonce_bytes.len() != NONCE_SIZE {
            return Err(CcrError::ConfigError(format!(
                "Nonce 长度无效: 期望 {} 字节, 实际 {} 字节",
                NONCE_SIZE,
                nonce_bytes.len()
            )));
        }

        // 解码密文
        let ciphertext = BASE64
            .decode(&encrypted.encrypted_payload)
            .map_err(|e| CcrError::ConfigError(format!("密文解码失败: {}", e)))?;

        // 派生密钥
        let key = Self::derive_key(password, &salt, &encrypted.encryption.kdf_params)?;

        // 重建 AAD
        let aad = Self::build_aad(
            &encrypted.version,
            &encrypted.format,
            &encrypted.exported_at,
            encrypted.account_count,
        );

        // AES-256-GCM 解密
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(
                nonce,
                aes_gcm::aead::Payload {
                    msg: &ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| CcrError::ConfigError("密码错误或文件已损坏".to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|e| CcrError::ConfigError(format!("解密后数据非有效 UTF-8: {}", e)))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let accounts_json =
            r#"{"main":{"account_id":"acc-123","saved_at":"2026-01-01T00:00:00Z"}}"#;
        let password = "test-password-123";
        let exported_at = Utc::now();

        let encrypted = ExportCrypto::encrypt_export(accounts_json, password, exported_at, 1)
            .expect("加密失败");

        assert_eq!(encrypted.version, "2.0");
        assert_eq!(encrypted.format, "encrypted");
        assert_eq!(encrypted.account_count, 1);
        assert_ne!(encrypted.encrypted_payload, accounts_json);

        let decrypted = ExportCrypto::decrypt_export(&encrypted, password).expect("解密失败");

        assert_eq!(decrypted, accounts_json);
    }

    #[test]
    fn test_wrong_password_fails() {
        let accounts_json = r#"{"test":"data"}"#;
        let exported_at = Utc::now();

        let encrypted =
            ExportCrypto::encrypt_export(accounts_json, "correct-password", exported_at, 1)
                .expect("加密失败");

        let result = ExportCrypto::decrypt_export(&encrypted, "wrong-password");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("密码错误或文件已损坏")
        );
    }

    #[test]
    fn test_different_exports_produce_different_ciphertext() {
        let accounts_json = r#"{"test":"data"}"#;
        let password = "same-password";
        let exported_at = Utc::now();

        let encrypted1 = ExportCrypto::encrypt_export(accounts_json, password, exported_at, 1)
            .expect("加密失败 1");
        let encrypted2 = ExportCrypto::encrypt_export(accounts_json, password, exported_at, 1)
            .expect("加密失败 2");

        // 随机盐和 nonce 保证每次密文不同
        assert_ne!(encrypted1.encrypted_payload, encrypted2.encrypted_payload);
        assert_ne!(encrypted1.encryption.salt, encrypted2.encryption.salt);
        assert_ne!(encrypted1.encryption.nonce, encrypted2.encryption.nonce);

        // 但两者都能正确解密
        assert_eq!(
            ExportCrypto::decrypt_export(&encrypted1, password).unwrap(),
            accounts_json
        );
        assert_eq!(
            ExportCrypto::decrypt_export(&encrypted2, password).unwrap(),
            accounts_json
        );
    }

    #[test]
    fn test_tampered_metadata_fails_decryption() {
        let accounts_json = r#"{"test":"data"}"#;
        let password = "test-password";
        let exported_at = Utc::now();

        let mut encrypted = ExportCrypto::encrypt_export(accounts_json, password, exported_at, 1)
            .expect("加密失败");

        // 篡改 account_count（AAD 不匹配，GCM 认证应失败）
        encrypted.account_count = 999;

        let result = ExportCrypto::decrypt_export(&encrypted, password);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_password_works() {
        // 空密码应该技术上可行（但 CLI 层会限制最小长度）
        let accounts_json = r#"{"test":"empty-password"}"#;
        let exported_at = Utc::now();

        let encrypted =
            ExportCrypto::encrypt_export(accounts_json, "", exported_at, 1).expect("加密失败");

        let decrypted = ExportCrypto::decrypt_export(&encrypted, "").expect("解密失败");

        assert_eq!(decrypted, accounts_json);
    }

    #[test]
    fn test_unicode_password_and_content() {
        let accounts_json = r#"{"描述":"中文测试账号","email":"test@例え.jp"}"#;
        let password = "密码🔐パスワード";
        let exported_at = Utc::now();

        let encrypted = ExportCrypto::encrypt_export(accounts_json, password, exported_at, 1)
            .expect("加密失败");

        let decrypted = ExportCrypto::decrypt_export(&encrypted, password).expect("解密失败");

        assert_eq!(decrypted, accounts_json);
    }

    #[test]
    fn test_large_payload() {
        // 模拟大量账号数据
        let mut accounts = String::from("{");
        for i in 0..100 {
            if i > 0 {
                accounts.push(',');
            }
            accounts.push_str(&format!(
                r#""account-{}": {{"account_id": "acc-{}", "saved_at": "2026-01-01T00:00:00Z"}}"#,
                i, i
            ));
        }
        accounts.push('}');

        let password = "large-payload-test";
        let exported_at = Utc::now();

        let encrypted =
            ExportCrypto::encrypt_export(&accounts, password, exported_at, 100).expect("加密失败");

        assert_eq!(encrypted.account_count, 100);

        let decrypted = ExportCrypto::decrypt_export(&encrypted, password).expect("解密失败");

        assert_eq!(decrypted, accounts);
    }
}
