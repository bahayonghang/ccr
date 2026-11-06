//! File delivery and recoverable trash use the same retained session writer.
use super::*;
use ccr_codex::{CodexSessionRestoreCandidate, CodexSessionRestorePlan};
use ccr_core::core::guarded_write::{VersionedWriteOutcome, WriteOptions, write_guarded_versioned};

#[derive(Clone, Debug)]
pub struct TrashItem {
    pub session_id: String,
    pub title: String,
    pub deleted_at: String,
    pub relative_path: String,
    pub codex_home: PathBuf,
    candidate: CodexSessionRestoreCandidate,
}

#[derive(Clone, Debug)]
pub struct PreparedRestore {
    pub item: TrashItem,
    plan: CodexSessionRestorePlan,
}

impl InsightsRequests {
    /// Initial native dialog location; never resolves a process HOME fallback.
    pub fn session_save_directory(&self) -> Option<PathBuf> {
        self.paths.as_ref().map(|p| p.provider_home.clone())
    }
    pub fn list_session_trash(
        &self,
        key: RequestKey,
    ) -> Result<TaskSubscription<Arc<Vec<TrashItem>>, Failure>, AdmissionError> {
        let requests = self.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.services.executor.read(key, move |cancel, _| {
            if !local {
                return Err(Failure::LocalOnly);
            }
            requests
                .read_session_trash_cancelled(|| cancel.is_cancelled())
                .map(Arc::new)
        })
    }
    #[cfg(test)]
    pub(super) fn read_session_trash(&self) -> Result<Vec<TrashItem>, Failure> {
        self.read_session_trash_cancelled(|| false)
    }
    pub(super) fn read_session_trash_cancelled(
        &self,
        cancelled: impl Fn() -> bool,
    ) -> Result<Vec<TrashItem>, Failure> {
        let home = self
            .paths
            .as_ref()
            .ok_or(Failure::Storage)?
            .provider_home
            .join(".codex");
        CodexSessionTrashService::with_codex_home(&home)
            .list_restore_candidates(cancelled)
            .map_err(|_| Failure::Source)
            .and_then(|items| items.ok_or(Failure::Interrupted))
            .map(|candidates| {
                candidates
                    .into_iter()
                    .map(|candidate| TrashItem {
                        session_id: candidate.record().session_id.clone(),
                        title: candidate.record().title.chars().take(512).collect(),
                        deleted_at: candidate.record().deleted_at.to_rfc3339(),
                        relative_path: candidate.record().original_relative_path.clone(),
                        codex_home: home.clone(),
                        candidate,
                    })
                    .collect()
            })
    }
    pub fn prepare_session_restore(
        &self,
        key: RequestKey,
        item: TrashItem,
    ) -> Result<TaskSubscription<Arc<PreparedRestore>, Failure>, AdmissionError> {
        let requests = self.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.services.executor.read(key, move |cancel, _| {
            if !local {
                return Err(Failure::LocalOnly);
            }
            requests.prepare_session_restore_item(item, || cancel.is_cancelled())
        })
    }
    pub(super) fn prepare_session_restore_item(
        &self,
        item: TrashItem,
        cancelled: impl Fn() -> bool,
    ) -> Result<Arc<PreparedRestore>, Failure> {
        let paths = self.paths.as_ref().ok_or(Failure::Storage)?;
        if item.codex_home != paths.provider_home.join(".codex") {
            return Err(Failure::Conflict);
        }
        let plan = CodexSessionTrashService::with_codex_home(&item.codex_home)
            .prepare_restore(&item.candidate, &cancelled)
            .map_err(|_| Failure::Source)?;
        if cancelled() {
            return Err(Failure::Interrupted);
        }
        let plan = plan.ok_or(Failure::Conflict)?;
        Ok(Arc::new(PreparedRestore { item, plan }))
    }
    pub fn start_session_restore(
        &self,
        key: RequestKey,
        prepared: Arc<PreparedRestore>,
    ) -> Result<(), Failure> {
        let requests = self.clone();
        self.start_session_work(key, move || requests.restore_session(&prepared))
    }
    pub(super) fn restore_session(&self, prepared: &PreparedRestore) -> Result<Outcome, Failure> {
        let item = &prepared.item;
        let paths = self.paths.as_ref().ok_or(Failure::Storage)?;
        if item.codex_home != paths.provider_home.join(".codex") {
            return Err(Failure::Conflict);
        }
        self.with_session_locks(|| {
            if !CodexSessionTrashService::with_codex_home(&item.codex_home)
                .restore_versioned(&prepared.plan)
                .map_err(|_| Failure::Partial)?
            {
                return Err(Failure::Conflict);
            }
            Ok(())
        })?;
        let archive_updated = self.archive_pool(paths).ok().is_some_and(|pool| {
            let registry = AgentSessionProviderRegistry::new(paths.provider_home.clone())
                .without_environment_overrides();
            sessions::refresh::refresh_archive_with_registry(&pool, &registry)
                .is_ok_and(|report| report.total.errors == 0)
        });
        Ok(Outcome {
            archive_id: String::new(),
            action: Action::Restore,
            session_id: item.session_id.clone(),
            archive_updated,
            codex_home: item.codex_home.clone(),
        })
    }
    pub fn start_session_save(
        &self,
        key: RequestKey,
        archive_id: String,
        export: Arc<CodexSessionExport>,
        destination: PathBuf,
    ) -> Result<(), Failure> {
        let requests = self.clone();
        self.start_session_work(key, move || {
            requests.save_session_export(&archive_id, &export, &destination)
        })
    }
    pub(super) fn save_session_export(
        &self,
        archive_id: &str,
        export: &CodexSessionExport,
        destination: &std::path::Path,
    ) -> Result<Outcome, Failure> {
        // The preview is explicit view-owned data. The retained writer holds it
        // only until completion; terminal state never contains transcript bytes.
        let paths = self.paths.as_ref().ok_or(Failure::Storage)?;
        if !destination.is_absolute()
            || destination
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err(Failure::Storage);
        }
        reject_destination_links(destination)?;
        if matches!(
            self.services.source,
            super::super::super::ConfigSource::Fixture { .. }
        ) {
            let root = paths.archive_db.parent().ok_or(Failure::Storage)?;
            let root = root.canonicalize().map_err(|_| Failure::Storage)?;
            let parent = destination
                .parent()
                .ok_or(Failure::Storage)?
                .canonicalize()
                .map_err(|_| Failure::Storage)?;
            if !parent.starts_with(&root) {
                return Err(Failure::Storage);
            }
        }
        self.with_session_locks(|| {
            reject_destination_links(destination)?;
            match write_guarded_versioned(
                destination,
                export.content.as_bytes(),
                "",
                &WriteOptions {
                    secret: true,
                    ..Default::default()
                },
            )
            .map_err(|_| Failure::Storage)?
            {
                VersionedWriteOutcome::Conflict => Err(Failure::Conflict),
                VersionedWriteOutcome::Written => Ok(()),
            }
        })?;
        Ok(Outcome {
            archive_id: archive_id.into(),
            action: Action::Save,
            session_id: export.session_id.clone(),
            archive_updated: true,
            codex_home: paths.provider_home.join(".codex"),
        })
    }
    fn with_session_locks<T>(
        &self,
        run: impl FnOnce() -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        if matches!(
            self.services.source,
            super::super::super::ConfigSource::Fixture { .. }
        ) {
            let root = self
                .paths
                .as_ref()
                .and_then(|p| p.archive_db.parent())
                .ok_or(Failure::Storage)?;
            LockManager::with_scoped_directory(&root.join("locks"), || Ok(run()))
                .map_err(|_| Failure::Storage)?
        } else {
            run()
        }
    }
}

fn reject_destination_links(path: &std::path::Path) -> Result<(), Failure> {
    for ancestor in path.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                let linked = metadata.file_type().is_symlink();
                #[cfg(windows)]
                let linked = {
                    use std::os::windows::fs::MetadataExt;
                    linked || metadata.file_attributes() & 0x400 != 0
                };
                if linked {
                    return Err(Failure::Storage);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err(Failure::Storage),
        }
    }
    Ok(())
}
