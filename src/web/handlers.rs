// 🔌 Web 请求处理器
// 处理所有 HTTP 请求并调用相应的 Service
// 🎯 异步架构 - 使用 Axum 提供高性能处理

use crate::managers::config::ConfigSection;
use crate::core::error::CcrError;
use crate::core::logging::ColorOutput;
use crate::services::{BackupService, ConfigService, HistoryService, SettingsService};
use crate::web::models::*;
use crate::web::system_info_cache::SystemInfoCache;
use axum::{
    extract::{Path, State as AxumState},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use std::sync::Arc;

/// 🔌 共享状态
///
/// 持有所有 Service 的引用，在所有请求处理器间共享
#[derive(Clone)]
pub struct AppState {
    pub config_service: Arc<ConfigService>,
    pub settings_service: Arc<SettingsService>,
    pub history_service: Arc<HistoryService>,
    pub backup_service: Arc<BackupService>,
    pub system_info_cache: Arc<SystemInfoCache>,
}

impl AppState {
    /// 🏗️ 创建新的应用状态
    pub fn new(
        config_service: Arc<ConfigService>,
        settings_service: Arc<SettingsService>,
        history_service: Arc<HistoryService>,
        backup_service: Arc<BackupService>,
        system_info_cache: Arc<SystemInfoCache>,
    ) -> Self {
        Self {
            config_service,
            settings_service,
            history_service,
            backup_service,
            system_info_cache,
        }
    }
}

// === 静态文件处理器 ===

/// 提供 HTML 页面
pub async fn serve_html() -> Html<&'static str> {
    Html(include_str!("../../web/index.html"))
}

/// 提供 CSS 样式文件
pub async fn serve_css() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../../web/style.css"),
    )
}

/// 提供 JavaScript 脚本文件
pub async fn serve_js() -> impl IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../../web/script.js"),
    )
}

// === API 处理器 ===

/// 处理列出配置
/// 🎯 异步处理，使用 spawn_blocking 避免阻塞运行时
pub async fn handle_list_configs(AxumState(state): AxumState<AppState>) -> Response {
    // 🚀 使用 spawn_blocking 执行同步操作
    let result = tokio::task::spawn_blocking(move || state.config_service.list_configs())
        .await
        .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(list) => {
            let configs: Vec<ConfigItem> = list
                .configs
                .into_iter()
                .map(|info| ConfigItem {
                    name: info.name.clone(),
                    description: info.description.clone(),
                    base_url: info.base_url.clone().unwrap_or_default(),
                    auth_token: ColorOutput::mask_sensitive(
                        &info.auth_token.clone().unwrap_or_default(),
                    ),
                    model: info.model,
                    small_fast_model: info.small_fast_model,
                    is_current: info.is_current,
                    is_default: info.is_default,
                    provider: info.provider,
                    provider_type: info.provider_type,
                    account: info.account,
                    tags: info.tags,
                })
                .collect();

            let response_data = ConfigListResponse {
                current_config: list.current_config,
                default_config: list.default_config,
                configs,
            };

            Json(ApiResponse::success(response_data)).into_response()
        }
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理切换配置
pub async fn handle_switch_config(Json(req): Json<SwitchRequest>) -> Response {
    // 🚀 使用 spawn_blocking 执行同步操作
    let config_name = req.config_name.clone();
    let result = tokio::task::spawn_blocking(move || crate::commands::switch_command(&config_name))
        .await
        .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(_) => Json(ApiResponse::success("配置切换成功")).into_response(),
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理添加配置
pub async fn handle_add_config(
    AxumState(state): AxumState<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> Response {
    let section = ConfigSection {
        description: req.description,
        base_url: Some(req.base_url),
        auth_token: Some(req.auth_token),
        model: req.model,
        small_fast_model: req.small_fast_model,
        provider: req.provider,
        provider_type: req.provider_type.and_then(|t| match t.as_str() {
            "official_relay" => Some(crate::managers::config::ProviderType::OfficialRelay),
            "third_party_model" => Some(crate::managers::config::ProviderType::ThirdPartyModel),
            _ => None,
        }),
        account: req.account,
        tags: req.tags,
    };

    let name = req.name.clone();
    let result = tokio::task::spawn_blocking(move || state.config_service.add_config(name, section))
        .await
        .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(_) => Json(ApiResponse::success("配置添加成功")).into_response(),
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理更新配置
pub async fn handle_update_config(
    AxumState(state): AxumState<AppState>,
    Path(old_name): Path<String>,
    Json(req): Json<UpdateConfigRequest>,
) -> Response {
    let section = ConfigSection {
        description: req.description,
        base_url: Some(req.base_url),
        auth_token: Some(req.auth_token),
        model: req.model,
        small_fast_model: req.small_fast_model,
        provider: req.provider,
        provider_type: req.provider_type.and_then(|t| match t.as_str() {
            "official_relay" => Some(crate::managers::config::ProviderType::OfficialRelay),
            "third_party_model" => Some(crate::managers::config::ProviderType::ThirdPartyModel),
            _ => None,
        }),
        account: req.account,
        tags: req.tags,
    };

    let new_name = req.name.clone();
    let result = tokio::task::spawn_blocking(move || {
        state
            .config_service
            .update_config(&old_name, new_name, section)
    })
    .await
    .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(_) => Json(ApiResponse::success("配置更新成功")).into_response(),
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理删除配置
pub async fn handle_delete_config(
    AxumState(state): AxumState<AppState>,
    Path(name): Path<String>,
) -> Response {
    let result = tokio::task::spawn_blocking(move || state.config_service.delete_config(&name))
        .await
        .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(_) => Json(ApiResponse::success("配置删除成功")).into_response(),
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理获取历史记录
pub async fn handle_get_history(AxumState(state): AxumState<AppState>) -> Response {
    let result = tokio::task::spawn_blocking(move || state.history_service.get_recent(50))
        .await
        .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(entries) => {
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

            Json(ApiResponse::success(response_data)).into_response()
        }
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理验证配置
pub async fn handle_validate() -> Response {
    let result = tokio::task::spawn_blocking(|| crate::commands::validate_command())
        .await
        .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(_) => Json(ApiResponse::success("验证通过")).into_response(),
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理清理备份
pub async fn handle_clean(
    AxumState(state): AxumState<AppState>,
    Json(req): Json<CleanRequest>,
) -> Response {
    let days = req.days;
    let dry_run = req.dry_run;
    let result = tokio::task::spawn_blocking(move || {
        state.backup_service.clean_old_backups(days, dry_run)
    })
    .await
    .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(result) => {
            let response_data = CleanResponse {
                deleted_count: result.deleted_count,
                skipped_count: result.skipped_count,
                total_size_mb: result.total_size as f64 / 1024.0 / 1024.0,
                dry_run,
            };
            Json(ApiResponse::success(response_data)).into_response()
        }
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理获取 Settings
pub async fn handle_get_settings(AxumState(state): AxumState<AppState>) -> Response {
    let result =
        tokio::task::spawn_blocking(move || state.settings_service.get_current_settings())
            .await
            .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(settings) => match serde_json::to_value(&settings) {
            Ok(settings_value) => {
                let response_data = SettingsResponse {
                    settings: settings_value,
                };
                Json(ApiResponse::success(response_data)).into_response()
            }
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(format!("序列化设置失败: {}", e));
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
            }
        },
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理获取 Settings 备份列表
pub async fn handle_get_settings_backups(AxumState(state): AxumState<AppState>) -> Response {
    let result = tokio::task::spawn_blocking(move || state.settings_service.list_backups())
        .await
        .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(backups) => {
            let backup_items: Vec<BackupItem> = backups
                .iter()
                .map(|backup| BackupItem {
                    filename: backup.filename.clone(),
                    path: backup.path.to_string_lossy().to_string(),
                    created_at: chrono::DateTime::<chrono::Utc>::from(backup.created_at)
                        .to_rfc3339(),
                    size_bytes: backup.size_bytes,
                })
                .collect();

            let response_data = SettingsBackupsResponse {
                backups: backup_items,
            };

            Json(ApiResponse::success(response_data)).into_response()
        }
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理恢复 Settings
pub async fn handle_restore_settings(
    AxumState(state): AxumState<AppState>,
    Json(req): Json<RestoreSettingsRequest>,
) -> Response {
    let backup_path = req.backup_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        state
            .settings_service
            .restore_settings(std::path::Path::new(&backup_path))
    })
    .await
    .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(_) => Json(ApiResponse::success("Settings 恢复成功")).into_response(),
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理导出配置
pub async fn handle_export(
    AxumState(state): AxumState<AppState>,
    Json(req): Json<ExportRequest>,
) -> Response {
    let include_secrets = req.include_secrets;
    let result = tokio::task::spawn_blocking(move || state.config_service.export_config(include_secrets))
        .await
        .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(content) => {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!(".ccs_config_{}.toml", timestamp);

            let response_data = ExportResponse { content, filename };
            Json(ApiResponse::success(response_data)).into_response()
        }
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理导入配置
pub async fn handle_import(
    AxumState(state): AxumState<AppState>,
    Json(req): Json<ImportRequest>,
) -> Response {
    let mode = if req.mode == "replace" {
        crate::services::config_service::ImportMode::Replace
    } else {
        crate::services::config_service::ImportMode::Merge
    };

    let content = req.content.clone();
    let backup = req.backup;
    let result = tokio::task::spawn_blocking(move || {
        state
            .config_service
            .import_config(&content, mode, backup)
    })
    .await
    .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(result) => {
            let response_data = ImportResponse {
                added: result.added,
                updated: result.updated,
                skipped: result.skipped,
            };
            Json(ApiResponse::success(response_data)).into_response()
        }
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理获取系统信息
/// 🎯 优化：使用缓存避免每次请求都扫描系统
pub async fn handle_get_system_info(AxumState(state): AxumState<AppState>) -> Response {
    // 🚀 直接从缓存读取，极快！
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

    Json(ApiResponse::success(system_info)).into_response()
}
