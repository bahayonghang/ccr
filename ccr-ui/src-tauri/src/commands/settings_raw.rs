//! Raw local settings/config editing with content-version conflict protection.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use ccr_config::{Platform, PlatformPaths};
use ccr_core::core::{
    BackupPolicy, VersionedWriteOutcome, WriteOptions, content_version_token,
    write_guarded_versioned,
};
use serde_json::{Value, json};
use tauri::State;

use crate::platform::local::LocalEnvironment;
use crate::platform::{EnvironmentType, ExecutionEnvironment};
use crate::state::AppState;

use super::codex::{
    codex_config_path, invalidate_codex_dashboard_overview_cache, validate_codex_config_raw,
};

#[derive(Debug, Clone, Copy)]
enum RawConfigKind {
    Claude,
    Codex,
}

pub(crate) async fn ensure_local_env(state: &AppState) -> Option<Value> {
    let environment = {
        let registry = state.env_registry.read().await;
        registry
            .active()
            .unwrap_or_else(|| std::sync::Arc::new(LocalEnvironment::new()))
    };
    unsupported_environment(environment.as_ref())
}

fn unsupported_environment(environment: &dyn ExecutionEnvironment) -> Option<Value> {
    let environment_type = environment.env_type();
    if environment_type == EnvironmentType::Local {
        return None;
    }

    Some(json!({
        "status": "unsupported_environment",
        "envType": environment_type,
    }))
}

fn claude_settings_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    Ok(home.join(".claude").join("settings.json"))
}

fn backup_dir(platform: Platform) -> Result<PathBuf, String> {
    PlatformPaths::new(platform)
        .map(|paths| paths.backups_dir)
        .map_err(|error| format!("解析 {} 备份目录失败: {error}", platform.display_name()))
}

pub(crate) fn read_raw_file(path: &Path) -> Result<Value, String> {
    match fs::read(path) {
        Ok(bytes) => {
            let content = String::from_utf8(bytes.clone())
                .map_err(|error| format!("配置文件不是有效 UTF-8: {error}"))?;
            Ok(json!({
                "status": "ok",
                "content": content,
                "token": content_version_token(&bytes),
                "path": path,
                "exists": true,
            }))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(json!({
            "status": "ok",
            "content": "",
            "token": "",
            "path": path,
            "exists": false,
        })),
        Err(error) => Err(format!("读取配置文件 {} 失败: {error}", path.display())),
    }
}

pub(crate) fn invalid_result(
    kind: &'static str,
    message: &'static str,
    position: Option<(usize, usize)>,
) -> Value {
    let mut result = json!({
        "status": "invalid",
        "kind": kind,
        "message": message,
    });
    if let Some((line, column)) = position {
        result["line"] = json!(line);
        result["column"] = json!(column);
    }
    result
}

fn validate_raw_config(kind: RawConfigKind, content: &str) -> Option<Value> {
    match kind {
        RawConfigKind::Claude => {
            let value: Value = match serde_json::from_str(content) {
                Ok(value) => value,
                Err(error) => {
                    return Some(invalid_result(
                        "syntax",
                        "Invalid JSON syntax",
                        Some((error.line(), error.column())),
                    ));
                }
            };
            if !value.is_object() {
                return Some(invalid_result(
                    "semantic",
                    "Claude settings must be a JSON object",
                    None,
                ));
            }
            if let Err(error) = serde_json::from_str::<ccr_types::ClaudeSettings>(content) {
                return Some(invalid_result(
                    "semantic",
                    "Claude settings contain invalid value types",
                    Some((error.line(), error.column())),
                ));
            }
        }
        RawConfigKind::Codex => {
            if let Err(error) = toml::from_str::<toml::Value>(content) {
                return Some(invalid_result(
                    "syntax",
                    "Invalid TOML syntax",
                    toml_error_position(content, &error),
                ));
            }
            if let Err(error) = validate_codex_config_raw(content) {
                return Some(invalid_result(
                    "semantic",
                    "Codex configuration contains invalid value types",
                    toml_error_position(content, &error),
                ));
            }
        }
    }

    None
}

pub(crate) fn toml_error_position(
    content: &str,
    error: &toml::de::Error,
) -> Option<(usize, usize)> {
    let offset = error.span()?.start.min(content.len());
    let prefix = content.get(..offset)?;
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.chars().count() + 1, |(_, tail)| {
            tail.chars().count() + 1
        });
    Some((line, column))
}

fn save_raw_file(
    kind: RawConfigKind,
    path: &Path,
    backup_dir: &Path,
    content: &str,
    expected_token: &str,
) -> Result<Value, String> {
    if let Some(invalid) = validate_raw_config(kind, content) {
        return Ok(invalid);
    }

    let prefix = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config")
        .to_string();
    write_raw_file_versioned(path, backup_dir, &prefix, content, expected_token, true)
}

pub(crate) fn write_raw_file_versioned(
    path: &Path,
    backup_dir: &Path,
    backup_prefix: &str,
    content: &str,
    expected_token: &str,
    secret: bool,
) -> Result<Value, String> {
    let options = WriteOptions {
        backup: BackupPolicy::Dir {
            dir: backup_dir.to_path_buf(),
            prefix: backup_prefix.to_string(),
        },
        secret,
        ..Default::default()
    };

    match write_guarded_versioned(path, content.as_bytes(), expected_token, &options)
        .map_err(|error| format!("写入配置文件 {} 失败: {error}", path.display()))?
    {
        VersionedWriteOutcome::Written => Ok(json!({
            "status": "saved",
            "token": content_version_token(content.as_bytes()),
        })),
        VersionedWriteOutcome::Conflict => Ok(json!({ "status": "conflict" })),
    }
}

fn file_layer(id: &str, label: &str, path: Option<&Path>, editable: bool) -> Value {
    let metadata = path.and_then(|candidate| fs::metadata(candidate).ok());
    let modified = metadata
        .as_ref()
        .and_then(|value| value.modified().ok())
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis() as u64);

    json!({
        "id": id,
        "label": label,
        "path": path,
        "exists": path.map(Path::exists),
        "size": metadata.as_ref().map(std::fs::Metadata::len),
        "mtime": modified,
        "editable": editable,
    })
}

fn claude_settings_layers(path: &Path) -> Value {
    #[cfg(target_os = "windows")]
    let managed = std::env::var_os("PROGRAMDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
        .join("ClaudeCode")
        .join("managed-settings.json");
    #[cfg(target_os = "macos")]
    let managed = PathBuf::from("/Library/Application Support/ClaudeCode/managed-settings.json");
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let managed = PathBuf::from("/etc/claude-code/managed-settings.json");

    json!({ "layers": [
        file_layer("managed", "Managed", Some(&managed), false),
        file_layer("project", "Project", None, false),
        file_layer("local", "Project local", None, false),
        file_layer("user", "User", Some(path), true),
    ] })
}

fn codex_config_layers(path: &Path) -> Result<Value, String> {
    let mut layers = vec![file_layer("user", "User", Some(path), true)];
    if let Some(directory) = path.parent()
        && directory.exists()
    {
        let mut overlays: Vec<PathBuf> = fs::read_dir(directory)
            .map_err(|error| format!("读取 Codex 配置目录失败: {error}"))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|candidate| {
                candidate
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".config.toml") && name != "config.toml")
            })
            .collect();
        overlays.sort();
        layers.extend(overlays.iter().map(|overlay| {
            let label = overlay
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("Profile overlay");
            file_layer("profile_overlay", label, Some(overlay), false)
        }));
    }
    Ok(json!({ "layers": layers }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_get_settings_raw_text(state: State<'_, AppState>) -> Result<Value, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return Ok(response);
    }
    let path = claude_settings_path()?;
    tokio::task::spawn_blocking(move || read_raw_file(&path))
        .await
        .map_err(|error| format!("读取 Claude settings 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn claude_save_settings_raw_text(
    state: State<'_, AppState>,
    content: String,
    token: String,
) -> Result<Value, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return Ok(response);
    }
    let path = claude_settings_path()?;
    let backup_dir = backup_dir(Platform::Claude)?;
    tokio::task::spawn_blocking(move || {
        save_raw_file(RawConfigKind::Claude, &path, &backup_dir, &content, &token)
    })
    .await
    .map_err(|error| format!("写入 Claude settings 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn codex_get_config_raw_text(state: State<'_, AppState>) -> Result<Value, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return Ok(response);
    }
    let path = codex_config_path()?;
    tokio::task::spawn_blocking(move || read_raw_file(&path))
        .await
        .map_err(|error| format!("读取 Codex config 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn codex_save_config_raw_text(
    state: State<'_, AppState>,
    content: String,
    token: String,
) -> Result<Value, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return Ok(response);
    }
    let path = codex_config_path()?;
    let backup_dir = backup_dir(Platform::Codex)?;
    let response = tokio::task::spawn_blocking(move || {
        save_raw_file(RawConfigKind::Codex, &path, &backup_dir, &content, &token)
    })
    .await
    .map_err(|error| format!("写入 Codex config 后台任务失败: {error}"))??;

    if response["status"] == "saved" {
        invalidate_codex_dashboard_overview_cache(state.inner()).await;
    }
    Ok(response)
}

#[ccr_tauri_command_macros::command]
pub async fn claude_list_settings_layers(state: State<'_, AppState>) -> Result<Value, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return Ok(response);
    }
    let path = claude_settings_path()?;
    tokio::task::spawn_blocking(move || Ok(claude_settings_layers(&path)))
        .await
        .map_err(|error| format!("探测 Claude settings 层级后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn codex_list_config_layers(state: State<'_, AppState>) -> Result<Value, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return Ok(response);
    }
    let path = codex_config_path()?;
    tokio::task::spawn_blocking(move || codex_config_layers(&path))
        .await
        .map_err(|error| format!("探测 Codex config 层级后台任务失败: {error}"))?
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_core::core::content_version_token;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn get_raw_file_reports_missing_target() {
        let temp_dir = tempdir().unwrap();
        let result = read_raw_file(&temp_dir.path().join("missing.json")).unwrap();

        assert_eq!(result["status"], "ok");
        assert_eq!(result["content"], "");
        assert_eq!(result["token"], "");
        assert_eq!(result["exists"], false);
    }

    #[test]
    fn claude_invalid_json_is_rejected_without_leaking_content() {
        let temp_dir = tempdir().unwrap();
        let target = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        fs::write(&target, b"{}").unwrap();
        let token = content_version_token(b"{}");
        let probe = "DO_NOT_LEAK_PROBE";

        let result = save_raw_file(
            RawConfigKind::Claude,
            &target,
            &backup_dir,
            &format!(r#"{{"permissions": {probe}}}"#),
            &token,
        )
        .unwrap();

        assert_eq!(result["status"], "invalid");
        assert_eq!(result["kind"], "syntax");
        assert!(result["line"].as_u64().is_some());
        assert!(!result["message"].as_str().unwrap().contains(probe));
        assert_eq!(fs::read(&target).unwrap(), b"{}");
        assert!(!backup_dir.exists());
    }

    #[test]
    fn claude_semantic_error_includes_position_without_leaking_content() {
        let temp_dir = tempdir().unwrap();
        let target = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        fs::write(&target, b"{}").unwrap();
        let token = content_version_token(b"{}");
        let probe = "DO_NOT_LEAK_PROBE";

        let result = save_raw_file(
            RawConfigKind::Claude,
            &target,
            &backup_dir,
            &format!(r#"{{"env": {{"{probe}": 42}}}}"#),
            &token,
        )
        .unwrap();

        assert_eq!(result["status"], "invalid");
        assert_eq!(result["kind"], "semantic");
        assert!(result["line"].as_u64().is_some());
        assert!(result["column"].as_u64().is_some());
        assert!(!result["message"].as_str().unwrap().contains(probe));
        assert_eq!(fs::read(&target).unwrap(), b"{}");
        assert!(!backup_dir.exists());
    }

    #[test]
    fn codex_semantic_error_is_rejected_without_leaking_content() {
        let temp_dir = tempdir().unwrap();
        let target = temp_dir.path().join("config.toml");
        let backup_dir = temp_dir.path().join("backups");
        fs::write(&target, b"model = 'gpt-5'").unwrap();
        let token = content_version_token(b"model = 'gpt-5'");
        let probe = "DO_NOT_LEAK_PROBE";

        let result = save_raw_file(
            RawConfigKind::Codex,
            &target,
            &backup_dir,
            &format!("model_context_window = '{probe}'"),
            &token,
        )
        .unwrap();

        assert_eq!(result["status"], "invalid");
        assert_eq!(result["kind"], "semantic");
        assert!(!result["message"].as_str().unwrap().contains(probe));
        assert_eq!(fs::read(&target).unwrap(), b"model = 'gpt-5'");
        assert!(!backup_dir.exists());
    }

    #[test]
    fn raw_save_rejects_stale_token_and_preserves_external_change() {
        let temp_dir = tempdir().unwrap();
        let target = temp_dir.path().join("config.toml");
        let backup_dir = temp_dir.path().join("backups");
        fs::write(&target, b"model = 'old'").unwrap();
        let stale_token = content_version_token(b"model = 'old'");
        fs::write(&target, b"model = 'external'").unwrap();

        let result = save_raw_file(
            RawConfigKind::Codex,
            &target,
            &backup_dir,
            "model = 'editor'",
            &stale_token,
        )
        .unwrap();

        assert_eq!(result["status"], "conflict");
        assert_eq!(fs::read(&target).unwrap(), b"model = 'external'");
        assert!(!backup_dir.exists());
    }

    #[test]
    fn raw_save_creates_missing_file_and_backs_up_existing_file() {
        let temp_dir = tempdir().unwrap();
        let target = temp_dir.path().join("config.toml");
        let backup_dir = temp_dir.path().join("backups");

        let created = save_raw_file(
            RawConfigKind::Codex,
            &target,
            &backup_dir,
            "model = 'first'",
            "",
        )
        .unwrap();
        let first_token = created["token"].as_str().unwrap().to_string();
        let saved = save_raw_file(
            RawConfigKind::Codex,
            &target,
            &backup_dir,
            "model = 'second'",
            &first_token,
        )
        .unwrap();

        assert_eq!(created["status"], "saved");
        assert_eq!(saved["status"], "saved");
        assert_eq!(fs::read_to_string(&target).unwrap(), "model = 'second'");
        assert_eq!(fs::read_dir(&backup_dir).unwrap().count(), 1);
    }
}
