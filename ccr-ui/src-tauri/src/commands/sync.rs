//! WebDAV 同步命令 — push / pull / status / folder CRUD / 账号增删测。

use ccr_sync::{
    SyncConfig, SyncConfigManager, SyncFolder, SyncFolderManager, SyncService, WebDavConfig,
    expand_path, insecure_loopback_http_enabled, validate_webdav_url,
};
use serde::{Deserialize, Serialize};
use std::fs;
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
    /// UI 表单明文入参：Secret 包裹，payload Debug/日志不泄露
    pub password: ccr_core::Secret,
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

/// 固定同步资产类型。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SyncAssetKind {
    Directory,
    File,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncEncryptionState {
    NotApplicable,
    V2Required,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncAssetOperationInput {
    pub id: String,
    #[serde(default)]
    pub force: bool,
    pub passphrase: Option<ccr_core::Secret>,
    #[serde(default)]
    pub migrate_plaintext_v1: bool,
}

#[derive(Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncAllAssetsInput {
    #[serde(default)]
    pub force: bool,
    pub passphrase: Option<ccr_core::Secret>,
    #[serde(default)]
    pub migrate_plaintext_v1: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncAssetGroup {
    Ccr,
    Claude,
    Codex,
}

impl SyncAssetGroup {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ccr => "ccr",
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SyncAssetDefinition {
    id: &'static str,
    group: SyncAssetGroup,
    name: &'static str,
    description: &'static str,
    kind: SyncAssetKind,
    sensitive: bool,
    local_path: &'static str,
    remote_segment: &'static str,
    canonical_name: Option<&'static str>,
}

/// 同步资产状态信息（前端 camelCase）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncAssetInfo {
    pub id: String,
    pub group: String,
    pub name: String,
    pub description: String,
    pub kind: SyncAssetKind,
    pub sensitive: bool,
    pub encryption_state: SyncEncryptionState,
    pub local_path: String,
    pub resolved_local_path: String,
    pub remote_path: String,
    pub local_exists: bool,
    pub remote_exists: Option<bool>,
    pub canonical_name: Option<String>,
}

const SYNC_ASSETS: &[SyncAssetDefinition] = &[
    SyncAssetDefinition {
        id: "ccr-platforms",
        group: SyncAssetGroup::Ccr,
        name: "CCR Platforms",
        description: "All CCR platform configuration, including auth, URL, key, and provider routing fields.",
        kind: SyncAssetKind::Directory,
        sensitive: true,
        local_path: "~/.ccr/platforms/",
        remote_segment: "platforms/",
        canonical_name: None,
    },
    SyncAssetDefinition {
        id: "claude-settings",
        group: SyncAssetGroup::Claude,
        name: "settings.json",
        description: "Claude Code user settings only; the rest of ~/.claude is intentionally excluded.",
        kind: SyncAssetKind::File,
        sensitive: true,
        local_path: "~/.claude/settings.json",
        remote_segment: "claude/settings.json",
        canonical_name: Some("settings.json"),
    },
    SyncAssetDefinition {
        id: "claude-memory",
        group: SyncAssetGroup::Claude,
        name: "CLAUDE.md",
        description: "Claude Code global memory and instructions file.",
        kind: SyncAssetKind::File,
        sensitive: false,
        local_path: "~/.claude/CLAUDE.md",
        remote_segment: "claude/CLAUDE.md",
        canonical_name: Some("CLAUDE.md"),
    },
    SyncAssetDefinition {
        id: "codex-config",
        group: SyncAssetGroup::Codex,
        name: "config.toml",
        description: "Codex user configuration only; the rest of ~/.codex is intentionally excluded.",
        kind: SyncAssetKind::File,
        sensitive: true,
        local_path: "~/.codex/config.toml",
        remote_segment: "codex/config.toml",
        canonical_name: Some("config.toml"),
    },
    SyncAssetDefinition {
        id: "codex-agents",
        group: SyncAssetGroup::Codex,
        name: "AGENTS.md",
        description: "Codex global instructions file.",
        kind: SyncAssetKind::File,
        sensitive: false,
        local_path: "~/.codex/AGENTS.md",
        remote_segment: "codex/AGENTS.md",
        canonical_name: Some("AGENTS.md"),
    },
];

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
        enabled: config.enabled,
        url: config.webdav_url.clone(),
        username: config.username.clone(),
        password: config.password.clone(),
        base_remote_path: normalize_remote_base(&config.remote_path),
        auto_sync: config.auto_sync,
    }
}

fn sync_config_from_webdav(webdav: &WebDavConfig, remote_path: &str) -> SyncConfig {
    SyncConfig {
        enabled: webdav.enabled,
        webdav_url: webdav.url.clone(),
        username: webdav.username.clone(),
        password: webdav.password.clone(),
        remote_path: remote_path.to_string(),
        auto_sync: webdav.auto_sync,
    }
}

fn sync_config_for_folder(
    manager: &SyncFolderManager,
    folder: &SyncFolder,
) -> Result<SyncConfig, String> {
    let webdav = load_webdav_config(manager)?;

    if !webdav_is_complete(&webdav) {
        return Err("WebDAV account is incomplete. Please configure WebDAV first.".to_string());
    }

    Ok(sync_config_from_webdav(&webdav, &folder.remote_path))
}

fn asset_by_id(id: &str) -> Result<&'static SyncAssetDefinition, String> {
    SYNC_ASSETS
        .iter()
        .find(|asset| asset.id == id)
        .ok_or_else(|| format!("Unknown sync asset: {id}"))
}

fn asset_remote_path(asset: &SyncAssetDefinition, webdav: &WebDavConfig) -> String {
    let base = normalize_remote_base(&webdav.base_remote_path);
    let segment = asset.remote_segment.trim_start_matches('/');
    let joined = format!("{}/{}", base, segment).replace('\\', "/");
    if asset.kind == SyncAssetKind::Directory && !joined.ends_with('/') {
        format!("{joined}/")
    } else {
        joined
    }
}

fn sync_config_for_asset(
    manager: &SyncFolderManager,
    asset: &SyncAssetDefinition,
) -> Result<SyncConfig, String> {
    let webdav = load_webdav_config(manager)?;

    if !webdav_is_complete(&webdav) {
        return Err("WebDAV account is incomplete. Please configure WebDAV first.".to_string());
    }

    Ok(sync_config_from_webdav(
        &webdav,
        &asset_remote_path(asset, &webdav),
    ))
}

fn webdav_is_complete(webdav: &WebDavConfig) -> bool {
    webdav.enabled
        && !webdav.url.trim().is_empty()
        && !webdav.username.trim().is_empty()
        && !webdav.password.expose().trim().is_empty()
}

fn sync_config_is_complete(config: &SyncConfig) -> bool {
    config.enabled
        && !config.webdav_url.trim().is_empty()
        && !config.username.trim().is_empty()
        && !config.password.expose().trim().is_empty()
}

fn load_legacy_sync_config() -> Result<SyncConfig, String> {
    SyncConfigManager::with_default()
        .and_then(|manager| manager.load())
        .map_err(|e| format!("Failed to load legacy WebDAV config: {e}"))
}

fn load_webdav_config(manager: &SyncFolderManager) -> Result<WebDavConfig, String> {
    load_webdav_config_with_legacy(manager, load_legacy_sync_config)
}

fn load_webdav_config_with_legacy(
    manager: &SyncFolderManager,
    load_legacy: impl FnOnce() -> Result<SyncConfig, String>,
) -> Result<WebDavConfig, String> {
    let canonical_exists = manager.config_path().exists();
    let folder_webdav = manager.get_webdav_config();
    if canonical_exists {
        return folder_webdav.map_err(|e| format!("Failed to load WebDAV config: {e}"));
    }

    if let Ok(config) = load_legacy()
        && sync_config_is_complete(&config)
    {
        let webdav = webdav_from_sync_config(&config);
        let mut canonical = SyncFolderManager::new(manager.config_path());
        canonical
            .update_webdav_config(webdav.clone())
            .map_err(|e| format!("Failed to migrate legacy WebDAV config: {e}"))?;
        return Ok(webdav);
    }

    match folder_webdav {
        Ok(mut webdav) => {
            if webdav.base_remote_path == WebDavConfig::default().base_remote_path {
                webdav.base_remote_path = normalize_remote_base(&SyncConfig::default().remote_path);
            }
            Ok(webdav)
        }
        Err(e) => Err(format!("Failed to load WebDAV config: {e}")),
    }
}

fn validate_webdav_transport(webdav_url: &str) -> Result<(), String> {
    validate_webdav_url(webdav_url, insecure_loopback_http_enabled())
        .map(|_| ())
        .map_err(|e| format!("Invalid WebDAV transport: {e}"))
}

fn resolve_asset_local_path(
    asset: &SyncAssetDefinition,
) -> Result<(PathBuf, Option<String>), String> {
    let expanded = match asset.group {
        SyncAssetGroup::Ccr => resolve_ccr_asset_local_path(asset),
        SyncAssetGroup::Claude | SyncAssetGroup::Codex => expand_path(asset.local_path),
    }
    .map_err(|e| format!("Failed to expand asset path {}: {e}", asset.local_path))?;

    if asset.kind == SyncAssetKind::Directory || expanded.exists() {
        return Ok((expanded, asset.canonical_name.map(str::to_string)));
    }

    let Some(canonical_name) = asset.canonical_name else {
        return Ok((expanded, None));
    };

    let Some(parent) = expanded.parent() else {
        return Ok((expanded, Some(canonical_name.to_string())));
    };

    if let Some(found) = find_case_insensitive_child(parent, canonical_name) {
        return Ok((found, Some(canonical_name.to_string())));
    }

    Ok((expanded, Some(canonical_name.to_string())))
}

fn resolve_ccr_asset_local_path(
    asset: &SyncAssetDefinition,
) -> ccr_core::core::error::Result<PathBuf> {
    let root = std::env::var_os("CCR_ROOT").map(PathBuf::from);
    resolve_ccr_asset_local_path_with_root(asset, root.as_deref())
}

fn resolve_ccr_asset_local_path_with_root(
    asset: &SyncAssetDefinition,
    root: Option<&Path>,
) -> ccr_core::core::error::Result<PathBuf> {
    if asset.id == "ccr-platforms"
        && let Some(root) = root
    {
        return Ok(root.join("platforms"));
    }

    expand_path(asset.local_path)
}

fn find_case_insensitive_child(parent: &Path, canonical_name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(parent).ok()?;
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        if file_name
            .to_string_lossy()
            .eq_ignore_ascii_case(canonical_name)
        {
            return Some(entry.path());
        }
    }
    None
}

/// `asset_info` 的远端存在性三态映射（**Property 5: Preservation**，保持不变）。
///
/// 把 `remote_exists()` 的 `Result<bool, E>` 映射为前端响应的 `Option<bool>`：
/// - `Ok(true)` → `Some(true)`（present / 远端就绪）
/// - `Ok(false)` → `Some(false)`（missing / 远端缺失）
/// - `Err(_)` → `None`（unknown / 「未测试」）
///
/// 语义与原内联的 `service.remote_exists().await.ok()` 完全一致，抽出仅为
/// 可在无 WebDAV 服务器的情况下对三态映射做单元测试。
fn asset_remote_exists_state<E>(result: Result<bool, E>) -> Option<bool> {
    result.ok()
}

async fn asset_info(
    manager: &SyncFolderManager,
    asset: &SyncAssetDefinition,
) -> Result<SyncAssetInfo, String> {
    let (local_path, canonical_name) = resolve_asset_local_path(asset)?;
    let local_exists = tokio::fs::try_exists(&local_path).await.unwrap_or(false);
    let webdav = load_webdav_config(manager)?;
    let remote_path = asset_remote_path(asset, &webdav);
    let remote_exists = if !webdav_is_complete(&webdav) {
        None
    } else {
        let config = sync_config_from_webdav(&webdav, &remote_path);
        match SyncService::new(&config).await {
            Ok(service) => asset_remote_exists_state(service.remote_exists().await),
            Err(_) => None,
        }
    };

    Ok(SyncAssetInfo {
        id: asset.id.to_string(),
        group: asset.group.as_str().to_string(),
        name: asset.name.to_string(),
        description: asset.description.to_string(),
        kind: asset.kind,
        sensitive: asset.sensitive,
        encryption_state: if asset.sensitive {
            SyncEncryptionState::V2Required
        } else {
            SyncEncryptionState::NotApplicable
        },
        local_path: asset.local_path.to_string(),
        resolved_local_path: local_path.display().to_string(),
        remote_path,
        local_exists,
        remote_exists,
        canonical_name,
    })
}

async fn push_asset_config(
    manager: &SyncFolderManager,
    asset: &SyncAssetDefinition,
    force: bool,
    passphrase: Option<&ccr_core::Secret>,
) -> Result<(), String> {
    let (local_path, _) = resolve_asset_local_path(asset)?;
    if !tokio::fs::try_exists(&local_path).await.unwrap_or(false) {
        return Err(format!(
            "Local asset does not exist: {}",
            local_path.display()
        ));
    }

    let sync_config = sync_config_for_asset(manager, asset)?;
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
                "sync_conflict: Remote asset already exists; rerun with force to overwrite."
                    .to_string(),
            );
        }
    }

    if asset.sensitive {
        let passphrase = require_asset_passphrase(asset, passphrase)?;
        service
            .push_sensitive(&local_path, asset.id, passphrase)
            .await
            .map_err(|e| format!("Push failed: {e}"))
    } else {
        service
            .push(&local_path, None)
            .await
            .map_err(|e| format!("Push failed: {e}"))
    }
}

async fn pull_asset_config(
    manager: &SyncFolderManager,
    asset: &SyncAssetDefinition,
    force: bool,
    passphrase: Option<&ccr_core::Secret>,
    migrate_plaintext_v1: bool,
) -> Result<Option<PathBuf>, String> {
    let (local_path, _) = resolve_asset_local_path(asset)?;
    let sync_config = sync_config_for_asset(manager, asset)?;
    let service = SyncService::new(&sync_config)
        .await
        .map_err(|e| format!("Failed to create SyncService: {e}"))?;

    let remote_exists = service
        .remote_exists()
        .await
        .map_err(|e| format!("Failed to check remote path: {e}"))?;
    if !remote_exists {
        return Err("Remote asset does not exist. Upload this asset first.".to_string());
    }

    let local_exists = tokio::fs::try_exists(&local_path)
        .await
        .map_err(|e| format!("Failed to check local path: {e}"))?;
    if local_exists && !force {
        return Err("sync_conflict: Local asset already exists; rerun with force to overwrite and create a backup.".to_string());
    }

    if asset.sensitive {
        let passphrase = require_asset_passphrase(asset, passphrase)?;
        service
            .pull_sensitive_with_backup(&local_path, asset.id, passphrase, migrate_plaintext_v1)
            .await
            .map_err(|e| format!("Pull failed: {e}"))
    } else {
        service
            .pull_with_backup(&local_path)
            .await
            .map_err(|e| format!("Pull failed: {e}"))
    }
}

async fn sync_asset_config(
    manager: &SyncFolderManager,
    asset: &SyncAssetDefinition,
    force: bool,
    passphrase: Option<&ccr_core::Secret>,
    migrate_plaintext_v1: bool,
) -> Result<Option<PathBuf>, String> {
    let (local_path, _) = resolve_asset_local_path(asset)?;
    let local_exists = tokio::fs::try_exists(&local_path)
        .await
        .map_err(|e| format!("Failed to check local path: {e}"))?;

    let sync_config = sync_config_for_asset(manager, asset)?;
    let service = SyncService::new(&sync_config)
        .await
        .map_err(|e| format!("Failed to create SyncService: {e}"))?;
    let remote_exists = service
        .remote_exists()
        .await
        .map_err(|e| format!("Failed to check remote path: {e}"))?;

    match decide_sync_asset_action(local_exists, remote_exists, force) {
        SyncAssetAction::PushCreate | SyncAssetAction::PushOverwrite => {
            push_asset_config(manager, asset, true, passphrase)
                .await
                .map(|_| None)
        }
        SyncAssetAction::Pull => {
            pull_asset_config(
                manager,
                asset,
                false,
                passphrase,
                migrate_plaintext_v1,
            )
            .await
        }
        SyncAssetAction::Conflict => Err(
            "sync_conflict: Local and remote assets both exist; choose force/prefer-local to overwrite remote."
                .to_string(),
        ),
        SyncAssetAction::Missing => Err(format!(
            "Neither local nor remote asset exists: {}",
            local_path.display()
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncAssetAction {
    PushCreate,
    PushOverwrite,
    Pull,
    Conflict,
    Missing,
}

fn decide_sync_asset_action(
    local_exists: bool,
    remote_exists: bool,
    force: bool,
) -> SyncAssetAction {
    match (local_exists, remote_exists, force) {
        (true, false, _) => SyncAssetAction::PushCreate,
        (false, true, _) => SyncAssetAction::Pull,
        (true, true, true) => SyncAssetAction::PushOverwrite,
        (true, true, false) => SyncAssetAction::Conflict,
        (false, false, _) => SyncAssetAction::Missing,
    }
}

fn require_asset_passphrase<'a>(
    asset: &SyncAssetDefinition,
    passphrase: Option<&'a ccr_core::Secret>,
) -> Result<&'a str, String> {
    let passphrase = passphrase
        .map(ccr_core::Secret::expose)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!(
                "sync_envelope_passphrase_required: Sensitive asset {} requires an operation passphrase.",
                asset.id
            )
        })?;
    Ok(passphrase)
}

fn save_webdav_to_manager(
    folder_manager: &mut SyncFolderManager,
    payload: WebDavConfigInput,
) -> Result<SyncConfig, String> {
    let mut config = build_sync_config(payload);
    config.remote_path = normalize_remote_base(&config.remote_path);
    validate_webdav_transport(&config.webdav_url)?;

    folder_manager
        .update_webdav_config(webdav_from_sync_config(&config))
        .map_err(|e| format!("Failed to save canonical WebDAV config: {e}"))?;

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

    service
        .pull_with_backup(&local_path)
        .await
        .map(|_| ())
        .map_err(|e| format!("Pull failed: {e}"))
}

fn asset_operation_label(direction: SyncAssetDirection) -> &'static str {
    match direction {
        SyncAssetDirection::Push => "upload",
        SyncAssetDirection::Pull => "download",
        SyncAssetDirection::Sync => "sync",
    }
}

// ── 命令实现 ──

#[tauri::command]
pub async fn list_sync_assets() -> Result<Vec<SyncAssetInfo>, String> {
    let manager = SyncFolderManager::with_default()
        .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;
    let mut assets = Vec::with_capacity(SYNC_ASSETS.len());
    for asset in SYNC_ASSETS {
        assets.push(asset_info(&manager, asset).await?);
    }
    Ok(assets)
}

#[tauri::command]
pub async fn sync_push_asset(
    payload: SyncAssetOperationInput,
) -> Result<SyncOperationResult, String> {
    sync_one_asset(payload, SyncAssetDirection::Push).await
}

#[tauri::command]
pub async fn sync_pull_asset(
    payload: SyncAssetOperationInput,
) -> Result<SyncOperationResult, String> {
    sync_one_asset(payload, SyncAssetDirection::Pull).await
}

#[tauri::command]
pub async fn sync_asset(payload: SyncAssetOperationInput) -> Result<SyncOperationResult, String> {
    sync_one_asset(payload, SyncAssetDirection::Sync).await
}

#[tauri::command]
pub async fn sync_all_assets(
    payload: Option<SyncAllAssetsInput>,
) -> Result<SyncOperationResult, String> {
    let payload = payload.unwrap_or_default();
    let start = Instant::now();
    let manager = SyncFolderManager::with_default()
        .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;
    run_asset_syncs(
        &manager,
        SYNC_ASSETS.iter().collect(),
        SyncAssetDirection::Sync,
        payload.force,
        payload.passphrase.as_ref(),
        payload.migrate_plaintext_v1,
        start,
    )
    .await
}

#[derive(Clone, Copy)]
enum SyncAssetDirection {
    Push,
    Pull,
    Sync,
}

async fn sync_one_asset(
    payload: SyncAssetOperationInput,
    direction: SyncAssetDirection,
) -> Result<SyncOperationResult, String> {
    let start = Instant::now();
    let manager = SyncFolderManager::with_default()
        .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;
    let asset = asset_by_id(&payload.id)?;
    run_asset_syncs(
        &manager,
        vec![asset],
        direction,
        payload.force,
        payload.passphrase.as_ref(),
        payload.migrate_plaintext_v1,
        start,
    )
    .await
}

async fn run_asset_syncs(
    manager: &SyncFolderManager,
    assets: Vec<&SyncAssetDefinition>,
    direction: SyncAssetDirection,
    force: bool,
    passphrase: Option<&ccr_core::Secret>,
    migrate_plaintext_v1: bool,
    start: Instant,
) -> Result<SyncOperationResult, String> {
    let total = assets.len();
    let mut success_count = 0usize;
    let mut failed = Vec::new();
    let mut backup_lines = Vec::new();

    for asset in assets {
        let result = match direction {
            SyncAssetDirection::Push => push_asset_config(manager, asset, force, passphrase)
                .await
                .map(|_| None),
            SyncAssetDirection::Pull => {
                pull_asset_config(manager, asset, force, passphrase, migrate_plaintext_v1).await
            }
            SyncAssetDirection::Sync => {
                sync_asset_config(manager, asset, force, passphrase, migrate_plaintext_v1).await
            }
        };

        match result {
            Ok(backup) => {
                success_count += 1;
                if let Some(path) = backup {
                    backup_lines.push(format!("{} backup: {}", asset.id, path.display()));
                }
            }
            Err(message) => failed.push(SyncOperationFailure {
                folder: asset.id.to_string(),
                message,
            }),
        }
    }

    let label = asset_operation_label(direction);
    let success = failed.is_empty();
    let mut message = if success {
        format!("Completed {label} for {success_count}/{total} sync asset(s).")
    } else {
        format!(
            "Completed {label} for {success_count}/{total} sync asset(s); {} failed.",
            failed.len()
        )
    };
    if !backup_lines.is_empty() {
        message.push('\n');
        message.push_str(&backup_lines.join("\n"));
    }

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

/// 把 legacy 同步根的 `remote_path` 规范为目录型路径（确保以 `/` 结尾）。
///
/// legacy 同步根（如 `/ccr`）始终是目录，但其存储路径经
/// `normalize_remote_base` 去掉了尾斜杠，会误走 `remote_exists()` 的文件
/// （GET）分支。补尾斜杠后即可走目录（PROPFIND）分支。空路径回退为 `/`。
fn directory_remote_path(remote_path: &str) -> String {
    let trimmed = remote_path.trim();
    if trimmed.is_empty() {
        return "/".to_string();
    }
    if trimmed.ends_with('/') {
        trimmed.to_string()
    } else {
        format!("{trimmed}/")
    }
}

/// 以目录型路径检查 legacy 同步根是否存在（走 PROPFIND 目录分支）。
async fn directory_remote_exists(config: &SyncConfig) -> Option<bool> {
    let mut dir_config = config.clone();
    dir_config.remote_path = directory_remote_path(&config.remote_path);
    match SyncService::new(&dir_config).await {
        Ok(service) => service.remote_exists().await.ok(),
        Err(_) => None,
    }
}

#[tauri::command]
pub async fn sync_status() -> Result<SyncStatusInfo, String> {
    let webdav = tokio::task::spawn_blocking(|| {
        let manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;
        load_webdav_config(&manager)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    let configured = webdav_is_complete(&webdav);
    let config = sync_config_from_webdav(&webdav, &webdav.base_remote_path);

    // 2. 如果已配置，测试连接并检查远程路径
    let (remote_accessible, remote_exists) = if configured {
        match SyncService::new(&config).await {
            Ok(service) => {
                let accessible = service.test_connection().await.is_ok();
                let exists = if accessible {
                    // legacy 同步根（如 `/ccr`）始终是目录。其存储的 remote_path 经
                    // `normalize_remote_base` 去掉了尾斜杠，会误走文件（GET）分支。
                    // 这里为目录型目标补尾斜杠，确保走 PROPFIND 目录分支。
                    directory_remote_exists(&config).await
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
        let mut folder_manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;

        save_webdav_to_manager(&mut folder_manager, payload)
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

    let result = match validate_webdav_transport(&config.webdav_url) {
        Err(message) => WebDavTestResult { ok: false, message },
        Ok(()) => match SyncService::new(&config).await {
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
        },
    };

    Ok(result)
}

/// 断开账号：清空 canonical folder-manager 配置，并删除 migration-only legacy 文件。
#[tauri::command]
pub async fn clear_webdav_config() -> Result<(), String> {
    tokio::task::spawn_blocking(|| {
        SyncConfigManager::with_default()
            .map_err(|e| format!("Failed to create legacy SyncConfigManager: {e}"))?
            .delete()
            .map_err(|e| format!("Failed to delete legacy sync config: {e}"))?;

        let mut folder_manager = SyncFolderManager::with_default()
            .map_err(|e| format!("Failed to create SyncFolderManager: {e}"))?;
        let cleared = WebDavConfig {
            enabled: false,
            username: String::new(),
            password: ccr_core::Secret::default(),
            ..WebDavConfig::default()
        };
        folder_manager
            .update_webdav_config(cleared)
            .map_err(|e| format!("Failed to clear canonical WebDAV config: {e}"))
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
            password: ccr_core::Secret::from("secret"),
            remote_path: Some("/ccr/".to_string()),
            auto_sync: Some(false),
        }
    }

    #[test]
    fn set_webdav_config_writes_only_canonical_folder_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sync_path = temp_dir.path().join("sync.toml");
        let folders_path = temp_dir.path().join("sync_folders.toml");
        let mut folder_manager = SyncFolderManager::new(&folders_path);

        let saved = save_webdav_to_manager(&mut folder_manager, webdav_payload())
            .expect("webdav config should save to canonical config");

        assert_eq!(saved.remote_path, "/ccr");
        assert!(
            !sync_path.exists(),
            "save must not recreate migration-only sync.toml"
        );

        let folders_config = folder_manager.load_config().unwrap();
        assert!(folders_config.webdav.enabled);
        assert_eq!(folders_config.webdav.url, "https://dav.example.com/");
        assert_eq!(folders_config.webdav.username, "user@example.com");
        assert_eq!(folders_config.webdav.password, "secret");
        assert_eq!(folders_config.webdav.base_remote_path, "/ccr");
    }

    #[test]
    fn set_webdav_config_rejects_non_loopback_http_before_persistence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let folders_path = temp_dir.path().join("sync_folders.toml");
        let mut folder_manager = SyncFolderManager::new(&folders_path);
        let mut payload = webdav_payload();
        payload.webdav_url = "http://dav.example.com/".to_string();

        let error = save_webdav_to_manager(&mut folder_manager, payload).unwrap_err();
        assert!(error.contains("https_required"));
        assert!(!folders_path.exists());
    }

    #[test]
    fn legacy_webdav_is_migrated_once_into_canonical_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let folders_path = temp_dir.path().join("sync_folders.toml");
        let folder_manager = SyncFolderManager::new(&folders_path);
        let legacy = build_sync_config(webdav_payload());

        let migrated = load_webdav_config_with_legacy(&folder_manager, || Ok(legacy)).unwrap();
        assert_eq!(migrated.url, "https://dav.example.com/");
        assert!(folders_path.exists());

        let canonical = load_webdav_config_with_legacy(&folder_manager, || {
            Err("legacy must not be read after canonical migration".to_string())
        })
        .unwrap();
        assert_eq!(canonical, migrated);
    }

    #[test]
    fn existing_incomplete_canonical_config_never_falls_back_to_legacy() {
        let temp_dir = tempfile::tempdir().unwrap();
        let folders_path = temp_dir.path().join("sync_folders.toml");
        let folder_manager = SyncFolderManager::new(&folders_path);
        let mut config = ccr_sync::SyncFoldersConfig::default();
        config.webdav.enabled = false;
        config.webdav.username = "canonical-user".to_string();
        config.webdav.password = ccr_core::Secret::from("canonical-password");
        folder_manager.save_config(&config).unwrap();

        let canonical = load_webdav_config_with_legacy(&folder_manager, || {
            panic!("legacy must not be read after canonical config exists")
        })
        .unwrap();

        assert!(!webdav_is_complete(&canonical));
    }

    #[test]
    fn add_sync_folder_uses_platforms_segment_for_default_config_folder() {
        let temp_dir = tempfile::tempdir().unwrap();
        let folders_path = temp_dir.path().join("sync_folders.toml");
        let mut folder_manager = SyncFolderManager::new(&folders_path);
        save_webdav_to_manager(&mut folder_manager, webdav_payload()).unwrap();

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
        let folders_path = temp_dir.path().join("sync_folders.toml");
        let mut folder_manager = SyncFolderManager::new(&folders_path);
        save_webdav_to_manager(&mut folder_manager, webdav_payload()).unwrap();
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

    #[test]
    fn sync_asset_manifest_contains_expected_ids() {
        let ids = SYNC_ASSETS.iter().map(|asset| asset.id).collect::<Vec<_>>();
        assert_eq!(
            ids,
            vec![
                "ccr-platforms",
                "claude-settings",
                "claude-memory",
                "codex-config",
                "codex-agents",
            ]
        );
    }

    #[test]
    fn asset_remote_path_uses_base_and_canonical_segments() {
        let webdav = WebDavConfig {
            enabled: true,
            url: "https://dav.example.com".to_string(),
            username: "user".to_string(),
            password: ccr_core::Secret::from("secret"),
            base_remote_path: "/ccr/".to_string(),
            auto_sync: false,
        };

        assert_eq!(
            asset_remote_path(&SYNC_ASSETS[0], &webdav),
            "/ccr/platforms/"
        );
        assert_eq!(
            asset_remote_path(&SYNC_ASSETS[2], &webdav),
            "/ccr/claude/CLAUDE.md"
        );
        assert_eq!(
            asset_remote_path(&SYNC_ASSETS[4], &webdav),
            "/ccr/codex/AGENTS.md"
        );
    }

    #[test]
    fn load_webdav_config_uses_legacy_default_base_for_unconfigured_folder_defaults() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sync_path = temp_dir.path().join("missing-sync.toml");
        let folders_path = temp_dir.path().join("missing-folders.toml");
        let sync_manager = SyncConfigManager::new(&sync_path);
        let folder_manager = SyncFolderManager::new(&folders_path);

        let webdav = load_webdav_config_with_legacy(&folder_manager, || {
            sync_manager
                .load()
                .map_err(|e| format!("Failed to load legacy WebDAV config: {e}"))
        })
        .expect("default WebDAV config should still be returned");

        assert_eq!(webdav.base_remote_path, "/ccr");
    }

    #[test]
    fn find_case_insensitive_child_matches_markdown_variants() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("Agents.md");
        std::fs::write(&file, "rules").unwrap();

        let found = find_case_insensitive_child(temp_dir.path(), "AGENTS.md")
            .expect("case-insensitive variant should be found");

        assert_eq!(found.file_name().unwrap().to_string_lossy(), "Agents.md");
    }

    #[test]
    fn ccr_asset_path_honors_environment_root_when_set() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ccr_root = temp_dir.path().join(".ccr");
        std::fs::create_dir_all(ccr_root.join("platforms")).unwrap();

        let resolved =
            resolve_ccr_asset_local_path_with_root(&SYNC_ASSETS[0], Some(&ccr_root)).unwrap();

        assert_eq!(resolved, ccr_root.join("platforms"));
    }

    #[test]
    fn sync_asset_direction_labels_distinguish_push_pull_and_sync() {
        assert_eq!(asset_operation_label(SyncAssetDirection::Push), "upload");
        assert_eq!(asset_operation_label(SyncAssetDirection::Pull), "download");
        assert_eq!(asset_operation_label(SyncAssetDirection::Sync), "sync");
    }

    #[test]
    fn sync_asset_truth_table_is_exhaustive_for_force_and_presence() {
        assert_eq!(
            decide_sync_asset_action(false, false, false),
            SyncAssetAction::Missing
        );
        assert_eq!(
            decide_sync_asset_action(false, false, true),
            SyncAssetAction::Missing
        );
        assert_eq!(
            decide_sync_asset_action(true, false, false),
            SyncAssetAction::PushCreate
        );
        assert_eq!(
            decide_sync_asset_action(true, false, true),
            SyncAssetAction::PushCreate
        );
        assert_eq!(
            decide_sync_asset_action(false, true, false),
            SyncAssetAction::Pull
        );
        assert_eq!(
            decide_sync_asset_action(false, true, true),
            SyncAssetAction::Pull
        );
        assert_eq!(
            decide_sync_asset_action(true, true, false),
            SyncAssetAction::Conflict
        );
        assert_eq!(
            decide_sync_asset_action(true, true, true),
            SyncAssetAction::PushOverwrite
        );
    }

    #[test]
    fn sensitive_assets_require_non_empty_operation_passphrase() {
        let sensitive = &SYNC_ASSETS[0];
        assert!(require_asset_passphrase(sensitive, None).is_err());
        assert!(require_asset_passphrase(sensitive, Some(&ccr_core::Secret::from(""))).is_err());
        assert_eq!(
            require_asset_passphrase(sensitive, Some(&ccr_core::Secret::from("sync-pass")))
                .unwrap(),
            "sync-pass"
        );
    }

    /// **Property 4: Routing** — legacy `sync_status` 的目录型目标必须走
    /// PROPFIND（目录）分支。`remote_exists()` 依据 `remote_path.ends_with('/')`
    /// 选择目录/文件分支，因此目录型路径规范化后必须以 `/` 结尾。
    ///
    /// 覆盖无尾斜杠（`/ccr`）与有尾斜杠（`/ccr/`）两种目录型目标。
    #[test]
    fn directory_remote_path_normalizes_directory_targets_with_trailing_slash() {
        // 无尾斜杠目录目标（normalize_remote_base 的产物）→ 补尾斜杠 → 走 PROPFIND。
        assert_eq!(directory_remote_path("/ccr"), "/ccr/");
        // 已有尾斜杠 → 保持不变 → 仍走 PROPFIND。
        assert_eq!(directory_remote_path("/ccr/"), "/ccr/");
        // 嵌套目录路径同样适用。
        assert_eq!(directory_remote_path("/ccr/platforms"), "/ccr/platforms/");
        // 两种形式都必须以 `/` 结尾（路由到目录分支的充要条件）。
        assert!(directory_remote_path("/ccr").ends_with('/'));
        assert!(directory_remote_path("/ccr/").ends_with('/'));
    }

    /// 空路径回退为根目录 `/`（仍是目录型，走 PROPFIND 分支）。
    #[test]
    fn directory_remote_path_empty_falls_back_to_root() {
        assert_eq!(directory_remote_path(""), "/");
        assert_eq!(directory_remote_path("   "), "/");
        assert!(directory_remote_path("").ends_with('/'));
    }

    /// **Property 5: Preservation** — `asset_info` 的远端存在性三态映射保持不变：
    /// `Ok(true)`→present、`Ok(false)`→missing、`Err`→unknown（「未测试」）。
    #[test]
    fn asset_remote_exists_state_maps_three_states() {
        assert_eq!(asset_remote_exists_state::<String>(Ok(true)), Some(true));
        assert_eq!(asset_remote_exists_state::<String>(Ok(false)), Some(false));
        assert_eq!(
            asset_remote_exists_state(Err("check could not be completed".to_string())),
            None
        );
    }
}
