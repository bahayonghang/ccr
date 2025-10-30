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

        // 提取 Codex 特定字段
        let (api_endpoint, token, organization) = Self::extract_codex_fields(profile)?;

        // 构建 settings
        let settings = CodexSettings {
            github: GitHubConfig {
                api_endpoint,
                token,
                organization,
            },
            model: profile.model.clone(),
        };

        // 保存 settings
        self.save_settings(&settings)?;

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
            CcrError::ValidationError("Codex profile 缺少 auth_token (GitHub token)".into())
        })?;

        Self::validate_github_token(token)?;

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
    fn test_validate_profile() {
        let platform = CodexPlatform::new().unwrap();

        // 有效的 profile
        let valid_profile = ProfileConfig {
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

        assert!(platform.validate_profile(&valid_profile).is_ok());

        // 无效的 profile（缺少 base_url）
        let mut invalid_profile = valid_profile.clone();
        invalid_profile.base_url = None;
        assert!(platform.validate_profile(&invalid_profile).is_err());

        // 无效的 profile（错误的 token 格式）
        let mut invalid_profile = valid_profile;
        invalid_profile.auth_token = Some("invalid_token".to_string());
        assert!(platform.validate_profile(&invalid_profile).is_err());
    }
}
