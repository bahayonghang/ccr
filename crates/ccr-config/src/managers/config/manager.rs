// 🔧 配置管理器
// 负责配置文件的加载、保存和管理

use crate::managers::config::CcsConfig;
use ccr_core::AutoCompletable;
use ccr_core::core::error::Result;
use std::path::{Path, PathBuf};

/// 🔧 配置管理器
///
/// 负责配置文件的加载、保存和管理
pub struct ConfigManager {
    config_path: PathBuf,
    file_handler: crate::managers::config_file_handler::ConfigFileHandler,
}

#[allow(dead_code)]
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

    /// Legacy callers retain the Claude domain. New callers must name a platform.
    /// Opening a repository never initializes or repairs files.
    pub fn with_default() -> Result<Self> {
        Self::for_platform("claude")
    }

    /// Open the named platform without inspecting registry order or writing files.
    pub fn for_platform(platform_name: &str) -> Result<Self> {
        let platform: crate::models::Platform = platform_name.parse()?;
        let paths = crate::models::PlatformPaths::new(platform)?;
        Ok(Self::new(paths.profiles_file))
    }

    /// Explicit initialization. Existing files, including invalid files, stay intact.
    pub fn ensure_initialized(&self) -> Result<()> {
        self.mutate_or_create(|_| Ok(()))
    }

    /// 📁 获取配置文件路径
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// 📖 加载配置文件
    #[allow(dead_code)]
    pub fn load(&self) -> Result<CcsConfig> {
        self.file_handler.load()
    }

    /// 🔄 加载配置并自动补全缺失字段（必要时写回）
    pub fn load_with_autofix(&self) -> Result<CcsConfig> {
        self.mutate(|config| {
            for section in config.sections.values_mut() {
                section.auto_complete();
            }
            Ok(config.clone())
        })
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
