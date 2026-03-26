//! Codex 命令 — Profiles/Settings/MCP/Agents/Auth/Usage。
//!
//! 配置文件位置: `~/.codex/config.toml`
//! Agents 目录:  `~/.codex/agents/`
//! Profiles:     通过 `ccr::create_platform(Platform::Codex)` 管理
//! Auth:         通过 `ccr::services::CodexAuthService` 管理
//! Usage:        通过 `ccr::services::CodexUsageService` 管理

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::State;

use ccr::models::OpenAiAuthMethod;
use ccr::platforms::CodexPlatform;
use ccr::services::{CodexAuthService, CodexSessionService, CodexUsageService};
use ccr::{PlatformConfig, ProfileConfig};

use crate::state::{AppState, CacheFillRegistration};

// ── 内部辅助类型 ──

/// 读取 ~/.codex/config.toml 的轻量代理（仅包含需要的字段）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CodexConfig {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodexMcpServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodexProfile {
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CodexSandboxWorkspaceWriteConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writable_roots: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_access: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CodexShellEnvironmentPolicyConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_only: Option<Vec<String>>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CodexToolsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_image: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CodexTuiConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_screen: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_tooltips: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CodexHistoryConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<i64>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CodexAnalyticsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CodexFeedbackConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CodexCustomModelsFile {
    #[serde(default)]
    pub models: Vec<String>,
}

const CODEX_BUILTIN_MODELS: &[&str] = &["gpt-5.3-codex", "gpt-5.4"];
const EXPLICIT_PLATFORM_STRING_FIELDS: &[&str] = &[
    "api_mode",
    "wire_api",
    "env_key",
    "auth_mode",
    "openai_login_method",
    "approval_policy",
    "sandbox_mode",
    "model_reasoning_effort",
    "network_access",
];
const EXPLICIT_PLATFORM_BOOL_FIELDS: &[&str] =
    &["requires_openai_auth", "disable_response_storage"];

// ── 文件 I/O 辅助函数 ──

fn codex_config_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    Ok(home.join(".codex").join("config.toml"))
}

fn codex_agents_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    Ok(home.join(".codex").join("agents"))
}

fn codex_custom_models_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    Ok(home
        .join(".ccr")
        .join("platforms")
        .join("codex")
        .join("custom-models.toml"))
}

fn read_codex_custom_models(path: &PathBuf) -> Result<CodexCustomModelsFile, String> {
    if !path.exists() {
        return Ok(CodexCustomModelsFile::default());
    }
    let content =
        fs::read_to_string(path).map_err(|e| format!("读取 Codex 自定义模型失败: {e}"))?;
    toml::from_str(&content).map_err(|e| format!("解析 Codex 自定义模型失败: {e}"))
}

fn write_codex_custom_models(path: &PathBuf, models: &CodexCustomModelsFile) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建模型目录失败: {e}"))?;
    }
    let content =
        toml::to_string_pretty(models).map_err(|e| format!("序列化 Codex 自定义模型失败: {e}"))?;
    let parent = path.parent().ok_or("无效的文件路径")?;
    let tmp =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("创建临时文件失败: {e}"))?;
    fs::write(tmp.path(), &content).map_err(|e| format!("写入临时文件失败: {e}"))?;
    tmp.persist(path)
        .map_err(|e| format!("持久化模型文件失败: {e}"))?;
    Ok(())
}

fn normalize_model_name(model: &str) -> Option<String> {
    let trimmed = model.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn codex_builtin_models() -> Vec<String> {
    CODEX_BUILTIN_MODELS
        .iter()
        .map(|model| (*model).to_string())
        .collect()
}

fn merge_codex_models(custom_models: &[String]) -> Vec<String> {
    let mut merged = codex_builtin_models();
    for model in custom_models {
        if !merged.iter().any(|item| item == model) {
            merged.push(model.clone());
        }
    }
    merged
}

fn sanitize_custom_models(models: Vec<String>) -> Vec<String> {
    let mut sanitized = Vec::new();
    for model in models {
        if let Some(normalized) = normalize_model_name(&model)
            && !CODEX_BUILTIN_MODELS
                .iter()
                .any(|builtin| *builtin == normalized)
            && !sanitized.iter().any(|item| item == &normalized)
        {
            sanitized.push(normalized);
        }
    }
    sanitized
}

fn read_codex_model_catalog() -> Result<(Vec<String>, Vec<String>, Vec<String>), String> {
    let builtin_models = codex_builtin_models();
    let path = codex_custom_models_path()?;
    let custom_models = sanitize_custom_models(read_codex_custom_models(&path)?.models);
    let models = merge_codex_models(&custom_models);
    Ok((builtin_models, custom_models, models))
}

fn codex_list_models_payload() -> Result<Value, String> {
    let (builtin_models, custom_models, models) = read_codex_model_catalog()?;
    Ok(json!({
        "builtin_models": builtin_models,
        "custom_models": custom_models,
        "models": models,
    }))
}

fn count_codex_agents(path: &PathBuf) -> Result<usize, String> {
    if !path.exists() {
        return Ok(0);
    }

    let mut count = 0usize;
    for entry in fs::read_dir(path).map_err(|e| format!("读取 agents 目录失败: {e}"))? {
        let entry = entry.map_err(|e| format!("遍历 agents 目录失败: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("md") {
            count += 1;
        }
    }

    Ok(count)
}

fn usage_number(usage: &Value, section: &str, field: &str) -> u64 {
    usage
        .get(section)
        .and_then(|value| value.get(field))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn usage_datetime(usage: &Value, section: &str, field: &str) -> Option<DateTime<Utc>> {
    usage
        .get(section)
        .and_then(|value| value.get(field))
        .and_then(Value::as_str)
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn codex_usage_freshness(last_activity_at: Option<DateTime<Utc>>) -> (&'static str, &'static str) {
    let Some(last_activity_at) = last_activity_at else {
        return ("empty", "暂无使用记录");
    };

    let age = Utc::now().signed_duration_since(last_activity_at);
    if age <= chrono::Duration::hours(6) {
        ("fresh", "最近 6 小时内有使用记录")
    } else if age <= chrono::Duration::days(7) {
        ("stale", "最近 7 天内有使用记录")
    } else {
        ("old", "最近使用记录已较久")
    }
}

fn build_codex_usage_summary(usage: &Value) -> Value {
    let last_activity_at = ["five_hour", "seven_day", "all_time"]
        .into_iter()
        .filter_map(|section| usage_datetime(usage, section, "window_end"))
        .max();
    let (freshness, freshness_description) = codex_usage_freshness(last_activity_at);

    let top_model = usage
        .get("by_model")
        .and_then(Value::as_object)
        .and_then(|models| {
            models
                .iter()
                .max_by_key(|(_, stats)| {
                    stats
                        .get("total_requests")
                        .and_then(Value::as_u64)
                        .unwrap_or(0)
                })
                .map(|(model, stats)| {
                    json!({
                        "model": model,
                        "total_requests": stats.get("total_requests").and_then(Value::as_u64).unwrap_or(0),
                        "total_input_tokens": stats.get("total_input_tokens").and_then(Value::as_u64).unwrap_or(0),
                        "total_output_tokens": stats.get("total_output_tokens").and_then(Value::as_u64).unwrap_or(0),
                        "window_end": stats.get("window_end").cloned().unwrap_or(Value::Null),
                    })
                })
        });

    json!({
        "last_activity_at": last_activity_at.map(|dt| dt.to_rfc3339()),
        "freshness": freshness,
        "freshness_description": freshness_description,
        "five_hour": {
            "total_requests": usage_number(usage, "five_hour", "total_requests"),
            "total_input_tokens": usage_number(usage, "five_hour", "total_input_tokens"),
            "total_output_tokens": usage_number(usage, "five_hour", "total_output_tokens"),
        },
        "seven_day": {
            "total_requests": usage_number(usage, "seven_day", "total_requests"),
            "total_input_tokens": usage_number(usage, "seven_day", "total_input_tokens"),
            "total_output_tokens": usage_number(usage, "seven_day", "total_output_tokens"),
        },
        "all_time": {
            "total_requests": usage_number(usage, "all_time", "total_requests"),
            "total_input_tokens": usage_number(usage, "all_time", "total_input_tokens"),
            "total_output_tokens": usage_number(usage, "all_time", "total_output_tokens"),
        },
        "top_model": top_model,
    })
}

fn read_codex_config(path: &PathBuf) -> Result<CodexConfig, String> {
    if !path.exists() {
        return Ok(CodexConfig::default());
    }
    let content = fs::read_to_string(path).map_err(|e| format!("读取 Codex 配置失败: {e}"))?;
    toml::from_str(&content).map_err(|e| format!("解析 Codex 配置失败: {e}"))
}

fn write_codex_config(path: &PathBuf, config: &CodexConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {e}"))?;
    }
    let content =
        toml::to_string_pretty(config).map_err(|e| format!("序列化 Codex 配置失败: {e}"))?;
    // 原子写入: 写到同目录临时文件再 rename
    let parent = path.parent().ok_or("无效的文件路径")?;
    let tmp =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("创建临时文件失败: {e}"))?;
    fs::write(tmp.path(), &content).map_err(|e| format!("写入临时文件失败: {e}"))?;
    tmp.persist(path)
        .map_err(|e| format!("持久化配置文件失败: {e}"))?;
    Ok(())
}

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

fn parse_i64_field(raw: &Value, field_name: &str) -> Result<Option<i64>, String> {
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

fn parse_string_array_field(raw: &Value, field_name: &str) -> Result<Option<Vec<String>>, String> {
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

fn parse_bool_map_field(
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

fn apply_optional_string_setting(
    target: &mut Option<String>,
    settings: &Map<String, Value>,
    field_name: &str,
) -> Result<(), String> {
    if let Some(raw) = settings.get(field_name) {
        *target = parse_string_field(raw, field_name)?;
    }
    Ok(())
}

fn apply_optional_bool_setting(
    target: &mut Option<bool>,
    settings: &Map<String, Value>,
    field_name: &str,
) -> Result<(), String> {
    if let Some(raw) = settings.get(field_name) {
        *target = parse_bool_field(raw, field_name)?;
    }
    Ok(())
}

fn apply_optional_i64_setting(
    target: &mut Option<i64>,
    settings: &Map<String, Value>,
    field_name: &str,
) -> Result<(), String> {
    if let Some(raw) = settings.get(field_name) {
        *target = parse_i64_field(raw, field_name)?;
    }
    Ok(())
}

fn apply_sandbox_workspace_write_setting(
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

    if nested.writable_roots.is_none() && nested.network_access.is_none() && nested.other.is_empty() {
        config.sandbox_workspace_write = None;
    }

    Ok(())
}

fn apply_shell_environment_policy_setting(
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

fn apply_tools_setting(config: &mut CodexConfig, raw: &Value) -> Result<(), String> {
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

fn apply_tui_setting(config: &mut CodexConfig, raw: &Value) -> Result<(), String> {
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
        nested.notifications = parse_bool_field(value, "tui.notifications")?;
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

fn apply_history_setting(config: &mut CodexConfig, raw: &Value) -> Result<(), String> {
    let Value::Object(obj) = raw else {
        return Err("字段 'history' 必须是对象".to_string());
    };

    let nested = config.history.get_or_insert_with(CodexHistoryConfig::default);
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

fn apply_analytics_setting(config: &mut CodexConfig, raw: &Value) -> Result<(), String> {
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

fn apply_feedback_setting(config: &mut CodexConfig, raw: &Value) -> Result<(), String> {
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

fn codex_settings_to_json(config: &CodexConfig) -> Value {
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

fn apply_codex_settings_update(config: &mut CodexConfig, settings: &Value) -> Result<(), String> {
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
    apply_optional_i64_setting(&mut config.model_context_window, obj, "model_context_window")?;
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
    let has_explicit_platform_fields = EXPLICIT_PLATFORM_STRING_FIELDS
        .iter()
        .chain(EXPLICIT_PLATFORM_BOOL_FIELDS.iter())
        .any(|field| obj.contains_key(*field));

    if !has_extra && !has_platform_data && !has_explicit_platform_fields {
        return Ok(None);
    }

    let mut platform_data = Map::new();

    merge_explicit_platform_fields(&mut platform_data, obj)?;

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

fn merge_optional_string_field(
    platform_data: &mut Map<String, Value>,
    obj: &Map<String, Value>,
    field_name: &str,
) -> Result<(), String> {
    if let Some(raw) = obj.get(field_name)
        && let Some(value) = parse_string_field(raw, field_name)?
    {
        platform_data.insert(field_name.to_string(), Value::String(value));
    }

    Ok(())
}

fn merge_optional_bool_field(
    platform_data: &mut Map<String, Value>,
    obj: &Map<String, Value>,
    field_name: &str,
) -> Result<(), String> {
    if let Some(raw) = obj.get(field_name)
        && let Some(value) = parse_bool_field(raw, field_name)?
    {
        platform_data.insert(field_name.to_string(), Value::Bool(value));
    }

    Ok(())
}

fn merge_explicit_platform_fields(
    platform_data: &mut Map<String, Value>,
    obj: &Map<String, Value>,
) -> Result<(), String> {
    for field_name in EXPLICIT_PLATFORM_STRING_FIELDS {
        merge_optional_string_field(platform_data, obj, field_name)?;
    }

    for field_name in EXPLICIT_PLATFORM_BOOL_FIELDS {
        merge_optional_bool_field(platform_data, obj, field_name)?;
    }

    Ok(())
}

fn apply_profile_config(
    profile: &mut ProfileConfig,
    obj: &Map<String, Value>,
) -> Result<(), String> {
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

    Ok(())
}

fn build_profile_from_config(config: &Value) -> Result<ProfileConfig, String> {
    let obj = config
        .as_object()
        .ok_or_else(|| "profile config 必须是对象".to_string())?;

    let mut profile = ProfileConfig::new();
    apply_profile_config(&mut profile, obj)?;
    Ok(profile)
}

fn patch_profile_with_config(profile: &mut ProfileConfig, config: &Value) -> Result<(), String> {
    let obj = config
        .as_object()
        .ok_or_else(|| "profile config 必须是对象".to_string())?;

    apply_profile_config(profile, obj)
}

fn openai_login_method_to_string(method: OpenAiAuthMethod) -> &'static str {
    match method {
        OpenAiAuthMethod::Chatgpt => "chatgpt",
        OpenAiAuthMethod::Api => "api",
    }
}

fn profile_to_json(
    platform: &CodexPlatform,
    current_profile: Option<&str>,
    credential_store: Option<&str>,
    name: String,
    profile: ProfileConfig,
) -> Value {
    let is_current = current_profile == Some(name.as_str());
    let auth_mode = CodexPlatform::profile_auth_mode(&profile);
    let openai_login_method =
        CodexPlatform::profile_openai_login_method(&profile).map(openai_login_method_to_string);
    let mut extra = profile.platform_data.clone();
    for field_name in EXPLICIT_PLATFORM_STRING_FIELDS
        .iter()
        .chain(EXPLICIT_PLATFORM_BOOL_FIELDS.iter())
    {
        extra.shift_remove(*field_name);
    }

    let env_export = platform.export_profile_env(&name).ok();
    let shell_export_script = platform
        .export_profile_shell_script(&name)
        .ok()
        .filter(|script| !script.trim().is_empty());

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
        "wire_api": profile.platform_data.get("wire_api").cloned(),
        "env_key": profile.platform_data.get("env_key").cloned(),
        "requires_openai_auth": profile.platform_data.get("requires_openai_auth").cloned(),
        "approval_policy": profile.platform_data.get("approval_policy").cloned(),
        "sandbox_mode": profile.platform_data.get("sandbox_mode").cloned(),
        "model_reasoning_effort": profile.platform_data.get("model_reasoning_effort").cloned(),
        "network_access": profile.platform_data.get("network_access").cloned(),
        "disable_response_storage": profile.platform_data.get("disable_response_storage").cloned(),
        "auth_mode": auth_mode.as_str(),
        "openai_login_method": openai_login_method,
        "credential_store": if is_current { credential_store } else { None },
        "auth_source": CodexPlatform::profile_auth_source(&profile),
        "env_export": env_export,
        "shell_export_script": shell_export_script,
        "is_current": is_current,
        "extra": extra,
    })
}

// ── Profiles ──

/// 列出 CCR Codex profiles（~/.ccr/platforms/codex/profiles.toml）
#[tauri::command]
pub async fn codex_list_profiles() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Codex profile 失败: {e}"))?;
        let credential_store = CodexAuthService::new()
            .map(|service| service.get_auth_state().store.as_str().to_string())
            .ok();
        let profiles: Vec<Value> = platform
            .load_profiles()
            .map_err(|e| format!("读取 Codex profiles 失败: {e}"))?
            .into_iter()
            .map(|(name, profile)| {
                profile_to_json(
                    &platform,
                    current_profile.as_deref(),
                    credential_store.as_deref(),
                    name,
                    profile,
                )
            })
            .collect();

        Ok(json!({ "profiles": profiles, "current_profile": current_profile }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 列出 Codex 可选模型（内置 + 自定义）
#[tauri::command]
pub async fn codex_list_models() -> Result<Value, String> {
    tokio::task::spawn_blocking(codex_list_models_payload)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 保存 Codex 自定义模型
#[tauri::command]
pub async fn codex_add_custom_model(model: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let normalized =
            normalize_model_name(&model).ok_or_else(|| "模型名称不能为空".to_string())?;
        let path = codex_custom_models_path()?;
        let mut file = read_codex_custom_models(&path)?;
        let mut custom_models = sanitize_custom_models(std::mem::take(&mut file.models));
        if !custom_models.iter().any(|item| item == &normalized)
            && !CODEX_BUILTIN_MODELS
                .iter()
                .any(|builtin| *builtin == normalized)
        {
            custom_models.push(normalized.clone());
        }
        file.models = custom_models.clone();
        write_codex_custom_models(&path, &file)?;
        Ok(json!({
            "model": normalized,
            "models": merge_codex_models(&custom_models),
            "message": "Codex 自定义模型已保存",
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 新增 Codex profile（写入 CCR profiles.toml）
#[tauri::command]
pub async fn codex_add_profile(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Codex profiles 失败: {e}"))?;
        if profiles.contains_key(&name) {
            return Err(format!("Codex Profile '{name}' 已存在"));
        }

        let profile = build_profile_from_config(&config)?;
        platform
            .save_profile(&name, &profile)
            .map_err(|e| format!("保存 Codex Profile 失败: {e}"))?;

        Ok(json!({ "message": format!("Codex Profile '{name}' 已添加") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 更新 Codex profile（核心字段覆盖 + extra/platform_data 整体替换）
#[tauri::command]
pub async fn codex_update_profile(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Codex profiles 失败: {e}"))?;
        let mut profile = profiles
            .get(&name)
            .cloned()
            .ok_or_else(|| format!("Codex Profile '{name}' 不存在"))?;

        patch_profile_with_config(&mut profile, &config)?;
        platform
            .save_profile(&name, &profile)
            .map_err(|e| format!("更新 Codex Profile 失败: {e}"))?;

        Ok(json!({ "message": format!("Codex Profile '{name}' 已更新") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 删除 Codex profile
#[tauri::command]
pub async fn codex_delete_profile(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        platform
            .delete_profile(&name)
            .map_err(|e| format!("删除 Codex Profile 失败: {e}"))?;
        Ok(json!({ "message": format!("Codex Profile '{name}' 已删除") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 应用 Codex profile
#[tauri::command]
pub async fn codex_apply_profile(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        platform
            .apply_profile(&name)
            .map_err(|e| format!("应用 Codex Profile 失败: {e}"))?;
        Ok(json!({ "message": format!("Codex Profile '{name}' 已应用") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 获取 Codex profile 导出的环境变量与 shell 脚本
#[tauri::command]
pub async fn codex_get_profile_env(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        let env_export = platform
            .export_profile_env(&name)
            .map_err(|e| format!("导出 Codex Profile 环境变量失败: {e}"))?;
        let shell_export_script = platform
            .export_profile_shell_script(&name)
            .map_err(|e| format!("生成 Codex Profile shell 导出脚本失败: {e}"))?;

        Ok(json!({
            "name": name,
            "env_export": env_export,
            "shell_export_script": shell_export_script,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── Settings ──

/// 获取 Codex 完整配置（去掉 mcp_servers 和 profiles）
#[tauri::command]
pub async fn codex_get_settings() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let path = codex_config_path()?;
        let config = read_codex_config(&path)?;
        Ok(codex_settings_to_json(&config))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 更新 Codex 配置（合并写入，不覆盖 mcp_servers/profiles）
#[tauri::command]
pub async fn codex_update_settings(settings: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let path = codex_config_path()?;
        let mut config = read_codex_config(&path)?;
        apply_codex_settings_update(&mut config, &settings)?;

        write_codex_config(&path, &config)?;
        Ok(json!({ "message": "Codex 配置已更新" }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── MCP Servers ──

/// 列出 config.toml 中的 [mcp_servers]
#[tauri::command]
pub async fn codex_list_mcp_servers() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let path = codex_config_path()?;
        let config = read_codex_config(&path)?;
        let servers: Vec<Value> = config
            .mcp_servers
            .unwrap_or_default()
            .into_iter()
            .map(|(name, server)| {
                json!({
                    "name": name,
                    "command": server.command,
                    "args": server.args,
                    "env": server.env,
                    "cwd": server.cwd,
                    "startup_timeout_ms": server.startup_timeout_ms,
                    "url": server.url,
                    "bearer_token": server.bearer_token,
                })
            })
            .collect();
        Ok(json!({ "servers": servers }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 添加 MCP 服务器到 config.toml
#[tauri::command]
pub async fn codex_add_mcp_server(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let path = codex_config_path()?;
        let mut cfg = read_codex_config(&path)?;

        if let Some(ref servers) = cfg.mcp_servers
            && servers.contains_key(&name)
        {
            return Err(format!("MCP 服务器 '{name}' 已存在"));
        }

        let server = parse_mcp_server(&config)?;
        cfg.mcp_servers
            .get_or_insert_with(HashMap::new)
            .insert(name.clone(), server);

        write_codex_config(&path, &cfg)?;
        Ok(json!({ "message": format!("MCP 服务器 '{name}' 已添加") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 更新已有 MCP 服务器
#[tauri::command]
pub async fn codex_update_mcp_server(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let path = codex_config_path()?;
        let mut cfg = read_codex_config(&path)?;

        let servers = cfg
            .mcp_servers
            .as_mut()
            .ok_or_else(|| format!("MCP 服务器 '{name}' 不存在"))?;

        if !servers.contains_key(&name) {
            return Err(format!("MCP 服务器 '{name}' 不存在"));
        }

        let server = parse_mcp_server(&config)?;
        servers.insert(name.clone(), server);

        write_codex_config(&path, &cfg)?;
        Ok(json!({ "message": format!("MCP 服务器 '{name}' 已更新") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 删除 MCP 服务器
#[tauri::command]
pub async fn codex_delete_mcp_server(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let path = codex_config_path()?;
        let mut cfg = read_codex_config(&path)?;

        let servers = cfg
            .mcp_servers
            .as_mut()
            .ok_or_else(|| format!("MCP 服务器 '{name}' 不存在"))?;

        if servers.remove(&name).is_none() {
            return Err(format!("MCP 服务器 '{name}' 不存在"));
        }

        write_codex_config(&path, &cfg)?;
        Ok(format!("MCP 服务器 '{name}' 已删除"))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── Agents (markdown files in ~/.codex/agents/) ──

/// 列出 ~/.codex/agents/ 下的所有 agent markdown 文件
#[tauri::command]
pub async fn codex_list_agents() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let agents_dir = codex_agents_dir()?;
        if !agents_dir.exists() {
            return Ok(json!({ "agents": [] }));
        }

        let mut agents: Vec<Value> = Vec::new();
        for entry in fs::read_dir(&agents_dir).map_err(|e| format!("读取 agents 目录失败: {e}"))?
        {
            let entry = entry.map_err(|e| format!("遍历 agents 目录失败: {e}"))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("读取 agent 文件 '{name}' 失败: {e}"))?;
                let (description, body) = extract_frontmatter_description(&content);
                agents.push(json!({
                    "name": name,
                    "description": description,
                    "content": body,
                }));
            }
        }
        Ok(json!({ "agents": agents }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 添加新 agent（写入 ~/.codex/agents/{name}.md）
#[tauri::command]
pub async fn codex_add_agent(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let agents_dir = codex_agents_dir()?;
        fs::create_dir_all(&agents_dir).map_err(|e| format!("创建 agents 目录失败: {e}"))?;

        let file_path = agents_dir.join(format!("{name}.md"));
        if file_path.exists() {
            return Err(format!("Agent '{name}' 已存在"));
        }

        let content = build_agent_markdown(&config);
        fs::write(&file_path, &content).map_err(|e| format!("写入 agent '{name}' 失败: {e}"))?;

        Ok(json!({ "message": format!("Agent '{name}' 已添加") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 更新已有 agent
#[tauri::command]
pub async fn codex_update_agent(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let agents_dir = codex_agents_dir()?;
        let file_path = agents_dir.join(format!("{name}.md"));
        if !file_path.exists() {
            return Err(format!("Agent '{name}' 不存在"));
        }

        let content = build_agent_markdown(&config);
        fs::write(&file_path, &content).map_err(|e| format!("更新 agent '{name}' 失败: {e}"))?;

        Ok(json!({ "message": format!("Agent '{name}' 已更新") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 删除 agent
#[tauri::command]
pub async fn codex_delete_agent(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let agents_dir = codex_agents_dir()?;
        let file_path = agents_dir.join(format!("{name}.md"));
        if !file_path.exists() {
            return Err(format!("Agent '{name}' 不存在"));
        }

        fs::remove_file(&file_path).map_err(|e| format!("删除 agent '{name}' 失败: {e}"))?;

        Ok(format!("Agent '{name}' 已删除"))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── Sessions ──

#[tauri::command]
pub async fn codex_list_sessions(
    limit: Option<usize>,
    query: Option<String>,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let codex_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".codex");
        let service = CodexSessionService::new(codex_dir);
        let sessions = service
            .list_sessions(limit.unwrap_or(120).max(1), query.as_deref())
            .map_err(|e| format!("读取 Codex sessions 失败: {e}"))?;
        Ok(json!({ "sessions": sessions }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn codex_get_session_detail(
    file_path: String,
    message_limit: Option<usize>,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let codex_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".codex");
        let service = CodexSessionService::new(codex_dir);
        let detail = service
            .get_session_detail(PathBuf::from(file_path).as_path(), message_limit)
            .map_err(|e| format!("读取 Codex session 详情失败: {e}"))?;
        Ok(json!(detail))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn codex_export_session(
    file_path: String,
    max_messages: Option<usize>,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let codex_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".codex");
        let service = CodexSessionService::new(codex_dir);
        let export = service
            .export_session_markdown(PathBuf::from(file_path).as_path(), max_messages)
            .map_err(|e| format!("导出 Codex session 失败: {e}"))?;
        Ok(json!(export))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn codex_clone_session(file_path: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let codex_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".codex");
        let service = CodexSessionService::new(codex_dir);
        let session = service
            .clone_session(PathBuf::from(file_path).as_path())
            .map_err(|e| format!("克隆 Codex session 失败: {e}"))?;
        Ok(json!({
            "message": "Codex session 已克隆",
            "session": session,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn codex_delete_session(file_path: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let codex_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".codex");
        let service = CodexSessionService::new(codex_dir);
        let target_path = PathBuf::from(&file_path);
        service
            .delete_session(target_path.as_path())
            .map_err(|e| format!("删除 Codex session 失败: {e}"))?;
        Ok(json!({
            "message": "Codex session 已删除",
            "file_path": file_path,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── Auth 账号管理 ──

/// 列出所有 Codex Auth 账号
#[tauri::command]
pub async fn codex_list_auth_accounts() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let snapshot = service
            .read_auth_snapshot()
            .map_err(|e| format!("读取认证快照失败: {e}"))?;
        let accounts = service
            .build_account_items(&snapshot)
            .map_err(|e| format!("列出账号失败: {e}"))?;

        let accounts: Vec<Value> = accounts
            .into_iter()
            .map(|item| {
                let freshness = &item.freshness;
                let is_expired = CodexAuthService::is_expired(item.expires_at);
                json!({
                    "name": item.name,
                    "description": item.description,
                    "email": item.email,
                    "is_current": item.is_current,
                    "is_virtual": item.is_virtual,
                    "last_used": item.last_used.map(|dt| dt.to_rfc3339()),
                    "last_refresh": item.last_refresh.map(|dt| dt.to_rfc3339()),
                    "freshness": freshness,
                    "freshness_icon": freshness.icon(),
                    "freshness_description": freshness.description(),
                    "expires_at": item.expires_at.map(|dt| dt.to_rfc3339()),
                    "is_expired": is_expired,
                })
            })
            .collect();

        Ok(json!({ "accounts": accounts, "login_state": snapshot.login_state }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 获取当前 Codex Auth 信息
#[tauri::command]
pub async fn codex_get_auth_current() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let snapshot = service
            .read_auth_snapshot()
            .map_err(|e| format!("读取认证快照失败: {e}"))?;

        let info = match snapshot.current_info.as_ref() {
            Some(current) => {
                let freshness = &current.freshness;
                let expires_at = snapshot.current_expires_at;
                let is_expired = CodexAuthService::is_expired(expires_at);
                Some(json!({
                    "account_id": current.account_id,
                    "email": current.email,
                    "last_refresh": current.last_refresh.map(|dt| dt.to_rfc3339()),
                    "freshness": freshness,
                    "freshness_icon": freshness.icon(),
                    "freshness_description": freshness.description(),
                    "expires_at": expires_at.map(|dt| dt.to_rfc3339()),
                    "is_expired": is_expired,
                }))
            }
            None => None,
        };

        let logged_in = info.is_some();

        Ok(json!({
            "logged_in": logged_in,
            "info": info,
            "login_state": snapshot.login_state,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 保存当前登录到命名账号
#[tauri::command]
pub async fn codex_save_auth(
    name: String,
    description: Option<String>,
    expires_at: Option<String>,
    force: Option<bool>,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let parsed_expires_at = expires_at
            .as_deref()
            .map(|value| {
                DateTime::parse_from_rfc3339(value)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| format!("expires_at 必须是 RFC3339 时间: {e}"))
            })
            .transpose()?;

        service
            .save_current(
                &name,
                description,
                parsed_expires_at,
                force.unwrap_or(false),
            )
            .map_err(|e| format!("{e}"))?;

        Ok(json!({ "success": true, "message": format!("Codex Auth 账号 '{name}' 已成功保存") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 切换到指定账号
#[tauri::command]
pub async fn codex_switch_auth(name: String) -> Result<Value, String> {
    let name_resp = name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        service.switch_account(&name).map_err(|e| format!("{e}"))?;

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    Ok(json!({ "success": true, "message": format!("已切换到 Codex Auth 账号 '{name_resp}'") }))
}

/// 删除指定账号
#[tauri::command]
pub async fn codex_delete_auth(name: String) -> Result<Value, String> {
    let name_resp = name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        service.delete_account(&name).map_err(|e| format!("{e}"))?;

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    Ok(json!({ "success": true, "message": format!("Codex Auth 账号 '{name_resp}' 已成功删除") }))
}

/// 检测运行中的 Codex 进程
#[tauri::command]
pub async fn codex_detect_process() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let pids = service.detect_codex_process();
        let has_running_process = !pids.is_empty();

        let warning = if has_running_process {
            Some(format!(
                "检测到 {} 个运行中的 Codex 进程 (PID: {})，切换账号前请先关闭这些进程",
                pids.len(),
                pids.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        } else {
            None
        };

        Ok(json!({
            "has_running_process": has_running_process,
            "pids": pids,
            "warning": warning,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── Quota 配额查询 ──

/// 查询所有 Codex 账号的配额余额
#[tauri::command]
pub async fn codex_get_all_quotas() -> Result<Value, String> {
    let service =
        ccr::services::CodexQuotaService::new().map_err(|e| format!("初始化配额服务失败: {e}"))?;
    let quotas = service.fetch_all_quotas().await;
    serde_json::to_value(&quotas).map_err(|e| format!("序列化配额数据失败: {e}"))
}

/// 查询指定 Codex 账号的配额余额
#[tauri::command]
pub async fn codex_get_quota(account: String) -> Result<Value, String> {
    let service =
        ccr::services::CodexQuotaService::new().map_err(|e| format!("初始化配额服务失败: {e}"))?;
    let quota = service.fetch_account_quota(&account).await;
    serde_json::to_value(&quota).map_err(|e| format!("序列化配额数据失败: {e}"))
}

// ── Usage 统计 ──

fn build_codex_usage_payload(rolling: ccr::services::CodexRollingUsage) -> Value {
    let by_model: serde_json::Map<String, Value> = rolling
        .by_model
        .into_iter()
        .map(|(model, stats)| {
            (
                model,
                json!({
                    "total_input_tokens": stats.total_input_tokens,
                    "total_output_tokens": stats.total_output_tokens,
                    "total_requests": stats.total_requests,
                    "window_start": stats.window_start.map(|dt| dt.to_rfc3339()),
                    "window_end": stats.window_end.map(|dt| dt.to_rfc3339()),
                }),
            )
        })
        .collect();

    json!({
        "five_hour": {
            "total_input_tokens": rolling.five_hour.total_input_tokens,
            "total_output_tokens": rolling.five_hour.total_output_tokens,
            "total_requests": rolling.five_hour.total_requests,
            "window_start": rolling.five_hour.window_start.map(|dt| dt.to_rfc3339()),
            "window_end": rolling.five_hour.window_end.map(|dt| dt.to_rfc3339()),
        },
        "seven_day": {
            "total_input_tokens": rolling.seven_day.total_input_tokens,
            "total_output_tokens": rolling.seven_day.total_output_tokens,
            "total_requests": rolling.seven_day.total_requests,
            "window_start": rolling.seven_day.window_start.map(|dt| dt.to_rfc3339()),
            "window_end": rolling.seven_day.window_end.map(|dt| dt.to_rfc3339()),
        },
        "all_time": {
            "total_input_tokens": rolling.all_time.total_input_tokens,
            "total_output_tokens": rolling.all_time.total_output_tokens,
            "total_requests": rolling.all_time.total_requests,
            "window_start": rolling.all_time.window_start.map(|dt| dt.to_rfc3339()),
            "window_end": rolling.all_time.window_end.map(|dt| dt.to_rfc3339()),
        },
        "by_model": Value::Object(by_model),
    })
}

fn compute_codex_usage_payload() -> Result<Value, String> {
    let codex_dir = dirs::home_dir()
        .ok_or_else(|| "无法获取用户主目录".to_string())?
        .join(".codex");

    let service = CodexUsageService::new(codex_dir);
    let rolling = service
        .compute_rolling_usage()
        .map_err(|e| format!("计算使用量失败: {e}"))?;

    Ok(build_codex_usage_payload(rolling))
}

/// 获取 Codex 使用量统计
#[tauri::command]
pub async fn codex_get_usage(
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<Value, String> {
    let force = force.unwrap_or(false);
    let cache_key = "codex:rolling_usage";

    if !force {
        if let Some(cached) = state.cache_get(cache_key).await {
            return Ok(cached);
        }

        match state.begin_cache_fill(cache_key).await {
            CacheFillRegistration::Wait(notify) => {
                notify.notified().await;
                if let Some(cached) = state.cache_get(cache_key).await {
                    return Ok(cached);
                }
            }
            CacheFillRegistration::Leader => {
                let result = tokio::task::spawn_blocking(compute_codex_usage_payload)
                    .await
                    .map_err(|e| format!("任务执行失败: {e}"))?;

                match result {
                    Ok(payload) => {
                        state
                            .cache_set(cache_key.to_string(), payload.clone(), 30)
                            .await;
                        state.finish_cache_fill(cache_key).await;
                        return Ok(payload);
                    }
                    Err(error) => {
                        state.finish_cache_fill(cache_key).await;
                        return Err(error);
                    }
                }
            }
        }
    }

    tokio::task::spawn_blocking(compute_codex_usage_payload)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 获取 Codex 仪表盘摘要
#[tauri::command]
pub async fn codex_get_dashboard_summary() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let path = codex_config_path()?;
        let config = read_codex_config(&path)?;
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Codex profile 失败: {e}"))?;
        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Codex profiles 失败: {e}"))?;

        let auth_service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;
        let auth_state = auth_service.get_auth_state();
        let auth_snapshot = auth_service
            .read_auth_snapshot()
            .map_err(|e| format!("读取认证快照失败: {e}"))?;
        let auth_accounts = auth_service
            .build_account_items(&auth_snapshot)
            .map_err(|e| format!("列出账号失败: {e}"))?;

        let current_auth_name = auth_accounts
            .iter()
            .find(|item| item.is_current)
            .map(|item| item.name.clone());
        let expired_accounts = auth_accounts
            .iter()
            .filter(|item| CodexAuthService::is_expired(item.expires_at))
            .count();
        let current_auth = auth_snapshot.current_info.as_ref().map(|current| {
            let freshness = &current.freshness;
            let expires_at = auth_snapshot.current_expires_at;
            json!({
                "name": current_auth_name,
                "account_id": current.account_id,
                "email": current.email,
                "last_refresh": current.last_refresh.map(|dt| dt.to_rfc3339()),
                "freshness": freshness,
                "freshness_icon": freshness.icon(),
                "freshness_description": freshness.description(),
                "expires_at": expires_at.map(|dt| dt.to_rfc3339()),
                "is_expired": CodexAuthService::is_expired(expires_at),
            })
        });

        let current_profile_summary = current_profile.as_ref().and_then(|name| {
            profiles.get(name).cloned().map(|profile| {
                profile_to_json(
                    &platform,
                    current_profile.as_deref(),
                    Some(auth_state.store.as_str()),
                    name.clone(),
                    profile,
                )
            })
        });
        let enabled_profiles = profiles
            .values()
            .filter(|profile| profile.enabled.unwrap_or(true))
            .count();

        let usage_payload = compute_codex_usage_payload()?;
        let agents_count = count_codex_agents(&codex_agents_dir()?)?;
        let sessions_count = CodexSessionService::new(
            dirs::home_dir()
                .ok_or_else(|| "无法获取用户主目录".to_string())?
                .join(".codex"),
        )
        .count_sessions()
        .map_err(|e| format!("统计 Codex sessions 失败: {e}"))?;
        let mcp_servers_total = config
            .mcp_servers
            .as_ref()
            .map(|servers| servers.len())
            .unwrap_or(0);
        let config_profiles_total = config
            .profiles
            .as_ref()
            .map(|items| items.len())
            .unwrap_or(0);

        Ok(json!({
            "auth": {
                "logged_in": current_auth.is_some(),
                "login_state": auth_snapshot.login_state,
                "store": auth_state.store.as_str(),
                "saved_accounts_total": auth_accounts.len(),
                "expired_accounts_total": expired_accounts,
                "current": current_auth,
            },
            "profiles": {
                "current_profile": current_profile,
                "total": profiles.len(),
                "enabled_total": enabled_profiles,
                "disabled_total": profiles.len().saturating_sub(enabled_profiles),
                "current": current_profile_summary,
            },
            "config": {
                "model": config.model,
                "model_provider": config.model_provider,
                "approval_policy": config.approval_policy,
                "sandbox_mode": config.sandbox_mode,
                "model_reasoning_effort": config.model_reasoning_effort,
                "model_reasoning_summary": config.model_reasoning_summary,
                "web_search": config.web_search,
                "disable_response_storage": config.disable_response_storage,
            },
            "usage": build_codex_usage_summary(&usage_payload),
            "inventory": {
                "mcp_servers_total": mcp_servers_total,
                "agents_total": agents_count,
                "sessions_total": sessions_count,
                "config_profiles_total": config_profiles_total,
            }
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── 私有辅助函数 ──

/// 从 JSON Value 解析 CodexMcpServer
fn parse_mcp_server(v: &Value) -> Result<CodexMcpServer, String> {
    Ok(CodexMcpServer {
        command: v.get("command").and_then(|x| x.as_str()).map(String::from),
        args: v.get("args").and_then(|x| x.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        }),
        env: v.get("env").and_then(|x| x.as_object()).map(|obj| {
            obj.iter()
                .filter_map(|(k, val)| val.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        }),
        cwd: v.get("cwd").and_then(|x| x.as_str()).map(String::from),
        startup_timeout_ms: v.get("startup_timeout_ms").and_then(|x| x.as_u64()),
        url: v.get("url").and_then(|x| x.as_str()).map(String::from),
        bearer_token: v
            .get("bearer_token")
            .and_then(|x| x.as_str())
            .map(String::from),
        other: HashMap::new(),
    })
}

/// 从 markdown 内容中提取 YAML frontmatter 的 description 字段
fn extract_frontmatter_description(content: &str) -> (Option<String>, String) {
    if let Some(rest) = content.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let frontmatter = &rest[..end];
        let body = rest[end + 5..].to_string();
        let description = frontmatter.lines().find_map(|line| {
            let line = line.trim();
            line.strip_prefix("description:")
                .map(|v| v.trim().to_string())
        });
        return (description, body);
    }
    (None, content.to_string())
}

/// 从 JSON config 构建 agent markdown 内容
fn build_agent_markdown(config: &Value) -> String {
    let description = config
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let content = config.get("content").and_then(|v| v.as_str()).unwrap_or("");

    if description.is_empty() {
        content.to_string()
    } else {
        format!("---\ndescription: {description}\n---\n{content}")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn restore_env_var(key: &str, previous: Option<String>) {
        unsafe {
            match previous {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    #[test]
    fn patch_profile_with_config_removes_missing_platform_data_keys() {
        let mut profile = ProfileConfig::new();
        profile
            .platform_data
            .insert("env_key".into(), json!("OPENAI_API_KEY"));
        profile
            .platform_data
            .insert("provider_model".into(), json!("gpt-5"));

        patch_profile_with_config(
            &mut profile,
            &json!({
                "extra": {
                    "provider_model": "gpt-4.1"
                }
            }),
        )
        .unwrap();

        assert_eq!(profile.platform_data.len(), 1);
        assert_eq!(
            profile.platform_data.get("provider_model"),
            Some(&json!("gpt-4.1"))
        );
        assert!(!profile.platform_data.contains_key("env_key"));
    }

    #[test]
    fn patch_profile_with_config_clears_platform_data_when_extra_is_empty_object() {
        let mut profile = ProfileConfig::new();
        profile
            .platform_data
            .insert("provider_model".into(), json!("gpt-5"));

        patch_profile_with_config(
            &mut profile,
            &json!({
                "extra": {}
            }),
        )
        .unwrap();

        assert!(profile.platform_data.is_empty());
    }

    #[test]
    fn patch_profile_with_config_clears_platform_data_when_extra_is_null() {
        let mut profile = ProfileConfig::new();
        profile
            .platform_data
            .insert("provider_model".into(), json!("gpt-5"));

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
                    "provider_model": "gpt-4.1",
                    "wire_api": "responses"
                },
                "platform_data": {
                    "provider_model": "gpt-5",
                    "model_reasoning_effort": "high"
                }
            }),
        )
        .unwrap();

        assert_eq!(
            profile.platform_data.get("provider_model"),
            Some(&json!("gpt-5"))
        );
        assert_eq!(
            profile.platform_data.get("wire_api"),
            Some(&json!("responses"))
        );
        assert_eq!(
            profile.platform_data.get("model_reasoning_effort"),
            Some(&json!("high"))
        );
    }

    #[test]
    fn apply_codex_settings_update_patches_nested_fields() {
        let mut config = CodexConfig {
            tui: Some(CodexTuiConfig {
                other: HashMap::from([(
                    "status_line".into(),
                    toml::Value::Array(vec![toml::Value::String("model".into())]),
                )]),
                ..Default::default()
            }),
            ..Default::default()
        };

        apply_codex_settings_update(
            &mut config,
            &json!({
                "sandbox_workspace_write": {
                    "writable_roots": ["D:/repo", "D:/workspace"],
                    "network_access": true
                },
                "shell_environment_policy": {
                    "include_only": ["PATH", "HOME"]
                },
                "tools": {
                    "view_image": true,
                    "web_search": false
                },
                "tui": {
                    "alternate_screen": "never",
                    "animations": false
                },
                "history": {
                    "persistence": "save-all",
                    "max_bytes": 1024
                },
                "analytics": {
                    "enabled": true
                },
                "feedback": {
                    "enabled": false
                },
                "features": {
                    "multi_agent": true,
                    "steer": false
                }
            }),
        )
        .unwrap();

        assert_eq!(
            config
                .sandbox_workspace_write
                .as_ref()
                .and_then(|value| value.writable_roots.clone()),
            Some(vec!["D:/repo".into(), "D:/workspace".into()])
        );
        assert_eq!(
            config
                .shell_environment_policy
                .as_ref()
                .and_then(|value| value.include_only.clone()),
            Some(vec!["PATH".into(), "HOME".into()])
        );
        assert_eq!(
            config.tools.as_ref().and_then(|value| value.view_image),
            Some(true)
        );
        assert_eq!(
            config.tools.as_ref().and_then(|value| value.web_search),
            Some(false)
        );
        assert_eq!(
            config
                .tui
                .as_ref()
                .and_then(|value| value.alternate_screen.clone()),
            Some("never".into())
        );
        assert_eq!(
            config.tui.as_ref().and_then(|value| value.animations),
            Some(false)
        );
        assert!(config
            .tui
            .as_ref()
            .is_some_and(|value| value.other.contains_key("status_line")));
        assert_eq!(
            config
                .history
                .as_ref()
                .and_then(|value| value.persistence.clone()),
            Some("save-all".into())
        );
        assert_eq!(
            config.history.as_ref().and_then(|value| value.max_bytes),
            Some(1024)
        );
        assert_eq!(
            config.analytics.as_ref().and_then(|value| value.enabled),
            Some(true)
        );
        assert_eq!(
            config.feedback.as_ref().and_then(|value| value.enabled),
            Some(false)
        );
        assert_eq!(
            config
                .features
                .as_ref()
                .and_then(|value| value.get("multi_agent"))
                .copied(),
            Some(true)
        );
    }

    #[test]
    fn apply_codex_settings_update_clears_nested_fields_with_nulls() {
        let mut config = CodexConfig {
            sandbox_workspace_write: Some(CodexSandboxWorkspaceWriteConfig {
                writable_roots: Some(vec!["D:/repo".into()]),
                network_access: Some(true),
                ..Default::default()
            }),
            history: Some(CodexHistoryConfig {
                persistence: Some("save-all".into()),
                max_bytes: Some(2048),
                ..Default::default()
            }),
            tools: Some(CodexToolsConfig {
                view_image: Some(true),
                web_search: Some(true),
                ..Default::default()
            }),
            features: Some(HashMap::from([("multi_agent".into(), true)])),
            ..Default::default()
        };

        apply_codex_settings_update(
            &mut config,
            &json!({
                "sandbox_workspace_write": {
                    "writable_roots": null,
                    "network_access": null
                },
                "history": {
                    "persistence": null,
                    "max_bytes": null
                },
                "tools": {
                    "view_image": null,
                    "web_search": null
                },
                "features": {}
            }),
        )
        .unwrap();

        assert!(config.sandbox_workspace_write.is_none());
        assert!(config.history.is_none());
        assert!(config.tools.is_none());
        assert!(config.features.is_none());
    }

    #[tokio::test]
    async fn codex_list_profiles_reads_only_ccr_profiles_source() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let ccr_root = temp_dir.path().join("ccr-root");
        let codex_dir = temp_dir.path().join("official-codex");
        fs::create_dir_all(&ccr_root).unwrap();
        fs::create_dir_all(&codex_dir).unwrap();

        let previous_root = std::env::var("CCR_ROOT").ok();
        let previous_codex_dir = std::env::var("CCR_CODEX_DIR").ok();

        unsafe {
            std::env::set_var("CCR_ROOT", &ccr_root);
            std::env::set_var("CCR_CODEX_DIR", &codex_dir);
        }

        let result = async {
            fs::write(
                codex_dir.join("config.toml"),
                r#"[profiles.legacy]
model = "legacy-model"
"#,
            )
            .map_err(|e| format!("写入官方 config.toml 失败: {e}"))?;

            let platform = ccr::create_platform(ccr::Platform::Codex)
                .map_err(|e| format!("创建 Codex 平台失败: {e}"))?;
            let mut profile = ProfileConfig::new();
            profile.model = Some("real-model".to_string());
            platform
                .save_profile("real", &profile)
                .map_err(|e| format!("写入 CCR profiles.toml 失败: {e}"))?;

            let payload = codex_list_profiles().await?;
            let profiles = payload
                .get("profiles")
                .and_then(Value::as_array)
                .ok_or_else(|| "profiles 字段缺失".to_string())?;

            assert_eq!(profiles.len(), 1);
            assert_eq!(
                profiles[0].get("name").and_then(Value::as_str),
                Some("real")
            );
            assert!(
                profiles
                    .iter()
                    .all(|entry| { entry.get("name").and_then(Value::as_str) != Some("legacy") })
            );

            Ok::<(), String>(())
        }
        .await;

        restore_env_var("CCR_ROOT", previous_root);
        restore_env_var("CCR_CODEX_DIR", previous_codex_dir);
        result.unwrap();
    }
}
