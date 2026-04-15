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

use ccr_config::{Platform, PlatformConfig, ProfileConfig};
use ccr_skills::{PromptPreset, PromptsManager};
use ccr::platforms::ClaudePlatform;
use ccr::services::ClaudeAuthService;
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
// ── Settings ──
// ═══════════════════════════════════════════════════════════

/// 读取 ~/.claude/settings.json，以 JSON Value 返回完整内容。
#[tauri::command]
pub async fn claude_get_settings(state: State<'_, AppState>) -> Result<Value, String> {
    read_active_claude_settings_raw(state.inner()).await
}

/// 将调用方提供的 JSON 合并写入 ~/.claude/settings.json。
#[tauri::command]
pub async fn claude_update_settings(
    state: State<'_, AppState>,
    settings: Value,
) -> Result<Value, String> {
    let mut current = read_active_claude_settings_raw(state.inner()).await?;
    merge_settings_patch(&mut current, settings)?;

    let validated: ccr_types::ClaudeSettings =
        serde_json::from_value(current).map_err(|e| format!("Invalid settings payload: {e}"))?;
    let result =
        serde_json::to_value(&validated).map_err(|e| format!("Serialization error: {e}"))?;

    write_active_claude_settings_raw(state.inner(), &result).await?;
    Ok(result)
}

// ═══════════════════════════════════════════════════════════
// ── MCP Servers（~/.claude.json）──
// ═══════════════════════════════════════════════════════════

#[tauri::command]
pub async fn claude_list_mcp_servers() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let config =
            read_claude_config().map_err(|e| format!("Failed to read .claude.json: {}", e))?;

        let servers: Vec<Value> = config
            .mcp_servers
            .into_iter()
            .map(|(name, server)| {
                serde_json::json!({
                    "name": name,
                    "command": server.command.unwrap_or_default(),
                    "args": server.args,
                    "env": server.env.unwrap_or_default(),
                    "type": server.server_type,
                    "url": server.url,
                    "disabled": server.disabled.unwrap_or(false),
                })
            })
            .collect();

        Ok(serde_json::json!({ "servers": servers }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_add_mcp_server(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut claude_config =
            read_claude_config().map_err(|e| format!("Failed to read .claude.json: {}", e))?;

        let entry: McpServerEntry = serde_json::from_value(config)
            .map_err(|e| format!("Invalid MCP server config: {}", e))?;

        claude_config.mcp_servers.insert(name, entry);
        write_claude_config(&claude_config)
            .map_err(|e| format!("Failed to write .claude.json: {}", e))?;

        Ok(serde_json::json!({ "success": true }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_update_mcp_server(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut claude_config =
            read_claude_config().map_err(|e| format!("Failed to read .claude.json: {}", e))?;

        if !claude_config.mcp_servers.contains_key(&name) {
            return Err(format!("MCP server '{}' not found", name));
        }

        let entry: McpServerEntry = serde_json::from_value(config)
            .map_err(|e| format!("Invalid MCP server config: {}", e))?;

        claude_config.mcp_servers.insert(name, entry);
        write_claude_config(&claude_config)
            .map_err(|e| format!("Failed to write .claude.json: {}", e))?;

        Ok(serde_json::json!({ "success": true }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_delete_mcp_server(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut claude_config =
            read_claude_config().map_err(|e| format!("Failed to read .claude.json: {}", e))?;

        if claude_config.mcp_servers.remove(&name).is_none() {
            return Err(format!("MCP server '{}' not found", name));
        }

        write_claude_config(&claude_config)
            .map_err(|e| format!("Failed to write .claude.json: {}", e))?;

        Ok(format!("MCP server '{}' deleted", name))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// ═══════════════════════════════════════════════════════════
// ── Agents（~/.claude/settings.json .agents[]）──
// ═══════════════════════════════════════════════════════════

#[tauri::command]
pub async fn claude_list_agents(state: State<'_, AppState>) -> Result<Value, String> {
    let settings = load_settings(state.inner()).await?;
    let agents = serde_json::to_value(&settings.agents)
        .map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "agents": agents }))
}

#[tauri::command]
pub async fn claude_add_agent(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let mut settings = load_settings(state.inner()).await?;

    let mut agent: ccr_types::Agent =
        serde_json::from_value(config).map_err(|e| format!("Invalid agent config: {}", e))?;
    // Ensure the name from the parameter takes precedence
    agent.name = name;

    settings.agents.push(agent);
    save_settings(state.inner(), &settings).await?;

    let result = serde_json::to_value(&settings.agents)
        .map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "agents": result }))
}

#[tauri::command]
pub async fn claude_update_agent(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let mut settings = load_settings(state.inner()).await?;

    let pos = settings
        .agents
        .iter()
        .position(|a| a.name == name)
        .ok_or_else(|| format!("Agent '{}' not found", name))?;

    let updated: ccr_types::Agent =
        serde_json::from_value(config).map_err(|e| format!("Invalid agent config: {}", e))?;
    settings.agents[pos] = updated;

    save_settings(state.inner(), &settings).await?;

    let result = serde_json::to_value(&settings.agents)
        .map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "agents": result }))
}

#[tauri::command]
pub async fn claude_delete_agent(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let mut settings = load_settings(state.inner()).await?;

    let original_len = settings.agents.len();
    settings.agents.retain(|a| a.name != name);

    if settings.agents.len() >= original_len {
        return Err(format!("Agent '{}' not found", name));
    }

    save_settings(state.inner(), &settings).await?;
    Ok(format!("Agent '{}' deleted", name))
}

// ═══════════════════════════════════════════════════════════
// ── Slash Commands（~/.claude/settings.json .slashCommands[]）──
// ═══════════════════════════════════════════════════════════

#[tauri::command]
pub async fn claude_list_slash_commands(state: State<'_, AppState>) -> Result<Value, String> {
    let settings = load_settings(state.inner()).await?;
    let commands = serde_json::to_value(&settings.slash_commands)
        .map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "commands": commands }))
}

#[tauri::command]
pub async fn claude_add_slash_command(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let mut settings = load_settings(state.inner()).await?;

    let mut cmd: ccr_types::SlashCommand = serde_json::from_value(config)
        .map_err(|e| format!("Invalid slash command config: {}", e))?;
    cmd.name = name;

    settings.slash_commands.push(cmd);
    save_settings(state.inner(), &settings).await?;

    let result = serde_json::to_value(&settings.slash_commands)
        .map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "commands": result }))
}

#[tauri::command]
pub async fn claude_update_slash_command(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let mut settings = load_settings(state.inner()).await?;

    let pos = settings
        .slash_commands
        .iter()
        .position(|c| c.name == name)
        .ok_or_else(|| format!("Slash command '{}' not found", name))?;

    let updated: ccr_types::SlashCommand = serde_json::from_value(config)
        .map_err(|e| format!("Invalid slash command config: {}", e))?;
    settings.slash_commands[pos] = updated;

    save_settings(state.inner(), &settings).await?;

    let result = serde_json::to_value(&settings.slash_commands)
        .map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "commands": result }))
}

#[tauri::command]
pub async fn claude_delete_slash_command(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let mut settings = load_settings(state.inner()).await?;

    let original_len = settings.slash_commands.len();
    settings.slash_commands.retain(|c| c.name != name);

    if settings.slash_commands.len() >= original_len {
        return Err(format!("Slash command '{}' not found", name));
    }

    save_settings(state.inner(), &settings).await?;
    Ok(format!("Slash command '{}' deleted", name))
}

// ═══════════════════════════════════════════════════════════
// ── Plugins（~/.claude/settings.json .plugins[]）──
// ═══════════════════════════════════════════════════════════

#[tauri::command]
pub async fn claude_list_plugins(state: State<'_, AppState>) -> Result<Value, String> {
    let settings = load_settings(state.inner()).await?;
    let plugins = serde_json::to_value(&settings.plugins)
        .map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "plugins": plugins }))
}

#[tauri::command]
pub async fn claude_add_plugin(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let mut settings = load_settings(state.inner()).await?;

    let mut plugin: ccr_types::Plugin =
        serde_json::from_value(config).map_err(|e| format!("Invalid plugin config: {}", e))?;
    plugin.name = name;

    settings.plugins.push(plugin);
    save_settings(state.inner(), &settings).await?;

    let result = serde_json::to_value(&settings.plugins)
        .map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "plugins": result }))
}

#[tauri::command]
pub async fn claude_update_plugin(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let mut settings = load_settings(state.inner()).await?;

    let pos = settings
        .plugins
        .iter()
        .position(|p| p.name == name)
        .ok_or_else(|| format!("Plugin '{}' not found", name))?;

    let updated: ccr_types::Plugin =
        serde_json::from_value(config).map_err(|e| format!("Invalid plugin config: {}", e))?;
    settings.plugins[pos] = updated;

    save_settings(state.inner(), &settings).await?;

    let result = serde_json::to_value(&settings.plugins)
        .map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "plugins": result }))
}

#[tauri::command]
pub async fn claude_delete_plugin(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let mut settings = load_settings(state.inner()).await?;

    let original_len = settings.plugins.len();
    settings.plugins.retain(|p| p.name != name);

    if settings.plugins.len() >= original_len {
        return Err(format!("Plugin '{}' not found", name));
    }

    save_settings(state.inner(), &settings).await?;
    Ok(format!("Plugin '{}' deleted", name))
}

// ═══════════════════════════════════════════════════════════
// ── Output Styles（~/.claude/output-styles/*.md）──
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize)]
struct OutputStyle {
    name: String,
    content: String,
}

#[tauri::command]
pub async fn claude_get_output_styles() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let dir = output_styles_dir()
            .map_err(|e| format!("Cannot determine output-styles dir: {}", e))?;

        if !dir.exists() {
            return Ok(serde_json::json!({ "styles": [] }));
        }

        let entries =
            fs::read_dir(&dir).map_err(|e| format!("Failed to read output-styles dir: {}", e))?;

        let mut styles: Vec<OutputStyle> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            styles.push(OutputStyle {
                name: stem.to_string(),
                content,
            });
        }

        styles.sort_by(|a, b| a.name.cmp(&b.name));
        let result =
            serde_json::to_value(&styles).map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "styles": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// 写入或覆盖一个 output style 文件（styles 是 [{name, content}] 数组）。
#[tauri::command]
pub async fn claude_update_output_styles(styles: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let dir = output_styles_dir()
            .map_err(|e| format!("Cannot determine output-styles dir: {}", e))?;

        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create output-styles dir: {}", e))?;

        let items: Vec<OutputStyle> = serde_json::from_value(styles)
            .map_err(|e| format!("Invalid output styles payload: {}", e))?;

        for item in &items {
            let path = dir.join(format!("{}.md", item.name));
            fs::write(&path, &item.content)
                .map_err(|e| format!("Failed to write style '{}': {}", item.name, e))?;
        }

        let result =
            serde_json::to_value(&items).map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "styles": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// ═══════════════════════════════════════════════════════════
// ── Statusline（~/.claude/settings.json .other["statusline"]）──
// ═══════════════════════════════════════════════════════════

#[tauri::command]
pub async fn claude_get_statusline(state: State<'_, AppState>) -> Result<Value, String> {
    let settings = load_settings(state.inner()).await?;
    let statusline = settings
        .other
        .get("statusline")
        .cloned()
        .unwrap_or(Value::Null);
    Ok(statusline)
}

#[tauri::command]
pub async fn claude_update_statusline(
    state: State<'_, AppState>,
    config: Value,
) -> Result<Value, String> {
    let mut settings = load_settings(state.inner()).await?;
    settings
        .other
        .insert("statusline".to_string(), config.clone());
    save_settings(state.inner(), &settings).await?;
    Ok(config)
}

// ═══════════════════════════════════════════════════════════
// ── Hooks（~/.claude/settings.json .hooks.{event}[].hooks[]）──
// ═══════════════════════════════════════════════════════════

#[tauri::command]
pub async fn claude_list_hooks(state: State<'_, AppState>) -> Result<Value, String> {
    let settings = load_settings(state.inner()).await?;
    let hooks =
        serde_json::to_value(&settings.hooks).map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "hooks": hooks }))
}

/// 整体替换 hooks 配置（官方 grouped hooks 对象）。
#[tauri::command]
pub async fn claude_update_hooks(
    state: State<'_, AppState>,
    hooks: Value,
) -> Result<Value, String> {
    let mut settings = load_settings(state.inner()).await?;

    let new_hooks: ccr_types::HooksConfig =
        serde_json::from_value(hooks).map_err(|e| format!("Invalid hooks payload: {}", e))?;
    settings.hooks = new_hooks;

    save_settings(state.inner(), &settings).await?;

    let result =
        serde_json::to_value(&settings.hooks).map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "hooks": result }))
}

// ═══════════════════════════════════════════════════════════
// ── Budgets（~/.claude/budget.toml via ccr::BudgetManager）──
// ═══════════════════════════════════════════════════════════

#[tauri::command]
pub async fn claude_get_budgets() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let budget_manager = BudgetManager::with_default()
            .map_err(|e| format!("BudgetManager init error: {}", e))?;

        let storage_dir = CostTracker::default_storage_dir()
            .map_err(|e| format!("CostTracker storage dir error: {}", e))?;
        let tracker =
            CostTracker::new(storage_dir).map_err(|e| format!("CostTracker init error: {}", e))?;

        let status = budget_manager
            .check_status(&tracker)
            .map_err(|e| format!("Budget status error: {}", e))?;
        let config = budget_manager.get_config();

        Ok(serde_json::json!({
            "enabled": status.enabled,
            "dailyLimit": config.daily_limit,
            "weeklyLimit": config.weekly_limit,
            "monthlyLimit": config.monthly_limit,
            "warnAtPercent": config.warn_at_percent,
            "currentCosts": {
                "today": status.current_costs.today,
                "thisWeek": status.current_costs.this_week,
                "thisMonth": status.current_costs.this_month,
            },
            "warnings": status.warnings.iter().map(|w| serde_json::json!({
                "period": w.period.to_string(),
                "currentCost": w.current_cost,
                "limit": w.limit,
                "usagePercent": w.usage_percent,
            })).collect::<Vec<_>>(),
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// 更新预算配置。budgets 可包含字段：enabled, dailyLimit, weeklyLimit, monthlyLimit,
/// warnAtPercent。
#[tauri::command]
pub async fn claude_update_budgets(budgets: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut budget_manager = BudgetManager::with_default()
            .map_err(|e| format!("BudgetManager init error: {}", e))?;

        if let Some(enabled) = budgets.get("enabled").and_then(|v| v.as_bool()) {
            if enabled {
                budget_manager
                    .enable()
                    .map_err(|e| format!("Failed to enable budget: {}", e))?;
            } else {
                budget_manager
                    .disable()
                    .map_err(|e| format!("Failed to disable budget: {}", e))?;
            }
        }

        // dailyLimit: null clears, number sets
        if let Some(daily_val) = budgets.get("dailyLimit") {
            let limit = if daily_val.is_null() {
                None
            } else {
                daily_val
                    .as_f64()
                    .map(Some)
                    .ok_or_else(|| "dailyLimit must be a number or null".to_string())?
            };
            budget_manager
                .set_daily_limit(limit)
                .map_err(|e| format!("Failed to set daily limit: {}", e))?;
        }

        if let Some(weekly_val) = budgets.get("weeklyLimit") {
            let limit = if weekly_val.is_null() {
                None
            } else {
                weekly_val
                    .as_f64()
                    .map(Some)
                    .ok_or_else(|| "weeklyLimit must be a number or null".to_string())?
            };
            budget_manager
                .set_weekly_limit(limit)
                .map_err(|e| format!("Failed to set weekly limit: {}", e))?;
        }

        if let Some(monthly_val) = budgets.get("monthlyLimit") {
            let limit = if monthly_val.is_null() {
                None
            } else {
                monthly_val
                    .as_f64()
                    .map(Some)
                    .ok_or_else(|| "monthlyLimit must be a number or null".to_string())?
            };
            budget_manager
                .set_monthly_limit(limit)
                .map_err(|e| format!("Failed to set monthly limit: {}", e))?;
        }

        if let Some(pct) = budgets
            .get("warnAtPercent")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
        {
            budget_manager
                .set_warn_threshold(pct)
                .map_err(|e| format!("Failed to set warn threshold: {}", e))?;
        }

        let config = budget_manager.get_config();
        Ok(serde_json::json!({
            "enabled": config.enabled,
            "dailyLimit": config.daily_limit,
            "weeklyLimit": config.weekly_limit,
            "monthlyLimit": config.monthly_limit,
            "warnAtPercent": config.warn_at_percent,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// ═══════════════════════════════════════════════════════════
// ── Prompts（~/.ccr/platforms/claude/prompts.toml via ccr::PromptsManager）──
// ═══════════════════════════════════════════════════════════

#[tauri::command]
pub async fn claude_list_prompts() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let manager = PromptsManager::new(Platform::Claude)
            .map_err(|e| format!("PromptsManager init error: {}", e))?;

        let presets = manager
            .list_presets()
            .map_err(|e| format!("Failed to list presets: {}", e))?;

        let result =
            serde_json::to_value(&presets).map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "presets": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// 整体替换 prompt presets 列表，或添加/更新单个 preset。
/// prompts 可以是 PromptPreset 数组，或单个 PromptPreset 对象（则执行 add/update）。
#[tauri::command]
pub async fn claude_update_prompts(prompts: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let manager = PromptsManager::new(Platform::Claude)
            .map_err(|e| format!("PromptsManager init error: {}", e))?;

        if let Some(arr) = prompts.as_array() {
            // 批量替换：先删除全部，再逐个添加
            let existing = manager
                .list_presets()
                .map_err(|e| format!("Failed to list presets: {}", e))?;
            for preset in &existing {
                let _ = manager.remove_preset(&preset.name);
            }
            for item in arr {
                let preset: PromptPreset = serde_json::from_value(item.clone())
                    .map_err(|e| format!("Invalid preset item: {}", e))?;
                manager
                    .add_preset(preset)
                    .map_err(|e| format!("Failed to add preset: {}", e))?;
            }
        } else {
            // 单个 preset：upsert（remove if exists, then add）
            let preset: PromptPreset = serde_json::from_value(prompts)
                .map_err(|e| format!("Invalid preset payload: {}", e))?;
            let _ = manager.remove_preset(&preset.name);
            manager
                .add_preset(preset)
                .map_err(|e| format!("Failed to add preset: {}", e))?;
        }

        let updated = manager
            .list_presets()
            .map_err(|e| format!("Failed to list presets: {}", e))?;
        let result =
            serde_json::to_value(&updated).map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "presets": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
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

#[tauri::command]
pub async fn claude_list_auth_accounts() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        let snapshot = service
            .read_auth_snapshot()
            .map_err(|e| format!("读取认证快照失败: {e}"))?;
        let accounts = service
            .build_account_items(&snapshot)
            .map_err(|e| format!("列出账号失败: {e}"))?;
        let runtime_summary = service
            .get_runtime_summary()
            .map_err(|e| format!("读取运行时摘要失败: {e}"))?;

        let accounts: Vec<Value> = accounts
            .into_iter()
            .map(|item| {
                let freshness = &item.freshness;
                json!({
                    "name": item.name,
                    "description": item.description,
                    "email": item.email,
                    "billing_type": item.billing_type,
                    "subscription_type": item.subscription_type,
                    "rate_limit_tier": item.rate_limit_tier,
                    "is_current": item.is_current,
                    "saved_at": item.saved_at.to_rfc3339(),
                    "last_used": item.last_used.map(|dt| dt.to_rfc3339()),
                    "expires_at": item.expires_at.map(|dt| dt.to_rfc3339()),
                    "is_expired": ClaudeAuthService::is_expired(item.expires_at),
                    "freshness": freshness,
                    "freshness_icon": freshness.icon(),
                    "freshness_description": freshness.description(),
                })
            })
            .collect();

        Ok(json!({
            "accounts": accounts,
            "login_state": snapshot.login_state,
            "runtime_summary": runtime_summary,
            "current_profile_auth_mode": runtime_summary.current_profile_auth_mode.map(|mode| mode.as_str()),
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn claude_get_auth_current() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        let runtime_summary = service
            .get_runtime_summary()
            .map_err(|e| format!("读取运行时摘要失败: {e}"))?;
        let current_info = service.get_current_auth_info().ok();
        let logged_in = current_info.is_some();

        let info = current_info.map(|info| {
            let freshness = &info.freshness;
            json!({
                "account_uuid": info.account_uuid,
                "email": info.email,
                "billing_type": info.billing_type,
                "subscription_type": info.subscription_type,
                "rate_limit_tier": info.rate_limit_tier,
                "expires_at": info.expires_at.map(|dt| dt.to_rfc3339()),
                "is_expired": ClaudeAuthService::is_expired(info.expires_at),
                "freshness": freshness,
                "freshness_icon": freshness.icon(),
                "freshness_description": freshness.description(),
            })
        });

        Ok(json!({
            "logged_in": logged_in,
            "info": info,
            "runtime_summary": runtime_summary,
            "login_state": runtime_summary.login_state,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn claude_save_auth(
    name: String,
    description: Option<String>,
    force: Option<bool>,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        service
            .save_current(&name, description, force.unwrap_or(false))
            .map_err(|e| format!("{e}"))?;
        Ok(json!({
            "success": true,
            "message": format!("Claude 官方账号 '{}' 已成功保存", name),
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn claude_switch_auth(name: String) -> Result<Value, String> {
    let name_resp = name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        service.switch_account(&name).map_err(|e| format!("{e}"))?;
        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    Ok(json!({
        "success": true,
        "message": format!("已切换到 Claude 官方账号 '{}'", name_resp),
    }))
}

#[tauri::command]
pub async fn claude_delete_auth(name: String) -> Result<Value, String> {
    let name_resp = name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        service.delete_account(&name).map_err(|e| format!("{e}"))?;
        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    Ok(json!({
        "success": true,
        "message": format!("Claude 官方账号 '{}' 已成功删除", name_resp),
    }))
}

/// 列出所有 Claude Code Profiles（~/.ccr/platforms/claude/profiles.toml）。
#[tauri::command]
pub async fn claude_list_profiles() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;
        let profiles: Vec<Value> = platform
            .load_profiles()
            .map_err(|e| format!("读取 Claude profiles 失败: {e}"))?
            .into_iter()
            .map(|(name, profile)| profile_to_json(current_profile.as_deref(), name, profile))
            .collect();

        Ok(json!({
            "profiles": profiles,
            "current_profile": current_profile,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 获取单个 Profile 详情。
#[tauri::command]
pub async fn claude_get_profile(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;
        let profile = platform
            .load_profiles()
            .map_err(|e| format!("读取 Claude profiles 失败: {e}"))?
            .shift_remove(&name)
            .ok_or_else(|| format!("Claude Profile '{name}' 不存在"))?;

        Ok(profile_to_json(current_profile.as_deref(), name, profile))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 创建新 Profile。
#[tauri::command]
pub async fn claude_add_profile(request: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let name = request
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| "Missing 'name' field".to_string())?
            .to_string();

        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Claude profiles 失败: {e}"))?;
        if profiles.contains_key(&name) {
            return Err(format!("Claude Profile '{name}' 已存在"));
        }

        let profile = build_profile_from_config(&request)?;

        platform
            .save_profile(&name, &profile)
            .map_err(|e| format!("保存 Claude Profile 失败: {e}"))?;

        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;

        Ok(profile_to_json(current_profile.as_deref(), name, profile))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 更新 Profile。
#[tauri::command]
pub async fn claude_update_profile(name: String, request: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Claude profiles 失败: {e}"))?;
        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;
        let existing = profiles
            .get(&name)
            .cloned()
            .ok_or_else(|| format!("Claude Profile '{name}' 不存在"))?;

        let target_name = request
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or(name.as_str())
            .to_string();
        if target_name != name && profiles.contains_key(&target_name) {
            return Err(format!("Claude Profile '{target_name}' 已存在"));
        }

        let mut profile = existing;
        patch_profile_with_config(&mut profile, &request)?;

        platform
            .save_profile(&target_name, &profile)
            .map_err(|e| format!("更新 Claude Profile 失败: {e}"))?;

        if target_name != name {
            platform
                .delete_profile(&name)
                .map_err(|e| format!("删除旧 Claude Profile 失败: {e}"))?;

            if current_profile.as_deref() == Some(name.as_str()) {
                platform
                    .apply_profile(&target_name)
                    .map_err(|e| format!("同步当前 Claude Profile 失败: {e}"))?;
            }
        }

        let latest_current = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;

        Ok(profile_to_json(
            latest_current.as_deref(),
            target_name,
            profile,
        ))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 删除 Profile。
#[tauri::command]
pub async fn claude_delete_profile(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        platform
            .delete_profile(&name)
            .map_err(|e| format!("删除 Claude Profile 失败: {e}"))?;
        Ok(json!({ "message": format!("Claude Profile '{name}' 已删除") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 应用 Profile。
#[tauri::command]
pub async fn claude_apply_profile(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        platform
            .apply_profile(&name)
            .map_err(|e| format!("应用 Claude Profile 失败: {e}"))?;
        Ok(json!({
            "success": true,
            "applied_profile": name,
            "message": format!("Claude Profile 已应用"),
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
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
