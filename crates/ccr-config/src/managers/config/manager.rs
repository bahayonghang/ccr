// 🔧 配置管理器
// 负责配置文件的加载、保存和管理

use crate::managers::config::CcsConfig;
use ccr_core::AutoCompletable;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::{BACKUP_KEEP, WriteOptions, write_guarded};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// 🔧 配置管理器
///
/// 负责配置文件的加载、保存和管理
pub struct ConfigManager {
    profiles_file: PathBuf,
    profiles_db: PathBuf,
}

#[allow(dead_code)]
impl ConfigManager {
    /// 🏗️ 创建新的配置管理器
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        let (profiles_file, profiles_db) =
            crate::managers::config::catalog::locations(config_path.as_ref());
        Self {
            profiles_file,
            profiles_db,
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

    /// Legacy TOML path. Migration uses it only to detect a file that still needs import.
    pub fn config_path(&self) -> &Path {
        &self.profiles_file
    }

    /// Catalog database path. Resource locks follow this path.
    pub fn profiles_db(&self) -> &Path {
        &self.profiles_db
    }

    /// 📖 加载配置文件
    #[allow(dead_code)]
    pub fn load(&self) -> Result<CcsConfig> {
        crate::managers::config::catalog::load_config(&self.profiles_file, &self.profiles_db)
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
        crate::managers::config::catalog::save_catalog(
            &self.profiles_file,
            &self.profiles_db,
            config,
        )
    }

    /// Canonical catalog TOML plus `document.revision`. `None` means no catalog.
    pub fn canonical_toml(&self) -> Result<Option<(String, String)>> {
        crate::managers::config::catalog::canonical_document(&self.profiles_file, &self.profiles_db)
    }

    /// Replace the catalog from TOML when `expected` matches `document.revision`.
    /// An empty token creates a catalog only when neither file exists.
    pub fn replace_toml(&self, content: &str, expected: &str) -> Result<String> {
        crate::managers::config::catalog::replace_document(
            &self.profiles_file,
            &self.profiles_db,
            content,
            expected,
        )
    }

    /// 💾 备份当前目录。遗留 TOML 仍在时备份该文件；迁移后导出受保护的规范 TOML。
    pub fn backup(&self, tag: Option<&str>) -> Result<PathBuf> {
        if self.profiles_file.is_file() {
            return crate::managers::config_file_handler::ConfigFileHandler::new(
                &self.profiles_file,
            )
            .backup(tag);
        }
        self.backup_catalog(tag)
    }

    fn backup_catalog(&self, tag: Option<&str>) -> Result<PathBuf> {
        let Some((content, _)) = self.canonical_toml()? else {
            return Err(CcrError::ConfigMissing(
                self.profiles_file.display().to_string(),
            ));
        };
        let dir = self.profiles_file.parent().ok_or_else(|| {
            CcrError::ConfigError(format!(
                "无法获取配置目录: {}",
                self.profiles_file.display()
            ))
        })?;
        fs::create_dir_all(dir)
            .map_err(|error| CcrError::ConfigError(format!("创建配置备份目录失败: {error}")))?;
        let file_name = self
            .profiles_file
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| CcrError::ConfigError("无效的配置文件名".into()))?;
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let backup_name = match tag {
            Some(tag) => format!(
                "{file_name}.{tag}_{timestamp}.{}-{nanos}.bak",
                std::process::id()
            ),
            None => format!("{file_name}.{timestamp}.{}-{nanos}.bak", std::process::id()),
        };
        let backup_path = dir.join(backup_name);
        write_guarded(
            &backup_path,
            content.as_bytes(),
            &WriteOptions {
                secret: true,
                ..WriteOptions::default()
            },
        )?;
        rotate_named_backups(dir, file_name)?;
        Ok(backup_path)
    }

    /// 📋 列出所有配置备份文件
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        crate::managers::config_file_handler::ConfigFileHandler::new(&self.profiles_file)
            .list_backups()
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

fn rotate_named_backups(dir: &Path, file_name: &str) -> Result<()> {
    let mut backups = Vec::new();
    for entry in fs::read_dir(dir)
        .map_err(|error| CcrError::ConfigError(format!("读取配置备份目录失败: {error}")))?
    {
        let path = entry
            .map_err(|error| CcrError::ConfigError(format!("读取配置备份项失败: {error}")))?
            .path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if path.is_file() && name.starts_with(file_name) && name.ends_with(".bak") {
            backups.push(path);
        }
    }
    if backups.len() <= BACKUP_KEEP {
        return Ok(());
    }
    backups.sort_by(|left, right| {
        let left_time = fs::metadata(left)
            .and_then(|metadata| metadata.modified())
            .ok();
        let right_time = fs::metadata(right)
            .and_then(|metadata| metadata.modified())
            .ok();
        right_time.cmp(&left_time).then_with(|| right.cmp(left))
    });
    for old in &backups[BACKUP_KEEP..] {
        if let Err(error) = fs::remove_file(old) {
            tracing::warn!(path = %old.display(), error = %error, "清理旧配置备份失败");
        }
    }
    Ok(())
}
