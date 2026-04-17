//! 统一 Skills 命令层。
//!
//! 这里仅保留参数校验、Tauri 对话框与错误映射；
//! 领域逻辑统一下沉到 `ccr_skills::SkillsService`。

use ccr_skills::{
    MarketplaceListResponse, MarketplaceSkill, NpxStatus, SkillContent, SkillFileContent,
    SkillFileEntry, SkillOperationResponse, SkillRecord, SkillSourceRecord, SkillsInstallRequest,
    SkillsInstallReviewResponse, SkillsInventoryQuery, SkillsInventoryResponse,
    SkillsNpxCapabilities, SkillsOnboardingCandidate, SkillsService, SkillsSyncRequest,
};
use serde::Serialize;
use tauri_plugin_dialog::DialogExt;

fn map_join_error(error: tokio::task::JoinError) -> String {
    format!("Task join error: {error}")
}

fn map_domain_error<T>(result: ccr_core::Result<T>) -> Result<T, String> {
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
pub async fn skills_files_list(
    skill_id: String,
    installation_id: Option<String>,
) -> Result<Vec<SkillFileEntry>, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.files_list(&skill_id, installation_id.as_deref()))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_file_get(
    skill_id: String,
    path: String,
    installation_id: Option<String>,
) -> Result<SkillFileContent, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.file_get(&skill_id, installation_id.as_deref(), &path))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_onboarding_candidates() -> Result<Vec<SkillsOnboardingCandidate>, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.onboarding_candidates())
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
pub async fn skills_prepare_install(
    request: SkillsInstallRequest,
) -> Result<SkillsInstallReviewResponse, String> {
    new_service()?
        .prepare_install(request)
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
pub async fn skills_npx_capabilities() -> Result<SkillsNpxCapabilities, String> {
    tokio::task::spawn_blocking(move || {
        let service = new_service()?;
        map_domain_error(service.npx_capabilities())
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

// ============================================================================
// Phase 5 — skills_ext Tauri 命令层
// 版本历史 / 回收站 / 启用禁用开关。领域逻辑全部落在 `ccr_skills::skills_ext`。
// ============================================================================

use ccr_skills::skills_ext::{
    DiffResult, FsTrashStore, FsVersionStore, SnapshotSource, ToggleStore, TrashEntry, Version,
    VersionMeta,
};
use std::path::PathBuf;

fn map_versioning_error<T>(
    result: Result<T, ccr_skills::skills_ext::VersioningError>,
) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

fn map_trash_error<T>(result: Result<T, ccr_skills::skills_ext::TrashError>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

fn map_toggle_error<T>(
    result: Result<T, ccr_skills::skills_ext::ToggleError>,
) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

// ------------------- Version Store --------------------------

#[tauri::command]
pub async fn skills_version_list(install_path: String) -> Result<Vec<VersionMeta>, String> {
    tokio::task::spawn_blocking(move || {
        let store = map_versioning_error(FsVersionStore::open())?;
        map_versioning_error(store.history(&PathBuf::from(install_path)))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_version_get(
    install_path: String,
    version_id: String,
) -> Result<Option<Version>, String> {
    tokio::task::spawn_blocking(move || {
        let store = map_versioning_error(FsVersionStore::open())?;
        map_versioning_error(store.get(&PathBuf::from(install_path), &version_id))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_version_snapshot(
    install_path: String,
    skill_name: String,
    message: String,
    source: Option<String>, // "auto" | "manual"；默认 manual
) -> Result<VersionMeta, String> {
    tokio::task::spawn_blocking(move || {
        let store = map_versioning_error(FsVersionStore::open())?;
        let snapshot_source = match source.as_deref() {
            Some("auto") => SnapshotSource::Auto,
            _ => SnapshotSource::Manual,
        };
        map_versioning_error(store.snapshot(
            &PathBuf::from(install_path),
            &skill_name,
            &message,
            snapshot_source,
        ))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_version_diff(
    install_path: String,
    old_id: String,
    new_id: String,
) -> Result<Option<DiffResult>, String> {
    tokio::task::spawn_blocking(move || {
        let store = map_versioning_error(FsVersionStore::open())?;
        map_versioning_error(store.diff(&PathBuf::from(install_path), &old_id, &new_id))
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_version_rollback(
    app: tauri::AppHandle,
    install_path: String,
    version_id: String,
) -> Result<VersionMeta, String> {
    let meta = tokio::task::spawn_blocking(move || {
        let store = map_versioning_error(FsVersionStore::open())?;
        map_versioning_error(store.rollback(&PathBuf::from(install_path), &version_id))
    })
    .await
    .map_err(map_join_error)?;
    // 回滚改写了 skill 目录，主动触发 watcher 刷新
    let _ = crate::skills_watcher::reload(&app);
    meta
}

// ------------------- Trash Store ----------------------------

#[tauri::command]
pub async fn skills_trash_list() -> Result<Vec<TrashEntry>, String> {
    tokio::task::spawn_blocking(move || {
        let store = map_trash_error(FsTrashStore::open())?;
        map_trash_error(store.list())
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_trash_soft_delete(
    app: tauri::AppHandle,
    install_path: String,
    skill_name: String,
) -> Result<TrashEntry, String> {
    let entry = tokio::task::spawn_blocking(move || {
        let store = map_trash_error(FsTrashStore::open())?;
        map_trash_error(store.move_to_trash(&PathBuf::from(install_path), &skill_name))
    })
    .await
    .map_err(map_join_error)?;
    // 源 skill 目录已被移走，watcher 感知变化
    let _ = crate::skills_watcher::reload(&app);
    entry
}

#[tauri::command]
pub async fn skills_trash_restore(
    app: tauri::AppHandle,
    trash_id: String,
) -> Result<String, String> {
    let path = tokio::task::spawn_blocking(move || {
        let store = map_trash_error(FsTrashStore::open())?;
        map_trash_error(store.restore(&trash_id)).map(|p| p.to_string_lossy().into_owned())
    })
    .await
    .map_err(map_join_error)?;
    let _ = crate::skills_watcher::reload(&app);
    path
}

#[tauri::command]
pub async fn skills_trash_purge(trash_id: String) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        let store = map_trash_error(FsTrashStore::open())?;
        map_trash_error(store.permanent_delete(&trash_id))
    })
    .await
    .map_err(map_join_error)?
}

// ------------------- Toggle (permissions.deny) --------------

#[tauri::command]
pub async fn skills_toggle_set(skill_name: String, enabled: bool) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        let store = map_toggle_error(ToggleStore::open())?;
        map_toggle_error(store.set_enabled(&skill_name, enabled))?;
        Ok::<bool, String>(enabled)
    })
    .await
    .map_err(map_join_error)?
}

#[tauri::command]
pub async fn skills_toggle_list_disabled() -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || {
        let store = map_toggle_error(ToggleStore::open())?;
        map_toggle_error(store.list_disabled())
    })
    .await
    .map_err(map_join_error)?
}

// ------------------- Taxonomy / Conflicts / Health (Phase 6-7) ---

use ccr_skills::skills_ext::{
    CategorySummary as TaxonomyCategorySummary, Classification, ConflictGroup, HealthReport,
    MergeSuggestion, SkillInput, classify_all as taxonomy_classify_all, compute_health,
    detect_conflicts as scanner_detect_conflicts, enabled_plugin_install_locations,
    merge_suggestions as taxonomy_merge_suggestions,
};

/// 前端 payload：用于分类/冲突/建议的最小字段。
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonomyInput {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub frontmatter_category: Option<String>,
    #[serde(default)]
    pub real_path: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonomyResponse {
    pub classifications: Vec<Classification>,
    pub categories: Vec<TaxonomyCategorySummary>,
    pub merge_suggestions: Vec<MergeSuggestion>,
    pub conflicts: Vec<ConflictGroup>,
    pub health: HealthReport,
}

#[tauri::command]
pub async fn skills_taxonomy_analyze(
    items: Vec<TaxonomyInput>,
) -> Result<TaxonomyResponse, String> {
    tokio::task::spawn_blocking(move || {
        // 转 SkillInput（借用 items 字段）
        let inputs: Vec<SkillInput> = items
            .iter()
            .map(|it| SkillInput {
                id: &it.id,
                name: &it.name,
                description: &it.description,
                frontmatter_category: it.frontmatter_category.as_deref(),
            })
            .collect();

        let (classifications, categories) = taxonomy_classify_all(&inputs);
        let merge = taxonomy_merge_suggestions(&inputs, &classifications);

        // Conflicts：name + real_path，real_path 缺失时用 id 兜底保证唯一
        let conflict_entries: Vec<(&str, &str, &str)> = items
            .iter()
            .map(|it| {
                (
                    it.id.as_str(),
                    it.name.as_str(),
                    it.real_path.as_deref().unwrap_or(it.id.as_str()),
                )
            })
            .collect();
        let conflicts = scanner_detect_conflicts(&conflict_entries);

        // Health 汇总
        let disabled_names: Vec<String> = match ccr_skills::skills_ext::ToggleStore::open() {
            Ok(store) => store.list_disabled().unwrap_or_default(),
            Err(_) => Vec::new(),
        };
        let plugin_count = dirs::home_dir()
            .map(|home| enabled_plugin_install_locations(&home).len())
            .unwrap_or(0);
        let health = compute_health(
            items.len(),
            &conflicts,
            &merge,
            &disabled_names,
            plugin_count,
        );

        Ok::<_, String>(TaxonomyResponse {
            classifications,
            categories,
            merge_suggestions: merge,
            conflicts,
            health,
        })
    })
    .await
    .map_err(map_join_error)?
}
