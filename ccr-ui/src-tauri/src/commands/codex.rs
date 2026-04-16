//! Codex 命令 — Profiles/Settings/MCP/Agents/Auth/Usage。
//!
//! 配置文件位置: `~/.codex/config.toml`
//! Agents 目录:  `~/.codex/agents/`
//! Profiles:     通过 `ccr_codex::CodexPlatform` 管理
//! Auth:         通过 `ccr_codex::CodexAuthService` 管理
//! Usage:        通过 `ccr_codex::CodexUsageService` 管理

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use tauri::State;

use ccr_codex::services::codex_session_service::CodexSessionInventory;
use ccr_codex::{
    CodexAuthService, CodexPlatform, CodexSessionService, CodexUsageService, OpenAiAuthMethod,
};
use ccr_config::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};

use crate::state::{AppState, CacheFillRegistration};

const CODEX_USAGE_CACHE_KEY: &str = "codex:rolling_usage";
const CODEX_DASHBOARD_OVERVIEW_CACHE_KEY: &str = "codex:dashboard_overview";

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CodexDashboardOverviewFingerprint {
    codex_config_hash: u64,
    registry_hash: u64,
    profiles_hash: u64,
    auth_registry_hash: u64,
    auth_dir_hash: u64,
    agents_signature: String,
    session_inventory_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodexDashboardOverviewCacheEntry {
    fingerprint: CodexDashboardOverviewFingerprint,
    payload: Value,
}

type CodexModelCatalog = (Vec<String>, Vec<String>, Vec<String>);

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
const KNOWN_MCP_FIELDS: &[&str] = &[
    "enabled",
    "disabled",
    "command",
    "args",
    "env",
    "env_vars",
    "cwd",
    "startup_timeout_ms",
    "startup_timeout_sec",
    "tool_timeout_sec",
    "url",
    "http_headers",
    "headers",
    "env_http_headers",
    "bearer_token",
    "bearer_token_env_var",
    "oauth_resource",
    "scopes",
    "enabled_tools",
    "disabled_tools",
    "required",
];

#[path = "codex_agent_sources.rs"]
mod agent_sources;
#[path = "codex_agents.rs"]
mod agents;
#[path = "codex_auth.rs"]
mod auth;
#[path = "codex_mcp.rs"]
mod mcp;
#[path = "codex_profiles.rs"]
mod profiles;
#[path = "codex_sessions.rs"]
mod sessions;
#[path = "codex_settings.rs"]
mod settings;
#[path = "codex_tray.rs"]
mod tray;
#[path = "codex_usage.rs"]
mod usage;

pub use agent_sources::*;
pub use agents::*;
pub use auth::*;
pub use mcp::*;
pub use profiles::*;
pub use sessions::*;
pub use settings::*;
pub use tray::*;
pub use usage::*;

// ── 文件 I/O 辅助函数 ──

pub(crate) fn codex_config_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    Ok(home.join(".codex").join("config.toml"))
}

pub(crate) fn codex_agents_dir() -> Result<PathBuf, String> {
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

fn read_codex_model_catalog() -> Result<CodexModelCatalog, String> {
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
        if path.extension().and_then(|value| value.to_str()) == Some("toml") {
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

fn hash_bytes<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn hash_file_contents(path: &std::path::Path) -> u64 {
    fs::read(path).map(|bytes| hash_bytes(&bytes)).unwrap_or(0)
}

fn collect_directory_files(dir: &std::path::Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_directory_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn hash_directory_contents(dir: &std::path::Path) -> u64 {
    if !dir.exists() {
        return 0;
    }

    let mut files = Vec::new();
    collect_directory_files(dir, &mut files);
    files.sort();

    let mut hasher = DefaultHasher::new();
    for path in files {
        path.to_string_lossy().hash(&mut hasher);
        if let Ok(bytes) = fs::read(&path) {
            bytes.hash(&mut hasher);
        }
    }

    hasher.finish()
}

fn codex_dashboard_overview_fingerprint(
    session_inventory: &CodexSessionInventory,
) -> Result<CodexDashboardOverviewFingerprint, String> {
    let codex_paths = PlatformPaths::new(Platform::Codex)
        .map_err(|e| format!("初始化 Codex 平台路径失败: {e}"))?;
    let auth_dir = codex_paths.platform_dir.join("auth");
    let auth_registry_path = codex_paths.platform_dir.join("auth_registry.toml");
    let agents_signature = count_codex_agents(&codex_agents_dir()?)?.to_string();

    Ok(CodexDashboardOverviewFingerprint {
        codex_config_hash: hash_file_contents(&codex_config_path()?),
        registry_hash: hash_file_contents(&codex_paths.registry_file),
        profiles_hash: hash_file_contents(&codex_paths.profiles_file),
        auth_registry_hash: hash_file_contents(&auth_registry_path),
        auth_dir_hash: hash_directory_contents(&auth_dir),
        agents_signature,
        session_inventory_signature: session_inventory.signature.clone(),
    })
}

fn build_codex_dashboard_overview_payload(
    session_inventory: &CodexSessionInventory,
) -> Result<Value, String> {
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

    let agents_count = count_codex_agents(&codex_agents_dir()?)?;
    let sessions_count = session_inventory.total_sessions;
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
        "inventory": {
            "mcp_servers_total": mcp_servers_total,
            "agents_total": agents_count,
            "sessions_total": sessions_count,
            "config_profiles_total": config_profiles_total,
        }
    }))
}

async fn get_cached_codex_usage_payload(state: &AppState, force: bool) -> Result<Value, String> {
    if !force {
        if let Some(cached) = state.cache_get(CODEX_USAGE_CACHE_KEY).await {
            return Ok(cached);
        }

        match state.begin_cache_fill(CODEX_USAGE_CACHE_KEY).await {
            CacheFillRegistration::Wait(notify) => {
                notify.notified().await;
                if let Some(cached) = state.cache_get(CODEX_USAGE_CACHE_KEY).await {
                    return Ok(cached);
                }
            }
            CacheFillRegistration::Leader => {
                let result = tokio::task::spawn_blocking(usage::compute_codex_usage_payload)
                    .await
                    .map_err(|e| format!("任务执行失败: {e}"))?;

                match result {
                    Ok(payload) => {
                        state
                            .cache_set(CODEX_USAGE_CACHE_KEY.to_string(), payload.clone(), 30)
                            .await;
                        state.finish_cache_fill(CODEX_USAGE_CACHE_KEY).await;
                        return Ok(payload);
                    }
                    Err(error) => {
                        state.finish_cache_fill(CODEX_USAGE_CACHE_KEY).await;
                        return Err(error);
                    }
                }
            }
        }
    }

    tokio::task::spawn_blocking(usage::compute_codex_usage_payload)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
}

async fn get_cached_codex_dashboard_overview_payload(
    state: &AppState,
    force: bool,
) -> Result<Value, String> {
    let session_inventory = CodexSessionService::new(
        dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".codex"),
    )
    .session_inventory()
    .map_err(|e| format!("统计 Codex sessions 失败: {e}"))?;
    let fingerprint = codex_dashboard_overview_fingerprint(&session_inventory)?;

    if !force {
        if let Some(cached) = state.cache_get(CODEX_DASHBOARD_OVERVIEW_CACHE_KEY).await {
            let entry: CodexDashboardOverviewCacheEntry = serde_json::from_value(cached)
                .map_err(|e| format!("仪表盘概览缓存解析失败: {e}"))?;
            if entry.fingerprint == fingerprint {
                return Ok(entry.payload);
            }
        }

        match state
            .begin_cache_fill(CODEX_DASHBOARD_OVERVIEW_CACHE_KEY)
            .await
        {
            CacheFillRegistration::Wait(notify) => {
                notify.notified().await;
                if let Some(cached) = state.cache_get(CODEX_DASHBOARD_OVERVIEW_CACHE_KEY).await {
                    let entry: CodexDashboardOverviewCacheEntry = serde_json::from_value(cached)
                        .map_err(|e| format!("仪表盘概览缓存解析失败: {e}"))?;
                    if entry.fingerprint == fingerprint {
                        return Ok(entry.payload);
                    }
                }
            }
            CacheFillRegistration::Leader => {
                let session_inventory_for_task = session_inventory.clone();
                let result = tokio::task::spawn_blocking(move || {
                    build_codex_dashboard_overview_payload(&session_inventory_for_task)
                })
                .await
                .map_err(|e| format!("任务执行失败: {e}"))?;

                match result {
                    Ok(payload) => {
                        let entry = CodexDashboardOverviewCacheEntry {
                            fingerprint: fingerprint.clone(),
                            payload: payload.clone(),
                        };
                        let cached_value = serde_json::to_value(entry)
                            .map_err(|e| format!("仪表盘概览缓存序列化失败: {e}"))?;
                        state
                            .cache_set(
                                CODEX_DASHBOARD_OVERVIEW_CACHE_KEY.to_string(),
                                cached_value,
                                300,
                            )
                            .await;
                        state
                            .finish_cache_fill(CODEX_DASHBOARD_OVERVIEW_CACHE_KEY)
                            .await;
                        return Ok(payload);
                    }
                    Err(error) => {
                        state
                            .finish_cache_fill(CODEX_DASHBOARD_OVERVIEW_CACHE_KEY)
                            .await;
                        return Err(error);
                    }
                }
            }
        }
    }

    tokio::task::spawn_blocking(move || build_codex_dashboard_overview_payload(&session_inventory))
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
}

pub(crate) async fn invalidate_codex_dashboard_overview_cache(state: &AppState) {
    state.cache_remove(CODEX_DASHBOARD_OVERVIEW_CACHE_KEY).await;
}

async fn invalidate_codex_usage_cache(state: &AppState) {
    state.cache_remove(CODEX_USAGE_CACHE_KEY).await;
}

fn invalidate_codex_session_inventory_cache() -> Result<(), String> {
    let codex_dir = dirs::home_dir()
        .ok_or_else(|| "无法获取用户主目录".to_string())?
        .join(".codex");
    let service = CodexSessionService::new(codex_dir);
    service
        .invalidate_inventory_cache()
        .map_err(|e| format!("清理 Codex session inventory 缓存失败: {e}"))
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

fn parse_u64_field(raw: &Value, field_name: &str) -> Result<Option<u64>, String> {
    let parsed = parse_i64_field(raw, field_name)?;
    match parsed {
        Some(value) if value < 0 => Err(format!("字段 '{field_name}' 不能为负数")),
        Some(value) => Ok(Some(value as u64)),
        None => Ok(None),
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

fn parse_string_map_field(
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

    if nested.writable_roots.is_none() && nested.network_access.is_none() && nested.other.is_empty()
    {
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

// ── 私有辅助函数 ──

fn json_to_toml_value(raw: &Value, field_name: &str) -> Result<toml::Value, String> {
    serde_json::from_value(raw.clone())
        .map_err(|_| format!("字段 '{field_name}' 无法转换为 TOML 值"))
}

fn validate_mcp_server(server: &CodexMcpServer) -> Result<(), String> {
    if server.command.is_none() && server.url.is_none() {
        return Err("MCP 服务器必须至少提供 command 或 url".to_string());
    }

    Ok(())
}

fn merge_codex_mcp_server(parsed: &mut CodexMcpServer, existing: &CodexMcpServer) {
    let mut merged_other = existing.other.clone();
    merged_other.extend(parsed.other.clone());
    parsed.other = merged_other;
}

/// 从 JSON Value 解析 CodexMcpServer
fn parse_mcp_server(v: &Value) -> Result<CodexMcpServer, String> {
    let obj = v
        .as_object()
        .ok_or_else(|| "MCP 服务器配置必须是对象".to_string())?;

    let enabled = if let Some(raw) = obj.get("enabled") {
        parse_bool_field(raw, "enabled")?
    } else if let Some(raw) = obj.get("disabled") {
        parse_bool_field(raw, "disabled")?.map(|disabled| !disabled)
    } else {
        None
    };

    let http_headers = if let Some(raw) = obj.get("http_headers") {
        parse_string_map_field(raw, "http_headers")?
    } else if let Some(raw) = obj.get("headers") {
        parse_string_map_field(raw, "headers")?
    } else {
        None
    };

    let mut other = HashMap::new();
    for (key, value) in obj {
        if KNOWN_MCP_FIELDS.contains(&key.as_str()) || value.is_null() {
            continue;
        }

        other.insert(key.clone(), json_to_toml_value(value, key)?);
    }

    Ok(CodexMcpServer {
        enabled,
        command: parse_string_field(obj.get("command").unwrap_or(&Value::Null), "command")?,
        args: parse_string_array_field(obj.get("args").unwrap_or(&Value::Null), "args")?,
        env: parse_string_map_field(obj.get("env").unwrap_or(&Value::Null), "env")?,
        env_vars: parse_string_array_field(
            obj.get("env_vars").unwrap_or(&Value::Null),
            "env_vars",
        )?,
        cwd: parse_string_field(obj.get("cwd").unwrap_or(&Value::Null), "cwd")?,
        startup_timeout_ms: parse_u64_field(
            obj.get("startup_timeout_ms").unwrap_or(&Value::Null),
            "startup_timeout_ms",
        )?,
        startup_timeout_sec: parse_u64_field(
            obj.get("startup_timeout_sec").unwrap_or(&Value::Null),
            "startup_timeout_sec",
        )?,
        tool_timeout_sec: parse_u64_field(
            obj.get("tool_timeout_sec").unwrap_or(&Value::Null),
            "tool_timeout_sec",
        )?,
        url: parse_string_field(obj.get("url").unwrap_or(&Value::Null), "url")?,
        http_headers,
        env_http_headers: parse_string_map_field(
            obj.get("env_http_headers").unwrap_or(&Value::Null),
            "env_http_headers",
        )?,
        bearer_token: parse_string_field(
            obj.get("bearer_token").unwrap_or(&Value::Null),
            "bearer_token",
        )?,
        bearer_token_env_var: parse_string_field(
            obj.get("bearer_token_env_var").unwrap_or(&Value::Null),
            "bearer_token_env_var",
        )?,
        oauth_resource: parse_string_field(
            obj.get("oauth_resource").unwrap_or(&Value::Null),
            "oauth_resource",
        )?,
        scopes: parse_string_array_field(obj.get("scopes").unwrap_or(&Value::Null), "scopes")?,
        enabled_tools: parse_string_array_field(
            obj.get("enabled_tools").unwrap_or(&Value::Null),
            "enabled_tools",
        )?,
        disabled_tools: parse_string_array_field(
            obj.get("disabled_tools").unwrap_or(&Value::Null),
            "disabled_tools",
        )?,
        required: parse_bool_field(obj.get("required").unwrap_or(&Value::Null), "required")?,
        other,
    })
}

/// 从 markdown 内容中提取 YAML frontmatter 的 description 字段
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
        assert!(
            config
                .tui
                .as_ref()
                .is_some_and(|value| value.other.contains_key("status_line"))
        );
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

            let platform = CodexPlatform::new().map_err(|e| format!("创建 Codex 平台失败: {e}"))?;
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

    #[test]
    fn parse_mcp_server_supports_official_fields_and_preserves_unknown_fields() {
        let parsed = parse_mcp_server(&json!({
            "enabled": false,
            "url": "https://api.openai.com/mcp",
            "http_headers": {
                "x-org": "openai"
            },
            "env_http_headers": {
                "Authorization": "OPENAI_API_KEY"
            },
            "bearer_token_env_var": "OPENAI_API_KEY",
            "oauth_resource": "https://api.openai.com/mcp",
            "scopes": ["docs.read"],
            "enabled_tools": ["search", "read"],
            "disabled_tools": ["write"],
            "env_vars": ["PATH", "HOME"],
            "startup_timeout_sec": 12,
            "tool_timeout_sec": 45,
            "required": true,
            "custom_field": "keep-me"
        }))
        .unwrap();

        assert_eq!(parsed.enabled, Some(false));
        assert_eq!(parsed.url.as_deref(), Some("https://api.openai.com/mcp"));
        assert_eq!(
            parsed
                .http_headers
                .as_ref()
                .and_then(|headers| headers.get("x-org"))
                .map(String::as_str),
            Some("openai")
        );
        assert_eq!(
            parsed
                .env_http_headers
                .as_ref()
                .and_then(|headers| headers.get("Authorization"))
                .map(String::as_str),
            Some("OPENAI_API_KEY")
        );
        assert_eq!(
            parsed.bearer_token_env_var.as_deref(),
            Some("OPENAI_API_KEY")
        );
        assert_eq!(parsed.required, Some(true));
        assert_eq!(
            parsed.other.get("custom_field"),
            Some(&toml::Value::String("keep-me".to_string()))
        );
    }

    #[test]
    fn parse_mcp_server_accepts_legacy_disabled_alias_and_merge_keeps_unknown_fields() {
        let mut parsed = parse_mcp_server(&json!({
            "disabled": true,
            "command": "npx"
        }))
        .unwrap();

        assert_eq!(parsed.enabled, Some(false));

        let existing = CodexMcpServer {
            enabled: Some(true),
            command: Some("npx".into()),
            args: None,
            env: None,
            env_vars: None,
            cwd: None,
            startup_timeout_ms: None,
            startup_timeout_sec: None,
            tool_timeout_sec: None,
            url: None,
            http_headers: None,
            env_http_headers: None,
            bearer_token: None,
            bearer_token_env_var: None,
            oauth_resource: None,
            scopes: None,
            enabled_tools: None,
            disabled_tools: None,
            required: None,
            other: HashMap::from([("legacy".into(), toml::Value::String("kept".into()))]),
        };

        merge_codex_mcp_server(&mut parsed, &existing);

        assert_eq!(
            parsed.other.get("legacy"),
            Some(&toml::Value::String("kept".to_string()))
        );
    }
}
