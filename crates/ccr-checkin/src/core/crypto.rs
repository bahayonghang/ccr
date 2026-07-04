// API Key 加密模块 - AES-256-GCM 对称加密
// 用于安全存储中转站签到功能的 API Key
//
// 注意：`aes_gcm` 通过 `generic-array` 0.x 暴露的 `from_slice` / `as_slice` 已标记废弃，
// 需待 `aes_gcm` 迁移到 `generic-array` 1.x 后方可消除。
#![allow(deprecated)]

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ccr_core::{Secret, WriteOptions, write_guarded};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// 加密密钥长度 (256 bits = 32 bytes)
const KEY_SIZE: usize = 32;
/// Nonce 长度 (96 bits = 12 bytes for GCM)
const NONCE_SIZE: usize = 12;
/// 密钥文件名
const CRYPTO_KEY_FILE: &str = "crypto.key";

/// 加密相关错误
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Failed to read crypto key: {0}")]
    KeyReadError(String),
    #[error("Failed to write crypto key: {0}")]
    KeyWriteError(String),
    #[error("Failed to create directory: {0}")]
    DirectoryError(String),
    #[error("Encryption failed: {0}")]
    EncryptionError(String),
    #[error("Decryption failed: {0}")]
    DecryptionError(String),
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    #[error("Invalid encrypted data format: {0}")]
    InvalidDataFormat(String),
}

/// 加密管理器
/// 负责 API Key 的加密和解密
pub struct CryptoManager {
    /// AES-256-GCM 密钥
    key: Key<Aes256Gcm>,
    /// 密钥文件路径
    key_path: PathBuf,
}

impl CryptoManager {
    /// 创建或加载加密管理器
    ///
    /// 如果密钥文件存在，则加载；否则生成新密钥
    ///
    /// # Arguments
    /// * `checkin_dir` - 签到数据目录路径 (如 ~/.ccr/checkin/)
    pub fn new(checkin_dir: &PathBuf) -> Result<Self, CryptoError> {
        // 确保目录存在
        if !checkin_dir.exists() {
            fs::create_dir_all(checkin_dir).map_err(|e| {
                CryptoError::DirectoryError(format!("{}: {}", checkin_dir.display(), e))
            })?;
        }

        let key_path = checkin_dir.join(CRYPTO_KEY_FILE);

        let key = if key_path.exists() {
            // 加载现有密钥
            Self::load_key(&key_path)?
        } else {
            // 生成新密钥
            let new_key = Self::generate_key();
            Self::save_key(&key_path, &new_key)?;
            new_key
        };

        Ok(Self { key, key_path })
    }

    /// 生成随机 256-bit 密钥
    fn generate_key() -> Key<Aes256Gcm> {
        let mut key_bytes = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut key_bytes);
        *Key::<Aes256Gcm>::from_slice(&key_bytes)
    }

    /// 从文件加载密钥
    fn load_key(path: &PathBuf) -> Result<Key<Aes256Gcm>, CryptoError> {
        let key_base64 = fs::read_to_string(path)
            .map_err(|e| CryptoError::KeyReadError(format!("{}: {}", path.display(), e)))?;

        let key_bytes = BASE64
            .decode(key_base64.trim())
            .map_err(|e| CryptoError::InvalidKeyFormat(format!("Base64 decode failed: {}", e)))?;

        if key_bytes.len() != KEY_SIZE {
            return Err(CryptoError::InvalidKeyFormat(format!(
                "Expected {} bytes, got {}",
                KEY_SIZE,
                key_bytes.len()
            )));
        }

        Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes))
    }

    /// 保存密钥到文件
    fn save_key(path: &Path, key: &Key<Aes256Gcm>) -> Result<(), CryptoError> {
        let key_base64 = BASE64.encode(key.as_slice());

        // 统一走 guarded write：temp 文件在写入内容之前即设为 0o600（消除明文 key
        // 短暂 world-readable 窗口）+ fsync + Windows 重试 rename + 跨进程文件锁。
        let opts = WriteOptions {
            secret: true,
            ..Default::default()
        };
        write_guarded(path, key_base64.as_bytes(), &opts)
            .map_err(|e| CryptoError::KeyWriteError(format!("{}: {}", path.display(), e)))?;

        Ok(())
    }

    /// 加密明文
    ///
    /// 返回格式: base64(nonce || ciphertext)
    ///
    /// # Arguments
    /// * `plaintext` - 要加密的明文（如 API Key）
    pub fn encrypt(&self, plaintext: &str) -> Result<String, CryptoError> {
        let cipher = Aes256Gcm::new(&self.key);

        // 生成随机 nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| CryptoError::EncryptionError(format!("{}", e)))?;

        // 组合 nonce + ciphertext 并 base64 编码
        let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(&combined))
    }

    /// 解密密文
    ///
    /// 返回 [`Secret`]：解密产物即包裹，Debug/日志路径不泄露；
    /// 原文仅经 `expose()` 流向 HTTP 头构造、明文导出等合法消费点
    ///
    /// # Arguments
    /// * `encrypted` - base64 编码的加密数据 (nonce || ciphertext)
    pub fn decrypt(&self, encrypted: &str) -> Result<Secret, CryptoError> {
        // Base64 解码
        let combined = BASE64
            .decode(encrypted.trim())
            .map_err(|e| CryptoError::InvalidDataFormat(format!("Base64 decode failed: {}", e)))?;

        // 检查最小长度 (nonce + 至少 1 byte ciphertext + 16 bytes tag)
        if combined.len() < NONCE_SIZE + 17 {
            return Err(CryptoError::InvalidDataFormat(
                "Encrypted data too short".to_string(),
            ));
        }

        // 分离 nonce 和 ciphertext
        let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        // 解密
        let cipher = Aes256Gcm::new(&self.key);
        let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_| {
            CryptoError::DecryptionError(
                "Decryption failed - invalid key or corrupted data".to_string(),
            )
        })?;

        String::from_utf8(plaintext)
            .map(Secret::new)
            .map_err(|e| CryptoError::DecryptionError(format!("Invalid UTF-8: {}", e)))
    }

    /// 获取密钥文件路径
    #[allow(dead_code)]
    pub fn key_path(&self) -> &PathBuf {
        &self.key_path
    }

    /// 检查密钥是否存在
    #[allow(dead_code)]
    pub fn key_exists(checkin_dir: &Path) -> bool {
        checkin_dir.join(CRYPTO_KEY_FILE).exists()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_encrypt_decrypt() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir for test");
        let crypto = CryptoManager::new(&temp_dir.path().to_path_buf())
            .expect("Failed to create CryptoManager for test");

        let original = "sk-1234567890abcdef";
        let encrypted = crypto
            .encrypt(original)
            .expect("Failed to encrypt test data");
        let decrypted = crypto
            .decrypt(&encrypted)
            .expect("Failed to decrypt test data");

        assert_eq!(decrypted, original);
        assert_ne!(encrypted, original);
    }

    #[test]
    fn test_different_nonces() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir for test");
        let crypto = CryptoManager::new(&temp_dir.path().to_path_buf())
            .expect("Failed to create CryptoManager for test");

        let original = "sk-1234567890abcdef";
        let encrypted1 = crypto
            .encrypt(original)
            .expect("Failed to encrypt test data 1");
        let encrypted2 = crypto
            .encrypt(original)
            .expect("Failed to encrypt test data 2");

        assert_ne!(encrypted1, encrypted2);

        assert_eq!(
            crypto
                .decrypt(&encrypted1)
                .expect("Failed to decrypt test data 1"),
            original
        );
        assert_eq!(
            crypto
                .decrypt(&encrypted2)
                .expect("Failed to decrypt test data 2"),
            original
        );
    }

    #[test]
    fn test_key_persistence() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir for test");
        let path = temp_dir.path().to_path_buf();

        let original = "sk-test-api-key";

        let encrypted = {
            let crypto =
                CryptoManager::new(&path).expect("Failed to create CryptoManager for test 1");
            crypto
                .encrypt(original)
                .expect("Failed to encrypt test data")
        };

        let decrypted = {
            let crypto =
                CryptoManager::new(&path).expect("Failed to create CryptoManager for test 2");
            crypto
                .decrypt(&encrypted)
                .expect("Failed to decrypt test data")
        };

        assert_eq!(decrypted, original);
    }

    // 🔐 密钥文件权限断言仅在 Unix 有意义；Windows 无 Unix 权限模型（NTFS ACL），
    // secret 选项在 Windows 上为 no-op，故该测试跳过。
    #[cfg(unix)]
    #[test]
    fn test_save_key_sets_owner_only_mode() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().expect("Failed to create temp dir for test");
        let path = temp_dir.path().to_path_buf();

        // 首次 new 会生成并保存密钥（走 save_key → write_guarded secret）
        CryptoManager::new(&path).expect("Failed to create CryptoManager for test");

        let key_path = path.join(CRYPTO_KEY_FILE);
        let mode = fs::metadata(&key_path)
            .expect("key file should exist")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn test_invalid_decrypt() {
        let temp_dir = TempDir::new().unwrap();
        let crypto = CryptoManager::new(&temp_dir.path().to_path_buf()).unwrap();

        assert!(crypto.decrypt("invalid!!!").is_err());
        assert!(crypto.decrypt("dG9vIHNob3J0").is_err());
    }
}
