//! Existing Codex typed settings model and patch policy extracted from Tauri.
//! Wire DTOs stay in Tauri. Parsed documents can contain credentials; no Debug.
use crate::config::profile_patch::fields::{parse_bool_field, parse_string_field};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::HashMap;

/// Default Codex model. The first builtin entry stays this id.
pub const DEFAULT_MODEL: &str = "gpt-6.1-sol";

/// Local configured catalog, not a remote capability claim. Older ids stay so
/// existing profiles are still recognized as builtins.
pub const BUILTIN_MODELS: &[&str] = &[
    DEFAULT_MODEL,
    "gpt-6-astra",
    "gpt-6.1-terra",
    "gpt-6-sol",
    "gpt-6-luna",
    "gpt-5.6-sol",
    "gpt-5.6-terra",
    "gpt-5.6-luna",
];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomModelsFile {
    #[serde(default)]
    pub models: Vec<String>,
}

pub fn builtin_models() -> Vec<String> {
    BUILTIN_MODELS
        .iter()
        .map(|model| (*model).to_string())
        .collect()
}
pub fn normalize_model_name(model: &str) -> Option<String> {
    let model = model.trim();
    (!model.is_empty()).then(|| model.to_string())
}
pub fn sanitize_custom_models(models: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for model in models {
        if let Some(model) = normalize_model_name(&model)
            && !BUILTIN_MODELS.contains(&model.as_str())
            && !result.contains(&model)
        {
            result.push(model);
        }
    }
    result
}
pub fn merge_models(custom: &[String]) -> Vec<String> {
    let mut models = builtin_models();
    for model in custom {
        if !models.contains(model) {
            models.push(model.clone());
        }
    }
    models
}

pub fn save_patch(
    document: &super::Document,
    snapshot: &super::Snapshot,
    patch: &Value,
) -> Result<super::SaveOutcome, super::Error> {
    let mut config = if snapshot.exists {
        toml::from_str::<CodexConfig>(&snapshot.content).map_err(|_| invalid())?
    } else {
        CodexConfig::default()
    };
    apply_codex_settings_update(&mut config, patch).map_err(|_| invalid())?;
    let content = toml::to_string_pretty(&config).map_err(|_| invalid())?;
    document.save_validated(&content, snapshot)
}

fn invalid() -> super::Error {
    super::Error::Invalid {
        line: None,
        column: None,
    }
}

/// 读取 ~/.codex/config.toml 的轻量代理（仅包含需要的字段）
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CodexConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_verbosity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_context_window: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_auto_compact_token_limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_response_storage: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_workspace_write: Option<CodexSandboxWorkspaceWriteConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_environment_policy: Option<CodexShellEnvironmentPolicyConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_opener: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub developer_instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<CodexToolsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tui: Option<CodexTuiConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_agent_reasoning: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_raw_agent_reasoning: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_for_update_on_startup: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_unstable_features_warning: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_use_rmcp_client: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<CodexHistoryConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<CodexAnalyticsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<CodexFeedbackConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, CodexMcpServer>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<HashMap<String, CodexProfile>>,
    /// 保留所有未知字段
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CodexMcpServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_vars: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_timeout_sec: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_timeout_sec: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_http_headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_token_env_var: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_resource: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CodexProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CodexSandboxWorkspaceWriteConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writable_roots: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_access: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CodexShellEnvironmentPolicyConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_only: Option<Vec<String>>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CodexToolsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_image: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum CodexTuiNotificationsConfig {
    Enabled(bool),
    Events(Vec<String>),
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CodexTuiConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_screen: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications: Option<CodexTuiNotificationsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_tooltips: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CodexHistoryConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<i64>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CodexAnalyticsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CodexFeedbackConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

pub fn parse_tui_notifications_field(
    raw: &Value,
) -> Result<Option<CodexTuiNotificationsConfig>, String> {
    match raw {
        Value::Null => Ok(None),
        Value::Bool(flag) => Ok(Some(CodexTuiNotificationsConfig::Enabled(*flag))),
        Value::Array(items) => {
            let mut events = Vec::new();
            for item in items {
                let Value::String(event) = item else {
                    return Err("字段 'tui.notifications' 必须是字符串数组".to_string());
                };
                let trimmed = event.trim();
                if !trimmed.is_empty() {
                    events.push(trimmed.to_string());
                }
            }
            Ok(Some(CodexTuiNotificationsConfig::Events(events)))
        }
        _ => Err("字段 'tui.notifications' 必须是布尔值或字符串数组".to_string()),
    }
}

pub fn parse_i64_field(raw: &Value, field_name: &str) -> Result<Option<i64>, String> {
    match raw {
        Value::Null => Ok(None),
        Value::Number(number) => number
            .as_i64()
            .map(Some)
            .ok_or_else(|| format!("字段 '{field_name}' 必须是整数")),
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                trimmed
                    .parse::<i64>()
                    .map(Some)
                    .map_err(|_| format!("字段 '{field_name}' 必须是整数"))
            }
        }
        _ => Err(format!("字段 '{field_name}' 必须是整数")),
    }
}

pub fn parse_u64_field(raw: &Value, field_name: &str) -> Result<Option<u64>, String> {
    let parsed = parse_i64_field(raw, field_name)?;
    match parsed {
        Some(value) if value < 0 => Err(format!("字段 '{field_name}' 不能为负数")),
        Some(value) => Ok(Some(value as u64)),
        None => Ok(None),
    }
}

pub fn parse_string_array_field(
    raw: &Value,
    field_name: &str,
) -> Result<Option<Vec<String>>, String> {
    match raw {
        Value::Null => Ok(None),
        Value::String(text) => {
            let items: Vec<String> = text
                .split(',')
                .map(|item| item.trim())
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect();
            if items.is_empty() {
                Ok(None)
            } else {
                Ok(Some(items))
            }
        }
        Value::Array(items) => {
            let mut values = Vec::new();
            for item in items {
                let Value::String(value) = item else {
                    return Err(format!("字段 '{field_name}' 必须是字符串数组"));
                };
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    values.push(trimmed.to_string());
                }
            }
            if values.is_empty() {
                Ok(None)
            } else {
                Ok(Some(values))
            }
        }
        _ => Err(format!("字段 '{field_name}' 必须是字符串数组")),
    }
}

pub fn parse_string_map_field(
    raw: &Value,
    field_name: &str,
) -> Result<Option<HashMap<String, String>>, String> {
    match raw {
        Value::Null => Ok(None),
        Value::Object(items) => {
            let mut values = HashMap::new();
            for (key, value) in items {
                let parsed = parse_string_field(value, &format!("{field_name}.{key}"))?;
                if let Some(text) = parsed {
                    values.insert(key.clone(), text);
                }
            }
            if values.is_empty() {
                Ok(None)
            } else {
                Ok(Some(values))
            }
        }
        _ => Err(format!("字段 '{field_name}' 必须是对象")),
    }
}

pub fn parse_bool_map_field(
    raw: &Value,
    field_name: &str,
) -> Result<Option<HashMap<String, bool>>, String> {
    match raw {
        Value::Null => Ok(None),
        Value::Object(items) => {
            let mut values = HashMap::new();
            for (key, value) in items {
                let parsed = parse_bool_field(value, &format!("{field_name}.{key}"))?;
                if let Some(flag) = parsed {
                    values.insert(key.clone(), flag);
                }
            }
            if values.is_empty() {
                Ok(None)
            } else {
                Ok(Some(values))
            }
        }
        _ => Err(format!("字段 '{field_name}' 必须是对象")),
    }
}

pub fn apply_optional_string_setting(
    target: &mut Option<String>,
    settings: &Map<String, Value>,
    field_name: &str,
) -> Result<(), String> {
    if let Some(raw) = settings.get(field_name) {
        *target = parse_string_field(raw, field_name)?;
    }
    Ok(())
}

pub fn apply_optional_bool_setting(
    target: &mut Option<bool>,
    settings: &Map<String, Value>,
    field_name: &str,
) -> Result<(), String> {
    if let Some(raw) = settings.get(field_name) {
        *target = parse_bool_field(raw, field_name)?;
    }
    Ok(())
}

pub fn apply_optional_i64_setting(
    target: &mut Option<i64>,
    settings: &Map<String, Value>,
    field_name: &str,
) -> Result<(), String> {
    if let Some(raw) = settings.get(field_name) {
        *target = parse_i64_field(raw, field_name)?;
    }
    Ok(())
}

pub fn apply_sandbox_workspace_write_setting(
    config: &mut CodexConfig,
    raw: &Value,
) -> Result<(), String> {
    let Value::Object(obj) = raw else {
        return Err("字段 'sandbox_workspace_write' 必须是对象".to_string());
    };

    let nested = config
        .sandbox_workspace_write
        .get_or_insert_with(CodexSandboxWorkspaceWriteConfig::default);
    if let Some(value) = obj.get("writable_roots") {
        nested.writable_roots =
            parse_string_array_field(value, "sandbox_workspace_write.writable_roots")?;
    }
    if let Some(value) = obj.get("network_access") {
        nested.network_access = parse_bool_field(value, "sandbox_workspace_write.network_access")?;
    }

    if nested.writable_roots.is_none() && nested.network_access.is_none() && nested.other.is_empty()
    {
        config.sandbox_workspace_write = None;
    }

    Ok(())
}

pub fn apply_shell_environment_policy_setting(
    config: &mut CodexConfig,
    raw: &Value,
) -> Result<(), String> {
    let Value::Object(obj) = raw else {
        return Err("字段 'shell_environment_policy' 必须是对象".to_string());
    };

    let nested = config
        .shell_environment_policy
        .get_or_insert_with(CodexShellEnvironmentPolicyConfig::default);
    if let Some(value) = obj.get("include_only") {
        nested.include_only =
            parse_string_array_field(value, "shell_environment_policy.include_only")?;
    }

    if nested.include_only.is_none() && nested.other.is_empty() {
        config.shell_environment_policy = None;
    }

    Ok(())
}

pub fn apply_tools_setting(config: &mut CodexConfig, raw: &Value) -> Result<(), String> {
    let Value::Object(obj) = raw else {
        return Err("字段 'tools' 必须是对象".to_string());
    };

    let nested = config.tools.get_or_insert_with(CodexToolsConfig::default);
    if let Some(value) = obj.get("view_image") {
        nested.view_image = parse_bool_field(value, "tools.view_image")?;
    }
    if let Some(value) = obj.get("web_search") {
        nested.web_search = parse_bool_field(value, "tools.web_search")?;
    }

    if nested.view_image.is_none() && nested.web_search.is_none() && nested.other.is_empty() {
        config.tools = None;
    }

    Ok(())
}

pub fn apply_tui_setting(config: &mut CodexConfig, raw: &Value) -> Result<(), String> {
    let Value::Object(obj) = raw else {
        return Err("字段 'tui' 必须是对象".to_string());
    };

    let nested = config.tui.get_or_insert_with(CodexTuiConfig::default);
    if let Some(value) = obj.get("alternate_screen") {
        nested.alternate_screen = parse_string_field(value, "tui.alternate_screen")?;
    }
    if let Some(value) = obj.get("animations") {
        nested.animations = parse_bool_field(value, "tui.animations")?;
    }
    if let Some(value) = obj.get("notifications") {
        nested.notifications = parse_tui_notifications_field(value)?;
    }
    if let Some(value) = obj.get("show_tooltips") {
        nested.show_tooltips = parse_bool_field(value, "tui.show_tooltips")?;
    }

    if nested.alternate_screen.is_none()
        && nested.animations.is_none()
        && nested.notifications.is_none()
        && nested.show_tooltips.is_none()
        && nested.other.is_empty()
    {
        config.tui = None;
    }

    Ok(())
}

pub fn apply_history_setting(config: &mut CodexConfig, raw: &Value) -> Result<(), String> {
    let Value::Object(obj) = raw else {
        return Err("字段 'history' 必须是对象".to_string());
    };

    let nested = config
        .history
        .get_or_insert_with(CodexHistoryConfig::default);
    if let Some(value) = obj.get("persistence") {
        nested.persistence = parse_string_field(value, "history.persistence")?;
    }
    if let Some(value) = obj.get("max_bytes") {
        nested.max_bytes = parse_i64_field(value, "history.max_bytes")?;
    }

    if nested.persistence.is_none() && nested.max_bytes.is_none() && nested.other.is_empty() {
        config.history = None;
    }

    Ok(())
}

pub fn apply_analytics_setting(config: &mut CodexConfig, raw: &Value) -> Result<(), String> {
    let Value::Object(obj) = raw else {
        return Err("字段 'analytics' 必须是对象".to_string());
    };

    let nested = config
        .analytics
        .get_or_insert_with(CodexAnalyticsConfig::default);
    if let Some(value) = obj.get("enabled") {
        nested.enabled = parse_bool_field(value, "analytics.enabled")?;
    }

    if nested.enabled.is_none() && nested.other.is_empty() {
        config.analytics = None;
    }

    Ok(())
}

pub fn apply_feedback_setting(config: &mut CodexConfig, raw: &Value) -> Result<(), String> {
    let Value::Object(obj) = raw else {
        return Err("字段 'feedback' 必须是对象".to_string());
    };

    let nested = config
        .feedback
        .get_or_insert_with(CodexFeedbackConfig::default);
    if let Some(value) = obj.get("enabled") {
        nested.enabled = parse_bool_field(value, "feedback.enabled")?;
    }

    if nested.enabled.is_none() && nested.other.is_empty() {
        config.feedback = None;
    }

    Ok(())
}

pub fn codex_settings_to_json(config: &CodexConfig) -> Value {
    json!({
        "model": config.model,
        "model_provider": config.model_provider,
        "model_reasoning_effort": config.model_reasoning_effort,
        "model_reasoning_summary": config.model_reasoning_summary,
        "model_verbosity": config.model_verbosity,
        "model_context_window": config.model_context_window,
        "model_auto_compact_token_limit": config.model_auto_compact_token_limit,
        "personality": config.personality,
        "approval_policy": config.approval_policy,
        "sandbox_mode": config.sandbox_mode,
        "disable_response_storage": config.disable_response_storage,
        "sandbox_workspace_write": config.sandbox_workspace_write.as_ref().map(|nested| json!({
            "writable_roots": nested.writable_roots,
            "network_access": nested.network_access,
        })),
        "shell_environment_policy": config.shell_environment_policy.as_ref().map(|nested| json!({
            "include_only": nested.include_only,
        })),
        "web_search": config.web_search,
        "file_opener": config.file_opener,
        "developer_instructions": config.developer_instructions,
        "instructions": config.instructions,
        "tools": config.tools.as_ref().map(|nested| json!({
            "view_image": nested.view_image,
            "web_search": nested.web_search,
        })),
        "tui": config.tui.as_ref().map(|nested| json!({
            "alternate_screen": nested.alternate_screen,
            "animations": nested.animations,
            "notifications": nested.notifications,
            "show_tooltips": nested.show_tooltips,
        })),
        "hide_agent_reasoning": config.hide_agent_reasoning,
        "show_raw_agent_reasoning": config.show_raw_agent_reasoning,
        "check_for_update_on_startup": config.check_for_update_on_startup,
        "suppress_unstable_features_warning": config.suppress_unstable_features_warning,
        "experimental_use_rmcp_client": config.experimental_use_rmcp_client,
        "history": config.history.as_ref().map(|nested| json!({
            "persistence": nested.persistence,
            "max_bytes": nested.max_bytes,
        })),
        "analytics": config.analytics.as_ref().map(|nested| json!({
            "enabled": nested.enabled,
        })),
        "feedback": config.feedback.as_ref().map(|nested| json!({
            "enabled": nested.enabled,
        })),
        "features": config.features,
    })
}

pub fn apply_codex_settings_update(
    config: &mut CodexConfig,
    settings: &Value,
) -> Result<(), String> {
    let obj = settings
        .as_object()
        .ok_or_else(|| "settings 必须是对象".to_string())?;

    apply_optional_string_setting(&mut config.model, obj, "model")?;
    apply_optional_string_setting(&mut config.model_provider, obj, "model_provider")?;
    apply_optional_string_setting(
        &mut config.model_reasoning_effort,
        obj,
        "model_reasoning_effort",
    )?;
    apply_optional_string_setting(
        &mut config.model_reasoning_summary,
        obj,
        "model_reasoning_summary",
    )?;
    apply_optional_string_setting(&mut config.model_verbosity, obj, "model_verbosity")?;
    apply_optional_i64_setting(
        &mut config.model_context_window,
        obj,
        "model_context_window",
    )?;
    apply_optional_i64_setting(
        &mut config.model_auto_compact_token_limit,
        obj,
        "model_auto_compact_token_limit",
    )?;
    apply_optional_string_setting(&mut config.personality, obj, "personality")?;
    apply_optional_string_setting(&mut config.approval_policy, obj, "approval_policy")?;
    apply_optional_string_setting(&mut config.sandbox_mode, obj, "sandbox_mode")?;
    apply_optional_bool_setting(
        &mut config.disable_response_storage,
        obj,
        "disable_response_storage",
    )?;
    apply_optional_string_setting(&mut config.web_search, obj, "web_search")?;
    apply_optional_string_setting(&mut config.file_opener, obj, "file_opener")?;
    apply_optional_string_setting(
        &mut config.developer_instructions,
        obj,
        "developer_instructions",
    )?;
    apply_optional_string_setting(&mut config.instructions, obj, "instructions")?;
    apply_optional_bool_setting(
        &mut config.hide_agent_reasoning,
        obj,
        "hide_agent_reasoning",
    )?;
    apply_optional_bool_setting(
        &mut config.show_raw_agent_reasoning,
        obj,
        "show_raw_agent_reasoning",
    )?;
    apply_optional_bool_setting(
        &mut config.check_for_update_on_startup,
        obj,
        "check_for_update_on_startup",
    )?;
    apply_optional_bool_setting(
        &mut config.suppress_unstable_features_warning,
        obj,
        "suppress_unstable_features_warning",
    )?;
    apply_optional_bool_setting(
        &mut config.experimental_use_rmcp_client,
        obj,
        "experimental_use_rmcp_client",
    )?;

    if let Some(raw) = obj.get("sandbox_workspace_write") {
        if raw.is_null() {
            config.sandbox_workspace_write = None;
        } else {
            apply_sandbox_workspace_write_setting(config, raw)?;
        }
    }
    if let Some(raw) = obj.get("shell_environment_policy") {
        if raw.is_null() {
            config.shell_environment_policy = None;
        } else {
            apply_shell_environment_policy_setting(config, raw)?;
        }
    }
    if let Some(raw) = obj.get("tools") {
        if raw.is_null() {
            config.tools = None;
        } else {
            apply_tools_setting(config, raw)?;
        }
    }
    if let Some(raw) = obj.get("tui") {
        if raw.is_null() {
            config.tui = None;
        } else {
            apply_tui_setting(config, raw)?;
        }
    }
    if let Some(raw) = obj.get("history") {
        if raw.is_null() {
            config.history = None;
        } else {
            apply_history_setting(config, raw)?;
        }
    }
    if let Some(raw) = obj.get("analytics") {
        if raw.is_null() {
            config.analytics = None;
        } else {
            apply_analytics_setting(config, raw)?;
        }
    }
    if let Some(raw) = obj.get("feedback") {
        if raw.is_null() {
            config.feedback = None;
        } else {
            apply_feedback_setting(config, raw)?;
        }
    }
    if let Some(raw) = obj.get("features") {
        config.features = parse_bool_map_field(raw, "features")?;
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::platform_settings::{Document, Format, SaveOutcome};
    #[test]
    fn original_revision_dirty_leaves_preserve_unknown_toml_and_conflict() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.toml");
        let initial = "model = 'before'\ncustom = ['keep']\n[tui]\nnotifications = ['agent-turn-complete']\ncustom = 'keep'\n[profiles.retained]\nmodel = 'profile'\n[mcp_servers.test]\ncommand = 'fixture'\n";
        std::fs::write(&path, initial).unwrap();
        let document = Document::new(
            path.clone(),
            None,
            ccr_core::BackupPolicy::None,
            Some(temp.path().join("locks")),
            Format::Toml,
        );
        let original = document.read().unwrap();
        assert!(matches!(
            save_patch(
                &document,
                &original,
                &json!({"model":"after", "tui":{"animations":false}})
            ),
            Ok(SaveOutcome::Saved { .. })
        ));
        let saved: toml::Value = toml::from_str(&document.read().unwrap().content).unwrap();
        assert_eq!(saved["model"].as_str(), Some("after"));
        assert_eq!(
            saved["custom"].as_array().unwrap()[0].as_str(),
            Some("keep")
        );
        assert_eq!(saved["tui"]["custom"].as_str(), Some("keep"));
        assert_eq!(
            saved["tui"]["notifications"].as_array().unwrap()[0].as_str(),
            Some("agent-turn-complete")
        );
        assert_eq!(
            saved["profiles"]["retained"]["model"].as_str(),
            Some("profile")
        );
        assert_eq!(
            saved["mcp_servers"]["test"]["command"].as_str(),
            Some("fixture")
        );
        assert_eq!(
            save_patch(&document, &original, &json!({"model":"stale"})).unwrap(),
            SaveOutcome::Conflict
        );
        let fresh = document.read().unwrap();
        assert!(matches!(
            save_patch(&document, &fresh, &json!({"model":null})),
            Ok(SaveOutcome::Saved { .. })
        ));
        let saved: toml::Value = toml::from_str(&document.read().unwrap().content).unwrap();
        assert!(saved.get("model").is_none());
    }
}
