// 🎯 系统和设置处理器
// 处理系统信息查询、历史记录、设置管理等请求

use crate::web::{
    error_utils::{spawn_blocking_string, *},
    handlers::AppState,
    models::*,
};
use axum::{Json, extract::State, response::Response};
use std::sync::Arc;

/// 处理获取系统信息请求
/// 🎯 使用缓存避免重复查询系统信息
pub async fn handle_get_system_info(State(state): State<AppState>) -> Response {
    // 🚀 直接从缓存读取，性能极快！
    let cached_info = state.system_info_cache.get();

    let system_info = SystemInfoResponse {
        hostname: cached_info.hostname,
        os: cached_info.os,
        os_version: cached_info.os_version,
        kernel_version: cached_info.kernel_version,
        cpu_brand: cached_info.cpu_brand,
        cpu_cores: cached_info.cpu_cores,
        cpu_usage: cached_info.cpu_usage,
        total_memory_gb: cached_info.total_memory_gb,
        used_memory_gb: cached_info.used_memory_gb,
        memory_usage_percent: cached_info.memory_usage_percent,
        total_swap_gb: cached_info.total_swap_gb,
        used_swap_gb: cached_info.used_swap_gb,
        uptime_seconds: cached_info.uptime_seconds,
    };

    success_response(system_info)
}

/// 处理获取历史记录请求
pub async fn handle_get_history(State(state): State<AppState>) -> Response {
    tracing::debug!("开始获取历史记录");

    let history_service = Arc::clone(&state.history_service);
    let entries = match history_service.get_recent_async(50).await {
        Ok(entries) => {
            tracing::debug!("成功加载 {} 条历史记录", entries.len());
            entries
        }
        Err(e) => return internal_server_error(e.to_string()),
    };

    tracing::debug!("准备序列化 {} 条历史记录为 JSON", entries.len());

    let json_entries: Vec<HistoryEntryJson> = entries
        .iter()
        .map(|entry| HistoryEntryJson {
            id: entry.id.clone(),
            timestamp: entry.timestamp.to_rfc3339(),
            operation: entry.operation.as_str().to_string(),
            actor: entry.actor.clone(),
            from_config: entry.details.from_config.clone(),
            to_config: entry.details.to_config.clone(),
            changes: entry
                .env_changes
                .iter()
                .map(|change| EnvChangeJson {
                    key: change.var_name.clone(),
                    old_value: change.old_value.clone(),
                    new_value: change.new_value.clone(),
                })
                .collect(),
        })
        .collect();

    let response_data = HistoryResponse {
        entries: json_entries.clone(),
        total: json_entries.len(),
    };

    tracing::debug!("返回 {} 条历史记录给前端", json_entries.len());
    success_response(response_data)
}

/// 处理验证配置请求
/// 🎯 通过 ValidateService 执行验证（修复层级违规）
pub async fn handle_validate(State(state): State<AppState>) -> Response {
    let validate_service = Arc::clone(&state.validate_service);
    let result = spawn_blocking_string(move || validate_service.quick_validate()).await;

    match result {
        Ok(report) => {
            if report.invalid_count == 0 {
                success_response(serde_json::json!({
                    "status": "success",
                    "message": "验证通过",
                    "valid_count": report.valid_count,
                    "invalid_count": report.invalid_count
                }))
            } else {
                success_response(serde_json::json!({
                    "status": "warning",
                    "message": format!("{} 个配置节验证失败", report.invalid_count),
                    "valid_count": report.valid_count,
                    "invalid_count": report.invalid_count,
                    "results": report.results.iter().map(|(name, valid, error)| {
                        serde_json::json!({
                            "name": name,
                            "valid": valid,
                            "error": error
                        })
                    }).collect::<Vec<_>>()
                }))
            }
        }
        Err(e) => internal_server_error(e),
    }
}

/// 处理清理旧备份请求
pub async fn handle_clean(
    State(state): State<AppState>,
    Json(req): Json<CleanRequest>,
) -> Response {
    let backup_service = Arc::clone(&state.backup_service);
    let result = backup_service
        .clean_old_backups_async(req.days, req.dry_run)
        .await;

    match result {
        Ok(result) => {
            let response_data = CleanResponse {
                deleted_count: result.deleted_count,
                skipped_count: result.skipped_count,
                total_size_mb: result.total_size as f64 / 1024.0 / 1024.0,
                dry_run: req.dry_run,
            };
            success_response(response_data)
        }
        Err(e) => internal_server_error(e.to_string()),
    }
}

/// 处理获取 Settings 请求
pub async fn handle_get_settings(State(state): State<AppState>) -> Response {
    let settings_service = Arc::clone(&state.settings_service);
    let settings = match settings_service.get_current_settings_async().await {
        Ok(settings) => settings,
        Err(e) => return internal_server_error(e.to_string()),
    };

    match serde_json::to_value(&settings) {
        Ok(settings_value) => {
            let response_data = SettingsResponse {
                settings: settings_value,
            };
            success_response(response_data)
        }
        Err(e) => internal_server_error(format!("序列化设置失败: {}", e)),
    }
}

/// 处理获取 Settings 备份列表请求
pub async fn handle_get_settings_backups(State(state): State<AppState>) -> Response {
    let settings_service = Arc::clone(&state.settings_service);
    let backups = match settings_service.list_backups_async().await {
        Ok(backups) => backups,
        Err(e) => return internal_server_error(e.to_string()),
    };

    let backup_items: Vec<BackupItem> = backups
        .iter()
        .map(|backup| BackupItem {
            filename: backup.filename.clone(),
            path: backup.path.to_string_lossy().to_string(),
            created_at: chrono::DateTime::<chrono::Utc>::from(backup.created_at).to_rfc3339(),
            size_bytes: backup.size_bytes,
        })
        .collect();

    let response_data = SettingsBackupsResponse {
        backups: backup_items,
    };

    success_response(response_data)
}

/// 处理恢复 Settings 请求
pub async fn handle_restore_settings(
    State(state): State<AppState>,
    Json(req): Json<RestoreSettingsRequest>,
) -> Response {
    let settings_service = Arc::clone(&state.settings_service);
    let backup_path = req.backup_path.clone();

    let result = settings_service
        .restore_settings_async(std::path::Path::new(&backup_path))
        .await;

    match result {
        Ok(_) => success_response("Settings 恢复成功"),
        Err(e) => internal_server_error(e.to_string()),
    }
}

/// 处理重新加载配置缓存请求
pub async fn handle_reload_config(State(state): State<AppState>) -> Response {
    let result = spawn_blocking_string(move || state.reload_config_cache()).await;

    match result {
        Ok(()) => success_response("配置缓存已重新加载"),
        Err(e) => internal_server_error(e),
    }
}
