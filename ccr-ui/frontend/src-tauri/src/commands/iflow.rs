//! iFlow 命令 — Settings/MCP/Slash Commands（stub 端点）。
//!
//! iFlow 平台尚未正式支持，大部分操作返回空结果或桩实现。
//! Settings 使用 ~/.iflow/settings.json。

use serde_json::{Value, json};
use std::io::Write as IoWrite;
use std::path::PathBuf;

// ── Config file helpers ──

/// 定位 ~/.iflow/settings.json
fn iflow_config_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let dir = home.join(".iflow");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("创建 .iflow 目录失败: {e}"))?;
    }
    Ok(dir.join("settings.json"))
}

/// 读取 iFlow settings.json，不存在时返回空对象
fn read_iflow_config() -> Result<Value, String> {
    let path = iflow_config_path()?;
    if !path.exists() {
        return Ok(json!({}));
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("读取 iFlow 配置文件失败: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("解析 iFlow JSON 失败: {e}"))
}

/// 原子写入 iFlow settings.json
fn write_iflow_config(config: &Value) -> Result<(), String> {
    let path = iflow_config_path()?;
    let parent = path.parent().ok_or_else(|| "无法获取父目录".to_string())?;
    let json_str =
        serde_json::to_string_pretty(config).map_err(|e| format!("序列化 iFlow 配置失败: {e}"))?;
    let mut tmp =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("创建临时文件失败: {e}"))?;
    tmp.write_all(json_str.as_bytes())
        .map_err(|e| format!("写入临时文件失败: {e}"))?;
    tmp.persist(&path)
        .map_err(|e| format!("持久化配置文件失败: {e}"))?;
    Ok(())
}

// ── Settings ──

#[tauri::command]
pub async fn iflow_get_settings() -> Result<Value, String> {
    tokio::task::spawn_blocking(read_iflow_config)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn iflow_update_settings(settings: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        write_iflow_config(&settings)?;
        Ok(json!({ "message": "iFlow 配置更新成功" }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── MCP Servers (stub — iFlow MCP 功能待实现) ──

#[tauri::command]
pub async fn iflow_list_mcp_servers() -> Result<Value, String> {
    Ok(json!([]))
}

#[tauri::command]
pub async fn iflow_add_mcp_server(name: String, config: Value) -> Result<Value, String> {
    let _ = (name, config);
    Err("iFlow MCP 服务器添加功能待实现".to_string())
}

#[tauri::command]
pub async fn iflow_update_mcp_server(name: String, config: Value) -> Result<Value, String> {
    let _ = (name, config);
    Err("iFlow MCP 服务器更新功能待实现".to_string())
}

#[tauri::command]
pub async fn iflow_delete_mcp_server(name: String) -> Result<String, String> {
    let _ = name;
    Err("iFlow MCP 服务器删除功能待实现".to_string())
}

// ── Slash Commands (stub — iFlow Slash Commands 功能待实现) ──

#[tauri::command]
pub async fn iflow_list_slash_commands() -> Result<Value, String> {
    Ok(json!({ "commands": [], "folders": [] }))
}
