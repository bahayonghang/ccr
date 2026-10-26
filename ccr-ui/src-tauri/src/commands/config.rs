//! 配置管理命令 — list / switch / add / delete / rename / duplicate / validate / import / export / history。

use serde::{Deserialize, Serialize};
use tauri::State;

#[cfg(test)]
use ccr_config::CcsConfig;
use ccr_config::{ConfigManager, ConfigService, ImportMode};
use ccr_core::CcrError;
use ccr_store::HistoryService;
use std::path::{Path, PathBuf};
use ts_rs::TS;

use crate::state::AppState;

mod adapter;
use adapter::{ConfigMutationResult, ConfigPatchInput, required_platform};

#[cfg(test)]
mod contract_tests;

/// 配置项详情
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/config/")]
pub struct ConfigInfo {
    pub version: String,
    pub name: String,
    pub description: String,
    pub base_url: String,
    pub auth_token: String, // masked
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub small_fast_model: Option<String>,
    pub is_current: bool,
    pub is_default: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub provider_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tags: Option<Vec<String>>,
    #[ts(as = "f64")]
    pub usage_count: u64,
    pub enabled: bool,
}

/// 历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/config/")]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: String,
    pub operation: String,
    pub actor: String,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/config/")]
pub struct ImportResult {
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
}

/// 导出结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/config/")]
pub struct ExportResult {
    pub content: String,
    pub filename: String,
}

fn destructive_confirmation_token(action: &str) -> String {
    format!("desktop-confirm:{action}")
}

fn validate_destructive_confirmation(
    action: &str,
    confirmation_token: Option<&str>,
) -> Result<(), String> {
    let expected = destructive_confirmation_token(action);
    match confirmation_token {
        Some(token) if token == expected => Ok(()),
        _ => Err(format!("配置操作 '{action}' 需要桌面确认后才能执行。")),
    }
}

fn resolve_managed_restore_path(
    config_path: &Path,
    backup_path: &str,
) -> Result<PathBuf, CcrError> {
    ccr_app::config::backups::resolve_managed_restore_path(config_path, backup_path)
}

fn restore_config_from_backup_path(backup_path: &Path) -> Result<(), CcrError> {
    let service = ConfigService::with_default()?;
    ccr_app::config::backups::restore(&service, backup_path)
}

// ── 配置管理 ──

#[ccr_tauri_command_macros::command]
pub async fn list_configs(platform: Option<String>) -> Result<Vec<ConfigInfo>, String> {
    let platform = required_platform(platform.as_deref()).map_err(|e| e.to_string())?;
    let result = tokio::task::spawn_blocking(move || {
        debug_assert_eq!(platform, ccr_config::Platform::Claude);
        let service = ccr_app::config::ConfigReadService::for_claude()
            .map_err(|e| format!("Failed to create ConfigManager: {e}"))?;
        let configs = adapter::list(&service).map_err(|e| e.to_string())?;

        Ok::<_, String>(configs)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[ccr_tauri_command_macros::command]
pub async fn switch_config(
    name: String,
    platform: Option<String>,
    enable: Option<bool>,
) -> Result<ConfigMutationResult, String> {
    let platform = required_platform(platform.as_deref()).map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || adapter::activate(platform, name, enable.unwrap_or(false)))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
        .map_err(|e| e.to_string())
}

#[ccr_tauri_command_macros::command]
pub async fn add_config(
    name: String,
    platform: Option<String>,
    data: Option<ConfigPatchInput>,
) -> Result<ConfigMutationResult, String> {
    let platform = required_platform(platform.as_deref()).map_err(|e| e.to_string())?;
    let data = data.ok_or_else(|| "config_payload_required".to_string())?;
    tokio::task::spawn_blocking(move || adapter::add(platform, &name, data))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
        .map_err(|e| e.to_string())
}

#[ccr_tauri_command_macros::command]
pub async fn delete_config(
    name: String,
    platform: Option<String>,
    confirmation_token: Option<String>,
) -> Result<ConfigMutationResult, String> {
    validate_destructive_confirmation("delete_config", confirmation_token.as_deref())?;
    let platform = required_platform(platform.as_deref()).map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || adapter::delete(platform, &name))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
        .map_err(|e| e.to_string())
}

#[ccr_tauri_command_macros::command]
pub async fn rename_config(
    old_name: String,
    new_name: String,
    platform: Option<String>,
) -> Result<ConfigMutationResult, String> {
    let platform = required_platform(platform.as_deref()).map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || adapter::rename(platform, &old_name, &new_name))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
        .map_err(|e| e.to_string())
}

#[ccr_tauri_command_macros::command]
pub async fn duplicate_config(
    source: String,
    target: String,
    platform: Option<String>,
) -> Result<ConfigMutationResult, String> {
    let platform = required_platform(platform.as_deref()).map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || adapter::duplicate(platform, &source, &target))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
        .map_err(|e| e.to_string())
}

#[ccr_tauri_command_macros::command]
pub async fn validate_configs() -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let service = ConfigService::with_default()
            .map_err(|e| format!("Failed to create ConfigService: {e}"))?;
        service
            .validate_all()
            .map_err(|e| format!("Validation failed: {e}"))?;
        Ok::<_, String>("All configurations are valid".to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn import_config(
    content: String,
    mode: String,
    backup: bool,
    confirmation_token: Option<String>,
) -> Result<ImportResult, String> {
    validate_destructive_confirmation("import_config", confirmation_token.as_deref())?;

    tokio::task::spawn_blocking(move || {
        let import_mode = match mode.to_lowercase().as_str() {
            "merge" => ImportMode::Merge,
            "replace" => ImportMode::Replace,
            _ => return Err(format!("Invalid import mode: {mode}")),
        };

        let service = ConfigService::with_default()
            .map_err(|e| format!("Failed to create ConfigService: {e}"))?;

        let result = service
            .import_config(&content, import_mode, backup)
            .map_err(|e| format!("Import failed: {e}"))?;

        Ok(ImportResult {
            added: result.added,
            updated: result.updated,
            skipped: result.skipped,
        })
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn restore_config(
    backup_path: String,
    confirmation_token: Option<String>,
) -> Result<String, String> {
    validate_destructive_confirmation("restore_config", confirmation_token.as_deref())?;

    tokio::task::spawn_blocking(move || {
        let manager = ConfigManager::with_default()
            .map_err(|e| format!("Failed to create ConfigManager: {e}"))?;
        let path = resolve_managed_restore_path(manager.config_path(), &backup_path)
            .map_err(|e| format!("Invalid restore backup: {e}"))?;
        restore_config_from_backup_path(&path)
            .map_err(|e| format!("Failed to restore config: {e}"))?;
        Ok(format!("Configuration restored from {}", path.display()))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn export_config(include_secrets: bool) -> Result<ExportResult, String> {
    tokio::task::spawn_blocking(move || {
        let service = ConfigService::with_default()
            .map_err(|e| format!("Failed to create ConfigService: {e}"))?;

        let content = service
            .export_config(include_secrets)
            .map_err(|e| format!("Failed to export config: {e}"))?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("ccr_config_export_{timestamp}.toml");

        Ok(ExportResult { content, filename })
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

// ── 历史记录 ──

#[ccr_tauri_command_macros::command]
pub async fn get_history(limit: Option<usize>) -> Result<Vec<HistoryEntry>, String> {
    let limit = limit.unwrap_or(100);

    let service = HistoryService::with_default()
        .map_err(|e| format!("Failed to create HistoryService: {e}"))?;

    let entries = service
        .get_recent_async(limit)
        .await
        .map_err(|e| format!("Failed to get history: {e}"))?;

    let json_entries: Vec<HistoryEntry> = entries
        .into_iter()
        .map(|e| HistoryEntry {
            id: e.id.to_string(),
            timestamp: e.timestamp.to_rfc3339(),
            operation: format!("{:?}", e.operation),
            actor: e.actor,
        })
        .collect();

    Ok(json_entries)
}

#[ccr_tauri_command_macros::command]
pub async fn clear_history() -> Result<String, String> {
    let service = HistoryService::with_default()
        .map_err(|e| format!("Failed to create HistoryService: {e}"))?;
    service
        .clear_async()
        .await
        .map_err(|e| format!("Failed to clear history: {e}"))?;
    Ok("History cleared successfully".to_string())
}

#[ccr_tauri_command_macros::command]
pub async fn update_config(
    name: String,
    platform: Option<String>,
    data: ConfigPatchInput,
    expected_version: Option<String>,
) -> Result<ConfigMutationResult, String> {
    let platform = required_platform(platform.as_deref()).map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        adapter::update(platform, &name, data, expected_version.as_deref())
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destructive_confirmation_requires_action_scoped_token() {
        assert!(validate_destructive_confirmation("delete_config", None).is_err());
        assert!(
            validate_destructive_confirmation("delete_config", Some("desktop-confirm:delete"))
                .is_err()
        );
        assert!(
            validate_destructive_confirmation(
                "delete_config",
                Some("desktop-confirm:delete_config")
            )
            .is_ok()
        );
    }

    #[test]
    fn restore_config_from_backup_path_rejects_missing_file() {
        let temp = tempfile::tempdir().expect("tempdir");
        let missing = temp.path().join("missing.toml");
        let error = restore_config_from_backup_path(&missing).expect_err("missing restore source");
        assert!(error.to_string().contains("missing.toml"));
    }

    #[test]
    fn restore_config_from_backup_path_rejects_invalid_toml_before_write() {
        let temp = tempfile::tempdir().expect("tempdir");
        let backup = temp.path().join("backup.toml");
        std::fs::write(&backup, "not = [valid").expect("write backup");
        let error = restore_config_from_backup_path(&backup).expect_err("invalid restore source");
        assert!(error.to_string().contains("TOML"));
    }

    #[test]
    fn restore_config_from_backup_path_accepts_ccr_config_shape() {
        let temp = tempfile::tempdir().expect("tempdir");
        let backup = temp.path().join("backup.toml");
        std::fs::write(
            &backup,
            "default_config = 'default'
current_config = 'default'
",
        )
        .expect("write backup");

        let parsed: CcsConfig =
            toml::from_str(&std::fs::read_to_string(&backup).expect("read backup"))
                .expect("valid restore TOML");
        assert_eq!(parsed.current_config, "default");
    }

    #[test]
    fn resolve_managed_restore_path_accepts_config_backup_filename_only() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config_path = temp.path().join("profiles.toml");
        let backup = resolve_managed_restore_path(
            &config_path,
            "profiles.toml.pre_restore_20260604_120000.bak",
        )
        .expect("managed backup");

        assert_eq!(
            backup,
            temp.path()
                .join("profiles.toml.pre_restore_20260604_120000.bak")
        );
    }

    #[test]
    fn resolve_managed_restore_path_rejects_path_traversal_and_absolute_paths() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config_path = temp.path().join("profiles.toml");

        for candidate in [
            "../profiles.toml.pre_restore_20260604_120000.bak",
            "nested/profiles.toml.pre_restore_20260604_120000.bak",
            "/tmp/profiles.toml.pre_restore_20260604_120000.bak",
        ] {
            let error = resolve_managed_restore_path(&config_path, candidate)
                .expect_err("unmanaged path must be rejected");
            assert!(error.to_string().contains("托管备份"));
        }
    }

    #[test]
    fn resolve_managed_restore_path_rejects_non_current_config_backup_names() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config_path = temp.path().join("profiles.toml");
        let error = resolve_managed_restore_path(&config_path, "config_20260604.toml.bak")
            .expect_err("wrong backup family");
        assert!(error.to_string().contains("当前配置备份"));
    }
}

// ── 退出确认设置 ──

#[ccr_tauri_command_macros::command]
pub async fn get_skip_exit_confirm(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(!state.desktop_shell_preferences().confirm_before_exit)
}

#[ccr_tauri_command_macros::command]
pub async fn set_skip_exit_confirm(skip: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.update_desktop_shell_preferences(|settings| {
        settings.confirm_before_exit = !skip;
    })?;
    Ok(())
}

// ── 备份清理 ──

#[ccr_tauri_command_macros::command]
pub async fn clean_backups() -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        // Preserve the legacy Tauri cleanup root; native callers inject their root.
        let root = dirs::home_dir()
            .ok_or_else(|| "Unable to determine home directory".to_string())?
            .join(".ccr");
        let result = ccr_app::config::backups::clean(&root);
        if !result.root_exists {
            return Ok("No CCR config directory found".to_string());
        }
        Ok(format!("Deleted {} backup file(s)", result.deleted))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
