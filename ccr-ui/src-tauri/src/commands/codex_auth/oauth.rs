//! One owner for OAuth admission, callback I/O, exchange, and secret cleanup.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{OwnedSemaphorePermit, mpsc, oneshot, watch};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use super::{CodexAuthMutationResponse, CodexOAuthPendingState, CodexOAuthTokenResponse};

type Work<T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send>>;
pub(super) type Events = Arc<dyn Fn(Event) + Send + Sync>;

pub(super) trait Backend: Send + Sync {
    fn now(&self) -> i64;
    fn remaining(&self, expires_at: i64) -> Duration {
        Duration::from_secs((expires_at - self.now()).max(1) as u64)
    }
    fn load(&self) -> Result<Option<CodexOAuthPendingState>, String>;
    fn save(&self, pending: &CodexOAuthPendingState) -> Result<(), String>;
    fn clear(&self) -> Result<(), String>;
    fn bind(&self, port: u16) -> Result<std::net::TcpListener, String>;
    fn exchange(&self, pending: CodexOAuthPendingState) -> Work<CodexOAuthTokenResponse>;
    fn commit(
        &self,
        tokens: CodexOAuthTokenResponse,
        name: Option<String>,
    ) -> Work<CodexAuthMutationResponse>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Outcome {
    Cancelled,
    TimedOut,
    Failed(String),
    CleanupFailed(String),
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => f.write_str("oauth_cancelled"),
            Self::TimedOut => f.write_str("oauth_timed_out"),
            Self::Failed(message) => write!(f, "oauth_failed: {message}"),
            Self::CleanupFailed(message) => write!(f, "oauth_cleanup_failed: {message}"),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum Event {
    Callback {
        login_id: String,
    },
    Timeout {
        login_id: String,
        callback_url: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Waiting,
    Callback,
    Exchanging,
    Committing,
    Terminal,
}

struct LoginState {
    pending: CodexOAuthPendingState,
    phase: Phase,
}

struct Completion {
    name: Option<String>,
    reply: oneshot::Sender<Result<CodexAuthMutationResponse, Outcome>>,
}

struct Login {
    id: String,
    state: Mutex<LoginState>,
    cancel: CancellationToken,
    complete: mpsc::Sender<Completion>,
    done: watch::Receiver<Option<Result<(), Outcome>>>,
    deadline: Instant,
    backend: Arc<dyn Backend>,
    events: Events,
    cleanup_retrying: AtomicBool,
}

#[derive(Default)]
pub(super) struct Controller {
    slot: Mutex<Option<Arc<Login>>>,
    startup: tokio::sync::Mutex<()>,
}

impl Controller {
    pub(super) async fn restore_admitted(
        &self,
        backend: Arc<dyn Backend>,
        events: Events,
        admission: impl Future<Output = Result<OwnedSemaphorePermit, String>>,
    ) -> Result<Option<CodexOAuthPendingState>, String> {
        if backend.load()?.is_none() {
            return Ok(None);
        }
        // Admission may wait while a control clears the persisted login.
        // start re-reads storage; restore must never recreate the old snapshot.
        self.start_admitted(
            backend,
            events,
            || Err("oauth_saved_login_missing".into()),
            admission,
        )
        .await
        .map(Some)
    }

    pub(super) async fn start_admitted(
        &self,
        backend: Arc<dyn Backend>,
        events: Events,
        create: impl FnOnce() -> Result<CodexOAuthPendingState, String>,
        admission: impl Future<Output = Result<OwnedSemaphorePermit, String>>,
    ) -> Result<CodexOAuthPendingState, String> {
        // Only starts use this lock. Control requests never wait for admission.
        let _startup = self.startup.lock().await;
        {
            let slot = self.slot.lock().map_err(|_| "oauth_slot_poisoned")?;
            if let Some(current) = slot.as_ref() {
                match current.done.borrow().as_ref() {
                    None => {
                        let state = current.state.lock().map_err(|_| "oauth_state_poisoned")?;
                        current.validate_active(&state)?;
                        return Ok(state.pending.clone());
                    }
                    Some(Err(Outcome::CleanupFailed(_))) => {
                        return Err("oauth_cleanup_failed".into());
                    }
                    Some(_) => {}
                }
            }
        }
        let admission = admission.await?;
        self.start(backend, events, create, Some(admission))
    }
    #[cfg(test)]
    pub(super) fn current(&self) -> Result<Option<CodexOAuthPendingState>, String> {
        self.login(None)?.map(|login| login.pending()).transpose()
    }
    /// Bind and save before publishing the ID. The runner owns the exact
    /// admission permit acquired by the command wrapper.
    pub(super) fn start(
        &self,
        backend: Arc<dyn Backend>,
        events: Events,
        create: impl FnOnce() -> Result<CodexOAuthPendingState, String>,
        admission: Option<OwnedSemaphorePermit>,
    ) -> Result<CodexOAuthPendingState, String> {
        let mut slot = self.slot.lock().map_err(|_| "oauth_slot_poisoned")?;
        if let Some(current) = slot.as_ref() {
            match current.done.borrow().as_ref() {
                None if !current.cancel.is_cancelled() => {
                    let state = current.state.lock().map_err(|_| "oauth_state_poisoned")?;
                    current.validate_active(&state)?;
                    return Ok(state.pending.clone());
                }
                None => return Err("oauth_cleanup_in_progress".into()),
                Some(Err(Outcome::CleanupFailed(_))) => return Err("oauth_cleanup_failed".into()),
                Some(_) => {}
            }
        }
        let pending = match backend.load()? {
            Some(pending) if pending.expires_at > backend.now() => pending,
            _ => create()?,
        };
        if pending.expires_at <= backend.now() {
            return Err("oauth_expired".into());
        }
        if let Some(callback) = pending.callback_url.as_ref() {
            validate_callback(&pending, callback.expose())?;
        }
        let listener = backend.bind(pending.port)?;
        listener
            .set_nonblocking(true)
            .map_err(|_| "oauth_listener_nonblocking_failed")?;
        let listener =
            TcpListener::from_std(listener).map_err(|_| "oauth_listener_runtime_failed")?;
        backend.save(&pending)?;
        let (complete, requests) = mpsc::channel(1);
        let (done, finished) = watch::channel(None);
        let login = Arc::new(Login {
            id: pending.login_id.clone(),
            deadline: Instant::now() + backend.remaining(pending.expires_at),
            state: Mutex::new(LoginState {
                phase: if pending.callback_url.is_some() {
                    Phase::Callback
                } else {
                    Phase::Waiting
                },
                pending: pending.clone(),
            }),
            cancel: CancellationToken::new(),
            complete,
            done: finished,
            backend,
            events,
            cleanup_retrying: AtomicBool::new(false),
        });
        *slot = Some(Arc::clone(&login));
        tokio::spawn(login.run(listener, requests, done, admission));
        Ok(pending)
    }

    fn login(&self, id: Option<&str>) -> Result<Option<Arc<Login>>, String> {
        let slot = self.slot.lock().map_err(|_| "oauth_slot_poisoned")?;
        let Some(login) = slot.as_ref() else {
            return Ok(None);
        };
        if id.is_some_and(|id| id != login.id) {
            return Err("oauth_login_id_changed".into());
        }
        Ok(Some(Arc::clone(login)))
    }

    pub(super) fn submit(&self, id: &str, url: &str) -> Result<(), String> {
        self.login(Some(id))?
            .ok_or("oauth_not_running")?
            .submit(url)
    }

    pub(super) async fn complete(
        &self,
        id: &str,
        name: Option<String>,
    ) -> Result<CodexAuthMutationResponse, String> {
        let login = self.login(Some(id))?.ok_or("oauth_not_running")?;
        let (reply, result) = oneshot::channel();
        {
            let mut state = login.state.lock().map_err(|_| "oauth_state_poisoned")?;
            login.validate_active(&state)?;
            if state.phase != Phase::Callback {
                return Err("oauth_callback_not_ready".into());
            }
            login
                .complete
                .try_send(Completion { name, reply })
                .map_err(|_| "oauth_completion_unavailable")?;
            state.phase = Phase::Exchanging;
        }
        result
            .await
            .map_err(|_| "oauth_completion_unavailable".to_string())?
            .map_err(|e| e.to_string())
    }

    /// Acknowledgement comes from the owner only after listener and secret
    /// cleanup. A commit that already started is completion-aware.
    #[cfg(test)]
    pub(super) async fn cancel(&self, id: Option<&str>) -> Result<(), String> {
        let Some(login) = self.login(id)? else {
            return Ok(());
        };
        self.cancel_login(login).await
    }

    pub(super) async fn cancel_or_clear_saved(
        &self,
        id: Option<&str>,
        backend: Arc<dyn Backend>,
    ) -> Result<(), String> {
        let login = {
            let slot = self.slot.lock().map_err(|_| "oauth_slot_poisoned")?;
            match slot.as_ref() {
                Some(login) => {
                    if id.is_some_and(|id| id != login.id) {
                        return Err("oauth_login_id_changed".into());
                    }
                    Arc::clone(login)
                }
                None => {
                    if let Some(pending) = backend.load()? {
                        if id.is_some_and(|id| id != pending.login_id) {
                            return Err("oauth_login_id_changed".into());
                        }
                        backend.clear()?;
                    }
                    return Ok(());
                }
            }
        };
        self.cancel_login(login).await
    }

    async fn cancel_login(&self, login: Arc<Login>) -> Result<(), String> {
        {
            let state = login.state.lock().map_err(|_| "oauth_state_poisoned")?;
            if state.phase == Phase::Committing {
                return Err("oauth_commit_in_progress".into());
            }
            login.cancel.cancel();
        }
        let mut done = login.done.clone();
        done.wait_for(|result| result.is_some())
            .await
            .map_err(|_| "oauth_owner_lost")?;
        let outcome = done.borrow().clone();
        match outcome {
            Some(Err(Outcome::CleanupFailed(error))) => {
                // Retry only the failed storage cleanup; no second terminal or
                // account mutation is produced. Keep the slot blocked on error.
                {
                    let slot = self.slot.lock().map_err(|_| "oauth_slot_poisoned")?;
                    if !slot
                        .as_ref()
                        .is_some_and(|current| Arc::ptr_eq(current, &login))
                    {
                        return Err("oauth_login_id_changed".into());
                    }
                    login
                        .cleanup_retrying
                        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                        .map_err(|_| "oauth_cleanup_in_progress")?;
                }
                // The claim keeps this failed slot exclusive. Disk I/O must
                // not hold the slot or login-state mutex used by controls.
                if login.backend.clear().is_err() {
                    login.cleanup_retrying.store(false, Ordering::Release);
                    return Err(format!("oauth_cleanup_failed: {error}"));
                }
                let mut slot = self.slot.lock().map_err(|_| "oauth_slot_poisoned")?;
                if slot
                    .as_ref()
                    .is_some_and(|current| Arc::ptr_eq(current, &login))
                {
                    *slot = None;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
#[path = "oauth_tests.rs"]
mod tests;

impl Login {
    fn pending(&self) -> Result<CodexOAuthPendingState, String> {
        self.state
            .lock()
            .map(|state| state.pending.clone())
            .map_err(|_| "oauth_state_poisoned".into())
    }

    fn validate_active(&self, state: &LoginState) -> Result<(), String> {
        if state.phase == Phase::Terminal || self.cancel.is_cancelled() {
            return Err("oauth_not_running".into());
        }
        if Instant::now() >= self.deadline || state.pending.expires_at <= self.backend.now() {
            return Err("oauth_expired".into());
        }
        Ok(())
    }

    fn submit(&self, url: &str) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|_| "oauth_state_poisoned")?;
        self.validate_active(&state)?;
        if state.phase != Phase::Waiting {
            return Err("oauth_callback_already_received".into());
        }
        validate_callback(&state.pending, url)?;
        let next = CodexOAuthPendingState {
            callback_url: Some(url.to_string().into()),
            ..state.pending.clone()
        };
        self.backend.save(&next)?;
        state.pending = next;
        state.phase = Phase::Callback;
        (self.events)(Event::Callback {
            login_id: self.id.clone(),
        });
        Ok(())
    }

    async fn bounded<T>(
        &self,
        cancel: &CancellationToken,
        future: impl Future<Output = Result<T, String>>,
    ) -> Result<T, Outcome> {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => Err(Outcome::Cancelled),
            _ = tokio::time::sleep_until(self.deadline) => Err(Outcome::TimedOut),
            result = future => result.map_err(Outcome::Failed),
        }
    }

    async fn listen(
        self: Arc<Self>,
        listener: TcpListener,
        stop: CancellationToken,
    ) -> Result<(), Outcome> {
        loop {
            let (mut stream, _) = self
                .bounded(&stop, async {
                    listener
                        .accept()
                        .await
                        .map_err(|_| "oauth_accept_failed".into())
                })
                .await?;
            self.bounded(&stop, async {
                let mut buffer = [0; 8192];
                let mut used = 0;
                while used < buffer.len() {
                    let size = stream
                        .read(&mut buffer[used..])
                        .await
                        .map_err(|_| "oauth_socket_read_failed")?;
                    if size == 0 {
                        break;
                    }
                    used += size;
                    if buffer[..used].windows(4).any(|part| part == b"\r\n\r\n") {
                        break;
                    }
                }
                let request = String::from_utf8_lossy(&buffer[..used]);
                let accepted = super::parse_http_request_path(&request)
                    .and_then(|path| {
                        super::callback_url_from_path(&path, self.pending().ok()?.port).ok()
                    })
                    .is_some_and(|url| self.submit(url.as_str()).is_ok());
                let (status, body) = if accepted {
                    ("200 OK", "Authorization received. Return to CCR.")
                } else {
                    ("400 Bad Request", "Invalid OAuth callback.")
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .map_err(|_| "oauth_socket_write_failed".into())
            })
            .await?;
        }
    }

    async fn run(
        self: Arc<Self>,
        listener: TcpListener,
        mut requests: mpsc::Receiver<Completion>,
        done: watch::Sender<Option<Result<(), Outcome>>>,
        admission: Option<OwnedSemaphorePermit>,
    ) {
        let stop = self.cancel.child_token();
        let mut listener_task = tokio::spawn(Arc::clone(&self).listen(listener, stop.clone()));
        let mut reply = None;
        let mut listener_joined = false;
        let mut result = tokio::select! {
            biased;
            _ = self.cancel.cancelled() => Err(Outcome::Cancelled),
            _ = tokio::time::sleep_until(self.deadline) => Err(Outcome::TimedOut),
            ended = &mut listener_task => {
                listener_joined = true;
                ended.unwrap_or_else(|_| Err(Outcome::Failed("oauth_listener_failed".into()))).and_then(|()| Err(Outcome::Failed("oauth_listener_stopped".into())))
            }
            request = requests.recv() => {
                match request {
                    None => Err(Outcome::Failed("oauth_completion_closed".into())),
                    Some(request) => {
                        reply = Some(request.reply);
                        self.exchange_and_commit(request.name).await
                    }
                }
            }
        };
        stop.cancel();
        if !listener_joined && listener_task.await.is_err() {
            result = Err(Outcome::CleanupFailed("oauth_listener_join_failed".into()));
        }
        // No cancellable await between secret cleanup and terminal publication.
        if let Err(error) = self.backend.clear() {
            result = Err(Outcome::CleanupFailed(error));
        }
        if let Ok(mut state) = self.state.lock() {
            state.phase = Phase::Terminal;
        }
        if matches!(result, Err(Outcome::TimedOut))
            && let Ok(pending) = self.pending()
        {
            (self.events)(Event::Timeout {
                login_id: self.id.clone(),
                callback_url: pending.redirect_uri,
            });
        }
        drop(admission);
        let terminal = result.as_ref().map(|_| ()).map_err(Clone::clone);
        done.send_replace(Some(terminal));
        if let Some(reply) = reply {
            let _ = reply.send(result);
        }
    }

    async fn exchange_and_commit(
        &self,
        name: Option<String>,
    ) -> Result<CodexAuthMutationResponse, Outcome> {
        let pending = self.pending().map_err(Outcome::Failed)?;
        let tokens = self
            .bounded(&self.cancel, self.backend.exchange(pending))
            .await?;
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| Outcome::Failed("oauth_state_poisoned".into()))?;
            if self.cancel.is_cancelled() {
                return Err(Outcome::Cancelled);
            }
            if Instant::now() >= self.deadline {
                return Err(Outcome::TimedOut);
            }
            state.phase = Phase::Committing;
        }
        // The secret writer cannot be detached by dropping a spawn_blocking
        // future. Once commit starts, admission remains held and cancellation
        // returns an explicit rejection until the owner completes.
        self.backend
            .commit(tokens, name)
            .await
            .map_err(Outcome::Failed)
    }
}

fn validate_callback(pending: &CodexOAuthPendingState, url: &str) -> Result<(), String> {
    let parsed = reqwest::Url::parse(url).map_err(|_| "oauth_invalid_callback")?;
    if parsed.scheme() != "http"
        || !matches!(parsed.host_str(), Some("localhost" | "127.0.0.1"))
        || parsed.port_or_known_default() != Some(pending.port)
        || parsed.path() != "/auth/callback"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err("oauth_invalid_callback_origin".into());
    }
    let pairs = parsed.query_pairs().collect::<Vec<_>>();
    let states = pairs
        .iter()
        .filter(|(key, _)| key == "state")
        .collect::<Vec<_>>();
    let codes = pairs
        .iter()
        .filter(|(key, _)| key == "code")
        .collect::<Vec<_>>();
    if states.len() != 1 || states[0].1 != pending.state.expose() {
        return Err("oauth_state_mismatch".into());
    }
    if codes.len() != 1 || codes[0].1.is_empty() {
        return Err("oauth_code_missing".into());
    }
    Ok(())
}
