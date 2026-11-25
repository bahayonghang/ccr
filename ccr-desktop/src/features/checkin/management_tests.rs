use super::*;

#[gpui_kit::test]
async fn manual_first_account_opens_its_detail_and_can_toggle_without_credentials(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-checkin", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("checkin-new-provider").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("checkin-new-provider", cx);
        window.render_frame(cx);
        window.input("Sample vendor", cx);
        window.press("tab", cx);
        window.input("https://sample.invalid", cx);
        window.click("checkin-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("checkin-new-account").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("checkin-new-account", cx);
        window.render_frame(cx);
        window.input("New account", cx);
        window.press("tab", cx);
        window.input("synthetic-session-247", cx);
        window.click("checkin-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("checkin-account-0").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window
                .find("checkin-detail")
                .label()
                .unwrap()
                .contains("New account")
        );
        assert!(!labels(window).contains("synthetic-session-247"));
        window.click("checkin-toggle-account", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("checkin-detail").and_then(|n| {
            n.label()
                .map(|s| s.contains("Disabled in the saved account."))
        }) == Some(true)
    })
    .await;
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn detail_connection_names_the_target_before_dispatch_and_fixture_refuses(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    ccr_core::core::LockManager::with_scoped_directory(&root.path().join("checkin-locks"), || {
        ccr_checkin::core::crypto::CryptoManager::new(&root.path().join("checkin-crypto")).unwrap();
        Ok(())
    })
    .unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-checkin", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |w, _| {
        w.try_find("checkin-account-1").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("checkin-account-1", cx);
        window.render_frame(cx);
        assert!(window.try_find("checkin-detail-cdk").is_none());
        window.click("checkin-detail-connection", cx);
        window.render_frame(cx);
        let notice = window
            .find("checkin-confirmation")
            .label()
            .unwrap()
            .to_owned();
        assert!(
            notice.contains("Desk account")
                && notice.contains("Fixture provider")
                && notice.contains("credentials")
        );
        window.click("checkin-entry-confirm", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |w, _| {
        w.try_find("checkin-operation-status")
            .and_then(|n| n.label().map(|s| s.contains("disabled in fixture mode")))
            == Some(true)
    })
    .await;
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn editing_an_account_removed_by_another_client_restores_the_account_list(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    let owner = RuntimeOwner::new().unwrap();
    let handle = open_size(cx, &owner, root.path(), 960., 640.);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-checkin", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |w, _| {
        w.try_find("checkin-account-1").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("checkin-account-1", cx);
        window.render_frame(cx);
        let db = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
        checkin_repo::delete_account(&db, "measured").unwrap();
        window.click(("checkin-edit-account", 1usize), cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |w, _| {
        w.try_find("checkin-credential-read-error").is_some()
            || w.try_find("checkin-account-0").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("checkin-editor").is_none());
        assert!(window.try_find("checkin-detail").is_none());
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("checkin-read").focused(), Some(true));
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}
