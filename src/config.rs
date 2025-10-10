// CCR 配置管理模块
// 负责读写和管理 ~/.ccs_config.toml 配置文件

use crate::error::{CcrError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 配置节
///
/// 代表一个具体的配置项（如 anthropic、anyrouter 等）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSection {
    /// 配置描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// API 基础 URL（可选，切换时必需）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// 认证令牌（可选，切换时必需）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,

    /// 默认模型名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// 快速小模型名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_fast_model: Option<String>,
}

impl ConfigSection {
    /// 验证配置节的完整性
    pub fn validate(&self) -> Result<()> {
        // 检查 base_url
        let base_url = self.base_url.as_ref()
            .ok_or_else(|| CcrError::ValidationError("base_url 不能为空".into()))?;

        if base_url.trim().is_empty() {
            return Err(CcrError::ValidationError("base_url 不能为空".into()));
        }

        // 简单的 URL 格式验证
        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            return Err(CcrError::ValidationError(
                "base_url 必须以 http:// 或 https:// 开头".into(),
            ));
        }

        // 检查 auth_token
        let auth_token = self.auth_token.as_ref()
            .ok_or_else(|| CcrError::ValidationError("auth_token 不能为空".into()))?;

        if auth_token.trim().is_empty() {
            return Err(CcrError::ValidationError("auth_token 不能为空".into()));
        }

        // 检查 model（可选，如果提供了则不能为空）
        if let Some(model) = &self.model {
            if model.trim().is_empty() {
                return Err(CcrError::ValidationError("model 不能为空字符串".into()));
            }
        }

        Ok(())
    }

    /// 判断配置是否完整可用
    pub fn is_complete(&self) -> bool {
        self.base_url.as_ref().map_or(false, |s| !s.trim().is_empty())
            && self.auth_token.as_ref().map_or(false, |s| !s.trim().is_empty())
    }

    /// 获取配置的人类可读描述
    pub fn display_description(&self) -> String {
        self.description
            .clone()
            .unwrap_or_else(|| "(无描述)".to_string())
    }
}

/// CCS 配置文件结构
///
/// 对应 ~/.ccs_config.toml 的完整结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcsConfig {
    /// 默认配置名称
    pub default_config: String,

    /// 当前活跃配置名称
    pub current_config: String,

    /// 所有配置节
    #[serde(flatten)]
    pub sections: HashMap<String, ConfigSection>,
}

impl CcsConfig {
    /// 创建新的配置
    pub fn new(default_config: String) -> Self {
        Self {
            current_config: default_config.clone(),
            default_config,
            sections: HashMap::new(),
        }
    }

    /// 获取指定配置节
    pub fn get_section(&self, name: &str) -> Result<&ConfigSection> {
        self.sections
            .get(name)
            .ok_or_else(|| CcrError::ConfigSectionNotFound(name.to_string()))
    }

    /// 获取当前配置节
    pub fn get_current_section(&self) -> Result<&ConfigSection> {
        self.get_section(&self.current_config)
    }

    /// 设置当前配置
    pub fn set_current(&mut self, name: &str) -> Result<()> {
        // 验证配置节存在
        if !self.sections.contains_key(name) {
            return Err(CcrError::ConfigSectionNotFound(name.to_string()));
        }
        self.current_config = name.to_string();
        Ok(())
    }

    /// 添加或更新配置节
    pub fn set_section(&mut self, name: String, section: ConfigSection) {
        self.sections.insert(name, section);
    }

    /// 删除配置节
    pub fn remove_section(&mut self, name: &str) -> Result<ConfigSection> {
        self.sections
            .remove(name)
            .ok_or_else(|| CcrError::ConfigSectionNotFound(name.to_string()))
    }

    /// 列出所有配置节名称
    pub fn list_sections(&self) -> Vec<String> {
        let mut names: Vec<String> = self.sections.keys().cloned().collect();
        names.sort();
        names
    }

    /// 验证所有配置节
    pub fn validate_all(&self) -> HashMap<String, Result<()>> {
        self.sections
            .iter()
            .map(|(name, section)| (name.clone(), section.validate()))
            .collect()
    }
}

/// 配置管理器
///
/// 负责加载、保存和管理配置文件
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    /// 使用默认配置路径创建管理器
    pub fn default() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        let config_path = home.join(".ccs_config.toml");
        Ok(Self::new(config_path))
    }

    /// 获取配置文件路径
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// 加载配置文件
    pub fn load(&self) -> Result<CcsConfig> {
        // 检查文件是否存在
        if !self.config_path.exists() {
            return Err(CcrError::ConfigMissing(
                self.config_path.display().to_string(),
            ));
        }

        // 读取文件内容
        let content = fs::read_to_string(&self.config_path).map_err(|e| {
            CcrError::ConfigError(format!("读取配置文件失败: {}", e))
        })?;

        // 解析 TOML
        let config: CcsConfig = toml::from_str(&content).map_err(|e| {
            CcrError::ConfigFormatInvalid(format!("TOML 解析失败: {}", e))
        })?;

        log::debug!(
            "成功加载配置文件: {:?}, 配置节数量: {}",
            self.config_path,
            config.sections.len()
        );

        Ok(config)
    }

    /// 保存配置文件
    pub fn save(&self, config: &CcsConfig) -> Result<()> {
        // 序列化为 TOML
        let content = toml::to_string_pretty(config).map_err(|e| {
            CcrError::ConfigError(format!("配置序列化失败: {}", e))
        })?;

        // 写入文件
        fs::write(&self.config_path, content).map_err(|e| {
            CcrError::ConfigError(format!("写入配置文件失败: {}", e))
        })?;

        log::debug!("配置文件已保存: {:?}", self.config_path);
        Ok(())
    }

    /// 更新当前配置并保存
    pub fn update_current(&self, config_name: &str) -> Result<()> {
        let mut config = self.load()?;
        config.set_current(config_name)?;
        self.save(&config)?;
        Ok(())
    }

    /// 备份配置文件
    pub fn backup(&self) -> Result<PathBuf> {
        if !self.config_path.exists() {
            return Err(CcrError::ConfigMissing(
                self.config_path.display().to_string(),
            ));
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = self
            .config_path
            .with_extension(format!("toml.{}.bak", timestamp));

        fs::copy(&self.config_path, &backup_path).map_err(|e| {
            CcrError::ConfigError(format!("备份配置文件失败: {}", e))
        })?;

        log::info!("配置文件已备份: {:?}", backup_path);
        Ok(backup_path)
    }

    /// 从备份恢复配置文件
    pub fn restore<P: AsRef<Path>>(&self, backup_path: P) -> Result<()> {
        let backup_path = backup_path.as_ref();

        if !backup_path.exists() {
            return Err(CcrError::ConfigMissing(
                backup_path.display().to_string(),
            ));
        }

        // 先验证备份文件是否有效
        let content = fs::read_to_string(backup_path).map_err(|e| {
            CcrError::ConfigError(format!("读取备份文件失败: {}", e))
        })?;

        let _: CcsConfig = toml::from_str(&content).map_err(|e| {
            CcrError::ConfigFormatInvalid(format!("备份文件格式无效: {}", e))
        })?;

        // 恢复前先备份当前配置
        if self.config_path.exists() {
            self.backup()?;
        }

        // 执行恢复
        fs::copy(backup_path, &self.config_path).map_err(|e| {
            CcrError::ConfigError(format!("恢复配置文件失败: {}", e))
        })?;

        log::info!("配置文件已从备份恢复: {:?}", backup_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_section() -> ConfigSection {
        ConfigSection {
            description: Some("Test config".into()),
            base_url: Some("https://api.test.com".into()),
            auth_token: Some("sk-test-token".into()),
            model: Some("test-model".into()),
            small_fast_model: Some("test-small-model".into()),
        }
    }

    #[test]
    fn test_config_section_validate() {
        let section = create_test_section();
        assert!(section.validate().is_ok());

        // 测试空 base_url
        let mut invalid = section.clone();
        invalid.base_url = Some("".into());
        assert!(invalid.validate().is_err());

        // 测试无效的 URL
        let mut invalid = section.clone();
        invalid.base_url = Some("not-a-url".into());
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_config_section_is_complete() {
        let section = create_test_section();
        assert!(section.is_complete());

        let mut incomplete = section.clone();
        incomplete.auth_token = None;
        assert!(!incomplete.is_complete());
    }

    #[test]
    fn test_ccs_config() {
        let mut config = CcsConfig::new("default".into());
        assert_eq!(config.default_config, "default");
        assert_eq!(config.current_config, "default");

        // 添加配置节
        config.set_section("test".into(), create_test_section());
        assert!(config.get_section("test").is_ok());
        assert!(config.get_section("nonexistent").is_err());

        // 设置当前配置
        assert!(config.set_current("test").is_ok());
        assert_eq!(config.current_config, "test");
        assert!(config.set_current("nonexistent").is_err());
    }

    #[test]
    fn test_config_manager_load_save() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // 创建测试配置
        let mut config = CcsConfig::new("test".into());
        config.set_section("test".into(), create_test_section());

        // 保存
        let manager = ConfigManager::new(&config_path);
        manager.save(&config).unwrap();
        assert!(config_path.exists());

        // 加载
        let loaded = manager.load().unwrap();
        assert_eq!(loaded.default_config, "test");
        assert!(loaded.get_section("test").is_ok());
    }

    #[test]
    fn test_config_manager_backup_restore() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // 创建原始配置
        let mut config = CcsConfig::new("original".into());
        config.set_section("test".into(), create_test_section());

        let manager = ConfigManager::new(&config_path);
        manager.save(&config).unwrap();

        // 备份
        let backup_path = manager.backup().unwrap();
        assert!(backup_path.exists());

        // 修改配置
        config.default_config = "modified".into();
        manager.save(&config).unwrap();

        // 恢复
        manager.restore(&backup_path).unwrap();
        let restored = manager.load().unwrap();
        assert_eq!(restored.default_config, "original");
    }
}
