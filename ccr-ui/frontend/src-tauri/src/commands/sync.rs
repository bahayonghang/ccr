//! WebDAV 同步命令 — push / pull / status / folder CRUD。

use ccr::sync::{SyncConfigManager, SyncFolderManager, SyncService};
use serde::{Deserialize, Serialize};
use std::time::Instant;

// ── 响应类型 ──

/// 同步状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusInfo {
    pub configured: bool,
    pub enabled: bool,
    pub webdav_url: String,
    pub username: String,
    pub remote_path: String,
    pub auto_sync: bool,
    pub remote_accessible: Option<bool>,
    pub remote_exists: Option<bool>,
}

/// 同步文件夹信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFolderInfo {
    pub name: String,
    pub description: String,
    pub local_path: String,
    pub remote_path: String,
    pub enabled: bool,
    pub auto_sync: bool,
}

/// 同步操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperationResult {
    pub success: bool,
    pub message: String,
    pub duration_ms: u64,
}

// ── 命令实现 ──

#[tauri::command]
pub async fn sync_push(force: Option<bool>) -> Result<SyncOperationResult, String> {
    let start = Instant::now();
    let _ = force; // force 暂留给未来实现（覆盖冲突检测）

    // 1. 在 blocking 上下文中加载配置
    let config = tokio::task::spawn_blocking(|| {
        let manager = SyncConfigManager::with_default()
            .map_err(|e| format!("Failed to create SyncConfigManager: {e}"))?;
        manager
            .load()
            .map_err(|e| format!("Failed to load sync config: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    if !config.enabled {
        return Err("Sync is not enabled. Please configure sync first.".to_string());
    }

    // 2. 创建 SyncService（异步）
    let service = SyncService::new(&config)
        .await
        .map_err(|e| format!("Failed to create SyncService: {e}"))?;

    // 3. 获取本地路径
    let local_path = ccr::sync::service::get_ccr_sync_path()
        .map_err(|e| format!("Failed to get sync path: {e}"))?;

    // 4. 执行推送
    service
        .push(&local_path, None)
        .await
        .map_err(|e| format!("Push failed: {e}"))?;

    Ok(SyncOperationResult {
        success: true,
        message: format!("Successfully pushed config to {}", config.remote_path),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

#[tauri::command]
pub async fn sync_pull(force: Option<bool>) -> Result<SyncOperationResult, String> {
    let start = Instant::now();
    let _ = force;

    // 1. 加载配置
    let config = tokio::task::spawn_blocking(|| {
        let manager = SyncConfigManager::with_default()
            .map_err(|e| format!("Failed to create SyncConfigManager: {e}"))?;
        manager
            .load()
            .map_err(|e| format!("Failed to load sync config: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    if !config.enabled {
        return Err("Sync is not enabled. Please configure sync first.".to_string());
    }

    // 2. 创建 SyncService
    let service = SyncService::new(&config)
        .await
        .map_err(|e| format!("Failed to create SyncService: {e}"))?;

    // 3. 获取本地路径
    let local_path = ccr::sync::service::get_ccr_sync_path()
        .map_err(|e| format!("Failed to get sync path: {e}"))?;

    // 4. 执行拉取
    service
        .pull(&local_path)
        .await
        .map_err(|e| format!("Pull failed: {e}"))?;

    Ok(SyncOperationResult {
        success: true,
        message: format!("Successfully pulled config from {}", config.remote_path),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

#[tauri::command]
pub async fn sync_status() -> Result<SyncStatusInfo, String> {
    // 1. 加载配置
    let config = tokio::task::spawn_blocking(|| {
        let manager = SyncConfigManager::with_default()
            .map_err(|e| format!("Failed to create SyncConfigManager: {e}"))?;
        manager
            .load()
            .map_err(|e| format!("Failed to load sync config: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    let configured = config.enabled;

    // 2. 如果已配置，测试连接并检查远程路径
    let (remote_accessible, remote_exists) = if configured {
        match SyncService::new(&config).await {
            Ok(service) => {
                let accessible = service.test_connection().await.is_ok();
                let exists = if accessible {
                    service.remote_exists().await.ok()
                } else {
                    Some(false)
                };
                (Some(accessible), exists)
            }
            Err(_) => (Some(false), None),
        }
    } else {
        (None, None)
    };

    Ok(SyncStatusInfo {
        configured,
        enabled: config.enabled,
        webdav_url: config.webdav_url,
        username: config.username,
        remote_path: config.remote_path,
        auto_sync: config.auto_sync,
        remote_accessible,
        remote_exists,
    })
}

#[tauri::command]
pub async fn list_sync_folders() -> Result<Vec<SyncFolderInfo>, String> {
    let folders = tokio::task::spawn_blocking(|| {
        let manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;
        manager
            .list_folders()
            .map_err(|e| format!("Failed to list folders: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    let result = folders
        .into_iter()
        .map(|f| SyncFolderInfo {
            name: f.name,
            description: f.description,
            local_path: f.local_path,
            remote_path: f.remote_path,
            enabled: f.enabled,
            auto_sync: f.auto_sync,
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub async fn add_sync_folder(
    name: String,
    local_path: String,
    remote_path: String,
) -> Result<SyncFolderInfo, String> {
    let name_clone = name.clone();
    let local_path_clone = local_path.clone();
    let remote_path_clone = remote_path.clone();

    let folder = tokio::task::spawn_blocking(move || {
        let mut manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;

        let folder = ccr::sync::folder::SyncFolder::builder()
            .name(name_clone.clone())
            .local_path(local_path_clone)
            .remote_path(remote_path_clone)
            .enabled(true)
            .build()
            .map_err(|e| format!("Invalid folder config: {e}"))?;

        manager
            .add_folder(folder.clone())
            .map_err(|e| format!("Failed to add folder: {e}"))?;

        Ok::<_, String>(folder)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(SyncFolderInfo {
        name: folder.name,
        description: folder.description,
        local_path: folder.local_path,
        remote_path: folder.remote_path,
        enabled: folder.enabled,
        auto_sync: folder.auto_sync,
    })
}

#[tauri::command]
pub async fn update_sync_folder(
    id: String,
    name: Option<String>,
    enabled: Option<bool>,
) -> Result<SyncFolderInfo, String> {
    // `id` is the folder name in the SyncFolderManager API.
    // `name` param is unused (renaming not supported by manager API).
    let _ = name;

    let folder_name = id.clone();
    let updated = tokio::task::spawn_blocking(move || {
        let mut manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;

        // Apply enable/disable if requested
        if let Some(should_enable) = enabled {
            if should_enable {
                manager
                    .enable_folder(&folder_name)
                    .map_err(|e| format!("Failed to enable folder: {e}"))?;
            } else {
                manager
                    .disable_folder(&folder_name)
                    .map_err(|e| format!("Failed to disable folder: {e}"))?;
            }
        }

        // Read back updated folder
        manager
            .get_folder(&folder_name)
            .map_err(|e| format!("Folder not found after update: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(SyncFolderInfo {
        name: updated.name,
        description: updated.description,
        local_path: updated.local_path,
        remote_path: updated.remote_path,
        enabled: updated.enabled,
        auto_sync: updated.auto_sync,
    })
}

#[tauri::command]
pub async fn delete_sync_folder(id: String) -> Result<SyncOperationResult, String> {
    let start = Instant::now();
    let folder_name = id.clone();

    tokio::task::spawn_blocking(move || {
        let mut manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;
        manager
            .remove_folder(&folder_name)
            .map_err(|e| format!("Failed to remove folder: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(SyncOperationResult {
        success: true,
        message: format!("Successfully deleted sync folder: {id}"),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}
