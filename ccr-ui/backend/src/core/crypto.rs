// API Key 加密模块 - AES-256-GCM 对称加密
// 用于安全存储中转站签到功能的 API Key

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
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
    fn save_key(path: &PathBuf, key: &Key<Aes256Gcm>) -> Result<(), CryptoError> {
        let key_base64 = BASE64.encode(key.as_slice());

        // 使用临时文件 + 原子重命名，确保安全
        let temp_path = path.with_extension("key.tmp");

        fs::write(&temp_path, &key_base64)
            .map_err(|e| CryptoError::KeyWriteError(format!("{}: {}", temp_path.display(), e)))?;

        fs::rename(&temp_path, path)
            .map_err(|e| CryptoError::KeyWriteError(format!("Atomic rename failed: {}", e)))?;

        // 尝试设置文件权限（仅限 Unix）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600); // 仅用户可读写
                let _ = fs::set_permissions(path, perms);
            }
        }

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
    /// # Arguments
    /// * `encrypted` - base64 编码的加密数据 (nonce || ciphertext)
    pub fn decrypt(&self, encrypted: &str) -> Result<String, CryptoError> {
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

/// 掩码显示 API Key
///
/// 例如: "sk-1234567890abcdef" -> "sk-****cdef"
#[allow(dead_code)]
pub fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 8 {
        return "*".repeat(api_key.len());
    }

    // 找到前缀分隔符
    let prefix_end = api_key.find('-').map(|i| i + 1).unwrap_or(0);
    let prefix = &api_key[..prefix_end];

    // 保留最后 4 个字符
    let suffix = &api_key[api_key.len() - 4..];

    format!("{}****{}", prefix, suffix)
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

        assert_eq!(original, decrypted);
        assert_ne!(original, encrypted); // 确保已加密
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

        // 每次加密应该产生不同的密文（因为 nonce 不同）
        assert_ne!(encrypted1, encrypted2);

        // 但都能正确解密
        assert_eq!(
            original,
            crypto
                .decrypt(&encrypted1)
                .expect("Failed to decrypt test data 1")
        );
        assert_eq!(
            original,
            crypto
                .decrypt(&encrypted2)
                .expect("Failed to decrypt test data 2")
        );
    }

    #[test]
    fn test_key_persistence() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir for test");
        let path = temp_dir.path().to_path_buf();

        let original = "sk-test-api-key";

        // 第一次创建并加密
        let encrypted = {
            let crypto =
                CryptoManager::new(&path).expect("Failed to create CryptoManager for test 1");
            crypto
                .encrypt(original)
                .expect("Failed to encrypt test data")
        };

        // 重新加载并解密
        let decrypted = {
            let crypto =
                CryptoManager::new(&path).expect("Failed to create CryptoManager for test 2");
            crypto
                .decrypt(&encrypted)
                .expect("Failed to decrypt test data")
        };

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("sk-1234567890abcdef"), "sk-****cdef");
        assert_eq!(mask_api_key("sk-abc"), "******"); // 6 chars <= 8, fully masked
        assert_eq!(mask_api_key("short"), "*****");
        assert_eq!(mask_api_key("12345678"), "********"); // exactly 8, fully masked
        assert_eq!(mask_api_key("123456789"), "****6789"); // 9 chars > 8, partial mask
    }

    #[test]
    fn test_invalid_decrypt() {
        let temp_dir = TempDir::new().unwrap();
        let crypto = CryptoManager::new(&temp_dir.path().to_path_buf()).unwrap();

        // 无效的 base64
        assert!(crypto.decrypt("invalid!!!").is_err());

        // 太短的数据
        assert!(crypto.decrypt("dG9vIHNob3J0").is_err());
    }
}
