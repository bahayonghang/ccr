//! TOML 斜杠命令共享模块
//!
//! Gemini CLI 和 Qwen 都使用 TOML 格式的斜杠命令文件（位于 `.{platform}/commands/` 目录）。
//! 本模块提取两者完全相同的 TOML 命令处理逻辑，通过平台名称参数化，消除约 300 行重复代码。

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

use crate::api::handlers::response::{bad_request, internal_error, ok, ok_message};

// ============ 共享类型 ============

/// UI 前端发送的斜杠命令请求
#[derive(Debug, Deserialize)]
pub struct UiSlashCommandRequest {
    pub name: String,
    pub description: String,
    pub command: String,
    #[serde(default)]
    pub folder: String,
}

/// TOML 命令文件结构
#[derive(Debug, Serialize)]
struct TomlCommandFile {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ============ 工具函数 ============

/// 从当前目录向上查找项目根目录（包含 .git 的目录）
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

/// 获取平台斜杠命令的项目级和用户级目录
///
/// - `platform_dir`: 平台目录名 (如 "gemini", "qwen")
///
/// 返回: (项目级目录, 用户级目录)
/// - 项目级: `{project_root}/.{platform}/commands/`
/// - 用户级: `~/.{platform}/commands/`
fn platform_commands_dirs(platform_dir: &str) -> Result<(PathBuf, PathBuf), String> {
    let project_root = find_project_root()?;
    let project = project_root
        .join(format!(".{}", platform_dir))
        .join("commands");
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let user = home.join(format!(".{}", platform_dir)).join("commands");
    Ok((project, user))
}

/// 从目录中列出所有 TOML 命令文件
fn list_toml_commands_from_dirs(
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

/// 按名称查找命令文件
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

// ============ 通用 Handler 函数 ============

/// 列出平台的所有 TOML 斜杠命令
pub fn list_slash_commands(platform_dir: &str) -> Response {
    let (project_dir, user_dir) = match platform_commands_dirs(platform_dir) {
        Ok(d) => d,
        Err(e) => return internal_error(e).into_response(),
    };

    let commands = match list_toml_commands_from_dirs(&project_dir, &user_dir) {
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

/// 添加 TOML 斜杠命令
pub fn add_slash_command(
    platform_dir: &str,
    platform_name: &str,
    req: UiSlashCommandRequest,
) -> Response {
    let (project_dir, _user_dir) = match platform_commands_dirs(platform_dir) {
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
        Ok(_) => {
            ok_message(format!("{} command created successfully", platform_name)).into_response()
        }
        Err(e) => internal_error(format!("写入文件失败: {}", e)).into_response(),
    }
}

/// 更新 TOML 斜杠命令
pub fn update_slash_command(
    platform_dir: &str,
    platform_name: &str,
    name: &str,
    req: UiSlashCommandRequest,
) -> Response {
    let (project_dir, user_dir) = match platform_commands_dirs(platform_dir) {
        Ok(d) => d,
        Err(e) => return internal_error(e).into_response(),
    };

    let target = match find_command_file_by_name(&project_dir, &user_dir, name) {
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
        Ok(_) => {
            ok_message(format!("{} command updated successfully", platform_name)).into_response()
        }
        Err(e) => internal_error(format!("写入文件失败: {}", e)).into_response(),
    }
}

/// 删除 TOML 斜杠命令
pub fn delete_slash_command(platform_dir: &str, platform_name: &str, name: &str) -> Response {
    let (project_dir, user_dir) = match platform_commands_dirs(platform_dir) {
        Ok(d) => d,
        Err(e) => return internal_error(e).into_response(),
    };

    let target = match find_command_file_by_name(&project_dir, &user_dir, name) {
        Ok(Some(p)) => p,
        Ok(None) => return bad_request("命令不存在").into_response(),
        Err(e) => return bad_request(e).into_response(),
    };

    match std::fs::remove_file(&target) {
        Ok(_) => {
            ok_message(format!("{} command deleted successfully", platform_name)).into_response()
        }
        Err(e) => internal_error(format!("删除文件失败: {}", e)).into_response(),
    }
}

/// TOML 斜杠命令不支持启用/禁用切换
pub fn toggle_not_supported(platform_display: &str) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "success": false,
            "data": null,
            "message": format!("{} custom commands do not support enable/disable toggle via API", platform_display)
        })),
    )
        .into_response()
}
