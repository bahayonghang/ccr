//! 统一 MCP 管理命令 — 跨平台 MCP 服务器聚合 CRUD。
//!
//! 适配器模式：读取各平台配置文件，以统一的 `UnifiedMcpServer` 格式返回，
//! 写入时根据 `platform` 字段分发到对应平台的配置文件。

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── 统一数据模型 ──

/// 统一 MCP 服务器表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMcpServer {
    pub platform: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub disabled: bool,
}

/// 平台能力矩阵
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformMcpCapability {
    pub platform: String,
    pub supports_toggle: bool,
    pub supports_url: bool,
}

/// 添加/更新请求
#[derive(Debug, Clone, Deserialize)]
pub struct UnifiedMcpRequest {
    pub platform: String,
    pub name: String,
    pub command: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub disabled: bool,
}

// ── 工具函数 ──

fn home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Cannot determine home directory".to_string())
}

// ── 平台适配器 — 列表 ──

fn list_claude_mcp() -> Result<Vec<UnifiedMcpServer>, String> {
    let path = home_dir()?.join(".claude.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Read .claude.json: {e}"))?;
    let config: Value =
        serde_json::from_str(&content).map_err(|e| format!("Parse .claude.json: {e}"))?;

    let Some(servers) = config.get("mcpServers").and_then(|v| v.as_object()) else {
        return Ok(vec![]);
    };

    Ok(servers
        .iter()
        .map(|(name, v)| UnifiedMcpServer {
            platform: "claude".into(),
            name: name.clone(),
            command: v.get("command").and_then(|c| c.as_str()).map(String::from),
            url: v.get("url").and_then(|u| u.as_str()).map(String::from),
            args: v
                .get("args")
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            env: v
                .get("env")
                .and_then(|e| serde_json::from_value(e.clone()).ok())
                .unwrap_or_default(),
            disabled: v.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false),
        })
        .collect())
}

fn list_codex_mcp() -> Result<Vec<UnifiedMcpServer>, String> {
    let path = home_dir()?.join(".codex").join("config.toml");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Read codex config: {e}"))?;
    let config: Value = toml::from_str::<toml::Value>(&content)
        .map_err(|e| format!("Parse codex config: {e}"))
        .and_then(|v| serde_json::to_value(v).map_err(|e| format!("Convert codex config: {e}")))?;

    let Some(servers) = config.get("mcp_servers").and_then(|v| v.as_object()) else {
        return Ok(vec![]);
    };

    Ok(servers
        .iter()
        .map(|(name, v)| UnifiedMcpServer {
            platform: "codex".into(),
            name: name.clone(),
            command: v.get("command").and_then(|c| c.as_str()).map(String::from),
            url: v.get("url").and_then(|u| u.as_str()).map(String::from),
            args: v
                .get("args")
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            env: v
                .get("env")
                .and_then(|e| serde_json::from_value(e.clone()).ok())
                .unwrap_or_default(),
            disabled: false,
        })
        .collect())
}

fn list_gemini_mcp() -> Result<Vec<UnifiedMcpServer>, String> {
    let path = home_dir()?.join(".gemini").join("settings.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Read gemini settings: {e}"))?;
    let config: Value =
        serde_json::from_str(&content).map_err(|e| format!("Parse gemini settings: {e}"))?;

    let Some(servers) = config.get("mcpServers").and_then(|v| v.as_object()) else {
        return Ok(vec![]);
    };

    Ok(servers
        .iter()
        .map(|(name, v)| UnifiedMcpServer {
            platform: "gemini".into(),
            name: name.clone(),
            command: v.get("command").and_then(|c| c.as_str()).map(String::from),
            url: None,
            args: v
                .get("args")
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            env: v
                .get("env")
                .and_then(|e| serde_json::from_value(e.clone()).ok())
                .unwrap_or_default(),
            disabled: false,
        })
        .collect())
}

fn list_droid_mcp() -> Result<Vec<UnifiedMcpServer>, String> {
    let path = home_dir()?.join(".factory").join("settings.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Read droid settings: {e}"))?;
    let config: Value =
        serde_json::from_str(&content).map_err(|e| format!("Parse droid settings: {e}"))?;

    let Some(servers) = config.get("mcpServers").and_then(|v| v.as_object()) else {
        return Ok(vec![]);
    };

    Ok(servers
        .iter()
        .map(|(name, v)| UnifiedMcpServer {
            platform: "droid".into(),
            name: name.clone(),
            command: v.get("command").and_then(|c| c.as_str()).map(String::from),
            url: v.get("url").and_then(|u| u.as_str()).map(String::from),
            args: v
                .get("args")
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            env: v
                .get("env")
                .and_then(|e| serde_json::from_value(e.clone()).ok())
                .unwrap_or_default(),
            disabled: false,
        })
        .collect())
}

// ── 能力矩阵 ──

fn capabilities() -> Vec<PlatformMcpCapability> {
    vec![
        PlatformMcpCapability {
            platform: "claude".into(),
            supports_toggle: true,
            supports_url: true,
        },
        PlatformMcpCapability {
            platform: "codex".into(),
            supports_toggle: false,
            supports_url: true,
        },
        PlatformMcpCapability {
            platform: "gemini".into(),
            supports_toggle: false,
            supports_url: false,
        },
        PlatformMcpCapability {
            platform: "droid".into(),
            supports_toggle: false,
            supports_url: true,
        },
    ]
}

// ═══════════════════════════════════════════════════════════
// ── Tauri Commands ──
// ═══════════════════════════════════════════════════════════

/// 列出所有（或指定）平台的 MCP 服务器
#[tauri::command]
pub async fn unified_list_mcp_servers(
    platforms: Option<Vec<String>>,
) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let all = ["claude", "codex", "gemini", "droid"];
        let targets: Vec<&str> = match &platforms {
            Some(p) if !p.is_empty() => all
                .iter()
                .filter(|a| p.iter().any(|b| b.eq_ignore_ascii_case(a)))
                .copied()
                .collect(),
            _ => all.to_vec(),
        };

        let mut servers = Vec::new();
        for platform in &targets {
            let result = match *platform {
                "claude" => list_claude_mcp(),
                "codex" => list_codex_mcp(),
                "gemini" => list_gemini_mcp(),
                "droid" => list_droid_mcp(),
                _ => continue,
            };
            match result {
                Ok(s) => servers.extend(s),
                Err(e) => tracing::warn!(platform, error = %e, "Failed to list MCP servers"),
            }
        }

        let caps: Vec<_> = capabilities()
            .into_iter()
            .filter(|c| targets.contains(&c.platform.as_str()))
            .collect();

        Ok(serde_json::json!({
            "servers": servers,
            "capabilities": caps,
            "total": servers.len(),
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 添加 MCP 服务器到指定平台（分发到对应平台的 config 文件）
#[tauri::command]
pub async fn unified_add_mcp_server(
    request: UnifiedMcpRequest,
) -> Result<serde_json::Value, String> {
    if request.name.is_empty() {
        return Err("name cannot be empty".to_string());
    }
    if request.command.is_none() && request.url.is_none() {
        return Err("Must provide command (stdio) or url (http)".to_string());
    }

    let platform = request.platform.clone();
    let name = request.name.clone();

    // 分发到对应平台的 invoke 命令逻辑
    // 这里复用前端已有的 per-platform 逻辑：构建 config Value 并调用对应写入
    tokio::task::spawn_blocking(move || {
        let config = serde_json::json!({
            "command": request.command,
            "url": request.url,
            "args": request.args,
            "env": request.env,
            "disabled": request.disabled,
        });

        match platform.as_str() {
            "claude" => {
                let mut cc = read_claude_config_for_unified()?;
                let entry: serde_json::Map<String, Value> =
                    serde_json::from_value(config).map_err(|e| format!("Invalid config: {e}"))?;
                cc.as_object_mut()
                    .and_then(|o| o.get_mut("mcpServers").and_then(|s| s.as_object_mut()))
                    .map(|servers| servers.insert(name.clone(), Value::Object(entry)));
                write_claude_config_for_unified(&cc)?;
            }
            "codex" | "gemini" | "droid" => {
                let (path, key) = platform_config_info(&platform)?;
                let mut cfg = read_json_config(&path)?;
                let servers = cfg
                    .as_object_mut()
                    .ok_or("Config is not an object")?
                    .entry(key)
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
                servers
                    .as_object_mut()
                    .ok_or("mcpServers is not an object")?
                    .insert(name.clone(), config);
                write_json_config(&path, &cfg)?;
            }
            _ => return Err(format!("Unsupported platform: {platform}")),
        }

        Ok(serde_json::json!({
            "message": format!("MCP server '{}' added to {}", name, platform)
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 删除指定平台的 MCP 服务器
#[tauri::command]
pub async fn unified_delete_mcp_server(platform: String, name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        match platform.as_str() {
            "claude" => {
                let mut cc = read_claude_config_for_unified()?;
                let removed = cc
                    .as_object_mut()
                    .and_then(|o| o.get_mut("mcpServers").and_then(|s| s.as_object_mut()))
                    .and_then(|servers| servers.remove(&name));
                if removed.is_none() {
                    return Err(format!("MCP server '{name}' not found on {platform}"));
                }
                write_claude_config_for_unified(&cc)?;
            }
            "codex" | "gemini" | "droid" => {
                let (path, key) = platform_config_info(&platform)?;
                let mut cfg = read_json_config(&path)?;
                let removed = cfg
                    .as_object_mut()
                    .and_then(|o| o.get_mut(&key).and_then(|s| s.as_object_mut()))
                    .and_then(|servers| servers.remove(&name));
                if removed.is_none() {
                    return Err(format!("MCP server '{name}' not found on {platform}"));
                }
                write_json_config(&path, &cfg)?;
            }
            _ => return Err(format!("Unsupported platform: {platform}")),
        }
        Ok(format!("MCP server '{name}' deleted from {platform}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

// ── 内部工具函数 ──

fn platform_config_info(platform: &str) -> Result<(PathBuf, String), String> {
    let home = home_dir()?;
    match platform {
        "codex" => Ok((
            home.join(".codex").join("config.toml"),
            "mcp_servers".into(),
        )),
        "gemini" => Ok((
            home.join(".gemini").join("settings.json"),
            "mcpServers".into(),
        )),
        "droid" => Ok((
            home.join(".factory").join("settings.json"),
            "mcpServers".into(),
        )),
        _ => Err(format!("Unknown platform: {platform}")),
    }
}

fn read_json_config(path: &PathBuf) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    let content = fs::read_to_string(path).map_err(|e| format!("Read config: {e}"))?;

    // codex uses TOML
    if path.extension().and_then(|e| e.to_str()) == Some("toml") {
        let toml_val: toml::Value =
            toml::from_str(&content).map_err(|e| format!("Parse TOML: {e}"))?;
        return serde_json::to_value(toml_val).map_err(|e| format!("Convert TOML: {e}"));
    }

    serde_json::from_str(&content).map_err(|e| format!("Parse JSON: {e}"))
}

fn write_json_config(path: &PathBuf, config: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Create dir: {e}"))?;
    }

    let content = if path.extension().and_then(|e| e.to_str()) == Some("toml") {
        // codex: convert back to TOML
        let toml_val: toml::Value =
            serde_json::from_value(config.clone()).map_err(|e| format!("Convert to TOML: {e}"))?;
        toml::to_string_pretty(&toml_val).map_err(|e| format!("Serialize TOML: {e}"))?
    } else {
        serde_json::to_string_pretty(config).map_err(|e| format!("Serialize JSON: {e}"))?
    };

    // 原子写入
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &content).map_err(|e| format!("Write temp file: {e}"))?;
    fs::rename(&tmp, path).map_err(|e| format!("Rename config: {e}"))?;
    Ok(())
}

fn read_claude_config_for_unified() -> Result<Value, String> {
    let path = home_dir()?.join(".claude.json");
    if !path.exists() {
        return Ok(serde_json::json!({"mcpServers": {}}));
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Read .claude.json: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Parse .claude.json: {e}"))
}

fn write_claude_config_for_unified(config: &Value) -> Result<(), String> {
    let path = home_dir()?.join(".claude.json");
    let content =
        serde_json::to_string_pretty(config).map_err(|e| format!("Serialize .claude.json: {e}"))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &content).map_err(|e| format!("Write temp: {e}"))?;
    fs::rename(&tmp, &path).map_err(|e| format!("Rename .claude.json: {e}"))?;
    Ok(())
}
