// 📁 CCR Sync Folder Manager
// 负责管理同步文件夹注册和配置
//
// 核心职责:
// - 📖 加载和保存 sync_folders.toml 配置文件
// - ➕ 添加、删除、更新文件夹注册
// - 🔍 查询和列出文件夹
// - ✅ 配置验证
// - 🔒 文件锁确保并发安全
// - ⚛️ 原子写入防止配置损坏

#![allow(dead_code)]

use crate::sync::folder::{FolderStats, SyncFolder, SyncFoldersConfig, WebDavConfig, expand_path};
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::fileio;
use ccr_core::core::guarded_write::WriteOptions;
use std::path::{Path, PathBuf};

use super::config::SyncConfigManager;

/// 🔧 Sync文件夹管理器
///
/// 独立管理同步文件夹注册配置，与 CLI 配置分离
///
/// # 配置文件位置优先级
///
/// 1. `CCR_SYNC_FOLDERS_CONFIG` 环境变量指定的路径
/// 2. `~/.ccr/sync_folders.toml`
///
/// # Examples
///
/// ```no_run
/// use ccr_sync::SyncFolderManager;
/// use ccr_sync::SyncFolder;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut manager = SyncFolderManager::with_default()?;
///
/// // 添加新文件夹
/// let folder = SyncFolder::builder()
///     .name("claude")
///     .local_path("~/.claude")
///     .remote_path("/ccr-sync/claude")
///     .build()?;
///
/// manager.add_folder(folder)?;
///
/// // 列出所有文件夹
/// let config = manager.load_config()?;
/// for folder in &config.folders {
///     println!("{}: {}", folder.name, folder.local_path);
/// }
/// # Ok(())
/// # }
/// ```
pub struct SyncFolderManager {
    config_path: PathBuf,
}

impl SyncFolderManager {
    /// 🏗️ 创建新的同步文件夹管理器
    ///
    /// # Arguments
    ///
    /// * `config_path` - 配置文件路径
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        let config_path = config_path.as_ref().to_path_buf();

        Self { config_path }
    }

    /// 🏠 使用默认配置路径创建管理器
    ///
    /// # 路径检测逻辑
    ///
    /// 1. 检查 `CCR_SYNC_FOLDERS_CONFIG` 环境变量
    /// 2. 使用 `~/.ccr/sync_folders.toml`
    ///
    /// # Errors
    ///
    /// 如果无法获取用户主目录，返回错误
    pub fn with_default() -> Result<Self> {
        // 1. 检查环境变量
        if let Ok(custom_path) = std::env::var("CCR_SYNC_FOLDERS_CONFIG") {
            tracing::debug!("📁 使用环境变量指定的sync_folders配置路径: {}", custom_path);
            return Ok(Self::new(custom_path));
        }

        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;

        // 使用 ~/.ccr/ 目录
        let unified_root = home.join(".ccr");
        let sync_folders_path = unified_root.join("sync_folders.toml");
        tracing::debug!("📁 使用sync_folders配置路径: {:?}", sync_folders_path);
        Ok(Self::new(sync_folders_path))
    }

    /// 📁 获取配置文件路径
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// 📖 加载同步文件夹配置
    ///
    /// 如果文件不存在，返回默认空配置
    ///
    /// # Errors
    ///
    /// 如果文件格式错误或读取失败，返回错误
    pub fn load_config(&self) -> Result<SyncFoldersConfig> {
        // 如果文件不存在，返回默认配置
        if !self.config_path.exists() {
            tracing::debug!("⚙️ sync_folders配置文件不存在，返回默认配置");
            return Ok(SyncFoldersConfig::default());
        }

        // 使用统一的 fileio 读取 TOML
        let config: SyncFoldersConfig = fileio::read_toml(&self.config_path)?;

        tracing::debug!(
            "✅ 成功加载sync_folders配置文件: {:?}, 文件夹数: {}",
            self.config_path,
            config.folders.len()
        );

        Ok(config)
    }

    /// 💾 保存同步文件夹配置
    ///
    /// 使用文件锁和原子写入确保并发安全
    ///
    /// # Errors
    ///
    /// 如果序列化失败、无法获取锁或写入失败，返回错误
    pub fn save_config(&self, config: &SyncFoldersConfig) -> Result<()> {
        // 验证配置
        let errors = config.validate();
        if !errors.is_empty() {
            return Err(CcrError::ValidationError(errors.join("; ")));
        }

        // 使用统一的 fileio 写入 TOML（guarded write 已内置统一锁目录的路径锁 +
        // 原子写，无需自建 <config_dir>/.locks 锁——消除锁目录 split-brain）。
        // sync_folders.toml 含 WebDAV 密码，与 sync.toml 同样用 secret 落盘
        //（Unix 上 0o600；Windows no-op）
        fileio::write_toml_opts(
            &self.config_path,
            config,
            &WriteOptions {
                secret: true,
                ..Default::default()
            },
        )?;

        tracing::info!(
            "✅ Sync文件夹配置文件已保存: {:?}, 文件夹数: {}",
            self.config_path,
            config.folders.len()
        );

        Ok(())
    }

    /// ➕ 添加新的同步文件夹
    ///
    /// # Arguments
    ///
    /// * `folder` - 要添加的文件夹
    ///
    /// # Errors
    ///
    /// - 如果文件夹名称已存在，返回错误
    /// - 如果文件夹配置无效，返回错误
    /// - 如果保存失败，返回错误
    pub fn add_folder(&mut self, folder: SyncFolder) -> Result<()> {
        // 验证文件夹配置
        let validation_errors = folder.validate();
        if !validation_errors.is_empty() {
            return Err(CcrError::ValidationError(validation_errors.join("; ")));
        }

        let mut config = self.load_config()?;

        // 检查名称是否已存在
        if config.has_folder(&folder.name) {
            return Err(CcrError::ConfigError(format!(
                "文件夹名称 '{}' 已存在",
                folder.name
            )));
        }

        config.folders.push(folder.clone());
        self.save_config(&config)?;

        tracing::info!("➕ 已添加同步文件夹: '{}'", folder.name);
        Ok(())
    }

    /// ❌ 删除同步文件夹
    ///
    /// # Arguments
    ///
    /// * `name` - 要删除的文件夹名称
    ///
    /// # Errors
    ///
    /// - 如果文件夹不存在，返回错误
    /// - 如果保存失败，返回错误
    pub fn remove_folder(&mut self, name: &str) -> Result<()> {
        let mut config = self.load_config()?;

        // 查找文件夹索引
        let index = config
            .folders
            .iter()
            .position(|f| f.name == name)
            .ok_or_else(|| CcrError::ConfigError(format!("文件夹 '{}' 不存在", name)))?;

        config.folders.remove(index);
        self.save_config(&config)?;

        tracing::info!("❌ 已删除同步文件夹: '{}'", name);
        Ok(())
    }

    /// 🔍 获取指定文件夹的配置
    ///
    /// # Arguments
    ///
    /// * `name` - 文件夹名称
    ///
    /// # Returns
    ///
    /// 文件夹配置的克隆
    ///
    /// # Errors
    ///
    /// 如果文件夹不存在，返回错误
    pub fn get_folder(&self, name: &str) -> Result<SyncFolder> {
        let config = self.load_config()?;

        config
            .find_folder(name)
            .cloned()
            .ok_or_else(|| CcrError::ConfigError(format!("文件夹 '{}' 不存在", name)))
    }

    /// 📋 列出所有文件夹
    ///
    /// # Returns
    ///
    /// 文件夹列表的克隆
    pub fn list_folders(&self) -> Result<Vec<SyncFolder>> {
        let config = self.load_config()?;
        Ok(config.folders.clone())
    }

    /// ✏️ 更新文件夹配置
    ///
    /// # Arguments
    ///
    /// * `name` - 要更新的文件夹名称
    /// * `folder` - 新的文件夹配置
    ///
    /// # Errors
    ///
    /// - 如果文件夹不存在，返回错误
    /// - 如果新配置无效，返回错误
    /// - 如果保存失败，返回错误
    pub fn update_folder(&mut self, name: &str, folder: SyncFolder) -> Result<()> {
        // 验证新配置
        let validation_errors = folder.validate();
        if !validation_errors.is_empty() {
            return Err(CcrError::ValidationError(validation_errors.join("; ")));
        }

        let mut config = self.load_config()?;

        // 查找文件夹
        let existing = config
            .find_folder_mut(name)
            .ok_or_else(|| CcrError::ConfigError(format!("文件夹 '{}' 不存在", name)))?;

        // 更新配置
        *existing = folder;

        self.save_config(&config)?;

        tracing::info!("✏️ 已更新同步文件夹: '{}'", name);
        Ok(())
    }

    /// ✅ 启用文件夹
    ///
    /// # Arguments
    ///
    /// * `name` - 文件夹名称
    ///
    /// # Errors
    ///
    /// - 如果文件夹不存在，返回错误
    /// - 如果保存失败，返回错误
    pub fn enable_folder(&mut self, name: &str) -> Result<()> {
        let mut config = self.load_config()?;

        let folder = config
            .find_folder_mut(name)
            .ok_or_else(|| CcrError::ConfigError(format!("文件夹 '{}' 不存在", name)))?;

        folder.enabled = true;
        self.save_config(&config)?;

        tracing::info!("✅ 已启用同步文件夹: '{}'", name);
        Ok(())
    }

    /// ❌ 禁用文件夹
    ///
    /// # Arguments
    ///
    /// * `name` - 文件夹名称
    ///
    /// # Errors
    ///
    /// - 如果文件夹不存在，返回错误
    /// - 如果保存失败，返回错误
    pub fn disable_folder(&mut self, name: &str) -> Result<()> {
        let mut config = self.load_config()?;

        let folder = config
            .find_folder_mut(name)
            .ok_or_else(|| CcrError::ConfigError(format!("文件夹 '{}' 不存在", name)))?;

        folder.enabled = false;
        self.save_config(&config)?;

        tracing::info!("❌ 已禁用同步文件夹: '{}'", name);
        Ok(())
    }

    /// 🔍 检查文件夹是否存在
    ///
    /// # Arguments
    ///
    /// * `name` - 文件夹名称
    ///
    /// # Returns
    ///
    /// 存在返回 true，否则返回 false
    pub fn has_folder(&self, name: &str) -> bool {
        self.load_config()
            .map(|config| config.has_folder(name))
            .unwrap_or(false)
    }

    /// 📊 获取统计信息
    pub fn stats(&self) -> Result<FolderStats> {
        let config = self.load_config()?;
        Ok(config.stats())
    }

    /// ☁️ 获取 WebDAV 配置
    pub fn get_webdav_config(&self) -> Result<WebDavConfig> {
        let config = self.load_config()?;
        Ok(config.webdav.clone())
    }

    /// ☁️ 更新 WebDAV 配置
    ///
    /// # Arguments
    ///
    /// * `webdav_config` - 新的 WebDAV 配置
    ///
    /// # Errors
    ///
    /// 如果保存失败，返回错误
    pub fn update_webdav_config(&mut self, webdav_config: WebDavConfig) -> Result<()> {
        let mut config = self.load_config()?;
        config.webdav = webdav_config;
        self.save_config(&config)?;

        tracing::info!("☁️ 已更新 WebDAV 配置");
        Ok(())
    }

    /// 🔄 从旧版 sync.toml 配置迁移到 sync_folders.toml
    ///
    /// 此方法执行自动迁移流程:
    /// 1. 检测 sync_folders.toml 是否已存在（存在则跳过迁移）
    /// 2. 尝试加载旧版 SyncConfig (从 sync.toml 或 ~/.ccs_sync.toml)
    /// 3. 创建默认文件夹注册:
    ///    - "conf" 文件夹用于主配置文件
    ///    - "claude" 文件夹（如果 ~/.claude/ 存在）
    ///    - "gemini" 文件夹（如果 ~/.gemini/ 存在）
    ///    - "codex" 文件夹（如果 ~/.codex/ 存在）
    /// 4. 保存迁移后的配置到 sync_folders.toml
    ///
    /// # Returns
    ///
    /// 返回是否执行了迁移 (true = 已迁移, false = 无需迁移)
    ///
    /// # Errors
    ///
    /// 如果迁移过程中发生错误，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ccr_sync::SyncFolderManager;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut manager = SyncFolderManager::with_default()?;
    ///
    /// // 尝试迁移（如果需要）
    /// let migrated = manager.migrate_from_legacy()?;
    ///
    /// if migrated {
    ///     println!("✅ 已完成配置迁移");
    /// } else {
    ///     println!("ℹ️  无需迁移");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn migrate_from_legacy(&mut self) -> Result<bool> {
        // 1. 检查是否已存在 sync_folders.toml
        if self.config_path.exists() {
            tracing::debug!("✅ sync_folders.toml 已存在，无需迁移");
            return Ok(false);
        }

        tracing::info!("🔄 开始迁移同步配置到多文件夹格式...");

        // 2. 尝试加载旧版 SyncConfig
        let sync_config_manager = SyncConfigManager::with_default()?;
        let old_sync_config = sync_config_manager.load()?;

        // 3. 创建新配置（直接在初始化时设置 webdav）
        let mut new_config = SyncFoldersConfig {
            webdav: WebDavConfig {
                url: old_sync_config.webdav_url.clone(),
                username: old_sync_config.username.clone(),
                password: old_sync_config.password.clone(),
                base_remote_path: old_sync_config
                    .remote_path
                    .trim_end_matches('/')
                    .to_string(),
            },
            ..Default::default()
        };

        // 4. 创建默认文件夹注册
        let mut folder_count = 0;

        // 4.1 主配置文件夹
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;

        let config_file_path = if home.join(".ccs_config.toml").exists() {
            "~/.ccs_config.toml"
        } else {
            "~/.ccr/config.toml"
        };

        let conf_folder = SyncFolder::builder()
            .name("conf")
            .description("主配置文件")
            .local_path(config_file_path)
            .remote_path(format!("{}/config", new_config.webdav.base_remote_path))
            .enabled(true)
            .build()?;

        new_config.folders.push(conf_folder);
        folder_count += 1;
        tracing::info!("  ✓ 已添加 'conf' 文件夹");

        // 4.2 检测并添加平台目录
        let platforms = vec![
            ("claude", "~/.claude", "Claude Code 配置"),
            (
                "gemini",
                "~/.gemini",
                "Antigravity CLI 配置（保留 gemini key 兼容）",
            ),
            ("codex", "~/.codex", "Codex CLI 配置"),
        ];

        for (name, path, description) in platforms {
            let expanded = expand_path(path)?;
            if expanded.exists() {
                let folder = SyncFolder::builder()
                    .name(name)
                    .description(description)
                    .local_path(path)
                    .remote_path(format!("{}/{}", new_config.webdav.base_remote_path, name))
                    .enabled(true)
                    .add_exclude_pattern("*.log")
                    .add_exclude_pattern("cache/")
                    .add_exclude_pattern(".locks/")
                    .build()?;

                new_config.folders.push(folder);
                folder_count += 1;
                tracing::info!("  ✓ 已添加 '{}' 文件夹", name);
            }
        }

        // 5. 保存迁移后的配置
        self.save_config(&new_config)?;

        tracing::info!("✅ 配置迁移完成! 已创建 {} 个同步文件夹", folder_count);

        Ok(true)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestSyncEnv;

    #[test]
    fn test_sync_folder_manager_load_save() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("sync_folders.toml");

        let manager = SyncFolderManager::new(&config_path);

        // 加载默认配置
        let config = manager.load_config().unwrap();
        assert_eq!(config.folders.len(), 0);

        // 保存配置
        let mut new_config = SyncFoldersConfig::default();
        new_config.webdav.url = "https://dav.example.com/".to_string();
        new_config.webdav.username = "test@example.com".to_string();
        new_config.webdav.password = ccr_core::Secret::from("password");

        new_config.folders.push(
            SyncFolder::builder()
                .name("test")
                .local_path("~/.test")
                .remote_path("/test")
                .build()
                .unwrap(),
        );

        manager.save_config(&new_config).unwrap();

        // 重新加载验证
        let loaded = manager.load_config().unwrap();
        assert_eq!(loaded.folders.len(), 1);
        assert_eq!(loaded.folders[0].name, "test");
    }

    #[test]
    fn test_sync_folder_manager_add_folder() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("sync_folders.toml");

        let mut manager = SyncFolderManager::new(&config_path);

        // 初始化 WebDAV 配置
        let mut config = SyncFoldersConfig::default();
        config.webdav.url = "https://dav.example.com/".to_string();
        config.webdav.username = "test@example.com".to_string();
        config.webdav.password = ccr_core::Secret::from("password");
        manager.save_config(&config).unwrap();

        // 添加文件夹
        let folder = SyncFolder::builder()
            .name("claude")
            .local_path("~/.claude")
            .remote_path("/ccr-sync/claude")
            .build()
            .unwrap();

        manager.add_folder(folder).unwrap();

        // 验证
        let loaded = manager.load_config().unwrap();
        assert_eq!(loaded.folders.len(), 1);
        assert_eq!(loaded.folders[0].name, "claude");
    }

    #[test]
    fn test_sync_folder_manager_add_duplicate() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("sync_folders.toml");

        let mut manager = SyncFolderManager::new(&config_path);

        // 初始化配置
        let mut config = SyncFoldersConfig::default();
        config.webdav.url = "https://dav.example.com/".to_string();
        config.webdav.username = "test@example.com".to_string();
        config.webdav.password = ccr_core::Secret::from("password");
        manager.save_config(&config).unwrap();

        let folder = SyncFolder::builder()
            .name("test")
            .local_path("~/.test")
            .remote_path("/test")
            .build()
            .unwrap();

        manager.add_folder(folder.clone()).unwrap();

        // 尝试添加重复
        let result = manager.add_folder(folder);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("已存在"));
    }

    #[test]
    fn test_sync_folder_manager_remove_folder() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("sync_folders.toml");

        let mut manager = SyncFolderManager::new(&config_path);

        // 初始化并添加文件夹
        let mut config = SyncFoldersConfig::default();
        config.webdav.url = "https://dav.example.com/".to_string();
        config.webdav.username = "test@example.com".to_string();
        config.webdav.password = ccr_core::Secret::from("password");

        config.folders.push(
            SyncFolder::builder()
                .name("test")
                .local_path("~/.test")
                .remote_path("/test")
                .build()
                .unwrap(),
        );
        manager.save_config(&config).unwrap();

        // 删除文件夹
        manager.remove_folder("test").unwrap();

        // 验证
        let loaded = manager.load_config().unwrap();
        assert_eq!(loaded.folders.len(), 0);
    }

    #[test]
    fn test_sync_folder_manager_get_folder() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("sync_folders.toml");

        let manager = SyncFolderManager::new(&config_path);

        // 初始化
        let mut config = SyncFoldersConfig::default();
        config.webdav.url = "https://dav.example.com/".to_string();
        config.webdav.username = "test@example.com".to_string();
        config.webdav.password = ccr_core::Secret::from("password");

        config.folders.push(
            SyncFolder::builder()
                .name("claude")
                .description("Claude Code")
                .local_path("~/.claude")
                .remote_path("/ccr-sync/claude")
                .build()
                .unwrap(),
        );
        manager.save_config(&config).unwrap();

        // 获取文件夹
        let folder = manager.get_folder("claude").unwrap();
        assert_eq!(folder.name, "claude");
        assert_eq!(folder.description, "Claude Code");

        // 获取不存在的文件夹
        let result = manager.get_folder("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_folder_manager_enable_disable() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("sync_folders.toml");

        let mut manager = SyncFolderManager::new(&config_path);

        // 初始化
        let mut config = SyncFoldersConfig::default();
        config.webdav.url = "https://dav.example.com/".to_string();
        config.webdav.username = "test@example.com".to_string();
        config.webdav.password = ccr_core::Secret::from("password");

        config.folders.push(
            SyncFolder::builder()
                .name("test")
                .local_path("~/.test")
                .remote_path("/test")
                .enabled(true)
                .build()
                .unwrap(),
        );
        manager.save_config(&config).unwrap();

        // 禁用
        manager.disable_folder("test").unwrap();
        let folder = manager.get_folder("test").unwrap();
        assert!(!folder.enabled);

        // 启用
        manager.enable_folder("test").unwrap();
        let folder = manager.get_folder("test").unwrap();
        assert!(folder.enabled);
    }

    // 🔐 sync_folders.toml 含 WebDAV 密码，落盘应为 owner-only（0o600）。
    // 权限断言仅在 Unix 有意义；Windows 权限模型（NTFS ACL）不同，secret 为 no-op。
    #[cfg(unix)]
    #[test]
    fn test_sync_folders_config_saved_with_owner_only_mode() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("sync_folders.toml");
        let manager = SyncFolderManager::new(&config_path);

        let mut config = SyncFoldersConfig::default();
        config.webdav.password = ccr_core::Secret::from("s3cret");
        manager.save_config(&config).unwrap();

        let mode = std::fs::metadata(&config_path)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    // 🔁 旧格式（明文密码）sync_folders.toml 读入 → 保存 → 再读：无损，磁盘仍为明文
    #[test]
    fn test_legacy_plaintext_folders_file_round_trip_lossless() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("sync_folders.toml");
        let legacy = r#"
version = "1.0"

[webdav]
url = "https://dav.example.com/"
username = "user@example.com"
password = "legacy-folders-password"
base_remote_path = "/ccr-sync"
"#;
        std::fs::write(&config_path, legacy).unwrap();

        let manager = SyncFolderManager::new(&config_path);
        let loaded = manager.load_config().unwrap();
        assert_eq!(loaded.webdav.password.expose(), "legacy-folders-password");

        manager.save_config(&loaded).unwrap();
        let on_disk = std::fs::read_to_string(&config_path).unwrap();
        assert!(on_disk.contains("legacy-folders-password"));

        let reloaded = manager.load_config().unwrap();
        assert_eq!(reloaded.webdav.password, loaded.webdav.password);
    }

    #[test]
    fn test_sync_folder_manager_migration() {
        let env = TestSyncEnv::new();

        // 创建旧版 sync.toml
        let old_config = crate::sync::config::SyncConfig {
            enabled: true,
            webdav_url: "https://dav.example.com/".to_string(),
            username: "test@example.com".to_string(),
            password: ccr_core::Secret::from("password"),
            remote_path: "/ccr-sync".to_string(),
            auto_sync: false,
        };

        let sync_config_manager = SyncConfigManager::new(env.sync_config_path());
        sync_config_manager.save(&old_config).unwrap();

        // 执行迁移
        let mut manager = SyncFolderManager::new(env.sync_folders_path());
        let migrated = manager.migrate_from_legacy().unwrap();

        assert!(migrated, "应该执行了迁移");

        // 验证迁移结果
        let config = manager.load_config().unwrap();

        // 验证 WebDAV 配置
        assert_eq!(config.webdav.url, "https://dav.example.com/");
        assert_eq!(config.webdav.username, "test@example.com");
        assert_eq!(config.webdav.password, "password");
        assert_eq!(config.webdav.base_remote_path, "/ccr-sync");

        // 验证至少有 conf 文件夹
        assert!(!config.folders.is_empty(), "应该至少有一个文件夹");
        assert!(
            config.folders.iter().any(|f| f.name == "conf"),
            "应该有 conf 文件夹"
        );

        // 再次调用迁移应该返回 false（已存在）
        let migrated_again = manager.migrate_from_legacy().unwrap();
        assert!(!migrated_again, "第二次迁移应该跳过");
    }
}
