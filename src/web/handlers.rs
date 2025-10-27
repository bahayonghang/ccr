// 🔌 Web 请求处理器
// 处理所有 HTTP 请求并调用相应的 Service
// 🎯 异步架构 - 使用 Axum 提供高性能处理

use crate::core::error::CcrError;
use crate::core::logging::ColorOutput;
use crate::managers::config::ConfigSection;
use crate::services::{BackupService, ConfigService, HistoryService, SettingsService, SyncService};
use crate::services::sync_service::get_ccr_sync_path;
use crate::managers::config::{ConfigManager, SyncConfig};
use crate::web::models::*;
use crate::web::system_info_cache::SystemInfoCache;
use axum::{
    Json,
    extract::{Path, State as AxumState},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
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
pub async fn handle_list_configs(AxumState(_state): AxumState<AppState>) -> Response {
    // 🆕 每次请求都重新创建 ConfigService 以获取最新的平台配置
    // 这样在 Unified 模式下切换平台后，能正确读取新平台的配置
    let result = tokio::task::spawn_blocking(move || {
        // 重新创建 ConfigService，它会自动检测当前平台
        let config_service = crate::services::ConfigService::default()?;
        config_service.list_configs()
    })
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
pub async fn handle_switch_config(
    AxumState(state): AxumState<AppState>,
    Json(req): Json<SwitchRequest>,
) -> Response {
    // 🚀 使用 spawn_blocking 执行同步操作
    let config_name = req.config_name.clone();
    let result = tokio::task::spawn_blocking(move || {
        // 确保历史记录服务被正确传递
        crate::commands::switch_command(&config_name)
    })
    .await
    .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(_) => {
            // 切换成功后，给文件系统一点时间确保历史记录写入完成
            // 这对于某些文件系统（特别是网络文件系统）可能是必要的
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            // 验证历史记录已成功写入
            match state.history_service.get_recent(1) {
                Ok(_) => Json(ApiResponse::success("配置切换成功")).into_response(),
                Err(e) => {
                    log::warn!("历史记录可能未正确保存: {}", e);
                    // 虽然历史记录可能有问题，但配置切换本身是成功的
                    Json(ApiResponse::success("配置切换成功（历史记录可能延迟）")).into_response()
                }
            }
        }
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
    let result =
        tokio::task::spawn_blocking(move || state.config_service.add_config(name, section))
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

// ===== ☁️ 同步相关处理器 =====

/// 获取同步状态
pub async fn handle_sync_status() -> Response {
    let manager = match ConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    };

    let config = match manager.load() {
        Ok(c) => c,
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    };

    let mut response = crate::web::models::SyncStatusResponse {
        configured: false,
        enabled: false,
        webdav_url: None,
        username: None,
        remote_path: None,
        auto_sync: None,
        sync_type: None,
        local_path: None,
        remote_exists: None,
    };

    if let Some(sync) = &config.settings.sync {
        response.configured = true;
        response.enabled = sync.enabled;
        response.webdav_url = Some(sync.webdav_url.clone());
        response.username = Some(sync.username.clone());
        response.remote_path = Some(sync.remote_path.clone());
        response.auto_sync = Some(sync.auto_sync);

        if let Ok(local_path) = get_ccr_sync_path() {
            response.local_path = Some(local_path.display().to_string());
            let sync_type = if local_path.is_dir() { "directory" } else { "file" };
            response.sync_type = Some(sync_type.to_string());

            // 检查远程是否存在（直接异步调用）
            match SyncService::new(sync).await {
                Ok(service) => match service.remote_exists().await {
                    Ok(exists) => response.remote_exists = Some(exists),
                    Err(_) => response.remote_exists = Some(false),
                },
                Err(_) => response.remote_exists = Some(false),
            }
        }
    }

    Json(crate::web::models::ApiResponse::success(response)).into_response()
}

/// 设置/更新同步配置
pub async fn handle_sync_config(Json(req): Json<crate::web::models::SyncConfigRequest>) -> Response {
    let manager = match ConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    };

    // 构建并保存配置
    let mut config = match manager.load() {
        Ok(c) => c,
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    };

    let sync_cfg = SyncConfig {
        enabled: req.enabled,
        webdav_url: req.webdav_url,
        username: req.username,
        password: req.password,
        remote_path: req.remote_path,
        auto_sync: req.auto_sync,
    };

    // 测试连接（直接异步调用）
    let service = match SyncService::new(&sync_cfg).await {
        Ok(s) => s,
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    };
    if let Err(e) = service.test_connection().await {
        let error_response: crate::web::models::ApiResponse<()> =
            crate::web::models::ApiResponse::error_without_data(e.user_message());
        return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
    }

    // 保存配置
    config.settings.sync = Some(sync_cfg);
    if let Err(e) = manager.save(&config) {
        let error_response: crate::web::models::ApiResponse<()> =
            crate::web::models::ApiResponse::error_without_data(e.user_message());
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
    }

    Json(crate::web::models::ApiResponse::success(
        crate::web::models::SyncOperationResponse {
            message: "同步配置已保存，并通过连接测试".to_string(),
        },
    ))
    .into_response()
}

/// 执行同步 push
pub async fn handle_sync_push(Json(req): Json<crate::web::models::SyncOperationRequest>) -> Response {
    let manager = match ConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    };

    let config = match manager.load() {
        Ok(c) => c,
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    };

    let sync_cfg = match &config.settings.sync {
        Some(s) if s.enabled => s,
        Some(_) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data("同步功能已禁用".into());
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
        None => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(
                    "同步功能未配置，请先运行 'ccr sync config' 或在页面中配置".into(),
                );
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    };

    let local_path = match get_ccr_sync_path() {
        Ok(p) => p,
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    };

    let push_result = {
        match SyncService::new(sync_cfg).await {
            Ok(service) => {
                if !req.force {
                    // 如果远程存在且未强制，提示错误（UI 上可做确认弹窗）
                    match service.remote_exists().await {
                        Ok(exists) => {
                            if exists {
                                Err(CcrError::SyncError("远程已存在同名内容，请使用 force 或先清理".into()))
                            } else {
                                service.push(&local_path).await
                            }
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    service.push(&local_path).await
                }
            }
            Err(e) => Err(e),
        }
    };

    match push_result {
        Ok(_) => Json(crate::web::models::ApiResponse::success(
            crate::web::models::SyncOperationResponse {
                message: "已成功上传到云端".to_string(),
            },
        ))
        .into_response(),
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
        }
    }
}

/// 执行同步 pull
pub async fn handle_sync_pull(Json(_req): Json<crate::web::models::SyncOperationRequest>) -> Response {
    let manager = match ConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    };

    let config = match manager.load() {
        Ok(c) => c,
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    };

    let sync_cfg = match &config.settings.sync {
        Some(s) if s.enabled => s,
        Some(_) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data("同步功能已禁用".into());
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
        None => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(
                    "同步功能未配置，请先运行 'ccr sync config' 或在页面中配置".into(),
                );
            return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
        }
    };

    let local_path = match get_ccr_sync_path() {
        Ok(p) => p,
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    };

    let pull_result = {
        match SyncService::new(sync_cfg).await {
            Ok(service) => service.pull(&local_path).await,
            Err(e) => Err(e),
        }
    };

    match pull_result {
        Ok(_) => Json(crate::web::models::ApiResponse::success(
            crate::web::models::SyncOperationResponse {
                message: "已成功从云端下载并应用".to_string(),
            },
        ))
        .into_response(),
        Err(e) => {
            let error_response: crate::web::models::ApiResponse<()> =
                crate::web::models::ApiResponse::error_without_data(e.user_message());
            (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
        }
    }
}

/// 处理获取历史记录
pub async fn handle_get_history(AxumState(state): AxumState<AppState>) -> Response {
    log::debug!("开始获取历史记录");

    let result = tokio::task::spawn_blocking(move || {
        let entries = state.history_service.get_recent(50)?;
        log::info!("成功加载 {} 条历史记录", entries.len());
        Ok(entries)
    })
    .await
    .unwrap_or_else(|e| Err(CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(entries) => {
            log::debug!("准备序列化 {} 条历史记录为 JSON", entries.len());

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

            log::debug!("返回 {} 条历史记录给前端", json_entries.len());
            Json(ApiResponse::success(response_data)).into_response()
        }
        Err(e) => {
            log::error!("获取历史记录失败: {}", e);
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
    let result =
        tokio::task::spawn_blocking(move || state.backup_service.clean_old_backups(days, dry_run))
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
    let result = tokio::task::spawn_blocking(move || state.settings_service.get_current_settings())
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
    let result =
        tokio::task::spawn_blocking(move || state.config_service.export_config(include_secrets))
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
        state.config_service.import_config(&content, mode, backup)
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

// ===== 🆕 平台管理处理器 (Unified Mode) =====

/// 处理获取平台信息
/// 🎯 返回当前模式（Legacy/Unified）和平台列表
pub async fn handle_get_platform_info() -> Response {
    use crate::managers::{ConfigManager, PlatformConfigManager};

    // 🔍 检测配置模式
    let (is_unified, unified_config_path) = ConfigManager::detect_unified_mode();

    if !is_unified {
        // Legacy 模式
        let response = PlatformInfoResponse {
            mode: "legacy".to_string(),
            current_platform: None,
            available_platforms: None,
        };
        return Json(ApiResponse::success(response)).into_response();
    }

    // Unified 模式
    let unified_path = match unified_config_path {
        Some(path) => path,
        None => {
            let error_response: ApiResponse<()> =
                ApiResponse::error_without_data("无法获取统一配置路径".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    };

    let platform_manager = PlatformConfigManager::new(unified_path);

    let result = tokio::task::spawn_blocking(move || {
        // 读取统一配置
        let unified_config = platform_manager.load()?;
        let current_platform = unified_config.current_platform.clone();

        // 构建平台列表
        let mut platforms: Vec<PlatformItem> = unified_config
            .platforms
            .into_iter()
            .map(|(name, entry)| PlatformItem {
                name: name.clone(),
                enabled: entry.enabled,
                current_profile: entry.current_profile,
                description: entry.description,
                last_used: entry.last_used,
                is_current: name == current_platform,
            })
            .collect();

        // 按名称排序
        platforms.sort_by(|a, b| a.name.cmp(&b.name));

        Ok::<(String, Vec<PlatformItem>), crate::core::error::CcrError>((current_platform, platforms))
    })
    .await
    .unwrap_or_else(|e| Err(crate::core::error::CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok((current_platform, platforms)) => {
            let response = PlatformInfoResponse {
                mode: "unified".to_string(),
                current_platform: Some(current_platform),
                available_platforms: Some(platforms),
            };
            Json(ApiResponse::success(response)).into_response()
        }
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// 处理切换平台
/// 🎯 在 Unified 模式下切换当前激活的平台
pub async fn handle_switch_platform(Json(req): Json<SwitchPlatformRequest>) -> Response {
    use crate::managers::{ConfigManager, PlatformConfigManager};

    // 🔍 检测配置模式
    let (is_unified, unified_config_path) = ConfigManager::detect_unified_mode();

    if !is_unified {
        let error_response: ApiResponse<()> =
            ApiResponse::error_without_data("平台切换仅在 Unified 模式下可用".to_string());
        return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
    }

    let unified_path = match unified_config_path {
        Some(path) => path,
        None => {
            let error_response: ApiResponse<()> =
                ApiResponse::error_without_data("无法获取统一配置路径".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    };

    let platform_name = req.platform_name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let platform_manager = PlatformConfigManager::new(unified_path);

        // 读取配置
        let mut unified_config = platform_manager.load()?;

        // 检查平台是否存在
        if !unified_config.platforms.contains_key(&platform_name) {
            return Err(crate::core::error::CcrError::ConfigSectionNotFound(
                format!("平台 '{}'", platform_name)
            ));
        }

        // 切换平台
        unified_config.current_platform = platform_name.clone();

        // 更新 last_used
        if let Some(entry) = unified_config.platforms.get_mut(&platform_name) {
            entry.last_used = Some(chrono::Utc::now().to_rfc3339());
        }

        // 保存配置
        platform_manager.save(&unified_config)?;

        Ok::<String, crate::core::error::CcrError>(platform_name)
    })
    .await
    .unwrap_or_else(|e| Err(crate::core::error::CcrError::ConfigError(format!("任务执行失败: {}", e))));

    match result {
        Ok(platform_name) => {
            let message = format!("已切换到平台: {}", platform_name);
            Json(ApiResponse::success(message)).into_response()
        }
        Err(e) => {
            let error_response: ApiResponse<()> = ApiResponse::error_without_data(e.user_message());
            (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
        }
    }
}
