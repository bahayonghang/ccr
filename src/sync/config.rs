// ☁️ CCR Sync 配置管理模块
// 📁 负责独立管理 WebDAV 同步配置
//
// 核心职责:
// - 🔍 读取 ~/.ccr/sync.toml 配置文件
// - 💾 保存同步配置
// - ✅ 验证同步配置完整性

use crate::core::error::{CcrError, Result};
use crate::core::fileio;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// ☁️ WebDAV 同步配置结构
///
/// 用于配置文件的云端同步，默认支持坚果云
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// 🔌 是否启用同步功能
    #[serde(default)]
    pub enabled: bool,

    /// 🌐 WebDAV 服务器地址
    ///
    /// 坚果云默认地址: <https://dav.jianguoyun.com/dav/>
    /// 其他WebDAV服务器也支持
    pub webdav_url: String,

    /// 👤 用户名
    ///
    /// 对于坚果云，这是您的邮箱地址
    pub username: String,

    /// 🔑 密码/应用密码
    ///
    /// ⚠️ 对于坚果云，请使用"应用密码"而非账户密码
    /// 获取方式：账户信息 -> 安全选项 -> 添加应用 -> 生成密码
    pub password: String,

    /// 📁 远程路径
    ///
    /// 配置在WebDAV服务器上的路径
    /// 默认: /ccr/
    #[serde(default = "default_remote_path")]
    pub remote_path: String,

    /// ⚡ 自动同步模式
    ///
    /// 启用后，每次配置操作后自动同步到云端
    #[serde(default)]
    pub auto_sync: bool,
}

/// 默认远程路径
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

/// 🔧 Sync 配置管理器
///
/// 独立管理同步配置，与CLI配置分离
///
/// 配置文件位置:
/// - Unified 模式: ~/.ccr/sync.toml
/// - Legacy 模式: ~/.ccs_sync.toml
pub struct SyncConfigManager {
    config_path: PathBuf,
}

#[allow(dead_code)]
impl SyncConfigManager {
    /// 🏗️ 创建新的同步配置管理器
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    /// 🏠 使用默认配置路径创建管理器
    ///
    /// 默认路径优先级:
    /// 1. CCR_SYNC_CONFIG_PATH 环境变量
    /// 2. ~/.ccr/sync.toml (Unified 模式)
    /// 3. ~/.ccs_sync.toml (Legacy 模式)
    pub fn with_default() -> Result<Self> {
        // 1. 检查环境变量
        if let Ok(custom_path) = std::env::var("CCR_SYNC_CONFIG_PATH") {
            tracing::debug!("📁 使用环境变量指定的sync配置路径: {}", custom_path);
            return Ok(Self::new(custom_path));
        }

        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;

        // 2. 检查 ~/.ccr/ 统一模式目录
        let unified_root = home.join(".ccr");
        if unified_root.exists() {
            let sync_config_path = unified_root.join("sync.toml");
            tracing::debug!("📁 Unified 模式: 使用sync配置路径: {:?}", sync_config_path);
            return Ok(Self::new(sync_config_path));
        }

        // 3. Legacy 模式
        let legacy_sync_path = home.join(".ccs_sync.toml");
        tracing::debug!("📁 Legacy 模式: 使用sync配置路径: {:?}", legacy_sync_path);
        Ok(Self::new(legacy_sync_path))
    }

    /// 📁 获取配置文件路径
    #[allow(dead_code)]
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// 📖 加载同步配置
    ///
    /// 如果文件不存在，返回默认配置（未启用状态）
    pub fn load(&self) -> Result<SyncConfig> {
        // 如果文件不存在，返回默认配置
        if !self.config_path.exists() {
            tracing::debug!("⚙️ sync配置文件不存在，返回默认配置");
            return Ok(SyncConfig::default());
        }

        // 使用统一的 fileio 读取 TOML
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

    /// 💾 保存同步配置
    ///
    /// 自动创建父目录（如果不存在）
    pub fn save(&self, config: &SyncConfig) -> Result<()> {
        // 使用统一的 fileio 写入 TOML（会自动创建父目录）
        fileio::write_toml(&self.config_path, config)?;

        tracing::debug!("✅ Sync配置文件已保存: {:?}", self.config_path);
        Ok(())
    }

    /// ❌ 删除同步配置
    ///
    /// 删除配置文件，等同于禁用同步功能
    #[allow(dead_code)]
    pub fn delete(&self) -> Result<()> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path)
                .map_err(|e| CcrError::ConfigError(format!("删除sync配置文件失败: {}", e)))?;
            tracing::info!("🗑️ Sync配置文件已删除: {:?}", self.config_path);
        }
        Ok(())
    }

    /// 🔍 检查同步配置是否存在且已启用
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.load().map(|config| config.enabled).unwrap_or(false)
    }
}

#[cfg(test)]
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

        // 创建测试配置
        let config = SyncConfig {
            enabled: true,
            webdav_url: "https://dav.jianguoyun.com/dav/".to_string(),
            username: "test@example.com".to_string(),
            password: "test_password".to_string(),
            remote_path: "/ccr/".to_string(),
            auto_sync: false,
        };

        // 保存
        let manager = SyncConfigManager::new(&config_path);
        manager.save(&config).unwrap();
        assert!(config_path.exists());

        // 加载
        let loaded = manager.load().unwrap();
        assert!(loaded.enabled);
        assert_eq!(loaded.username, "test@example.com");
        assert_eq!(loaded.remote_path, "/ccr/");
    }

    #[test]
    fn test_sync_config_manager_delete() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("sync.toml");

        let config = SyncConfig::default();
        let manager = SyncConfigManager::new(&config_path);

        // 保存并删除
        manager.save(&config).unwrap();
        assert!(config_path.exists());

        manager.delete().unwrap();
        assert!(!config_path.exists());
    }
}
