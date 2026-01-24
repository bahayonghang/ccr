// Gemini CLI API 处理器
//
// 使用统一的响应工具模块减少重复代码

use axum::{Json, extract::Path, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::api::handlers::response::{bad_request, internal_error, ok, ok_message};
use crate::managers::config::gemini_manager::GeminiConfigManager;
use crate::models::platforms::gemini::{GeminiConfig, GeminiMcpServer, GeminiMcpServerRequest};

const PLATFORM: &str = "Gemini";

// ============ 辅助宏 ============

/// 初始化 Manager 并处理错误
macro_rules! with_gemini_manager {
    ($body:expr) => {
        match GeminiConfigManager::default() {
            Ok(manager) => $body(manager),
            Err(e) => internal_error(format!("初始化 {} 配置管理器失败: {}", PLATFORM, e))
                .into_response(),
        }
    };
}

// ============ MCP 服务器管理 ============

/// GET /api/gemini/mcp - 列出所有 MCP 服务器
pub async fn list_gemini_mcp_servers() -> impl IntoResponse {
    with_gemini_manager!(|manager: GeminiConfigManager| {
        match manager.list_mcp_servers() {
            Ok(servers) => {
                let servers_vec: Vec<_> = servers
                    .into_iter()
                    .map(|(name, server)| {
                        json!({
                            "name": name,
                            "command": server.command,
                            "args": server.args,
                            "env": server.env,
                            "cwd": server.cwd,
                            "timeout": server.timeout,
                            "trust": server.trust,
                            "includeTools": server.include_tools,
                        })
                    })
                    .collect();
                ok(servers_vec).into_response()
            }
            Err(e) => internal_error(format!("读取 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

/// POST /api/gemini/mcp - 添加 MCP 服务器
pub async fn add_gemini_mcp_server(
    Json(request): Json<GeminiMcpServerRequest>,
) -> impl IntoResponse {
    with_gemini_manager!(|manager: GeminiConfigManager| {
        let server = GeminiMcpServer {
            command: request.command,
            args: request.args,
            env: request.env,
            cwd: request.cwd,
            timeout: request.timeout,
            trust: request.trust,
            include_tools: request.include_tools,
            other: HashMap::new(),
        };

        match manager.add_mcp_server(request.name.clone(), server) {
            Ok(()) => ok_message(format!("MCP 服务器 '{}' 添加成功", request.name)).into_response(),
            Err(e) => bad_request(format!("添加 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

/// PUT /api/gemini/mcp/:name - 更新 MCP 服务器
pub async fn update_gemini_mcp_server(
    Path(name): Path<String>,
    Json(request): Json<GeminiMcpServerRequest>,
) -> impl IntoResponse {
    with_gemini_manager!(|manager: GeminiConfigManager| {
        let server = GeminiMcpServer {
            command: request.command,
            args: request.args,
            env: request.env,
            cwd: request.cwd,
            timeout: request.timeout,
            trust: request.trust,
            include_tools: request.include_tools,
            other: HashMap::new(),
        };

        match manager.update_mcp_server(&name, server) {
            Ok(()) => ok_message(format!("MCP 服务器 '{}' 更新成功", name)).into_response(),
            Err(e) => bad_request(format!("更新 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

/// DELETE /api/gemini/mcp/:name - 删除 MCP 服务器
pub async fn delete_gemini_mcp_server(Path(name): Path<String>) -> impl IntoResponse {
    with_gemini_manager!(|manager: GeminiConfigManager| {
        match manager.delete_mcp_server(&name) {
            Ok(()) => ok_message(format!("MCP 服务器 '{}' 删除成功", name)).into_response(),
            Err(e) => bad_request(format!("删除 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

// ============ 基础配置管理 ============

/// GET /api/gemini/config - 获取完整配置
pub async fn get_gemini_config() -> impl IntoResponse {
    with_gemini_manager!(|manager: GeminiConfigManager| {
        match manager.get_config() {
            Ok(config) => ok(config).into_response(),
            Err(e) => internal_error(format!("读取配置失败: {}", e)).into_response(),
        }
    })
}

/// PUT /api/gemini/config - 更新完整配置
pub async fn update_gemini_config(Json(config): Json<GeminiConfig>) -> impl IntoResponse {
    with_gemini_manager!(|manager: GeminiConfigManager| {
        match manager.update_config(&config) {
            Ok(()) => ok_message("配置更新成功").into_response(),
            Err(e) => bad_request(format!("更新配置失败: {}", e)).into_response(),
        }
    })
}

#[derive(Debug, Deserialize)]
pub(crate) struct UiSlashCommandRequest {
    pub name: String,
    pub description: String,
    pub command: String,
    #[serde(default)]
    pub folder: String,
}

#[derive(Debug, Serialize)]
struct TomlCommandFile {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn find_project_root() -> Result<PathBuf, String> {
    let start = std::env::current_dir().map_err(|e| format!("无法获取当前目录: {}", e))?;
    let mut current = start.as_path();
    loop {
        if current.join(".git").exists() {
            return Ok(current.to_path_buf());
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return Ok(start),
        }
    }
}

fn gemini_commands_dirs() -> Result<(PathBuf, PathBuf), String> {
    let project_root = find_project_root()?;
    let project = project_root.join(".gemini").join("commands");
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let user = home.join(".gemini").join("commands");
    Ok((project, user))
}

fn list_toml_commands(
    project_dir: &PathBuf,
    user_dir: &PathBuf,
) -> Result<Vec<crate::models::api::SlashCommand>, String> {
    let mut chosen: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();

    for base in [project_dir, user_dir] {
        if !base.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(base)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }
            let rel = path.strip_prefix(base).map_err(|e| e.to_string())?;
            let rel_no_ext = rel.with_extension("");
            let key = rel_no_ext.to_string_lossy().replace('\\', "/");
            chosen.entry(key).or_insert_with(|| path.to_path_buf());
        }
    }

    let mut commands = Vec::new();
    for (key, path) in chosen {
        let content = std::fs::read_to_string(&path).map_err(|e| format!("读取命令失败: {}", e))?;
        let value: toml::Value =
            toml::from_str(&content).map_err(|e| format!("解析 TOML 失败: {}", e))?;
        let prompt = value
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("缺少 prompt 字段: {}", key))?
            .to_string();
        let description = value
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                prompt
                    .lines()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or("Command")
                    .trim()
                    .to_string()
            });

        let (folder, name) = match key.rsplit_once('/') {
            Some((f, n)) => (f.to_string(), n.to_string()),
            None => (String::new(), key.clone()),
        };

        commands.push(crate::models::api::SlashCommand {
            name,
            description,
            command: prompt,
            args: None,
            disabled: false,
            folder,
        });
    }

    commands.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(commands)
}

pub async fn list_gemini_slash_commands() -> impl IntoResponse {
    let (project_dir, user_dir) = match gemini_commands_dirs() {
        Ok(d) => d,
        Err(e) => return internal_error(e).into_response(),
    };

    let commands = match list_toml_commands(&project_dir, &user_dir) {
        Ok(c) => c,
        Err(e) => return internal_error(e).into_response(),
    };

    let folders: Vec<String> = commands
        .iter()
        .filter_map(|c| {
            if c.folder.is_empty() {
                None
            } else {
                Some(c.folder.clone())
            }
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    ok(json!({ "commands": commands, "folders": folders })).into_response()
}

pub async fn add_gemini_slash_command(Json(req): Json<UiSlashCommandRequest>) -> impl IntoResponse {
    let (project_dir, _user_dir) = match gemini_commands_dirs() {
        Ok(d) => d,
        Err(e) => return internal_error(e).into_response(),
    };

    let mut target_dir = project_dir;
    if !req.folder.trim().is_empty() {
        target_dir = target_dir.join(req.folder.trim());
    }

    if let Err(e) = std::fs::create_dir_all(&target_dir) {
        return internal_error(format!("创建目录失败: {}", e)).into_response();
    }

    let file_path = target_dir.join(format!("{}.toml", req.name));
    let file = TomlCommandFile {
        prompt: req.command,
        description: Some(req.description),
    };

    let toml_str = match toml::to_string_pretty(&file) {
        Ok(s) => s,
        Err(e) => return internal_error(format!("序列化 TOML 失败: {}", e)).into_response(),
    };

    match std::fs::write(&file_path, toml_str) {
        Ok(_) => ok_message("Gemini command created successfully").into_response(),
        Err(e) => internal_error(format!("写入文件失败: {}", e)).into_response(),
    }
}

fn find_command_file_by_name(
    project_dir: &PathBuf,
    user_dir: &PathBuf,
    name: &str,
) -> Result<Option<PathBuf>, String> {
    let mut matches = Vec::new();
    for base in [project_dir, user_dir] {
        if !base.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(base)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }
            if path.file_stem().and_then(|s| s.to_str()) == Some(name) {
                matches.push(path.to_path_buf());
            }
        }
    }
    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches.remove(0))),
        _ => Err("存在多个同名命令文件，请先重命名以避免歧义".to_string()),
    }
}

pub async fn update_gemini_slash_command(
    Path(name): Path<String>,
    Json(req): Json<UiSlashCommandRequest>,
) -> impl IntoResponse {
    let (project_dir, user_dir) = match gemini_commands_dirs() {
        Ok(d) => d,
        Err(e) => return internal_error(e).into_response(),
    };

    let target = match find_command_file_by_name(&project_dir, &user_dir, &name) {
        Ok(Some(p)) => p,
        Ok(None) => return bad_request("命令不存在").into_response(),
        Err(e) => return bad_request(e).into_response(),
    };

    let file = TomlCommandFile {
        prompt: req.command,
        description: Some(req.description),
    };

    let toml_str = match toml::to_string_pretty(&file) {
        Ok(s) => s,
        Err(e) => return internal_error(format!("序列化 TOML 失败: {}", e)).into_response(),
    };

    match std::fs::write(&target, toml_str) {
        Ok(_) => ok_message("Gemini command updated successfully").into_response(),
        Err(e) => internal_error(format!("写入文件失败: {}", e)).into_response(),
    }
}

pub async fn delete_gemini_slash_command(Path(name): Path<String>) -> impl IntoResponse {
    let (project_dir, user_dir) = match gemini_commands_dirs() {
        Ok(d) => d,
        Err(e) => return internal_error(e).into_response(),
    };

    let target = match find_command_file_by_name(&project_dir, &user_dir, &name) {
        Ok(Some(p)) => p,
        Ok(None) => return bad_request("命令不存在").into_response(),
        Err(e) => return bad_request(e).into_response(),
    };

    match std::fs::remove_file(&target) {
        Ok(_) => ok_message("Gemini command deleted successfully").into_response(),
        Err(e) => internal_error(format!("删除文件失败: {}", e)).into_response(),
    }
}

pub async fn toggle_gemini_slash_command(Path(_name): Path<String>) -> impl IntoResponse {
    (
        axum::http::StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "success": false,
            "data": null,
            "message": "Gemini CLI custom commands do not support enable/disable toggle via API"
        })),
    )
        .into_response()
}
