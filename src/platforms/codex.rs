// 💻 Codex Platform 实现
// 📦 Codex CLI 平台配置管理
//
// 核心职责:
// - 📋 管理 Codex profiles
// - ⚙️ 操作 Codex config.toml / auth.json
// - 🔄 两路分发: Official (完全重置) / ThirdParty (read-modify-write)
// - 💾 仅支持 Unified 模式

use crate::core::error::{CcrError, Result};
use crate::managers::codex_config::CodexConfigManager;
use crate::models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use crate::platforms::base;
use indexmap::IndexMap;
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use tracing::debug;

/// 💻 Codex Platform 实现
///
/// ## 配置文件
/// - Profiles: `~/.ccr/platforms/codex/profiles.toml`
/// - Config: `~/.codex/config.toml`
/// - Auth: `~/.codex/auth.json`
///
/// ## 分发模式
/// - **Official**: 完全重置 config.toml + auth.json 到默认状态
/// - **ThirdParty**: read-modify-write，仅更新 provider 相关字段
pub struct CodexPlatform {
    paths: PlatformPaths,
    config_manager: CodexConfigManager,
}

impl CodexPlatform {
    /// 🏗️ 创建新的 Codex Platform 实例
    pub fn new() -> Result<Self> {
        let paths = PlatformPaths::new(Platform::Codex)?;
        let config_manager = CodexConfigManager::with_default()?;
        Ok(Self {
            paths,
            config_manager,
        })
    }

    // ═══════════════════════════════════════════════════════════
    // 🔍 Profile 分类方法
    // ═══════════════════════════════════════════════════════════

    /// 🔍 判断是否为官方配置
    ///
    /// 优先检查 provider_type 字段，回退检查 base_url
    fn is_official_profile(profile: &ProfileConfig) -> bool {
        // provider_type 优先
        if let Some(pt) = &profile.provider_type {
            return pt == "official_relay";
        }
        // 回退：无 base_url 视为官方
        profile
            .base_url
            .as_ref()
            .is_none_or(|url| url.trim().is_empty())
    }

    /// 🔍 检查是否为已弃用的 GitHub 模式
    fn is_legacy_github_profile(profile: &ProfileConfig) -> bool {
        Self::platform_string(profile, "api_mode")
            .is_some_and(|mode| mode.eq_ignore_ascii_case("github"))
    }

    // ═══════════════════════════════════════════════════════════
    // 🔧 平台数据辅助方法
    // ═══════════════════════════════════════════════════════════

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

    fn ensure_toml_table(
        value: &mut toml::Value,
    ) -> Result<&mut toml::map::Map<String, toml::Value>> {
        if !matches!(value, toml::Value::Table(_)) {
            *value = toml::Value::Table(toml::map::Map::new());
        }
        value
            .as_table_mut()
            .ok_or_else(|| CcrError::ConfigError("TOML table expected".into()))
    }

    // ═══════════════════════════════════════════════════════════
    // 🏛️ 官方模式 - 完全重置
    // ═══════════════════════════════════════════════════════════

    /// 🏛️ 应用官方配置（备份后完全重置）
    fn apply_official_profile(&self) -> Result<()> {
        self.config_manager.reset_to_defaults("pre_official")?;
        tracing::info!("✅ 已切换到 Codex 官方配置（完全重置）");
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════
    // 🔧 第三方模式 - read-modify-write
    // ═══════════════════════════════════════════════════════════

    /// 🔧 应用第三方配置（read-modify-write，保留非 provider 字段）
    fn apply_third_party_profile(&self, name: &str, profile: &ProfileConfig) -> Result<()> {
        let provider_id = Self::resolve_provider_id(name, profile);
        let provider_name = profile
            .description
            .clone()
            .or_else(|| profile.provider.clone())
            .unwrap_or_else(|| provider_id.clone());
        let base_url = profile.base_url.clone().unwrap_or_else(|| {
            debug!("Codex profile {} 未配置 base_url，使用空字符串", name);
            String::new()
        });
        let wire_api = Self::resolve_wire_api(profile)?;
        let requires_auth = Self::platform_bool(profile, "requires_openai_auth").unwrap_or(true);
        let env_key = Self::platform_string(profile, "env_key");
        // provider_model: 仅当用户在 platform_data 中显式设置时才写入 provider 表
        // 不回退到 profile.model，避免 model 在 root 和 provider 表中重复
        let provider_model = Self::platform_string(profile, "provider_model");
        let token = profile
            .auth_token
            .as_ref()
            .ok_or_else(|| {
                CcrError::ValidationError("Codex profile 缺少 auth_token (API key)".into())
            })?
            .clone();

        // 读取现有配置（保留所有非 provider 字段）
        let mut config = self.config_manager.load_config()?;
        let root = Self::ensure_toml_table(&mut config)?;

        // 仅修改 provider 相关字段
        if let Some(model) = profile.model.as_ref() {
            root.insert("model".into(), toml::Value::String(model.clone()));
        }

        root.insert(
            "model_provider".into(),
            toml::Value::String(provider_id.clone()),
        );

        // 可选设置（来自 platform_data）
        if let Some(policy) = Self::platform_string(profile, "approval_policy") {
            root.insert("approval_policy".into(), toml::Value::String(policy));
        }

        if let Some(sandbox) = Self::platform_string(profile, "sandbox_mode") {
            root.insert("sandbox_mode".into(), toml::Value::String(sandbox));
        }

        if let Some(reasoning) = Self::platform_string(profile, "model_reasoning_effort") {
            root.insert(
                "model_reasoning_effort".into(),
                toml::Value::String(reasoning),
            );
        }

        if let Some(network_access) = Self::platform_string(profile, "network_access") {
            root.insert("network_access".into(), toml::Value::String(network_access));
        }

        if let Some(disable_response_storage) =
            Self::platform_bool(profile, "disable_response_storage")
        {
            root.insert(
                "disable_response_storage".into(),
                toml::Value::Boolean(disable_response_storage),
            );
        }

        // 更新 model_providers 表
        let providers_value = root
            .entry("model_providers")
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
        let providers_table = Self::ensure_toml_table(providers_value)?;

        let provider_value = providers_table
            .entry(provider_id.clone())
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
        let provider_table = Self::ensure_toml_table(provider_value)?;

        provider_table.insert("name".into(), toml::Value::String(provider_name));
        provider_table.insert("base_url".into(), toml::Value::String(base_url));
        provider_table.insert("wire_api".into(), toml::Value::String(wire_api));
        provider_table.insert(
            "requires_openai_auth".into(),
            toml::Value::Boolean(requires_auth),
        );

        // env_key: 告诉 Codex 从哪个环境变量读取 API key（如 MISTRAL_API_KEY）
        // 原理说明：
        // 官方原生流程推荐用户在系统的环境变量中配置此 key。
        // 但 CCR 采用一种更优雅的「免环境污染」方案：我们不仅在此定义 env_key，
        // 后续还会将这个 token 也就是 API Key 直接注入到底层的 auth.json 中，
        // 键名采用此处的 env_key 名称。Codex CLI 在遇到自定义 Provider 时，可以成功从 auth.json 匹配该名称。
        if let Some(ref ek) = env_key {
            provider_table.insert("env_key".into(), toml::Value::String(ek.clone()));
        } else {
            // 切换到未设置 env_key 的 profile 时，清理历史残留
            provider_table.remove("env_key");
        }

        if let Some(model) = provider_model {
            provider_table.insert("model".into(), toml::Value::String(model));
        } else {
            // provider_model 是可选字段，未设置时应移除旧值
            provider_table.remove("model");
        }

        // 原子写入 config.toml
        self.config_manager.save_config_atomic(&config)?;

        // 更新 auth.json（read-modify-write）
        // 核心机制：
        // 1. 确定 auth key 名称：
        //    - requires_openai_auth=true 或无 env_key 时，强制复用 OpenAI 默认的 OPENAI_API_KEY
        //    - 否则使用上面配置的 env_key 值作为 JSON key（例如 "MISTRAL_API_KEY"）
        // 2. 将此键值原子性写入 auth.json：
        //    此操作绕过了环境变量的配置阶段，为系统实现了环境隔离。
        //    只要在 config.toml 的 [model_providers.<id>] 块指定了 env_key，
        //    并在 auth.json 配置同名的键，CodexCLI 的解析器仍能正确读取。
        let auth_key_name = if requires_auth {
            "OPENAI_API_KEY".to_string()
        } else {
            env_key
                .clone()
                .unwrap_or_else(|| "OPENAI_API_KEY".to_string())
        };

        let mut auth = self.config_manager.load_auth()?;
        auth.insert(auth_key_name, JsonValue::String(token));
        self.config_manager.save_auth_atomic(&auth)?;

        tracing::info!(
            "✅ 已写入 Codex config ({}) 并更新 auth.json",
            self.config_manager.config_path().display()
        );
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════
    // 📋 Profile 文件操作
    // ═══════════════════════════════════════════════════════════

    /// 📋 从 TOML 文件加载 profiles
    fn load_profiles_from_file(&self) -> Result<IndexMap<String, ProfileConfig>> {
        base::load_profiles_from_toml(&self.paths.profiles_file)
    }

    /// 💾 保存 profiles 到 TOML 文件
    fn save_profiles_to_file(&self, profiles: &IndexMap<String, ProfileConfig>) -> Result<()> {
        base::save_profiles_to_toml(&self.paths.profiles_file, profiles, "codex", &self.paths)
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
        self.config_manager.config_path().to_path_buf()
    }

    fn apply_profile(&self, name: &str) -> Result<()> {
        // 加载 profile
        let profiles = self.load_profiles()?;
        let profile = profiles
            .get(name)
            .ok_or_else(|| CcrError::ProfileNotFound(name.to_string()))?;

        // 验证
        self.validate_profile(profile)?;

        // 两路分发: Official / ThirdParty
        if Self::is_official_profile(profile) {
            self.apply_official_profile()?;
        } else {
            self.apply_third_party_profile(name, profile)?;
        }

        // 更新 profiles.toml 中的 current_config
        base::update_current_config(&self.paths.profiles_file, name)?;

        // 更新注册表 current_profile
        base::update_registry_current_profile("codex", name)?;

        tracing::info!("✅ 已应用 Codex profile: {}", name);
        Ok(())
    }

    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()> {
        // 向后兼容：api_mode=github 返回明确弃用错误
        if Self::is_legacy_github_profile(profile) {
            return Err(CcrError::ValidationError(
                "GitHub 模式 (api_mode=github) 已弃用，请使用第三方模式 (wire_api=responses/chat) 替代".into(),
            ));
        }

        // 官方配置允许 base_url 和 auth_token 为空
        if Self::is_official_profile(profile) {
            return Ok(());
        }

        // 第三方配置：检查 base_url
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

        if token.trim().is_empty() {
            return Err(CcrError::ValidationError(
                "Codex profile 缺少有效的 API key".into(),
            ));
        }

        // 验证 wire_api
        Self::resolve_wire_api(profile)?;

        Ok(())
    }

    fn get_current_profile(&self) -> Result<Option<String>> {
        base::get_current_profile_from_registry("codex")
    }

    fn get_env_var_names(&self) -> Vec<String> {
        vec![]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    // ═══════════════════════════════════════════════════════════
    // 🔍 Profile 分类测试
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_provider_type_classification() {
        // provider_type = official_relay → 官方
        let official = ProfileConfig {
            description: Some("Official".into()),
            base_url: None,
            auth_token: None,
            model: None,
            small_fast_model: None,
            provider: None,
            provider_type: Some("official_relay".into()),
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        assert!(CodexPlatform::is_official_profile(&official));

        // provider_type = third_party → 非官方
        let third_party = ProfileConfig {
            provider_type: Some("third_party".into()),
            base_url: Some("https://api.example.com".into()),
            ..official.clone()
        };
        assert!(!CodexPlatform::is_official_profile(&third_party));

        // 无 provider_type，无 base_url → 官方 (回退)
        let no_type_no_url = ProfileConfig {
            provider_type: None,
            base_url: None,
            ..official.clone()
        };
        assert!(CodexPlatform::is_official_profile(&no_type_no_url));

        // 无 provider_type，有 base_url → 非官方 (回退)
        let no_type_with_url = ProfileConfig {
            provider_type: None,
            base_url: Some("https://api.example.com".into()),
            ..official.clone()
        };
        assert!(!CodexPlatform::is_official_profile(&no_type_with_url));

        // 空 base_url → 官方
        let empty_url = ProfileConfig {
            provider_type: None,
            base_url: Some("  ".into()),
            ..official.clone()
        };
        assert!(CodexPlatform::is_official_profile(&empty_url));
    }

    // ═══════════════════════════════════════════════════════════
    // ✅ 验证测试
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_codex_platform_basic() {
        if let Ok(platform) = CodexPlatform::new() {
            assert_eq!(platform.platform_name(), "codex");
            assert_eq!(platform.platform_type(), Platform::Codex);
            let settings_path = platform.get_settings_path();
            assert!(
                settings_path
                    .file_name()
                    .map(|n| n.to_string_lossy() == "config.toml")
                    .unwrap_or(false),
                "settings path should point to config.toml, got {:?}",
                settings_path
            );
        }
    }

    #[test]
    fn test_validate_profile_modes() {
        let platform = CodexPlatform::new().unwrap();

        // 第三方 API (wire_api=responses)
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
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        custom_profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        assert!(platform.validate_profile(&custom_profile).is_ok());

        // wire_api 无效值
        custom_profile
            .platform_data
            .insert("wire_api".into(), json!("invalid"));
        assert!(platform.validate_profile(&custom_profile).is_err());
    }

    #[test]
    fn test_legacy_github_profile_error() {
        let platform = CodexPlatform::new().unwrap();

        let mut github_profile = ProfileConfig {
            description: Some("GitHub Legacy".to_string()),
            base_url: Some("https://api.github.com".to_string()),
            auth_token: Some("ghp_1234567890abcdefghij".to_string()),
            model: Some("gpt-4".to_string()),
            small_fast_model: None,
            provider: Some("github".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        github_profile
            .platform_data
            .insert("api_mode".into(), json!("github"));

        let result = platform.validate_profile(&github_profile);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("弃用"),
            "should mention deprecation, got: {}",
            err_msg
        );
    }

    // ═══════════════════════════════════════════════════════════
    // 💾 写入测试
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_official_resets_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }

        let config_manager = CodexConfigManager::with_default().unwrap();

        // 先写入一些数据
        let mut table = toml::map::Map::new();
        table.insert(
            "model_provider".into(),
            toml::Value::String("custom".into()),
        );
        config_manager
            .save_config_atomic(&toml::Value::Table(table))
            .unwrap();

        // 验证数据存在
        let loaded = config_manager.load_config().unwrap();
        assert!(loaded.get("model_provider").is_some());

        // 重置
        config_manager.reset_to_defaults("pre_official").unwrap();

        // 验证已重置
        let after = config_manager.load_config().unwrap();
        assert!(after.as_table().unwrap().is_empty());

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_third_party_preserves_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }

        let config_manager = CodexConfigManager::with_default().unwrap();

        // 先写入一些用户自定义字段
        let mut table = toml::map::Map::new();
        table.insert(
            "approval_policy".into(),
            toml::Value::String("unless-allow-listed".into()),
        );
        table.insert(
            "user_custom_field".into(),
            toml::Value::String("should_be_preserved".into()),
        );
        config_manager
            .save_config_atomic(&toml::Value::Table(table))
            .unwrap();

        // 模拟第三方 profile 应用 (read-modify-write)
        let mut config = config_manager.load_config().unwrap();
        let root = config.as_table_mut().unwrap();
        root.insert(
            "model_provider".into(),
            toml::Value::String("test-provider".into()),
        );
        config_manager.save_config_atomic(&config).unwrap();

        // 验证原有字段被保留
        let after = config_manager.load_config().unwrap();
        let after_table = after.as_table().unwrap();
        assert_eq!(
            after_table
                .get("user_custom_field")
                .and_then(|v| v.as_str()),
            Some("should_be_preserved")
        );
        assert_eq!(
            after_table.get("model_provider").and_then(|v| v.as_str()),
            Some("test-provider")
        );

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_apply_third_party_writes_config() {
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
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));

        let result = platform.apply_third_party_profile("packy", &profile);
        if result.is_ok() {
            let config_path = temp_dir.path().join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).unwrap();
                assert!(content.contains("packycode"));

                // Verify: model at root level
                let parsed: toml::Value = toml::from_str(&content).unwrap();
                let root = parsed.as_table().unwrap();
                assert_eq!(
                    root.get("model").and_then(|v| v.as_str()),
                    Some("gpt-4.1-mini"),
                    "model should be at root level"
                );

                // Verify: model NOT inside provider table (no provider_model set)
                let providers = root
                    .get("model_providers")
                    .and_then(|v| v.as_table())
                    .unwrap();
                let provider = providers
                    .get("packycode")
                    .and_then(|v| v.as_table())
                    .unwrap();
                assert!(
                    provider.get("model").is_none(),
                    "model should NOT be in provider table when provider_model is not set, got: {:?}",
                    provider
                );
            }

            // 检查 auth.json
            let auth_path = temp_dir.path().join("auth.json");
            if auth_path.exists() {
                let content = std::fs::read_to_string(&auth_path).unwrap();
                assert!(content.contains("sk-packy"));
            }
        }

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_third_party_writes_env_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Mistral Provider".to_string()),
            base_url: Some("https://api.mistral.ai/v1".to_string()),
            auth_token: Some("mistral-key-123".to_string()),
            model: Some("mistral-large".to_string()),
            small_fast_model: None,
            provider: Some("mistral".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("chat"));
        profile
            .platform_data
            .insert("env_key".into(), json!("MISTRAL_API_KEY"));
        profile
            .platform_data
            .insert("requires_openai_auth".into(), json!(false));

        let result = platform.apply_third_party_profile("mistral", &profile);
        if result.is_ok() {
            // 验证 config.toml 包含 env_key
            let config_path = temp_dir.path().join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).unwrap();
                assert!(
                    content.contains("env_key"),
                    "config.toml should contain env_key, got: {}",
                    content
                );
                assert!(
                    content.contains("MISTRAL_API_KEY"),
                    "env_key should be MISTRAL_API_KEY, got: {}",
                    content
                );
            }

            // 验证 auth.json 使用 MISTRAL_API_KEY 而非 OPENAI_API_KEY
            let auth_path = temp_dir.path().join("auth.json");
            if auth_path.exists() {
                let content = std::fs::read_to_string(&auth_path).unwrap();
                assert!(
                    content.contains("MISTRAL_API_KEY"),
                    "auth.json should use MISTRAL_API_KEY, got: {}",
                    content
                );
                assert!(
                    !content.contains("OPENAI_API_KEY"),
                    "auth.json should NOT use OPENAI_API_KEY when env_key is set, got: {}",
                    content
                );
            }
        }

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_third_party_default_auth_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("OpenAI Compatible".to_string()),
            base_url: Some("https://api.openai-proxy.com/v1".to_string()),
            auth_token: Some("sk-proxy-key".to_string()),
            model: None,
            small_fast_model: None,
            provider: Some("proxy".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        // 不设置 env_key，requires_openai_auth 默认 true

        let result = platform.apply_third_party_profile("proxy", &profile);
        if result.is_ok() {
            // 验证 auth.json 使用默认的 OPENAI_API_KEY
            let auth_path = temp_dir.path().join("auth.json");
            if auth_path.exists() {
                let content = std::fs::read_to_string(&auth_path).unwrap();
                assert!(
                    content.contains("OPENAI_API_KEY"),
                    "should default to OPENAI_API_KEY, got: {}",
                    content
                );
            }

            // 验证 config.toml 不包含 env_key（因为未设置）
            let config_path = temp_dir.path().join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).unwrap();
                assert!(
                    !content.contains("env_key"),
                    "should not write env_key when not provided, got: {}",
                    content
                );
            }
        }

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_provider_model_explicit_writes_to_provider_table() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Custom Provider".to_string()),
            base_url: Some("https://api.example.com/v1".to_string()),
            auth_token: Some("sk-test".to_string()),
            model: Some("gpt-4".to_string()),
            small_fast_model: None,
            provider: Some("custom".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        // Explicitly set provider_model — should appear in provider table
        profile
            .platform_data
            .insert("provider_model".into(), json!("custom-gpt-4-alias"));

        let result = platform.apply_third_party_profile("custom", &profile);
        if result.is_ok() {
            let config_path = temp_dir.path().join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).unwrap();
                let parsed: toml::Value = toml::from_str(&content).unwrap();
                let root = parsed.as_table().unwrap();

                // Root model should be profile.model
                assert_eq!(
                    root.get("model").and_then(|v| v.as_str()),
                    Some("gpt-4"),
                    "root model should be profile.model"
                );

                // Provider table model should be provider_model
                let providers = root
                    .get("model_providers")
                    .and_then(|v| v.as_table())
                    .unwrap();
                let provider = providers.get("custom").and_then(|v| v.as_table()).unwrap();
                assert_eq!(
                    provider.get("model").and_then(|v| v.as_str()),
                    Some("custom-gpt-4-alias"),
                    "provider table should have explicit provider_model"
                );
            }
        }

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_third_party_clears_stale_optional_provider_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }

        let config_manager = CodexConfigManager::with_default().unwrap();

        // 先写入旧版/历史遗留字段：env_key + provider_table.model
        let mut provider_table = toml::map::Map::new();
        provider_table.insert("name".into(), toml::Value::String("custom".into()));
        provider_table.insert(
            "base_url".into(),
            toml::Value::String("https://api.example.com/v1".into()),
        );
        provider_table.insert("wire_api".into(), toml::Value::String("responses".into()));
        provider_table.insert("requires_openai_auth".into(), toml::Value::Boolean(false));
        provider_table.insert("env_key".into(), toml::Value::String("OLD_API_KEY".into()));
        provider_table.insert(
            "model".into(),
            toml::Value::String("old-provider-model".into()),
        );

        let mut providers_table = toml::map::Map::new();
        providers_table.insert("custom".into(), toml::Value::Table(provider_table));

        let mut root_table = toml::map::Map::new();
        root_table.insert(
            "model_provider".into(),
            toml::Value::String("custom".into()),
        );
        root_table.insert(
            "model_providers".into(),
            toml::Value::Table(providers_table),
        );
        config_manager
            .save_config_atomic(&toml::Value::Table(root_table))
            .unwrap();

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Custom Provider".to_string()),
            base_url: Some("https://api.example.com/v1".to_string()),
            auth_token: Some("sk-test".to_string()),
            model: Some("new-root-model".to_string()),
            small_fast_model: None,
            provider: Some("custom".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("requires_openai_auth".into(), json!(false));
        // 故意不设置 env_key / provider_model，验证旧字段会被清理

        platform
            .apply_third_party_profile("custom", &profile)
            .unwrap();

        let config = config_manager.load_config().unwrap();
        let root = config.as_table().unwrap();
        let providers = root
            .get("model_providers")
            .and_then(|v| v.as_table())
            .unwrap();
        let provider = providers.get("custom").and_then(|v| v.as_table()).unwrap();

        assert!(
            provider.get("env_key").is_none(),
            "stale env_key should be removed, got: {:?}",
            provider
        );
        assert!(
            provider.get("model").is_none(),
            "stale provider model should be removed, got: {:?}",
            provider
        );

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }
}
