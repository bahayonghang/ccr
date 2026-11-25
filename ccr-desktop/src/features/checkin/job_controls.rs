use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[gpui_kit::test]
async fn filtered_header_selection_and_row_use_confirmed_ids_and_return_to_credential_result(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    let owner = RuntimeOwner::new().unwrap();
    let http = tokio::runtime::Runtime::new().unwrap();
    let listener = http
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let conn = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
    conn.execute("UPDATE checkin_providers SET base_url=?1", [&url])
        .unwrap();
    let crypto = ccr_core::core::LockManager::with_scoped_directory(
        &root.path().join("checkin-locks"),
        || Ok(ccr_checkin::CryptoManager::new(&root.path().join("checkin-crypto")).unwrap()),
    )
    .unwrap();
    let encrypted = crypto
        .encrypt(r#"{"session":"private-test-value"}"#)
        .unwrap();
    conn.execute(
        "UPDATE checkin_accounts SET cookies_json_encrypted=?1",
        [encrypted],
    )
    .unwrap();
    let requests = Arc::new(AtomicUsize::new(0));
    let count = requests.clone();
    let server = http.spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buffer = [0; 4096];
        let n = stream.read(&mut buffer).await.unwrap();
        assert!(
            std::str::from_utf8(&buffer[..n])
                .unwrap()
                .starts_with("GET /api/user/self")
        );
        count.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(200)).await;
        stream
            .write_all(
                b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .await
            .unwrap();
    });
    let retained = services(&owner, root.path()).with_checkin_loopback_fixture();
    let handle = open_services(cx, retained.clone(), 1280., 800.);
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        w.click("nav-checkin", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(2), |w, _| {
        w.try_find("checkin-all").is_some()
    })
    .await;
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        w.click("checkin-search", cx);
        w.input("Desk", cx);
        w.render_frame(cx);
        w.click("checkin-all", cx);
        w.render_frame(cx);
        let all = w.find("checkin-confirmation");
        let notice = all.label().unwrap();
        assert!(
            notice.contains("2 accounts")
                && notice.contains("Unmeasured account")
                && notice.contains("Desk account")
                && notice.contains("Fixture provider")
        );
        w.click("checkin-cancel-delete", cx);
        w.render_frame(cx);
        w.click("checkin-pick-measured", cx);
        w.render_frame(cx);
        w.click("checkin-selected-run", cx);
        w.render_frame(cx);
        let selected = w.find("checkin-confirmation");
        assert!(
            selected.label().unwrap().contains("Desk account")
                && !selected.label().unwrap().contains("Unmeasured")
        );
        w.click("checkin-cancel-delete", cx);
        w.render_frame(cx);
        assert_eq!(requests.load(Ordering::SeqCst), 0);
        w.click("checkin-refresh-measured", cx);
        w.render_frame(cx);
        assert!(
            w.find("checkin-confirmation")
                .label()
                .unwrap()
                .contains("(measured)")
        );
        w.click("checkin-entry-confirm", cx);
        w.render_frame(cx);
        w.click("checkin-all", cx);
        w.render_frame(cx);
        assert!(w.try_find("checkin-confirmation").is_none());
        w.click("nav-usage", cx);
        w.render_frame(cx);
        w.click("nav-checkin", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(2), |w, _| {
        let state = retained.checkin_operation_state();
        if !state.busy {
            assert!(
                state
                    .last_job
                    .as_ref()
                    .is_some_and(|r| r.items[0].error_code.as_deref() == Some("cookie_expired")),
                "terminal: {:?}, status {:?}, requests {}",
                state.result,
                state.last_job,
                requests.load(Ordering::SeqCst)
            );
        }
        w.try_find("result-edit-measured").is_some()
    })
    .await;
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        assert!(
            w.find("checkin-result-measured")
                .label()
                .unwrap()
                .contains("cookie_expired")
        );
        w.click("result-edit-measured", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(2), |w, _| {
        w.try_find("checkin-save").is_some() && w.try_find("checkin-credential-loading").is_none()
    })
    .await;
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        w.click("checkin-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(2), |w, _| {
        let state = retained.checkin_operation_state();
        assert!(
            !matches!(state.result, Some(Err(_))),
            "save error {:?}",
            state.result
        );
        w.try_find("result-edit-measured").is_some()
            && w.try_find("checkin-pick-measured").is_some()
    })
    .await;
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        w.simulate_next_frame(cx);
        w.render_frame(cx);
        assert_eq!(w.find("result-edit-measured").focused(), Some(true));
        assert!(!labels(w).contains("private-test-value"));
        w.remove_window();
    })
    .unwrap();
    http.block_on(async {
        tokio::time::timeout(Duration::from_secs(3), server)
            .await
            .unwrap()
            .unwrap()
    });
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    owner.shutdown();
}
