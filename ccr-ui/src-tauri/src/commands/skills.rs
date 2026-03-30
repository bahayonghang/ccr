//! 统一 Skills 命令层。
//!
//! 这里仅保留参数校验、Tauri 对话框与错误映射；
//! 领域逻辑统一下沉到 `ccr::SkillsService`。

use ccr::models::skills::SkillOperationResponse;
use ccr::{
    MarketplaceListResponse, MarketplaceSkill, NpxStatus, SkillContent, SkillRecord,
    SkillSourceRecord, SkillsInstallRequest, SkillsInventoryQuery, SkillsInventoryResponse,
    SkillsService, SkillsSyncRequest,
};
use serde::Serialize;
use tauri_plugin_dialog::DialogExt;

fn map_join_error(error: tokio::task::JoinError) -> String {
    format!("Task join error: {error}")
}

fn map_domain_error<T>(result: ccr::Result<T>) -> Result<T, String> {
    result.map_err(|error| error.to_string())
}

fn new_service() -> Result<SkillsService, String> {
    SkillsService::new().map_err(|error| error.to_string())
}

#[derive(Debug, Serialize)]
pub struct FolderPickResult {
    pub path: Option<String>,
}

#[tauri::command]
pub async fn skills_inventory(
    query: Option<SkillsInventoryQuery>,
) -> Result<SkillsInventoryResponse, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.inventory(query))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_detail(skill_id: String) -> Result<SkillRecord, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.detail(&skill_id))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_content_get(
    skill_id: String,
    installation_id: Option<String>,
) -> Result<SkillContent, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.content_get(&skill_id, installation_id.as_deref()))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_content_save(
    skill_id: String,
    installation_id: String,
    raw: String,
) -> Result<SkillContent, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.content_save(&skill_id, &installation_id, &raw))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_install(
    request: SkillsInstallRequest,
) -> Result<SkillOperationResponse, String> {
    new_service()?
        .install(request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn skills_sync(request: SkillsSyncRequest) -> Result<SkillOperationResponse, String> {
    new_service()?
        .sync(request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn skills_remove_installation(
    skill_id: String,
    installation_id: String,
) -> Result<SkillOperationResponse, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.remove_installation(&skill_id, &installation_id))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_remove_skill(skill_id: String) -> Result<SkillOperationResponse, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.remove_skill(&skill_id))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_sources_list() -> Result<Vec<SkillSourceRecord>, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.sources_list())
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_source_add_git(
    app: tauri::AppHandle,
    url: String,
) -> Result<SkillSourceRecord, String> {
    let record = tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.source_add_git(&url))
    })
    .await
    .map_err(map_join_error)?;
    let _ = crate::skills_watcher::reload(&app);
    record
}

#[tauri::command]
pub async fn skills_source_add_local(
    app: tauri::AppHandle,
    path: String,
) -> Result<SkillSourceRecord, String> {
    let record = tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.source_add_local(&path))
    })
    .await
    .map_err(map_join_error)?;
    let _ = crate::skills_watcher::reload(&app);
    record
}

#[tauri::command]
pub async fn skills_source_sync(source_id: String) -> Result<SkillSourceRecord, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.source_sync(&source_id))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_source_remove(app: tauri::AppHandle, source_id: String) -> Result<(), String> {
    let result = tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.source_remove(&source_id))
    })
    .await
    .map_err(map_join_error)?;
    let _ = crate::skills_watcher::reload(&app);
    result
}

#[tauri::command]
pub async fn skills_marketplace_list(
    query: Option<String>,
    page: usize,
    page_size: usize,
) -> Result<MarketplaceListResponse, String> {
    new_service()?
        .marketplace_list(query, page, page_size)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn skills_marketplace_detail(package_id: String) -> Result<MarketplaceSkill, String> {
    new_service()?
        .marketplace_detail(&package_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn skills_npx_status() -> Result<NpxStatus, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        Ok::<_, String>(service.npx_status())
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_pick_folder(app: tauri::AppHandle) -> Result<FolderPickResult, String> {
    let (tx, rx) = tokio::sync::oneshot::channel::<Option<String>>();
    app.dialog().file().pick_folder(move |folder_path| {
        let _ = tx.send(folder_path.map(|path| path.to_string()));
    });

    let path = rx.await.map_err(|error| format!("Dialog error: {error}"))?;
    Ok(FolderPickResult { path })
}
