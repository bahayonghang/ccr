//! Desktop-local command favorites/history. The domain owns all SQL and persistence.
use std::sync::{Arc, Mutex};

use ccr_app::runtime::{AdmissionError, Completion, EnvironmentId, RequestKey, TaskSubscription};
use ccr_core::core::log_redact::redact_log_text;
pub use ccr_db::models::ui_state::AddFavoriteRequest;
use ccr_db::{
    database::{self, DbAccess},
    managers::ui_state::UiStateManager,
};

use super::{ConfigSource, Services};

pub(super) type ManagerCache = Arc<Mutex<Option<Arc<UiStateManager>>>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiStateError {
    Storage,
    UnsupportedEnvironment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FavoriteRow {
    pub id: String,
    /// Visible user metadata; application code must never populate it from credentials.
    pub command: String,
    pub display_name: Option<String>,
    pub module: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryRow {
    pub id: String,
    pub command: String,
    pub success: bool,
    pub duration_ms: u64,
    pub executed_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiStateSnapshot {
    pub favorites: Vec<FavoriteRow>,
    pub history: Vec<HistoryRow>,
}

#[derive(Clone)]
pub struct UiStateRequests(Services);

impl UiStateRequests {
    /// Captures explicit paths and a shared lazy cache; performs no I/O.
    pub fn new(services: &Services) -> Self {
        Self(services.clone())
    }

    // Only called by bounded blocking workers. Initialization failures are retryable.
    fn manager(&self) -> Result<Arc<UiStateManager>, UiStateError> {
        let mut cached = self
            .0
            .ui_state_cache
            .lock()
            .map_err(|_| UiStateError::Storage)?;
        if let Some(manager) = cached.as_ref() {
            return Ok(manager.clone());
        }
        let pool = match &self.0.source {
            ConfigSource::Local => match database::get_pool() {
                Some(pool) => pool.clone(),
                None => database::initialize_app_pool().map_err(|_| UiStateError::Storage)?,
            },
            ConfigSource::Fixture { registry, .. } => {
                let root = registry.parent().ok_or(UiStateError::Storage)?;
                let home = root.join("ui-state-migration-home");
                std::fs::create_dir_all(&home).map_err(|_| UiStateError::Storage)?;
                // Fixture migration must never discover legacy personal files.
                if std::fs::read_dir(&home)
                    .map_err(|_| UiStateError::Storage)?
                    .next()
                    .is_some()
                {
                    return Err(UiStateError::Storage);
                }
                let pool = database::create_pool(&root.join("ui-state.db"), None)
                    .map_err(|_| UiStateError::Storage)?;
                {
                    let conn = pool.get().map_err(|_| UiStateError::Storage)?;
                    database::migrations::run_all_migrations(&conn, &home)
                        .map_err(|_| UiStateError::Storage)?;
                }
                pool
            }
        };
        let manager = Arc::new(UiStateManager::with_db(DbAccess::Pool(pool)));
        *cached = Some(manager.clone());
        Ok(manager)
    }

    pub fn load(
        &self,
        key: RequestKey,
    ) -> Result<TaskSubscription<Arc<UiStateSnapshot>, UiStateError>, AdmissionError> {
        let requests = self.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.0.executor.read(key, move |_, _| {
            if !local {
                return Err(UiStateError::UnsupportedEnvironment);
            }
            let manager = requests.manager()?;
            let favorites = manager
                .try_get_favorites()
                .map_err(|_| UiStateError::Storage)?;
            let history = manager
                .try_get_history(None)
                .map_err(|_| UiStateError::Storage)?;
            Ok(Arc::new(UiStateSnapshot {
                favorites: favorites
                    .into_iter()
                    .map(|row| FavoriteRow {
                        id: row.id,
                        command: redact_log_text(&row.command),
                        display_name: row.display_name.map(|text| redact_log_text(&text)),
                        module: redact_log_text(&row.module),
                        created_at: row.created_at.to_rfc3339(),
                    })
                    .collect(),
                history: history
                    .into_iter()
                    .map(|row| HistoryRow {
                        id: row.id,
                        command: redact_log_text(&row.command),
                        success: row.success,
                        duration_ms: row.duration_ms,
                        executed_at: row.executed_at.to_rfc3339(),
                    })
                    .collect(),
            }))
        })
    }

    pub fn add_favorite(
        &self,
        key: RequestKey,
        request: AddFavoriteRequest,
    ) -> Result<TaskSubscription<(), UiStateError>, AdmissionError> {
        self.mutate(key, move |manager| {
            manager.add_favorite(request).map(|_| ())
        })
    }

    pub fn remove_favorite(
        &self,
        key: RequestKey,
        id: String,
    ) -> Result<TaskSubscription<(), UiStateError>, AdmissionError> {
        self.mutate(key, move |manager| manager.remove_favorite(&id).map(|_| ()))
    }

    pub fn clear_history(
        &self,
        key: RequestKey,
    ) -> Result<TaskSubscription<(), UiStateError>, AdmissionError> {
        self.mutate(key, UiStateManager::clear_history)
    }

    fn mutate(
        &self,
        key: RequestKey,
        work: impl FnOnce(&UiStateManager) -> Result<(), String> + Send + 'static,
    ) -> Result<TaskSubscription<(), UiStateError>, AdmissionError> {
        let requests = self.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.0.executor.job(key, move |cancel, _| {
            if !local {
                return Completion::Failed(UiStateError::UnsupportedEnvironment);
            }
            let manager = match requests.manager() {
                Ok(manager) => manager,
                Err(error) => return Completion::Failed(error),
            };
            if cancel.is_cancelled() {
                return Completion::Cancelled;
            }
            // A successful committed mutation remains Success even if cancellation
            // races its return; no replay or fabricated Cancelled result.
            match work(&manager) {
                Ok(()) => Completion::Success(()),
                Err(_) => Completion::Failed(UiStateError::Storage),
            }
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_app::runtime::{Platform, RuntimeOwner, TaskEvent};
    use std::time::Duration;

    fn key() -> RequestKey {
        RequestKey {
            environment: EnvironmentId::Local("local".into()),
            platform: Platform::Claude,
            generation: 0,
            request_id: 1,
        }
    }
    fn fixture(owner: &RuntimeOwner, root: &std::path::Path) -> UiStateRequests {
        UiStateRequests::new(&Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.join("profiles.toml"),
                registry: root.join("config.toml"),
                delay: Duration::ZERO,
            },
        ))
    }

    #[gpui_kit::test]
    async fn ui_state_real_storage_retry_redaction_and_committed_cancellation(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap();
        assert!(database::get_pool().is_none());
        let root = tempfile::TempDir::new().unwrap();
        let requests = fixture(&owner, root.path());
        assert!(!root.path().join("ui-state.db").exists()); // constructor is path-only
        let mut read = requests.load(key()).unwrap();
        assert!(
            matches!(read.next().await, Some(TaskEvent::Terminal(value)) if matches!(value.as_ref(), Completion::Success(snapshot) if snapshot.favorites.is_empty()))
        );
        let (committed_tx, committed_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let mut write = requests
            .mutate(key(), move |manager| {
                manager.add_favorite(AddFavoriteRequest {
                    command: "list".into(),
                    args: vec!["private-argument".into()],
                    display_name: Some("Bearer private-token-value".into()),
                    module: "config".into(),
                })?;
                committed_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            })
            .unwrap();
        committed_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        write.request_cancel();
        release_tx.send(()).unwrap();
        assert!(
            matches!(write.next().await, Some(TaskEvent::Terminal(value)) if matches!(value.as_ref(), Completion::Success(())))
        );
        let mut read = requests.load(key()).unwrap();
        let Some(TaskEvent::Terminal(value)) = read.next().await else {
            panic!("missing terminal")
        };
        let Completion::Success(snapshot) = value.as_ref() else {
            panic!("read failed")
        };
        assert_eq!(snapshot.favorites.len(), 1);
        assert!(!format!("{snapshot:?}").contains("private-argument"));
        assert!(!format!("{snapshot:?}").contains("private-token-value"));
        let mut remote = key();
        remote.environment = EnvironmentId::Ssh("remote".into());
        let mut read = requests.load(remote).unwrap();
        assert!(
            matches!(read.next().await, Some(TaskEvent::Terminal(value)) if matches!(value.as_ref(), Completion::Failed(UiStateError::UnsupportedEnvironment)))
        );
        owner.shutdown();
        assert!(database::get_pool().is_none());
    }

    #[gpui_kit::test]
    async fn ui_state_initialization_error_can_retry_without_home_or_global_pool(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap();
        let root = tempfile::TempDir::new().unwrap();
        let blocked = root.path().join("blocked");
        std::fs::write(&blocked, "fixture").unwrap();
        let requests = fixture(&owner, &blocked);
        let mut read = requests.load(key()).unwrap();
        assert!(
            matches!(read.next().await, Some(TaskEvent::Terminal(value)) if matches!(value.as_ref(), Completion::Failed(UiStateError::Storage)))
        );
        std::fs::remove_file(&blocked).unwrap();
        std::fs::create_dir(&blocked).unwrap();
        let mut read = requests.load(key()).unwrap();
        assert!(
            matches!(read.next().await, Some(TaskEvent::Terminal(value)) if matches!(value.as_ref(), Completion::Success(snapshot) if snapshot.history.is_empty()))
        );
        owner.shutdown();
    }
}
