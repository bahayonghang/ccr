// ✨ Antigravity Platform 实现 (legacy key: gemini)
// 📦 Google Antigravity CLI 平台配置管理
//
// 核心职责:
// - 📋 管理 Antigravity profiles（CCR 内部 platform key 仍为 gemini）
// - ⚙️ 操作 Antigravity settings.json
// - 🔐 验证 Google API key 格式
// - 💾 仅支持 Unified 模式

use crate::models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use ccr_config::platforms::base;
use ccr_core::Validatable;
use ccr_core::core::error::{CcrError, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Antigravity 字段提取结果: (api_key, project_id, region, api_version)
type GeminiFields = (String, Option<String>, Option<String>, Option<String>);

/// ✨ Antigravity Platform 实现
///
/// ## 配置文件
/// - Profiles: `~/.ccr/platforms/gemini/profiles.toml`
/// - Settings: `~/.gemini/antigravity-cli/settings.json`
///
/// ## Google API Key 格式
/// Google API keys 通常以 `AIza` 开头
pub struct GeminiPlatform {
    paths: PlatformPaths,
    settings_path: PathBuf,
}

/// ✨ Antigravity 设置结构
///
/// Antigravity settings.json 格式
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
    /// 🏗️ 创建新的 Antigravity Platform 实例
    pub fn new() -> Result<Self> {
        let paths = PlatformPaths::new(Platform::Gemini)?;
        let settings_path = Self::default_settings_path()?;
        Ok(Self {
            paths,
            settings_path,
        })
    }

    fn default_cli_dir() -> Result<PathBuf> {
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        Ok(home.join(".gemini").join("antigravity-cli"))
    }

    fn default_settings_path() -> Result<PathBuf> {
        Ok(Self::default_cli_dir()?.join("settings.json"))
    }

    /// 📋 从 TOML 文件加载 profiles
    fn load_profiles_from_file(&self) -> Result<IndexMap<String, ProfileConfig>> {
        base::load_profiles_from_toml(&self.paths.profiles_file)
    }

    /// 💾 保存 profiles 到 TOML 文件
    fn save_profiles_to_file(&self, profiles: &IndexMap<String, ProfileConfig>) -> Result<()> {
        base::save_profiles_to_toml(&self.paths.profiles_file, profiles, "gemini", &self.paths)
    }

    /// 📖 加载 Antigravity settings
    #[expect(dead_code)]
    fn load_settings(&self) -> Result<GeminiSettings> {
        if !self.settings_path.exists() {
            return Err(CcrError::SettingsMissing(
                self.settings_path.display().to_string(),
            ));
        }

        let content = fs::read_to_string(&self.settings_path)
            .map_err(|e| CcrError::SettingsError(format!("读取 Antigravity 设置失败: {}", e)))?;

        let settings: GeminiSettings = serde_json::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("解析 Antigravity 设置失败: {}", e)))?;

        Ok(settings)
    }

    /// 💾 保存 Antigravity settings
    fn save_settings(&self, settings: &GeminiSettings) -> Result<()> {
        self.paths.ensure_directories()?;
        Self::ensure_antigravity_dir(&self.settings_path)?;

        // 序列化为 JSON
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| CcrError::SettingsError(format!("序列化 Antigravity 设置失败: {}", e)))?;

        // 写入文件
        fs::write(&self.settings_path, content)
            .map_err(|e| CcrError::SettingsError(format!("写入 Antigravity 设置失败: {}", e)))?;

        tracing::info!(
            path = ?self.settings_path,
            corr = ccr_core::current_log_correlation_id(),
            "saved Antigravity settings"
        );
        Ok(())
    }

    fn ensure_antigravity_dir(settings_path: &std::path::Path) -> Result<()> {
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CcrError::SettingsError(format!("创建 Antigravity 设置目录失败: {}", e))
            })?;
        }
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

    /// 📋 从 ProfileConfig 提取 Antigravity 特定字段
    fn extract_gemini_fields(profile: &ProfileConfig) -> Result<GeminiFields> {
        // 写入 Antigravity 配置文件需要原文（合法明文消费点）
        let api_key = profile
            .auth_token
            .as_ref()
            .ok_or_else(|| CcrError::ValidationError("缺少 api_key 字段".into()))?
            .expose()
            .to_string();

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

        self.save_profiles_to_file(&profiles)?;
        base::reconcile_registry_current_profile_after_delete("gemini", name, &profiles)
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

        // 验证
        self.validate_profile(profile)?;

        // 提取 Antigravity 特定字段
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

        // 使用 base 模块更新注册表
        base::update_registry_current_profile("gemini", name)?;

        tracing::info!(
            profile = name,
            corr = ccr_core::current_log_correlation_id(),
            "applied Antigravity profile"
        );
        Ok(())
    }

    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()> {
        // 检查必需字段：API key
        let api_key = profile.auth_token.as_ref().ok_or_else(|| {
            CcrError::ValidationError("Antigravity profile 缺少 auth_token (API key)".into())
        })?;
        Self::validate_api_key(api_key.expose())?;

        Ok(())
    }

    fn get_current_profile(&self) -> Result<Option<String>> {
        base::get_current_profile_from_registry("gemini")
    }

    fn get_env_var_names(&self) -> Vec<String> {
        vec!["GEMINI_API_KEY".into()]
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
#[allow(clippy::unwrap_used)]
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
                    .contains("antigravity-cli")
            );
        }
    }

    #[test]
    fn test_default_settings_path_uses_antigravity_cli_dir() {
        let path = GeminiPlatform::default_settings_path()
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");

        assert!(path.ends_with(".gemini/antigravity-cli/settings.json"));
        assert!(!path.contains(".ccr/platforms/gemini/settings.json"));
    }

    #[test]
    fn test_validate_profile() {
        let platform = GeminiPlatform::new().unwrap();

        // 有效的 profile
        let valid_profile = ProfileConfig {
            description: Some("Google Antigravity".to_string()),
            base_url: None,
            auth_token: Some(ccr_core::Secret::from("AIzaSyDtWl5vKg1234567890abcdefgh")),
            model: Some("gemini-pro".to_string()),
            small_fast_model: None,
            provider: Some("google".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
            ..Default::default()
        };

        assert!(platform.validate_profile(&valid_profile).is_ok());

        // 无效的 profile（缺少 API key）
        let mut invalid_profile = valid_profile.clone();
        invalid_profile.auth_token = None;
        assert!(platform.validate_profile(&invalid_profile).is_err());

        // 无效的 profile（错误的 API key 格式）
        let mut invalid_profile = valid_profile;
        invalid_profile.auth_token = Some(ccr_core::Secret::from("invalid_key"));
        assert!(platform.validate_profile(&invalid_profile).is_err());
    }
}
