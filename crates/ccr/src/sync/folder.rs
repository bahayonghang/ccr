// 📁 CCR Sync Folder 数据模型
// 负责定义同步文件夹的数据结构和操作
//
// 核心职责:
// - 🗂️ 定义 SyncFolder 同步文件夹结构
// - 📦 定义 SyncFoldersConfig 配置容器
// - 🔄 路径扩展和验证
// - 🏗️ Builder 模式便捷构建

use ccr_core::core::error::{CcrError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 📁 同步文件夹
///
/// 代表一个独立的同步单元，可以是文件或目录
///
/// # Examples
///
/// ```no_run
/// use ccr::SyncFolder;
///
/// let folder = SyncFolder::builder()
///     .name("claude")
///     .description("Claude Code configuration")
///     .local_path("~/.claude")
///     .remote_path("/ccr-sync/claude")
///     .enabled(true)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncFolder {
    /// 📝 唯一标识名称
    ///
    /// 用于命令行操作: `ccr sync <name> push`
    pub name: String,

    /// 📄 描述信息
    ///
    /// 人类可读的描述，方便识别用途
    #[serde(default)]
    pub description: String,

    /// 📂 本地路径
    ///
    /// 支持 `~` 扩展为用户主目录
    /// 可以是文件或目录
    pub local_path: String,

    /// ☁️ 远程 WebDAV 路径
    ///
    /// 完整的远程路径（相对于 WebDAV 根目录）
    pub remote_path: String,

    /// ✅ 是否启用同步
    ///
    /// false 时不参与批量同步操作
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// ⚡ 自动同步标志（预留）
    ///
    /// 未来功能：变更时自动同步
    #[serde(default)]
    pub auto_sync: bool,

    /// 🚫 排除模式列表
    ///
    /// 符合这些模式的文件/目录不会被同步
    /// 例如: ["*.log", ".locks/", "cache/"]
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

/// 默认启用状态
fn default_enabled() -> bool {
    true
}

impl SyncFolder {
    /// 🏗️ 创建 Builder 用于构建 SyncFolder
    pub fn builder() -> SyncFolderBuilder {
        SyncFolderBuilder::new()
    }

    /// 🔄 扩展本地路径（将 ~ 替换为用户主目录）
    ///
    /// # Returns
    ///
    /// 绝对路径的 PathBuf
    ///
    /// # Errors
    ///
    /// 如果无法获取用户主目录，返回错误
    pub fn expand_local_path(&self) -> Result<PathBuf> {
        expand_path(&self.local_path)
    }

    /// ✅ 验证文件夹配置
    ///
    /// 检查:
    /// - 名称非空
    /// - 本地路径非空
    /// - 远程路径非空
    /// - 本地路径可扩展
    ///
    /// # Returns
    ///
    /// 验证错误列表，为空表示验证通过
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // 检查名称
        if self.name.trim().is_empty() {
            errors.push("文件夹名称不能为空".to_string());
        }

        // 检查名称格式（仅允许字母、数字、下划线、短横线）
        if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            errors.push(format!(
                "文件夹名称 '{}' 包含非法字符，只允许字母、数字、下划线和短横线",
                self.name
            ));
        }

        // 检查本地路径
        if self.local_path.trim().is_empty() {
            errors.push("本地路径不能为空".to_string());
        } else {
            // 尝试扩展路径
            if let Err(e) = self.expand_local_path() {
                errors.push(format!("无法扩展本地路径 '{}': {}", self.local_path, e));
            }
        }

        // 检查远程路径
        if self.remote_path.trim().is_empty() {
            errors.push("远程路径不能为空".to_string());
        }

        errors
    }

    /// 📊 检查本地路径是否存在
    ///
    /// # Returns
    ///
    /// 路径存在返回 true，否则返回 false
    pub fn local_path_exists(&self) -> bool {
        self.expand_local_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// 📁 检查本地路径是否为目录
    ///
    /// # Returns
    ///
    /// 是目录返回 true，是文件或不存在返回 false
    #[allow(dead_code)]
    pub fn is_directory(&self) -> bool {
        self.expand_local_path()
            .map(|p| p.is_dir())
            .unwrap_or(false)
    }
}

/// 🏗️ SyncFolder Builder
///
/// 便捷构建 SyncFolder 的构建器模式
pub struct SyncFolderBuilder {
    name: Option<String>,
    description: String,
    local_path: Option<String>,
    remote_path: Option<String>,
    enabled: bool,
    auto_sync: bool,
    exclude_patterns: Vec<String>,
}

#[allow(dead_code)]
impl SyncFolderBuilder {
    /// 创建新的 Builder
    fn new() -> Self {
        Self {
            name: None,
            description: String::new(),
            local_path: None,
            remote_path: None,
            enabled: true,
            auto_sync: false,
            exclude_patterns: Vec::new(),
        }
    }

    /// 设置名称
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// 设置描述
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// 设置本地路径
    pub fn local_path(mut self, path: impl Into<String>) -> Self {
        self.local_path = Some(path.into());
        self
    }

    /// 设置远程路径
    pub fn remote_path(mut self, path: impl Into<String>) -> Self {
        self.remote_path = Some(path.into());
        self
    }

    /// 设置启用状态
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// 设置自动同步
    #[allow(dead_code)]
    pub fn auto_sync(mut self, auto_sync: bool) -> Self {
        self.auto_sync = auto_sync;
        self
    }

    /// 设置排除模式
    pub fn exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    /// 添加单个排除模式
    pub fn add_exclude_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// 构建 SyncFolder
    ///
    /// # Errors
    ///
    /// 如果缺少必需字段，返回错误
    pub fn build(self) -> Result<SyncFolder> {
        let name = self
            .name
            .ok_or_else(|| CcrError::ValidationError("缺少必需字段: name".to_string()))?;

        let local_path = self
            .local_path
            .ok_or_else(|| CcrError::ValidationError("缺少必需字段: local_path".to_string()))?;

        let remote_path = self
            .remote_path
            .ok_or_else(|| CcrError::ValidationError("缺少必需字段: remote_path".to_string()))?;

        let folder = SyncFolder {
            name,
            description: self.description,
            local_path,
            remote_path,
            enabled: self.enabled,
            auto_sync: self.auto_sync,
            exclude_patterns: self.exclude_patterns,
        };

        // 验证配置
        let errors = folder.validate();
        if !errors.is_empty() {
            return Err(CcrError::ValidationError(errors.join("; ")));
        }

        Ok(folder)
    }
}

/// ☁️ WebDAV 配置
///
/// 从 sync_config.rs 复用，所有文件夹共享此配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebDavConfig {
    /// 🌐 WebDAV 服务器地址
    pub url: String,

    /// 👤 用户名
    pub username: String,

    /// 🔑 密码/应用密码
    pub password: String,

    /// 📁 基础远程路径
    ///
    /// 所有文件夹的远程路径相对于此基础路径
    #[serde(default = "default_base_remote_path")]
    pub base_remote_path: String,
}

/// 默认基础远程路径
fn default_base_remote_path() -> String {
    "/ccr-sync".to_string()
}

impl Default for WebDavConfig {
    fn default() -> Self {
        Self {
            url: "https://dav.jianguoyun.com/dav/".to_string(),
            username: String::new(),
            password: String::new(),
            base_remote_path: default_base_remote_path(),
        }
    }
}

/// 📦 同步文件夹配置容器
///
/// 包含 WebDAV 配置和所有注册的同步文件夹
///
/// # Config File Format
///
/// ```toml
/// version = "1.0"
///
/// [webdav]
/// url = "https://dav.jianguoyun.com/dav/"
/// username = "user@example.com"
/// password = "app_password"
/// base_remote_path = "/ccr-sync"
///
/// [[folder]]
/// name = "claude"
/// description = "Claude Code configuration"
/// local_path = "~/.claude"
/// remote_path = "/ccr-sync/claude"
/// enabled = true
/// auto_sync = false
/// exclude_patterns = ["*.log", ".locks/", "cache/"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFoldersConfig {
    /// 📌 配置版本
    #[serde(default = "default_version")]
    pub version: String,

    /// ☁️ WebDAV 配置（共享）
    pub webdav: WebDavConfig,

    /// 📁 文件夹列表
    #[serde(default)]
    pub folders: Vec<SyncFolder>,
}

/// 默认配置版本
fn default_version() -> String {
    "1.0".to_string()
}

impl Default for SyncFoldersConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            webdav: WebDavConfig::default(),
            folders: Vec::new(),
        }
    }
}

impl SyncFoldersConfig {
    /// ✅ 验证配置
    ///
    /// 检查:
    /// - 所有文件夹名称唯一
    /// - 所有文件夹配置有效
    ///
    /// # Returns
    ///
    /// 验证错误列表，为空表示验证通过
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // 检查文件夹名称唯一性
        let mut names = std::collections::HashSet::new();
        for folder in &self.folders {
            if !names.insert(&folder.name) {
                errors.push(format!("文件夹名称重复: '{}'", folder.name));
            }
        }

        // 验证每个文件夹
        for folder in &self.folders {
            let folder_errors = folder.validate();
            for error in folder_errors {
                errors.push(format!("文件夹 '{}': {}", folder.name, error));
            }
        }

        // 检查 WebDAV 配置
        if self.webdav.url.trim().is_empty() {
            errors.push("WebDAV URL 不能为空".to_string());
        }
        if self.webdav.username.trim().is_empty() {
            errors.push("WebDAV 用户名不能为空".to_string());
        }

        errors
    }

    /// 🔍 根据名称查找文件夹
    ///
    /// # Returns
    ///
    /// 找到返回引用，否则返回 None
    pub fn find_folder(&self, name: &str) -> Option<&SyncFolder> {
        self.folders.iter().find(|f| f.name == name)
    }

    /// 🔍 根据名称查找文件夹（可变引用）
    pub fn find_folder_mut(&mut self, name: &str) -> Option<&mut SyncFolder> {
        self.folders.iter_mut().find(|f| f.name == name)
    }

    /// ✅ 检查文件夹名称是否存在
    pub fn has_folder(&self, name: &str) -> bool {
        self.find_folder(name).is_some()
    }

    /// 📋 列出所有启用的文件夹
    pub fn enabled_folders(&self) -> Vec<&SyncFolder> {
        self.folders.iter().filter(|f| f.enabled).collect()
    }

    /// 📊 统计信息
    pub fn stats(&self) -> FolderStats {
        let total = self.folders.len();
        let enabled = self.folders.iter().filter(|f| f.enabled).count();
        let disabled = total - enabled;

        FolderStats {
            total,
            enabled,
            disabled,
        }
    }
}

/// 📊 文件夹统计信息
#[derive(Debug, Clone)]
pub struct FolderStats {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
}

/// 🔄 扩展路径（将 ~ 替换为用户主目录）
///
/// # Examples
///
/// ```no_run
/// use ccr::expand_path;
///
/// let path = expand_path("~/.claude").unwrap();
/// // 返回类似: /home/username/.claude
/// ```
///
/// # Errors
///
/// 如果无法获取用户主目录，返回错误
pub fn expand_path(path: &str) -> Result<PathBuf> {
    if path.starts_with('~') {
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;

        if path == "~" {
            return Ok(home);
        }

        if let Some(relative) = path.strip_prefix("~/") {
            return Ok(home.join(relative));
        }

        // 不支持 ~user 形式
        Err(CcrError::ConfigError(format!(
            "不支持的路径格式: {}（仅支持 ~ 或 ~/path）",
            path
        )))
    } else {
        Ok(PathBuf::from(path))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_folder_builder() {
        let folder = SyncFolder::builder()
            .name("claude")
            .description("Claude Code config")
            .local_path("~/.claude")
            .remote_path("/ccr-sync/claude")
            .enabled(true)
            .add_exclude_pattern("*.log")
            .add_exclude_pattern(".locks/")
            .build()
            .unwrap();

        assert_eq!(folder.name, "claude");
        assert_eq!(folder.description, "Claude Code config");
        assert_eq!(folder.local_path, "~/.claude");
        assert_eq!(folder.remote_path, "/ccr-sync/claude");
        assert!(folder.enabled);
        assert_eq!(folder.exclude_patterns.len(), 2);
    }

    #[test]
    fn test_sync_folder_validation() {
        let folder = SyncFolder {
            name: "test".to_string(),
            description: "Test folder".to_string(),
            local_path: "~/.test".to_string(),
            remote_path: "/test".to_string(),
            enabled: true,
            auto_sync: false,
            exclude_patterns: vec![],
        };

        let errors = folder.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_sync_folder_validation_empty_name() {
        let folder = SyncFolder {
            name: "".to_string(),
            description: "Test".to_string(),
            local_path: "~/.test".to_string(),
            remote_path: "/test".to_string(),
            enabled: true,
            auto_sync: false,
            exclude_patterns: vec![],
        };

        let errors = folder.validate();
        assert!(!errors.is_empty());
        assert!(errors[0].contains("名称不能为空"));
    }

    #[test]
    fn test_sync_folder_validation_invalid_name() {
        let folder = SyncFolder {
            name: "test folder".to_string(), // 空格非法
            description: "Test".to_string(),
            local_path: "~/.test".to_string(),
            remote_path: "/test".to_string(),
            enabled: true,
            auto_sync: false,
            exclude_patterns: vec![],
        };

        let errors = folder.validate();
        assert!(!errors.is_empty());
        assert!(errors[0].contains("非法字符"));
    }

    #[test]
    fn test_expand_path() {
        let home = dirs::home_dir().unwrap();

        // 测试 ~ 扩展
        let expanded = expand_path("~").unwrap();
        assert_eq!(expanded, home);

        // 测试 ~/path 扩展
        let expanded = expand_path("~/.test").unwrap();
        assert_eq!(expanded, home.join(".test"));

        // 测试普通路径
        let expanded = expand_path("/absolute/path").unwrap();
        assert_eq!(expanded, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_sync_folders_config_validation() {
        let mut config = SyncFoldersConfig::default();
        config.webdav.url = "https://dav.example.com/".to_string();
        config.webdav.username = "test@example.com".to_string();
        config.webdav.password = "password".to_string();

        config.folders.push(
            SyncFolder::builder()
                .name("folder1")
                .local_path("~/.test1")
                .remote_path("/test1")
                .build()
                .unwrap(),
        );

        config.folders.push(
            SyncFolder::builder()
                .name("folder2")
                .local_path("~/.test2")
                .remote_path("/test2")
                .build()
                .unwrap(),
        );

        let errors = config.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_sync_folders_config_duplicate_names() {
        let mut config = SyncFoldersConfig::default();
        config.webdav.url = "https://dav.example.com/".to_string();
        config.webdav.username = "test@example.com".to_string();

        config.folders.push(
            SyncFolder::builder()
                .name("test")
                .local_path("~/.test1")
                .remote_path("/test1")
                .build()
                .unwrap(),
        );

        config.folders.push(
            SyncFolder::builder()
                .name("test") // 重复名称
                .local_path("~/.test2")
                .remote_path("/test2")
                .build()
                .unwrap(),
        );

        let errors = config.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("重复")));
    }

    #[test]
    fn test_sync_folders_config_find_folder() {
        let mut config = SyncFoldersConfig::default();
        config.folders.push(
            SyncFolder::builder()
                .name("test")
                .local_path("~/.test")
                .remote_path("/test")
                .build()
                .unwrap(),
        );

        assert!(config.find_folder("test").is_some());
        assert!(config.find_folder("nonexistent").is_none());
    }

    #[test]
    fn test_sync_folders_config_enabled_folders() {
        let mut config = SyncFoldersConfig::default();
        config.folders.push(
            SyncFolder::builder()
                .name("enabled")
                .local_path("~/.test1")
                .remote_path("/test1")
                .enabled(true)
                .build()
                .unwrap(),
        );

        config.folders.push(
            SyncFolder::builder()
                .name("disabled")
                .local_path("~/.test2")
                .remote_path("/test2")
                .enabled(false)
                .build()
                .unwrap(),
        );

        let enabled = config.enabled_folders();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "enabled");
    }

    #[test]
    fn test_folder_stats() {
        let mut config = SyncFoldersConfig::default();
        config.folders.push(
            SyncFolder::builder()
                .name("f1")
                .local_path("~/.test1")
                .remote_path("/test1")
                .enabled(true)
                .build()
                .unwrap(),
        );

        config.folders.push(
            SyncFolder::builder()
                .name("f2")
                .local_path("~/.test2")
                .remote_path("/test2")
                .enabled(false)
                .build()
                .unwrap(),
        );

        let stats = config.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.enabled, 1);
        assert_eq!(stats.disabled, 1);
    }
}
