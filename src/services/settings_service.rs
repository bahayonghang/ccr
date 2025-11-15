// 📝 设置服务
// 封装 Claude Code 设置相关的业务逻辑

use crate::core::error::Result;
use crate::managers::config::ConfigSection;
use crate::managers::settings::{ClaudeSettings, SettingsManager};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 📦 备份信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BackupInfo {
    pub filename: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub created_at: std::time::SystemTime,
}

/// 📝 设置服务
///
/// 封装所有 Claude Code 设置相关的业务逻辑
pub struct SettingsService {
    settings_manager: Arc<SettingsManager>,
}

#[allow(dead_code)]
impl SettingsService {
    /// 🏗️ 创建新的设置服务
    pub fn new(settings_manager: Arc<SettingsManager>) -> Self {
        Self { settings_manager }
    }

    /// 🏠 使用默认设置管理器创建服务
    pub fn with_default() -> Result<Self> {
        let settings_manager = Arc::new(SettingsManager::with_default()?);
        Ok(Self::new(settings_manager))
    }

    /// 📖 获取当前设置
    pub fn get_current_settings(&self) -> Result<ClaudeSettings> {
        self.settings_manager.load()
    }

    /// 🔄 应用配置到设置
    ///
    /// 从配置节更新 settings.json 中的环境变量
    ///
    /// # Arguments
    /// - `section` - 配置节
    ///
    /// # Process
    /// 1. 加载当前设置(或创建新设置)
    /// 2. 清空旧的 ANTHROPIC_* 变量
    /// 3. 从配置节设置新的环境变量
    /// 4. 原子保存设置文件
    pub fn apply_config(&self, section: &ConfigSection) -> Result<()> {
        let mut settings = self
            .settings_manager
            .load()
            .unwrap_or_else(|_| ClaudeSettings::new());

        settings.update_from_config(section);
        self.settings_manager.save_atomic(&settings)?;

        Ok(())
    }

    /// 💾 备份当前设置
    ///
    /// # Arguments
    /// - `name` - 备份名称(可选,会加入到文件名中)
    ///
    /// # Returns
    /// 备份文件的路径
    pub fn backup_settings(&self, name: Option<&str>) -> Result<PathBuf> {
        self.settings_manager.backup(name)
    }

    /// 🔄 从备份恢复设置
    ///
    /// # Arguments
    /// - `backup_path` - 备份文件路径
    ///
    /// # Process
    /// 1. 验证备份文件存在且格式有效
    /// 2. 备份当前设置(pre_restore)
    /// 3. 从备份恢复
    pub fn restore_settings(&self, backup_path: &Path) -> Result<()> {
        self.settings_manager.restore(backup_path)
    }

    /// 📋 列出所有备份
    ///
    /// # Returns
    /// 备份信息列表,按修改时间倒序排列(最新的在前)
    pub fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        let backup_paths = self.settings_manager.list_backups()?;

        let mut backups = Vec::new();
        for path in backup_paths {
            if let Ok(metadata) = std::fs::metadata(&path)
                && let Ok(modified) = metadata.modified()
            {
                backups.push(BackupInfo {
                    filename: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    path: path.clone(),
                    size_bytes: metadata.len(),
                    created_at: modified,
                });
            }
        }

        Ok(backups)
    }

    /// 📁 获取设置管理器
    pub fn settings_manager(&self) -> &Arc<SettingsManager> {
        &self.settings_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::lock::LockManager;
    use crate::managers::config::ConfigSection;
    use tempfile::tempdir;

    fn create_test_section() -> ConfigSection {
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
        }
    }

    #[test]
    fn test_settings_service_apply_config() {
        let temp_dir = tempdir().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = Arc::new(SettingsManager::new(
            settings_path,
            backup_dir,
            lock_manager,
        ));
        let service = SettingsService::new(manager);

        // 应用配置
        let section = create_test_section();
        service.apply_config(&section).unwrap();

        // 验证设置
        let settings = service.get_current_settings().unwrap();
        assert_eq!(
            settings.env.get("ANTHROPIC_BASE_URL"),
            Some(&"https://api.test.com".to_string())
        );
        assert_eq!(
            settings.env.get("ANTHROPIC_AUTH_TOKEN"),
            Some(&"sk-test-token".to_string())
        );
    }
}
