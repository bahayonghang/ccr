//! WebDAV 同步命令 — push / pull / status / folder CRUD / 账号增删测。

use ccr_sync::{
    SyncConfig, SyncConfigManager, SyncFolder, SyncFolderManager, SyncService, WebDavConfig,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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
    /// 是否已保存密码（永不下发明文密码）
    pub has_password: bool,
    pub remote_accessible: Option<bool>,
    pub remote_exists: Option<bool>,
}

/// 账号写入入参（前端 camelCase）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebDavConfigInput {
    pub webdav_url: String,
    pub username: String,
    pub password: String,
    pub remote_path: Option<String>,
    pub auto_sync: Option<bool>,
}

/// 账号详情（不含密码，前端 camelCase）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebDavConfigDetails {
    pub enabled: bool,
    pub webdav_url: String,
    pub username: String,
    pub remote_path: String,
    pub auto_sync: bool,
    pub has_password: bool,
}

/// 连接测试结果（前端 camelCase）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebDavTestResult {
    pub ok: bool,
    pub message: String,
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

/// 同步操作失败项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncOperationFailure {
    pub folder: String,
    pub message: String,
}

/// 同步操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncOperationResult {
    pub success: bool,
    pub message: String,
    pub duration_ms: u64,
    pub total: usize,
    pub success_count: usize,
    pub failed: Vec<SyncOperationFailure>,
}

fn operation_result(
    success: bool,
    message: impl Into<String>,
    duration_ms: u64,
    total: usize,
    success_count: usize,
    failed: Vec<SyncOperationFailure>,
) -> SyncOperationResult {
    SyncOperationResult {
        success,
        message: message.into(),
        duration_ms,
        total,
        success_count,
        failed,
    }
}

fn folder_info(folder: SyncFolder) -> SyncFolderInfo {
    SyncFolderInfo {
        name: folder.name,
        description: folder.description,
        local_path: folder.local_path,
        remote_path: folder.remote_path,
        enabled: folder.enabled,
        auto_sync: folder.auto_sync,
    }
}

fn normalize_remote_base(base: &str) -> String {
    let trimmed = base.trim();
    if trimmed.is_empty() {
        return "/ccr".to_string();
    }

    let with_leading = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    };

    with_leading.trim_end_matches('/').to_string()
}

fn default_remote_segment(name: &str) -> &str {
    if name == "config" { "platforms" } else { name }
}

fn resolve_remote_path(name: &str, remote_path: &str, webdav: &WebDavConfig) -> String {
    let trimmed = remote_path.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }

    format!(
        "{}/{}",
        normalize_remote_base(&webdav.base_remote_path),
        default_remote_segment(name)
    )
}

fn webdav_from_sync_config(config: &SyncConfig) -> WebDavConfig {
    WebDavConfig {
        url: config.webdav_url.clone(),
        username: config.username.clone(),
        password: config.password.clone(),
        base_remote_path: normalize_remote_base(&config.remote_path),
    }
}

fn sync_config_from_webdav(webdav: &WebDavConfig, remote_path: &str) -> SyncConfig {
    SyncConfig {
        enabled: true,
        webdav_url: webdav.url.clone(),
        username: webdav.username.clone(),
        password: webdav.password.clone(),
        remote_path: remote_path.to_string(),
        auto_sync: false,
    }
}

fn sync_config_for_folder(
    manager: &SyncFolderManager,
    folder: &SyncFolder,
) -> Result<SyncConfig, String> {
    let webdav = manager
        .get_webdav_config()
        .map_err(|e| format!("Failed to load WebDAV config: {e}"))?;

    if webdav.url.trim().is_empty()
        || webdav.username.trim().is_empty()
        || webdav.password.trim().is_empty()
    {
        return Err("WebDAV account is incomplete. Please configure WebDAV first.".to_string());
    }

    Ok(sync_config_from_webdav(&webdav, &folder.remote_path))
}

fn save_webdav_to_managers(
    sync_manager: &SyncConfigManager,
    folder_manager: &mut SyncFolderManager,
    payload: WebDavConfigInput,
) -> Result<SyncConfig, String> {
    let mut config = sync_manager.load().unwrap_or_default();
    let new_config = build_sync_config(payload);
    config.enabled = true;
    config.webdav_url = new_config.webdav_url;
    config.username = new_config.username;
    config.password = new_config.password;
    config.remote_path = normalize_remote_base(&new_config.remote_path);
    config.auto_sync = new_config.auto_sync;

    sync_manager
        .save(&config)
        .map_err(|e| format!("Failed to save sync config: {e}"))?;

    folder_manager
        .update_webdav_config(webdav_from_sync_config(&config))
        .map_err(|e| format!("Failed to save sync folders WebDAV config: {e}"))?;

    Ok(config)
}

fn add_folder_with_manager(
    manager: &mut SyncFolderManager,
    name: String,
    local_path: String,
    remote_path: String,
    description: Option<String>,
) -> Result<SyncFolder, String> {
    let webdav_config = manager
        .get_webdav_config()
        .map_err(|e| format!("Failed to load WebDAV config: {e}"))?;
    let final_remote_path = resolve_remote_path(&name, &remote_path, &webdav_config);
    let final_description = description.unwrap_or_else(|| format!("{} folder", name));

    let folder = SyncFolder::builder()
        .name(name.clone())
        .description(final_description)
        .local_path(local_path)
        .remote_path(final_remote_path)
        .enabled(true)
        .build()
        .map_err(|e| format!("Invalid folder config: {e}"))?;

    manager
        .add_folder(folder.clone())
        .map_err(|e| format!("Failed to add folder: {e}"))?;

    Ok(folder)
}

fn update_folder_with_manager(
    manager: &mut SyncFolderManager,
    id: String,
    name: Option<String>,
    enabled: Option<bool>,
    local_path: Option<String>,
    remote_path: Option<String>,
    description: Option<String>,
) -> Result<SyncFolder, String> {
    let mut folder = manager
        .get_folder(&id)
        .map_err(|e| format!("Folder not found: {e}"))?;
    let webdav_config = manager
        .get_webdav_config()
        .map_err(|e| format!("Failed to load WebDAV config: {e}"))?;

    if let Some(name) = name.filter(|value| !value.trim().is_empty()) {
        folder.name = name;
    }
    if let Some(enabled) = enabled {
        folder.enabled = enabled;
    }
    if let Some(local_path) = local_path.filter(|value| !value.trim().is_empty()) {
        folder.local_path = local_path;
    }
    if let Some(remote_path) = remote_path {
        folder.remote_path = resolve_remote_path(&folder.name, &remote_path, &webdav_config);
    }
    if let Some(description) = description {
        folder.description = description;
    }

    manager
        .update_folder(&id, folder.clone())
        .map_err(|e| format!("Failed to update folder: {e}"))?;

    Ok(folder)
}

async fn push_folder_config(
    manager: &SyncFolderManager,
    folder: &SyncFolder,
    force: bool,
) -> Result<(), String> {
    if !folder.enabled {
        return Err("Folder is disabled. Enable it before syncing.".to_string());
    }

    let local_path = folder
        .expand_local_path()
        .map_err(|e| format!("Failed to expand local path: {e}"))?;

    if !local_path.exists() {
        return Err(format!(
            "Local path does not exist: {}",
            local_path.display()
        ));
    }

    let sync_config = sync_config_for_folder(manager, folder)?;
    let service = SyncService::new(&sync_config)
        .await
        .map_err(|e| format!("Failed to create SyncService: {e}"))?;

    if !force {
        let exists = service
            .remote_exists()
            .await
            .map_err(|e| format!("Failed to check remote path: {e}"))?;
        if exists {
            return Err(
                "Remote content already exists; rerun with force to overwrite.".to_string(),
            );
        }
    }

    service
        .push(&local_path, None)
        .await
        .map_err(|e| format!("Push failed: {e}"))
}

async fn pull_folder_config(
    manager: &SyncFolderManager,
    folder: &SyncFolder,
    force: bool,
) -> Result<(), String> {
    if !folder.enabled {
        return Err("Folder is disabled. Enable it before syncing.".to_string());
    }

    let local_path = folder
        .expand_local_path()
        .map_err(|e| format!("Failed to expand local path: {e}"))?;

    let sync_config = sync_config_for_folder(manager, folder)?;
    let service = SyncService::new(&sync_config)
        .await
        .map_err(|e| format!("Failed to create SyncService: {e}"))?;

    let remote_exists = service
        .remote_exists()
        .await
        .map_err(|e| format!("Failed to check remote path: {e}"))?;
    if !remote_exists {
        return Err("Remote content does not exist. Upload this folder first.".to_string());
    }

    let local_exists = tokio::fs::try_exists(&local_path)
        .await
        .map_err(|e| format!("Failed to check local path: {e}"))?;
    if local_exists && !force {
        return Err("Local content already exists; rerun with force to overwrite.".to_string());
    }

    if local_exists {
        let backup_path = backup_path_for(&local_path);
        if tokio::fs::try_exists(&backup_path).await.unwrap_or(false) {
            tokio::fs::remove_dir_all(&backup_path)
                .await
                .map_err(|e| format!("Failed to remove old backup: {e}"))?;
        }
        tokio::fs::rename(&local_path, &backup_path)
            .await
            .map_err(|e| format!("Failed to back up local content: {e}"))?;
    }

    service
        .pull(&local_path)
        .await
        .map_err(|e| format!("Pull failed: {e}"))
}

fn backup_path_for(local_path: &Path) -> PathBuf {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    PathBuf::from(format!("{}.{}.bak", local_path.display(), timestamp))
}

// ── 命令实现 ──

#[tauri::command]
pub async fn sync_push(force: Option<bool>) -> Result<SyncOperationResult, String> {
    sync_enabled_folders(SyncDirection::Push, force.unwrap_or(false)).await
}

#[tauri::command]
pub async fn sync_pull(force: Option<bool>) -> Result<SyncOperationResult, String> {
    sync_enabled_folders(SyncDirection::Pull, force.unwrap_or(false)).await
}

#[tauri::command]
pub async fn sync_push_folder(
    id: String,
    force: Option<bool>,
) -> Result<SyncOperationResult, String> {
    sync_one_folder(id, SyncDirection::Push, force.unwrap_or(false)).await
}

#[tauri::command]
pub async fn sync_pull_folder(
    id: String,
    force: Option<bool>,
) -> Result<SyncOperationResult, String> {
    sync_one_folder(id, SyncDirection::Pull, force.unwrap_or(false)).await
}

#[derive(Clone, Copy)]
enum SyncDirection {
    Push,
    Pull,
}

async fn sync_enabled_folders(
    direction: SyncDirection,
    force: bool,
) -> Result<SyncOperationResult, String> {
    let start = Instant::now();
    let manager = SyncFolderManager::with_default()
        .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;
    let folders = tokio::task::spawn_blocking({
        let manager = SyncFolderManager::new(manager.config_path());
        move || {
            manager
                .load_config()
                .map(|config| {
                    config
                        .folders
                        .into_iter()
                        .filter(|folder| folder.enabled)
                        .collect::<Vec<_>>()
                })
                .map_err(|e| format!("Failed to load sync folders: {e}"))
        }
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    if folders.is_empty() {
        return Ok(operation_result(
            false,
            "No enabled sync folders. Apply or enable a folder before syncing.",
            start.elapsed().as_millis() as u64,
            0,
            0,
            Vec::new(),
        ));
    }

    run_folder_syncs(&manager, folders, direction, force, start).await
}

async fn sync_one_folder(
    id: String,
    direction: SyncDirection,
    force: bool,
) -> Result<SyncOperationResult, String> {
    let start = Instant::now();
    let manager = SyncFolderManager::with_default()
        .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;
    let folder = tokio::task::spawn_blocking({
        let manager = SyncFolderManager::new(manager.config_path());
        let id = id.clone();
        move || {
            manager
                .get_folder(&id)
                .map_err(|e| format!("Failed to load sync folder: {e}"))
        }
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    run_folder_syncs(&manager, vec![folder], direction, force, start).await
}

async fn run_folder_syncs(
    manager: &SyncFolderManager,
    folders: Vec<SyncFolder>,
    direction: SyncDirection,
    force: bool,
    start: Instant,
) -> Result<SyncOperationResult, String> {
    let total = folders.len();
    let mut success_count = 0usize;
    let mut failed = Vec::new();

    for folder in folders {
        let result = match direction {
            SyncDirection::Push => push_folder_config(manager, &folder, force).await,
            SyncDirection::Pull => pull_folder_config(manager, &folder, force).await,
        };

        match result {
            Ok(()) => success_count += 1,
            Err(message) => failed.push(SyncOperationFailure {
                folder: folder.name,
                message,
            }),
        }
    }

    let label = match direction {
        SyncDirection::Push => "upload",
        SyncDirection::Pull => "download",
    };
    let success = failed.is_empty();
    let message = if success {
        format!("Completed {label} for {success_count}/{total} enabled folder(s).")
    } else {
        format!(
            "Completed {label} for {success_count}/{total} enabled folder(s); {} failed.",
            failed.len()
        )
    };

    Ok(operation_result(
        success,
        message,
        start.elapsed().as_millis() as u64,
        total,
        success_count,
        failed,
    ))
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
        has_password: !config.password.is_empty(),
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

    let result = folders.into_iter().map(folder_info).collect();

    Ok(result)
}

#[tauri::command]
pub async fn add_sync_folder(
    name: String,
    local_path: String,
    remote_path: String,
    description: Option<String>,
) -> Result<SyncFolderInfo, String> {
    let folder = tokio::task::spawn_blocking(move || {
        let mut manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;

        add_folder_with_manager(&mut manager, name, local_path, remote_path, description)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(folder_info(folder))
}

#[tauri::command]
pub async fn update_sync_folder(
    id: String,
    name: Option<String>,
    enabled: Option<bool>,
    local_path: Option<String>,
    remote_path: Option<String>,
    description: Option<String>,
) -> Result<SyncFolderInfo, String> {
    let updated = tokio::task::spawn_blocking(move || {
        let mut manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;

        update_folder_with_manager(
            &mut manager,
            id,
            name,
            enabled,
            local_path,
            remote_path,
            description,
        )
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(folder_info(updated))
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

    Ok(operation_result(
        true,
        format!("Successfully deleted sync folder: {id}"),
        start.elapsed().as_millis() as u64,
        1,
        1,
        Vec::new(),
    ))
}

// ── 账号管理（set / test / clear）──

/// 由 UI 表单构建 SyncConfig（不持久化）。
fn build_sync_config(payload: WebDavConfigInput) -> SyncConfig {
    SyncConfig {
        enabled: true,
        webdav_url: payload.webdav_url,
        username: payload.username,
        password: payload.password,
        remote_path: payload
            .remote_path
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "/ccr/".to_string()),
        auto_sync: payload.auto_sync.unwrap_or(false),
    }
}

/// 持久化 WebDAV 账号，保存即启用。
#[tauri::command]
pub async fn set_webdav_config(payload: WebDavConfigInput) -> Result<WebDavConfigDetails, String> {
    let saved = tokio::task::spawn_blocking(move || {
        let sync_manager = SyncConfigManager::with_default()
            .map_err(|e| format!("Failed to create SyncConfigManager: {e}"))?;
        let mut folder_manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;

        save_webdav_to_managers(&sync_manager, &mut folder_manager, payload)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(WebDavConfigDetails {
        enabled: saved.enabled,
        webdav_url: saved.webdav_url,
        username: saved.username,
        remote_path: saved.remote_path,
        auto_sync: saved.auto_sync,
        has_password: !saved.password.is_empty(),
    })
}

/// 测试 WebDAV 凭据（不持久化）。
/// 永远返回 Ok，UI 通过 ok / message 区分。
#[tauri::command]
pub async fn test_webdav_config(payload: WebDavConfigInput) -> Result<WebDavTestResult, String> {
    let config = build_sync_config(payload);

    let result = match SyncService::new(&config).await {
        Ok(service) => match service.test_connection().await {
            Ok(()) => WebDavTestResult {
                ok: true,
                message: "ok".to_string(),
            },
            Err(e) => WebDavTestResult {
                ok: false,
                message: format!("{e}"),
            },
        },
        Err(e) => WebDavTestResult {
            ok: false,
            message: format!("{e}"),
        },
    };

    Ok(result)
}

/// 断开账号：物理删除 ~/.ccr/sync.toml。
#[tauri::command]
pub async fn clear_webdav_config() -> Result<(), String> {
    tokio::task::spawn_blocking(|| {
        let manager = SyncConfigManager::with_default()
            .map_err(|e| format!("Failed to create SyncConfigManager: {e}"))?;
        manager
            .delete()
            .map_err(|e| format!("Failed to delete sync config: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_sync::{SyncConfigManager, SyncFolderManager};

    fn webdav_payload() -> WebDavConfigInput {
        WebDavConfigInput {
            webdav_url: "https://dav.example.com/".to_string(),
            username: "user@example.com".to_string(),
            password: "secret".to_string(),
            remote_path: Some("/ccr/".to_string()),
            auto_sync: Some(false),
        }
    }

    #[test]
    fn set_webdav_config_updates_legacy_and_folder_webdav_configs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sync_path = temp_dir.path().join("sync.toml");
        let folders_path = temp_dir.path().join("sync_folders.toml");
        let sync_manager = SyncConfigManager::new(&sync_path);
        let mut folder_manager = SyncFolderManager::new(&folders_path);

        let saved = save_webdav_to_managers(&sync_manager, &mut folder_manager, webdav_payload())
            .expect("webdav config should save to both config files");

        assert_eq!(saved.remote_path, "/ccr");

        let legacy = sync_manager.load().unwrap();
        assert!(legacy.enabled);
        assert_eq!(legacy.webdav_url, "https://dav.example.com/");
        assert_eq!(legacy.username, "user@example.com");
        assert_eq!(legacy.password, "secret");
        assert_eq!(legacy.remote_path, "/ccr");

        let folders_config = folder_manager.load_config().unwrap();
        assert_eq!(folders_config.webdav.url, "https://dav.example.com/");
        assert_eq!(folders_config.webdav.username, "user@example.com");
        assert_eq!(folders_config.webdav.password, "secret");
        assert_eq!(folders_config.webdav.base_remote_path, "/ccr");
    }

    #[test]
    fn add_sync_folder_uses_platforms_segment_for_default_config_folder() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sync_path = temp_dir.path().join("sync.toml");
        let folders_path = temp_dir.path().join("sync_folders.toml");
        let sync_manager = SyncConfigManager::new(&sync_path);
        let mut folder_manager = SyncFolderManager::new(&folders_path);
        save_webdav_to_managers(&sync_manager, &mut folder_manager, webdav_payload()).unwrap();

        let folder = add_folder_with_manager(
            &mut folder_manager,
            "config".to_string(),
            "~/.ccr/platforms/".to_string(),
            "".to_string(),
            None,
        )
        .unwrap();

        assert_eq!(folder.name, "config");
        assert_eq!(folder.local_path, "~/.ccr/platforms/");
        assert_eq!(folder.remote_path, "/ccr/platforms");
    }

    #[test]
    fn update_sync_folder_upserts_enabled_paths_and_description() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sync_path = temp_dir.path().join("sync.toml");
        let folders_path = temp_dir.path().join("sync_folders.toml");
        let sync_manager = SyncConfigManager::new(&sync_path);
        let mut folder_manager = SyncFolderManager::new(&folders_path);
        save_webdav_to_managers(&sync_manager, &mut folder_manager, webdav_payload()).unwrap();
        add_folder_with_manager(
            &mut folder_manager,
            "config".to_string(),
            "~/.ccr/platforms/".to_string(),
            "".to_string(),
            Some("old".to_string()),
        )
        .unwrap();

        let updated = update_folder_with_manager(
            &mut folder_manager,
            "config".to_string(),
            None,
            Some(false),
            Some("~/.ccr/platforms-v2/".to_string()),
            Some("".to_string()),
            Some("new".to_string()),
        )
        .unwrap();

        assert!(!updated.enabled);
        assert_eq!(updated.local_path, "~/.ccr/platforms-v2/");
        assert_eq!(updated.remote_path, "/ccr/platforms");
        assert_eq!(updated.description, "new");

        let persisted = folder_manager.get_folder("config").unwrap();
        assert_eq!(persisted, updated);
    }
}
