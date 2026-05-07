// 🎯 CCR 平台配置管理模块
// 📁 负责读写和管理 ~/.ccr/config.toml 统一配置注册表
//
// 核心职责:
// - 🔍 管理多平台配置注册表
// - 💾 追踪当前激活的平台和 profile
// - ✅ 平台启用/禁用状态管理
// - 📋 跨平台配置协调

use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::fileio;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 🎨 平台注册信息
///
/// 存储单个平台在注册表中的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfigEntry {
    /// 🔌 平台是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// ▶️ 当前激活的 profile 名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile: Option<String>,

    /// 📝 平台描述(可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 🕐 最后使用时间(ISO 8601 格式)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for PlatformConfigEntry {
    fn default() -> Self {
        Self {
            enabled: true,
            current_profile: None,
            description: None,
            last_used: None,
        }
    }
}

/// 📦 统一配置注册表结构
///
/// 对应 ~/.ccr/config.toml 的完整结构
///
/// 结构说明:
/// - 🎯 default_platform: 默认平台(首次启动时使用)
/// - ▶️ current_platform: 当前激活的平台
/// - 📋 platforms: 所有平台的注册信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConfig {
    /// 🎯 默认平台
    #[serde(default = "default_platform")]
    #[serde(skip_serializing)]
    pub default_platform: String,

    /// ▶️ 当前激活的平台
    #[serde(default = "default_platform")]
    #[serde(skip_serializing)]
    pub current_platform: String,

    /// 📋 平台注册表(使用 flatten 序列化)
    #[serde(flatten)]
    pub platforms: IndexMap<String, PlatformConfigEntry>,
}

fn default_platform() -> String {
    "claude".to_string()
}

impl Default for UnifiedConfig {
    fn default() -> Self {
        let mut platforms = IndexMap::new();

        // 默认启用 Claude 平台
        platforms.insert(
            "claude".to_string(),
            PlatformConfigEntry {
                enabled: true,
                current_profile: None,
                description: Some("Claude Code AI Assistant".to_string()),
                last_used: None,
            },
        );

        Self {
            default_platform: "claude".to_string(),
            current_platform: "claude".to_string(),
            platforms,
        }
    }
}

#[allow(dead_code)]
impl UnifiedConfig {
    /// 🔍 获取指定平台的注册信息
    pub fn get_platform(&self, name: &str) -> Result<&PlatformConfigEntry> {
        self.platforms
            .get(name)
            .ok_or_else(|| CcrError::PlatformNotFound(name.to_string()))
    }

    /// 🔍 获取指定平台的可变注册信息
    pub fn get_platform_mut(&mut self, name: &str) -> Result<&mut PlatformConfigEntry> {
        self.platforms
            .get_mut(name)
            .ok_or_else(|| CcrError::PlatformNotFound(name.to_string()))
    }

    /// ▶️ 获取当前平台的注册信息
    /// Reset legacy routing fields before saving a migrated registry.
    pub fn clean_legacy_routing_fields(&mut self) {
        self.default_platform = default_platform();
        self.current_platform = default_platform();
    }

    #[allow(dead_code)]
    #[deprecated(note = "use get_current_profile(platform_name) instead")]
    pub fn get_current_platform(&self) -> Result<&PlatformConfigEntry> {
        self.get_platform(&self.current_platform)
    }

    /// 🔄 切换当前平台
    ///
    /// 切换前会验证目标平台是否存在且启用
    #[deprecated(note = "active-platform routing is retired; use per-platform current_profile")]
    pub fn set_current_platform(&mut self, name: &str) -> Result<()> {
        // ✅ 验证平台存在
        let registry = self.get_platform(name)?;

        // ✅ 验证平台已启用
        if !registry.enabled {
            return Err(CcrError::ConfigError(format!(
                "平台 '{}' 未启用，无法切换",
                name
            )));
        }

        // 🕐 更新当前平台的最后使用时间
        // 克隆 current_platform 以避免借用冲突
        let old_platform = self.current_platform.clone();
        if let Ok(current_registry) = self.get_platform_mut(&old_platform) {
            current_registry.last_used = Some(chrono::Utc::now().to_rfc3339());
        }

        self.current_platform = name.to_string();
        tracing::debug!("✅ 切换到平台: {}", name);
        Ok(())
    }

    /// ➕ 注册新平台
    ///
    /// 如果平台已存在，则更新其信息
    pub fn register_platform(&mut self, name: String, registry: PlatformConfigEntry) -> Result<()> {
        if self.platforms.contains_key(&name) {
            tracing::debug!("🔄 更新平台注册信息: {}", name);
        } else {
            tracing::debug!("➕ 注册新平台: {}", name);
        }

        self.platforms.insert(name, registry);
        Ok(())
    }

    /// ➖ 注销平台
    ///
    /// 如果要注销的是当前平台，会自动切换到默认平台
    pub fn unregister_platform(&mut self, name: &str) -> Result<PlatformConfigEntry> {
        // ✅ 防止注销当前平台时没有后备平台
        if name == self.current_platform && self.platforms.len() <= 1 {
            return Err(CcrError::ConfigError("不能注销唯一的平台".to_string()));
        }

        // 如果注销的是当前平台，切换到默认平台
        if name == self.current_platform {
            self.current_platform = self.default_platform.clone();
            tracing::debug!("⚠️ 注销当前平台，自动切换到: {}", self.default_platform);
        }

        self.platforms
            .shift_remove(name)
            .ok_or_else(|| CcrError::PlatformNotFound(name.to_string()))
    }

    /// 🔌 启用平台
    pub fn enable_platform(&mut self, name: &str) -> Result<()> {
        let registry = self.get_platform_mut(name)?;
        registry.enabled = true;
        tracing::debug!("✅ 启用平台: {}", name);
        Ok(())
    }

    /// 🔌 禁用平台
    ///
    /// 如果要禁用的是当前平台，会自动切换到默认平台
    pub fn disable_platform(&mut self, name: &str) -> Result<()> {
        // ✅ 防止禁用当前激活的平台
        if name == self.current_platform {
            return Err(CcrError::ConfigError("不能禁用当前激活的平台".to_string()));
        }

        let registry = self.get_platform_mut(name)?;
        registry.enabled = false;
        tracing::debug!("🔌 禁用平台: {}", name);
        Ok(())
    }

    /// 📜 列出所有已启用的平台名称
    pub fn list_enabled_platforms(&self) -> Vec<&String> {
        self.platforms
            .iter()
            .filter(|(_, registry)| registry.enabled)
            .map(|(name, _)| name)
            .collect()
    }

    /// 📜 列出所有平台名称(包括禁用的)
    pub fn list_all_platforms(&self) -> Vec<&String> {
        self.platforms.keys().collect()
    }

    /// 🔄 设置平台的当前 profile
    pub fn set_platform_profile(&mut self, platform_name: &str, profile_name: &str) -> Result<()> {
        let registry = self.get_platform_mut(platform_name)?;
        registry.current_profile = Some(profile_name.to_string());
        registry.last_used = Some(chrono::Utc::now().to_rfc3339());
        tracing::debug!("✅ 设置平台 {} 的 profile: {}", platform_name, profile_name);
        Ok(())
    }

    /// 🔍 获取平台的当前 profile
    pub fn get_platform_profile(&self, platform_name: &str) -> Result<Option<&str>> {
        let registry = self.get_platform(platform_name)?;
        Ok(registry.current_profile.as_deref())
    }

    /// Set the current profile for a specific platform.
    pub fn set_current_profile(
        &mut self,
        platform_name: &str,
        profile_name: Option<&str>,
    ) -> Result<()> {
        let registry = self.get_platform_mut(platform_name)?;
        registry.current_profile = profile_name.map(str::to_string);
        registry.last_used = Some(chrono::Utc::now().to_rfc3339());
        tracing::debug!("set platform {} current profile", platform_name);
        Ok(())
    }

    /// Get the current profile for a specific platform.
    pub fn get_current_profile(&self, platform_name: &str) -> Result<Option<&str>> {
        self.get_platform_profile(platform_name)
    }
}

/// 🔧 平台配置管理器
///
/// 负责统一配置注册表的加载、保存和管理
///
/// 主要功能:
/// - 📖 从磁盘加载 TOML 配置注册表
/// - 💾 保存配置到磁盘
/// - 🔍 解析和验证配置格式
/// - 🎯 管理多平台协调
pub struct PlatformConfigManager {
    config_path: PathBuf,
}

#[allow(dead_code)]
impl PlatformConfigManager {
    /// 🏗️ 创建新的平台配置管理器
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    /// 🏠 使用默认配置路径创建管理器
    ///
    /// 默认路径: ~/.ccr/config.toml
    ///
    /// ⚙️ **开发者注意**：
    /// 可以通过环境变量 `CCR_ROOT` 覆盖默认根目录
    ///
    /// 示例：
    /// ```bash
    /// export CCR_ROOT=/tmp/ccr_dev
    /// cargo run -- platform list
    /// ```
    pub fn with_default() -> Result<Self> {
        // 🔍 检查环境变量
        let ccr_root = if let Ok(custom_root) = std::env::var("CCR_ROOT") {
            PathBuf::from(custom_root)
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
            home.join(".ccr")
        };

        let config_path = ccr_root.join("config.toml");
        tracing::debug!("使用平台配置路径: {:?}", config_path);
        Ok(Self::new(config_path))
    }

    /// 📁 获取配置文件路径
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// 📁 获取 CCR 根目录
    #[allow(dead_code)]
    pub fn root_dir(&self) -> Result<&Path> {
        self.config_path
            .parent()
            .ok_or_else(|| CcrError::ConfigError("配置文件路径没有父目录".into()))
    }

    /// 📖 加载配置文件
    ///
    /// 执行步骤:
    /// 1. ✅ 检查文件是否存在
    /// 2. 📄 读取文件内容
    /// 3. 🔍 解析 TOML 格式
    ///
    /// 如果文件不存在，返回默认配置(不自动创建文件)
    ///
    /// ⚠️ **并发安全**: 此方法不加锁，调用方需要在外层使用 CONFIG_LOCK 保护 RMW 序列
    pub fn load(&self) -> Result<UnifiedConfig> {
        // ✅ 检查文件是否存在
        if !self.config_path.exists() {
            tracing::debug!("⚠️ 配置文件不存在，使用默认配置: {:?}", self.config_path);
            return Ok(UnifiedConfig::default());
        }

        // 使用统一的 fileio 读取 TOML
        let config: UnifiedConfig = fileio::read_toml(&self.config_path)?;

        tracing::debug!(
            "✅ 成功加载平台配置文件: {:?}, 平台数量: {}",
            self.config_path,
            config.platforms.len()
        );

        Ok(config)
    }

    /// 💾 保存配置文件
    ///
    /// 执行步骤:
    /// 1. 🗂️ 确保父目录存在
    /// 2. 📝 序列化为 TOML 格式
    /// 3. 💾 写入磁盘
    ///
    /// ⚠️ **并发安全**: 此方法不加锁，调用方需要在外层使用 CONFIG_LOCK 保护 RMW 序列
    pub fn save(&self, config: &UnifiedConfig) -> Result<()> {
        let mut clean_config = config.clone();
        clean_config.clean_legacy_routing_fields();
        // 使用统一的 fileio 写入 TOML（会自动创建父目录）
        fileio::write_toml(&self.config_path, &clean_config)?;

        tracing::debug!("✅ 平台配置文件已保存: {:?}", self.config_path);
        Ok(())
    }

    /// 🔄 加载或创建默认配置
    ///
    /// 如果配置文件不存在，创建并保存默认配置
    pub fn load_or_create_default(&self) -> Result<UnifiedConfig> {
        if self.config_path.exists() {
            self.load()
        } else {
            let default_config = UnifiedConfig::default();
            self.save(&default_config)?;
            tracing::debug!("✅ 创建默认平台配置文件: {:?}", self.config_path);
            Ok(default_config)
        }
    }

    /// 💾 备份配置文件
    ///
    /// 执行流程:
    /// 1. ✅ 验证源文件存在
    /// 2. 🏷️ 生成带时间戳的备份文件名
    /// 3. 📋 复制文件到备份位置
    ///
    /// 文件名格式: config.toml.{timestamp}.bak
    /// 备份位置: ~/.ccr/backups/
    pub fn backup(&self, tag: Option<&str>) -> Result<PathBuf> {
        // ✅ 验证源文件存在
        if !self.config_path.exists() {
            return Err(CcrError::ConfigMissing(
                self.config_path.display().to_string(),
            ));
        }

        // 🗂️ 确保备份目录存在
        let backup_dir = self.root_dir()?.join("backups");
        fs::create_dir_all(&backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("创建备份目录失败: {}", e)))?;

        // 🏷️ 生成备份文件名(带时间戳)
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_filename = if let Some(tag_str) = tag {
            format!("config_{}_{}.toml.bak", tag_str, timestamp)
        } else {
            format!("config_{}.toml.bak", timestamp)
        };

        let backup_path = backup_dir.join(backup_filename);

        // 📋 复制文件
        fs::copy(&self.config_path, &backup_path)
            .map_err(|e| CcrError::ConfigError(format!("备份配置文件失败: {}", e)))?;

        tracing::debug!("✅ 平台配置已备份到: {:?}", backup_path);
        Ok(backup_path)
    }

    /// 🔄 从备份恢复配置
    pub fn restore(&self, backup_path: &Path) -> Result<()> {
        if !backup_path.exists() {
            return Err(CcrError::ConfigMissing(backup_path.display().to_string()));
        }

        // 📋 复制备份文件到配置位置
        fs::copy(backup_path, &self.config_path)
            .map_err(|e| CcrError::ConfigError(format!("恢复配置文件失败: {}", e)))?;

        tracing::debug!("✅ 已从备份恢复配置: {:?}", backup_path);
        Ok(())
    }

    /// 📜 列出所有备份文件
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let backup_dir = self.root_dir()?.join("backups");

        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        for entry in fs::read_dir(&backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("读取备份目录失败: {}", e)))?
        {
            let entry =
                entry.map_err(|e| CcrError::ConfigError(format!("读取备份目录条目失败: {}", e)))?;
            let path = entry.path();

            if path.is_file()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("config_") && n.ends_with(".toml.bak"))
                    .unwrap_or(false)
            {
                backups.push(path);
            }
        }

        // 按时间排序(最新的在前)
        backups.sort_by(|a, b| b.cmp(a));
        Ok(backups)
    }

    /// 🧹 清理旧备份
    ///
    /// 保留最近 N 个备份，删除其余的
    pub fn cleanup_old_backups(&self, keep_count: usize) -> Result<usize> {
        let backups = self.list_backups()?;

        if backups.len() <= keep_count {
            return Ok(0);
        }

        let to_delete = &backups[keep_count..];
        let mut deleted_count = 0;

        for backup_path in to_delete {
            if let Err(e) = fs::remove_file(backup_path) {
                tracing::warn!("删除备份文件失败: {:?}, 错误: {}", backup_path, e);
            } else {
                deleted_count += 1;
                tracing::debug!("🧹 删除旧备份: {:?}", backup_path);
            }
        }

        tracing::debug!("✅ 清理完成，删除了 {} 个旧备份", deleted_count);
        Ok(deleted_count)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_config_default() {
        let config = UnifiedConfig::default();
        assert_eq!(config.default_platform, "claude");
        assert_eq!(config.current_platform, "claude");
        assert!(config.platforms.contains_key("claude"));
    }

    #[test]
    fn test_platform_registry_default() {
        let registry = PlatformConfigEntry::default();
        assert!(registry.enabled);
        assert!(registry.current_profile.is_none());
    }

    #[test]
    fn test_set_current_profile() {
        let mut config = UnifiedConfig::default();

        // 添加新平台
        config.platforms.insert(
            "codex".to_string(),
            PlatformConfigEntry {
                enabled: true,
                current_profile: None,
                description: Some("Codex CLI".to_string()),
                last_used: None,
            },
        );

        // 切换到 codex
        config
            .set_current_profile("codex", Some("default"))
            .unwrap();
        assert_eq!(
            config.get_current_profile("codex").unwrap(),
            Some("default")
        );

        config.set_current_profile("codex", None).unwrap();
        assert_eq!(config.get_current_profile("codex").unwrap(), None);
    }

    #[test]
    fn test_disable_platform() {
        let mut config = UnifiedConfig::default();

        // 添加新平台
        config.platforms.insert(
            "gemini".to_string(),
            PlatformConfigEntry {
                enabled: true,
                current_profile: None,
                description: Some("Google Gemini CLI".to_string()),
                last_used: None,
            },
        );

        // 禁用 gemini
        config.disable_platform("gemini").unwrap();
        assert!(!config.get_platform("gemini").unwrap().enabled);
    }

    #[test]
    fn test_cannot_disable_current_platform() {
        let mut config = UnifiedConfig::default();

        // 尝试禁用当前平台应该失败
        let result = config.disable_platform("claude");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_enabled_platforms() {
        let mut config = UnifiedConfig::default();

        // 添加多个平台
        config.platforms.insert(
            "codex".to_string(),
            PlatformConfigEntry {
                enabled: true,
                current_profile: None,
                description: None,
                last_used: None,
            },
        );
        config.platforms.insert(
            "gemini".to_string(),
            PlatformConfigEntry {
                enabled: false,
                current_profile: None,
                description: None,
                last_used: None,
            },
        );

        let enabled = config.list_enabled_platforms();
        assert_eq!(enabled.len(), 2); // claude + codex
        assert!(enabled.contains(&&"claude".to_string()));
        assert!(enabled.contains(&&"codex".to_string()));
        assert!(!enabled.contains(&&"gemini".to_string()));
    }
}
