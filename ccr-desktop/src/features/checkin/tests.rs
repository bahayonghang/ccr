//! Rendered local check-in page. Provider calls and account writes stay out of this test.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::runtime::ConfigSource;
use crate::shell::Shell;
use ccr_app::runtime::RuntimeOwner;
use ccr_db::{
    database::{self, repositories::checkin_repo},
    models::checkin::{BalanceSnapshot, CheckinAccount, CheckinProvider},
};
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{AppContext, TestAppContext, WindowBounds, WindowOptions, px, size};
use std::time::Duration;

fn services(owner: &RuntimeOwner, root: &std::path::Path) -> Services {
    Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.join("profiles.toml"),
            registry: root.join("registry.toml"),
            delay: Duration::ZERO,
        },
    )
}

fn open(
    cx: &mut TestAppContext,
    owner: &RuntimeOwner,
    root: &std::path::Path,
) -> gpui_kit::AnyWindowHandle {
    open_size(cx, owner, root, 1280., 800.)
}

fn open_size(
    cx: &mut TestAppContext,
    owner: &RuntimeOwner,
    root: &std::path::Path,
    width: f32,
    height: f32,
) -> gpui_kit::AnyWindowHandle {
    cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::centered(
                    None,
                    size(px(width), px(height)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| {
                cx.new(|cx| {
                    Shell::new_with_preferences(
                        services(owner, root),
                        Ok(Default::default()),
                        window,
                        cx,
                    )
                })
            },
        )
        .unwrap()
        .0
    })
}

fn labels(window: &gpui_kit::Window) -> String {
    gpui_kit::base::test_support::snapshots(window)
        .into_iter()
        .filter_map(|node| node.label().map(str::to_string))
        .collect::<Vec<_>>()
        .join("\n")
}

fn at(stamp: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(stamp)
        .unwrap()
        .with_timezone(&chrono::Utc)
}

fn seed(root: &std::path::Path) {
    let home = root.join("checkin-migration-home");
    std::fs::create_dir_all(&home).unwrap();
    let pool = database::create_pool(&root.join("checkin.db"), None).unwrap();
    let conn = pool.get().unwrap();
    database::migrations::run_all_migrations(&conn, &home).unwrap();
    let mut provider = CheckinProvider::new(
        "Fixture provider".into(),
        "https://example.invalid/checkin-secret-sentinel".into(),
    );
    provider.id = "provider-1".into();
    checkin_repo::insert_provider(&conn, &provider).unwrap();

    let mut measured = CheckinAccount::new(
        provider.id.clone(),
        "Desk account".into(),
        "checkin-secret-sentinel".into(),
        "checkin-secret-sentinel".into(),
    );
    measured.id = "measured".into();
    measured.created_at = at("2024-01-01T00:00:00Z");
    measured.last_checkin_at = Some(at("2024-03-03T00:00:00Z"));
    measured.extra_config = r#"{"token":"checkin-secret-sentinel"}"#.into();
    checkin_repo::insert_account(&conn, &measured).unwrap();
    let mut balance = BalanceSnapshot::new(measured.id.clone(), 10.0, 2.5, 7.5, "USD".into());
    balance.recorded_at = at("2024-04-04T00:00:00Z");
    checkin_repo::insert_balance(&conn, &balance).unwrap();

    let mut unmeasured = CheckinAccount::new(
        provider.id,
        "Unmeasured account".into(),
        "checkin-secret-sentinel".into(),
        "checkin-secret-sentinel".into(),
    );
    unmeasured.id = "unmeasured".into();
    unmeasured.created_at = at("2024-02-02T00:00:00Z");
    checkin_repo::insert_account(&conn, &unmeasured).unwrap();
}

#[gpui_kit::test]
async fn checkin_page_without_a_database_stays_absent(cx: &mut TestAppContext) {
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
        window.try_find("checkin-absent").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.find("page-checkin").visible());
        assert!(window.try_find("nav-checkin").is_some());
        let text = labels(window);
        assert!(text.contains("No local check-in database was found."));
        assert!(text.contains("does not create a database"));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        assert!(!text.contains(" / "));
        assert!(window.try_find("checkin-account-0").is_none());
        assert!(labels(window).contains("Local check-ins"));
        for (tab, control) in [
            ("checkin-tab-providers", "checkin-new-provider"),
            ("checkin-tab-records", "checkin-records-empty"),
            ("checkin-tab-transfer", "checkin-transfer-path"),
            ("checkin-tab-accounts", "checkin-new-provider"),
        ] {
            window.click(tab, cx);
            window.render_frame(cx);
            assert!(window.try_find(control).is_some(), "{tab} opens its task");
        }
        let action = window.find("checkin-new-provider");
        assert!(action.bounds().size.width < px(250.));
        window.click("checkin-new-provider", cx);
        window.render_frame(cx);
        assert!(window.try_find("checkin-editor").is_some());
        for (index, label) in [
            (0, "Name"),
            (1, "Base URL"),
            (2, "Check-in path"),
            (3, "Balance path"),
            (4, "User info path"),
            (5, "Auth header"),
            (6, "Auth prefix"),
        ] {
            assert_eq!(
                window.find(format!("checkin-caption-{index}")).label(),
                Some(label)
            );
        }
        window.click("checkin-cancel-edit", cx);
        window.render_frame(cx);
        assert!(window.try_find("checkin-absent").is_some());
        assert!(!root.path().join("checkin-crypto").exists());
        assert!(!root.path().join("checkin-migration-home").exists());
        assert!(!root.path().join("checkin.db").exists());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn checkin_page_shows_stored_rows_and_hides_secrets(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-checkin", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("checkin-account-1").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let text = labels(window);
        assert!(text.contains("Unmeasured account"));
        assert!(text.contains("Desk account"));
        assert!(text.contains("No stored balance."));
        assert!(text.contains("No check-in is recorded for this account."));
        assert!(text.contains("7.5"));
        assert!(!text.contains("checkin-secret-sentinel"));
        assert!(!text.contains("example.invalid"));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        assert!(!text.contains(" / "));
        window.click("checkin-account-1", cx);
        window.render_frame(cx);
        let measured = labels(window);
        assert!(measured.contains("Desk account"));
        assert!(measured.contains("Provider: Fixture provider"));
        assert!(measured.contains("Stored remaining 7.5 USD."));
        assert!(measured.contains("Stored total 10."));
        assert!(measured.contains("Stored used 2.5."));
        assert!(measured.contains("2024-04-04T00:00:00+00:00"));
        assert!(measured.contains("Last check-in at 2024-03-03T00:00:00+00:00."));
        assert!(measured.contains("does not contact the provider"));
        assert!(!measured.contains("checkin-secret-sentinel"));
        assert!(!measured.contains("example.invalid"));
        assert!(!measured.contains(" / "));
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
fn account_tools_native_selection_cancel_and_batch_delete(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    {
        let conn = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
        checkin_repo::insert_record(
            &conn,
            &ccr_db::models::checkin::CheckinRecord::success("measured".into(), None, None),
        )
        .unwrap();
    }
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    #[track_caller]
    fn wait(
        cx: &mut TestAppContext,
        handle: gpui_kit::AnyWindowHandle,
        mut ready: impl FnMut(&mut Window) -> bool,
    ) {
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            cx.run_until_parked();
            if cx
                .update_window(handle, |_, window, cx| {
                    window.render_frame(cx);
                    ready(window)
                })
                .unwrap()
            {
                cx.run_until_parked();
                return;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "account tools did not reach expected state"
            );
            cx.executor().advance_clock(Duration::from_millis(10));
            std::thread::yield_now();
        }
    }
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-checkin", cx);
    })
    .unwrap();
    wait(cx, handle, |window| {
        window.try_find("checkin-account-1").is_some()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("checkin-tools-open", cx);
    })
    .unwrap();
    wait(cx, handle, |window| {
        window.try_find("checkin-select-1").is_some()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("checkin-select-0", cx);
        window.render_frame(cx);
        window.click("checkin-batch-delete", cx);
        window.render_frame(cx);
        assert!(window.try_find("checkin-tools-confirm").is_some());
        window.press("escape", cx);
        window.render_frame(cx);
        assert!(window.try_find("checkin-tools-confirm").is_none());
        assert!(window.try_find("checkin-select-1").is_some());
        window.click("checkin-batch-delete", cx);
        window.render_frame(cx);
        window.click("checkin-tools-confirm", cx);
    })
    .unwrap();
    wait(cx, handle, |window| {
        labels(window).contains("Deleted: 1; Failed: 0")
    });
    fn reveal(cx: &mut TestAppContext, handle: gpui_kit::AnyWindowHandle, id: &'static str) {
        for _ in 0..20 {
            let ready = cx
                .update_window(handle, |_, window, cx| {
                    window.render_frame(cx);
                    let bounds = window.find(id).bounds();
                    let page = window.find("checkin-page").bounds();
                    if bounds.center().y > page.origin.y + px(20.)
                        && bounds.center().y < page.origin.y + page.size.height - px(20.)
                    {
                        return true;
                    }
                    let delta = if bounds.center().y < page.origin.y + px(20.) {
                        400.
                    } else {
                        -400.
                    };
                    window.scroll(
                        "checkin-page",
                        gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(delta))),
                        cx,
                    );
                    window.render_frame(cx);
                    false
                })
                .unwrap();
            if ready {
                return;
            }
            cx.run_until_parked();
        }
        panic!("control did not scroll into view: {id}");
    }
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("checkin-select-0", cx);
        window.render_frame(cx);
    })
    .unwrap();
    reveal(cx, handle, "checkin-filter-success");
    cx.update_window(handle, |_, window, cx| {
        window.click("checkin-filter-success", cx);
        window.render_frame(cx);
    })
    .unwrap();
    reveal(cx, handle, "checkin-filter-records");
    cx.update_window(handle, |_, window, cx| {
        window.click("checkin-filter-records", cx);
    })
    .unwrap();
    wait(cx, handle, |window| {
        labels(window).contains("Matching records: 1")
    });
    reveal(cx, handle, "checkin-dashboard-read");
    cx.update_window(handle, |_, window, cx| {
        window.click("checkin-dashboard-read", cx);
    })
    .unwrap();
    wait(cx, handle, |window| {
        labels(window).contains("Checked days: 1")
    });
    reveal(cx, handle, "checkin-tools-close");
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("checkin-select-1").is_none());
        assert!(!labels(window).contains("checkin-secret-sentinel"));
        window.click("checkin-tools-close", cx);
        window.render_frame(cx);
        assert!(window.try_find("checkin-extended").is_none());
        window.click("checkin-tools-open", cx);
        window.render_frame(cx);
        assert!(labels(window).contains("Checked days: 1"));

        window.remove_window();
    })
    .unwrap();
    let conn = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
    assert_eq!(checkin_repo::get_all_accounts(&conn).unwrap().len(), 1);
    owner.shutdown();
}

#[gpui_kit::test]
async fn minimum_window_keeps_account_filters_and_unfiltered_balance_totals(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    let conn = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
    let mut other = CheckinProvider::new("Second vendor".into(), "https://second.invalid".into());
    other.id = "second".into();
    checkin_repo::insert_provider(&conn, &other).unwrap();
    conn.execute(
        "UPDATE checkin_accounts SET provider_id='second' WHERE id='unmeasured'",
        [],
    )
    .unwrap();
    let owner = RuntimeOwner::new().unwrap();
    for (w, h) in [(960., 640.), (1280., 800.)] {
        let handle = open_size(cx, &owner, root.path(), w, h);
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("nav-checkin", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |window, _| {
            window.try_find("checkin-account-1").is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let summary = window
                .find("checkin-balance-summary")
                .label()
                .unwrap()
                .to_string();
            assert!(summary.contains("USD") && summary.contains("7.5") && summary.contains("1/2"));
            window.click("checkin-search", cx);
            window.input("vEnDoR", cx);
            window.render_frame(cx);
            assert!(labels(window).contains("Unmeasured account"));
            assert!(
                !labels(window).contains("Desk account"),
                "{:?} {}",
                window.find("checkin-search").value(),
                labels(window)
            );
            window.click("checkin-provider-filter-0", cx);
            window.render_frame(cx);
            assert!(window.try_find("checkin-clear-filters").is_some());
            assert!(window.try_find("checkin-account-0").is_none());
            assert_eq!(
                window.find("checkin-balance-summary").label(),
                Some(summary.as_str())
            );
            window.click("checkin-clear-filters", cx);
            window.render_frame(cx);
            window.click("checkin-account-1", cx);
            window.render_frame(cx);
            assert!(
                window
                    .find("checkin-detail")
                    .label()
                    .unwrap()
                    .contains("Desk account")
            );
            window.click("checkin-back", cx);
            window.render_frame(cx);
            assert!(window.try_find("checkin-detail").is_none());
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("checkin-account-1").focused(), Some(true));
            window.press("enter", cx);
            window.render_frame(cx);
            assert!(
                window
                    .find("checkin-detail")
                    .label()
                    .unwrap()
                    .contains("Desk account")
            );
            window.click("checkin-back", cx);
            window.render_frame(cx);
            window.click("checkin-search", cx);
            window.input("DeSk", cx);
            window.render_frame(cx);
            assert!(labels(window).contains("Desk account"));
            assert!(!labels(window).contains("Unmeasured account"));
            assert_eq!(
                window.find("checkin-balance-summary").label(),
                Some(summary.as_str())
            );
            window.remove_window();
        })
        .unwrap();
    }
    owner.shutdown();
}

#[gpui_kit::test]
async fn unreadable_database_retries_into_empty_store_then_configured_provider(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let db = root.path().join("checkin.db");
    std::fs::create_dir(&db).unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open_size(cx, &owner, root.path(), 960., 640.);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-checkin", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window
            .try_find("checkin-status")
            .is_some_and(|n| n.label().is_some_and(|l| l.contains("could not be read")))
    })
    .await;
    std::fs::remove_dir(&db).unwrap();
    let conn = rusqlite::Connection::open(&db).unwrap();
    let migrations = tempfile::tempdir().unwrap();
    database::migrations::run_all_migrations(&conn, migrations.path()).unwrap();
    cx.update_window(handle, |_, window, cx| {
        assert!(window.try_find("checkin-absent").is_none());
        window.click("checkin-read", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("checkin-new-provider").is_some()
    })
    .await;
    let mut second = CheckinProvider::new(
        "Configured provider".into(),
        "https://example.invalid".into(),
    );
    second.id = "configured".into();
    checkin_repo::insert_provider(&conn, &second).unwrap();
    cx.update_window(handle, |_, window, cx| {
        assert!(
            window
                .find("checkin-empty")
                .label()
                .unwrap()
                .contains("No providers")
        );
        window.click("checkin-read", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("checkin-new-account").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert!(labels(window).contains("Configured provider"));
        let action = window.find("checkin-new-account");
        assert!(action.bounds().size.width < px(250.));
        assert!(action.bounds().bottom() <= window.find("checkin-page").bounds().bottom());
        window.click("checkin-new-account", cx);
        window.render_frame(cx);
        assert!(window.try_find("checkin-editor").is_some());
        assert!(labels(window).contains("Configured provider"));
        window.click("checkin-cancel-edit", cx);
        window.render_frame(cx);
        assert!(window.try_find("checkin-new-account").is_some());
        window.remove_window();
    })
    .unwrap();
    assert!(!root.path().join("checkin-crypto").exists());
    assert!(!root.path().join("checkin-migration-home").exists());
    owner.shutdown();
}

#[gpui_kit::test]
async fn provider_details_show_only_cache_status_and_locate_associated_accounts(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    let db = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
    db.execute(
        "UPDATE checkin_providers SET builtin_id='builtin-anyrouter' WHERE id='provider-1'",
        [],
    )
    .unwrap();
    checkin_repo::upsert_waf_cookies(
        &db,
        "provider-1",
        r#"{"cached":"cache-secret-sentinel"}"#,
        at("2020-01-01T00:00:00Z"),
        at("2020-01-01T00:01:00Z"),
    )
    .unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-checkin", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("checkin-account-0").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("checkin-tab-providers", cx);
        window.render_frame(cx);
        window.click("checkin-provider-info-0", cx);
        window.render_frame(cx);
        assert!(
            window
                .find("checkin-provider-status")
                .label()
                .unwrap()
                .contains("Missing or expired")
        );
        assert!(!labels(window).contains("cache-secret-sentinel"));
        window.click("checkin-provider-accounts-0", cx);
        window.render_frame(cx);
        assert!(window.try_find("checkin-account-1").is_some());
        window.remove_window();
    })
    .unwrap();
    assert_eq!(
        db.query_row("SELECT COUNT(*) FROM checkin_waf_cookies", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    owner.shutdown();
}

#[path = "management_tests.rs"]
mod management;
