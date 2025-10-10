// CCR 设置管理模块
// 负责读写和管理 ~/.claude/settings.json 文件
// 这是 CCR 的核心模块,直接操作 Claude Code 的配置文件

use crate::config::ConfigSection;
use crate::error::{CcrError, Result};
use crate::lock::LockManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::NamedTempFile;

/// Claude Code 设置结构
///
/// 对应 ~/.claude/settings.json 的结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSettings {
    /// 环境变量配置
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// 其他设置字段(扁平化存储,保持原样)
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl ClaudeSettings {
    /// 创建新的设置
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            other: HashMap::new(),
        }
    }

    /// 清空所有 ANTHROPIC_ 前缀的环境变量
    pub fn clear_anthropic_vars(&mut self) {
        self.env
            .retain(|key, _| !key.starts_with("ANTHROPIC_"));
        log::debug!("清空所有 ANTHROPIC_* 环境变量");
    }

    /// 从配置节更新环境变量
    ///
    /// 先清空所有 ANTHROPIC_* 变量，然后设置新的值
    pub fn update_from_config(&mut self, section: &ConfigSection) {
        // 清空旧的 ANTHROPIC_* 变量
        self.clear_anthropic_vars();

        // 设置 base_url（可选）
        if let Some(base_url) = &section.base_url {
            self.env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                base_url.clone(),
            );
        }

        // 设置 auth_token（可选）
        if let Some(auth_token) = &section.auth_token {
            self.env.insert(
                "ANTHROPIC_AUTH_TOKEN".to_string(),
                auth_token.clone(),
            );
        }

        // 设置 model（可选）
        if let Some(model) = &section.model {
            self.env
                .insert("ANTHROPIC_MODEL".to_string(), model.clone());
        }

        if let Some(small_model) = &section.small_fast_model {
            self.env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                small_model.clone(),
            );
        }

        log::info!("环境变量已从配置更新");
    }

    /// 验证关键环境变量是否存在
    pub fn validate(&self) -> Result<()> {
        let required_vars = vec!["ANTHROPIC_BASE_URL", "ANTHROPIC_AUTH_TOKEN"];

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

    /// 获取 ANTHROPIC_* 环境变量状态（用于展示）
    pub fn anthropic_env_status(&self) -> HashMap<String, Option<String>> {
        let mut status = HashMap::new();
        let vars = vec![
            "ANTHROPIC_BASE_URL",
            "ANTHROPIC_AUTH_TOKEN",
            "ANTHROPIC_MODEL",
            "ANTHROPIC_SMALL_FAST_MODEL",
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

/// 设置管理器
///
/// 负责加载、保存、备份和恢复 Claude Code 设置文件
pub struct SettingsManager {
    settings_path: PathBuf,
    backup_dir: PathBuf,
    lock_manager: LockManager,
}

impl SettingsManager {
    /// 创建新的设置管理器
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

    /// 使用默认路径创建管理器
    pub fn default() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::SettingsError("无法获取用户主目录".into()))?;

        let settings_path = home.join(".claude").join("settings.json");
        let backup_dir = home.join(".claude").join("backups");
        let lock_manager = LockManager::default()?;

        Ok(Self::new(settings_path, backup_dir, lock_manager))
    }

    /// 获取设置文件路径
    pub fn settings_path(&self) -> &Path {
        &self.settings_path
    }

    /// 加载设置文件
    pub fn load(&self) -> Result<ClaudeSettings> {
        if !self.settings_path.exists() {
            return Err(CcrError::SettingsMissing(
                self.settings_path.display().to_string(),
            ));
        }

        let content = fs::read_to_string(&self.settings_path).map_err(|e| {
            CcrError::SettingsError(format!("读取设置文件失败: {}", e))
        })?;

        let settings: ClaudeSettings = serde_json::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("解析设置文件失败: {}", e)))?;

        log::debug!("成功加载设置文件: {:?}", self.settings_path);
        Ok(settings)
    }

    /// 原子保存设置文件
    ///
    /// 使用文件锁和临时文件确保写入的原子性和安全性
    pub fn save_atomic(&self, settings: &ClaudeSettings) -> Result<()> {
        // 获取文件锁
        let _lock = self
            .lock_manager
            .lock_settings(Duration::from_secs(10))?;

        // 确保目录存在
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CcrError::SettingsError(format!("创建设置目录失败: {}", e))
            })?;
        }

        // 序列化为 JSON (美化格式)
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| CcrError::SettingsError(format!("序列化设置失败: {}", e)))?;

        // 写入临时文件
        let temp_file = if let Some(parent) = self.settings_path.parent() {
            NamedTempFile::new_in(parent)
        } else {
            NamedTempFile::new()
        }
        .map_err(|e| CcrError::SettingsError(format!("创建临时文件失败: {}", e)))?;

        fs::write(temp_file.path(), content).map_err(|e| {
            CcrError::SettingsError(format!("写入临时文件失败: {}", e))
        })?;

        // 原子替换
        temp_file.persist(&self.settings_path).map_err(|e| {
            CcrError::SettingsError(format!("原子替换文件失败: {}", e))
        })?;

        log::info!("设置文件已原子保存: {:?}", self.settings_path);
        Ok(())
    }

    /// 备份设置文件
    pub fn backup(&self, config_name: Option<&str>) -> Result<PathBuf> {
        if !self.settings_path.exists() {
            return Err(CcrError::SettingsMissing(
                self.settings_path.display().to_string(),
            ));
        }

        // 确保备份目录存在
        fs::create_dir_all(&self.backup_dir).map_err(|e| {
            CcrError::SettingsError(format!("创建备份目录失败: {}", e))
        })?;

        // 生成备份文件名
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_filename = if let Some(name) = config_name {
            format!("settings.{}.{}.json.bak", name, timestamp)
        } else {
            format!("settings.{}.json.bak", timestamp)
        };

        let backup_path = self.backup_dir.join(backup_filename);

        // 复制文件
        fs::copy(&self.settings_path, &backup_path).map_err(|e| {
            CcrError::SettingsError(format!("备份设置文件失败: {}", e))
        })?;

        log::info!("设置文件已备份: {:?}", backup_path);
        Ok(backup_path)
    }

    /// 从备份恢复设置文件
    pub fn restore<P: AsRef<Path>>(&self, backup_path: P) -> Result<()> {
        let backup_path = backup_path.as_ref();

        if !backup_path.exists() {
            return Err(CcrError::SettingsMissing(
                backup_path.display().to_string(),
            ));
        }

        // 验证备份文件格式
        let content = fs::read_to_string(backup_path).map_err(|e| {
            CcrError::SettingsError(format!("读取备份文件失败: {}", e))
        })?;

        let _: ClaudeSettings = serde_json::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("备份文件格式无效: {}", e)))?;

        // 恢复前先备份当前设置
        if self.settings_path.exists() {
            self.backup(Some("pre_restore"))?;
        }

        // 获取文件锁
        let _lock = self
            .lock_manager
            .lock_settings(Duration::from_secs(10))?;

        // 执行恢复
        fs::copy(backup_path, &self.settings_path).map_err(|e| {
            CcrError::SettingsError(format!("恢复设置文件失败: {}", e))
        })?;

        log::info!("设置文件已从备份恢复: {:?}", backup_path);
        Ok(())
    }

    /// 列出所有备份文件
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        if !self.backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.backup_dir).map_err(|e| {
            CcrError::SettingsError(format!("读取备份目录失败: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                CcrError::SettingsError(format!("读取目录项失败: {}", e))
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("bak") {
                backups.push(path);
            }
        }

        // 按修改时间排序（最新的在前）
        backups.sort_by(|a, b| {
            let a_time = fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = fs::metadata(b).and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        Ok(backups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigSection;

    fn create_test_config_section() -> ConfigSection {
        ConfigSection {
            description: Some("Test".into()),
            base_url: Some("https://api.test.com".into()),
            auth_token: Some("sk-test-token".into()),
            model: Some("test-model".into()),
            small_fast_model: Some("test-small".into()),
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
        settings.env.insert("ANTHROPIC_AUTH_TOKEN".into(), "token".into());

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
}
