// 🔧 配置管理器
// 负责配置文件的加载、保存和管理

use crate::core::error::{CcrError, Result};
use crate::core::fileio;
use crate::managers::config::ccs_config::CcsConfig;
use crate::managers::config::migration::MigrationStatus;
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

    /// 🏠 使用默认配置路径创建管理器
    pub fn with_default() -> Result<Self> {
        // 🔍 首先检测是否为 Unified 模式
        let (is_unified, unified_config_path) = Self::detect_unified_mode();

        if is_unified {
            // 📦 Unified 模式
            if let Some(ref unified_path) = unified_config_path {
                let unified_root = unified_path
                    .parent()
                    .ok_or_else(|| CcrError::ConfigError("无法获取 CCR 根目录".into()))?;

                let platform_config_manager =
                    crate::managers::PlatformConfigManager::new(unified_path.clone());
                if let Ok(unified_config) = platform_config_manager.load() {
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
                            std::fs::create_dir_all(parent_dir).map_err(|e| {
                                CcrError::ConfigError(format!("创建平台目录失败: {}", e))
                            })?;
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
                    return Ok(Self::new(platform_profiles_path));
                }
            }
        }

        // 🔍 Legacy 模式
        let config_path = if let Ok(custom_path) = std::env::var("CCR_CONFIG_PATH") {
            std::path::PathBuf::from(custom_path)
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
            home.join(".ccs_config.toml")
        };

        tracing::debug!("📁 Legacy 模式: 使用配置路径: {:?}", config_path);
        Ok(Self::new(config_path))
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

    // === 多平台支持和迁移检测方法 ===

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

    /// 🔄 检测是否应该迁移到统一模式
    pub fn should_migrate(&self) -> Result<bool> {
        if !self.config_path.exists() {
            return Ok(false);
        }

        let (is_unified, _) = Self::detect_unified_mode();
        if is_unified {
            return Ok(false);
        }

        let config = self.load()?;
        Ok(config.sections.len() >= 2)
    }

    /// 📊 获取迁移状态信息
    pub fn get_migration_status(&self) -> MigrationStatus {
        let (is_unified, unified_path) = Self::detect_unified_mode();
        let legacy_exists = self.config_path.exists();

        let legacy_section_count = if legacy_exists {
            self.load().ok().map(|c| c.sections.len()).unwrap_or(0)
        } else {
            0
        };

        MigrationStatus {
            is_unified_mode: is_unified,
            legacy_config_exists: legacy_exists,
            legacy_config_path: self.config_path.clone(),
            unified_config_path: unified_path,
            legacy_section_count,
            should_migrate: self.should_migrate().unwrap_or(false),
        }
    }

    /// 🎯 获取当前配置模式
    #[allow(dead_code)]
    pub fn get_current_mode() -> &'static str {
        let (is_unified, _) = Self::detect_unified_mode();
        if is_unified { "Unified" } else { "Legacy" }
    }
}
