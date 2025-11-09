// Sync Handlers - WebDAV Configuration Synchronization
// Execute sync commands and manage sync configuration

use crate::executor;
use crate::models::*;
use axum::{Json, response::IntoResponse};

/// 解析 sync status 输出
fn parse_sync_status(output: &str) -> Option<SyncConfigDetails> {
    let mut enabled = false;
    let mut webdav_url = String::new();
    let mut username = String::new();
    let mut remote_path = String::new();
    let mut auto_sync = false;
    let mut remote_file_exists = None;

    for line in output.lines() {
        let line = line.trim();

        // 状态：✓ 已启用
        if line.contains("状态") && line.contains("已启用") {
            enabled = true;
        }

        // WebDAV 服务器：https://...
        if line.contains("WebDAV 服务器") || line.contains("服务器") {
            if let Some(url) = line.split('│').nth(1) {
                webdav_url = url.trim().to_string();
            }
        }

        // 用户名：xxx
        if line.contains("用户名") {
            if let Some(user) = line.split('│').nth(1) {
                username = user.trim().to_string();
            }
        }

        // 远程路径：/ccr/.ccs_config.toml
        if line.contains("远程路径") || line.contains("远程文件路径") {
            if let Some(path) = line.split('│').nth(1) {
                remote_path = path.trim().to_string();
            }
        }

        // 自动同步：✓ 开启 / ✗ 关闭
        if line.contains("自动同步") {
            if line.contains("开启") || line.contains("✓") {
                auto_sync = true;
            }
        }

        // 远程配置文件存在
        if line.contains("远程配置文件存在") || line.contains("远程文件存在") {
            remote_file_exists = Some(true);
        } else if line.contains("远程配置文件不存在") || line.contains("远程文件不存在")
        {
            remote_file_exists = Some(false);
        }
    }

    if !webdav_url.is_empty() {
        Some(SyncConfigDetails {
            enabled,
            webdav_url,
            username,
            remote_path,
            auto_sync,
            remote_file_exists,
        })
    } else {
        None
    }
}

/// POST /api/sync/config - Interactive sync configuration (not supported in web API)
pub async fn configure_sync() -> impl IntoResponse {
    ApiResponse::<String>::error(
        "Interactive sync configuration is not supported via web API. Please use CLI: ccr sync config".to_string()
    )
}

/// GET /api/sync/status - Get sync status and configuration
pub async fn get_sync_status() -> impl IntoResponse {
    let args = vec!["sync".to_string(), "status".to_string()];

    match executor::execute_command(args).await {
        Ok(output) => {
            if output.success {
                let configured = !output.stdout.contains("同步功能未配置")
                    && !output.stdout.contains("Sync not configured");
                let config = if configured {
                    parse_sync_status(&output.stdout)
                } else {
                    None
                };

                let response = SyncStatusResponse {
                    success: true,
                    output: output.stdout,
                    configured,
                    config,
                };
                ApiResponse::success(response)
            } else {
                ApiResponse::<SyncStatusResponse>::error(output.stderr)
            }
        }
        Err(e) => ApiResponse::<SyncStatusResponse>::error(e.to_string()),
    }
}

/// POST /api/sync/push - Upload config to cloud
pub async fn push_config(Json(req): Json<SyncOperationRequest>) -> impl IntoResponse {
    let mut args = vec!["sync".to_string(), "push".to_string()];

    if req.force {
        args.push("--force".to_string());
    }

    match executor::execute_command(args).await {
        Ok(output) => {
            let response = SyncOperationResponse {
                success: output.success,
                output: output.stdout,
                error: output.stderr,
                duration_ms: output.duration_ms,
            };
            ApiResponse::success(response)
        }
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

/// POST /api/sync/pull - Download config from cloud
pub async fn pull_config(Json(req): Json<SyncOperationRequest>) -> impl IntoResponse {
    let mut args = vec!["sync".to_string(), "pull".to_string()];

    if req.force {
        args.push("--force".to_string());
    }

    match executor::execute_command(args).await {
        Ok(output) => {
            let response = SyncOperationResponse {
                success: output.success,
                output: output.stdout,
                error: output.stderr,
                duration_ms: output.duration_ms,
            };
            ApiResponse::success(response)
        }
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

/// GET /api/sync/info - Get sync feature information
pub async fn get_sync_info() -> impl IntoResponse {
    let info = SyncInfoResponse {
        feature_name: "WebDAV 云同步".to_string(),
        description:
            "使用 WebDAV 协议在多台设备间同步 CCR 配置文件，支持目录同步，智能排除不必要的文件"
                .to_string(),
        supported_services: vec![
            "坚果云 (Nutstore)".to_string(),
            "Nextcloud".to_string(),
            "ownCloud".to_string(),
            "任何标准 WebDAV 服务器".to_string(),
        ],
        setup_steps: vec![
            "在 CLI 中运行 'ccr sync config' 配置 WebDAV 连接".to_string(),
            "输入 WebDAV 服务器地址、用户名和密码（坚果云建议使用应用密码）".to_string(),
            "系统会自动测试连接是否成功".to_string(),
            "使用 'ccr sync push' 上传或 'ccr sync pull' 下载配置".to_string(),
            "支持强制模式：'ccr sync push --force' 或 'ccr sync pull --force'".to_string(),
        ],
        security_notes: vec![
            "密码存储在本地独立配置文件中：~/.ccr/sync.toml（推荐权限：chmod 600）".to_string(),
            "强烈建议使用应用密码而非账户密码（坚果云：账户设置 → 安全选项 → 添加应用）"
                .to_string(),
            "配置隔离：sync 配置独立保存，不与 CLI profiles 配置混在一起".to_string(),
            "同步内容：~/.ccr/ 目录（包含 config.toml, profiles.toml 等）".to_string(),
            "自动排除：backups/、history/、ccr-ui/、.locks/、.git/ 等目录".to_string(),
            "自动排除：*.tmp、*.lock、*.bak 等临时文件".to_string(),
            "远程文件未加密，依赖 WebDAV 服务器的安全性（建议使用 HTTPS）".to_string(),
        ],
    };

    ApiResponse::success(info)
}

// ============================================================================
// 📁 Sync Folder Management API Handlers (Multi-folder sync v2.5+)
// ============================================================================

/// GET /api/sync/folders - List all sync folders
pub async fn list_sync_folders() -> impl IntoResponse {
    let args = vec!["sync".to_string(), "folder".to_string(), "list".to_string()];

    match executor::execute_command(args).await {
        Ok(output) => {
            if output.success {
                // Parse the folder list from CLI output
                // For now, we'll execute the command and let the CLI format the output
                // TODO: Parse structured output when available
                ApiResponse::success(SyncOperationResponse {
                    success: true,
                    output: output.stdout,
                    error: String::new(),
                    duration_ms: output.duration_ms,
                })
            } else {
                ApiResponse::<SyncOperationResponse>::error(output.stderr)
            }
        }
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

/// POST /api/sync/folders - Add a new sync folder
pub async fn add_sync_folder(Json(req): Json<AddSyncFolderRequest>) -> impl IntoResponse {
    let mut args = vec![
        "sync".to_string(),
        "folder".to_string(),
        "add".to_string(),
        req.name.clone(),
        req.local_path.clone(),
    ];

    if let Some(remote_path) = &req.remote_path {
        args.push("-r".to_string());
        args.push(remote_path.clone());
    }

    if let Some(description) = &req.description {
        args.push("-d".to_string());
        args.push(description.clone());
    }

    match executor::execute_command(args).await {
        Ok(output) => {
            if output.success {
                ApiResponse::success(SyncFolderOperationResponse {
                    success: true,
                    message: format!("Successfully added sync folder: {}", req.name),
                })
            } else {
                ApiResponse::<SyncFolderOperationResponse>::error(output.stderr)
            }
        }
        Err(e) => ApiResponse::<SyncFolderOperationResponse>::error(e.to_string()),
    }
}

/// DELETE /api/sync/folders/:name - Remove a sync folder
pub async fn remove_sync_folder(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let args = vec![
        "sync".to_string(),
        "folder".to_string(),
        "remove".to_string(),
        name.clone(),
    ];

    match executor::execute_command(args).await {
        Ok(output) => {
            if output.success {
                ApiResponse::success(SyncFolderOperationResponse {
                    success: true,
                    message: format!("Successfully removed sync folder: {}", name),
                })
            } else {
                ApiResponse::<SyncFolderOperationResponse>::error(output.stderr)
            }
        }
        Err(e) => ApiResponse::<SyncFolderOperationResponse>::error(e.to_string()),
    }
}

/// GET /api/sync/folders/:name - Get folder information
pub async fn get_sync_folder_info(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let args = vec![
        "sync".to_string(),
        "folder".to_string(),
        "info".to_string(),
        name.clone(),
    ];

    match executor::execute_command(args).await {
        Ok(output) => {
            if output.success {
                ApiResponse::success(SyncOperationResponse {
                    success: true,
                    output: output.stdout,
                    error: String::new(),
                    duration_ms: output.duration_ms,
                })
            } else {
                ApiResponse::<SyncOperationResponse>::error(output.stderr)
            }
        }
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

/// PUT /api/sync/folders/:name/enable - Enable a sync folder
pub async fn enable_sync_folder(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let args = vec![
        "sync".to_string(),
        "folder".to_string(),
        "enable".to_string(),
        name.clone(),
    ];

    match executor::execute_command(args).await {
        Ok(output) => {
            if output.success {
                ApiResponse::success(SyncFolderOperationResponse {
                    success: true,
                    message: format!("Successfully enabled sync folder: {}", name),
                })
            } else {
                ApiResponse::<SyncFolderOperationResponse>::error(output.stderr)
            }
        }
        Err(e) => ApiResponse::<SyncFolderOperationResponse>::error(e.to_string()),
    }
}

/// PUT /api/sync/folders/:name/disable - Disable a sync folder
pub async fn disable_sync_folder(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let args = vec![
        "sync".to_string(),
        "folder".to_string(),
        "disable".to_string(),
        name.clone(),
    ];

    match executor::execute_command(args).await {
        Ok(output) => {
            if output.success {
                ApiResponse::success(SyncFolderOperationResponse {
                    success: true,
                    message: format!("Successfully disabled sync folder: {}", name),
                })
            } else {
                ApiResponse::<SyncFolderOperationResponse>::error(output.stderr)
            }
        }
        Err(e) => ApiResponse::<SyncFolderOperationResponse>::error(e.to_string()),
    }
}

/// POST /api/sync/folders/:name/push - Push a specific folder to cloud
pub async fn push_sync_folder(
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(req): Json<SyncOperationRequest>,
) -> impl IntoResponse {
    let mut args = vec!["sync".to_string(), name.clone(), "push".to_string()];

    if req.force {
        args.push("--force".to_string());
    }

    match executor::execute_command(args).await {
        Ok(output) => ApiResponse::success(SyncFolderSyncResponse {
            success: output.success,
            output: output.stdout,
            error: output.stderr,
            duration_ms: output.duration_ms,
        }),
        Err(e) => ApiResponse::<SyncFolderSyncResponse>::error(e.to_string()),
    }
}

/// POST /api/sync/folders/:name/pull - Pull a specific folder from cloud
pub async fn pull_sync_folder(
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(req): Json<SyncOperationRequest>,
) -> impl IntoResponse {
    let mut args = vec!["sync".to_string(), name.clone(), "pull".to_string()];

    if req.force {
        args.push("--force".to_string());
    }

    match executor::execute_command(args).await {
        Ok(output) => ApiResponse::success(SyncFolderSyncResponse {
            success: output.success,
            output: output.stdout,
            error: output.stderr,
            duration_ms: output.duration_ms,
        }),
        Err(e) => ApiResponse::<SyncFolderSyncResponse>::error(e.to_string()),
    }
}

/// GET /api/sync/folders/:name/status - Get status of a specific folder
pub async fn get_sync_folder_status(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let args = vec!["sync".to_string(), name.clone(), "status".to_string()];

    match executor::execute_command(args).await {
        Ok(output) => {
            if output.success {
                ApiResponse::success(SyncOperationResponse {
                    success: true,
                    output: output.stdout,
                    error: String::new(),
                    duration_ms: output.duration_ms,
                })
            } else {
                ApiResponse::<SyncOperationResponse>::error(output.stderr)
            }
        }
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

// ============================================================================
// 🔄 Batch Operations API Handlers
// ============================================================================

/// POST /api/sync/all/push - Push all enabled folders to cloud
pub async fn push_all_folders(Json(req): Json<SyncOperationRequest>) -> impl IntoResponse {
    let mut args = vec!["sync".to_string(), "all".to_string(), "push".to_string()];

    if req.force {
        args.push("--force".to_string());
    }

    match executor::execute_command(args).await {
        Ok(output) => ApiResponse::success(SyncOperationResponse {
            success: output.success,
            output: output.stdout,
            error: output.stderr,
            duration_ms: output.duration_ms,
        }),
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

/// POST /api/sync/all/pull - Pull all enabled folders from cloud
pub async fn pull_all_folders(Json(req): Json<SyncOperationRequest>) -> impl IntoResponse {
    let mut args = vec!["sync".to_string(), "all".to_string(), "pull".to_string()];

    if req.force {
        args.push("--force".to_string());
    }

    match executor::execute_command(args).await {
        Ok(output) => ApiResponse::success(SyncOperationResponse {
            success: output.success,
            output: output.stdout,
            error: output.stderr,
            duration_ms: output.duration_ms,
        }),
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

/// GET /api/sync/all/status - Get status of all folders
pub async fn get_all_folders_status() -> impl IntoResponse {
    let args = vec!["sync".to_string(), "all".to_string(), "status".to_string()];

    match executor::execute_command(args).await {
        Ok(output) => {
            if output.success {
                ApiResponse::success(SyncOperationResponse {
                    success: true,
                    output: output.stdout,
                    error: String::new(),
                    duration_ms: output.duration_ms,
                })
            } else {
                ApiResponse::<SyncOperationResponse>::error(output.stderr)
            }
        }
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}
