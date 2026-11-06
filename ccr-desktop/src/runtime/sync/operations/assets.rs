//! Fixed asset transfers use the canonical WebDAV config and domain envelope owner.
use super::*;
pub use ccr_sync::sync::assets::AssetDirection;
use ccr_sync::sync::assets::{AssetRoots, AssetService, SYNC_ASSETS};

pub struct AssetRequest {
    pub revision: String,
    pub id: Option<String>,
    pub direction: AssetDirection,
    pub force: bool,
    pub passphrase: Option<Secret>,
    pub import_plaintext_v1: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetRow {
    pub id: String,
    pub local_path: String,
    pub local_exists: bool,
    pub remote_exists: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetResult {
    pub id: String,
    pub result: Result<Option<String>, SyncError>,
}
fn roots(source: &ConfigSource) -> Result<AssetRoots, SyncError> {
    let roots = super::super::super::resources::roots(source).map_err(|_| SyncError::Storage)?;
    // Fixed asset paths match the established Tauri catalog, including CCR_ROOT.
    Ok(AssetRoots::from_home(&roots.home, roots.ccr))
}
fn service(
    source: &ConfigSource,
    revision: &str,
    loopback: bool,
) -> Result<AssetService, SyncError> {
    let snapshot = document(source)?
        .read()
        .map_err(|_| SyncError::Unreadable)?;
    if snapshot.token != revision {
        return Err(SyncError::Conflict);
    }
    let config: SyncFoldersConfig =
        toml::from_str(&snapshot.content).map_err(|_| SyncError::Unreadable)?;
    let roots = roots(source)?;
    #[cfg(test)]
    if loopback {
        return Ok(AssetService::for_test_loopback(config.webdav, roots));
    }
    let _ = loopback;
    Ok(AssetService::new(config.webdav, roots))
}
fn error(message: String) -> SyncError {
    if message.contains("sync_envelope_passphrase_required") {
        SyncError::AssetPassphrase
    } else if message.contains("sync_envelope_plaintext_v1") {
        SyncError::AssetLegacy
    } else if message.contains("sync_envelope_") {
        SyncError::AssetDecrypt
    } else if message.contains("sync_conflict") {
        SyncError::AssetConflict
    } else {
        SyncError::Operation
    }
}
pub(super) async fn inventory(
    source: &ConfigSource,
    revision: &str,
    probe: bool,
    loopback: bool,
) -> Result<SyncOutcome, SyncError> {
    if probe && (source.is_benchmark() || (source.is_fixture() && !loopback)) {
        return Err(SyncError::FixtureNetwork);
    }
    let service = service(source, revision, loopback)?;
    let mut rows = Vec::new();
    for asset in SYNC_ASSETS {
        let info = service.info(asset.id, probe).await.map_err(error)?;
        rows.push(AssetRow {
            id: asset.id.into(),
            local_path: redact_log_text(&info.local_path.to_string_lossy()),
            local_exists: info.local_exists,
            remote_exists: info.remote_exists,
        });
    }
    Ok(SyncOutcome::AssetInventory(rows))
}
pub(super) async fn transfer(
    source: &ConfigSource,
    request: AssetRequest,
    loopback: bool,
) -> Result<SyncOutcome, SyncError> {
    if source.is_benchmark() || (source.is_fixture() && !loopback) {
        return Err(SyncError::FixtureNetwork);
    }
    if let Some(id) = &request.id {
        ccr_sync::sync::assets::asset_by_id(id).map_err(|_| SyncError::Invalid)?;
    }
    let service = service(source, &request.revision, loopback)?;
    let mut results = Vec::new();
    for asset in SYNC_ASSETS
        .iter()
        .filter(|asset| request.id.as_ref().is_none_or(|id| id == asset.id))
    {
        let result = service
            .execute(
                asset.id,
                request.direction,
                request.force,
                request.passphrase.as_ref(),
                request.import_plaintext_v1,
            )
            .await
            .map(|backup| backup.map(|path| redact_log_text(&path.to_string_lossy())))
            .map_err(error);
        results.push(AssetResult {
            id: asset.id.into(),
            result,
        });
    }
    Ok(SyncOutcome::Assets(results))
}
