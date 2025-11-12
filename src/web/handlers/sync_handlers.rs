// ☁️ 同步功能处理器
// 处理 WebDAV 同步相关的所有请求

use crate::managers::sync_config::{SyncConfig, SyncConfigManager};
use crate::services::SyncService;
use crate::services::sync_service::get_ccr_sync_path;
use crate::web::{error_utils::*, models::*};
use axum::Json;
use axum::response::Response;

/// 获取同步状态
pub async fn handle_sync_status() -> Response {
    let sync_manager = match SyncConfigManager::with_default() {
        Ok(m) => m,
        Err(e) => return internal_server_error(e.to_string()),
    };

    let sync = match sync_manager.load() {
        Ok(s) => s,
        Err(e) => return internal_server_error(e.to_string()),
    };

    let mut response = SyncStatusResponse {
        configured: sync.enabled,
        enabled: sync.enabled,
        webdav_url: Some(sync.webdav_url.clone()),
        username: Some(sync.username.clone()),
        remote_path: Some(sync.remote_path.clone()),
        auto_sync: Some(sync.auto_sync),
        sync_type: None,
        local_path: None,
        remote_exists: None,
    };

    if sync.enabled
        && let Ok(local_path) = get_ccr_sync_path()
    {
        response.local_path = Some(local_path.display().to_string());
        let sync_type = if local_path.is_dir() {
            "directory"
        } else {
            "file"
        };
        response.sync_type = Some(sync_type.to_string());

        // 检查远程是否存在（直接异步调用）
        match SyncService::new(&sync).await {
            Ok(service) => match service.remote_exists().await {
                Ok(exists) => response.remote_exists = Some(exists),
                Err(_) => response.remote_exists = Some(false),
            },
            Err(_) => response.remote_exists = Some(false),
        }
    }

    success_response(response)
}

/// 设置/更新同步配置
pub async fn handle_sync_config(Json(req): Json<SyncConfigRequest>) -> Response {
    let sync_manager = match SyncConfigManager::with_default() {
        Ok(m) => m,
        Err(e) => return internal_server_error(e.to_string()),
    };

    let sync_cfg = SyncConfig {
        enabled: req.enabled,
        webdav_url: req.webdav_url,
        username: req.username,
        password: req.password,
        remote_path: req.remote_path,
        auto_sync: req.auto_sync,
    };

    // 测试连接
    let service = match SyncService::new(&sync_cfg).await {
        Ok(s) => s,
        Err(e) => return bad_request(e.to_string()),
    };

    if let Err(e) = service.test_connection().await {
        return bad_request(e.to_string());
    }

    // 保存配置到独立文件
    if let Err(e) = sync_manager.save(&sync_cfg) {
        return internal_server_error(e.to_string());
    }

    success_response(SyncOperationResponse {
        message: "同步配置已保存，并通过连接测试".to_string(),
    })
}

/// 执行同步 push
pub async fn handle_sync_push(Json(req): Json<SyncOperationRequest>) -> Response {
    let sync_manager = match SyncConfigManager::with_default() {
        Ok(m) => m,
        Err(e) => return internal_server_error(e.to_string()),
    };

    let sync_cfg = match sync_manager.load() {
        Ok(c) => c,
        Err(e) => return internal_server_error(e.to_string()),
    };

    if !sync_cfg.enabled {
        return bad_request("同步功能未配置或已禁用");
    }

    let local_path = match get_ccr_sync_path() {
        Ok(p) => p,
        Err(e) => return internal_server_error(e.to_string()),
    };

    let service = match SyncService::new(&sync_cfg).await {
        Ok(s) => s,
        Err(e) => return internal_server_error(e.to_string()),
    };

    let push_result = if !req.force {
        // 如果远程存在且未强制，提示错误（UI 上可做确认弹窗）
        match service.remote_exists().await {
            Ok(exists) => {
                if exists {
                    return bad_request("远程已存在同名内容，请使用 force 或先清理");
                } else {
                    service.push(&local_path, None).await
                }
            }
            Err(e) => Err(e),
        }
    } else {
        service.push(&local_path, None).await
    };

    match push_result {
        Ok(_) => success_response(SyncOperationResponse {
            message: "已成功上传到云端".to_string(),
        }),
        Err(e) => bad_request(e.to_string()),
    }
}

/// 执行同步 pull
pub async fn handle_sync_pull(Json(_req): Json<SyncOperationRequest>) -> Response {
    let sync_manager = match SyncConfigManager::with_default() {
        Ok(m) => m,
        Err(e) => return internal_server_error(e.to_string()),
    };

    let sync_cfg = match sync_manager.load() {
        Ok(c) => c,
        Err(e) => return internal_server_error(e.to_string()),
    };

    if !sync_cfg.enabled {
        return bad_request("同步功能未配置或已禁用");
    }

    let local_path = match get_ccr_sync_path() {
        Ok(p) => p,
        Err(e) => return internal_server_error(e.to_string()),
    };

    let service = match SyncService::new(&sync_cfg).await {
        Ok(s) => s,
        Err(e) => return internal_server_error(e.to_string()),
    };

    match service.pull(&local_path).await {
        Ok(_) => success_response(SyncOperationResponse {
            message: "已成功从云端下载并应用".to_string(),
        }),
        Err(e) => bad_request(e.to_string()),
    }
}
