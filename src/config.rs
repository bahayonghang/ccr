// ⚙️ CCR 配置管理模块
// 📁 负责读写和管理 ~/.ccs_config.toml 配置文件
//
// 核心职责:
// - 🔍 解析 TOML 配置文件
// - 💾 保存配置到文件
// - ✅ 验证配置完整性
// - 📋 管理多个配置节

use crate::error::{CcrError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 📝 配置节结构
///
/// 代表一个具体的 API 配置（如 anthropic、anyrouter 等）
/// 
/// 每个配置节包含:
/// - 🏷️ 描述信息
/// - 🌐 API 基础 URL
/// - 🔑 认证令牌
/// - 🤖 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSection {
    /// 📝 配置描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 🌐 API 基础 URL（切换时必需）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// 🔑 认证令牌（切换时必需）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,

    /// 🤖 默认模型名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// ⚡ 快速小模型名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_fast_model: Option<String>,
}

impl ConfigSection {
    /// ✅ 验证配置节的完整性
    /// 
    /// 验证规则:
    /// 1. 🌐 base_url 必须存在且符合 URL 格式
    /// 2. 🔑 auth_token 必须存在且非空
    /// 3. 🤖 model 如果提供则不能为空字符串
    pub fn validate(&self) -> Result<()> {
        // 🌐 检查 base_url
        let base_url = self.base_url.as_ref()
            .ok_or_else(|| CcrError::ValidationError("base_url 不能为空".into()))?;

        if base_url.trim().is_empty() {
            return Err(CcrError::ValidationError("base_url 不能为空".into()));
        }

        // 🔍 简单的 URL 格式验证
        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            return Err(CcrError::ValidationError(
                "base_url 必须以 http:// 或 https:// 开头".into(),
            ));
        }

        // 🔑 检查 auth_token
        let auth_token = self.auth_token.as_ref()
            .ok_or_else(|| CcrError::ValidationError("auth_token 不能为空".into()))?;

        if auth_token.trim().is_empty() {
            return Err(CcrError::ValidationError("auth_token 不能为空".into()));
        }

        // 🤖 检查 model（可选，如果提供了则不能为空）
        if let Some(model) = &self.model {
            if model.trim().is_empty() {
                return Err(CcrError::ValidationError("model 不能为空字符串".into()));
            }
        }

        Ok(())
    }

    /// 📝 获取配置的人类可读描述
    pub fn display_description(&self) -> String {
        self.description
            .clone()
            .unwrap_or_else(|| "(无描述)".to_string())
    }
}

/// 📦 CCS 配置文件总体结构
///
/// 对应 ~/.ccs_config.toml 的完整结构
/// 
/// 结构说明:
/// - 🎯 default_config: 默认配置名
/// - ▶️ current_config: 当前激活配置
/// - 📋 sections: 所有具体配置节的集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcsConfig {
    /// 🎯 默认配置名称
    pub default_config: String,

    /// ▶️ 当前活跃配置名称
    pub current_config: String,

    /// 📋 所有配置节（使用 flatten 序列化）
    #[serde(flatten)]
    pub sections: HashMap<String, ConfigSection>,
}

impl CcsConfig {
    /// 🔍 获取指定配置节
    pub fn get_section(&self, name: &str) -> Result<&ConfigSection> {
        self.sections
            .get(name)
            .ok_or_else(|| CcrError::ConfigSectionNotFound(name.to_string()))
    }

    /// ▶️ 获取当前配置节
    pub fn get_current_section(&self) -> Result<&ConfigSection> {
        self.get_section(&self.current_config)
    }

    /// 🔄 设置当前配置
    /// 
    /// 切换前会验证目标配置是否存在
    pub fn set_current(&mut self, name: &str) -> Result<()> {
        // ✅ 验证配置节存在
        if !self.sections.contains_key(name) {
            return Err(CcrError::ConfigSectionNotFound(name.to_string()));
        }
        self.current_config = name.to_string();
        Ok(())
    }

    /// ➕ 添加或更新配置节
    pub fn set_section(&mut self, name: String, section: ConfigSection) {
        self.sections.insert(name, section);
    }

    /// ➖ 删除配置节
    pub fn remove_section(&mut self, name: &str) -> Result<ConfigSection> {
        self.sections
            .remove(name)
            .ok_or_else(|| CcrError::ConfigSectionNotFound(name.to_string()))
    }

    /// 📜 列出所有配置节名称（已排序）
    pub fn list_sections(&self) -> Vec<String> {
        let mut names: Vec<String> = self.sections.keys().cloned().collect();
        names.sort();
        names
    }

    /// ✅ 验证所有配置节
    /// 
    /// 返回每个配置节的验证结果 HashMap
    pub fn validate_all(&self) -> HashMap<String, Result<()>> {
        self.sections
            .iter()
            .map(|(name, section)| (name.clone(), section.validate()))
            .collect()
    }
}

/// 🔧 配置管理器
///
/// 负责配置文件的加载、保存和管理
/// 
/// 主要功能:
/// - 📖 从磁盘加载 TOML 配置
/// - 💾 保存配置到磁盘
/// - 🔍 解析和验证配置格式
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// 🏗️ 创建新的配置管理器
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    /// 🏠 使用默认配置路径创建管理器
    /// 
    /// 默认路径: ~/.ccs_config.toml
    pub fn default() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        let config_path = home.join(".ccs_config.toml");
        Ok(Self::new(config_path))
    }

    /// 📁 获取配置文件路径
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// 📖 加载配置文件
    /// 
    /// 执行步骤:
    /// 1. ✅ 检查文件是否存在
    /// 2. 📄 读取文件内容
    /// 3. 🔍 解析 TOML 格式
    pub fn load(&self) -> Result<CcsConfig> {
        // ✅ 检查文件是否存在
        if !self.config_path.exists() {
            return Err(CcrError::ConfigMissing(
                self.config_path.display().to_string(),
            ));
        }

        // 📄 读取文件内容
        let content = fs::read_to_string(&self.config_path).map_err(|e| {
            CcrError::ConfigError(format!("读取配置文件失败: {}", e))
        })?;

        // 🔍 解析 TOML
        let config: CcsConfig = toml::from_str(&content).map_err(|e| {
            CcrError::ConfigFormatInvalid(format!("TOML 解析失败: {}", e))
        })?;

        log::debug!(
            "✅ 成功加载配置文件: {:?}, 配置节数量: {}",
            self.config_path,
            config.sections.len()
        );

        Ok(config)
    }

    /// 💾 保存配置文件
    /// 
    /// 执行步骤:
    /// 1. 📝 序列化为 TOML 格式
    /// 2. 💾 写入磁盘
    pub fn save(&self, config: &CcsConfig) -> Result<()> {
        // 📝 序列化为 TOML（美化格式）
        let content = toml::to_string_pretty(config).map_err(|e| {
            CcrError::ConfigError(format!("配置序列化失败: {}", e))
        })?;

        // 💾 写入文件
        fs::write(&self.config_path, content).map_err(|e| {
            CcrError::ConfigError(format!("写入配置文件失败: {}", e))
        })?;

        log::debug!("✅ 配置文件已保存: {:?}", self.config_path);
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
    fn test_ccs_config() {
        let mut config = CcsConfig {
            default_config: "default".into(),
            current_config: "default".into(),
            sections: HashMap::new(),
        };
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
        let mut config = CcsConfig {
            default_config: "test".into(),
            current_config: "test".into(),
            sections: HashMap::new(),
        };
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
}
