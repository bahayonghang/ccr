use super::*;
use ccr_sync::sync::clear::{ClearError, ClearSession};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderResult {
    pub name: String,
    pub result: Result<Option<String>, SyncError>,
}

fn legacy(source: &ConfigSource) -> Result<PathBuf, SyncError> {
    match source {
        ConfigSource::Benchmark(layout) => Ok(layout.path("ccr/sync.toml")),
        ConfigSource::Local => ccr_sync::SyncConfigManager::with_default()
            .map(|manager| manager.config_path().to_owned())
            .map_err(|_| SyncError::Storage),
        ConfigSource::Fixture { registry, .. } => registry
            .parent()
            .map(|root| root.join("sync.toml"))
            .ok_or(SyncError::Storage),
    }
}
fn error(error: ClearError) -> SyncError {
    match error {
        ClearError::Conflict => SyncError::Conflict,
        ClearError::Invalid => SyncError::Invalid,
        ClearError::Storage => SyncError::Storage,
        ClearError::LegacyRemaining => SyncError::LegacyRemaining,
    }
}
pub(super) fn prepare(source: &ConfigSource, revision: &str) -> Result<SyncOutcome, SyncError> {
    let canonical = canonical_file(source)?;
    let legacy = legacy(source)?;
    let session = ccr_sync::sync::clear::prepare(&canonical, &legacy).map_err(error)?;
    if !session.matches(&canonical, &legacy, revision) {
        return Err(SyncError::Conflict);
    }
    Ok(SyncOutcome::ClearPrepared(Arc::new(session)))
}
pub(super) fn clear(
    source: &ConfigSource,
    revision: &str,
    session: &ClearSession,
) -> Result<SyncOutcome, SyncError> {
    let canonical = canonical_file(source)?;
    if !session.matches(&canonical, &legacy(source)?, revision) {
        return Err(SyncError::Conflict);
    }
    let run = || {
        session
            .clear()
            .map_err(error)
            .map(|()| SyncOutcome::Cleared)
    };
    if let Some(layout) = source.benchmark() {
        ccr_core::core::LockManager::with_scoped_directory(&layout.roots.locks, || Ok(run()))
            .map_err(|_| SyncError::Storage)?
    } else if source.is_fixture() {
        ccr_core::core::LockManager::with_scoped_directory(
            &canonical.parent().ok_or(SyncError::Storage)?.join("locks"),
            || Ok(run()),
        )
        .map_err(|_| SyncError::Storage)?
    } else {
        run()
    }
}
pub(super) async fn batch(
    source: &ConfigSource,
    revision: &str,
    upload: bool,
    loopback: bool,
) -> Result<SyncOutcome, SyncError> {
    let snapshot = document(source)?
        .read()
        .map_err(|_| SyncError::Unreadable)?;
    if snapshot.token != revision {
        return Err(SyncError::Conflict);
    }
    let config: SyncFoldersConfig =
        toml::from_str(&snapshot.content).map_err(|_| SyncError::Unreadable)?;
    let mut results = Vec::new();
    for (folder, item) in config
        .folders
        .iter()
        .enumerate()
        .filter(|(_, folder)| folder.enabled)
    {
        let operation = if upload {
            Operation::Push {
                revision: revision.into(),
                folder,
            }
        } else {
            Operation::Pull {
                revision: revision.into(),
                folder,
            }
        };
        let result = Box::pin(execute(source, operation, loopback))
            .await
            .and_then(|value| match value {
                SyncOutcome::Pushed => Ok(None),
                SyncOutcome::Pulled { backup } => Ok(backup),
                _ => Err(SyncError::Invalid),
            });
        results.push(FolderResult {
            name: redact_log_text(&item.name),
            result,
        });
    }
    if results.is_empty() {
        return Err(SyncError::Invalid);
    }
    Ok(SyncOutcome::Batch(results))
}
