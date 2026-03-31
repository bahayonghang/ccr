// 📁 配置文件处理器
// 负责配置文件的 I/O 操作
//
// 🎯 设计目的:
// - 从 ConfigManager 中分离文件 I/O 逻辑
// - 提供统一的文件读写接口
// - 支持配置文件的备份和恢复

use crate::managers::config::CcsConfig;
use ccr_core::AutoCompletable;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::fileio;
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

/// 📁 配置文件处理器
///
/// 封装配置文件的所有文件系统操作
///
/// # 职责
/// - 📖 加载配置文件
/// - 💾 保存配置文件
/// - 🔄 备份配置文件
/// - 📋 管理备份历史
pub struct ConfigFileHandler {
    config_path: PathBuf,
}

impl ConfigFileHandler {
    /// 🏗️ 创建新的配置文件处理器
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    /// 📁 获取配置文件路径
    #[allow(dead_code)]
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// 📖 加载配置文件
    ///
    /// 执行步骤:
    /// 1. ✅ 检查文件是否存在
    /// 2. 📄 读取文件内容
    /// 3. 🔍 解析 TOML 格式
    ///
    /// ⚠️ **并发安全**: 此方法不加锁，调用方需要在外层使用 CONFIG_LOCK 保护 RMW 序列
    ///
    /// 注意: 此方法为纯读取，不会自动修复或写回文件。
    /// 如需自动补全并保存，请使用 `load_with_autofix`。
    pub fn load(&self) -> Result<CcsConfig> {
        // ✅ 检查文件是否存在
        if !self.config_path.exists() {
            return Err(CcrError::ConfigMissing(
                self.config_path.display().to_string(),
            ));
        }

        // 使用统一的 fileio 读取 TOML
        let config: CcsConfig = fileio::read_toml(&self.config_path)?;

        tracing::debug!(
            "✅ 成功加载配置文件: {:?}, 配置节数量: {}",
            self.config_path,
            config.sections.len()
        );

        Ok(config)
    }

    /// 🔄 加载配置并自动补全缺失字段（必要时写回）
    ///
    /// 执行步骤:
    /// 1. 📖 调用 `load` 读取配置
    /// 2. 🔍 自动补全缺失字段
    /// 3. 💾 如果有变更则保存
    ///
    /// ⚠️ **并发安全**: 此方法不加锁，调用方需要在外层使用 CONFIG_LOCK 保护 RMW 序列
    pub fn load_with_autofix(&self) -> Result<CcsConfig> {
        let mut config = self.load()?;

        // 🔄 自动补全缺失字段
        let mut modified = false;
        for (name, section) in &mut config.sections {
            if section.auto_complete() {
                tracing::debug!("🔄 自动补全配置节 '{}' 的缺失字段", name);
                modified = true;
            }
        }

        // 💾 如果有字段被自动补全，保存配置
        if modified {
            tracing::info!("💾 检测到缺失字段已自动补全，保存配置文件");
            self.save(&config)?;
        }

        Ok(config)
    }

    /// 💾 保存配置文件
    ///
    /// 执行步骤:
    /// 1. 📝 序列化为 TOML 格式
    /// 2. 💾 写入磁盘
    ///
    /// ⚠️ **并发安全**: 此方法不加锁，调用方需要在外层使用 CONFIG_LOCK 保护 RMW 序列
    pub fn save(&self, config: &CcsConfig) -> Result<()> {
        // 使用统一的 fileio 写入 TOML
        fileio::write_toml(&self.config_path, config)?;

        tracing::debug!("✅ 配置文件已保存: {:?}", self.config_path);
        Ok(())
    }

    /// 💾 备份配置文件
    ///
    /// 执行流程:
    /// 1. ✅ 验证源文件存在
    /// 2. 🏷️ 生成带时间戳的备份文件名
    /// 3. 📋 复制文件到备份位置
    /// 4. 🧹 自动清理旧备份(只保留最近10个)
    ///
    /// 文件名格式:
    /// - 有标签: .ccs_config.toml.{tag}_{timestamp}.bak
    /// - 无标签: .ccs_config.toml.{timestamp}.bak
    ///
    /// 备份位置: 与配置文件同目录
    pub fn backup(&self, tag: Option<&str>) -> Result<PathBuf> {
        // ✅ 验证源文件存在
        if !self.config_path.exists() {
            return Err(CcrError::ConfigMissing(
                self.config_path.display().to_string(),
            ));
        }

        // 🏷️ 生成备份文件名(带时间戳)
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = if let Some(tag_str) = tag {
            self.config_path
                .with_extension(format!("toml.{}_{}.bak", tag_str, timestamp))
        } else {
            self.config_path
                .with_extension(format!("toml.{}.bak", timestamp))
        };

        // 📋 复制文件
        fs::copy(&self.config_path, &backup_path)
            .map_err(|e| CcrError::ConfigError(format!("备份配置文件失败: {}", e)))?;

        tracing::info!("💾 配置文件已备份: {:?}", backup_path);

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
                "🧹 已自动清理 {} 个旧配置备份,保留最近 {} 个",
                to_delete.len(),
                MAX_BACKUPS
            );
        }

        Ok(backup_path)
    }

    /// 📋 列出所有配置备份文件
    ///
    /// 返回所有配置文件的 .bak 备份,按修改时间倒序排列(最新的在前)
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let config_dir = self
            .config_path
            .parent()
            .ok_or_else(|| CcrError::ConfigError("无法获取配置目录".into()))?;

        if !config_dir.exists() {
            return Ok(vec![]);
        }

        let config_filename = self
            .config_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| CcrError::ConfigError("无效的配置文件名".into()))?;

        let mut backups = Vec::new();

        // 📂 遍历配置目录
        for entry in fs::read_dir(config_dir)
            .map_err(|e| CcrError::ConfigError(format!("读取配置目录失败: {}", e)))?
        {
            let entry =
                entry.map_err(|e| CcrError::ConfigError(format!("读取目录项失败: {}", e)))?;

            let path = entry.path();
            let filename = path.file_name().and_then(|n| n.to_str());

            // 🔍 只收集配置文件的 .bak 文件
            // 例如: .ccs_config.toml.20240101_120000.bak
            if let Some(name) = filename
                && path.is_file()
                && name.starts_with(config_filename)
                && name.ends_with(".bak")
            {
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
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::managers::config::{ConfigSection, GlobalSettings};
    use indexmap::IndexMap;

    fn create_test_section() -> ConfigSection {
        ConfigSection {
            description: Some("Test config".into()),
            base_url: Some("https://api.test.com".into()),
            auth_token: Some("sk-test-token".into()),
            model: Some("test-model".into()),
            small_fast_model: Some("test-small-model".into()),
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
    fn test_file_handler_load_save() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // 创建测试配置
        let mut config = CcsConfig {
            default_config: "test".into(),
            current_config: "test".into(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };
        config.set_section("test".into(), create_test_section());

        // 保存
        let handler = ConfigFileHandler::new(&config_path);
        handler.save(&config).unwrap();
        assert!(config_path.exists());

        // 加载
        let loaded = handler.load().unwrap();
        assert_eq!(loaded.default_config, "test");
        assert!(loaded.get_section("test").is_ok());
    }

    #[test]
    fn test_file_handler_backup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        // 创建测试配置
        let mut config = CcsConfig {
            default_config: "test".into(),
            current_config: "test".into(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };
        config.set_section("test".into(), create_test_section());

        let handler = ConfigFileHandler::new(&config_path);
        handler.save(&config).unwrap();

        // 测试备份
        let backup_path = handler.backup(Some("test")).unwrap();
        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains("test_"));

        // 测试列出备份
        let backups = handler.list_backups().unwrap();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn test_file_handler_backup_auto_cleanup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        // 创建测试配置
        let mut config = CcsConfig {
            default_config: "test".into(),
            current_config: "test".into(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };
        config.set_section("test".into(), create_test_section());

        let handler = ConfigFileHandler::new(&config_path);
        handler.save(&config).unwrap();

        // 创建15个备份
        for i in 0..15 {
            handler.backup(Some(&format!("tag{}", i))).unwrap();
            // 短暂延迟确保时间戳不同
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // 验证只保留了最近10个备份
        let backups = handler.list_backups().unwrap();
        assert_eq!(
            backups.len(),
            10,
            "应该只保留10个配置备份,但实际有 {} 个",
            backups.len()
        );
    }
}
