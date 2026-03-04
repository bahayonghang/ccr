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
