// Sync Handlers - WebDAV Configuration Synchronization
// Execute sync commands and manage sync configuration

use crate::models::api::*;
use axum::{Json, response::IntoResponse};
use ccr::sync::{SyncConfig, SyncConfigManager, SyncFolderManager, SyncService};
use std::time::Instant;

/// POST /api/sync/config - Interactive sync configuration (not supported in web API)
pub async fn configure_sync() -> impl IntoResponse {
    ApiResponse::<String>::error(
        "Interactive sync configuration is not supported via web API. Please use CLI: ccr sync config".to_string()
    )
}

/// GET /api/sync/status - Get sync status and configuration
pub async fn get_sync_status() -> impl IntoResponse {
    let _start = Instant::now();
    let result = tokio::task::spawn_blocking(move || {
        let manager = SyncConfigManager::with_default()
            .map_err(|e| format!("Failed to create SyncConfigManager: {}", e))?;

        let config = manager
            .load()
            .map_err(|e| format!("Failed to load sync config: {}", e))?;

        // 检查配置是否已启用
        if !config.enabled {
            return Ok::<_, String>((false, config, None::<SyncConfigDetails>));
        }

        Ok((true, config, None))
    })
    .await;

    match result {
        Ok(Ok((configured, config, _))) => {
            // 在异步上下文中检查远程连接
            let remote_exists = if configured {
                match SyncService::new(&config).await {
                    Ok(service) => match service.test_connection().await {
                        Ok(_) => Some(true),
                        Err(_) => Some(false),
                    },
                    Err(_) => Some(false),
                }
            } else {
                None
            };

            // 构建输出字符串（模拟 CLI 输出）
            let output = format!(
                "状态: {}\nWebDAV 服务器: {}\n用户名: {}\n远程路径: {}\n自动同步: {}\n远程配置文件存在: {}",
                if configured {
                    "✓ 已启用"
                } else {
                    "✗ 未启用"
                },
                config.webdav_url,
                config.username,
                config.remote_path,
                if config.auto_sync {
                    "✓ 开启"
                } else {
                    "✗ 关闭"
                },
                match remote_exists {
                    Some(true) => "✓ 存在",
                    Some(false) => "✗ 不存在或无法访问",
                    None => "未知",
                }
            );

            let details = if configured {
                Some(SyncConfigDetails {
                    enabled: config.enabled,
                    webdav_url: config.webdav_url.clone(),
                    username: config.username.clone(),
                    remote_path: config.remote_path.clone(),
                    auto_sync: config.auto_sync,
                    remote_file_exists: remote_exists,
                })
            } else {
                None
            };

            let response = SyncStatusResponse {
                success: true,
                output,
                configured,
                config: details,
            };
            ApiResponse::success(response)
        }
        Ok(Err(e)) => ApiResponse::<SyncStatusResponse>::error(e),
        Err(e) => ApiResponse::<SyncStatusResponse>::error(e.to_string()),
    }
}

/// POST /api/sync/push - Upload config to cloud
pub async fn push_config(Json(_req): Json<SyncOperationRequest>) -> impl IntoResponse {
    let start = Instant::now();

    // 1. 获取配置
    let manager = match SyncConfigManager::with_default() {
        Ok(m) => m,
        Err(e) => return ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    };

    let config = match manager.load() {
        Ok(c) => c,
        Err(e) => return ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    };

    if !config.enabled {
        return ApiResponse::<SyncOperationResponse>::error("Sync is disabled".to_string());
    }

    // 2. 创建服务
    let service = match SyncService::new(&config).await {
        Ok(s) => s,
        Err(e) => return ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    };

    // 3. 获取本地路径
    let local_path = match ccr::sync::service::get_ccr_sync_path() {
        Ok(p) => p,
        Err(e) => return ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    };

    // 4. 执行上传
    // 使用 None 作为 allowed_paths 表示允许所有（或者应该使用 excludes？）
    // SyncService::push 内部会处理目录上传
    match service.push(&local_path, None).await {
        Ok(_) => {
            let duration = start.elapsed().as_millis() as u64;
            let response = SyncOperationResponse {
                success: true,
                output: format!("Successfully pushed config to {}", config.remote_path),
                error: String::new(),
                duration_ms: duration,
            };
            ApiResponse::success(response)
        }
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

/// POST /api/sync/pull - Download config from cloud
pub async fn pull_config(Json(_req): Json<SyncOperationRequest>) -> impl IntoResponse {
    let start = Instant::now();

    // 1. 获取配置
    let manager = match SyncConfigManager::with_default() {
        Ok(m) => m,
        Err(e) => return ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    };

    let config = match manager.load() {
        Ok(c) => c,
        Err(e) => return ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    };

    if !config.enabled {
        return ApiResponse::<SyncOperationResponse>::error("Sync is disabled".to_string());
    }

    // 2. 创建服务
    let service = match SyncService::new(&config).await {
        Ok(s) => s,
        Err(e) => return ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    };

    // 3. 获取本地路径
    let local_path = match ccr::sync::service::get_ccr_sync_path() {
        Ok(p) => p,
        Err(e) => return ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    };

    // 4. 执行下载
    match service.pull(&local_path).await {
        Ok(_) => {
            let duration = start.elapsed().as_millis() as u64;
            let response = SyncOperationResponse {
                success: true,
                output: format!("Successfully pulled config from {}", config.remote_path),
                error: String::new(),
                duration_ms: duration,
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
    let start = Instant::now();
    let result = tokio::task::spawn_blocking(move || {
        let manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {}", e))?;
        let folders = manager
            .list_folders()
            .map_err(|e| format!("Failed to list folders: {}", e))?;
        Ok::<_, String>(folders)
    })
    .await;

    match result {
        Ok(Ok(folders)) => {
            // 格式化输出
            let mut output = String::new();
            for folder in &folders {
                output.push_str(&format!(
                    "{} ({}) - {}\n  Local: {}\n  Remote: {}\n\n",
                    folder.name,
                    if folder.enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    },
                    folder.description,
                    folder.local_path,
                    folder.remote_path
                ));
            }

            let response = SyncOperationResponse {
                success: true,
                output,
                error: String::new(),
                duration_ms: start.elapsed().as_millis() as u64,
            };
            ApiResponse::success(response)
        }
        Ok(Err(e)) => ApiResponse::<SyncOperationResponse>::error(e),
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

/// POST /api/sync/folders - Add a new sync folder
pub async fn add_sync_folder(Json(req): Json<AddSyncFolderRequest>) -> impl IntoResponse {
    let folder_name = req.name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {}", e))?;

        // 构建 SyncFolder
        let folder = ccr::sync::folder::SyncFolder::builder()
            .name(req.name.clone())
            .local_path(req.local_path)
            .remote_path(
                req.remote_path
                    .unwrap_or_else(|| format!("/ccr-sync/{}", req.name)),
            )
            .description(req.description.unwrap_or_default())
            .enabled(true)
            .build()
            .map_err(|e| format!("Invalid folder config: {}", e))?;

        manager
            .add_folder(folder)
            .map_err(|e| format!("Failed to add folder: {}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => ApiResponse::success(SyncFolderOperationResponse {
            success: true,
            message: format!("Successfully added sync folder: {}", folder_name),
        }),
        Ok(Err(e)) => ApiResponse::<SyncFolderOperationResponse>::error(e),
        Err(e) => ApiResponse::<SyncFolderOperationResponse>::error(e.to_string()),
    }
}

/// DELETE /api/sync/folders/:name - Remove a sync folder
pub async fn remove_sync_folder(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let name_clone = name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {}", e))?;
        manager
            .remove_folder(&name_clone)
            .map_err(|e| format!("Failed to remove folder: {}", e))?;
        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => ApiResponse::success(SyncFolderOperationResponse {
            success: true,
            message: format!("Successfully removed sync folder: {}", name),
        }),
        Ok(Err(e)) => ApiResponse::<SyncFolderOperationResponse>::error(e),
        Err(e) => ApiResponse::<SyncFolderOperationResponse>::error(e.to_string()),
    }
}

/// GET /api/sync/folders/:name - Get folder information
pub async fn get_sync_folder_info(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let start = Instant::now();
    let result = tokio::task::spawn_blocking(move || {
        let manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {}", e))?;
        let folder = manager
            .get_folder(&name)
            .map_err(|e| format!("Folder not found: {}", e))?;
        Ok::<_, String>(folder)
    })
    .await;

    match result {
        Ok(Ok(folder)) => {
            let output = format!(
                "Name: {}\nDescription: {}\nLocal Path: {}\nRemote Path: {}\nEnabled: {}\nAuto Sync: {}\nExclude Patterns: {:?}",
                folder.name,
                folder.description,
                folder.local_path,
                folder.remote_path,
                folder.enabled,
                folder.auto_sync,
                folder.exclude_patterns
            );

            ApiResponse::success(SyncOperationResponse {
                success: true,
                output,
                error: String::new(),
                duration_ms: start.elapsed().as_millis() as u64,
            })
        }
        Ok(Err(e)) => ApiResponse::<SyncOperationResponse>::error(e),
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

/// PUT /api/sync/folders/:name/enable - Enable a sync folder
pub async fn enable_sync_folder(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let name_clone = name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {}", e))?;
        manager
            .enable_folder(&name_clone)
            .map_err(|e| format!("Failed to enable folder: {}", e))?;
        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => ApiResponse::success(SyncFolderOperationResponse {
            success: true,
            message: format!("Successfully enabled sync folder: {}", name),
        }),
        Ok(Err(e)) => ApiResponse::<SyncFolderOperationResponse>::error(e),
        Err(e) => ApiResponse::<SyncFolderOperationResponse>::error(e.to_string()),
    }
}

/// PUT /api/sync/folders/:name/disable - Disable a sync folder
pub async fn disable_sync_folder(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let name_clone = name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {}", e))?;
        manager
            .disable_folder(&name_clone)
            .map_err(|e| format!("Failed to disable folder: {}", e))?;
        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => ApiResponse::success(SyncFolderOperationResponse {
            success: true,
            message: format!("Successfully disabled sync folder: {}", name),
        }),
        Ok(Err(e)) => ApiResponse::<SyncFolderOperationResponse>::error(e),
        Err(e) => ApiResponse::<SyncFolderOperationResponse>::error(e.to_string()),
    }
}

/// POST /api/sync/folders/:name/push - Push a specific folder to cloud
pub async fn push_sync_folder(
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(_req): Json<SyncOperationRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    // 1. 获取文件夹配置
    let folder_info = tokio::task::spawn_blocking(move || {
        let manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {}", e))?;
        let config = manager
            .load_config()
            .map_err(|e| format!("Failed to load config: {}", e))?;
        let folder = config
            .find_folder(&name)
            .cloned()
            .ok_or_else(|| format!("Folder '{}' not found", name))?;

        // 构造 SyncConfig 用于创建 Service
        let sync_config = SyncConfig {
            enabled: folder.enabled,
            webdav_url: config.webdav.url.clone(),
            username: config.webdav.username.clone(),
            password: config.webdav.password.clone(),
            remote_path: folder.remote_path.clone(), // 使用文件夹的远程路径
            auto_sync: folder.auto_sync,
        };

        Ok::<_, String>((folder, sync_config))
    })
    .await;

    match folder_info {
        Ok(Ok((folder, sync_config))) => {
            if !folder.enabled {
                return ApiResponse::<SyncFolderSyncResponse>::error(
                    "Folder is disabled".to_string(),
                );
            }

            match SyncService::new(&sync_config).await {
                Ok(service) => {
                    let local_path = match folder.expand_local_path() {
                        Ok(p) => p,
                        Err(e) => {
                            return ApiResponse::<SyncFolderSyncResponse>::error(e.to_string());
                        }
                    };

                    match service.push(&local_path, None).await {
                        Ok(_) => ApiResponse::success(SyncFolderSyncResponse {
                            success: true,
                            output: format!("Successfully pushed folder '{}'", folder.name),
                            error: String::new(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }),
                        Err(e) => ApiResponse::<SyncFolderSyncResponse>::error(e.to_string()),
                    }
                }
                Err(e) => ApiResponse::<SyncFolderSyncResponse>::error(e.to_string()),
            }
        }
        Ok(Err(e)) => ApiResponse::<SyncFolderSyncResponse>::error(e),
        Err(e) => ApiResponse::<SyncFolderSyncResponse>::error(e.to_string()),
    }
}

/// POST /api/sync/folders/:name/pull - Pull a specific folder from cloud
pub async fn pull_sync_folder(
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(_req): Json<SyncOperationRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    // 1. 获取文件夹配置
    let folder_info = tokio::task::spawn_blocking(move || {
        let manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {}", e))?;
        let config = manager
            .load_config()
            .map_err(|e| format!("Failed to load config: {}", e))?;
        let folder = config
            .find_folder(&name)
            .cloned()
            .ok_or_else(|| format!("Folder '{}' not found", name))?;

        let sync_config = SyncConfig {
            enabled: folder.enabled,
            webdav_url: config.webdav.url.clone(),
            username: config.webdav.username.clone(),
            password: config.webdav.password.clone(),
            remote_path: folder.remote_path.clone(),
            auto_sync: folder.auto_sync,
        };

        Ok::<_, String>((folder, sync_config))
    })
    .await;

    match folder_info {
        Ok(Ok((folder, sync_config))) => {
            if !folder.enabled {
                return ApiResponse::<SyncFolderSyncResponse>::error(
                    "Folder is disabled".to_string(),
                );
            }

            match SyncService::new(&sync_config).await {
                Ok(service) => {
                    let local_path = match folder.expand_local_path() {
                        Ok(p) => p,
                        Err(e) => {
                            return ApiResponse::<SyncFolderSyncResponse>::error(e.to_string());
                        }
                    };

                    match service.pull(&local_path).await {
                        Ok(_) => ApiResponse::success(SyncFolderSyncResponse {
                            success: true,
                            output: format!("Successfully pulled folder '{}'", folder.name),
                            error: String::new(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }),
                        Err(e) => ApiResponse::<SyncFolderSyncResponse>::error(e.to_string()),
                    }
                }
                Err(e) => ApiResponse::<SyncFolderSyncResponse>::error(e.to_string()),
            }
        }
        Ok(Err(e)) => ApiResponse::<SyncFolderSyncResponse>::error(e),
        Err(e) => ApiResponse::<SyncFolderSyncResponse>::error(e.to_string()),
    }
}

/// GET /api/sync/folders/:name/status - Get status of a specific folder
pub async fn get_sync_folder_status(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let start = Instant::now();
    let folder_info = tokio::task::spawn_blocking(move || {
        let manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {}", e))?;
        let config = manager
            .load_config()
            .map_err(|e| format!("Failed to load config: {}", e))?;
        let folder = config
            .find_folder(&name)
            .cloned()
            .ok_or_else(|| format!("Folder '{}' not found", name))?;

        let sync_config = SyncConfig {
            enabled: folder.enabled,
            webdav_url: config.webdav.url.clone(),
            username: config.webdav.username.clone(),
            password: config.webdav.password.clone(),
            remote_path: folder.remote_path.clone(),
            auto_sync: folder.auto_sync,
        };

        Ok::<_, String>((folder, sync_config))
    })
    .await;

    match folder_info {
        Ok(Ok((folder, sync_config))) => match SyncService::new(&sync_config).await {
            Ok(service) => match service.test_connection().await {
                Ok(_) => match service.remote_exists().await {
                    Ok(exists) => {
                        let output = format!(
                            "Folder: {}\nRemote: {}\nConnection: OK\nRemote Exists: {}",
                            folder.name,
                            folder.remote_path,
                            if exists { "Yes" } else { "No" }
                        );
                        ApiResponse::success(SyncOperationResponse {
                            success: true,
                            output,
                            error: String::new(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        })
                    }
                    Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
                },
                Err(e) => {
                    ApiResponse::<SyncOperationResponse>::error(format!("Connection failed: {}", e))
                }
            },
            Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
        },
        Ok(Err(e)) => ApiResponse::<SyncOperationResponse>::error(e),
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

// ============================================================================
// 🔄 Batch Operations API Handlers
// ============================================================================

/// POST /api/sync/all/push - Push all enabled folders to cloud
pub async fn push_all_folders(Json(_req): Json<SyncOperationRequest>) -> impl IntoResponse {
    let start = Instant::now();
    // Implementation omitted for brevity - would iterate folders and push each
    // For now returning mock success or TODO
    // Note: Implementing parsing of all folders structure and async iteration

    // For simplicity, let's just return a message saying it's not fully implemented yet in this refactor
    // Or we could implement it properly:
    // 1. Load config
    // 2. Iterate enabled folders
    // 3. Push each (serially or parallel)

    let result = tokio::task::spawn_blocking(move || {
        let manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {}", e))?;
        let config = manager
            .load_config()
            .map_err(|e| format!("Failed to load config: {}", e))?;
        Ok::<_, String>(config)
    })
    .await;

    match result {
        Ok(Ok(config)) => {
            let mut output = String::new();
            // let mut success_count = 0;
            let mut fail_count = 0;

            for folder in config.enabled_folders() {
                let sync_config = SyncConfig {
                    enabled: true,
                    webdav_url: config.webdav.url.clone(),
                    username: config.webdav.username.clone(),
                    password: config.webdav.password.clone(),
                    remote_path: folder.remote_path.clone(),
                    auto_sync: folder.auto_sync,
                };

                if let Ok(service) = SyncService::new(&sync_config).await {
                    if let Ok(local_path) = folder.expand_local_path() {
                        if service.push(&local_path, None).await.is_ok() {
                            output.push_str(&format!("✓ Pushed {}\n", folder.name));
                            // success_count += 1;
                        } else {
                            output.push_str(&format!("✗ Failed to push {}\n", folder.name));
                            fail_count += 1;
                        }
                    }
                } else {
                    output.push_str(&format!("✗ Failed to connect for {}\n", folder.name));
                    fail_count += 1;
                }
            }

            let response = SyncOperationResponse {
                success: fail_count == 0,
                output,
                error: if fail_count > 0 {
                    format!("{} folders failed", fail_count)
                } else {
                    String::new()
                },
                duration_ms: start.elapsed().as_millis() as u64,
            };
            ApiResponse::success(response)
        }
        Ok(Err(e)) => ApiResponse::<SyncOperationResponse>::error(e),
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

/// POST /api/sync/all/pull - Pull all enabled folders from cloud
/// Same logic as push_all_folders but with pull
pub async fn pull_all_folders(Json(_req): Json<SyncOperationRequest>) -> impl IntoResponse {
    let start = Instant::now();
    let result = tokio::task::spawn_blocking(move || {
        let manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {}", e))?;
        let config = manager
            .load_config()
            .map_err(|e| format!("Failed to load config: {}", e))?;
        Ok::<_, String>(config)
    })
    .await;

    match result {
        Ok(Ok(config)) => {
            let mut output = String::new();
            // let mut success_count = 0;
            let mut fail_count = 0;

            for folder in config.enabled_folders() {
                let sync_config = SyncConfig {
                    enabled: true,
                    webdav_url: config.webdav.url.clone(),
                    username: config.webdav.username.clone(),
                    password: config.webdav.password.clone(),
                    remote_path: folder.remote_path.clone(),
                    auto_sync: folder.auto_sync,
                };

                if let Ok(service) = SyncService::new(&sync_config).await {
                    if let Ok(local_path) = folder.expand_local_path() {
                        if service.pull(&local_path).await.is_ok() {
                            output.push_str(&format!("✓ Pulled {}\n", folder.name));
                            // success_count += 1;
                        } else {
                            output.push_str(&format!("✗ Failed to pull {}\n", folder.name));
                            fail_count += 1;
                        }
                    }
                } else {
                    output.push_str(&format!("✗ Failed to connect for {}\n", folder.name));
                    fail_count += 1;
                }
            }

            let response = SyncOperationResponse {
                success: fail_count == 0,
                output,
                error: if fail_count > 0 {
                    format!("{} folders failed", fail_count)
                } else {
                    String::new()
                },
                duration_ms: start.elapsed().as_millis() as u64,
            };
            ApiResponse::success(response)
        }
        Ok(Err(e)) => ApiResponse::<SyncOperationResponse>::error(e),
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}

/// GET /api/sync/all/status - Get status of all folders
pub async fn get_all_folders_status() -> impl IntoResponse {
    let start = Instant::now();
    // Simplified status check
    let result = tokio::task::spawn_blocking(move || {
        let manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {}", e))?;
        let stats = manager
            .stats()
            .map_err(|e| format!("Failed to get stats: {}", e))?;
        Ok::<_, String>(stats)
    })
    .await;

    match result {
        Ok(Ok(stats)) => {
            let output = format!(
                "Total Folders: {}\nEnabled: {}\nDisabled: {}",
                stats.total, stats.enabled, stats.disabled
            );

            ApiResponse::success(SyncOperationResponse {
                success: true,
                output,
                error: String::new(),
                duration_ms: start.elapsed().as_millis() as u64,
            })
        }
        Ok(Err(e)) => ApiResponse::<SyncOperationResponse>::error(e),
        Err(e) => ApiResponse::<SyncOperationResponse>::error(e.to_string()),
    }
}
