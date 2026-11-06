//! Rendered sync controls use isolated canonical configuration files.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::runtime::ConfigSource;
use crate::shell::Shell;
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{AppContext, TestAppContext, WindowBounds, WindowOptions, px, size};
use std::time::Duration;

#[gpui_kit::test]
async fn saved_settings_and_operation_results_survive_leaving_the_view(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let services = services(&owner, root.path());
    let retained = services.clone();
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::centered(
                    None,
                    size(px(1280.), px(1200.)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| SyncView::new(services, window, cx)),
        )
        .unwrap()
    });
    view.update(cx, |this, cx| this.refresh(cx));
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).snapshot.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("sync-edit", cx);
        window.render_frame(cx);
        assert!(window.find("sync-editor").visible());
        let editor = view.read(cx).editor.as_ref().unwrap();
        let url = editor.url.clone();
        let username = editor.username.clone();
        url.update(cx, |state, cx| {
            state.set_value("https://dav.example.test/", window, cx)
        });
        username.update(cx, |state, cx| state.set_value("fixture-user", window, cx));
        window.click("sync-save", cx);
        view.update(cx, |this, cx| this.suspend(cx));
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        !retained.sync_operation_state().busy
    })
    .await;
    view.update(cx, |this, cx| this.refresh(cx));
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        matches!(view.read(cx).snapshot, Some(SyncSnapshot::Present(_)))
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(labels(window).contains("WebDAV settings saved."));
        assert!(view.read(cx).editor.is_none());
        assert!(matches!(
            retained.sync_operation_state().result,
            Some(Ok(SyncOutcome::Saved))
        ));
        window.click("sync-test", cx);
        view.update(cx, |this, cx| this.suspend(cx));
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        !retained.sync_operation_state().busy
    })
    .await;
    view.update(cx, |this, cx| this.refresh(cx));
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(labels(window).contains("Remote sync is disabled for fixture data."));
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

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

#[gpui_kit::test]
async fn sync_page_without_a_canonical_file_stays_absent(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-sync", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("sync-absent").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.find("page-sync").visible());
        let text = labels(window);
        assert!(text.contains("No canonical sync config file was found."));
        assert!(text.contains("Legacy sync config is not read."));
        assert!(text.contains("Remote access was not tested."));
        assert!(!text.contains("jianguoyun"));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        assert!(!text.contains(" / "));
        assert!(!root.path().join("sync_folders.toml").exists());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn sync_page_shows_local_folders_without_credentials(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("sync_folders.toml"),
        r#"
version = "1.0"
[webdav]
enabled = true
url = "https://sync-user-sentinel:sync-password-sentinel@dav.example.test/dav/"
username = "sync-user-sentinel"
password = "sync-password-sentinel"
[[folders]]
name = "claude-fixture"
local_path = "~/.claude"
remote_path = "/ccr-sync/claude"
enabled = true
"#,
    )
    .unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-sync", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("sync-folder-0").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let text = labels(window);
        assert!(text.contains("A WebDAV URL is stored."));
        assert!(text.contains("A WebDAV username is stored."));
        assert!(text.contains("A WebDAV password is stored in the local file."));
        assert!(text.contains("Remote access was not tested."));
        assert!(text.contains("claude-fixture"));
        assert!(text.contains("enabled"));
        assert!(!text.contains("sync-user-sentinel"));
        assert!(!text.contains("sync-password-sentinel"));
        assert!(!text.contains("dav.example.test"));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        assert!(!text.contains(" / "));
        window.click("sync-pull", cx);
        window.render_frame(cx);
        assert!(window.find("sync-confirmation").visible());
        window.click("sync-read", cx);
        window.render_frame(cx);
        assert!(window.try_find("sync-confirmation").is_none());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn sync_page_reports_an_unreadable_file_without_its_secret(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("sync_folders.toml"),
        "password = \"sync-password-sentinel\"\n[webdav\n",
    )
    .unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-sync", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window
            .try_find("sync-status")
            .and_then(|node| {
                node.label()
                    .map(|label| label.contains("could not be read"))
            })
            .unwrap_or(false)
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let text = labels(window);
        assert!(text.contains("The canonical sync config could not be read."));
        assert!(text.contains("Remote access was not tested."));
        assert!(!text.contains("sync-password-sentinel"));
        assert!(!text.contains(" / "));
        assert!(window.try_find("sync-local").is_none());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn asset_controls_require_confirmation_and_clear_passphrase_on_leave(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("sync_folders.toml"),
        "[webdav]\nenabled=false\nurl=''\nusername=''\npassword=''\n",
    )
    .unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let services = services(&owner, root.path());
    let retained = services.clone();
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(crate::locales::Language::English);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::centered(
                    None,
                    size(px(1280.), px(1000.)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| SyncView::new(services, window, cx)),
        )
        .unwrap()
    });
    view.update(cx, |v, cx| v.refresh(cx));
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).snapshot.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.scroll(
            "sync-page",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(-1500.))),
            cx,
        );
        window.click("sync-assets-all", cx);
        window.render_frame(cx);
        assert!(window.find("sync-asset-editor").visible());
        view.read(cx)
            .asset_editor
            .as_ref()
            .unwrap()
            .passphrase
            .clone()
            .update(cx, |state, cx| {
                state.set_value("asset-passphrase-sentinel", window, cx)
            });
        window.render_frame(cx);
        assert!(!labels(window).contains("asset-passphrase-sentinel"));
        window.scroll(
            "sync-page",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(-1500.))),
            cx,
        );
        window.click("sync-asset-pull", cx);
        window.render_frame(cx);
        window.scroll(
            "sync-page",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(-1500.))),
            cx,
        );
        assert!(window.find("sync-asset-confirmation").visible());
        assert!(!retained.sync_operation_state().busy);
        window.click("sync-asset-confirm", cx);
        assert!(view.read(cx).asset_editor.is_none());
        view.update(cx, |v, cx| v.suspend(cx));
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        !retained.sync_operation_state().busy
    })
    .await;
    assert_eq!(
        retained.sync_operation_state().result,
        Some(Err(SyncError::FixtureNetwork))
    );
    view.update(cx, |v, cx| v.refresh(cx));
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.scroll(
            "sync-page",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(-1500.))),
            cx,
        );
        window.click("sync-assets-all", cx);
        window.render_frame(cx);
        view.update(cx, |v, cx| v.suspend(cx));
        assert!(view.read(cx).asset_editor.is_none());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}
