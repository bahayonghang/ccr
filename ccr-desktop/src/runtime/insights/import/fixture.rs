use super::{ImportOwner, Options};
use crate::runtime::{ConfigSource, insights::InsightsPaths};
use ccr_app::usage::import::{
    JobEvent, SourceKind, SyncSummaryEvent, error::LlmusageAdapterError, events::SourceSyncStats,
};
use std::{io::Read, path::Path, time::Duration};
use tokio_util::sync::CancellationToken;

const MARKER: &str = ".ccr-gpui-usage-import-fixture";
const IDENTITY: &[u8] = b"ccr-gpui-usage-import-fixture-v1";

#[derive(Clone, Copy)]
pub enum Scenario {
    Cancel,
    Partial,
}

impl Scenario {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "cancel" => Some(Self::Cancel),
            "partial" => Some(Self::Partial),
            _ => None,
        }
    }

    /// Validates this opt-in before creating the window. The runner itself consumes
    /// no fixture paths and can only emit fixed events and wait for cancellation.
    pub(crate) fn validate(
        source: &ConfigSource,
        paths: &InsightsPaths,
        checkout: &Path,
    ) -> Result<(), &'static str> {
        const ERROR: &str =
            "Usage import fixture requires a marked, isolated root under outputs/desktop";
        let ConfigSource::Fixture {
            profiles, registry, ..
        } = source
        else {
            return Err(ERROR);
        };
        let allowed = checkout
            .canonicalize()
            .map_err(|_| ERROR)?
            .join("outputs")
            .join("desktop");
        if allowed.canonicalize().map_err(|_| ERROR)? != allowed {
            return Err(ERROR);
        }
        let root = profiles
            .parent()
            .and_then(|root| root.canonicalize().ok())
            .filter(|root| root != &allowed && root.starts_with(&allowed))
            .ok_or(ERROR)?;
        let marker = root.join(MARKER).canonicalize().map_err(|_| ERROR)?;
        if !marker.starts_with(&root) || !marker.is_file() {
            return Err(ERROR);
        }
        let mut identity = Vec::new();
        std::fs::File::open(marker)
            .map_err(|_| ERROR)?
            .take(IDENTITY.len() as u64 + 1)
            .read_to_end(&mut identity)
            .map_err(|_| ERROR)?;
        if identity != IDENTITY {
            return Err(ERROR);
        }
        for path in [
            profiles,
            registry,
            &paths.archive_db,
            &paths.usage_root,
            &paths.provider_home,
        ] {
            if !path.is_absolute() {
                return Err(ERROR);
            }
            let resolved = match path.canonicalize() {
                Ok(resolved) => resolved,
                // Legacy catalog paths and the archive may intentionally be absent.
                Err(e)
                    if e.kind() == std::io::ErrorKind::NotFound
                        && std::fs::symlink_metadata(path)
                            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound) =>
                {
                    path.parent()
                        .and_then(|parent| parent.canonicalize().ok())
                        .zip(path.file_name())
                        .map(|(parent, name)| parent.join(name))
                        .ok_or(ERROR)?
                }
                Err(_) => return Err(ERROR),
            };
            if !resolved.starts_with(&root) {
                return Err(ERROR);
            }
        }
        Ok(())
    }

    pub(super) async fn run(
        self,
        options: Options,
        owner: &ImportOwner,
        cancel: &CancellationToken,
    ) -> Result<SyncSummaryEvent, LlmusageAdapterError> {
        if cancel.is_cancelled() {
            return Err(LlmusageAdapterError::Cancelled);
        }
        let source = options.source.unwrap_or(SourceKind::Codex);
        owner.event(JobEvent::Progress {
            source,
            files_scanned: 1,
            records_imported: 0,
            current_file: None,
        });
        match self {
            Self::Cancel => {
                if tokio::time::timeout(Duration::from_secs(30), cancel.cancelled())
                    .await
                    .is_err()
                {
                    return Err(LlmusageAdapterError::TimedOut);
                }
                // Keep the real owner in CancelRequested during bounded synthetic cleanup.
                tokio::time::sleep(Duration::from_millis(1800)).await;
                Err(LlmusageAdapterError::Cancelled)
            }
            Self::Partial => {
                tokio::select! {
                    () = cancel.cancelled() => return Err(LlmusageAdapterError::Cancelled),
                    () = tokio::time::sleep(Duration::from_millis(600)) => {}
                }
                owner.event(JobEvent::SourceFinished {
                    source,
                    stats: SourceSyncStats {
                        source,
                        files_processed: 2,
                        events_seen: 3,
                        events_inserted: 2,
                        last_error: Some("synthetic source error".into()),
                        ..Default::default()
                    },
                });
                Ok(SyncSummaryEvent {
                    sources: 1,
                    total_seen: 3,
                    total_inserted: 2,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn accepts_only_a_marked_fixture_with_paths_beneath_the_declared_root() {
        let workspace = tempfile::tempdir().unwrap();
        let allowed = workspace.path().join("outputs/desktop");
        let root = allowed.join("usage-fixture");
        fs::create_dir_all(&root).unwrap();
        let source = ConfigSource::Fixture {
            profiles: root.join("profiles.toml"),
            registry: root.join("registry.toml"),
            delay: Duration::ZERO,
        };
        let mut paths = InsightsPaths {
            usage_root: root.join("usage"),
            provider_home: root.join("home"),
            archive_db: root.join("archive.db"),
        };
        fs::create_dir_all(&paths.usage_root).unwrap();
        fs::create_dir_all(&paths.provider_home).unwrap();
        assert!(Scenario::validate(&source, &paths, workspace.path()).is_err());
        fs::write(root.join(MARKER), IDENTITY).unwrap();
        assert!(Scenario::validate(&source, &paths, workspace.path()).is_ok());
        assert!(Scenario::validate(&ConfigSource::Local, &paths, workspace.path()).is_err());
        assert!(Scenario::validate(&source, &paths, &root).is_err());
        paths.archive_db = workspace.path().join("foreign.db");
        assert!(Scenario::validate(&source, &paths, workspace.path()).is_err());
        paths.archive_db = PathBuf::from("relative.db");
        assert!(Scenario::validate(&source, &paths, workspace.path()).is_err());
    }
}
