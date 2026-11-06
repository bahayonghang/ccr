//! Captured-root OAuth adapter. Its caller tracks `run` until owner cleanup.
use super::*;
use crate::application::platform_roots::PlatformRoots;
use ccr_codex::services::{CodexOAuthPendingSnapshot, CodexOAuthPendingStore};
use ccr_codex::{
    CodexAuthExportAccount, CodexAuthJson, CodexAuthService, CodexAuthTokens, ImportMode,
};
use ccr_core::{Secret, core::LockManager};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativePhase {
    Waiting,
    Callback,
    Completing,
    Succeeded,
    Cancelled,
    TimedOut,
    Failed,
    CleanupFailed,
}

/// Deliberately no Debug/Serialize: the authorization URL is a scoped secret.
#[derive(Clone)]
pub struct NativeStatus {
    pub login_id: String,
    pub authorization_url: Secret,
    pub phase: NativePhase,
    pub fixture: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeError {
    Busy,
    Invalid,
    Failed,
    CleanupFailed,
    Cancelled,
    TimedOut,
}

#[derive(Default)]
pub struct NativeSession {
    controller: Arc<Controller>,
    status: Arc<Mutex<Option<NativeStatus>>>,
    active: AtomicBool,
    cancel_requested: AtomicBool,
    scope: Mutex<Option<CleanupScope>>,
}

#[derive(Clone, PartialEq, Eq)]
struct CleanupScope {
    path: std::path::PathBuf,
    locks: std::path::PathBuf,
    store: std::path::PathBuf,
    runtime: std::path::PathBuf,
}
impl CleanupScope {
    fn from_roots(roots: &PlatformRoots) -> Self {
        Self {
            path: roots.ccr.join("platforms/codex/oauth_pending.json"),
            locks: roots.locks.clone(),
            store: roots.codex_store.clone(),
            runtime: roots.codex_runtime.clone(),
        }
    }
    fn snapshot(&self) -> Result<CodexOAuthPendingSnapshot, NativeError> {
        LockManager::with_scoped_directory(&self.locks, || {
            CodexOAuthPendingStore::with_path(self.path.clone()).snapshot()
        })
        .map_err(|_| NativeError::Failed)
    }
}
/// Scope/revision consent. No Debug/Serialize and no plaintext in UI projections.
pub struct NativeCleanupPlan {
    scope: CleanupScope,
    login_id: Option<String>,
    snapshot: CodexOAuthPendingSnapshot,
}

impl NativeSession {
    /// Claim before runtime admission so a queued start can still be cancelled.
    pub fn claim(&self) -> Result<(), NativeError> {
        self.active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| NativeError::Busy)?;
        self.cancel_requested.store(false, Ordering::Release);
        *self.scope.lock().map_err(|_| NativeError::Failed)? = None;
        *self.status.lock().map_err(|_| NativeError::Failed)? = None;
        Ok(())
    }
    pub fn abandon_claim(&self) {
        self.active.store(false, Ordering::Release);
    }
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
    pub fn status(&self) -> Option<NativeStatus> {
        self.status.lock().ok()?.clone()
    }
    /// No I/O: navigation can request cancellation directly on the UI thread.
    pub fn request_cancel(&self) {
        self.cancel_requested.store(true, Ordering::Release);
    }
    pub fn submit(&self, id: &str, callback: &str) -> Result<(), NativeError> {
        self.controller
            .submit(id, callback)
            .map_err(|_| NativeError::Invalid)
    }
    pub fn complete(&self, id: &str, name: Option<String>) -> Result<CommitResult, NativeError> {
        if self.cancel_requested.load(Ordering::Acquire) {
            return Err(NativeError::Cancelled);
        }
        if let Some(status) = self
            .status
            .lock()
            .map_err(|_| NativeError::Failed)?
            .as_mut()
        {
            if status.login_id != id || status.phase != NativePhase::Callback {
                return Err(NativeError::Invalid);
            }
            status.phase = NativePhase::Completing;
        }
        tokio::runtime::Handle::current()
            .block_on(self.controller.complete(id, name))
            .map_err(|_| NativeError::Failed)
    }
    pub fn retry_cleanup(&self) -> Result<(), NativeError> {
        tokio::runtime::Handle::current()
            .block_on(self.controller.cancel(None))
            .map_err(|_| NativeError::CleanupFailed)?;
        if let Some(status) = self
            .status
            .lock()
            .map_err(|_| NativeError::Failed)?
            .as_mut()
        {
            status.phase = NativePhase::Cancelled;
            status.authorization_url = "".into();
        }
        Ok(())
    }

    fn cleanup_login(&self, scope: &CleanupScope) -> Result<Option<String>, NativeError> {
        let status = self.status();
        let owned = self.is_active()
            || status
                .as_ref()
                .is_some_and(|status| status.phase == NativePhase::CleanupFailed);
        if !owned {
            return Ok(None);
        }
        if self.scope.lock().map_err(|_| NativeError::Failed)?.as_ref() != Some(scope) {
            return Err(NativeError::Invalid);
        }
        status
            .map(|status| Some(status.login_id))
            .ok_or(NativeError::Busy)
    }
    pub fn prepare_cleanup(&self, roots: &PlatformRoots) -> Result<NativeCleanupPlan, NativeError> {
        let scope = CleanupScope::from_roots(roots);
        let login_id = self.cleanup_login(&scope)?;
        let snapshot = scope.snapshot()?; // Unlike load, preparation never expires/deletes data.
        if login_id
            .as_deref()
            .is_some_and(|id| snapshot.login_id() != Some(id))
        {
            return Err(NativeError::Invalid);
        }
        Ok(NativeCleanupPlan {
            scope,
            login_id,
            snapshot,
        })
    }
    pub fn cancel_captured(&self, plan: &NativeCleanupPlan) -> Result<(), NativeError> {
        let current_login = self.cleanup_login(&plan.scope)?;
        if current_login.is_some() && current_login != plan.login_id {
            return Err(NativeError::Invalid);
        }
        let current = plan.scope.snapshot()?;
        if !current.same_login(&plan.snapshot) {
            return Err(NativeError::Invalid);
        }
        if let Some(id) = current_login {
            // Controller cancellation remains commit-aware and joins its cleanup.
            tokio::runtime::Handle::current()
                .block_on(self.controller.cancel(Some(&id)))
                .map_err(|_| NativeError::CleanupFailed)?;
        }
        let cleared = LockManager::with_scoped_directory(&plan.scope.locks, || {
            CodexOAuthPendingStore::with_path(plan.scope.path.clone())
                .clear_if_snapshot(&plan.snapshot)
        })
        .map_err(|_| NativeError::CleanupFailed)?;
        if !cleared {
            return Err(NativeError::Invalid);
        }
        Ok(())
    }
    /// Runs only on the bounded app worker, never GPUI's render/input thread.
    pub fn run(
        self: &Arc<Self>,
        roots: PlatformRoots,
        cancelled: impl Fn() -> bool,
        changed: impl Fn() + Send + Sync + 'static,
    ) -> Result<(), NativeError> {
        let fixture = roots.fixture;
        if let Ok(mut scope) = self.scope.lock() {
            *scope = Some(CleanupScope::from_roots(&roots));
        } else {
            self.active.store(false, Ordering::Release);
            return Err(NativeError::Failed);
        }
        let backend = Arc::new(CapturedBackend::new(roots));
        let result = tokio::runtime::Handle::current().block_on(self.run_with_backend(
            backend,
            || {
                let mut pending = new_oauth_pending()?;
                if fixture {
                    pending.auth_url =
                        "Fixture: external authorization disabled / 隔离演示：已禁用外部授权"
                            .into();
                }
                Ok(pending)
            },
            fixture,
            cancelled,
            Arc::new(changed),
        ));
        self.active.store(false, Ordering::Release);
        result
    }
    async fn run_with_backend(
        self: &Arc<Self>,
        backend: Arc<dyn Backend>,
        create: impl FnOnce() -> Result<CodexOAuthPendingState, String>,
        fixture: bool,
        cancelled: impl Fn() -> bool,
        changed: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<(), NativeError> {
        if cancelled() || self.cancel_requested.load(Ordering::Acquire) {
            return Err(NativeError::Cancelled);
        }
        let status = self.status.clone();
        let callback_changed = changed.clone();
        let events = Arc::new(move |event| {
            if let Ok(mut state) = status.lock()
                && let Some(state) = state.as_mut()
            {
                match event {
                    Event::Callback { login_id } if state.login_id == login_id => {
                        state.phase = NativePhase::Callback;
                        state.authorization_url = "".into();
                    }
                    _ => {}
                }
            }
            callback_changed();
        });
        let pending = self
            .controller
            .start(backend, events, create, None)
            .map_err(|_| NativeError::Failed)?;
        let id = pending.login_id.clone();
        *self.status.lock().map_err(|_| NativeError::Failed)? = Some(NativeStatus {
            login_id: id.clone(),
            authorization_url: pending.auth_url,
            phase: if pending.callback_url.is_some() {
                NativePhase::Callback
            } else {
                NativePhase::Waiting
            },
            fixture,
        });
        if self
            .controller
            .current()
            .map_err(|_| NativeError::Failed)?
            .is_some_and(|pending| pending.callback_url.is_some())
            && let Some(status) = self
                .status
                .lock()
                .map_err(|_| NativeError::Failed)?
                .as_mut()
        {
            status.phase = NativePhase::Callback;
            status.authorization_url = "".into();
        }
        changed();
        let terminal = self.controller.wait_for_terminal(&id);
        tokio::pin!(terminal);
        let result = loop {
            tokio::select! {
                result = &mut terminal => break result,
                () = tokio::time::sleep(Duration::from_millis(25)) => {
                    if cancelled() || self.cancel_requested.load(Ordering::Acquire) {
                        // A committing owner refuses cancellation; keep joining it.
                        let _ = self.controller.request_cancel(Some(&id));
                    }
                }
            }
        };
        let error = result.err().map(|error| {
            if error.contains("cleanup_failed") {
                NativeError::CleanupFailed
            } else if error.contains("timed_out") {
                NativeError::TimedOut
            } else if error.contains("cancelled") {
                NativeError::Cancelled
            } else {
                NativeError::Failed
            }
        });
        if let Ok(mut status) = self.status.lock()
            && let Some(status) = status.as_mut()
        {
            status.authorization_url = "".into();
            status.phase = match error {
                None => NativePhase::Succeeded,
                Some(NativeError::Cancelled) => NativePhase::Cancelled,
                Some(NativeError::TimedOut) => NativePhase::TimedOut,
                Some(NativeError::CleanupFailed) => NativePhase::CleanupFailed,
                _ => NativePhase::Failed,
            };
        }
        changed();
        error.map_or(Ok(()), Err)
    }
}

struct CapturedBackend {
    roots: PlatformRoots,
    owned: Mutex<Option<CodexOAuthPendingSnapshot>>,
}
impl CapturedBackend {
    fn new(roots: PlatformRoots) -> Self {
        Self {
            roots,
            owned: Mutex::new(None),
        }
    }
    fn store(&self) -> CodexOAuthPendingStore {
        CodexOAuthPendingStore::with_path(self.roots.ccr.join("platforms/codex/oauth_pending.json"))
    }
    fn scoped<T>(&self, work: impl FnOnce() -> ccr_core::Result<T>) -> Result<T, String> {
        LockManager::with_scoped_directory(&self.roots.locks, work)
            .map_err(|_| "oauth_storage_failed".into())
    }
}
impl Backend for CapturedBackend {
    fn now(&self) -> i64 {
        Utc::now().timestamp()
    }
    fn load(&self) -> Result<Option<CodexOAuthPendingState>, String> {
        let (pending, snapshot) = self.scoped(|| self.store().load_owned(self.now()))?;
        *self.owned.lock().map_err(|_| "oauth_storage_failed")? = Some(snapshot);
        Ok(pending)
    }
    fn save(&self, pending: &CodexOAuthPendingState) -> Result<(), String> {
        let snapshot = self.scoped(|| self.store().save_owned(pending))?;
        *self.owned.lock().map_err(|_| "oauth_storage_failed")? = Some(snapshot);
        Ok(())
    }
    fn clear(&self) -> Result<(), String> {
        let snapshot = self
            .owned
            .lock()
            .map_err(|_| "oauth_storage_failed")?
            .clone()
            .ok_or("oauth_storage_unowned")?;
        if !self.scoped(|| self.store().clear_if_snapshot(&snapshot))? {
            return Err("oauth_pending_changed".into());
        }
        Ok(())
    }
    fn bind(&self, port: u16) -> Result<std::net::TcpListener, String> {
        std::net::TcpListener::bind(("127.0.0.1", port))
            .map_err(|_| "oauth_callback_port_unavailable".into())
    }
    fn exchange(&self, pending: CodexOAuthPendingState) -> Work<TokenResponse> {
        let fixture = self.roots.fixture;
        Box::pin(async move {
            // Fixture UI may exercise callback/cancel/timeout, never real exchange.
            if fixture {
                return Err("oauth_fixture_network_disabled".into());
            }
            let callback = pending.callback_url.ok_or("oauth_callback_not_ready")?;
            let url = Url::parse(callback.expose()).map_err(|_| "oauth_invalid_callback")?;
            let code = url
                .query_pairs()
                .find(|(key, _)| key == "code")
                .map(|(_, value)| value.to_string())
                .ok_or("oauth_code_missing")?;
            exchange_oauth_tokens_at(
                OAUTH_TOKEN_URL,
                &code,
                pending.code_verifier.expose(),
                &pending.redirect_uri,
            )
            .await
        })
    }
    fn commit(&self, tokens: TokenResponse, name: Option<String>) -> Work<CommitResult> {
        let roots = self.roots.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                LockManager::with_scoped_directory(&roots.locks, || {
                    let service = roots.codex_auth();
                    let auth = CodexAuthJson {
                        openai_api_key: None,
                        tokens: Some(CodexAuthTokens {
                            id_token: Some(tokens.id_token),
                            access_token: Some(tokens.access_token),
                            refresh_token: tokens.refresh_token,
                            account_id: None,
                        }),
                        last_refresh: Some(Utc::now().to_rfc3339()),
                    };
                    let export = service
                        .build_export_account_from_auth_json(auth, None, None, None, None)?;
                    commit_account(&service, export, name, true)
                        .map_err(ccr_core::CcrError::ConfigError)
                })
                .map_err(|_| "oauth_account_commit_failed".into())
            })
            .await
            .map_err(|_| "oauth_account_worker_failed".to_owned())?
        })
    }
}

/// Shared import/name/switch policy; persistence remains with CodexAuthService.
pub fn commit_account(
    service: &CodexAuthService,
    export: CodexAuthExportAccount,
    preferred_name: Option<String>,
    switch: bool,
) -> Result<CommitResult, String> {
    let explicit = preferred_name
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty());
    let hint = export
        .auth_data
        .as_ref()
        .and_then(|auth| {
            let token = auth.tokens.as_ref()?.id_token.as_ref()?;
            let parts: Vec<_> = token.split('.').collect();
            if parts.len() != 3 {
                return None;
            }
            let payload = ccr_codex::utils::decode_base64url(parts[1])?;
            let payload: serde_json::Value = serde_json::from_slice(&payload).ok()?;
            payload
                .get("email")?
                .as_str()?
                .split('@')
                .next()
                .map(str::to_owned)
        })
        .or_else(|| {
            export
                .api_provider_name
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_owned)
        })
        .or_else(|| {
            let auth = export.auth_data.as_ref()?;
            if auth
                .openai_api_key
                .as_deref()
                .is_some_and(|key| !key.trim().is_empty())
            {
                Some("openai-api".into())
            } else {
                auth.tokens.as_ref()?.account_id.clone()
            }
        });
    let name = if let Some(name) = explicit {
        service.reserve_explicit_account_name(&name)
    } else {
        service.suggest_account_name(hint.as_deref())
    }
    .map_err(|_| "oauth_account_name_invalid")?;
    let payload = serde_json::to_string(&serde_json::json!({
        "version": "1.0", "exported_at": Utc::now().to_rfc3339(), "accounts": {name.clone(): export}
    }))
    .map_err(|_| "oauth_account_payload_invalid")?;
    service
        .import_accounts(&payload, ImportMode::Merge, false)
        .map_err(|_| "oauth_account_import_failed")?;
    if switch {
        service
            .switch_account(&name)
            .map_err(|_| "oauth_account_imported_switch_failed")?;
    }
    Ok(CommitResult::account(name, switch))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn captured_cleanup_plan_preserves_changed_pending_and_owner_scope() {
        let root = tempfile::tempdir().unwrap();
        let roots = PlatformRoots::fixture(root.path()).unwrap();
        let backend = CapturedBackend::new(roots.clone());
        let mut pending = new_oauth_pending().unwrap();
        pending.login_id = "reviewed".into();
        pending.expires_at = 0;
        backend.save(&pending).unwrap();
        let session = NativeSession::default();
        let plan = session.prepare_cleanup(&roots).unwrap();
        let path = CleanupScope::from_roots(&roots).path;
        assert!(path.exists(), "prepare must preserve expired pending state");
        pending.state = "newer-controlled-nonce".into();
        CapturedBackend::new(roots.clone()).save(&pending).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(session.cancel_captured(&plan), Err(NativeError::Invalid));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(
            backend.clear().is_err(),
            "old backend proof cannot remove a later pending nonce"
        );
        session.claim().unwrap();
        *session.scope.lock().unwrap() = Some(CleanupScope::from_roots(&roots));
        *session.status.lock().unwrap() = Some(NativeStatus {
            login_id: "reviewed".into(),
            authorization_url: "".into(),
            phase: NativePhase::Waiting,
            fixture: true,
        });
        let other = tempfile::tempdir().unwrap();
        assert!(matches!(
            session.prepare_cleanup(&PlatformRoots::fixture(other.path()).unwrap()),
            Err(NativeError::Invalid)
        ));
        assert!(other.path().read_dir().unwrap().next().is_none());
        let current = session.prepare_cleanup(&roots).unwrap();
        session.status.lock().unwrap().as_mut().unwrap().login_id = "replacement".into();
        assert_eq!(session.cancel_captured(&current), Err(NativeError::Invalid));
        session.abandon_claim();
        *session.status.lock().unwrap() = None;
        let current = session.prepare_cleanup(&roots).unwrap();
        session.cancel_captured(&current).unwrap();
        assert!(!path.exists());
    }

    struct TestBackend {
        captured: CapturedBackend,
        commits: AtomicBool,
        release_commit: CancellationToken,
        commit_started: tokio::sync::Notify,
        timeout: Duration,
    }
    impl Backend for TestBackend {
        fn now(&self) -> i64 {
            self.captured.now()
        }
        fn remaining(&self, _: i64) -> Duration {
            self.timeout
        }
        fn load(&self) -> Result<Option<CodexOAuthPendingState>, String> {
            self.captured.load()
        }
        fn save(&self, pending: &CodexOAuthPendingState) -> Result<(), String> {
            self.captured.save(pending)
        }
        fn clear(&self) -> Result<(), String> {
            self.captured.clear()
        }
        fn bind(&self, port: u16) -> Result<std::net::TcpListener, String> {
            self.captured.bind(port)
        }
        fn exchange(&self, _: CodexOAuthPendingState) -> Work<TokenResponse> {
            Box::pin(async {
                Ok(TokenResponse {
                    id_token: "synthetic-id".into(),
                    access_token: "synthetic-access".into(),
                    refresh_token: None,
                })
            })
        }
        fn commit(&self, _: TokenResponse, _: Option<String>) -> Work<CommitResult> {
            self.commits.store(true, Ordering::Release);
            self.commit_started.notify_one();
            let release = self.release_commit.clone();
            Box::pin(async move {
                release.cancelled().await;
                Ok(CommitResult::account("synthetic".into(), true))
            })
        }
    }
    fn pending() -> CodexOAuthPendingState {
        let mut pending = new_oauth_pending().unwrap();
        pending.port = std::net::TcpListener::bind(("127.0.0.1", 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        pending.redirect_uri = format!("http://localhost:{}/auth/callback", pending.port);
        pending
    }
    fn backend(root: &std::path::Path, timeout: Duration) -> Arc<TestBackend> {
        Arc::new(TestBackend {
            captured: CapturedBackend::new(PlatformRoots::fixture(root).unwrap()),
            commits: AtomicBool::new(false),
            release_commit: CancellationToken::new(),
            commit_started: tokio::sync::Notify::new(),
            timeout,
        })
    }
    #[tokio::test]
    async fn native_owner_callback_is_not_commit_and_close_joins_started_commit() {
        let temp = tempfile::tempdir().unwrap();
        let backend = backend(temp.path(), Duration::from_secs(3));
        let session = Arc::new(NativeSession::default());
        session.claim().unwrap();
        *session.scope.lock().unwrap() = Some(CleanupScope::from_roots(&backend.captured.roots));
        let cancel = CancellationToken::new();
        let changed = Arc::new(tokio::sync::Notify::new());
        let changed_event = changed.clone();
        let running_session = session.clone();
        let running_backend = backend.clone();
        let running_cancel = cancel.clone();
        let runner = tokio::spawn(async move {
            running_session
                .run_with_backend(
                    running_backend,
                    || Ok(pending()),
                    true,
                    move || running_cancel.is_cancelled(),
                    Arc::new(move || changed_event.notify_one()),
                )
                .await
        });
        changed.notified().await;
        let pending = session.controller.current().unwrap().unwrap();
        let callback = format!(
            "{}?state={}&code=synthetic-code",
            pending.redirect_uri,
            pending.state.expose()
        );
        assert_eq!(
            session.submit("stale-id", &callback),
            Err(NativeError::Invalid)
        );
        session.submit(&pending.login_id, &callback).unwrap();
        assert_eq!(session.status().unwrap().phase, NativePhase::Callback);
        assert!(!backend.commits.load(Ordering::Acquire));
        let cleanup_plan = session.prepare_cleanup(&backend.captured.roots).unwrap();
        assert!(
            !session
                .status()
                .unwrap()
                .authorization_url
                .expose()
                .contains("state=")
        );
        let completing = session.clone();
        let id = pending.login_id.clone();
        let complete =
            tokio::task::spawn_blocking(move || completing.complete(&id, Some("synthetic".into())));
        backend.commit_started.notified().await;
        let refusing = session.clone();
        assert_eq!(
            tokio::task::spawn_blocking(move || refusing.cancel_captured(&cleanup_plan))
                .await
                .unwrap(),
            Err(NativeError::CleanupFailed)
        );
        cancel.cancel();
        tokio::time::sleep(Duration::from_millis(35)).await;
        assert!(!runner.is_finished() && !complete.is_finished());
        backend.release_commit.cancel();
        assert!(complete.await.unwrap().unwrap().success);
        assert_eq!(runner.await.unwrap(), Ok(()));
        assert_eq!(session.status().unwrap().phase, NativePhase::Succeeded);
        assert!(
            session
                .status()
                .unwrap()
                .authorization_url
                .expose()
                .is_empty()
        );
        assert!(backend.load().unwrap().is_none());
        let cleared = session.controller.current().unwrap().unwrap();
        assert!(
            cleared.state.expose().is_empty()
                && cleared.code_verifier.expose().is_empty()
                && cleared.callback_url.is_none()
        );
        assert!(std::net::TcpListener::bind(("127.0.0.1", pending.port)).is_ok());
    }

    #[tokio::test]
    async fn captured_active_cleanup_joins_its_listener_and_pending_owner() {
        let temp = tempfile::tempdir().unwrap();
        let backend = backend(temp.path(), Duration::from_secs(3));
        let session = Arc::new(NativeSession::default());
        session.claim().unwrap();
        *session.scope.lock().unwrap() = Some(CleanupScope::from_roots(&backend.captured.roots));
        let changed = Arc::new(tokio::sync::Notify::new());
        let event = changed.clone();
        let running = session.clone();
        let running_backend = backend.clone();
        let runner = tokio::spawn(async move {
            running
                .run_with_backend(
                    running_backend,
                    || Ok(pending()),
                    true,
                    || false,
                    Arc::new(move || event.notify_one()),
                )
                .await
        });
        changed.notified().await;
        let pending = session.controller.current().unwrap().unwrap();
        let plan = session.prepare_cleanup(&backend.captured.roots).unwrap();
        let cancelling = session.clone();
        assert_eq!(
            tokio::task::spawn_blocking(move || cancelling.cancel_captured(&plan))
                .await
                .unwrap(),
            Ok(())
        );
        assert_eq!(runner.await.unwrap(), Err(NativeError::Cancelled));
        assert!(backend.load().unwrap().is_none());
        assert!(std::net::TcpListener::bind(("127.0.0.1", pending.port)).is_ok());
        session.abandon_claim();
    }
    #[tokio::test]
    async fn native_owner_timeout_cleans_and_fixture_exchange_never_uses_network() {
        let temp = tempfile::tempdir().unwrap();
        let backend = backend(temp.path(), Duration::from_millis(20));
        let session = Arc::new(NativeSession::default());
        assert_eq!(
            session
                .run_with_backend(
                    backend.clone(),
                    || Ok(pending()),
                    true,
                    || false,
                    Arc::new(|| {})
                )
                .await,
            Err(NativeError::TimedOut)
        );
        assert_eq!(session.status().unwrap().phase, NativePhase::TimedOut);
        assert!(backend.load().unwrap().is_none());
        assert!(!backend.commits.load(Ordering::Acquire));
        assert!(
            backend
                .captured
                .exchange(pending())
                .await
                .err()
                .unwrap()
                .contains("fixture_network_disabled")
        );
    }
}
