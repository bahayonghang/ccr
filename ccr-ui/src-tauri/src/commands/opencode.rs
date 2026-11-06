//! OpenCode 命令：Settings / TUI / Agents / Commands / Plugins。
//!
//! 以官方 OpenCode 文档为准：
//! - `~/.config/opencode/opencode.json` 作为主配置
//! - `~/.config/opencode/tui.json` 作为 TUI 配置
//! - `~/.config/opencode/{agents,commands,plugins}` 作为全局目录
//! - `.opencode/{agents,commands,plugins}` 作为项目目录
//!
//! 兼容读取旧路径 `~/.opencode/*`，但所有写入都会落到新路径。

use ccr_app::resources::platform_files::{
    Kind as ResourceKind, Store as ResourceStore, opencode as resource_codec,
};
pub use resource_codec::{OpenCodeAgentRecord, OpenCodeCommandRecord};
#[cfg(test)]
use resource_codec::{parse_markdown_document, render_markdown_document, yaml_bool, yaml_string};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use ts_rs::TS;

use super::wire::OpenJsonValueDto;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/opencode/")]
pub struct OpenCodeThemeRecord {
    pub id: String,
    pub name: String,
    pub theme_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/opencode/")]
pub struct OpenCodePluginFileRecord {
    pub name: String,
    pub path: String,
    pub scope: String,
    #[ts(as = "f64")]
    pub size: u64,
}

fn open_json(value: Value) -> Result<OpenJsonValueDto, String> {
    value.try_into()
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct OpenCodeScopedRequest {
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    project_root: Option<String>,
}

#[derive(Debug, Clone)]
struct ResolvedScopeDir {
    scope: String,
    dir: PathBuf,
}

fn opencode_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())
}

fn opencode_config_dir_from_home(home: &Path) -> PathBuf {
    home.join(".config").join("opencode")
}

fn opencode_legacy_dir_from_home(home: &Path) -> PathBuf {
    home.join(".opencode")
}

pub(crate) fn opencode_config_dir() -> Result<PathBuf, String> {
    Ok(opencode_config_dir_from_home(&opencode_home_dir()?))
}

fn opencode_legacy_dir() -> Result<PathBuf, String> {
    Ok(opencode_legacy_dir_from_home(&opencode_home_dir()?))
}

fn opencode_config_path() -> Result<PathBuf, String> {
    Ok(opencode_config_dir()?.join("opencode.json"))
}

fn opencode_legacy_config_path() -> Result<PathBuf, String> {
    Ok(opencode_legacy_dir()?.join("config.json"))
}

fn opencode_tui_path() -> Result<PathBuf, String> {
    Ok(opencode_config_dir()?.join("tui.json"))
}

fn opencode_legacy_keybindings_path() -> Result<PathBuf, String> {
    Ok(opencode_legacy_dir()?.join("keybindings.json"))
}

fn detect_project_root_from(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join("opencode.json").exists()
            || current.join(".opencode").exists()
            || current.join(".git").exists()
        {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn current_project_root() -> Option<PathBuf> {
    std::env::current_dir()
        .ok()
        .and_then(|path| detect_project_root_from(&path))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录 '{}' 失败: {e}", parent.to_string_lossy()))?;
    }
    Ok(())
}

fn resolve_scope_dir(
    request: &OpenCodeScopedRequest,
    folder: &str,
) -> Result<ResolvedScopeDir, String> {
    match request.scope.as_deref().unwrap_or("global") {
        "global" => Ok(ResolvedScopeDir {
            scope: "global".to_string(),
            dir: opencode_config_dir()?.join(folder),
        }),
        "project" => {
            let project_root = request
                .project_root
                .as_ref()
                .map(PathBuf::from)
                .or_else(current_project_root)
                .ok_or_else(|| "未检测到项目目录，无法使用 project scope".to_string())?;
            Ok(ResolvedScopeDir {
                scope: "project".to_string(),
                dir: project_root.join(".opencode").join(folder),
            })
        }
        other => Err(format!("不支持的 OpenCode scope: {other}")),
    }
}

fn read_json_file(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(json!({}));
    }
    let content = fs::read_to_string(path)
        .map_err(|e| format!("读取文件 '{}' 失败: {e}", path.to_string_lossy()))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("解析 JSON '{}' 失败: {e}", path.to_string_lossy()))
}

fn read_json_file_with_fallback(primary: &Path, fallback: &Path) -> Result<Value, String> {
    if primary.exists() {
        return read_json_file(primary);
    }
    if fallback.exists() {
        return read_json_file(fallback);
    }
    Ok(json!({}))
}

fn write_json_file(path: &Path, value: &Value) -> Result<(), String> {
    ensure_parent_dir(path)?;
    let content =
        serde_json::to_string_pretty(value).map_err(|e| format!("序列化 JSON 失败: {e}"))?;
    fs::write(path, content).map_err(|e| format!("写入文件 '{}' 失败: {e}", path.to_string_lossy()))
}

fn merge_json_objects(current: Value, incoming: Value) -> Value {
    match (current, incoming) {
        (Value::Object(mut current_map), Value::Object(incoming_map)) => {
            for (key, value) in incoming_map {
                current_map.insert(key, value);
            }
            Value::Object(current_map)
        }
        (_, replacement) => replacement,
    }
}

fn list_scope_candidates(folder: &str) -> Result<Vec<ResolvedScopeDir>, String> {
    let mut scopes = vec![ResolvedScopeDir {
        scope: "global".to_string(),
        dir: opencode_config_dir()?.join(folder),
    }];

    if let Some(project_root) = current_project_root() {
        scopes.push(ResolvedScopeDir {
            scope: "project".to_string(),
            dir: project_root.join(".opencode").join(folder),
        });
    }

    Ok(scopes)
}

fn resource_store(scope: &ResolvedScopeDir, kind: ResourceKind) -> Result<ResourceStore, String> {
    ResourceStore::with_default_storage(scope.dir.clone(), kind).map_err(|error| error.to_string())
}
fn list_agents_internal() -> Result<Vec<OpenCodeAgentRecord>, String> {
    let mut records = Vec::new();
    for scope in list_scope_candidates("agents")? {
        records.extend(resource_codec::list_agents(
            &resource_store(&scope, ResourceKind::OpenCodeAgent)?,
            &scope.scope,
        )?);
    }
    Ok(records)
}
fn list_commands_internal() -> Result<Vec<OpenCodeCommandRecord>, String> {
    let mut records = Vec::new();
    for scope in list_scope_candidates("commands")? {
        records.extend(resource_codec::list_commands(
            &resource_store(&scope, ResourceKind::OpenCodeCommand)?,
            &scope.scope,
        )?);
    }
    Ok(records)
}
fn upsert_agent_internal(
    name: &str,
    config: &Map<String, Value>,
    scope: &ResolvedScopeDir,
) -> Result<OpenCodeAgentRecord, String> {
    resource_codec::upsert_agent(
        &resource_store(scope, ResourceKind::OpenCodeAgent)?,
        name,
        config,
        &scope.scope,
    )
}
fn upsert_command_internal(
    name: &str,
    config: &Map<String, Value>,
    scope: &ResolvedScopeDir,
) -> Result<OpenCodeCommandRecord, String> {
    resource_codec::upsert_command(
        &resource_store(scope, ResourceKind::OpenCodeCommand)?,
        name,
        config,
        &scope.scope,
    )
}
fn delete_markdown_doc(
    name: &str,
    request: &OpenCodeScopedRequest,
    folder: &str,
) -> Result<String, String> {
    let kind = match folder {
        "agents" => ResourceKind::OpenCodeAgent,
        "commands" => ResourceKind::OpenCodeCommand,
        _ => return Err("Unsupported resource kind".into()),
    };
    let scope = resolve_scope_dir(request, folder)?;
    resource_codec::delete(&resource_store(&scope, kind)?, name)?;
    Ok(name.to_string())
}

fn scan_plugin_files(dir: &Path, scope: &str) -> Result<Vec<OpenCodePluginFileRecord>, String> {
    resource_codec::list_plugins(dir)
        .map(|files| {
            files
                .into_iter()
                .map(|file| OpenCodePluginFileRecord {
                    name: file.name,
                    path: file.path.to_string_lossy().into_owned(),
                    scope: scope.into(),
                    size: file.size,
                })
                .collect()
        })
        .map_err(|error| error.to_string())
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_get_settings() -> Result<OpenJsonValueDto, String> {
    let primary = opencode_config_path()?;
    let legacy = opencode_legacy_config_path()?;
    open_json(read_json_file_with_fallback(&primary, &legacy)?)
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_update_settings(
    settings: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let primary = opencode_config_path()?;
    let legacy = opencode_legacy_config_path()?;
    let current = read_json_file_with_fallback(&primary, &legacy)?;
    let merged = merge_json_objects(current, settings.into());
    write_json_file(&primary, &merged)?;
    open_json(merged)
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_get_tui_settings() -> Result<OpenJsonValueDto, String> {
    let primary = opencode_tui_path()?;
    let legacy = opencode_legacy_keybindings_path()?;
    if primary.exists() {
        return open_json(read_json_file(&primary)?);
    }
    if legacy.exists() {
        let keybinds = read_json_file(&legacy)?;
        return open_json(json!({ "keybinds": keybinds }));
    }
    open_json(json!({}))
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_update_tui_settings(
    settings: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let primary = opencode_tui_path()?;
    let current = read_json_file(&primary)?;
    let merged = merge_json_objects(current, settings.into());
    write_json_file(&primary, &merged)?;
    open_json(merged)
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_get_keybindings() -> Result<OpenJsonValueDto, String> {
    let tui: Value = opencode_get_tui_settings().await?.into();
    open_json(
        tui.as_object()
            .and_then(|map| map.get("keybinds"))
            .cloned()
            .unwrap_or_else(|| json!({})),
    )
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_update_keybindings(
    keybindings: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let request = open_json(json!({ "keybinds": Value::from(keybindings) }))?;
    let updated: Value = opencode_update_tui_settings(request).await?.into();
    open_json(
        updated
            .as_object()
            .and_then(|map| map.get("keybinds"))
            .cloned()
            .unwrap_or_else(|| json!({})),
    )
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_list_themes() -> Result<Vec<OpenCodeThemeRecord>, String> {
    Ok(resource_codec::THEMES
        .iter()
        .map(|(id, name, theme_type)| OpenCodeThemeRecord {
            id: (*id).into(),
            name: (*name).into(),
            theme_type: (*theme_type).into(),
        })
        .collect())
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_list_agents() -> Result<OpenJsonValueDto, String> {
    open_json(
        serde_json::to_value(list_agents_internal()?)
            .map_err(|e| format!("序列化 OpenCode agents 失败: {e}"))?,
    )
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_add_agent(config: OpenJsonValueDto) -> Result<OpenJsonValueDto, String> {
    let config: Value = config.into();
    let object = config
        .as_object()
        .ok_or_else(|| "OpenCode agent 配置必须是 JSON object".to_string())?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "OpenCode agent name 为必填项".to_string())?;
    let scoped: OpenCodeScopedRequest =
        serde_json::from_value(config.clone()).map_err(|e| format!("解析 scope 失败: {e}"))?;
    let scope = resolve_scope_dir(&scoped, "agents")?;
    open_json(
        serde_json::to_value(upsert_agent_internal(name, object, &scope)?)
            .map_err(|e| format!("序列化 OpenCode agent 失败: {e}"))?,
    )
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_update_agent(config: OpenJsonValueDto) -> Result<OpenJsonValueDto, String> {
    opencode_add_agent(config).await
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_delete_agent(
    name: String,
    context: Option<OpenJsonValueDto>,
) -> Result<String, String> {
    let scoped = context
        .map(Value::from)
        .map(serde_json::from_value::<OpenCodeScopedRequest>)
        .transpose()
        .map_err(|e| format!("解析 OpenCode agent scope 失败: {e}"))?
        .unwrap_or_default();
    delete_markdown_doc(&name, &scoped, "agents")
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_list_commands() -> Result<OpenJsonValueDto, String> {
    open_json(
        serde_json::to_value(list_commands_internal()?)
            .map_err(|e| format!("序列化 OpenCode commands 失败: {e}"))?,
    )
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_add_command(config: OpenJsonValueDto) -> Result<OpenJsonValueDto, String> {
    let config: Value = config.into();
    let object = config
        .as_object()
        .ok_or_else(|| "OpenCode command 配置必须是 JSON object".to_string())?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "OpenCode command name 为必填项".to_string())?;
    let scoped: OpenCodeScopedRequest =
        serde_json::from_value(config.clone()).map_err(|e| format!("解析 scope 失败: {e}"))?;
    let scope = resolve_scope_dir(&scoped, "commands")?;
    open_json(
        serde_json::to_value(upsert_command_internal(name, object, &scope)?)
            .map_err(|e| format!("序列化 OpenCode command 失败: {e}"))?,
    )
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_update_command(config: OpenJsonValueDto) -> Result<OpenJsonValueDto, String> {
    opencode_add_command(config).await
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_delete_command(
    name: String,
    context: Option<OpenJsonValueDto>,
) -> Result<String, String> {
    let scoped = context
        .map(Value::from)
        .map(serde_json::from_value::<OpenCodeScopedRequest>)
        .transpose()
        .map_err(|e| format!("解析 OpenCode command scope 失败: {e}"))?
        .unwrap_or_default();
    delete_markdown_doc(&name, &scoped, "commands")
}

#[ccr_tauri_command_macros::command]
pub async fn opencode_list_local_plugins() -> Result<Vec<OpenCodePluginFileRecord>, String> {
    let mut records = Vec::new();
    records.extend(scan_plugin_files(
        &opencode_config_dir()?.join("plugins"),
        "global",
    )?);
    if let Some(project_root) = current_project_root() {
        records.extend(scan_plugin_files(
            &project_root.join(".opencode").join("plugins"),
            "project",
        )?);
    }
    Ok(records)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn config_dir_uses_xdg_layout() {
        let dir = opencode_config_dir_from_home(Path::new("/tmp/test-home"));
        assert_eq!(dir, PathBuf::from("/tmp/test-home/.config/opencode"));
    }

    #[test]
    fn merge_json_objects_preserves_unrelated_keys() {
        let current = json!({
            "model": "anthropic/claude-sonnet-4-5",
            "permission": {
                "bash": "ask"
            }
        });
        let merged = merge_json_objects(
            current,
            json!({
                "server": {
                    "port": 4096
                }
            }),
        );

        assert_eq!(merged["model"], json!("anthropic/claude-sonnet-4-5"));
        assert_eq!(merged["permission"]["bash"], json!("ask"));
        assert_eq!(merged["server"]["port"], json!(4096));
    }

    #[test]
    fn parse_markdown_frontmatter_round_trip() {
        let raw = "---\ndescription: Review code\nmode: subagent\nhidden: true\n---\n\nYou are a reviewer.\n";
        let parsed = parse_markdown_document(raw).unwrap();
        assert_eq!(
            yaml_string(&parsed.frontmatter, "description").as_deref(),
            Some("Review code")
        );
        assert_eq!(yaml_bool(&parsed.frontmatter, "hidden"), Some(true));
        assert_eq!(parsed.body, "You are a reviewer.");

        let rendered = render_markdown_document(&parsed).unwrap();
        assert!(rendered.contains("description: Review code"));
        assert!(rendered.contains("You are a reviewer."));
    }

    #[test]
    fn agent_upsert_preserves_unknown_frontmatter_and_filename() {
        let temp = tempdir().unwrap();
        let agents_dir = temp.path().join("agents");
        fs::create_dir_all(&agents_dir).unwrap();
        let path = agents_dir.join("review.md");
        fs::write(
            &path,
            "---\ndescription: Review code\nmode: subagent\ncustom_flag: true\n---\n\nOld body\n",
        )
        .unwrap();

        let store = ResourceStore::new(
            agents_dir,
            temp.path().join("backups"),
            temp.path().join("locks"),
            ResourceKind::OpenCodeAgent,
        );

        let config = json!({
            "description": "Review code carefully",
            "body": "New body",
            "temperature": 0.1
        });

        let record =
            resource_codec::upsert_agent(&store, "review", config.as_object().unwrap(), "global")
                .unwrap();
        assert_eq!(record.name, "review");
        assert_eq!(record.body, "New body");
        assert_eq!(record.temperature, Some(0.1));
        assert_eq!(record.other.unwrap()["custom_flag"], json!(true));
        assert!(record.path.ends_with("review.md"));
    }

    #[test]
    fn plugin_scan_only_includes_script_files() {
        let temp = tempdir().unwrap();
        let plugins_dir = temp.path().join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();
        fs::write(plugins_dir.join("notify.ts"), "export default {}").unwrap();
        fs::write(plugins_dir.join("README.md"), "ignored").unwrap();

        let plugins = scan_plugin_files(&plugins_dir, "global").unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "notify.ts");
    }
}
