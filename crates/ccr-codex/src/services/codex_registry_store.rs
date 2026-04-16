// 📦 Codex Auth Registry 统一存储层
//
// 消除 CodexAuthService / CodexOAuthTokenService / CodexQuotaService
// 三处重复的 registry load/save 逻辑，统一注册表访问模式：
// - load: 只读，不加锁
// - save: 文件锁 + 备份 + 原子写入
// - backup: 独立备份操作

use crate::models::CodexAuthRegistry;
use ccr_core::core::atomic_writer::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::lock::LockManager;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const REGISTRY_LOCK_RESOURCE: &str = "codex_auth_registry";
const REGISTRY_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

/// Codex Auth 注册表统一存储层
///
/// 提供对 `auth_registry.toml` 的标准化访问：
/// - 读取：无锁，直接解析文件
/// - 写入：文件锁 + 备份 + 原子写入（最安全路径）
pub struct CodexRegistryStore {
    /// auth_registry.toml 路径
    registry_path: PathBuf,
    /// 备份目录
    backup_dir: PathBuf,
    /// 锁目录
    lock_dir: PathBuf,
}

impl CodexRegistryStore {
    /// 从 CCR codex 平台目录构造
    pub fn new(ccr_codex_dir: &Path) -> Self {
        Self {
            registry_path: ccr_codex_dir.join("auth_registry.toml"),
            backup_dir: ccr_codex_dir.join("auth").join("backups"),
            lock_dir: ccr_codex_dir.join(".locks"),
        }
    }

    /// 加载注册表（只读，无锁）
    pub fn load(&self) -> Result<CodexAuthRegistry> {
        if !self.registry_path.exists() {
            return Ok(CodexAuthRegistry::default());
        }

        let content = fs::read_to_string(&self.registry_path)
            .map_err(|e| CcrError::ConfigError(format!("读取注册表失败: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析注册表失败: {}", e)))
    }

    /// 保存注册表（文件锁 + 备份 + 原子写入）
    pub fn save(&self, registry: &CodexAuthRegistry) -> Result<()> {
        let lock_manager = LockManager::new(&self.lock_dir);
        let _lock = lock_manager.lock_resource(REGISTRY_LOCK_RESOURCE, REGISTRY_LOCK_TIMEOUT)?;

        // 确保目录存在
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::ConfigError(format!("创建目录失败: {}", e)))?;
        }

        let content = toml::to_string_pretty(registry)
            .map_err(|e| CcrError::ConfigError(format!("序列化注册表失败: {}", e)))?;

        // 写入前备份
        let _ = self.backup();

        AtomicWriter::new(&self.registry_path)
            .write_string(&content)
            .map_err(|e| CcrError::ConfigError(format!("写入注册表失败: {}", e)))?;

        self.ensure_private_permissions(&self.registry_path);
        Ok(())
    }

    /// 备份当前注册表文件
    pub fn backup(&self) -> Result<Option<PathBuf>> {
        if !self.registry_path.exists() {
            return Ok(None);
        }

        fs::create_dir_all(&self.backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("创建备份目录失败: {}", e)))?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("auth_registry_{}.toml", timestamp);
        let backup_path = self.backup_dir.join(&backup_name);

        fs::copy(&self.registry_path, &backup_path)
            .map_err(|e| CcrError::ConfigError(format!("备份注册表失败: {}", e)))?;

        Ok(Some(backup_path))
    }

    fn ensure_private_permissions(&self, path: &Path) {
        crate::utils::ensure_private_permissions(path);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::{CodexAuthAccount, OpenAiAuthMethod};
    use chrono::Utc;

    #[test]
    fn test_load_missing_returns_default() {
        let temp = tempfile::tempdir().unwrap();
        let store = CodexRegistryStore::new(temp.path());
        let registry = store.load().unwrap();
        assert_eq!(registry.version, "1.0");
        assert!(registry.accounts.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let store = CodexRegistryStore::new(temp.path());

        let mut registry = CodexAuthRegistry {
            current_auth: Some("test".to_string()),
            ..Default::default()
        };
        registry.accounts.insert(
            "test".to_string(),
            CodexAuthAccount {
                description: Some("Test account".to_string()),
                account_id: "acc-1".to_string(),
                auth_method: Some(OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
            },
        );

        store.save(&registry).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.current_auth, Some("test".to_string()));
        assert!(loaded.accounts.contains_key("test"));
    }

    #[test]
    fn test_backup_creates_file() {
        let temp = tempfile::tempdir().unwrap();
        let store = CodexRegistryStore::new(temp.path());

        // 先保存内容
        store.save(&CodexAuthRegistry::default()).unwrap();

        // 备份
        let backup_path = store.backup().unwrap();
        assert!(backup_path.is_some());
        assert!(backup_path.unwrap().exists());
    }

    #[test]
    fn test_backup_missing_file_returns_none() {
        let temp = tempfile::tempdir().unwrap();
        let store = CodexRegistryStore::new(temp.path());
        let result = store.backup().unwrap();
        assert!(result.is_none());
    }
}
