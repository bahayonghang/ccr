// 💻 Codex Platform 实现
// 📦 GitHub Copilot CLI 平台配置管理
//
// 核心职责:
// - 📋 管理 Codex profiles
// - ⚙️ 操作 Codex settings.json
// - 🔐 验证 GitHub token 格式
// - 💾 仅支持 Unified 模式

use crate::core::error::{CcrError, Result};
use crate::managers::PlatformConfigManager;
use crate::models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use crate::utils::Validatable;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::env;
use std::fs;
use std::path::PathBuf;

/// 💻 Codex Platform 实现
///
/// ## 配置文件
/// - Profiles: `~/.ccr/platforms/codex/profiles.toml`
/// - Settings: `~/.ccr/platforms/codex/settings.json`
///
/// ## GitHub Token 格式
/// 支持以下前缀的 GitHub token：
/// - `ghp_` - Personal Access Token
/// - `gho_` - OAuth Token
/// - `github_pat_` - Fine-grained Personal Access Token
pub struct CodexPlatform {
    paths: PlatformPaths,
}

/// 🔐 Codex 设置结构
///
/// Codex settings.json 格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexSettings {
    /// GitHub 配置
    pub github: GitHubConfig,
    /// 默认模型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// 🔐 GitHub 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// API 端点
    pub api_endpoint: String,
    /// GitHub Token
    pub token: String,
    /// 组织名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
}

impl CodexPlatform {
    /// 🏗️ 创建新的 Codex Platform 实例
    pub fn new() -> Result<Self> {
        let paths = PlatformPaths::new(Platform::Codex)?;
        Ok(Self { paths })
    }

    /// 📁 获取 Codex CLI 配置目录
    fn codex_dir() -> Result<PathBuf> {
        if let Ok(custom) = env::var("CCR_CODEX_DIR") {
            return Ok(PathBuf::from(custom));
        }

        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        Ok(home.join(".codex"))
    }

    /// ⚙️ 获取 config.toml 路径
    fn codex_config_path() -> Result<PathBuf> {
        Ok(Self::codex_dir()?.join("config.toml"))
    }

    /// 🔑 获取 auth.json 路径
    fn codex_auth_path() -> Result<PathBuf> {
        Ok(Self::codex_dir()?.join("auth.json"))
    }

    /// 🔍 判断是否使用 GitHub Copilot CLI 兼容模式
    fn is_github_profile(profile: &ProfileConfig) -> bool {
        if let Some(mode) = Self::platform_string(profile, "api_mode") {
            if mode.eq_ignore_ascii_case("github") {
                return true;
            }
            if mode.eq_ignore_ascii_case("custom") {
                return false;
            }
        }

        if profile
            .platform_data
            .get("wire_api")
            .and_then(|v| v.as_str())
            .is_some()
        {
            return false;
        }

        if let Some(base_url) = profile.base_url.as_deref() {
            return base_url.contains("github.com");
        }

        // 默认按自定义 API 处理
        false
    }

    fn platform_string(profile: &ProfileConfig, key: &str) -> Option<String> {
        profile
            .platform_data
            .get(key)
            .and_then(|value| match value {
                JsonValue::String(text) if !text.trim().is_empty() => Some(text.trim().to_string()),
                JsonValue::Number(num) => Some(num.to_string()),
                JsonValue::Bool(flag) => Some(flag.to_string()),
                _ => None,
            })
    }

    fn platform_bool(profile: &ProfileConfig, key: &str) -> Option<bool> {
        profile
            .platform_data
            .get(key)
            .and_then(|value| match value {
                JsonValue::Bool(flag) => Some(*flag),
                JsonValue::String(text) => {
                    let trimmed = text.trim().to_lowercase();
                    match trimmed.as_str() {
                        "true" | "1" => Some(true),
                        "false" | "0" => Some(false),
                        _ => None,
                    }
                }
                _ => None,
            })
    }

    fn sanitize_identifier(raw: &str) -> String {
        let mut sanitized = raw
            .trim()
            .to_lowercase()
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                    ch
                } else {
                    '-'
                }
            })
            .collect::<String>();

        sanitized = sanitized.trim_matches('-').to_string();
        if sanitized.is_empty() {
            "custom-provider".into()
        } else {
            sanitized
        }
    }

    fn resolve_wire_api(profile: &ProfileConfig) -> Result<String> {
        let protocol = Self::platform_string(profile, "wire_api")
            .or_else(|| Self::platform_string(profile, "api_protocol"))
            .unwrap_or_else(|| "responses".into());

        let normalized = protocol.to_lowercase();
        if normalized == "responses" || normalized == "chat" {
            Ok(normalized)
        } else {
            Err(CcrError::ValidationError(format!(
                "wire_api 必须为 responses 或 chat，当前值: {protocol}"
            )))
        }
    }

    fn resolve_provider_id(name: &str, profile: &ProfileConfig) -> String {
        let candidate = Self::platform_string(profile, "provider_id")
            .or_else(|| profile.provider.clone())
            .unwrap_or_else(|| name.to_string());
        Self::sanitize_identifier(&candidate)
    }

    fn resolve_env_key(profile: &ProfileConfig, provider_id: &str) -> String {
        if let Some(key) = Self::platform_string(profile, "env_key") {
            return key;
        }

        provider_id
            .to_uppercase()
            .replace('-', "_")
            .trim()
            .to_string()
            + "_API_KEY"
    }

    fn apply_custom_profile(&self, name: &str, profile: &ProfileConfig) -> Result<()> {
        let config_path = Self::codex_config_path()?;
        let auth_path = Self::codex_auth_path()?;
        let codex_dir = config_path
            .parent()
            .ok_or_else(|| CcrError::ConfigError("无效的 Codex 配置目录".into()))?;

        fs::create_dir_all(codex_dir)
            .map_err(|e| CcrError::SettingsError(format!("创建 Codex 目录失败: {}", e)))?;

        let provider_id = Self::resolve_provider_id(name, profile);
        let provider_name = profile
            .description
            .clone()
            .or_else(|| profile.provider.clone())
            .unwrap_or_else(|| provider_id.clone());
        let base_url = profile.base_url.clone().unwrap_or_default();
        let wire_api = Self::resolve_wire_api(profile)?;
        let env_key = Self::resolve_env_key(profile, &provider_id);
        let requires_auth = Self::platform_bool(profile, "requires_openai_auth").unwrap_or(true);
        let provider_model =
            Self::platform_string(profile, "provider_model").or_else(|| profile.model.clone());
        let token = profile
            .auth_token
            .as_ref()
            .ok_or_else(|| {
                CcrError::ValidationError("Codex profile 缺少 auth_token (API key)".into())
            })?
            .clone();

        let mut lines = Vec::new();
        lines.push("# --- model provider managed by CCR ---".into());

        if let Some(model) = profile.model.as_ref() {
            lines.push(format!("model = \"{}\"", model));
        }

        lines.push(format!("model_provider = \"{}\"", provider_id));

        if let Some(policy) = Self::platform_string(profile, "approval_policy") {
            lines.push(format!("approval_policy = \"{}\"", policy));
        }

        if let Some(sandbox) = Self::platform_string(profile, "sandbox_mode") {
            lines.push(format!("sandbox_mode = \"{}\"", sandbox));
        }

        if let Some(reasoning) = Self::platform_string(profile, "model_reasoning_effort") {
            lines.push(format!("model_reasoning_effort = \"{}\"", reasoning));
        }

        lines.push(String::new());
        lines.push(format!("[model_providers.{}]", provider_id));
        lines.push(format!("name = \"{}\"", provider_name));
        lines.push(format!("base_url = \"{}\"", base_url));
        lines.push(format!("wire_api = \"{}\"", wire_api));
        lines.push(format!("env_key = \"{}\"", env_key));
        lines.push(format!("requires_openai_auth = {}", requires_auth));

        if let Some(model) = provider_model {
            lines.push(format!("model = \"{}\"", model));
        }

        lines.push(String::new());

        fs::write(&config_path, lines.join("\n"))
            .map_err(|e| CcrError::SettingsError(format!("写入 Codex config 失败: {}", e)))?;

        // 更新 auth.json
        let auth_entries = if auth_path.exists() {
            let content = fs::read_to_string(&auth_path).map_err(|e| {
                CcrError::SettingsError(format!("读取 Codex auth.json 失败: {}", e))
            })?;
            serde_json::from_str::<JsonMap<String, JsonValue>>(&content)
                .unwrap_or_else(|_| JsonMap::new())
        } else {
            JsonMap::new()
        };

        let mut merged = auth_entries;
        merged.insert(env_key.clone(), JsonValue::String(token.clone()));
        merged.insert("OPENAI_API_KEY".into(), JsonValue::String(token));

        let auth_content = serde_json::to_string_pretty(&JsonValue::Object(merged))
            .map_err(|e| CcrError::SettingsError(format!("序列化 auth.json 失败: {}", e)))?;

        fs::write(&auth_path, auth_content)
            .map_err(|e| CcrError::SettingsError(format!("写入 auth.json 失败: {}", e)))?;

        log::info!(
            "✅ 已写入 Codex config ({}) 并更新 auth.json",
            config_path.display()
        );
        Ok(())
    }

    /// 📋 从 TOML 文件加载 profiles
    fn load_profiles_from_file(&self) -> Result<IndexMap<String, ProfileConfig>> {
        if !self.paths.profiles_file.exists() {
            return Ok(IndexMap::new());
        }

        // 读取文件
        let content = fs::read_to_string(&self.paths.profiles_file)
            .map_err(|e| CcrError::ConfigError(format!("读取 Codex 配置失败: {}", e)))?;

        // 解析 TOML
        let profiles: IndexMap<String, ProfileConfig> = toml::from_str(&content)
            .map_err(|e| CcrError::ConfigFormatInvalid(format!("Codex 配置格式错误: {}", e)))?;

        Ok(profiles)
    }

    /// 💾 保存 profiles 到 TOML 文件
    fn save_profiles_to_file(&self, profiles: &IndexMap<String, ProfileConfig>) -> Result<()> {
        // 确保目录存在
        self.paths.ensure_directories()?;

        // 序列化为 TOML
        let content = toml::to_string_pretty(profiles)
            .map_err(|e| CcrError::ConfigError(format!("序列化 Codex 配置失败: {}", e)))?;

        // 写入文件
        fs::write(&self.paths.profiles_file, content)
            .map_err(|e| CcrError::ConfigError(format!("写入 Codex 配置失败: {}", e)))?;

        log::info!("✅ 已保存 Codex profiles: {:?}", self.paths.profiles_file);
        Ok(())
    }

    /// 📖 加载 Codex settings
    #[allow(dead_code)]
    fn load_settings(&self) -> Result<CodexSettings> {
        if !self.paths.settings_file.exists() {
            return Err(CcrError::SettingsMissing(
                self.paths.settings_file.display().to_string(),
            ));
        }

        let content = fs::read_to_string(&self.paths.settings_file)
            .map_err(|e| CcrError::SettingsError(format!("读取 Codex 设置失败: {}", e)))?;

        let settings: CodexSettings = serde_json::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("解析 Codex 设置失败: {}", e)))?;

        Ok(settings)
    }

    /// 💾 保存 Codex settings
    fn save_settings(&self, settings: &CodexSettings) -> Result<()> {
        // 确保目录存在
        self.paths.ensure_directories()?;

        // 序列化为 JSON
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| CcrError::SettingsError(format!("序列化 Codex 设置失败: {}", e)))?;

        // 写入文件
        fs::write(&self.paths.settings_file, content)
            .map_err(|e| CcrError::SettingsError(format!("写入 Codex 设置失败: {}", e)))?;

        log::info!("✅ 已保存 Codex settings: {:?}", self.paths.settings_file);
        Ok(())
    }

    /// 🔐 验证 GitHub Token 格式
    ///
    /// 支持的 token 格式：
    /// - `ghp_` - Personal Access Token
    /// - `gho_` - OAuth Token
    /// - `github_pat_` - Fine-grained Personal Access Token
    fn validate_github_token(token: &str) -> Result<()> {
        let valid_prefixes = ["ghp_", "gho_", "github_pat_"];

        if !valid_prefixes
            .iter()
            .any(|prefix| token.starts_with(prefix))
        {
            return Err(CcrError::ValidationError(format!(
                "无效的 GitHub token 格式，应以 {} 之一开头",
                valid_prefixes.join(", ")
            )));
        }

        if token.len() < 20 {
            return Err(CcrError::ValidationError("GitHub token 长度不足".into()));
        }

        Ok(())
    }

    /// 📋 从 ProfileConfig 提取 Codex 特定字段
    fn extract_codex_fields(profile: &ProfileConfig) -> Result<(String, String, Option<String>)> {
        let api_endpoint = profile
            .base_url
            .as_ref()
            .ok_or_else(|| CcrError::ValidationError("缺少 api_endpoint 字段".into()))?
            .clone();

        let token = profile
            .auth_token
            .as_ref()
            .ok_or_else(|| CcrError::ValidationError("缺少 token 字段".into()))?
            .clone();

        let organization = profile
            .platform_data
            .get("organization")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok((api_endpoint, token, organization))
    }
}

impl PlatformConfig for CodexPlatform {
    fn platform_name(&self) -> &str {
        "codex"
    }

    fn platform_type(&self) -> Platform {
        Platform::Codex
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
        Self::codex_config_path().unwrap_or_else(|_| self.paths.settings_file.clone())
    }

    fn apply_profile(&self, name: &str) -> Result<()> {
        // 加载 profile
        let profiles = self.load_profiles()?;
        let profile = profiles
            .get(name)
            .ok_or_else(|| CcrError::ProfileNotFound(name.to_string()))?;

        // 验证
        self.validate_profile(profile)?;

        if Self::is_github_profile(profile) {
            // GitHub Copilot CLI 兼容模式
            let (api_endpoint, token, organization) = Self::extract_codex_fields(profile)?;
            let settings = CodexSettings {
                github: GitHubConfig {
                    api_endpoint,
                    token,
                    organization,
                },
                model: profile.model.clone(),
            };
            self.save_settings(&settings)?;
        } else {
            // 自定义 Codex API (config.toml)
            self.apply_custom_profile(name, profile)?;
        }

        // 在 Unified 模式下，同步更新注册表中的 current_profile
        let platform_config_mgr = PlatformConfigManager::with_default()?;
        let mut unified_config = platform_config_mgr.load()?;

        // 更新 Codex 平台的 current_profile
        unified_config.set_platform_profile("codex", name)?;

        // 保存注册表
        platform_config_mgr.save(&unified_config)?;

        log::debug!("✅ 已更新注册表 current_profile: {}", name);

        log::info!("✅ 已应用 Codex profile: {}", name);
        Ok(())
    }

    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()> {
        // 检查 base_url
        let base_url = profile.base_url.as_ref().ok_or_else(|| {
            CcrError::ValidationError("Codex profile 缺少 base_url (api_endpoint)".into())
        })?;

        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            return Err(CcrError::ValidationError(
                "api_endpoint 必须以 http:// 或 https:// 开头".into(),
            ));
        }

        // 检查 auth_token
        let token = profile.auth_token.as_ref().ok_or_else(|| {
            CcrError::ValidationError("Codex profile 缺少 auth_token (API key/token)".into())
        })?;

        if Self::is_github_profile(profile) {
            Self::validate_github_token(token)?;
        } else if token.trim().is_empty() {
            return Err(CcrError::ValidationError(
                "Codex profile 缺少有效的 API key".into(),
            ));
        } else {
            // 自定义模式时需要验证 wire_api
            Self::resolve_wire_api(profile)?;
        }

        Ok(())
    }

    fn get_current_profile(&self) -> Result<Option<String>> {
        // Codex 在 Unified 模式下，从注册表读取 current_profile
        let platform_config_mgr = PlatformConfigManager::with_default()?;
        let unified_config = platform_config_mgr.load()?;

        // 获取 Codex 平台的注册信息
        let codex_entry = unified_config.get_platform("codex")?;
        Ok(codex_entry.current_profile.clone())
    }
}

impl Validatable for CodexSettings {
    fn validate(&self) -> Result<()> {
        // 验证 API endpoint
        if !self.github.api_endpoint.starts_with("http://")
            && !self.github.api_endpoint.starts_with("https://")
        {
            return Err(CcrError::ValidationError(
                "GitHub API endpoint 必须以 http:// 或 https:// 开头".into(),
            ));
        }

        // 验证 token
        CodexPlatform::validate_github_token(&self.github.token)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_github_token() {
        // 有效的 tokens
        assert!(CodexPlatform::validate_github_token("ghp_1234567890abcdefghij").is_ok());
        assert!(CodexPlatform::validate_github_token("gho_1234567890abcdefghij").is_ok());
        assert!(CodexPlatform::validate_github_token("github_pat_1234567890abcdefghij").is_ok());

        // 无效的 tokens
        assert!(CodexPlatform::validate_github_token("invalid_token").is_err());
        assert!(CodexPlatform::validate_github_token("ghp_short").is_err());
        assert!(CodexPlatform::validate_github_token("").is_err());
    }

    #[test]
    fn test_codex_settings_structure() {
        let settings = CodexSettings {
            github: GitHubConfig {
                api_endpoint: "https://api.github.com".to_string(),
                token: "ghp_1234567890abcdefghij".to_string(),
                organization: Some("my-org".to_string()),
            },
            model: Some("gpt-4".to_string()),
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_codex_platform_basic() {
        if let Ok(platform) = CodexPlatform::new() {
            assert_eq!(platform.platform_name(), "codex");
            assert_eq!(platform.platform_type(), Platform::Codex);
            assert!(
                platform
                    .get_settings_path()
                    .to_str()
                    .unwrap()
                    .contains("codex")
            );
        }
    }

    #[test]
    fn test_validate_profile_modes() {
        let platform = CodexPlatform::new().unwrap();

        // GitHub 兼容模式
        let github_profile = ProfileConfig {
            description: Some("GitHub Copilot".to_string()),
            base_url: Some("https://api.github.com".to_string()),
            auth_token: Some("ghp_1234567890abcdefghij".to_string()),
            model: Some("gpt-4".to_string()),
            small_fast_model: None,
            provider: Some("github".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            platform_data: IndexMap::new(),
        };
        assert!(platform.validate_profile(&github_profile).is_ok());

        // 自定义 API
        let mut custom_profile = ProfileConfig {
            description: Some("PackyCode".to_string()),
            base_url: Some("https://api.packyapi.com/v1".to_string()),
            auth_token: Some("sk-packy".to_string()),
            model: Some("gpt-4.1-mini".to_string()),
            small_fast_model: None,
            provider: Some("packycode".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            platform_data: IndexMap::new(),
        };
        custom_profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        assert!(platform.validate_profile(&custom_profile).is_ok());

        custom_profile
            .platform_data
            .insert("wire_api".into(), json!("invalid"));
        assert!(platform.validate_profile(&custom_profile).is_err());
    }

    #[test]
    fn test_apply_custom_profile_writes_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("PackyCode Relay".to_string()),
            base_url: Some("https://api.packyapi.com/v1".to_string()),
            auth_token: Some("sk-packy".to_string()),
            model: Some("gpt-4.1-mini".to_string()),
            small_fast_model: None,
            provider: Some("packycode".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("env_key".into(), json!("PACKYCODE_API_KEY"));

        platform.apply_custom_profile("packy", &profile).unwrap();

        let config_path = temp_dir.path().join("config.toml");
        let auth_path = temp_dir.path().join("auth.json");

        let config_content = fs::read_to_string(config_path).expect("config.toml exists");
        assert!(config_content.contains("model_provider = \"packycode\""));
        assert!(config_content.contains("[model_providers.packycode]"));

        let auth_content = fs::read_to_string(auth_path).expect("auth.json exists");
        assert!(auth_content.contains("PACKYCODE_API_KEY"));

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }
}
