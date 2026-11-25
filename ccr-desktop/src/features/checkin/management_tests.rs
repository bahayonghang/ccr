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

#[gpui_kit::test]
async fn account_tools_read_fresh_balances_from_current_accounts(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    let path = root.path().join("checkin.db");
    let db = rusqlite::Connection::open(&path).unwrap();
    db.pragma_update(None, "foreign_keys", false).unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        w.click("nav-checkin", cx);
    })
    .unwrap();
    wait(cx, handle, |w| w.try_find("checkin-tools-open").is_some());
    cx.update_window(handle, |_, w, cx| {
        w.click("checkin-tools-open", cx);
        w.render_frame(cx);
    })
    .unwrap();
    // A read must work even when writer initialization is unavailable.
    std::fs::write(
        root.path().join("checkin-migration-home/occupied"),
        "fixture",
    )
    .unwrap();
    for (id, unit, total) in [
        ("measured", " USD ", 10.),
        ("unmeasured", "USD", 0.),
        ("orphan", "EUR", 200.),
    ] {
        checkin_repo::insert_balance(
            &db,
            &BalanceSnapshot::new(id.into(), total, 0., total, unit.into()),
        )
        .unwrap();
    }
    cx.update_window(handle, |_, w, cx| w.click("checkin-balance-stats", cx))
        .unwrap();
    wait(cx, handle, |w| {
        w.try_find("checkin-balance-summary").and_then(|n| {
            n.label().map(|s| {
                s.contains("USD · 2/2 accounts") && s.contains("Balance 10") && !s.contains("EUR")
            })
        }) == Some(true)
    });
    cx.update_window(handle, |_, w, _| w.remove_window())
        .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn account_tools_read_waf_status_without_cleaning_or_initializing(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    let path = root.path().join("checkin.db");
    let db = rusqlite::Connection::open(&path).unwrap();
    checkin_repo::upsert_waf_cookies(
        &db,
        "provider-1",
        "invalid-private-value",
        at("2020-01-01T00:00:00Z"),
        at("2020-01-02T00:00:00Z"),
    )
    .unwrap();
    checkin_repo::upsert_waf_cookies(
        &db,
        "cf-provider-1",
        "must-never-parse-this-value",
        at("2020-01-01T00:00:00Z"),
        at("2099-01-01T00:00:00Z"),
    )
    .unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        w.click("nav-checkin", cx);
    })
    .unwrap();
    wait(cx, handle, |w| w.try_find("checkin-tools-open").is_some());
    cx.update_window(handle, |_, w, cx| {
        w.click("checkin-tools-open", cx);
        w.render_frame(cx);
    })
    .unwrap();
    cx.update_window(handle, |_, w, cx| w.click("checkin-waf-status", cx))
        .unwrap();
    wait(cx, handle, |w| {
        labels(w).contains("WAF · Missing or expired · Cloudflare · Cached")
    });
    cx.update_window(handle, |_, w, _| {
        assert!(!labels(w).contains("must-never-parse-this-value"))
    })
    .unwrap();
    assert_eq!(
        db.query_row("SELECT COUNT(*) FROM checkin_waf_cookies", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert!(!root.path().join("checkin-locks").exists());
    drop(db);
    std::fs::remove_file(&path).unwrap();
    cx.update_window(handle, |_, w, cx| w.click("checkin-waf-status", cx))
        .unwrap();
    wait(cx, handle, |w| w.try_find("checkin-absent").is_some());
    assert!(!path.exists());
    cx.update_window(handle, |_, w, _| w.remove_window())
        .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn provider_detail_reads_cf_cache_for_its_own_provider(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    let db = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
    db.execute(
        "UPDATE checkin_providers SET builtin_id='builtin-runawaytime' WHERE id='provider-1'",
        [],
    )
    .unwrap();
    let expires = at("2099-01-01T00:00:00Z");
    checkin_repo::upsert_waf_cookies(
        &db,
        "cf-provider-2",
        "unparsed-private-value",
        at("2020-01-01T00:00:00Z"),
        expires,
    )
    .unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        w.click("nav-checkin", cx);
    })
    .unwrap();
    wait(cx, handle, |w| w.try_find("checkin-tools-open").is_some());
    cx.update_window(handle, |_, w, cx| {
        w.click("checkin-tab-providers", cx);
        w.render_frame(cx);
        w.click("checkin-provider-info-0", cx);
        w.render_frame(cx);
        assert!(
            w.find("checkin-provider-status")
                .label()
                .unwrap()
                .contains("Missing or expired")
        );
    })
    .unwrap();
    checkin_repo::upsert_waf_cookies(
        &db,
        "cf-provider-1",
        "unparsed-private-value",
        at("2020-01-01T00:00:00Z"),
        expires,
    )
    .unwrap();
    cx.update_window(handle, |_, w, cx| w.click("checkin-read", cx))
        .unwrap();
    wait(cx, handle, |w| {
        w.try_find("checkin-provider-status")
            .and_then(|n| n.label().map(|s| s.contains("cache available")))
            == Some(true)
    });
    cx.update_window(handle, |_, w, _| {
        assert!(!labels(w).contains("unparsed-private-value"));
        w.remove_window();
    })
    .unwrap();
    owner.shutdown();
}
