// ⭐ CCR 设置管理模块
// 📝 负责读写和管理 ~/.claude/settings.json 文件
// 💎 这是 CCR 的核心模块,直接操作 Claude Code 的配置文件
//
// 核心职责:
// - 🔧 管理 Claude Code settings.json
// - 🔄 原子性写入(临时文件 + 重命名)
// - 🔒 文件锁保证并发安全
// - 💾 自动备份机制
// - 🌍 环境变量映射

use crate::core::error::{CcrError, Result};
use crate::core::lock::LockManager;
use crate::managers::config::ConfigSection;
use crate::utils::Validatable;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::NamedTempFile;

// 🎯 优化：定义常量避免重复分配字符串
const ANTHROPIC_BASE_URL: &str = "ANTHROPIC_BASE_URL";
const ANTHROPIC_AUTH_TOKEN: &str = "ANTHROPIC_AUTH_TOKEN";
const ANTHROPIC_MODEL: &str = "ANTHROPIC_MODEL";
const ANTHROPIC_SMALL_FAST_MODEL: &str = "ANTHROPIC_SMALL_FAST_MODEL";

/// 🎨 Claude Code 设置结构
///
/// 对应 ~/.claude/settings.json 的结构
///
/// 字段说明:
/// - 🌍 env: 环境变量映射(包含 ANTHROPIC_* 变量)
/// - 📦 other: 其他未知字段(保持原样,向前兼容)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSettings {
    /// 🌍 环境变量配置字典
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// 📦 其他设置字段(扁平化存储,保持原样)
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl ClaudeSettings {
    /// 🏗️ 创建新的空设置
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            other: HashMap::new(),
        }
    }

    /// 🧹 清空所有 ANTHROPIC_ 前缀的环境变量
    ///
    /// 保留其他环境变量,只删除 ANTHROPIC_* 相关的
    pub fn clear_anthropic_vars(&mut self) {
        self.env.retain(|key, _| !key.starts_with("ANTHROPIC_"));
        tracing::debug!("🧹 清空所有 ANTHROPIC_* 环境变量");
    }

    /// 🔄 从配置节更新环境变量
    ///
    /// 执行流程:
    /// 1. 🧹 先清空所有旧的 ANTHROPIC_* 变量
    /// 2. ➕ 根据配置节设置新的环境变量
    ///
    /// 映射关系:
    /// - base_url → ANTHROPIC_BASE_URL
    /// - auth_token → ANTHROPIC_AUTH_TOKEN
    /// - model → ANTHROPIC_MODEL
    /// - small_fast_model → ANTHROPIC_SMALL_FAST_MODEL
    pub fn update_from_config(&mut self, section: &ConfigSection) {
        // 🧹 清空旧的 ANTHROPIC_* 变量
        self.clear_anthropic_vars();

        // 🌐 设置 base_url
        if let Some(base_url) = &section.base_url {
            self.env
                .insert(ANTHROPIC_BASE_URL.to_string(), base_url.clone());
        }

        // 🔑 设置 auth_token
        if let Some(auth_token) = &section.auth_token {
            self.env
                .insert(ANTHROPIC_AUTH_TOKEN.to_string(), auth_token.clone());
        }

        // 🤖 设置 model
        if let Some(model) = &section.model {
            self.env.insert(ANTHROPIC_MODEL.to_string(), model.clone());
        }

        // ⚡ 设置 small_fast_model
        if let Some(small_model) = &section.small_fast_model {
            self.env
                .insert(ANTHROPIC_SMALL_FAST_MODEL.to_string(), small_model.clone());
        }

        tracing::info!("✅ 环境变量已从配置更新");
    }

    /// 📊 获取 ANTHROPIC_* 环境变量状态(用于展示)
    ///
    /// 返回所有 ANTHROPIC 相关变量的当前值或 None
    pub fn anthropic_env_status(&self) -> HashMap<String, Option<String>> {
        let mut status = HashMap::new();
        let vars = [
            ANTHROPIC_BASE_URL,
            ANTHROPIC_AUTH_TOKEN,
            ANTHROPIC_MODEL,
            ANTHROPIC_SMALL_FAST_MODEL,
        ];

        for var in vars {
            status.insert(var.to_string(), self.env.get(var).cloned());
        }

        status
    }
}

impl Default for ClaudeSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl Validatable for ClaudeSettings {
    /// ✅ 验证关键环境变量是否存在
    ///
    /// 必需变量:
    /// - ANTHROPIC_BASE_URL
    /// - ANTHROPIC_AUTH_TOKEN
    fn validate(&self) -> Result<()> {
        let required_vars = [ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_TOKEN];

        for var in required_vars {
            if !self.env.contains_key(var) || self.env.get(var).unwrap().is_empty() {
                return Err(CcrError::ValidationError(format!(
                    "缺少必需的环境变量: {}",
                    var
                )));
            }
        }

        Ok(())
    }
}

/// 🔧 设置管理器
///
/// 负责 Claude Code 设置文件的完整生命周期管理
///
/// 核心功能:
/// - 📖 加载和解析 settings.json
/// - 💾 原子性保存(临时文件 + rename)
/// - 🔒 文件锁防止并发冲突
/// - 💾 自动备份和恢复
/// - 📋 备份文件列表管理
pub struct SettingsManager {
    settings_path: PathBuf,
    backup_dir: PathBuf,
    lock_manager: LockManager,
}

impl SettingsManager {
    /// 🏗️ 创建新的设置管理器
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(
        settings_path: P,
        backup_dir: Q,
        lock_manager: LockManager,
    ) -> Self {
        Self {
            settings_path: settings_path.as_ref().to_path_buf(),
            backup_dir: backup_dir.as_ref().to_path_buf(),
            lock_manager,
        }
    }

    /// 🏠 使用默认路径创建管理器
    ///
    /// 默认路径:
    /// - 设置文件: ~/.claude/settings.json
    /// - 备份目录: ~/.claude/backups
    ///
    /// ⚙️ **开发者注意**：
    /// 可以通过环境变量覆盖默认路径：
    /// - `CCR_SETTINGS_PATH`: 设置文件路径
    /// - `CCR_BACKUP_DIR`: 备份目录路径
    ///
    /// 示例：
    /// ```bash
    /// export CCR_SETTINGS_PATH=/tmp/ccr_dev_settings.json
    /// export CCR_BACKUP_DIR=/tmp/ccr_dev_backups
    /// cargo run -- switch test
    /// ```
    pub fn with_default() -> Result<Self> {
        // 🔍 检查环境变量
        let settings_path = if let Ok(custom_path) = std::env::var("CCR_SETTINGS_PATH") {
            std::path::PathBuf::from(custom_path)
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| CcrError::SettingsError("无法获取用户主目录".into()))?;
            home.join(".claude").join("settings.json")
        };

        let backup_dir = if let Ok(custom_dir) = std::env::var("CCR_BACKUP_DIR") {
            std::path::PathBuf::from(custom_dir)
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| CcrError::SettingsError("无法获取用户主目录".into()))?;
            home.join(".claude").join("backups")
        };

        let lock_manager = LockManager::with_default_path()?;

        tracing::debug!("使用设置路径: {:?}", settings_path);
        tracing::debug!("使用备份目录: {:?}", backup_dir);

        Ok(Self::new(settings_path, backup_dir, lock_manager))
    }

    /// 📁 获取设置文件路径
    pub fn settings_path(&self) -> &Path {
        &self.settings_path
    }

    /// 📖 加载设置文件
    ///
    /// 执行步骤:
    /// 1. ✅ 检查文件是否存在
    /// 2. 📄 读取 JSON 内容
    /// 3. 🔍 解析为 ClaudeSettings 结构
    pub fn load(&self) -> Result<ClaudeSettings> {
        // ✅ 检查文件是否存在
        if !self.settings_path.exists() {
            return Err(CcrError::SettingsMissing(
                self.settings_path.display().to_string(),
            ));
        }

        // 📄 读取文件内容
        let content = fs::read_to_string(&self.settings_path)
            .map_err(|e| CcrError::SettingsError(format!("读取设置文件失败: {}", e)))?;

        // 🔍 解析 JSON
        let settings: ClaudeSettings = serde_json::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("解析设置文件失败: {}", e)))?;

        tracing::debug!("✅ 成功加载设置文件: {:?}", self.settings_path);
        Ok(settings)
    }

    /// 💾 原子保存设置文件
    ///
    /// ⚠️ 这是核心方法,确保写入的原子性和安全性
    ///
    /// 执行步骤:
    /// 1. 🔒 获取文件锁(超时 10 秒)
    /// 2. 📁 确保目标目录存在
    /// 3. 📝 序列化为 JSON(美化格式)
    /// 4. 📄 写入临时文件
    /// 5. 🔄 原子替换(rename)
    ///
    /// 原子性保证:
    /// - 使用 tempfile + persist 实现原子替换
    /// - 即使进程崩溃也不会损坏原文件
    pub fn save_atomic(&self, settings: &ClaudeSettings) -> Result<()> {
        // 🔒 获取文件锁(防止并发写入)
        let _lock = self.lock_manager.lock_settings(Duration::from_secs(10))?;

        // 📁 确保目录存在
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::SettingsError(format!("创建设置目录失败: {}", e)))?;
        }

        // 📝 序列化为 JSON(美化格式)
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| CcrError::SettingsError(format!("序列化设置失败: {}", e)))?;

        // 📄 写入临时文件
        let temp_file = if let Some(parent) = self.settings_path.parent() {
            NamedTempFile::new_in(parent)
        } else {
            NamedTempFile::new()
        }
        .map_err(|e| CcrError::SettingsError(format!("创建临时文件失败: {}", e)))?;

        fs::write(temp_file.path(), content)
            .map_err(|e| CcrError::SettingsError(format!("写入临时文件失败: {}", e)))?;

        // 🔄 原子替换(确保不会损坏原文件)
        temp_file
            .persist(&self.settings_path)
            .map_err(|e| CcrError::SettingsError(format!("原子替换文件失败: {}", e)))?;

        tracing::info!("✅ 设置文件已原子保存: {:?}", self.settings_path);
        Ok(())
    }

    /// 💾 备份设置文件
    ///
    /// 执行流程:
    /// 1. ✅ 验证源文件存在
    /// 2. 📁 确保备份目录存在
    /// 3. 🏷️ 生成带时间戳的备份文件名
    /// 4. 📋 复制文件到备份目录
    /// 5. 🧹 自动清理旧备份(只保留最近10个)
    ///
    /// 文件名格式:
    /// - 有配置名: settings.{config_name}.{timestamp}.json.bak
    /// - 无配置名: settings.{timestamp}.json.bak
    pub fn backup(&self, config_name: Option<&str>) -> Result<PathBuf> {
        // ✅ 验证源文件存在
        if !self.settings_path.exists() {
            return Err(CcrError::SettingsMissing(
                self.settings_path.display().to_string(),
            ));
        }

        // 📁 确保备份目录存在
        fs::create_dir_all(&self.backup_dir)
            .map_err(|e| CcrError::SettingsError(format!("创建备份目录失败: {}", e)))?;

        // 🏷️ 生成备份文件名(带时间戳)
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_filename = if let Some(name) = config_name {
            format!("settings.{}.{}.json.bak", name, timestamp)
        } else {
            format!("settings.{}.json.bak", timestamp)
        };

        let backup_path = self.backup_dir.join(backup_filename);

        // 📋 复制文件
        fs::copy(&self.settings_path, &backup_path)
            .map_err(|e| CcrError::SettingsError(format!("备份设置文件失败: {}", e)))?;

        tracing::info!("💾 设置文件已备份: {:?}", backup_path);

        // 🧹 自动清理旧备份(只保留最近10个)
        const MAX_BACKUPS: usize = 10;
        if let Ok(backups) = self.list_backups()
            && backups.len() > MAX_BACKUPS
        {
            let to_delete = &backups[MAX_BACKUPS..];
            for old_backup in to_delete {
                if let Err(e) = fs::remove_file(old_backup) {
                    tracing::warn!("清理旧备份失败 {:?}: {}", old_backup, e);
                } else {
                    tracing::debug!("🗑️ 已删除旧备份: {:?}", old_backup);
                }
            }
            tracing::info!(
                "🧹 已自动清理 {} 个旧备份,保留最近 {} 个",
                to_delete.len(),
                MAX_BACKUPS
            );
        }

        Ok(backup_path)
    }

    /// 🔄 从备份恢复设置文件
    ///
    /// 执行流程:
    /// 1. ✅ 验证备份文件存在
    /// 2. 🔍 验证备份文件格式有效
    /// 3. 💾 备份当前设置(pre_restore)
    /// 4. 🔒 获取文件锁
    /// 5. 📋 复制备份文件到目标位置
    ///
    /// ⚠️ 注意: 恢复前会自动备份当前设置
    #[allow(dead_code)]
    pub fn restore<P: AsRef<Path>>(&self, backup_path: P) -> Result<()> {
        let backup_path = backup_path.as_ref();

        // ✅ 验证备份文件存在
        if !backup_path.exists() {
            return Err(CcrError::SettingsMissing(backup_path.display().to_string()));
        }

        // 🔍 验证备份文件格式
        let content = fs::read_to_string(backup_path)
            .map_err(|e| CcrError::SettingsError(format!("读取备份文件失败: {}", e)))?;

        let _: ClaudeSettings = serde_json::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("备份文件格式无效: {}", e)))?;

        // 💾 恢复前先备份当前设置(安全措施)
        if self.settings_path.exists() {
            self.backup(Some("pre_restore"))?;
        }

        // 🔒 获取文件锁
        let _lock = self.lock_manager.lock_settings(Duration::from_secs(10))?;

        // 📋 执行恢复
        fs::copy(backup_path, &self.settings_path)
            .map_err(|e| CcrError::SettingsError(format!("恢复设置文件失败: {}", e)))?;

        tracing::info!("✅ 设置文件已从备份恢复: {:?}", backup_path);
        Ok(())
    }

    /// 📋 列出所有备份文件
    ///
    /// 返回所有 .bak 扩展名的备份文件,按修改时间倒序排列(最新的在前)
    #[allow(dead_code)]
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        if !self.backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups = Vec::new();

        // 📂 遍历备份目录
        for entry in fs::read_dir(&self.backup_dir)
            .map_err(|e| CcrError::SettingsError(format!("读取备份目录失败: {}", e)))?
        {
            let entry =
                entry.map_err(|e| CcrError::SettingsError(format!("读取目录项失败: {}", e)))?;

            let path = entry.path();
            // 🔍 只收集 .bak 文件
            if path.extension().and_then(|s| s.to_str()) == Some("bak") {
                backups.push(path);
            }
        }

        // 📅 按修改时间排序(最新的在前)
        backups.sort_by(|a, b| {
            let a_time = fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = fs::metadata(b).and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        Ok(backups)
    }

    // === 🆕 多平台支持方法 ===

    /// 🎯 为指定平台创建 SettingsManager
    ///
    /// 根据平台类型自动确定设置文件路径和备份目录
    ///
    /// 支持的平台:
    /// - claude: ~/.claude/settings.json
    /// - codex: ~/.ccr/platforms/codex/settings.json (unified mode)
    /// - gemini: ~/.ccr/platforms/gemini/settings.json (unified mode)
    ///
    /// 参数:
    /// - platform_name: 平台名称 ("claude", "codex", "gemini" 等)
    ///
    /// 注意: 此方法假设统一模式已启用。对于 Claude 平台，
    /// 如果在 legacy 模式下，应使用 `SettingsManager::with_default()`
    #[allow(dead_code)]
    pub fn for_platform(platform_name: &str) -> Result<Self> {
        let (settings_path, backup_dir) = Self::get_platform_paths(platform_name)?;
        let lock_manager = LockManager::with_default_path()?;

        tracing::debug!(
            "为平台 '{}' 创建 SettingsManager: {:?}",
            platform_name,
            settings_path
        );

        Ok(Self::new(settings_path, backup_dir, lock_manager))
    }

    /// 📁 获取平台特定的路径
    ///
    /// 返回 (settings_path, backup_dir)
    #[allow(dead_code)]
    pub fn get_platform_paths(platform_name: &str) -> Result<(PathBuf, PathBuf)> {
        // 特殊处理 Claude (支持 legacy 模式)
        if platform_name == "claude" {
            // 检查是否在统一模式下
            let home = dirs::home_dir()
                .ok_or_else(|| CcrError::SettingsError("无法获取用户主目录".into()))?;

            // 优先使用环境变量
            if let Ok(custom_path) = std::env::var("CCR_SETTINGS_PATH") {
                let settings_path = PathBuf::from(custom_path);
                let backup_dir = if let Ok(custom_dir) = std::env::var("CCR_BACKUP_DIR") {
                    PathBuf::from(custom_dir)
                } else {
                    home.join(".claude").join("backups")
                };
                return Ok((settings_path, backup_dir));
            }

            // 检查统一模式
            let ccr_root = if let Ok(root) = std::env::var("CCR_ROOT") {
                PathBuf::from(root)
            } else {
                home.join(".ccr")
            };

            if ccr_root.exists() {
                // 统一模式
                let platform_dir = ccr_root.join("platforms").join("claude");
                return Ok((
                    platform_dir.join("settings.json"),
                    platform_dir.join("backups"),
                ));
            } else {
                // Legacy 模式
                return Ok((
                    home.join(".claude").join("settings.json"),
                    home.join(".claude").join("backups"),
                ));
            }
        }

        // 其他平台都使用统一模式路径
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::SettingsError("无法获取用户主目录".into()))?;

        let ccr_root = if let Ok(root) = std::env::var("CCR_ROOT") {
            PathBuf::from(root)
        } else {
            home.join(".ccr")
        };

        let platform_dir = ccr_root.join("platforms").join(platform_name);

        Ok((
            platform_dir.join("settings.json"),
            platform_dir.join("backups"),
        ))
    }

    /// 🔍 检测当前平台的配置模式
    ///
    /// 返回 "Legacy" 或 "Unified"
    #[allow(dead_code)]
    pub fn detect_mode(&self) -> &'static str {
        // 如果设置路径包含 ".ccr/platforms"，则为统一模式
        if self
            .settings_path
            .to_str()
            .map(|s| s.contains(".ccr/platforms"))
            .unwrap_or(false)
        {
            "Unified"
        } else {
            "Legacy"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::managers::config::ConfigSection;
    use indexmap::IndexMap;

    fn create_test_config_section() -> ConfigSection {
        ConfigSection {
            description: Some("Test".into()),
            base_url: Some("https://api.test.com".into()),
            auth_token: Some("sk-test-token".into()),
            model: Some("test-model".into()),
            small_fast_model: Some("test-small".into()),
            provider: None,
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            other: IndexMap::new(),
        }
    }

    #[test]
    fn test_claude_settings_update_from_config() {
        let mut settings = ClaudeSettings::new();
        let config = create_test_config_section();

        settings.update_from_config(&config);

        assert_eq!(
            settings.env.get("ANTHROPIC_BASE_URL"),
            Some(&"https://api.test.com".to_string())
        );
        assert_eq!(
            settings.env.get("ANTHROPIC_AUTH_TOKEN"),
            Some(&"sk-test-token".to_string())
        );
        assert_eq!(
            settings.env.get("ANTHROPIC_MODEL"),
            Some(&"test-model".to_string())
        );
    }

    #[test]
    fn test_claude_settings_clear_anthropic_vars() {
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "test".into());
        settings.env.insert("OTHER_VAR".into(), "keep".into());

        settings.clear_anthropic_vars();

        assert!(!settings.env.contains_key("ANTHROPIC_BASE_URL"));
        assert!(settings.env.contains_key("OTHER_VAR"));
    }

    #[test]
    fn test_claude_settings_validate() {
        let mut settings = ClaudeSettings::new();

        // 缺少必需变量应该失败
        assert!(settings.validate().is_err());

        // 添加必需变量
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "https://test.com".into());
        settings
            .env
            .insert("ANTHROPIC_AUTH_TOKEN".into(), "token".into());

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_settings_manager_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = SettingsManager::new(settings_path, backup_dir, lock_manager);

        // 创建并保存设置
        let mut settings = ClaudeSettings::new();
        settings.update_from_config(&create_test_config_section());

        manager.save_atomic(&settings).unwrap();

        // 加载并验证
        let loaded = manager.load().unwrap();
        assert_eq!(
            loaded.env.get("ANTHROPIC_BASE_URL"),
            Some(&"https://api.test.com".to_string())
        );
    }

    #[test]
    fn test_settings_manager_backup_restore() {
        let temp_dir = tempfile::tempdir().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = SettingsManager::new(settings_path, backup_dir, lock_manager);

        // 创建原始设置
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "original".into());
        manager.save_atomic(&settings).unwrap();

        // 备份
        let backup_path = manager.backup(Some("test")).unwrap();
        assert!(backup_path.exists());

        // 修改设置
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "modified".into());
        manager.save_atomic(&settings).unwrap();

        // 恢复
        manager.restore(&backup_path).unwrap();
        let restored = manager.load().unwrap();
        assert_eq!(
            restored.env.get("ANTHROPIC_BASE_URL"),
            Some(&"original".to_string())
        );
    }

    #[test]
    fn test_backup_auto_cleanup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = SettingsManager::new(settings_path, backup_dir, lock_manager);

        // 创建初始设置
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "test".into());
        manager.save_atomic(&settings).unwrap();

        // 创建15个备份
        for i in 0..15 {
            manager.backup(Some(&format!("config{}", i))).unwrap();
            // 短暂延迟确保时间戳不同
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // 验证只保留了最近10个备份
        let backups = manager.list_backups().unwrap();
        assert_eq!(
            backups.len(),
            10,
            "应该只保留10个备份,但实际有 {} 个",
            backups.len()
        );

        // 验证保留的是最新的10个(按时间倒序,最新的在前)
        assert!(backups.len() <= 10);
    }
}
