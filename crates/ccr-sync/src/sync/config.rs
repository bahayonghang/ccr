// ☁️ CCR Sync 配置管理模块
// 📁 负责独立管理 WebDAV 同步配置
//
// 核心职责:
// - 🔍 读取 ~/.ccr/sync.toml 配置文件
// - 💾 保存同步配置
// - ✅ 验证同步配置完整性

use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::fileio;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// ☁️ WebDAV 同步配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    #[serde(default)]
    pub enabled: bool,
    pub webdav_url: String,
    pub username: String,
    pub password: String,
    #[serde(default = "default_remote_path")]
    pub remote_path: String,
    #[serde(default)]
    pub auto_sync: bool,
}

fn default_remote_path() -> String {
    "/ccr/".to_string()
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            webdav_url: "https://dav.jianguoyun.com/dav/".to_string(),
            username: String::new(),
            password: String::new(),
            remote_path: default_remote_path(),
            auto_sync: false,
        }
    }
}

pub struct SyncConfigManager {
    config_path: PathBuf,
}

#[allow(dead_code)]
impl SyncConfigManager {
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    pub fn with_default() -> Result<Self> {
        if let Ok(custom_path) = std::env::var("CCR_SYNC_CONFIG_PATH") {
            tracing::debug!("📁 使用环境变量指定的sync配置路径: {}", custom_path);
            return Ok(Self::new(custom_path));
        }

        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        let sync_config_path = home.join(".ccr").join("sync.toml");
        tracing::debug!("📁 使用sync配置路径: {:?}", sync_config_path);
        Ok(Self::new(sync_config_path))
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn load(&self) -> Result<SyncConfig> {
        if !self.config_path.exists() {
            tracing::debug!("⚙️ sync配置文件不存在，返回默认配置");
            return Ok(SyncConfig::default());
        }

        let config: SyncConfig = fileio::read_toml(&self.config_path)?;
        tracing::debug!(
            "✅ 成功加载sync配置文件: {:?}, 状态: {}",
            self.config_path,
            if config.enabled {
                "已启用"
            } else {
                "未启用"
            }
        );
        Ok(config)
    }

    pub fn save(&self, config: &SyncConfig) -> Result<()> {
        fileio::write_toml(&self.config_path, config)?;
        tracing::debug!("✅ Sync配置文件已保存: {:?}", self.config_path);
        Ok(())
    }

    pub fn delete(&self) -> Result<()> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path)
                .map_err(|e| CcrError::ConfigError(format!("删除sync配置文件失败: {}", e)))?;
            tracing::info!("🗑️ Sync配置文件已删除: {:?}", self.config_path);
        }
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.load().map(|config| config.enabled).unwrap_or(false)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.remote_path, "/ccr/");
        assert!(!config.auto_sync);
    }

    #[test]
    fn test_sync_config_manager_load_save() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("sync.toml");
        let config = SyncConfig {
            enabled: true,
            webdav_url: "https://dav.jianguoyun.com/dav/".to_string(),
            username: "test@example.com".to_string(),
            password: "test_password".to_string(),
            remote_path: "/ccr/".to_string(),
            auto_sync: false,
        };

        let manager = SyncConfigManager::new(&config_path);
        manager.save(&config).unwrap();
        assert!(config_path.exists());

        let loaded = manager.load().unwrap();
        assert!(loaded.enabled);
        assert_eq!(loaded.username, "test@example.com");
        assert_eq!(loaded.remote_path, "/ccr/");
    }

    #[test]
    fn test_sync_config_manager_delete() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("sync.toml");
        let manager = SyncConfigManager::new(&config_path);

        manager.save(&SyncConfig::default()).unwrap();
        assert!(config_path.exists());

        manager.delete().unwrap();
        assert!(!config_path.exists());
    }
}
