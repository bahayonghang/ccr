//! Codex 命令 — Profiles/Settings/MCP/Agents/Auth/Usage。
//!
//! 配置文件位置: `~/.codex/config.toml`
//! Agents 目录:  `~/.codex/agents/`
//! Profiles:     通过 `ccr::create_platform(Platform::Codex)` 管理
//! Auth:         通过 `ccr::services::CodexAuthService` 管理
//! Usage:        通过 `ccr::services::CodexUsageService` 管理

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use ccr::services::{CodexAuthService, CodexUsageService};
use ccr::{Platform, create_platform};

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
    pub web_search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_opener: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub developer_instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
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

// ── 文件 I/O 辅助函数 ──

fn codex_config_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    Ok(home.join(".codex").join("config.toml"))
}

fn codex_agents_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    Ok(home.join(".codex").join("agents"))
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

// ── Profiles ──

/// 列出 Codex config.toml 中的 [profiles] 段
#[tauri::command]
pub async fn codex_list_profiles() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let path = codex_config_path()?;
        let config = read_codex_config(&path)?;

        // 查找当前 profile (enabled=true 或 currentProfile 字段)
        let current_profile = config
            .other
            .get("currentProfile")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| {
                // 在 CCR 平台层查找当前 profile
                create_platform(Platform::Codex)
                    .ok()
                    .and_then(|p| p.get_current_profile().ok().flatten())
            });

        let profiles: Vec<Value> = config
            .profiles
            .unwrap_or_default()
            .into_iter()
            .map(|(name, profile)| {
                json!({
                    "name": name,
                    "model": profile.model,
                    "approval_policy": profile.approval_policy,
                    "sandbox_mode": profile.sandbox_mode,
                    "model_reasoning_effort": profile.model_reasoning_effort,
                })
            })
            .collect();
        Ok(json!({ "profiles": profiles, "current_profile": current_profile }))
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
        Ok(json!({
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
            "web_search": config.web_search,
            "file_opener": config.file_opener,
            "developer_instructions": config.developer_instructions,
            "instructions": config.instructions,
            "hide_agent_reasoning": config.hide_agent_reasoning,
            "show_raw_agent_reasoning": config.show_raw_agent_reasoning,
            "check_for_update_on_startup": config.check_for_update_on_startup,
            "suppress_unstable_features_warning": config.suppress_unstable_features_warning,
            "experimental_use_rmcp_client": config.experimental_use_rmcp_client,
        }))
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

        macro_rules! apply_str {
            ($field:ident) => {
                if let Some(v) = settings.get(stringify!($field)).and_then(|v| v.as_str()) {
                    config.$field = Some(v.to_string());
                }
            };
        }
        macro_rules! apply_bool {
            ($field:ident) => {
                if let Some(v) = settings.get(stringify!($field)).and_then(|v| v.as_bool()) {
                    config.$field = Some(v);
                }
            };
        }
        macro_rules! apply_i64 {
            ($field:ident) => {
                if let Some(v) = settings.get(stringify!($field)).and_then(|v| v.as_i64()) {
                    config.$field = Some(v);
                }
            };
        }

        apply_str!(model);
        apply_str!(model_provider);
        apply_str!(model_reasoning_effort);
        apply_str!(model_reasoning_summary);
        apply_str!(model_verbosity);
        apply_i64!(model_context_window);
        apply_i64!(model_auto_compact_token_limit);
        apply_str!(personality);
        apply_str!(approval_policy);
        apply_str!(sandbox_mode);
        apply_bool!(disable_response_storage);
        apply_str!(web_search);
        apply_str!(file_opener);
        apply_str!(developer_instructions);
        apply_str!(instructions);
        apply_bool!(hide_agent_reasoning);
        apply_bool!(show_raw_agent_reasoning);
        apply_bool!(check_for_update_on_startup);
        apply_bool!(suppress_unstable_features_warning);
        apply_bool!(experimental_use_rmcp_client);

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

// ── Auth 账号管理 ──

/// 列出所有 Codex Auth 账号
#[tauri::command]
pub async fn codex_list_auth_accounts() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let accounts = service
            .list_accounts()
            .map_err(|e| format!("列出账号失败: {e}"))?;

        let login_state = service
            .get_login_state()
            .map_err(|e| format!("获取登录状态失败: {e}"))?;

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

        Ok(json!({ "accounts": accounts, "login_state": login_state }))
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

        let login_state = service
            .get_login_state()
            .map_err(|e| format!("获取登录状态失败: {e}"))?;

        let info = match service.get_current_auth_info() {
            Ok(current) => {
                let freshness = &current.freshness;
                let expires_at = service.load_registry().ok().and_then(|reg| {
                    reg.current_auth
                        .and_then(|name| reg.accounts.get(&name).and_then(|a| a.expires_at))
                });
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
            Err(_) => None,
        };

        let logged_in = info.is_some();

        Ok(json!({
            "logged_in": logged_in,
            "info": info,
            "login_state": login_state,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 保存当前登录到命名账号
#[tauri::command]
pub async fn codex_save_auth(name: String, description: Option<String>) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        service
            .save_current(&name, description, None, true)
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

// ── Usage 统计 ──

/// 获取 Codex 使用量统计
#[tauri::command]
pub async fn codex_get_usage() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let codex_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".codex");

        let service = CodexUsageService::new(codex_dir);
        let rolling = service
            .compute_rolling_usage()
            .map_err(|e| format!("计算使用量失败: {e}"))?;

        // 将 by_model HashMap 转为 JSON object
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

        Ok(json!({
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
