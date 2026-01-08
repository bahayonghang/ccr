// 🔧 配置管理器
// 负责配置文件的加载、保存和管理

use crate::core::error::{CcrError, Result};
use crate::core::fileio;
use crate::managers::config::ccs_config::CcsConfig;
use crate::managers::config::types::GlobalSettings;
use indexmap::IndexMap;
use std::path::{Path, PathBuf};

/// 🔧 配置管理器
///
/// 负责配置文件的加载、保存和管理
pub struct ConfigManager {
    config_path: PathBuf,
    file_handler: crate::managers::config_file_handler::ConfigFileHandler,
}

impl ConfigManager {
    /// 🏗️ 创建新的配置管理器
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        let path_buf = config_path.as_ref().to_path_buf();
        let file_handler = crate::managers::config_file_handler::ConfigFileHandler::new(&path_buf);

        Self {
            config_path: path_buf,
            file_handler,
        }
    }

    /// 🏠 使用默认配置路径创建管理器 (Unified 模式)
    pub fn with_default() -> Result<Self> {
        // 🔍 检测 Unified 模式配置路径
        let (is_unified, unified_config_path) = Self::detect_unified_mode();

        if !is_unified {
            return Err(CcrError::ConfigError(
                "未找到 Unified 模式配置。请先运行 'ccr init' 初始化配置。".into(),
            ));
        }

        let unified_path = unified_config_path
            .ok_or_else(|| CcrError::ConfigError("无法获取 Unified 配置路径".into()))?;

        let unified_root = unified_path
            .parent()
            .ok_or_else(|| CcrError::ConfigError("无法获取 CCR 根目录".into()))?;

        let platform_config_manager =
            crate::managers::PlatformConfigManager::new(unified_path.clone());
        let unified_config = platform_config_manager.load_or_create_default()?;

        let platform = &unified_config.current_platform;
        let platform_profiles_path = unified_root
            .join("platforms")
            .join(platform)
            .join("profiles.toml");

        // 如果 profiles.toml 不存在，创建默认空配置
        if !platform_profiles_path.exists() {
            tracing::debug!(
                "⚙️  未找到平台 profiles 文件: {:?}，正在创建默认空配置",
                platform_profiles_path
            );

            if let Some(parent_dir) = platform_profiles_path.parent() {
                std::fs::create_dir_all(parent_dir)
                    .map_err(|e| CcrError::ConfigError(format!("创建平台目录失败: {}", e)))?;
            }

            let default_ccs = CcsConfig {
                default_config: "default".to_string(),
                current_config: "default".to_string(),
                settings: GlobalSettings::default(),
                sections: IndexMap::new(),
            };

            fileio::write_toml(&platform_profiles_path, &default_ccs)?;
        }

        tracing::debug!(
            "🔄 Unified 模式: 使用平台 {} 的配置路径: {:?}",
            platform,
            platform_profiles_path
        );
        Ok(Self::new(platform_profiles_path))
    }

    /// 📁 获取配置文件路径
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// 📖 加载配置文件
    pub fn load(&self) -> Result<CcsConfig> {
        self.file_handler.load()
    }

    /// 💾 保存配置文件
    pub fn save(&self, config: &CcsConfig) -> Result<()> {
        self.file_handler.save(config)
    }

    /// 💾 备份配置文件
    pub fn backup(&self, tag: Option<&str>) -> Result<PathBuf> {
        self.file_handler.backup(tag)
    }

    /// 📋 列出所有配置备份文件
    #[allow(dead_code)]
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        self.file_handler.list_backups()
    }

    // === Unified 模式检测方法 ===

    /// 🔍 检测是否启用了统一模式
    pub fn detect_unified_mode() -> (bool, Option<PathBuf>) {
        // 1. 检查环境变量
        if let Ok(ccr_root) = std::env::var("CCR_ROOT") {
            let root_path = PathBuf::from(ccr_root);
            let config_path = root_path.join("config.toml");
            return (true, Some(config_path));
        }

        // 2. 检查默认统一配置路径
        if let Some(home) = dirs::home_dir() {
            let unified_root = home.join(".ccr");
            let unified_config = unified_root.join("config.toml");

            if unified_root.exists() && unified_config.exists() {
                return (true, Some(unified_config));
            }
        }

        (false, None)
    }
}
