//! Route captured SSH assets through the existing sync protocol/envelope owner.
use super::*;
use ccr_app::environments::{
    config_mirror::{ConfigMirror, validate_materialized_paths},
    io::{EntryKind, EnvironmentIo, EnvironmentPath, Error},
};
use ccr_sync::{
    SyncConfig, SyncService,
    sync::assets::{AssetRoots, AssetService, SYNC_ASSETS},
};
use std::collections::BTreeSet;
use tokio_util::sync::CancellationToken;

const CONFIG: &str = ".ccr/sync_folders.toml";
fn error(error: Error) -> SyncError {
    match error {
        Error::Conflict => SyncError::Conflict,
        Error::Partial => SyncError::RemotePartial,
        _ => SyncError::Remote,
    }
}
fn snapshot(content: &str, exists: bool) -> Result<SyncSnapshot, SyncError> {
    if !exists {
        return Ok(SyncSnapshot::Absent);
    }
    let config: SyncFoldersConfig = toml::from_str(content).map_err(|_| SyncError::Unreadable)?;
    let mut local = project(
        config,
        ccr_core::core::content_version_token(content.as_bytes()),
    );
    if local.folders.is_empty() {
        local.folders = legacy_folders(content)?;
    }
    Ok(SyncSnapshot::Present(local))
}
pub(super) async fn read(
    io: &EnvironmentIo,
    cancel: CancellationToken,
) -> Result<SyncSnapshot, SyncError> {
    let content = io.read(CONFIG, cancel).await.map_err(error)?;
    snapshot(&content.content, content.exists)
}
pub(super) fn relative(io: &EnvironmentIo, path: &str) -> Result<String, SyncError> {
    let EnvironmentPath::Ssh(home) = io.roots().home() else {
        return Err(SyncError::Remote);
    };
    let path = path
        .strip_prefix("~/")
        .or_else(|| path.strip_prefix(&format!("{}/", home.as_str().trim_end_matches('/'))))
        .unwrap_or(path)
        .trim_end_matches('/');
    if path.is_empty() || path.starts_with('/') {
        return Err(SyncError::Invalid);
    }
    validate_materialized_paths([path]).map_err(error)?;
    io.roots().relative(path).map_err(error)?;
    Ok(path.into())
}
fn sqlite(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
    name.ends_with(".db")
        || name.ends_with(".sqlite")
        || name.ends_with(".sqlite3")
        || name.ends_with("-wal")
        || name.ends_with("-shm")
        || name.ends_with("-journal")
}
async fn paths(
    io: &EnvironmentIo,
    root: &str,
    cancel: CancellationToken,
) -> Result<(BTreeSet<String>, BTreeSet<String>), SyncError> {
    let mut files = BTreeSet::new();
    let mut directories = BTreeSet::new();
    let mut pending = vec![(root.to_string(), 0)];
    let mut bytes = 0u64;
    while let Some((path, depth)) = pending.pop() {
        if depth > 12 || files.len() + directories.len() + pending.len() > 128 {
            return Err(SyncError::Invalid);
        }
        validate_materialized_paths([path.as_str()]).map_err(error)?;
        if sqlite(&path) {
            return Err(SyncError::RemoteDatabase);
        }
        if io
            .is_directory(&path, cancel.clone())
            .await
            .map_err(error)?
        {
            directories.insert(path.clone());
            let entries = io
                .entries(&path, cancel.clone())
                .await
                .map_err(error)?
                .ok_or(SyncError::Conflict)?;
            for entry in entries {
                if entry.kind == EntryKind::Unsupported {
                    return Err(SyncError::Invalid);
                }
                pending.push((format!("{path}/{}", entry.name), depth + 1));
            }
        } else {
            if let Some((size, _)) = io.metadata(&path, cancel.clone()).await.map_err(error)? {
                bytes = bytes.saturating_add(size);
                if size > 2 * 1024 * 1024 || bytes > 16 * 1024 * 1024 {
                    return Err(SyncError::Invalid);
                }
            }
            files.insert(path);
        }
    }
    Ok((files, directories))
}
fn outputs(
    root: &Path,
    relative: &str,
    existing_directories: &BTreeSet<String>,
) -> Result<BTreeSet<String>, SyncError> {
    let mut files = BTreeSet::new();
    let mut pending = vec![(relative.to_string(), 0)];
    let mut total = 0;
    let mut nodes = 0;
    while let Some((path, depth)) = pending.pop() {
        nodes += 1;
        if depth > 12 || nodes + pending.len() > 128 {
            return Err(SyncError::Invalid);
        }
        validate_materialized_paths([path.as_str()]).map_err(error)?;
        if sqlite(&path) {
            return Err(SyncError::RemoteDatabase);
        }
        let metadata = match std::fs::symlink_metadata(root.join(&path)) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => return Err(SyncError::Storage),
        };
        if metadata.is_symlink() {
            return Err(SyncError::Invalid);
        }
        if metadata.is_dir() {
            let mut count = 0;
            for entry in std::fs::read_dir(root.join(&path)).map_err(|_| SyncError::Storage)? {
                count += 1;
                let entry = entry.map_err(|_| SyncError::Storage)?;
                pending.push((
                    format!(
                        "{path}/{}",
                        entry.file_name().to_str().ok_or(SyncError::Invalid)?
                    ),
                    depth + 1,
                ));
            }
            if count == 0 && !existing_directories.contains(&path) {
                return Err(SyncError::RemoteEmptyDirectory);
            }
        } else if metadata.is_file() {
            total += metadata.len();
            if metadata.len() > 2 * 1024 * 1024 || total > 16 * 1024 * 1024 {
                return Err(SyncError::Invalid);
            }
            files.insert(path);
        } else {
            return Err(SyncError::Invalid);
        }
    }
    Ok(files)
}
fn verify_revision(mirror: &ConfigMirror, revision: &str) -> Result<(), SyncError> {
    let content = std::fs::read(mirror.root().join(CONFIG)).unwrap_or_default();
    if ccr_core::core::content_version_token(&content) != revision {
        return Err(SyncError::Conflict);
    }
    Ok(())
}
fn asset_service(config: &SyncFoldersConfig, root: &Path, loopback: bool) -> AssetService {
    let roots = AssetRoots::from_home(root, root.join(".ccr"));
    #[cfg(test)]
    if loopback {
        return AssetService::for_test_loopback(config.webdav.clone(), roots);
    }
    let _ = loopback;
    AssetService::new(config.webdav.clone(), roots)
}
async fn folder_service(
    config: &SyncFoldersConfig,
    remote_path: String,
    loopback: bool,
) -> Result<SyncService, SyncError> {
    let webdav = &config.webdav;
    let config = SyncConfig {
        enabled: webdav.enabled,
        webdav_url: webdav.url.clone(),
        username: webdav.username.clone(),
        password: webdav.password.clone(),
        remote_path,
        auto_sync: false,
    };
    #[cfg(test)]
    if loopback {
        return SyncService::new_for_test_loopback(&config)
            .await
            .map_err(|_| SyncError::Invalid);
    }
    let _ = loopback;
    SyncService::new(&config)
        .await
        .map_err(|_| SyncError::Invalid)
}
async fn asset_inventory(
    io: &EnvironmentIo,
    service: Option<&AssetService>,
    cancel: CancellationToken,
) -> Result<SyncOutcome, SyncError> {
    let mut rows = Vec::with_capacity(SYNC_ASSETS.len());
    for asset in SYNC_ASSETS {
        let path = relative(io, asset.local_path)?;
        let local_exists = if asset.kind == ccr_sync::sync::assets::SyncAssetKind::Directory {
            io.is_directory(&path, cancel.clone()).await.ok()
        } else {
            io.metadata(&path, cancel.clone())
                .await
                .ok()
                .map(|entry| entry.is_some())
        };
        let remote_exists = if let Some(service) = service {
            io.check_authority().map_err(error)?;
            service.probe_remote(asset.id).await.ok()
        } else {
            None
        };
        let EnvironmentPath::Ssh(full) = io.roots().relative(&path).map_err(error)? else {
            return Err(SyncError::Remote);
        };
        rows.push(AssetRow {
            id: asset.id.into(),
            local_path: Some(redact_log_text(full.as_str())),
            local_exists,
            remote_exists,
        });
    }
    io.check_authority().map_err(error)?;
    super::operations::inventory_outcome(rows, service.is_some())
}

pub(super) async fn execute(
    source: &ConfigSource,
    io: EnvironmentIo,
    operation: Operation,
    loopback: bool,
    cancel: CancellationToken,
) -> Result<SyncOutcome, SyncError> {
    if matches!(operation, Operation::AssetInventory { probe: false, .. }) {
        return asset_inventory(&io, None, cancel).await;
    }
    let config = io.read(CONFIG, cancel.clone()).await.map_err(error)?;
    let revision = config.version();
    let expected = match &operation {
        Operation::Save(patch) => &patch.revision,
        Operation::Folder { revision, .. }
        | Operation::Test { revision }
        | Operation::Push { revision, .. }
        | Operation::Pull { revision, .. }
        | Operation::Batch { revision, .. }
        | Operation::AssetInventory { revision, .. } => revision,
        Operation::Assets(request) => &request.revision,
        Operation::PrepareClear { .. } | Operation::Clear { .. } => {
            return Err(SyncError::Unsupported);
        }
    };
    if expected != revision {
        return Err(SyncError::Conflict);
    }
    if matches!(operation, Operation::Save(_) | Operation::Folder { .. }) {
        let mirror =
            ConfigMirror::capture(io, BTreeSet::from([CONFIG.into()]), None, cancel.clone())
                .await
                .map_err(error)?;
        let source = ConfigSource::Fixture {
            profiles: mirror.root().join(".ccr/profiles.toml"),
            registry: mirror.root().join(".ccr/registry.toml"),
            delay: std::time::Duration::ZERO,
        };
        let outcome = match operation {
            Operation::Folder { revision, change } => {
                super::operations::save_folder_for_environment(
                    &source,
                    &revision,
                    change,
                    Some(mirror.io()),
                )?
            }
            operation => Box::pin(super::operations::execute(&source, operation, false)).await?,
        };
        mirror.publish(cancel).await.map_err(error)?;
        return Ok(outcome);
    }
    let config: SyncFoldersConfig =
        toml::from_str(&config.content).map_err(|_| SyncError::Unreadable)?;
    let network = !source.is_benchmark() && (!source.is_fixture() || loopback);
    if matches!(operation, Operation::AssetInventory { probe: true, .. }) {
        if !network {
            return Err(SyncError::FixtureNetwork);
        }
        let stage = ConfigMirror::capture(
            io.clone(),
            BTreeSet::from([CONFIG.into()]),
            None,
            cancel.clone(),
        )
        .await
        .map_err(error)?;
        verify_revision(&stage, revision)?;
        let service = asset_service(&config, stage.root(), loopback);
        let outcome = asset_inventory(&io, Some(&service), cancel.clone()).await;
        if io.read(CONFIG, cancel).await.map_err(error)?.version() != revision {
            return Err(SyncError::Conflict);
        }
        return outcome;
    }
    if !network {
        return Err(SyncError::FixtureNetwork);
    }
    if matches!(operation, Operation::Test { .. }) {
        folder_service(&config, config.webdav.base_remote_path.clone(), loopback)
            .await?
            .test_connection()
            .await
            .map_err(|_| SyncError::Operation)?;
        io.check_authority().map_err(error)?;
        return Ok(SyncOutcome::Connected);
    }
    if let Operation::Assets(request) = operation {
        if let Some(id) = &request.id {
            ccr_sync::sync::assets::asset_by_id(id).map_err(|_| SyncError::Invalid)?;
        }
        let mut results = Vec::new();
        for asset in SYNC_ASSETS
            .iter()
            .filter(|asset| request.id.as_ref().is_none_or(|id| id == asset.id))
        {
            let result = async {
                if asset.id == "ccr-platforms" {
                    return Err(SyncError::RemoteDatabase);
                }
                let path = relative(&io, asset.local_path)?;
                let mirror = ConfigMirror::capture(
                    io.clone(),
                    BTreeSet::from([CONFIG.into(), path.clone()]),
                    None,
                    cancel.clone(),
                )
                .await
                .map_err(error)?;
                verify_revision(&mirror, revision)?;
                mirror.verify(cancel.clone()).await.map_err(error)?;
                asset_service(&config, mirror.root(), loopback)
                    .execute(
                        asset.id,
                        request.direction,
                        request.force,
                        request.passphrase.as_ref(),
                        request.import_plaintext_v1,
                    )
                    .await
                    .map_err(super::operations::asset_error)?;
                // WebDAV 的临时备份不替代 SSH 主机上的 CAS 前像备份。
                mirror
                    .publish_outputs(
                        outputs(mirror.root(), &path, &BTreeSet::new())?,
                        cancel.clone(),
                    )
                    .await
                    .map_err(error)?;
                Ok(None)
            }
            .await;
            results.push(super::operations::AssetResult {
                id: asset.id.into(),
                result,
            });
        }
        return Ok(SyncOutcome::Assets(results));
    }
    let (selected, upload, batch) = match operation {
        Operation::Push { folder, .. } => (Some(folder), true, false),
        Operation::Pull { folder, .. } => (Some(folder), false, false),
        Operation::Batch { upload, .. } => (None, upload, true),
        _ => return Err(SyncError::Invalid),
    };
    let mut results = Vec::new();
    for (index, folder) in config.folders.iter().enumerate().filter(|(index, folder)| {
        selected.is_none_or(|selected| selected == *index) && folder.enabled
    }) {
        let _ = index;
        let result = async {
            let path = relative(&io, &folder.local_path)?;
            let (mut inputs, directories) = paths(&io, &path, cancel.clone()).await?;
            inputs.insert(CONFIG.into());
            let mirror = ConfigMirror::capture(io.clone(), inputs, None, cancel.clone())
                .await
                .map_err(error)?;
            verify_revision(&mirror, revision)?;
            for directory in &directories {
                let directory = mirror.root().join(directory);
                std::fs::create_dir_all(&directory).map_err(|_| SyncError::Storage)?;
                ccr_core::core::guarded_write::restrict_owner_only_directory(&directory)
                    .map_err(|_| SyncError::Storage)?;
            }
            let local = mirror.root().join(&path);
            let mut remote_path = folder.remote_path.clone();
            if local.is_dir() && !remote_path.ends_with('/') {
                remote_path.push('/');
            }
            let service = folder_service(&config, remote_path, loopback).await?;
            mirror.verify(cancel.clone()).await.map_err(error)?;
            if upload {
                service
                    .push(&local, None)
                    .await
                    .map_err(|_| SyncError::Operation)?;
            } else {
                service
                    .pull_with_backup(&local)
                    .await
                    .map_err(|_| SyncError::Operation)?;
                mirror
                    .publish_outputs(outputs(mirror.root(), &path, &directories)?, cancel.clone())
                    .await
                    .map_err(error)?;
            }
            io.check_authority().map_err(error)?;
            Ok(None)
        }
        .await;
        results.push(super::operations::FolderResult {
            name: redact_log_text(&folder.name),
            result,
        });
    }
    if batch {
        return Ok(SyncOutcome::Batch(results));
    }
    match results.pop().ok_or(SyncError::Invalid)?.result? {
        _ if upload => Ok(SyncOutcome::Pushed),
        backup => Ok(SyncOutcome::Pulled { backup }),
    }
}
