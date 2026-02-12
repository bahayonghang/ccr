// 🏭 Droid Platform 实现
// 📦 Factory Droid CLI 平台配置管理
//
// 核心职责:
// - 📋 管理 Droid profiles
// - ⚙️ 操作 ~/.factory/settings.json
// - 🔗 兼容现有 ProfileConfig 结构
//
// 📁 配置结构 (Unified 模式):
// - CCR 配置: `~/.ccr/platforms/droid/profiles.toml`
// - Droid 设置: `~/.factory/settings.json`

use crate::core::error::{CcrError, Result};
use crate::managers::PlatformConfigManager;
use crate::managers::config::ConfigSection;
use crate::models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use crate::platforms::base;
use crate::utils::Validatable;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════
// 📦 Droid 配置结构定义
// ═══════════════════════════════════════════════════════════

/// 🏭 Droid Settings 结构
///
/// 对应 ~/.factory/settings.json 的结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DroidSettings {
    /// 🤖 自定义模型列表
    #[serde(default, rename = "customModels")]
    pub custom_models: Vec<DroidCustomModel>,

    /// 📦 其他设置字段 (保持原样)
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// 🤖 Droid 自定义模型结构
///
/// Factory Droid CLI 的 customModels 数组元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroidCustomModel {
    /// 模型标识符 (API 使用)
    pub model: String,

    /// 显示名称 (CLI 界面显示)
    #[serde(skip_serializing_if = "Option::is_none", rename = "displayName")]
    pub display_name: Option<String>,

    /// API 端点 URL
    #[serde(rename = "baseUrl")]
    pub base_url: String,

    /// API 密钥
    #[serde(rename = "apiKey")]
    pub api_key: String,

    /// 提供商类型: anthropic / openai / generic-chat-completion-api
    pub provider: String,

    /// 最大输出 token 数
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxOutputTokens")]
    pub max_output_tokens: Option<u32>,
}

// ═══════════════════════════════════════════════════════════
// 🏭 DroidPlatform 实现
// ═══════════════════════════════════════════════════════════

/// 🏭 Droid Platform 实现
///
/// ## 配置结构 (Unified 模式)
///
/// - CCR 配置: `~/.ccr/platforms/droid/profiles.toml`
/// - Droid 设置: `~/.factory/settings.json`
pub struct DroidPlatform {
    paths: PlatformPaths,
    settings_path: PathBuf,
}

impl DroidPlatform {
    /// 🏗️ 创建新的 Droid Platform 实例
    pub fn new() -> Result<Self> {
        let paths = PlatformPaths::new(Platform::Droid)?;

        // Droid 使用固定路径 ~/.factory/settings.json
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        let settings_path = home.join(".factory").join("settings.json");

        Ok(Self {
            paths,
            settings_path,
        })
    }

    /// 📖 加载 Droid settings.json
    fn load_droid_settings(&self) -> Result<DroidSettings> {
        if !self.settings_path.exists() {
            return Ok(DroidSettings::default());
        }

        let content = fs::read_to_string(&self.settings_path)
            .map_err(|e| CcrError::SettingsError(format!("读取 Droid 设置失败: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| CcrError::ConfigFormatInvalid(format!("解析 Droid 设置失败: {}", e)))
    }

    /// 💾 保存 Droid settings.json
    fn save_droid_settings(&self, settings: &DroidSettings) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::SettingsError(format!("创建 Droid 设置目录失败: {}", e)))?;
        }

        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| CcrError::SettingsError(format!("序列化 Droid 设置失败: {}", e)))?;

        fs::write(&self.settings_path, content)
            .map_err(|e| CcrError::SettingsError(format!("写入 Droid 设置失败: {}", e)))?;

        tracing::info!("✅ 已保存 Droid 设置: {:?}", self.settings_path);
        Ok(())
    }

    /// 📋 从 ProfileConfig 转换为 DroidCustomModel
    fn profile_to_custom_model(profile: &ProfileConfig) -> DroidCustomModel {
        // 从 platform_data 中提取 maxOutputTokens
        let max_output_tokens = profile
            .platform_data
            .get("max_output_tokens")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        DroidCustomModel {
            model: profile.model.clone().unwrap_or_default(),
            display_name: profile.description.clone(),
            base_url: profile.base_url.clone().unwrap_or_default(),
            api_key: profile.auth_token.clone().unwrap_or_default(),
            provider: profile
                .provider
                .clone()
                .unwrap_or_else(|| "anthropic".to_string()),
            max_output_tokens,
        }
    }

    /// 📋 从 ConfigSection 转换为 ProfileConfig
    #[expect(dead_code)]
    fn section_to_profile(section: &ConfigSection) -> ProfileConfig {
        base::section_to_profile(section)
    }

    /// 📋 从 ProfileConfig 转换为 ConfigSection
    fn profile_to_section(profile: &ProfileConfig) -> Result<ConfigSection> {
        base::profile_to_section(profile)
    }

    /// 💾 保存 profiles 到 TOML 文件
    fn save_profiles(&self, profiles: &IndexMap<String, ProfileConfig>) -> Result<()> {
        base::save_profiles_to_toml(&self.paths.profiles_file, profiles, "droid", &self.paths)
    }

    /// 🔄 更新 profiles.toml 中的 current_config 字段
    fn update_current_config_in_profiles(&self, name: &str) -> Result<()> {
        base::update_current_config(&self.paths.profiles_file, name)
    }

    /// 📖 从 TOML 文件加载 profiles
    fn load_profiles_from_file(&self) -> Result<IndexMap<String, ProfileConfig>> {
        base::load_profiles_from_toml(&self.paths.profiles_file)
    }
}

impl PlatformConfig for DroidPlatform {
    fn platform_name(&self) -> &str {
        "droid"
    }

    fn platform_type(&self) -> Platform {
        Platform::Droid
    }

    fn load_profiles(&self) -> Result<IndexMap<String, ProfileConfig>> {
        self.load_profiles_from_file()
    }

    fn save_profile(&self, name: &str, profile: &ProfileConfig) -> Result<()> {
        // 先验证
        self.validate_profile(profile)?;

        let mut profiles = self.load_profiles()?;
        profiles.insert(name.to_string(), profile.clone());
        self.save_profiles(&profiles)
    }

    fn delete_profile(&self, name: &str) -> Result<()> {
        let mut profiles = self.load_profiles()?;
        if profiles.shift_remove(name).is_none() {
            return Err(CcrError::ProfileNotFound(name.to_string()));
        }
        self.save_profiles(&profiles)
    }

    fn get_settings_path(&self) -> PathBuf {
        self.settings_path.clone()
    }

    fn apply_profile(&self, name: &str) -> Result<()> {
        // 加载 profile
        let profiles = self.load_profiles()?;
        let profile = profiles
            .get(name)
            .ok_or_else(|| CcrError::ProfileNotFound(name.to_string()))?;

        // 转换为 ConfigSection 并验证
        let section = Self::profile_to_section(profile)?;
        section.validate()?;

        // 加载现有 Droid 设置
        let mut settings = self.load_droid_settings()?;

        // 🔄 替换策略：用当前 profile 生成的 customModel 替换整个 customModels 数组
        let custom_model = Self::profile_to_custom_model(profile);
        settings.custom_models = vec![custom_model];

        // 保存 Droid 设置
        self.save_droid_settings(&settings)?;

        // 更新 profiles.toml 中的 current_config
        self.update_current_config_in_profiles(name)?;

        // 同步更新注册表中的 current_profile
        let platform_config_mgr = PlatformConfigManager::with_default()?;
        let mut unified_config = platform_config_mgr.load()?;

        // 更新 Droid 平台的 current_profile
        unified_config.set_platform_profile("droid", name)?;

        // 保存注册表
        platform_config_mgr.save(&unified_config)?;

        tracing::debug!("✅ 已更新注册表 current_profile: {}", name);
        tracing::info!("✅ 已应用 Droid profile: {}", name);
        Ok(())
    }

    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()> {
        // Droid 需要: base_url, auth_token
        if profile.base_url.is_none()
            || profile
                .base_url
                .as_ref()
                .map(|s| s.is_empty())
                .unwrap_or(true)
        {
            return Err(CcrError::ValidationError("缺少 base_url".into()));
        }

        if profile.auth_token.is_none()
            || profile
                .auth_token
                .as_ref()
                .map(|s| s.is_empty())
                .unwrap_or(true)
        {
            return Err(CcrError::ValidationError("缺少 auth_token (apiKey)".into()));
        }

        Ok(())
    }

    fn get_current_profile(&self) -> Result<Option<String>> {
        base::get_current_profile_from_registry("droid")
    }

    fn get_env_var_names(&self) -> Vec<&'static str> {
        // Droid 不使用环境变量，直接写入 settings.json
        vec![]
    }
}

// ═══════════════════════════════════════════════════════════
// 🧪 测试
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_droid_custom_model_serialization() {
        let model = DroidCustomModel {
            model: "claude-sonnet-4-5-20250929".to_string(),
            display_name: Some("My Claude".to_string()),
            base_url: "https://api.example.com/v1".to_string(),
            api_key: "sk-test-key".to_string(),
            provider: "anthropic".to_string(),
            max_output_tokens: Some(8192),
        };

        let json = serde_json::to_string_pretty(&model).unwrap();
        assert!(json.contains("baseUrl"));
        assert!(json.contains("apiKey"));
        assert!(json.contains("displayName"));
        assert!(json.contains("maxOutputTokens"));
    }

    #[test]
    fn test_droid_settings_serialization() {
        let settings = DroidSettings {
            custom_models: vec![DroidCustomModel {
                model: "test-model".to_string(),
                display_name: Some("Test".to_string()),
                base_url: "https://api.test.com".to_string(),
                api_key: "sk-test".to_string(),
                provider: "openai".to_string(),
                max_output_tokens: None,
            }],
            other: HashMap::new(),
        };

        let json = serde_json::to_string_pretty(&settings).unwrap();
        assert!(json.contains("customModels"));

        // 反序列化
        let parsed: DroidSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.custom_models.len(), 1);
        assert_eq!(parsed.custom_models[0].model, "test-model");
    }

    #[test]
    fn test_profile_to_custom_model() {
        let mut profile = ProfileConfig::new();
        profile.model = Some("claude-sonnet-4-5".to_string());
        profile.base_url = Some("https://api.anthropic.com/v1".to_string());
        profile.auth_token = Some("sk-ant-xxx".to_string());
        profile.provider = Some("anthropic".to_string());
        profile.description = Some("My Anthropic".to_string());
        profile
            .platform_data
            .insert("max_output_tokens".to_string(), serde_json::json!(8192));

        let custom_model = DroidPlatform::profile_to_custom_model(&profile);

        assert_eq!(custom_model.model, "claude-sonnet-4-5");
        assert_eq!(custom_model.base_url, "https://api.anthropic.com/v1");
        assert_eq!(custom_model.api_key, "sk-ant-xxx");
        assert_eq!(custom_model.provider, "anthropic");
        assert_eq!(custom_model.display_name, Some("My Anthropic".to_string()));
        assert_eq!(custom_model.max_output_tokens, Some(8192));
    }

    #[test]
    fn test_platform_trait_impl() {
        if let Ok(platform) = DroidPlatform::new() {
            assert_eq!(platform.platform_name(), "droid");
            assert_eq!(platform.platform_type(), Platform::Droid);
            assert!(
                platform
                    .get_settings_path()
                    .to_str()
                    .unwrap()
                    .contains("factory")
            );
        }
    }
}
