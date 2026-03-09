//! Claude Code 命令 — MCP/Agents/Slash Commands/Plugins/Settings/OutputStyles/
//! Statusline/Hooks/Budgets/Prompts。
//!
//! 所有读写操作直接访问 ~/.claude/settings.json（通过 ccr::SettingsManager）和
//! ~/.claude.json（内联实现 ClaudeConfigManager 逻辑）。

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use ccr::managers::prompts_manager::PromptsManager;
use ccr::models::prompt::PromptPreset;
use ccr::{BudgetManager, CostTracker, SettingsManager};

use tauri::State;

use crate::state::AppState;
use ccr_db::database::repositories::claude_profile_repo;
use ccr_db::models::claude_profile::ClaudeProfile;

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

fn load_settings() -> Result<ccr_types::ClaudeSettings, String> {
    let manager =
        SettingsManager::with_default().map_err(|e| format!("Failed to init manager: {}", e))?;
    let path = manager.settings_path();
    if !path.exists() {
        return Ok(ccr_types::ClaudeSettings::default());
    }
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read settings: {}", e))?;
    let settings: ccr_types::ClaudeSettings =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))?;
    Ok(settings)
}

fn save_settings(settings: &ccr_types::ClaudeSettings) -> Result<(), String> {
    let manager =
        SettingsManager::with_default().map_err(|e| format!("Failed to init manager: {}", e))?;
    let path = manager.settings_path();
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    // 原子写入：先写临时文件，再重命名
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, &content).map_err(|e| format!("Failed to write temp file: {}", e))?;
    fs::rename(&tmp_path, path).map_err(|e| format!("Failed to rename settings file: {}", e))?;
    Ok(())
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
pub async fn claude_get_settings() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let settings = load_settings()?;
        serde_json::to_value(settings).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// 将调用方提供的 JSON 合并写入 ~/.claude/settings.json。
#[tauri::command]
pub async fn claude_update_settings(settings: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let parsed: ccr_types::ClaudeSettings = serde_json::from_value(settings)
            .map_err(|e| format!("Invalid settings payload: {}", e))?;
        save_settings(&parsed)?;
        let result =
            serde_json::to_value(&parsed).map_err(|e| format!("Serialization error: {}", e))?;
        Ok(result)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
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
pub async fn claude_list_agents() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let settings = load_settings()?;
        let agents = serde_json::to_value(&settings.agents)
            .map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "agents": agents }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_add_agent(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut settings = load_settings()?;

        let mut agent: ccr_types::Agent =
            serde_json::from_value(config).map_err(|e| format!("Invalid agent config: {}", e))?;
        // Ensure the name from the parameter takes precedence
        agent.name = name;

        settings.agents.push(agent);
        save_settings(&settings)?;

        let result = serde_json::to_value(&settings.agents)
            .map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "agents": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_update_agent(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut settings = load_settings()?;

        let pos = settings
            .agents
            .iter()
            .position(|a| a.name == name)
            .ok_or_else(|| format!("Agent '{}' not found", name))?;

        let updated: ccr_types::Agent =
            serde_json::from_value(config).map_err(|e| format!("Invalid agent config: {}", e))?;
        settings.agents[pos] = updated;

        save_settings(&settings)?;

        let result = serde_json::to_value(&settings.agents)
            .map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "agents": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_delete_agent(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut settings = load_settings()?;

        let original_len = settings.agents.len();
        settings.agents.retain(|a| a.name != name);

        if settings.agents.len() >= original_len {
            return Err(format!("Agent '{}' not found", name));
        }

        save_settings(&settings)?;
        Ok(format!("Agent '{}' deleted", name))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// ═══════════════════════════════════════════════════════════
// ── Slash Commands（~/.claude/settings.json .slashCommands[]）──
// ═══════════════════════════════════════════════════════════

#[tauri::command]
pub async fn claude_list_slash_commands() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let settings = load_settings()?;
        let commands = serde_json::to_value(&settings.slash_commands)
            .map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "commands": commands }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_add_slash_command(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut settings = load_settings()?;

        let mut cmd: ccr_types::SlashCommand = serde_json::from_value(config)
            .map_err(|e| format!("Invalid slash command config: {}", e))?;
        cmd.name = name;

        settings.slash_commands.push(cmd);
        save_settings(&settings)?;

        let result = serde_json::to_value(&settings.slash_commands)
            .map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "commands": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_update_slash_command(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut settings = load_settings()?;

        let pos = settings
            .slash_commands
            .iter()
            .position(|c| c.name == name)
            .ok_or_else(|| format!("Slash command '{}' not found", name))?;

        let updated: ccr_types::SlashCommand = serde_json::from_value(config)
            .map_err(|e| format!("Invalid slash command config: {}", e))?;
        settings.slash_commands[pos] = updated;

        save_settings(&settings)?;

        let result = serde_json::to_value(&settings.slash_commands)
            .map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "commands": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_delete_slash_command(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut settings = load_settings()?;

        let original_len = settings.slash_commands.len();
        settings.slash_commands.retain(|c| c.name != name);

        if settings.slash_commands.len() >= original_len {
            return Err(format!("Slash command '{}' not found", name));
        }

        save_settings(&settings)?;
        Ok(format!("Slash command '{}' deleted", name))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// ═══════════════════════════════════════════════════════════
// ── Plugins（~/.claude/settings.json .plugins[]）──
// ═══════════════════════════════════════════════════════════

#[tauri::command]
pub async fn claude_list_plugins() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let settings = load_settings()?;
        let plugins = serde_json::to_value(&settings.plugins)
            .map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "plugins": plugins }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_add_plugin(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut settings = load_settings()?;

        let mut plugin: ccr_types::Plugin =
            serde_json::from_value(config).map_err(|e| format!("Invalid plugin config: {}", e))?;
        plugin.name = name;

        settings.plugins.push(plugin);
        save_settings(&settings)?;

        let result = serde_json::to_value(&settings.plugins)
            .map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "plugins": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_update_plugin(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut settings = load_settings()?;

        let pos = settings
            .plugins
            .iter()
            .position(|p| p.name == name)
            .ok_or_else(|| format!("Plugin '{}' not found", name))?;

        let updated: ccr_types::Plugin =
            serde_json::from_value(config).map_err(|e| format!("Invalid plugin config: {}", e))?;
        settings.plugins[pos] = updated;

        save_settings(&settings)?;

        let result = serde_json::to_value(&settings.plugins)
            .map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "plugins": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_delete_plugin(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut settings = load_settings()?;

        let original_len = settings.plugins.len();
        settings.plugins.retain(|p| p.name != name);

        if settings.plugins.len() >= original_len {
            return Err(format!("Plugin '{}' not found", name));
        }

        save_settings(&settings)?;
        Ok(format!("Plugin '{}' deleted", name))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
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
pub async fn claude_get_statusline() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let settings = load_settings()?;
        let statusline = settings
            .other
            .get("statusline")
            .cloned()
            .unwrap_or(Value::Null);
        Ok(statusline)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_update_statusline(config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut settings = load_settings()?;
        settings
            .other
            .insert("statusline".to_string(), config.clone());
        save_settings(&settings)?;
        Ok(config)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// ═══════════════════════════════════════════════════════════
// ── Hooks（~/.claude/settings.json .hooks[]）──
// ═══════════════════════════════════════════════════════════

#[tauri::command]
pub async fn claude_list_hooks() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let settings = load_settings()?;
        let hooks = serde_json::to_value(&settings.hooks)
            .map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "hooks": hooks }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// 整体替换 hooks 列表（hooks 是 [{event, command, enabled, description}] 数组）。
#[tauri::command]
pub async fn claude_update_hooks(hooks: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut settings = load_settings()?;

        let new_hooks: Vec<ccr_types::Hook> =
            serde_json::from_value(hooks).map_err(|e| format!("Invalid hooks payload: {}", e))?;
        settings.hooks = new_hooks;

        save_settings(&settings)?;

        let result = serde_json::to_value(&settings.hooks)
            .map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "hooks": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
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
        let manager = PromptsManager::new(ccr::Platform::Claude)
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
        let manager = PromptsManager::new(ccr::Platform::Claude)
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
// ── Profiles（SQLite via ccr-db）──
// ═══════════════════════════════════════════════════════════

/// 列出所有 Claude Code Profiles。
#[tauri::command]
pub async fn claude_list_profiles(state: State<'_, AppState>) -> Result<Value, String> {
    let pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        let profiles = claude_profile_repo::get_all_profiles(&conn)
            .map_err(|e| format!("Query error: {e}"))?;
        let current_name = profiles.iter().find(|p| p.is_current).map(|p| p.name.clone());

        // 返回摘要而非完整快照，避免将 secrets 暴露给前端
        let summaries: Vec<Value> = profiles
            .iter()
            .map(|p| {
                // 从快照中提取统计信息
                let (mcp_count, style_count) = serde_json::from_str::<Value>(&p.snapshot_json)
                    .map(|snap| {
                        let mc = snap
                            .get("mcp_servers")
                            .and_then(|v| v.as_object())
                            .map(|o| o.len())
                            .unwrap_or(0);
                        let sc = snap
                            .get("output_styles")
                            .and_then(|v| v.as_array())
                            .map(|a| a.len())
                            .unwrap_or(0);
                        (mc, sc)
                    })
                    .unwrap_or((0, 0));

                serde_json::json!({
                    "id": p.id,
                    "name": p.name,
                    "description": p.description,
                    "tags": p.tags,
                    "is_current": p.is_current,
                    "enabled": p.enabled,
                    "created_at": p.created_at.to_rfc3339(),
                    "updated_at": p.updated_at.to_rfc3339(),
                    "snapshot_stats": {
                        "mcp_count": mcp_count,
                        "style_count": style_count,
                    }
                })
            })
            .collect();

        Ok(serde_json::json!({
            "profiles": summaries,
            "current_profile": current_name
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 获取单个 Profile 详情。
#[tauri::command]
pub async fn claude_get_profile(
    state: State<'_, AppState>,
    name: String,
) -> Result<Value, String> {
    let pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        let profile = claude_profile_repo::get_profile_by_name(&conn, &name)
            .map_err(|e| format!("Query error: {e}"))?
            .ok_or_else(|| format!("Profile '{}' not found", name))?;
        serde_json::to_value(&profile).map_err(|e| format!("Serialization error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 从当前活跃配置捕获快照 JSON。
fn capture_current_snapshot() -> Result<String, String> {
    // 1. 读取 settings.json
    let settings = load_settings().unwrap_or_default();
    let settings_value =
        serde_json::to_value(&settings).map_err(|e| format!("Settings serialize error: {e}"))?;

    // 2. 读取 .claude.json (MCP servers)
    let mcp_servers = read_claude_config()
        .map(|c| {
            serde_json::to_value(&c.mcp_servers)
                .unwrap_or_else(|_| serde_json::json!({}))
        })
        .unwrap_or_else(|_| serde_json::json!({}));

    // 3. 扫描 output-styles/*.md
    let output_styles: Vec<Value> = output_styles_dir()
        .and_then(|dir| {
            if !dir.exists() {
                return Ok(Vec::new());
            }
            let mut styles = Vec::new();
            for entry in fs::read_dir(&dir)?.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                if let (Some(stem), Ok(content)) =
                    (path.file_stem().and_then(|s| s.to_str()), fs::read_to_string(&path))
                {
                    styles.push(serde_json::json!({
                        "name": stem,
                        "content": content,
                    }));
                }
            }
            Ok(styles)
        })
        .unwrap_or_default();

    // 4. 读取 budget 配置（可选）
    let budget = BudgetManager::with_default()
        .ok()
        .map(|m| {
            let config = m.get_config();
            serde_json::json!({
                "enabled": config.enabled,
                "dailyLimit": config.daily_limit,
                "weeklyLimit": config.weekly_limit,
                "monthlyLimit": config.monthly_limit,
                "warnAtPercent": config.warn_at_percent,
            })
        });

    // 组合为完整快照
    let snapshot = serde_json::json!({
        "settings": settings_value,
        "mcp_servers": mcp_servers,
        "output_styles": output_styles,
        "budget": budget,
    });

    serde_json::to_string(&snapshot).map_err(|e| format!("Snapshot serialize error: {e}"))
}

/// 创建新 Profile（可从当前配置自动快照）。
#[tauri::command]
pub async fn claude_add_profile(
    state: State<'_, AppState>,
    request: Value,
) -> Result<Value, String> {
    let pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let name = request
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'name' field".to_string())?
            .to_string();
        let description = request
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let tags = request
            .get("tags")
            .map(|v| v.to_string());
        let enabled = request
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let snapshot_from_current = request
            .get("snapshot_from_current")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // 确定快照内容
        let snapshot_json = if snapshot_from_current {
            capture_current_snapshot()?
        } else {
            request
                .get("snapshot_json")
                .and_then(|v| v.as_str())
                .unwrap_or("{}")
                .to_string()
        };

        let mut profile = ClaudeProfile::new(name, snapshot_json);
        profile.description = description;
        profile.tags = tags;
        profile.enabled = enabled;

        let conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        claude_profile_repo::insert_profile(&conn, &profile)
            .map_err(|e| format!("Insert error: {e}"))?;

        serde_json::to_value(&profile).map_err(|e| format!("Serialization error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 更新 Profile。
#[tauri::command]
pub async fn claude_update_profile(
    state: State<'_, AppState>,
    name: String,
    request: Value,
) -> Result<Value, String> {
    let pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        let mut profile = claude_profile_repo::get_profile_by_name(&conn, &name)
            .map_err(|e| format!("Query error: {e}"))?
            .ok_or_else(|| format!("Profile '{}' not found", name))?;

        // 合并更新字段
        if let Some(new_name) = request.get("name").and_then(|v| v.as_str()) {
            profile.name = new_name.to_string();
        }
        if let Some(desc) = request.get("description") {
            profile.description = desc.as_str().map(|s| s.to_string());
        }
        if let Some(tags) = request.get("tags") {
            profile.tags = Some(tags.to_string());
        }
        if let Some(enabled) = request.get("enabled").and_then(|v| v.as_bool()) {
            profile.enabled = enabled;
        }
        if let Some(snapshot) = request.get("snapshot_json").and_then(|v| v.as_str()) {
            profile.snapshot_json = snapshot.to_string();
        }
        // 支持重新从当前配置快照
        if request
            .get("snapshot_from_current")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            profile.snapshot_json = capture_current_snapshot()?;
        }

        claude_profile_repo::update_profile(&conn, &name, &profile)
            .map_err(|e| format!("Update error: {e}"))?;

        serde_json::to_value(&profile).map_err(|e| format!("Serialization error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 删除 Profile。
#[tauri::command]
pub async fn claude_delete_profile(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;

        // 单事务：条件清除 current 标记 + 删除
        let tx = conn.unchecked_transaction().map_err(|e| format!("TX error: {e}"))?;
        tx.execute(
            "UPDATE claude_profiles SET is_current = 0 WHERE name = ?1 AND is_current = 1",
            [&name],
        )
        .map_err(|e| format!("Clear current error: {e}"))?;

        let deleted = tx
            .execute(
                "DELETE FROM claude_profiles WHERE name = ?1",
                [&name],
            )
            .map_err(|e| format!("Delete error: {e}"))?;
        tx.commit().map_err(|e| format!("Commit error: {e}"))?;

        if deleted > 0 {
            Ok(format!("Profile '{}' deleted", name))
        } else {
            Err(format!("Profile '{}' not found", name))
        }
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 应用 Profile — 将快照写回文件系统并设为当前。
#[tauri::command]
pub async fn claude_apply_profile(
    state: State<'_, AppState>,
    name: String,
) -> Result<Value, String> {
    let pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;

        let profile = claude_profile_repo::get_profile_by_name(&conn, &name)
            .map_err(|e| format!("Query error: {e}"))?
            .ok_or_else(|| format!("Profile '{}' not found", name))?;

        let snapshot: Value = serde_json::from_str(&profile.snapshot_json)
            .map_err(|e| format!("Snapshot parse error: {e}"))?;

        // 1. 写入 settings.json
        if let Some(settings_val) = snapshot.get("settings") {
            let settings: ccr_types::ClaudeSettings =
                serde_json::from_value(settings_val.clone())
                    .map_err(|e| format!("Settings parse error: {e}"))?;
            save_settings(&settings)?;
        }

        // 2. 写入 .claude.json (MCP servers)
        if let Some(mcp_val) = snapshot.get("mcp_servers") {
            let mut config = read_claude_config()
                .map_err(|e| format!("Read .claude.json error: {e}"))?;
            let servers: HashMap<String, McpServerEntry> =
                serde_json::from_value(mcp_val.clone())
                    .map_err(|e| format!("Malformed mcp_servers in snapshot: {e}"))?;
            config.mcp_servers = servers;
            write_claude_config(&config)
                .map_err(|e| format!("Write .claude.json error: {e}"))?;
        }

        // 3. 同步 output-styles 目录
        if let Some(styles_arr) = snapshot.get("output_styles").and_then(|v| v.as_array()) {
            let dir = output_styles_dir()
                .map_err(|e| format!("Output styles dir error: {e}"))?;
            fs::create_dir_all(&dir)
                .map_err(|e| format!("Create output-styles dir error: {e}"))?;

            // 清空现有 .md 文件后写入快照中的文件
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("md") {
                        let _ = fs::remove_file(&path);
                    }
                }
            }

            for style in styles_arr {
                if let (Some(sname), Some(content)) =
                    (style.get("name").and_then(|v| v.as_str()),
                     style.get("content").and_then(|v| v.as_str()))
                {
                    // 安全校验：拒绝含路径分隔符或 ".." 的文件名，防止路径穿越
                    if sname.contains('/')
                        || sname.contains('\\')
                        || sname.contains("..")
                        || sname.is_empty()
                    {
                        return Err(format!(
                            "Invalid output style name '{}': must not contain path separators or '..'",
                            sname
                        ));
                    }
                    let path = dir.join(format!("{}.md", sname));
                    // 二次校验：规范路径必须在目标目录内
                    let canonical_dir = dir.canonicalize()
                        .map_err(|e| format!("Canonicalize dir error: {e}"))?;
                    // 写入后再 canonicalize 需要文件存在，先用 starts_with 检查 parent
                    if !path.starts_with(&dir) {
                        return Err(format!(
                            "Output style '{}' resolves outside target directory",
                            sname
                        ));
                    }
                    fs::write(&path, content)
                        .map_err(|e| format!("Write style '{}' error: {e}", sname))?;
                    // 写入后验证规范路径仍在目录内
                    if let Ok(canonical_path) = path.canonicalize()
                        && !canonical_path.starts_with(&canonical_dir)
                    {
                        let _ = fs::remove_file(&path);
                        return Err(format!(
                            "Output style '{}' resolved outside target directory after write",
                            sname
                        ));
                    }
                }
            }
        }

        // 4. 更新 budget 配置（可选）— 完整 round-trip
        if let Some(budget_val) = snapshot.get("budget")
            && !budget_val.is_null()
            && let Ok(mut budget_manager) = BudgetManager::with_default()
        {
            if let Some(enabled) = budget_val.get("enabled").and_then(|v| v.as_bool()) {
                let _ = if enabled {
                    budget_manager.enable()
                } else {
                    budget_manager.disable()
                };
            }
            // 处理 limit 字段：数值 → Some，null/缺失 → None（清除旧值）
            if let Some(daily_val) = budget_val.get("dailyLimit") {
                let _ = budget_manager
                    .set_daily_limit(daily_val.as_f64());
            }
            if let Some(weekly_val) = budget_val.get("weeklyLimit") {
                let _ = budget_manager
                    .set_weekly_limit(weekly_val.as_f64());
            }
            if let Some(monthly_val) = budget_val.get("monthlyLimit") {
                let _ = budget_manager
                    .set_monthly_limit(monthly_val.as_f64());
            }
            // 恢复 warnAtPercent
            if let Some(warn_pct) = budget_val
                .get("warnAtPercent")
                .and_then(|v| v.as_u64())
            {
                let _ = budget_manager
                    .set_warn_threshold(warn_pct as u8);
            }
        }

        // 5. 设为当前 Profile
        claude_profile_repo::set_current_profile(&conn, &name)
            .map_err(|e| format!("Set current error: {e}"))?;

        Ok(serde_json::json!({
            "success": true,
            "applied_profile": name,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
