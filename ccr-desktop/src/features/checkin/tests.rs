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
    cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::centered(
                    None,
                    size(px(1280.), px(800.)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| Shell::new(services(owner, root), window, cx)),
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
        assert!(!text.contains("7.5"));
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
