use super::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::sync::{Notify, Semaphore};

struct Fixture {
    disk: Mutex<Option<CodexOAuthPendingState>>,
    listener: Mutex<Option<std::net::TcpListener>>,
    port: u16,
    fail_save: AtomicBool,
    fail_clear: AtomicBool,
    commits: AtomicUsize,
    commit_started: Arc<Notify>,
    commit_release: Mutex<Option<CancellationToken>>,
    exchange_started: Arc<Notify>,
    exchange_release: CancellationToken,
    cleanup_started: Arc<Notify>,
    cleanup_release: Mutex<Option<std::sync::mpsc::Receiver<()>>>,
    timeout: Duration,
    endpoint: Option<String>,
}

impl Fixture {
    fn new(timeout: Duration, endpoint: Option<String>) -> Arc<Self> {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("isolated listener");
        let port = listener.local_addr().expect("port").port();
        Arc::new(Self {
            disk: Mutex::new(None),
            listener: Mutex::new(Some(listener)),
            port,
            fail_save: AtomicBool::new(false),
            fail_clear: AtomicBool::new(false),
            commits: AtomicUsize::new(0),
            commit_started: Arc::new(Notify::new()),
            commit_release: Mutex::new(None),
            exchange_started: Arc::new(Notify::new()),
            exchange_release: CancellationToken::new(),
            cleanup_started: Arc::new(Notify::new()),
            cleanup_release: Mutex::new(None),
            timeout,
            endpoint,
        })
    }

    fn pending(&self, id: &str) -> CodexOAuthPendingState {
        CodexOAuthPendingState {
            login_id: id.into(),
            auth_url: "https://example.invalid/?state=sentinel-state".into(),
            redirect_uri: format!("http://localhost:{}/auth/callback", self.port),
            code_verifier: "sentinel-verifier".into(),
            state: "sentinel-state".into(),
            port: self.port,
            expires_at: 1300,
            callback_url: None,
        }
    }

    fn callback(&self) -> String {
        format!(
            "http://localhost:{}/auth/callback?state=sentinel-state&code=sentinel-code",
            self.port
        )
    }
}

impl Backend for Fixture {
    fn now(&self) -> i64 {
        1000
    }
    fn remaining(&self, _: i64) -> Duration {
        self.timeout
    }
    fn load(&self) -> Result<Option<CodexOAuthPendingState>, String> {
        let mut disk = self.disk.lock().expect("disk");
        if disk
            .as_ref()
            .is_some_and(|pending| pending.expires_at <= self.now())
        {
            *disk = None;
        }
        Ok(disk.clone())
    }
    fn save(&self, pending: &CodexOAuthPendingState) -> Result<(), String> {
        if self.fail_save.load(Ordering::SeqCst) {
            return Err("storage_denied".into());
        }
        *self.disk.lock().expect("disk") = Some(pending.clone());
        Ok(())
    }
    fn clear(&self) -> Result<(), String> {
        self.cleanup_started.notify_one();
        if let Some(release) = self.cleanup_release.lock().expect("cleanup gate").take() {
            release
                .recv_timeout(Duration::from_secs(3))
                .expect("cleanup released within watchdog");
        }
        if self.fail_clear.load(Ordering::SeqCst) {
            return Err("cleanup_denied".into());
        }
        *self.disk.lock().expect("disk") = None;
        Ok(())
    }
    fn bind(&self, port: u16) -> Result<std::net::TcpListener, String> {
        self.listener
            .lock()
            .expect("listener")
            .take()
            .map(Ok)
            .unwrap_or_else(|| {
                std::net::TcpListener::bind(("127.0.0.1", port)).map_err(|_| "bind_failed".into())
            })
    }
    fn exchange(&self, pending: CodexOAuthPendingState) -> Work<CodexOAuthTokenResponse> {
        let started = Arc::clone(&self.exchange_started);
        let release = self.exchange_release.clone();
        let endpoint = self.endpoint.clone();
        Box::pin(async move {
            started.notify_one();
            if let Some(endpoint) = endpoint {
                return super::super::exchange_oauth_tokens_at(
                    &endpoint,
                    "sentinel-code",
                    pending.code_verifier.expose(),
                    &pending.redirect_uri,
                )
                .await;
            }
            release.cancelled().await;
            Ok(CodexOAuthTokenResponse {
                id_token: "sentinel-id-token".into(),
                access_token: "sentinel-access-token".into(),
                refresh_token: None,
            })
        })
    }
    fn commit(
        &self,
        _: CodexOAuthTokenResponse,
        _: Option<String>,
    ) -> Work<CodexAuthMutationResponse> {
        self.commits.fetch_add(1, Ordering::SeqCst);
        let started = Arc::clone(&self.commit_started);
        let release = self.commit_release.lock().expect("commit gate").clone();
        Box::pin(async move {
            started.notify_one();
            if let Some(release) = release {
                release.cancelled().await;
            }
            Ok(CodexAuthMutationResponse::account(
                "synthetic-account".into(),
                true,
            ))
        })
    }
}

fn start(controller: &Controller, fixture: &Arc<Fixture>) -> CodexOAuthPendingState {
    controller
        .start(
            fixture.clone(),
            Arc::new(|_| {}),
            || Ok(fixture.pending("first")),
            None,
        )
        .expect("start")
}

async fn watchdog<T>(work: impl Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(2), work)
        .await
        .expect("owner acknowledgement before watchdog")
}

#[tokio::test]
async fn oauth_start_reuses_live_id_before_waiting_for_same_module_admission() {
    use crate::commands::runtime_policy::{acquire_oauth_admission, execute};
    let fixture = Fixture::new(Duration::from_secs(30), None);
    let controller = Controller::default();
    let (mutation_started, started) = oneshot::channel();
    let (mutation_release, release) = oneshot::channel();
    let mutation = tokio::spawn(execute("codex_save_auth", async {
        mutation_started.send(()).expect("mutation started");
        release.await.expect("mutation release");
        Ok::<_, String>(())
    }));
    started.await.expect("module permit held");
    let mut first = std::pin::pin!(execute(
        "codex_oauth_login_start",
        controller.start_admitted(
            fixture.clone(),
            Arc::new(|_| {}),
            || Ok(fixture.pending("first")),
            acquire_oauth_admission(),
        )
    ));
    std::future::poll_fn(|cx| {
        assert!(Future::poll(first.as_mut(), cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    assert!(controller.current().expect("current").is_none());
    mutation_release.send(()).expect("release mutation");
    mutation
        .await
        .expect("mutation join")
        .expect("mutation result");
    first.await.expect("first login");
    let repeated = watchdog(execute(
        "codex_oauth_login_start",
        controller.start_admitted(
            fixture.clone(),
            Arc::new(|_| {}),
            || panic!("active start cannot create a new login"),
            std::future::pending(),
        ),
    ))
    .await
    .expect("immediate same ID");
    assert_eq!(repeated.login_id, "first");
    let mut next_mutation =
        std::pin::pin!(execute("codex_save_auth", async { Ok::<_, String>(()) }));
    std::future::poll_fn(|cx| {
        assert!(Future::poll(next_mutation.as_mut(), cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    watchdog(execute(
        "codex_oauth_login_cancel",
        controller.cancel(Some("first")),
    ))
    .await
    .expect("control reaches owner");
    next_mutation
        .await
        .expect("module permit released after cleanup");
}

#[tokio::test]
async fn recovered_callback_must_match_state_before_listener_publication() {
    let fixture = Fixture::new(Duration::from_secs(30), None);
    let controller = Controller::default();
    let mut pending = fixture.pending("recovered");
    pending.callback_url = Some(fixture.callback().replace("sentinel-state", "wrong").into());
    *fixture.disk.lock().expect("disk") = Some(pending);
    let error = controller
        .start(
            fixture.clone(),
            Arc::new(|_| {}),
            || panic!("load existing"),
            None,
        )
        .expect_err("state mismatch");
    assert_eq!(error, "oauth_state_mismatch");
    assert!(controller.current().expect("current").is_none());
    assert!(fixture.listener.lock().expect("listener").is_some());
    controller
        .cancel_or_clear_saved(Some("recovered"), fixture.clone())
        .await
        .expect("cancel unstarted saved state");
    assert!(fixture.disk.lock().expect("disk").is_none());
}

#[tokio::test]
async fn start_owns_listener_and_storage_before_return_and_reuses_live_id() {
    let fixture = Fixture::new(Duration::from_secs(30), None);
    let controller = Controller::default();
    let pending = start(&controller, &fixture);
    assert!(std::net::TcpListener::bind(("127.0.0.1", pending.port)).is_err());
    assert_eq!(
        fixture
            .disk
            .lock()
            .expect("disk")
            .as_ref()
            .expect("saved")
            .login_id,
        "first"
    );
    let second = controller
        .start(
            fixture.clone(),
            Arc::new(|_| {}),
            || panic!("second start cannot create a login"),
            None,
        )
        .expect("reuse");
    assert_eq!(second.login_id, pending.login_id);
    watchdog(controller.cancel(Some("first")))
        .await
        .expect("cancel and cleanup");
    assert!(fixture.disk.lock().expect("disk").is_none());
    assert!(std::net::TcpListener::bind(("127.0.0.1", pending.port)).is_ok());
}

#[tokio::test]
async fn bind_race_and_save_failure_never_publish_active_login() {
    let fixture = Fixture::new(Duration::from_secs(30), None);
    let controller = Controller::default();
    let held = fixture
        .listener
        .lock()
        .expect("listener")
        .take()
        .expect("held listener");
    assert!(
        controller
            .start(
                fixture.clone(),
                Arc::new(|_| {}),
                || Ok(fixture.pending("first")),
                None
            )
            .is_err()
    );
    assert!(controller.current().expect("current").is_none());
    assert!(fixture.disk.lock().expect("disk").is_none());
    drop(held);
    fixture.fail_save.store(true, Ordering::SeqCst);
    assert!(
        controller
            .start(
                fixture.clone(),
                Arc::new(|_| {}),
                || Ok(fixture.pending("first")),
                None
            )
            .is_err()
    );
    assert!(controller.current().expect("current").is_none());
    assert!(std::net::TcpListener::bind(("127.0.0.1", fixture.port)).is_ok());
}

#[tokio::test]
async fn wrong_id_state_origin_and_duplicate_callback_leave_owner_unchanged() {
    let fixture = Fixture::new(Duration::from_secs(30), None);
    let controller = Controller::default();
    start(&controller, &fixture);
    assert!(controller.cancel(Some("stale")).await.is_err());
    assert!(controller.submit("stale", &fixture.callback()).is_err());
    assert!(
        controller
            .submit(
                "first",
                &fixture.callback().replace("sentinel-state", "wrong")
            )
            .is_err()
    );
    assert!(
        controller
            .submit(
                "first",
                &fixture.callback().replace("localhost", "example.invalid")
            )
            .is_err()
    );
    fixture.fail_save.store(true, Ordering::SeqCst);
    assert!(controller.submit("first", &fixture.callback()).is_err());
    assert!(
        controller
            .current()
            .expect("current")
            .expect("active")
            .callback_url
            .is_none()
    );
    fixture.fail_save.store(false, Ordering::SeqCst);
    controller
        .submit("first", &fixture.callback())
        .expect("valid callback");
    assert!(controller.submit("first", &fixture.callback()).is_err());
    watchdog(controller.cancel(Some("first")))
        .await
        .expect("cancel");
    assert!(controller.submit("first", &fixture.callback()).is_err());
    assert_eq!(fixture.commits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn silent_socket_times_out_and_releases_listener_without_success() {
    let fixture = Fixture::new(Duration::from_millis(60), None);
    let controller = Controller::default();
    start(&controller, &fixture);
    let mut socket = tokio::net::TcpStream::connect(("127.0.0.1", fixture.port))
        .await
        .expect("connect");
    let login = controller
        .login(Some("first"))
        .expect("lookup")
        .expect("login");
    let mut done = login.done.clone();
    watchdog(done.wait_for(|value| value.is_some()))
        .await
        .expect("timeout terminal");
    assert_eq!(*done.borrow(), Some(Err(Outcome::TimedOut)));
    let mut buf = [0; 1];
    assert_eq!(
        watchdog(socket.read(&mut buf))
            .await
            .expect("socket closed"),
        0
    );
    assert!(std::net::TcpListener::bind(("127.0.0.1", fixture.port)).is_ok());
    assert!(fixture.disk.lock().expect("disk").is_none());
    assert_eq!(fixture.commits.load(Ordering::SeqCst), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_exchange_keeps_admission_until_cleanup_finishes() {
    for control in ["codex_oauth_login_cancel", "codex_release_oauth_port"] {
        let fixture = Fixture::new(Duration::from_secs(30), None);
        let controller = Arc::new(Controller::default());
        let gate = Arc::new(Semaphore::new(1));
        let permit = gate.clone().acquire_owned().await.expect("permit");
        controller
            .start(
                fixture.clone(),
                Arc::new(|_| {}),
                || Ok(fixture.pending("first")),
                Some(permit),
            )
            .expect("start returned");
        assert!(gate.clone().try_acquire_owned().is_err());
        controller
            .submit("first", &fixture.callback())
            .expect("callback");
        let completing = {
            let controller = controller.clone();
            tokio::spawn(async move {
                crate::commands::runtime_policy::execute(
                    "codex_oauth_login_completed",
                    controller.complete("first", None),
                )
                .await
            })
        };
        watchdog(fixture.exchange_started.notified()).await;
        assert!(
            watchdog(crate::commands::runtime_policy::execute(
                "codex_oauth_submit_callback_url",
                async { controller.submit("first", &fixture.callback()) },
            ))
            .await
            .is_err()
        );
        assert!(
            watchdog(crate::commands::runtime_policy::execute(
                "codex_is_oauth_port_in_use",
                async {
                    Ok::<_, String>(
                        std::net::TcpListener::bind(("127.0.0.1", fixture.port)).is_err(),
                    )
                },
            ))
            .await
            .expect("port query reaches the owned listener")
        );
        assert!(watchdog(controller.complete("first", None)).await.is_err());
        let (release, cleanup) = std::sync::mpsc::channel();
        *fixture.cleanup_release.lock().expect("gate") = Some(cleanup);
        let cancelling = {
            let controller = controller.clone();
            tokio::spawn(async move {
                crate::commands::runtime_policy::execute(control, controller.cancel(Some("first")))
                    .await
            })
        };
        watchdog(fixture.cleanup_started.notified()).await;
        assert!(!cancelling.is_finished());
        assert!(gate.clone().try_acquire_owned().is_err());
        assert!(
            controller
                .start(
                    fixture.clone(),
                    Arc::new(|_| {}),
                    || Ok(fixture.pending("second")),
                    None
                )
                .is_err()
        );
        release.send(()).expect("release cleanup");
        watchdog(cancelling)
            .await
            .expect("cancel join")
            .expect("cancel");
        let error = watchdog(completing)
            .await
            .expect("complete join")
            .expect_err("cancelled exchange cannot succeed");
        assert_eq!(error, "oauth_cancelled");
        assert!(gate.try_acquire_owned().is_ok());
        assert_eq!(fixture.commits.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn successful_completion_is_unique_and_does_not_expose_sentinels() {
    let fixture = Fixture::new(Duration::from_secs(30), None);
    let controller = Controller::default();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let events = observed.clone();
    controller
        .start(
            fixture.clone(),
            Arc::new(move |event| events.lock().expect("events").push(format!("{event:?}"))),
            || Ok(fixture.pending("first")),
            None,
        )
        .expect("start");
    controller
        .submit("first", &fixture.callback())
        .expect("callback");
    fixture.exchange_release.cancel();
    let result = watchdog(controller.complete("first", None))
        .await
        .expect("complete");
    assert!(result.success);
    assert!(controller.complete("first", None).await.is_err());
    assert_eq!(fixture.commits.load(Ordering::SeqCst), 1);
    let visible = format!(
        "{:?} {}",
        observed.lock().expect("events"),
        serde_json::to_string(&result).expect("result")
    );
    for sentinel in [
        "sentinel-state",
        "sentinel-code",
        "sentinel-verifier",
        "sentinel-access-token",
        "sentinel-id-token",
    ] {
        assert!(!visible.contains(sentinel));
    }
    assert!(fixture.disk.lock().expect("disk").is_none());
}

#[tokio::test]
async fn commit_rejects_cancellation_and_retains_admission_until_completion() {
    let fixture = Fixture::new(Duration::from_secs(30), None);
    let controller = Arc::new(Controller::default());
    let gate = Arc::new(Semaphore::new(1));
    let release = CancellationToken::new();
    *fixture.commit_release.lock().expect("commit gate") = Some(release.clone());
    controller
        .start(
            fixture.clone(),
            Arc::new(|_| {}),
            || Ok(fixture.pending("first")),
            Some(gate.clone().acquire_owned().await.expect("admission")),
        )
        .expect("start");
    controller
        .submit("first", &fixture.callback())
        .expect("callback");
    fixture.exchange_release.cancel();
    let completing = {
        let controller = controller.clone();
        tokio::spawn(async move { controller.complete("first", None).await })
    };
    watchdog(fixture.commit_started.notified()).await;
    let rejected = watchdog(crate::commands::runtime_policy::execute(
        "codex_oauth_login_cancel",
        controller.cancel(Some("first")),
    ))
    .await
    .expect_err("account commit cannot be cancelled");
    assert_eq!(rejected, "oauth_commit_in_progress");
    assert!(!completing.is_finished());
    assert!(gate.clone().try_acquire_owned().is_err());
    assert!(fixture.disk.lock().expect("disk").is_some());
    assert!(controller.complete("first", None).await.is_err());
    assert_eq!(fixture.commits.load(Ordering::SeqCst), 1);
    release.cancel();
    assert!(
        watchdog(completing)
            .await
            .expect("complete join")
            .expect("commit completed")
            .success
    );
    assert!(gate.try_acquire_owned().is_ok());
    assert!(fixture.disk.lock().expect("disk").is_none());
    assert!(std::net::TcpListener::bind(("127.0.0.1", fixture.port)).is_ok());
    assert_eq!(fixture.commits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn expired_restore_creates_fresh_id_and_cleanup_failure_is_visible() {
    let fixture = Fixture::new(Duration::from_secs(30), None);
    let controller = Controller::default();
    let mut expired = fixture.pending("expired");
    expired.expires_at = 999;
    *fixture.disk.lock().expect("disk") = Some(expired);
    assert_eq!(start(&controller, &fixture).login_id, "first");
    fixture.fail_clear.store(true, Ordering::SeqCst);
    let error = watchdog(controller.cancel(Some("first")))
        .await
        .expect_err("cleanup failure");
    assert!(error.starts_with("oauth_cleanup_failed"));
    assert!(
        controller
            .start(
                fixture.clone(),
                Arc::new(|_| {}),
                || Ok(fixture.pending("second")),
                None
            )
            .is_err()
    );
    fixture.fail_clear.store(false, Ordering::SeqCst);
    controller
        .cancel(Some("first"))
        .await
        .expect("retry storage cleanup");
    assert!(controller.current().expect("current").is_none());
}

#[tokio::test]
async fn restore_waiting_for_admission_does_not_resurrect_cancelled_pending() {
    let fixture = Fixture::new(Duration::from_secs(30), None);
    *fixture.disk.lock().expect("disk") = Some(fixture.pending("saved"));
    let controller = Controller::default();
    let gate = Arc::new(Semaphore::new(1));
    let held = Arc::clone(&gate).acquire_owned().await.expect("held");
    let mut restore = std::pin::pin!(controller.restore_admitted(
        fixture.clone(),
        Arc::new(|_| {}),
        async { gate.acquire_owned().await.map_err(|_| "closed".into()) },
    ));
    std::future::poll_fn(|cx| {
        assert!(Future::poll(restore.as_mut(), cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    controller
        .cancel_or_clear_saved(Some("saved"), fixture.clone())
        .await
        .expect("cancel persisted login before admission");
    assert!(fixture.disk.lock().expect("disk").is_none());
    drop(held);
    let result = watchdog(restore).await;
    let published = controller.current().expect("current");
    let persisted = fixture.disk.lock().expect("disk").clone();
    controller.cancel(None).await.expect("cleanup fixture");
    assert_eq!(
        result.map(|pending| pending.map(|p| p.login_id)),
        Err("oauth_saved_login_missing".into())
    );
    assert!(published.is_none());
    assert!(persisted.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cleanup_retry_has_one_owner_without_blocking_control_locks() {
    let fixture = Fixture::new(Duration::from_secs(30), None);
    let controller = Arc::new(Controller::default());
    start(&controller, &fixture);
    fixture.fail_clear.store(true, Ordering::SeqCst);
    watchdog(controller.cancel(Some("first")))
        .await
        .expect_err("cleanup fails");
    fixture.fail_clear.store(false, Ordering::SeqCst);
    // Consume notifications from the two failed clear attempts.
    fixture.cleanup_started.notified().await;
    let (release, cleanup) = std::sync::mpsc::channel();
    *fixture.cleanup_release.lock().expect("cleanup gate") = Some(cleanup);
    let owner = Arc::clone(&controller);
    let retry = tokio::spawn(async move { owner.cancel(Some("first")).await });
    watchdog(fixture.cleanup_started.notified()).await;
    let second = watchdog(controller.cancel(Some("first"))).await;
    assert_eq!(second, Err("oauth_cleanup_in_progress".into()));
    assert!(
        controller
            .current()
            .expect("unblocked control lock")
            .is_some()
    );
    assert!(
        controller
            .start(
                fixture.clone(),
                Arc::new(|_| {}),
                || Ok(fixture.pending("second")),
                None
            )
            .is_err()
    );
    release.send(()).expect("release cleanup");
    watchdog(retry)
        .await
        .expect("join")
        .expect("single cleanup owner");
    assert!(fixture.disk.lock().expect("disk").is_none());
}

#[tokio::test]
async fn stale_cleanup_retry_cannot_clear_a_replacement_login() {
    let fixture = Fixture::new(Duration::from_secs(30), None);
    let controller = Controller::default();
    start(&controller, &fixture);
    let stale_login = controller
        .login(Some("first"))
        .expect("login")
        .expect("active");
    fixture.fail_clear.store(true, Ordering::SeqCst);
    watchdog(controller.cancel(Some("first")))
        .await
        .expect_err("first cleanup fails");
    fixture.fail_clear.store(false, Ordering::SeqCst);
    controller
        .cancel(Some("first"))
        .await
        .expect("retry clears slot");
    controller
        .start(
            fixture.clone(),
            Arc::new(|_| {}),
            || Ok(fixture.pending("second")),
            None,
        )
        .expect("replacement login");
    // A second cancel may already have captured the previous owner before
    // the first retry cleared its slot. Resume that exact stale owner here.
    let stale_result = controller.cancel_login(stale_login).await;
    let saved_id = fixture
        .disk
        .lock()
        .expect("disk")
        .as_ref()
        .map(|pending| pending.login_id.clone());
    controller
        .cancel(Some("second"))
        .await
        .expect("cleanup fixture");
    assert_eq!(saved_id.as_deref(), Some("second"));
    assert_eq!(stale_result, Err("oauth_login_id_changed".into()));
}

#[tokio::test]
async fn slow_http_body_is_cancelled_and_server_observes_closed_connection() {
    let server = TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("HTTP fixture");
    let endpoint = format!(
        "http://{}/token",
        server.local_addr().expect("server address")
    );
    let (body_sent, body_ready) = oneshot::channel();
    let server_task = tokio::spawn(async move {
        let (mut connection, _) = server.accept().await.expect("accept HTTP");
        connection
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100000\r\nConnection: close\r\n\r\n{")
            .await
            .expect("send partial body");
        body_sent.send(()).expect("body signal");
        let mut ignored = Vec::new();
        connection
            .read_to_end(&mut ignored)
            .await
            .expect("client closes connection");
    });
    let fixture = Fixture::new(Duration::from_secs(30), Some(endpoint));
    let controller = Arc::new(Controller::default());
    start(&controller, &fixture);
    controller
        .submit("first", &fixture.callback())
        .expect("callback");
    let completing = {
        let controller = controller.clone();
        tokio::spawn(async move {
            crate::commands::runtime_policy::execute(
                "codex_oauth_login_completed",
                controller.complete("first", None),
            )
            .await
        })
    };
    watchdog(body_ready).await.expect("body read active");
    watchdog(controller.cancel(Some("first")))
        .await
        .expect("cancel slow body");
    assert_eq!(
        watchdog(completing)
            .await
            .expect("complete join")
            .expect_err("cancelled"),
        "oauth_cancelled"
    );
    watchdog(server_task).await.expect("server sees EOF");
    assert_eq!(fixture.commits.load(Ordering::SeqCst), 0);
    assert!(fixture.disk.lock().expect("disk").is_none());
}

#[tokio::test]
async fn slow_http_body_and_silent_accepted_socket_share_the_login_deadline() {
    let server = TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("HTTP fixture");
    let endpoint = format!(
        "http://{}/token",
        server.local_addr().expect("server address")
    );
    let (body_sent, body_ready) = oneshot::channel();
    let server_task = tokio::spawn(async move {
        let (mut connection, _) = server.accept().await.expect("accept HTTP");
        connection
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100000\r\nConnection: close\r\n\r\n{")
            .await
            .expect("partial body");
        body_sent.send(()).expect("body started");
        let mut ignored = Vec::new();
        let _ = connection.read_to_end(&mut ignored).await;
    });
    let fixture = Fixture::new(Duration::from_millis(400), Some(endpoint));
    let controller = Arc::new(Controller::default());
    start(&controller, &fixture);
    let _silent = tokio::net::TcpStream::connect(("127.0.0.1", fixture.port))
        .await
        .expect("silent accepted socket");
    controller
        .submit("first", &fixture.callback())
        .expect("manual callback remains reachable");
    let completing = {
        let controller = controller.clone();
        tokio::spawn(async move { controller.complete("first", None).await })
    };
    watchdog(body_ready).await.expect("body reached");
    assert_eq!(
        watchdog(completing)
            .await
            .expect("complete join")
            .expect_err("timeout"),
        "oauth_timed_out"
    );
    watchdog(server_task)
        .await
        .expect("server connection released");
    assert!(fixture.disk.lock().expect("disk").is_none());
    assert!(std::net::TcpListener::bind(("127.0.0.1", fixture.port)).is_ok());
    assert_eq!(fixture.commits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn token_endpoint_error_and_invalid_body_never_return_secret_content() {
    for status in ["400 Bad Request", "200 OK"] {
        let server = TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("HTTP fixture");
        let endpoint = format!(
            "http://{}/token",
            server.local_addr().expect("server address")
        );
        let server_task = tokio::spawn(async move {
            let (mut connection, _) = server.accept().await.expect("HTTP accept");
            let mut request = [0; 8192];
            let _ = connection.read(&mut request).await.expect("request");
            let body = "sentinel-access-token sentinel-verifier sentinel-state";
            connection.write_all(format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.expect("response");
        });
        let error = watchdog(super::super::exchange_oauth_tokens_at(
            &endpoint,
            "sentinel-code",
            "sentinel-verifier",
            "http://localhost:1455/auth/callback",
        ))
        .await
        .err()
        .expect("invalid response");
        for sentinel in [
            "sentinel-access-token",
            "sentinel-verifier",
            "sentinel-state",
            "sentinel-code",
        ] {
            assert!(!error.contains(sentinel));
        }
        watchdog(server_task).await.expect("server join");
    }
}
