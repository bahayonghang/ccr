//! 统一 MCP 管理命令 — 跨平台 MCP 服务器聚合 CRUD。
//!
//! Claude Code 使用 `claude_mcp_config` 统一解析 local/project/user scope 与
//! precedence；Codex/Gemini 暂保留既有配置语义。

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use crate::commands::claude_mcp_config::{
    ClaudeMcpDiagnostic, add_claude_mcp_server_default, delete_claude_mcp_server_default,
    list_claude_mcp_default, parse_scope, update_claude_mcp_server_default,
};

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
    pub headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_tools: Vec<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_state: Option<String>,
    #[serde(default)]
    pub effective: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_config: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformMcpCapability {
    pub platform: String,
    pub supports_toggle: bool,
    pub supports_url: bool,
    pub supports_headers: bool,
    pub supports_timeout: bool,
    pub supports_cwd: bool,
    pub supports_trust: bool,
    pub supports_include_tools: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnifiedMcpRequest {
    pub platform: String,
    pub name: String,
    #[serde(default)]
    pub scope: Option<String>,
    pub command: Option<String>,
    pub url: Option<String>,
    #[serde(default, deserialize_with = "default_when_null")]
    pub args: Vec<String>,
    #[serde(default, deserialize_with = "default_when_null")]
    pub env: HashMap<String, String>,
    #[serde(default, deserialize_with = "default_when_null")]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub timeout: Option<i64>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub trust: Option<bool>,
    #[serde(default, deserialize_with = "default_when_null")]
    pub include_tools: Vec<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
}

fn default_when_null<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

fn home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Cannot determine home directory".to_string())
}

fn antigravity_mcp_config_path() -> Result<PathBuf, String> {
    Ok(home_dir()?
        .join(".gemini")
        .join("antigravity-cli")
        .join("mcp_config.json"))
}

fn value_string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
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

    let Some(servers) = config.get("mcp_servers").and_then(Value::as_object) else {
        return Ok(vec![]);
    };

    Ok(servers
        .iter()
        .map(|(name, v)| UnifiedMcpServer {
            platform: "codex".into(),
            name: name.clone(),
            command: v.get("command").and_then(Value::as_str).map(String::from),
            url: v.get("url").and_then(Value::as_str).map(String::from),
            args: value_string_array(v.get("args")),
            env: v
                .get("env")
                .and_then(|e| serde_json::from_value(e.clone()).ok())
                .unwrap_or_default(),
            headers: HashMap::new(),
            timeout: None,
            cwd: None,
            trust: None,
            include_tools: Vec::new(),
            disabled: false,
            scope: None,
            source_path: Some(path.to_string_lossy().to_string()),
            approval_state: None,
            effective: true,
            hidden_by: None,
            raw_config: None,
        })
        .collect())
}

fn list_gemini_mcp() -> Result<Vec<UnifiedMcpServer>, String> {
    let path = antigravity_mcp_config_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Read antigravity mcp config: {e}"))?;
    let config: Value =
        serde_json::from_str(&content).map_err(|e| format!("Parse antigravity mcp config: {e}"))?;

    let Some(servers) = config.get("mcpServers").and_then(Value::as_object) else {
        return Ok(vec![]);
    };

    Ok(servers
        .iter()
        .map(|(name, v)| UnifiedMcpServer {
            platform: "gemini".into(),
            name: name.clone(),
            command: v.get("command").and_then(Value::as_str).map(String::from),
            url: v
                .get("serverUrl")
                .or_else(|| v.get("url"))
                .or_else(|| v.get("httpUrl"))
                .and_then(Value::as_str)
                .map(String::from),
            args: value_string_array(v.get("args")),
            env: v
                .get("env")
                .and_then(|e| serde_json::from_value(e.clone()).ok())
                .unwrap_or_default(),
            headers: HashMap::new(),
            timeout: None,
            cwd: None,
            trust: None,
            include_tools: Vec::new(),
            disabled: false,
            scope: None,
            source_path: Some(path.to_string_lossy().to_string()),
            approval_state: None,
            effective: true,
            hidden_by: None,
            raw_config: None,
        })
        .collect())
}

fn capabilities() -> Vec<PlatformMcpCapability> {
    vec![
        PlatformMcpCapability {
            platform: "claude".into(),
            supports_toggle: true,
            supports_url: true,
            supports_headers: true,
            supports_timeout: true,
            supports_cwd: true,
            supports_trust: true,
            supports_include_tools: true,
        },
        PlatformMcpCapability {
            platform: "codex".into(),
            supports_toggle: false,
            supports_url: true,
            supports_headers: false,
            supports_timeout: false,
            supports_cwd: false,
            supports_trust: false,
            supports_include_tools: false,
        },
        PlatformMcpCapability {
            platform: "gemini".into(),
            supports_toggle: false,
            supports_url: true,
            supports_headers: false,
            supports_timeout: false,
            supports_cwd: false,
            supports_trust: false,
            supports_include_tools: false,
        },
    ]
}

#[ccr_tauri_command_macros::command]
pub async fn unified_list_mcp_servers(
    platforms: Option<Vec<String>>,
) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let all = ["claude", "codex", "gemini"];
        let targets: Vec<&str> = match &platforms {
            Some(p) if !p.is_empty() => all
                .iter()
                .filter(|a| p.iter().any(|b| b.eq_ignore_ascii_case(a)))
                .copied()
                .collect(),
            _ => all.to_vec(),
        };

        let mut servers = Vec::new();
        let mut diagnostics: Vec<ClaudeMcpDiagnostic> = Vec::new();
        for platform in &targets {
            match *platform {
                "claude" => match list_claude_mcp_default() {
                    Ok(list) => {
                        diagnostics.extend(list.diagnostics);
                        servers.extend(list.servers.into_iter().map(|server| UnifiedMcpServer {
                            platform: server.platform,
                            name: server.name,
                            command: server.command,
                            url: server.url,
                            args: server.args,
                            env: server.env,
                            headers: server.headers,
                            timeout: server.timeout,
                            cwd: server.cwd,
                            trust: server.trust,
                            include_tools: server.include_tools,
                            disabled: server.disabled,
                            scope: Some(server.scope),
                            source_path: server.source_path,
                            approval_state: server.approval_state,
                            effective: server.effective,
                            hidden_by: server.hidden_by,
                            raw_config: server.raw_config,
                        }));
                    }
                    Err(e) => {
                        tracing::warn!(platform, error = %e, "Failed to list MCP servers");
                        diagnostics.push(ClaudeMcpDiagnostic {
                            level: "error".into(),
                            message: e,
                            source_path: None,
                            scope: Some("claude".into()),
                            matched: None,
                        });
                    }
                },
                "codex" => match list_codex_mcp() {
                    Ok(s) => servers.extend(s),
                    Err(e) => tracing::warn!(platform, error = %e, "Failed to list MCP servers"),
                },
                "gemini" => match list_gemini_mcp() {
                    Ok(s) => servers.extend(s),
                    Err(e) => tracing::warn!(platform, error = %e, "Failed to list MCP servers"),
                },
                _ => {}
            }
        }

        let caps: Vec<_> = capabilities()
            .into_iter()
            .filter(|c| targets.contains(&c.platform.as_str()))
            .collect();
        let total = servers.len();

        Ok(serde_json::json!({
            "servers": servers,
            "capabilities": caps,
            "diagnostics": diagnostics,
            "total": total,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

fn request_to_config(request: &UnifiedMcpRequest) -> Value {
    let mut config = Map::new();
    let has_command = request
        .command
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    let has_url = request
        .url
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());

    if let Some(command) = request
        .command
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        config.insert("command".into(), Value::String(command.to_string()));
    } else if request.platform == "claude" && has_url {
        config.insert("command".into(), Value::Null);
    }

    if let Some(url) = request
        .url
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        if request.platform == "claude" {
            config.insert("type".into(), Value::String("http".into()));
        }
        let url_key = if request.platform == "gemini" {
            "serverUrl"
        } else {
            "url"
        };
        config.insert(url_key.into(), Value::String(url.to_string()));
    } else if request.platform == "claude" && has_command {
        config.insert("type".into(), Value::Null);
        config.insert("url".into(), Value::Null);
    }

    if !request.args.is_empty() {
        config.insert(
            "args".into(),
            Value::Array(request.args.iter().cloned().map(Value::String).collect()),
        );
    } else if request.platform == "claude" && has_url {
        config.insert("args".into(), Value::Null);
    }

    if !request.env.is_empty() {
        config.insert(
            "env".into(),
            serde_json::to_value(&request.env).unwrap_or_else(|_| Value::Object(Map::new())),
        );
    }

    if request.platform == "claude" {
        if !request.headers.is_empty() {
            config.insert(
                "headers".into(),
                serde_json::to_value(&request.headers)
                    .unwrap_or_else(|_| Value::Object(Map::new())),
            );
        }
        if let Some(timeout) = request.timeout {
            config.insert("timeout".into(), Value::Number(timeout.into()));
        }
        if let Some(cwd) = request.cwd.as_ref().filter(|value| !value.is_empty()) {
            config.insert("cwd".into(), Value::String(cwd.clone()));
        }
        if let Some(trust) = request.trust {
            config.insert("trust".into(), Value::Bool(trust));
        }
        if !request.include_tools.is_empty() {
            config.insert(
                "include_tools".into(),
                Value::Array(
                    request
                        .include_tools
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect(),
                ),
            );
        }
        if let Some(disabled) = request.disabled {
            config.insert("disabled".into(), Value::Bool(disabled));
        }
    } else if request.disabled.unwrap_or(false) {
        // Codex/Gemini keep the previous unified add/delete semantics and do
        // not receive Claude-only fields such as headers, trust, or scope.
        config.insert("disabled".into(), Value::Bool(true));
    }

    Value::Object(config)
}

#[ccr_tauri_command_macros::command]
pub async fn unified_add_mcp_server(
    request: UnifiedMcpRequest,
) -> Result<serde_json::Value, String> {
    if request.name.trim().is_empty() {
        return Err("name cannot be empty".to_string());
    }
    let has_command = request
        .command
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    let has_url = request
        .url
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    if !has_command && !has_url {
        return Err("Must provide command (stdio) or url (http)".to_string());
    }

    let platform = request.platform.clone();
    let name = request.name.clone();

    tokio::task::spawn_blocking(move || {
        let config = request_to_config(&request);

        match platform.as_str() {
            "claude" => {
                add_claude_mcp_server_default(
                    name.clone(),
                    config,
                    parse_scope(request.scope.as_deref()),
                )?;
            }
            "codex" | "gemini" => {
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

#[ccr_tauri_command_macros::command]
pub async fn unified_update_mcp_server(
    platform: String,
    name: String,
    request: UnifiedMcpRequest,
) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let config = request_to_config(&request);
        match platform.as_str() {
            "claude" => update_claude_mcp_server_default(
                name,
                config,
                parse_scope(request.scope.as_deref()),
            ),
            _ => Err(format!(
                "Unified MCP update is only implemented for Claude; platform {platform} still uses add/delete"
            )),
        }
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn unified_delete_mcp_server(
    platform: String,
    name: String,
    scope: Option<String>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        match platform.as_str() {
            "claude" => {
                delete_claude_mcp_server_default(name.clone(), parse_scope(scope.as_deref()))?;
            }
            "codex" | "gemini" => {
                let (path, key) = platform_config_info(&platform)?;
                let mut cfg = read_json_config(&path)?;
                let removed = cfg
                    .as_object_mut()
                    .and_then(|o| o.get_mut(&key).and_then(Value::as_object_mut))
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

fn platform_config_info(platform: &str) -> Result<(PathBuf, String), String> {
    let home = home_dir()?;
    match platform {
        "codex" => Ok((
            home.join(".codex").join("config.toml"),
            "mcp_servers".into(),
        )),
        "gemini" => Ok((antigravity_mcp_config_path()?, "mcpServers".into())),
        _ => Err(format!("Unknown platform: {platform}")),
    }
}

fn read_json_config(path: &PathBuf) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    let content = fs::read_to_string(path).map_err(|e| format!("Read config: {e}"))?;

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
        let toml_val: toml::Value =
            serde_json::from_value(config.clone()).map_err(|e| format!("Convert to TOML: {e}"))?;
        toml::to_string_pretty(&toml_val).map_err(|e| format!("Serialize TOML: {e}"))?
    } else {
        serde_json::to_string_pretty(config).map_err(|e| format!("Serialize JSON: {e}"))?
    };

    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &content).map_err(|e| format!("Write temp file: {e}"))?;
    fs::rename(&tmp, path).map_err(|e| format!("Rename config: {e}"))?;
    Ok(())
}
