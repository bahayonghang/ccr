use super::*;
use std::sync::atomic::AtomicUsize;

struct Fixture {
    pending: Mutex<Option<CodexOAuthPendingState>>,
    clears: AtomicUsize,
    fail_clear: AtomicBool,
    commits: AtomicUsize,
    commit_started: tokio::sync::Notify,
    commit_release: CancellationToken,
    committed: Arc<AtomicBool>,
    timeout: Duration,
}

impl Fixture {
    fn new(timeout: Duration) -> Arc<Self> {
        Arc::new(Self {
            pending: Mutex::new(None),
            clears: AtomicUsize::new(0),
            fail_clear: AtomicBool::new(false),
            commits: AtomicUsize::new(0),
            commit_started: tokio::sync::Notify::new(),
            commit_release: CancellationToken::new(),
            committed: Arc::new(AtomicBool::new(false)),
            timeout,
        })
    }
    fn state(&self) -> CodexOAuthPendingState {
        let socket = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = socket.local_addr().unwrap().port();
        CodexOAuthPendingState {
            login_id: "isolated-login".into(),
            auth_url: "https://example.invalid/?state=secret-state".into(),
            redirect_uri: format!("http://localhost:{port}/auth/callback"),
            code_verifier: "secret-verifier".into(),
            state: "secret-state".into(),
            port,
            expires_at: 1200,
            callback_url: None,
        }
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
        Ok(self.pending.lock().unwrap().clone())
    }
    fn save(&self, pending: &CodexOAuthPendingState) -> Result<(), String> {
        *self.pending.lock().unwrap() = Some(pending.clone());
        Ok(())
    }
    fn clear(&self) -> Result<(), String> {
        self.clears.fetch_add(1, Ordering::SeqCst);
        if self.fail_clear.load(Ordering::SeqCst) {
            return Err("cleanup_denied".into());
        }
        *self.pending.lock().unwrap() = None;
        Ok(())
    }
    fn bind(&self, port: u16) -> Result<std::net::TcpListener, String> {
        std::net::TcpListener::bind(("127.0.0.1", port)).map_err(|_| "bind_failed".into())
    }
    fn exchange(&self, _: CodexOAuthPendingState) -> Work<TokenResponse> {
        Box::pin(async {
            Ok(TokenResponse {
                id_token: "secret-id".into(),
                access_token: "secret-access".into(),
                refresh_token: None,
            })
        })
    }
    fn commit(&self, _: TokenResponse, _: Option<String>) -> Work<CommitResult> {
        self.commits.fetch_add(1, Ordering::SeqCst);
        self.commit_started.notify_one();
        let release = self.commit_release.clone();
        let committed = self.committed.clone();
        Box::pin(async move {
            release.cancelled().await;
            committed.store(true, Ordering::SeqCst);
            Ok(CommitResult::account("synthetic-account".into(), true))
        })
    }
}

#[test]
fn pkce_matches_rfc7636_s256_vector() {
    assert_eq!(
        generate_code_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
}

#[tokio::test]
async fn callback_identity_cancel_and_secret_cleanup_are_owned() {
    let fixture = Fixture::new(Duration::from_secs(3));
    let controller = Controller::default();
    let pending = controller
        .start(
            fixture.clone(),
            Arc::new(|_| {}),
            || Ok(fixture.state()),
            None,
        )
        .unwrap();
    let callback = format!(
        "{}?state=secret-state&code=secret-code",
        pending.redirect_uri
    );
    assert!(controller.submit("another-login", &callback).is_err());
    assert!(
        controller
            .submit(
                &pending.login_id,
                &callback.replace("state=secret-state", "state=wrong")
            )
            .is_err()
    );
    controller.submit(&pending.login_id, &callback).unwrap();
    assert!(controller.submit(&pending.login_id, &callback).is_err());
    controller.request_cancel(Some(&pending.login_id)).unwrap();
    assert!(
        controller
            .wait_for_terminal(&pending.login_id)
            .await
            .is_err()
    );
    assert!(fixture.pending.lock().unwrap().is_none());
    assert_eq!(fixture.clears.load(Ordering::SeqCst), 1);
    assert!(!fixture.committed.load(Ordering::SeqCst));
    assert!(std::net::TcpListener::bind(("127.0.0.1", pending.port)).is_ok());
}

#[tokio::test]
async fn deadline_cleans_before_terminal_and_releases_listener() {
    let fixture = Fixture::new(Duration::from_millis(25));
    let controller = Controller::default();
    let pending = controller
        .start(
            fixture.clone(),
            Arc::new(|_| {}),
            || Ok(fixture.state()),
            None,
        )
        .unwrap();
    assert!(
        controller
            .wait_for_terminal(&pending.login_id)
            .await
            .unwrap_err()
            .contains("timed_out")
    );
    assert!(fixture.pending.lock().unwrap().is_none());
    assert_eq!(fixture.clears.load(Ordering::SeqCst), 1);
    assert!(std::net::TcpListener::bind(("127.0.0.1", pending.port)).is_ok());
}

#[tokio::test]
async fn shutdown_waits_for_commit_instead_of_detaching_writer() {
    let fixture = Fixture::new(Duration::from_secs(3));
    let controller = Arc::new(Controller::default());
    let pending = controller
        .start(
            fixture.clone(),
            Arc::new(|_| {}),
            || Ok(fixture.state()),
            None,
        )
        .unwrap();
    controller
        .submit(
            &pending.login_id,
            &format!(
                "{}?state=secret-state&code=secret-code",
                pending.redirect_uri
            ),
        )
        .unwrap();
    let completing = controller.clone();
    let id = pending.login_id.clone();
    let complete = tokio::spawn(async move { completing.complete(&id, None).await });
    fixture.commit_started.notified().await;
    assert_eq!(
        controller.request_cancel(None).unwrap_err(),
        "oauth_commit_in_progress"
    );
    let waiting = controller.clone();
    let id = pending.login_id.clone();
    let terminal = tokio::spawn(async move { waiting.wait_for_terminal(&id).await });
    tokio::task::yield_now().await;
    assert!(!complete.is_finished() && !terminal.is_finished());
    assert!(fixture.pending.lock().unwrap().is_some());
    fixture.commit_release.cancel();
    let committed = complete.await.unwrap().unwrap();
    assert!(committed.success);
    terminal.await.unwrap().unwrap();
    assert!(fixture.pending.lock().unwrap().is_none());
    assert!(
        !serde_json::to_string(&committed)
            .unwrap()
            .contains("secret-")
    );
    assert_eq!(fixture.clears.load(Ordering::SeqCst), 1);
    assert!(fixture.committed.load(Ordering::SeqCst));
}

#[tokio::test]
async fn silent_socket_times_out_and_releases_listener_without_success() {
    let fixture = Fixture::new(Duration::from_millis(60));
    let controller = Controller::default();
    let pending = controller
        .start(
            fixture.clone(),
            Arc::new(|_| {}),
            || Ok(fixture.state()),
            None,
        )
        .unwrap();
    let mut socket = tokio::net::TcpStream::connect(("127.0.0.1", pending.port))
        .await
        .unwrap();
    let login = controller.login(Some(&pending.login_id)).unwrap().unwrap();
    let mut done = login.done.clone();
    tokio::time::timeout(
        Duration::from_secs(2),
        done.wait_for(|value| value.is_some()),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(*done.borrow(), Some(Err(Outcome::TimedOut)));
    let mut buf = [0; 1];
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(2), socket.read(&mut buf))
            .await
            .unwrap()
            .unwrap(),
        0
    );
    assert!(std::net::TcpListener::bind(("127.0.0.1", pending.port)).is_ok());
    assert!(fixture.pending.lock().unwrap().is_none());
    assert_eq!(fixture.commits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn stale_cleanup_retry_cannot_clear_a_replacement_login() {
    let fixture = Fixture::new(Duration::from_secs(30));
    let controller = Controller::default();
    let first = controller
        .start(
            fixture.clone(),
            Arc::new(|_| {}),
            || Ok(fixture.state()),
            None,
        )
        .unwrap();
    let stale_login = controller.login(Some(&first.login_id)).unwrap().unwrap();
    fixture.fail_clear.store(true, Ordering::SeqCst);
    tokio::time::timeout(
        Duration::from_secs(2),
        controller.cancel(Some(&first.login_id)),
    )
    .await
    .unwrap()
    .expect_err("first cleanup fails");
    fixture.fail_clear.store(false, Ordering::SeqCst);
    controller.cancel(Some(&first.login_id)).await.unwrap();
    controller
        .start(
            fixture.clone(),
            Arc::new(|_| {}),
            || {
                let mut pending = fixture.state();
                pending.login_id = "replacement-login".into();
                Ok(pending)
            },
            None,
        )
        .unwrap();
    // 旧重试保留捕获的 owner；不能因 slot 已替换而清理新登录。
    let stale_result = controller.cancel_login(stale_login).await;
    let saved_id = fixture
        .pending
        .lock()
        .unwrap()
        .as_ref()
        .map(|pending| pending.login_id.clone());
    controller.cancel(Some("replacement-login")).await.unwrap();
    assert_eq!(saved_id.as_deref(), Some("replacement-login"));
    assert_eq!(stale_result, Err("oauth_login_id_changed".into()));
    assert_eq!(fixture.commits.load(Ordering::SeqCst), 0);
}
