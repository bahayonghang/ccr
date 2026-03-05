//! OpenCode 命令 — Settings/Keybindings/Themes。
//!
//! 配置文件路径:
//!   ~/.opencode/config.json  — 主设置
//!   ~/.opencode/keybindings.json — 快捷键
//!
//! 主题为内置静态列表（OpenCode 不通过用户文件管理主题）。

use serde_json::Value;
use std::path::PathBuf;

// ── 内部工具函数 ──

/// 返回 ~/.opencode/ 目录
fn opencode_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    Ok(home.join(".opencode"))
}

/// 读取 JSON 文件，不存在时返回空 Object
async fn read_json_file(path: PathBuf) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        if !path.exists() {
            return Ok(serde_json::json!({}));
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取文件失败 {}: {e}", path.display()))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("解析 JSON 失败 {}: {e}", path.display()))
    })
    .await
    .map_err(|e| format!("spawn_blocking 失败: {e}"))?
}

/// 原子写入 JSON 文件
async fn write_json_file(path: PathBuf, value: Value) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败 {}: {e}", parent.display()))?;
        }
        let tmp_path = path.with_extension("json.tmp");
        let content =
            serde_json::to_string_pretty(&value).map_err(|e| format!("序列化 JSON 失败: {e}"))?;
        std::fs::write(&tmp_path, &content).map_err(|e| format!("写入临时文件失败: {e}"))?;
        std::fs::rename(&tmp_path, &path).map_err(|e| format!("原子重命名失败: {e}"))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("spawn_blocking 失败: {e}"))?
}

// ── Settings ──

#[tauri::command]
pub async fn opencode_get_settings() -> Result<Value, String> {
    let path = opencode_dir()?.join("config.json");
    read_json_file(path).await
}

#[tauri::command]
pub async fn opencode_update_settings(settings: Value) -> Result<Value, String> {
    let path = opencode_dir()?.join("config.json");
    // 读取现有设置，浅合并传入字段
    let mut current = read_json_file(path.clone()).await?;
    if let (Some(cur_obj), Some(new_obj)) = (current.as_object_mut(), settings.as_object()) {
        for (k, v) in new_obj {
            cur_obj.insert(k.clone(), v.clone());
        }
    } else {
        current = settings;
    }
    write_json_file(path, current.clone()).await?;
    Ok(current)
}

// ── Keybindings ──

#[tauri::command]
pub async fn opencode_get_keybindings() -> Result<Value, String> {
    let path = opencode_dir()?.join("keybindings.json");
    read_json_file(path).await
}

#[tauri::command]
pub async fn opencode_update_keybindings(keybindings: Value) -> Result<Value, String> {
    let path = opencode_dir()?.join("keybindings.json");
    write_json_file(path, keybindings.clone()).await?;
    Ok(keybindings)
}

// ── Themes ──
// OpenCode 内置主题列表（不通过用户文件管理）

#[tauri::command]
pub async fn opencode_list_themes() -> Result<Value, String> {
    let themes = serde_json::json!([
        { "id": "dark",            "name": "Dark",             "type": "dark" },
        { "id": "light",           "name": "Light",            "type": "light" },
        { "id": "catppuccin-mocha","name": "Catppuccin Mocha", "type": "dark" },
        { "id": "catppuccin-latte","name": "Catppuccin Latte", "type": "light" },
        { "id": "dracula",         "name": "Dracula",          "type": "dark" },
        { "id": "nord",            "name": "Nord",             "type": "dark" },
        { "id": "one-dark",        "name": "One Dark",         "type": "dark" },
        { "id": "github-dark",     "name": "GitHub Dark",      "type": "dark" },
        { "id": "github-light",    "name": "GitHub Light",     "type": "light" },
        { "id": "solarized-dark",  "name": "Solarized Dark",   "type": "dark" },
        { "id": "solarized-light", "name": "Solarized Light",  "type": "light" },
        { "id": "tokyo-night",     "name": "Tokyo Night",      "type": "dark" }
    ]);
    Ok(themes)
}
