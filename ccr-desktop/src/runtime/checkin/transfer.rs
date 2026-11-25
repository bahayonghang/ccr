//! A preview belongs to its reader. Only an explicit apply enters writable storage.
use super::*;
use ccr_checkin::ExportManager;
use ccr_checkin::managers::checkin::export_manager::{ExportError, ImportSession};

struct TransferPaths {
    database: PathBuf,
    crypto: PathBuf,
    locks: Option<PathBuf>,
}

impl TransferPaths {
    fn capture(source: &ConfigSource) -> Result<Self, OperationError> {
        Ok(match source {
            ConfigSource::Local => Self {
                database: database::ui_database_path().map_err(|_| OperationError::Storage)?,
                crypto: CheckinService::default_checkin_dir()
                    .map_err(|_| OperationError::Storage)?,
                locks: None,
            },
            ConfigSource::Benchmark(layout) => Self {
                database: layout.path("ui/ccr-ui.db"),
                crypto: layout.path("crypto/checkin"),
                locks: Some(layout.path("locks")),
            },
            ConfigSource::Fixture { registry, .. } => {
                if !registry.is_absolute() {
                    return Err(OperationError::FixturePath);
                }
                let root = registry
                    .parent()
                    .ok_or(OperationError::FixturePath)?
                    .canonicalize()
                    .map_err(|_| OperationError::Storage)?;
                let database = root.join("checkin.db");
                let crypto = root.join("checkin-crypto");
                let locks = root.join("checkin-locks");
                for path in [&database, &crypto, &locks] {
                    reject_links(path)?;
                }
                Self {
                    database,
                    crypto,
                    locks: Some(locks),
                }
            }
        })
    }
}

pub(super) fn existing_export_storage(source: &ConfigSource) -> Result<Storage, OperationError> {
    let paths = TransferPaths::capture(source)?;
    if !paths.database.is_file() {
        return Err(OperationError::Storage);
    }
    let pool = ccr_core::core::sqlite::open_read_only_sqlite_pool(&paths.database)
        .map_err(|_| OperationError::Storage)?;
    Ok(Storage {
        db: DbAccess::Pool(pool),
        crypto: paths.crypto,
        locks: paths.locks,
    })
}

impl Services {
    pub fn checkin_import_preview(
        &self,
        key: RequestKey,
        path: PathBuf,
    ) -> Result<TaskSubscription<Arc<ImportSession>, OperationError>, AdmissionError> {
        let source = self.source.clone();
        let local = EnvironmentPolicy::Independent
            .admit(&key.environment)
            .is_ok();
        self.executor.read(key, move |cancel, _| {
            if !local {
                return Err(OperationError::Unsupported);
            }
            if cancel.is_cancelled() {
                return Err(OperationError::Interrupted);
            }
            extended::validate_transfer_path(&source, &path)?;
            let target = TransferPaths::capture(&source)?;
            ExportManager::preview_file_at(&path, &target.database, &target.crypto)
                .map(Arc::new)
                .map_err(extended::import_error)
        })
    }
}

impl CheckinOperations {
    pub(super) fn commit_transfer(
        &self,
        source: &ConfigSource,
        session: &ImportSession,
        overwrite: bool,
    ) -> Result<ExtendedReport, OperationError> {
        extended::validate_transfer_path(source, session.path())?;
        let target = TransferPaths::capture(source)?;
        let apply = || {
            ExportManager::apply_file_at(
                session,
                &target.database,
                &target.crypto,
                overwrite,
                || {
                    self.storage(source).map(|s| s.db).map_err(|_| {
                        ExportError::ImportFailed("Cannot open captured storage".into())
                    })
                },
            )
        };
        let result = match target.locks {
            Some(ref locks) => {
                ccr_core::core::LockManager::with_scoped_directory(locks, || Ok(apply()))
                    .map_err(|_| OperationError::Storage)?
            }
            None => apply(),
        };
        match result {
            Ok(receipt) => Ok(ExtendedReport::Imported(receipt)),
            Err(ExportError::PartialImport(receipt)) => {
                self.state
                    .lock()
                    .map_err(|_| OperationError::Storage)?
                    .extended = Some(Arc::new(ExtendedReport::Imported(*receipt)));
                Err(OperationError::Storage)
            }
            Err(error) => Err(extended::import_error(error)),
        }
    }
}
