//! Claude Code 命令 — MCP/Agents/Slash Commands/Plugins/Settings/OutputStyles/
//! Statusline/Hooks/Budgets/Prompts。
//!
//! 所有读写操作直接访问 ~/.claude/settings.json（通过 ccr::SettingsManager）和
//! ~/.claude.json（内联实现 ClaudeConfigManager 逻辑）。

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tauri::State;

use ccr::platforms::ClaudePlatform;
use ccr::services::ClaudeAuthService;
use ccr_config::{Platform, PlatformConfig, ProfileConfig};
use ccr_skills::{PromptPreset, PromptsManager};
use ccr_store::{BudgetManager, CostTracker};

use crate::platform::local::LocalEnvironment;
use crate::platform::{EnvError, ExecutionEnvironment};
use crate::state::AppState;

// ── 内联 ClaudeConfigManager（读写 ~/.claude.json 的 MCP 服务器）──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ClaudeConfig {
    #[serde(default, rename = "mcpServers")]
    mcp_servers: HashMap<String, McpServerEntry>,
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct McpServerEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub server_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

fn claude_json_path() -> std::io::Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "HOME/USERPROFILE environment variable not set",
            )
        })?;
    Ok(PathBuf::from(home).join(".claude.json"))
}

fn read_claude_config() -> std::io::Result<ClaudeConfig> {
    let path = claude_json_path()?;
    if !path.exists() {
        return Ok(ClaudeConfig::default());
    }
    let content = fs::read_to_string(&path)?;
    serde_json::from_str(&content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse .claude.json: {}", e),
        )
    })
}

fn write_claude_config(config: &ClaudeConfig) -> std::io::Result<()> {
    let path = claude_json_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    // Write via temp file for atomicity
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, &path)?;
    Ok(())
}

// ── Settings（~/.claude/settings.json）Helper ──
//
// 使用 ccr_types::ClaudeSettings（带有 agents/plugins/hooks 等 typed fields），
// 而非 ccr::ClaudeSettings（仅 env + other flatten）。

async fn active_environment(state: &AppState) -> Arc<dyn ExecutionEnvironment> {
    let registry = state.env_registry.read().await;
    registry
        .active()
        .unwrap_or_else(|| Arc::new(LocalEnvironment::new()))
}

async fn read_claude_settings_from_env(
    env: Arc<dyn ExecutionEnvironment>,
) -> Result<Value, String> {
    match env.read_config("claude", "settings.json").await {
        Ok(content) => {
            if content.trim().is_empty() {
                return Ok(json!({}));
            }

            let value: Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse Claude settings JSON: {e}"))?;
            if !value.is_object() {
                return Err("Claude settings must be a JSON object".to_string());
            }
            Ok(value)
        }
        Err(EnvError::ConfigNotFound(_)) => Ok(json!({})),
        Err(EnvError::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(err) => Err(format!(
            "Failed to read Claude settings from {}: {err}",
            env.display_name()
        )),
    }
}

async fn read_active_claude_settings_raw(state: &AppState) -> Result<Value, String> {
    let env = active_environment(state).await;
    read_claude_settings_from_env(env).await
}

async fn write_claude_settings_to_env(
    env: Arc<dyn ExecutionEnvironment>,
    settings: &Value,
) -> Result<(), String> {
    if !settings.is_object() {
        return Err("Claude settings must be a JSON object".to_string());
    }

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize Claude settings: {e}"))?;

    env.write_config("claude", "settings.json", &content)
        .await
        .map_err(|e| {
            format!(
                "Failed to write Claude settings to {}: {e}",
                env.display_name()
            )
        })
}

async fn write_active_claude_settings_raw(
    state: &AppState,
    settings: &Value,
) -> Result<(), String> {
    let env = active_environment(state).await;
    write_claude_settings_to_env(env, settings).await
}

fn merge_settings_patch(current: &mut Value, patch: Value) -> Result<(), String> {
    let current_obj = current
        .as_object_mut()
        .ok_or_else(|| "Current Claude settings must be a JSON object".to_string())?;
    let patch_obj = patch
        .as_object()
        .ok_or_else(|| "Claude settings patch must be a JSON object".to_string())?;

    for (key, value) in patch_obj {
        current_obj.insert(key.clone(), value.clone());
    }

    Ok(())
}

async fn load_settings(state: &AppState) -> Result<ccr_types::ClaudeSettings, String> {
    let raw = read_active_claude_settings_raw(state).await?;
    serde_json::from_value(raw).map_err(|e| format!("Failed to parse settings: {e}"))
}

async fn save_settings(
    state: &AppState,
    settings: &ccr_types::ClaudeSettings,
) -> Result<(), String> {
    let raw =
        serde_json::to_value(settings).map_err(|e| format!("Failed to serialize settings: {e}"))?;
    write_active_claude_settings_raw(state, &raw).await
}

// ── Output Styles directory ──

fn output_styles_dir() -> std::io::Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "HOME/USERPROFILE not set")
        })?;
    Ok(PathBuf::from(home).join(".claude").join("output-styles"))
}

// ═══════════════════════════════════════════════════════════
// ── 子模块导出 ──
// ═══════════════════════════════════════════════════════════

#[path = "claude_agents.rs"]
mod agents;
#[path = "claude_auth.rs"]
mod auth;
#[path = "claude_hooks.rs"]
mod hooks;
#[path = "claude_mcp.rs"]
mod mcp;
#[path = "claude_plugins.rs"]
mod plugins;
#[path = "claude_profiles.rs"]
mod profiles;
#[path = "claude_settings.rs"]
mod settings;
#[path = "claude_slash.rs"]
mod slash;

pub use agents::*;
pub use auth::*;
pub use hooks::*;
pub use mcp::*;
pub use plugins::*;
pub use profiles::*;
pub use settings::*;
pub use slash::*;

// ═══════════════════════════════════════════════════════════
// ── Output Styles（~/.claude/output-styles/*.md）──
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize)]
struct OutputStyle {
    name: String,
    content: String,
}

// ═══════════════════════════════════════════════════════════
// ── Profiles（CCR Core）──
// ═══════════════════════════════════════════════════════════

fn parse_string_field(raw: &Value, field_name: &str) -> Result<Option<String>, String> {
    match raw {
        Value::Null => Ok(None),
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        _ => Err(format!("字段 '{field_name}' 必须是字符串")),
    }
}

fn parse_tags_field(raw: &Value) -> Result<Option<Vec<String>>, String> {
    match raw {
        Value::Null => Ok(None),
        Value::String(text) => {
            let tags: Vec<String> = text
                .split(',')
                .map(|item| item.trim())
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect();
            if tags.is_empty() {
                Ok(None)
            } else {
                Ok(Some(tags))
            }
        }
        Value::Array(items) => {
            let mut tags: Vec<String> = Vec::new();
            for item in items {
                let Value::String(tag) = item else {
                    return Err("字段 'tags' 必须是字符串数组".to_string());
                };
                let trimmed = tag.trim();
                if !trimmed.is_empty() {
                    tags.push(trimmed.to_string());
                }
            }
            if tags.is_empty() {
                Ok(None)
            } else {
                Ok(Some(tags))
            }
        }
        _ => Err("字段 'tags' 必须是字符串或字符串数组".to_string()),
    }
}

fn parse_bool_field(raw: &Value, field_name: &str) -> Result<Option<bool>, String> {
    match raw {
        Value::Null => Ok(None),
        Value::Bool(flag) => Ok(Some(*flag)),
        _ => Err(format!("字段 '{field_name}' 必须是布尔值")),
    }
}

fn parse_usage_count_field(raw: &Value) -> Result<Option<u32>, String> {
    match raw {
        Value::Null => Ok(None),
        Value::Number(number) => {
            let value = number
                .as_u64()
                .ok_or_else(|| "字段 'usage_count' 必须是非负整数".to_string())?;
            let count =
                u32::try_from(value).map_err(|_| "字段 'usage_count' 超出范围".to_string())?;
            Ok(Some(count))
        }
        _ => Err("字段 'usage_count' 必须是数字".to_string()),
    }
}

fn parse_extra_field(
    raw: &Value,
    field_name: &str,
) -> Result<Option<serde_json::Map<String, Value>>, String> {
    match raw {
        Value::Null => Ok(None),
        Value::Object(map) => Ok(Some(map.clone())),
        _ => Err(format!("字段 '{field_name}' 必须是对象")),
    }
}

fn parse_platform_data_update(
    obj: &Map<String, Value>,
) -> Result<Option<Map<String, Value>>, String> {
    let has_extra = obj.contains_key("extra");
    let has_platform_data = obj.contains_key("platform_data");

    if !has_extra && !has_platform_data {
        return Ok(None);
    }

    let mut platform_data = Map::new();

    if let Some(raw) = obj.get("extra")
        && let Some(extra) = parse_extra_field(raw, "extra")?
    {
        platform_data.extend(extra);
    }

    if let Some(raw) = obj.get("platform_data")
        && let Some(extra) = parse_extra_field(raw, "platform_data")?
    {
        platform_data.extend(extra);
    }

    Ok(Some(platform_data))
}

fn build_profile_from_config(config: &Value) -> Result<ProfileConfig, String> {
    let obj = config
        .as_object()
        .ok_or_else(|| "profile config 必须是对象".to_string())?;

    let mut profile = ProfileConfig::new();
    patch_profile_with_config(&mut profile, config)?;

    if let Some(platform_data) = parse_platform_data_update(obj)? {
        profile.platform_data = platform_data.into_iter().collect();
    }

    if let Some(raw) = obj.get("auth_mode") {
        match raw {
            Value::Null => {
                profile
                    .platform_data
                    .shift_remove(ClaudePlatform::AUTH_MODE_FIELD);
            }
            Value::String(value) if !value.trim().is_empty() => {
                profile.platform_data.insert(
                    ClaudePlatform::AUTH_MODE_FIELD.to_string(),
                    Value::String(value.trim().to_string()),
                );
            }
            Value::String(_) => {
                return Err("字段 'auth_mode' 不能为空字符串".to_string());
            }
            _ => return Err("字段 'auth_mode' 必须是字符串".to_string()),
        }
    }

    Ok(profile)
}

fn patch_profile_with_config(profile: &mut ProfileConfig, config: &Value) -> Result<(), String> {
    let obj = config
        .as_object()
        .ok_or_else(|| "profile config 必须是对象".to_string())?;

    if let Some(raw) = obj.get("description") {
        profile.description = parse_string_field(raw, "description")?;
    }
    if let Some(raw) = obj.get("base_url") {
        profile.base_url = parse_string_field(raw, "base_url")?;
    }
    if let Some(raw) = obj.get("auth_token") {
        profile.auth_token = parse_string_field(raw, "auth_token")?;
    }
    if let Some(raw) = obj.get("model") {
        profile.model = parse_string_field(raw, "model")?;
    }
    if let Some(raw) = obj.get("small_fast_model") {
        profile.small_fast_model = parse_string_field(raw, "small_fast_model")?;
    }
    if let Some(raw) = obj.get("provider") {
        profile.provider = parse_string_field(raw, "provider")?;
    }
    if let Some(raw) = obj.get("provider_type") {
        profile.provider_type = parse_string_field(raw, "provider_type")?;
    }
    if let Some(raw) = obj.get("account") {
        profile.account = parse_string_field(raw, "account")?;
    }
    if let Some(raw) = obj.get("tags") {
        profile.tags = parse_tags_field(raw)?;
    }
    if let Some(raw) = obj.get("usage_count") {
        profile.usage_count = parse_usage_count_field(raw)?;
    }
    if let Some(raw) = obj.get("enabled") {
        profile.enabled = parse_bool_field(raw, "enabled")?;
    }

    if let Some(platform_data) = parse_platform_data_update(obj)? {
        profile.platform_data = platform_data.into_iter().collect();
    }

    if let Some(raw) = obj.get("auth_mode") {
        match raw {
            Value::Null => {
                profile
                    .platform_data
                    .shift_remove(ClaudePlatform::AUTH_MODE_FIELD);
            }
            Value::String(value) if !value.trim().is_empty() => {
                profile.platform_data.insert(
                    ClaudePlatform::AUTH_MODE_FIELD.to_string(),
                    Value::String(value.trim().to_string()),
                );
            }
            Value::String(_) => {
                return Err("字段 'auth_mode' 不能为空字符串".to_string());
            }
            _ => return Err("字段 'auth_mode' 必须是字符串".to_string()),
        }
    }

    Ok(())
}

fn profile_to_json(current_profile: Option<&str>, name: String, profile: ProfileConfig) -> Value {
    let is_current = current_profile == Some(name.as_str());
    let auth_mode = ClaudePlatform::profile_auth_mode(&profile);
    let auth_source = ClaudePlatform::profile_auth_source(&profile);
    let mut extra = profile.platform_data.clone();
    extra.shift_remove(ClaudePlatform::AUTH_MODE_FIELD);

    json!({
        "name": name,
        "description": profile.description,
        "base_url": profile.base_url,
        "auth_token": profile.auth_token,
        "model": profile.model,
        "small_fast_model": profile.small_fast_model,
        "provider": profile.provider,
        "provider_type": profile.provider_type,
        "account": profile.account,
        "tags": profile.tags,
        "usage_count": profile.usage_count,
        "enabled": profile.enabled,
        "platform_data": profile.platform_data,
        "auth_mode": auth_mode.as_str(),
        "auth_source": auth_source,
        "is_current": is_current,
        "extra": extra,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use crate::platform::{CliStatus, EnvironmentType, PlatformInfo};

    #[derive(Clone)]
    enum MockReadBehavior {
        Content(String),
        Missing,
        Error(String),
    }

    struct MockEnvironment {
        behavior: MockReadBehavior,
        writes: Mutex<Vec<String>>,
    }

    impl MockEnvironment {
        fn with_content(content: impl Into<String>) -> Self {
            Self {
                behavior: MockReadBehavior::Content(content.into()),
                writes: Mutex::new(Vec::new()),
            }
        }

        fn missing() -> Self {
            Self {
                behavior: MockReadBehavior::Missing,
                writes: Mutex::new(Vec::new()),
            }
        }

        fn error(message: impl Into<String>) -> Self {
            Self {
                behavior: MockReadBehavior::Error(message.into()),
                writes: Mutex::new(Vec::new()),
            }
        }

        fn last_written_json(&self) -> Option<Value> {
            let guard = self.writes.lock().unwrap();
            guard
                .last()
                .and_then(|content| serde_json::from_str::<Value>(content).ok())
        }
    }

    #[async_trait::async_trait]
    impl ExecutionEnvironment for MockEnvironment {
        fn env_type(&self) -> EnvironmentType {
            EnvironmentType::Local
        }

        fn display_name(&self) -> String {
            "Mock".to_string()
        }

        fn env_id(&self) -> String {
            "mock".to_string()
        }

        async fn list_platforms(&self) -> Result<Vec<PlatformInfo>, EnvError> {
            Ok(Vec::new())
        }

        async fn read_config(&self, _platform: &str, path: &str) -> Result<String, EnvError> {
            match &self.behavior {
                MockReadBehavior::Content(content) => Ok(content.clone()),
                MockReadBehavior::Missing => Err(EnvError::ConfigNotFound(path.to_string())),
                MockReadBehavior::Error(message) => Err(EnvError::Other(message.clone())),
            }
        }

        async fn write_config(
            &self,
            _platform: &str,
            _path: &str,
            content: &str,
        ) -> Result<(), EnvError> {
            self.writes.lock().unwrap().push(content.to_string());
            Ok(())
        }

        async fn detect_cli_status(&self) -> Result<Vec<CliStatus>, EnvError> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn patch_profile_with_config_clears_platform_data_when_extra_is_null() {
        let mut profile = ProfileConfig::new();
        profile
            .platform_data
            .insert("provider_model".into(), json!("claude-sonnet-4-5"));

        patch_profile_with_config(
            &mut profile,
            &json!({
                "extra": null
            }),
        )
        .unwrap();

        assert!(profile.platform_data.is_empty());
    }

    #[test]
    fn patch_profile_with_config_prefers_platform_data_over_extra() {
        let mut profile = ProfileConfig::new();

        patch_profile_with_config(
            &mut profile,
            &json!({
                "extra": {
                    "provider_model": "claude-3-5-sonnet",
                    "budget": "small"
                },
                "platform_data": {
                    "provider_model": "claude-sonnet-4-5",
                    "workspace": "team-a"
                }
            }),
        )
        .unwrap();

        assert_eq!(
            profile.platform_data.get("provider_model"),
            Some(&json!("claude-sonnet-4-5"))
        );
        assert_eq!(profile.platform_data.get("budget"), Some(&json!("small")));
        assert_eq!(
            profile.platform_data.get("workspace"),
            Some(&json!("team-a"))
        );
    }

    #[tokio::test]
    async fn read_claude_settings_from_env_reads_top_level_env() {
        let env = Arc::new(MockEnvironment::with_content(
            json!({
                "$schema": "https://example.com/schema",
                "env": {
                    "ANTHROPIC_BASE_URL": "https://example.com",
                    "ANTHROPIC_AUTH_TOKEN": "sk-test"
                },
                "statusLine": {
                    "type": "command",
                    "command": "echo ok"
                }
            })
            .to_string(),
        ));

        let result = read_claude_settings_from_env(env).await.unwrap();

        assert_eq!(
            result
                .get("env")
                .and_then(Value::as_object)
                .and_then(|env| env.get("ANTHROPIC_BASE_URL"))
                .and_then(Value::as_str),
            Some("https://example.com")
        );
        assert_eq!(
            result
                .get("statusLine")
                .and_then(Value::as_object)
                .is_some(),
            true
        );
    }

    #[tokio::test]
    async fn read_claude_settings_from_env_returns_empty_object_when_missing() {
        let env = Arc::new(MockEnvironment::missing());
        let result = read_claude_settings_from_env(env).await.unwrap();

        assert_eq!(result, json!({}));
    }

    #[tokio::test]
    async fn read_claude_settings_from_env_surfaces_non_missing_errors() {
        let env = Arc::new(MockEnvironment::error("permission denied"));
        let error = read_claude_settings_from_env(env).await.unwrap_err();

        assert!(error.contains("permission denied"));
    }

    #[test]
    fn merge_settings_patch_replaces_env_and_preserves_unknown_fields() {
        let mut current = json!({
            "$schema": "https://example.com/schema",
            "env": {
                "OLD_KEY": "old",
                "KEEP_ME": "stale"
            },
            "statusLine": {
                "type": "command",
                "command": "echo old"
            },
            "enabledPlugins": ["alpha"]
        });

        merge_settings_patch(
            &mut current,
            json!({
                "env": {
                    "NEW_KEY": "new"
                },
                "model": "claude-sonnet-4-5-20250929"
            }),
        )
        .unwrap();

        assert_eq!(current["env"], json!({ "NEW_KEY": "new" }));
        assert_eq!(
            current["statusLine"],
            json!({
                "type": "command",
                "command": "echo old"
            })
        );
        assert_eq!(current["enabledPlugins"], json!(["alpha"]));
        assert_eq!(current["$schema"], json!("https://example.com/schema"));
        assert_eq!(current["model"], json!("claude-sonnet-4-5-20250929"));
    }

    #[tokio::test]
    async fn typed_save_roundtrips_unknown_top_level_fields() {
        let env = Arc::new(MockEnvironment::with_content(
            json!({
                "$schema": "https://example.com/schema",
                "env": {
                    "ANTHROPIC_BASE_URL": "https://example.com"
                },
                "hooks": {
                    "UserPromptSubmit": [
                        {
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": "echo before"
                                }
                            ]
                        }
                    ]
                },
                "enabledPlugins": ["alpha"],
                "statusLine": {
                    "type": "command",
                    "command": "echo before"
                },
                "agents": []
            })
            .to_string(),
        ));

        let raw = read_claude_settings_from_env(env.clone()).await.unwrap();
        let mut settings: ccr_types::ClaudeSettings = serde_json::from_value(raw).unwrap();
        settings
            .env
            .insert("ANTHROPIC_AUTH_TOKEN".into(), "sk-test".into());
        settings.agents.push(ccr_types::Agent {
            name: "reviewer".into(),
            model: "opus".into(),
            tools: vec!["Read".into()],
            system_prompt: Some("check".into()),
            disabled: false,
            other: HashMap::new(),
        });

        let serialized = serde_json::to_value(settings).unwrap();
        write_claude_settings_to_env(env.clone(), &serialized)
            .await
            .unwrap();

        let written = env.last_written_json().unwrap();
        assert_eq!(written["$schema"], json!("https://example.com/schema"));
        assert_eq!(written["enabledPlugins"], json!(["alpha"]));
        assert_eq!(
            written["statusLine"],
            json!({
                "type": "command",
                "command": "echo before"
            })
        );
        assert_eq!(
            written["hooks"],
            json!({
                "UserPromptSubmit": [
                    {
                        "hooks": [
                            {
                                "type": "command",
                                "command": "echo before"
                            }
                        ]
                    }
                ]
            })
        );
        assert_eq!(written["env"]["ANTHROPIC_AUTH_TOKEN"], json!("sk-test"));
        assert_eq!(written["agents"][0]["name"], json!("reviewer"));
    }

    #[tokio::test]
    async fn claude_update_settings_accepts_official_grouped_hooks() {
        let env = Arc::new(MockEnvironment::with_content(
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://example.com",
                    "ANTHROPIC_AUTH_TOKEN": "sk-test"
                },
                "hooks": {
                    "PreToolUse": [
                        {
                            "matcher": "Bash",
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": "./security-check.sh"
                                }
                            ]
                        }
                    ]
                }
            })
            .to_string(),
        ));

        let raw = read_claude_settings_from_env(env.clone()).await.unwrap();
        let mut current = raw;
        merge_settings_patch(
            &mut current,
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://example.com",
                    "ANTHROPIC_AUTH_TOKEN": "sk-updated"
                }
            }),
        )
        .unwrap();

        let validated: ccr_types::ClaudeSettings = serde_json::from_value(current).unwrap();
        let serialized = serde_json::to_value(validated).unwrap();
        write_claude_settings_to_env(env.clone(), &serialized)
            .await
            .unwrap();

        let written = env.last_written_json().unwrap();
        assert_eq!(written["env"]["ANTHROPIC_AUTH_TOKEN"], json!("sk-updated"));
        assert_eq!(
            written["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            json!("./security-check.sh")
        );
    }
}
