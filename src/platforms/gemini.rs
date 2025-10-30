// ✨ Gemini Platform 实现
// 📦 Google Gemini CLI 平台配置管理
//
// 核心职责:
// - 📋 管理 Gemini profiles
// - ⚙️ 操作 Gemini settings.json
// - 🔐 验证 Google API key 格式
// - 💾 仅支持 Unified 模式

use crate::core::error::{CcrError, Result};
use crate::managers::PlatformConfigManager;
use crate::models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use crate::utils::Validatable;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Gemini字段提取结果: (api_key, region, language, context_length)
type GeminiFields = (String, Option<String>, Option<String>, Option<String>);

/// ✨ Gemini Platform 实现
///
/// ## 配置文件
/// - Profiles: `~/.ccr/platforms/gemini/profiles.toml`
/// - Settings: `~/.ccr/platforms/gemini/settings.json`
///
/// ## Google API Key 格式
/// Google API keys 通常以 `AIza` 开头
pub struct GeminiPlatform {
    paths: PlatformPaths,
}

/// ✨ Gemini 设置结构
///
/// Gemini settings.json 格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiSettings {
    /// Google 配置
    pub google: GoogleConfig,
}

/// 🔐 Google 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleConfig {
    /// API Key
    pub api_key: String,

    /// Google Cloud Project ID（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,

    /// Region（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// API Version（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
}

impl GeminiPlatform {
    /// 🏗️ 创建新的 Gemini Platform 实例
    pub fn new() -> Result<Self> {
        let paths = PlatformPaths::new(Platform::Gemini)?;
        Ok(Self { paths })
    }

    /// 📋 从 TOML 文件加载 profiles
    fn load_profiles_from_file(&self) -> Result<IndexMap<String, ProfileConfig>> {
        if !self.paths.profiles_file.exists() {
            return Ok(IndexMap::new());
        }

        // 读取文件
        let content = fs::read_to_string(&self.paths.profiles_file)
            .map_err(|e| CcrError::ConfigError(format!("读取 Gemini 配置失败: {}", e)))?;

        // 解析 TOML
        let profiles: IndexMap<String, ProfileConfig> = toml::from_str(&content)
            .map_err(|e| CcrError::ConfigFormatInvalid(format!("Gemini 配置格式错误: {}", e)))?;

        Ok(profiles)
    }

    /// 💾 保存 profiles 到 TOML 文件
    fn save_profiles_to_file(&self, profiles: &IndexMap<String, ProfileConfig>) -> Result<()> {
        // 确保目录存在
        self.paths.ensure_directories()?;

        // 序列化为 TOML
        let content = toml::to_string_pretty(profiles)
            .map_err(|e| CcrError::ConfigError(format!("序列化 Gemini 配置失败: {}", e)))?;

        // 写入文件
        fs::write(&self.paths.profiles_file, content)
            .map_err(|e| CcrError::ConfigError(format!("写入 Gemini 配置失败: {}", e)))?;

        log::info!("✅ 已保存 Gemini profiles: {:?}", self.paths.profiles_file);
        Ok(())
    }

    /// 📖 加载 Gemini settings
    #[allow(dead_code)]
    fn load_settings(&self) -> Result<GeminiSettings> {
        if !self.paths.settings_file.exists() {
            return Err(CcrError::SettingsMissing(
                self.paths.settings_file.display().to_string(),
            ));
        }

        let content = fs::read_to_string(&self.paths.settings_file)
            .map_err(|e| CcrError::SettingsError(format!("读取 Gemini 设置失败: {}", e)))?;

        let settings: GeminiSettings = serde_json::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("解析 Gemini 设置失败: {}", e)))?;

        Ok(settings)
    }

    /// 💾 保存 Gemini settings
    fn save_settings(&self, settings: &GeminiSettings) -> Result<()> {
        // 确保目录存在
        self.paths.ensure_directories()?;

        // 序列化为 JSON
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| CcrError::SettingsError(format!("序列化 Gemini 设置失败: {}", e)))?;

        // 写入文件
        fs::write(&self.paths.settings_file, content)
            .map_err(|e| CcrError::SettingsError(format!("写入 Gemini 设置失败: {}", e)))?;

        log::info!("✅ 已保存 Gemini settings: {:?}", self.paths.settings_file);
        Ok(())
    }

    /// 🔐 验证 Google API Key 格式
    ///
    /// Google API keys 通常：
    /// - 以 `AIza` 开头
    /// - 长度约 39 个字符
    fn validate_api_key(api_key: &str) -> Result<()> {
        if !api_key.starts_with("AIza") {
            return Err(CcrError::ValidationError(
                "Google API key 应以 'AIza' 开头".into(),
            ));
        }

        if api_key.len() < 30 {
            return Err(CcrError::ValidationError("Google API key 长度不足".into()));
        }

        Ok(())
    }

    /// 📋 从 ProfileConfig 提取 Gemini 特定字段
    fn extract_gemini_fields(profile: &ProfileConfig) -> Result<GeminiFields> {
        let api_key = profile
            .auth_token
            .as_ref()
            .ok_or_else(|| CcrError::ValidationError("缺少 api_key 字段".into()))?
            .clone();

        let project_id = profile
            .platform_data
            .get("project_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let region = profile
            .platform_data
            .get("region")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let api_version = profile
            .platform_data
            .get("api_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok((api_key, project_id, region, api_version))
    }
}

impl PlatformConfig for GeminiPlatform {
    fn platform_name(&self) -> &str {
        "gemini"
    }

    fn platform_type(&self) -> Platform {
        Platform::Gemini
    }

    fn load_profiles(&self) -> Result<IndexMap<String, ProfileConfig>> {
        self.load_profiles_from_file()
    }

    fn save_profile(&self, name: &str, profile: &ProfileConfig) -> Result<()> {
        // 先验证
        self.validate_profile(profile)?;

        // 加载现有 profiles
        let mut profiles = self.load_profiles()?;

        // 添加/更新 profile
        profiles.insert(name.to_string(), profile.clone());

        // 保存
        self.save_profiles_to_file(&profiles)
    }

    fn delete_profile(&self, name: &str) -> Result<()> {
        let mut profiles = self.load_profiles()?;

        if profiles.shift_remove(name).is_none() {
            return Err(CcrError::ProfileNotFound(name.to_string()));
        }

        self.save_profiles_to_file(&profiles)
    }

    fn get_settings_path(&self) -> PathBuf {
        self.paths.settings_file.clone()
    }

    fn apply_profile(&self, name: &str) -> Result<()> {
        // 加载 profile
        let profiles = self.load_profiles()?;
        let profile = profiles
            .get(name)
            .ok_or_else(|| CcrError::ProfileNotFound(name.to_string()))?;

        // 验证
        self.validate_profile(profile)?;

        // 提取 Gemini 特定字段
        let (api_key, project_id, region, api_version) = Self::extract_gemini_fields(profile)?;

        // 构建 settings
        let settings = GeminiSettings {
            google: GoogleConfig {
                api_key,
                project_id,
                region,
                api_version,
            },
        };

        // 保存 settings
        self.save_settings(&settings)?;

        // 在 Unified 模式下，同步更新注册表中的 current_profile
        let platform_config_mgr = PlatformConfigManager::with_default()?;
        let mut unified_config = platform_config_mgr.load()?;

        // 更新 Gemini 平台的 current_profile
        unified_config.set_platform_profile("gemini", name)?;

        // 保存注册表
        platform_config_mgr.save(&unified_config)?;

        log::debug!("✅ 已更新注册表 current_profile: {}", name);

        log::info!("✅ 已应用 Gemini profile: {}", name);
        Ok(())
    }

    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()> {
        // 检查必需字段：API key
        if profile.auth_token.is_none() {
            return Err(CcrError::ValidationError(
                "Gemini profile 缺少 auth_token (API key)".into(),
            ));
        }

        let api_key = profile.auth_token.as_ref().unwrap();
        Self::validate_api_key(api_key)?;

        Ok(())
    }

    fn get_current_profile(&self) -> Result<Option<String>> {
        // Gemini 在 Unified 模式下，从注册表读取 current_profile
        let platform_config_mgr = PlatformConfigManager::with_default()?;
        let unified_config = platform_config_mgr.load()?;

        // 获取 Gemini 平台的注册信息
        let gemini_entry = unified_config.get_platform("gemini")?;
        Ok(gemini_entry.current_profile.clone())
    }
}

impl Validatable for GeminiSettings {
    fn validate(&self) -> Result<()> {
        // 验证 API key
        GeminiPlatform::validate_api_key(&self.google.api_key)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_api_key() {
        // 有效的 API key
        assert!(GeminiPlatform::validate_api_key("AIzaSyDtWl5vKg1234567890abcdefgh").is_ok());

        // 无效的 API keys
        assert!(GeminiPlatform::validate_api_key("invalid_key").is_err());
        assert!(GeminiPlatform::validate_api_key("AIzaShort").is_err());
        assert!(GeminiPlatform::validate_api_key("").is_err());
    }

    #[test]
    fn test_gemini_settings_structure() {
        let settings = GeminiSettings {
            google: GoogleConfig {
                api_key: "AIzaSyDtWl5vKg1234567890abcdefgh".to_string(),
                project_id: Some("my-project-123".to_string()),
                region: Some("us-central1".to_string()),
                api_version: Some("v1".to_string()),
            },
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_gemini_platform_basic() {
        if let Ok(platform) = GeminiPlatform::new() {
            assert_eq!(platform.platform_name(), "gemini");
            assert_eq!(platform.platform_type(), Platform::Gemini);
            assert!(
                platform
                    .get_settings_path()
                    .to_str()
                    .unwrap()
                    .contains("gemini")
            );
        }
    }

    #[test]
    fn test_validate_profile() {
        let platform = GeminiPlatform::new().unwrap();

        // 有效的 profile
        let valid_profile = ProfileConfig {
            description: Some("Google Gemini".to_string()),
            base_url: None,
            auth_token: Some("AIzaSyDtWl5vKg1234567890abcdefgh".to_string()),
            model: Some("gemini-pro".to_string()),
            small_fast_model: None,
            provider: Some("google".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            platform_data: IndexMap::new(),
        };

        assert!(platform.validate_profile(&valid_profile).is_ok());

        // 无效的 profile（缺少 API key）
        let mut invalid_profile = valid_profile.clone();
        invalid_profile.auth_token = None;
        assert!(platform.validate_profile(&invalid_profile).is_err());

        // 无效的 profile（错误的 API key 格式）
        let mut invalid_profile = valid_profile;
        invalid_profile.auth_token = Some("invalid_key".to_string());
        assert!(platform.validate_profile(&invalid_profile).is_err());
    }
}
