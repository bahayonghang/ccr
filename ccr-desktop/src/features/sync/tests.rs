//! Rendered sync controls use isolated canonical configuration files.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::runtime::ConfigSource;
use crate::runtime::sync::SyncLocal;
use crate::shell::Shell;
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{AppContext, Focusable, TestAppContext, WindowBounds, WindowOptions, px, size};
use std::time::Duration;

struct SyncWithNeighbor {
    view: Entity<SyncView>,
}

impl Render for SyncWithNeighbor {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                Button::new("sync-neighbor")
                    .label("Neighbor")
                    .on_click(|_, _, _| {}),
            )
            .child(div().min_h_0().flex_1().child(self.view.clone()))
    }
}

#[gpui_kit::test]
async fn ssh_sync_unavailable_database_and_disconnect_controls_do_not_start_actions(
    cx: &mut TestAppContext,
) {
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let source = services(&owner, root.path());
    source.select_environment(ccr_app::runtime::EnvironmentId::Ssh(
        "synthetic-host".into(),
    ));
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(crate::locales::Language::English);
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
            |window, cx| cx.new(|cx| SyncView::new(source, window, cx)),
        )
        .unwrap()
    });
    view.update(cx, |this, _| {
        this.snapshot = Some(SyncSnapshot::Present(SyncLocal {
            revision: "synthetic-revision".into(),
            webdav_enabled: true,
            url: SyncUrl::Endpoint {
                origin: "https://fixture.invalid".into(),
                path_prefix: Some("/dav/".into()),
            },
            asset_remote_base: Some("/fixture-root".into()),
            username_present: true,
            password_present: true,
            folders: Vec::new(),
        }))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(labels(window).contains("Profile databases, disconnecting WebDAV"));
        assert!(
            labels(window)
                .contains("All-assets transfer includes CCR Platforms and requires Local.")
        );
        window.click("sync-disconnect", cx);
        window.click("sync-assets-all", cx);
        window.click("sync-asset-details-ccr-platforms", cx);
        window.render_frame(cx);
        assert!(window.try_find(("sync-asset", 0usize)).is_none());
        assert!(view.read(cx).asset_editor.is_none());
        assert!(view.read(cx).management_confirmation.is_none());
        assert!(!view.read(cx).operation.busy);
        window.click("sync-asset-back", cx);
        window.render_frame(cx);
        window.click("sync-asset-details-claude-settings", cx);
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.press("tab", cx);
        window.press("tab", cx);
        window.render_frame(cx);
        assert_eq!(window.find(("sync-asset", 1usize)).focused(), Some(true));
        window.press("enter", cx);
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("sync-asset-push").focused(), Some(true));
        window.press("tab", cx);
        window.press("enter", cx);
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        let input = asset_passphrase(&view, cx);
        assert!(input.read(cx).focus_handle(cx).is_focused(window));
        assert_eq!(
            window.find("sync-asset-transfer-environment").label(),
            Some("Execution environment · SSH")
        );
        window.click("sync-asset-force", cx);
        window.render_frame(cx);
        assert!(labels(window).contains("previous contents are backed up on the SSH host"));
        input.update(cx, |state, cx| {
            state.set_value("discarded-passphrase", window, cx)
        });
        window.focus(&input.read(cx).focus_handle(cx), cx);
        window.press("escape", cx);
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(view.read(cx).asset_editor.is_none());
        assert!(input.read(cx).value().is_empty());
        assert_eq!(window.find(("sync-asset", 1usize)).focused(), Some(true));
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn disconnect_and_batch_actions_preserve_versions_and_never_contact_fixture_remote(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("sync_folders.toml");
    let legacy = root.path().join("sync.toml");
    let owner = RuntimeOwner::new().unwrap();
    let mut config = ccr_sync::SyncFoldersConfig::default();
    config.webdav.url = "https://fixture.invalid/".into();
    config.webdav.username = "fixture-user".into();
    config.webdav.password = "private-fixture-password".into();
    for (name, enabled) in [("enabled", true), ("disabled", false)] {
        let mut folder = ccr_sync::SyncFolder::builder()
            .name(name)
            .local_path(root.path().join(name).to_str().unwrap())
            .remote_path(format!("/{name}"))
            .build()
            .unwrap();
        folder.enabled = enabled;
        config.folders.push(folder);
    }
    let mut value: toml::Value = toml::from_str(&toml::to_string(&config).unwrap()).unwrap();
    value
        .as_table_mut()
        .unwrap()
        .insert("future".into(), 42.into());
    value["webdav"]
        .as_table_mut()
        .unwrap()
        .insert("extension".into(), "preserved".into());
    std::fs::write(&path, toml::to_string(&value).unwrap()).unwrap();
    std::fs::write(&legacy, b"legacy-private-fixture-password").unwrap();
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        let (handle, harness) = gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::centered(
                    None,
                    size(px(1280.), px(1200.)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| {
                cx.new(|cx| SyncWithNeighbor {
                    view: cx.new(|cx| SyncView::new(services(&owner, root.path()), window, cx)),
                })
            },
        )
        .unwrap();
        (handle, harness.read(cx).view.clone())
    });
    view.update(cx, |this, cx| this.refresh(cx));
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).snapshot.is_some()
    })
    .await;
    for action in ["sync-push-all", "sync-pull-all"] {
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            view.update(cx, |view, cx| view.focus_main(window, cx));
            window.press("shift-tab", cx);
            window.render_frame(cx);
            assert_eq!(window.find("sync-neighbor").focused(), Some(true));
            window.click(action, cx);
            window.render_frame(cx);
            assert!(window.find("sync-management-confirmation").visible());
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("sync-management-cancel").focused(), Some(true));
            for (key, focused) in [
                ("tab", "sync-management-confirm"),
                ("tab", "sync-management-cancel"),
                ("shift-tab", "sync-management-confirm"),
                ("shift-tab", "sync-management-cancel"),
            ] {
                window.press(key, cx);
                window.render_frame(cx);
                assert_eq!(
                    window.find(focused).focused(),
                    Some(true),
                    "{action} · {key}"
                );
            }
            window.press("escape", cx);
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert!(!view.read(cx).operation.busy);
            assert!(window.try_find("sync-management-confirmation").is_none());
            assert_eq!(window.find(action).focused(), Some(true));
            window.click(action, cx);
            window.render_frame(cx);
            window.click("sync-management-confirm", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).operation.busy
        })
        .await;
        let state = cx.read(|cx| view.read(cx).operation.clone());
        let Some(Ok(SyncOutcome::Batch(rows))) = &state.result else {
            panic!("expected per-folder result")
        };
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "enabled");
        assert_eq!(rows[0].result, Err(SyncError::FixtureNetwork));
    }
    for attempt in 0..3 {
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("sync-disconnect", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).management_confirmation.is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(!labels(window).contains("private-fixture-password"));
            if attempt == 0 {
                window.click("sync-management-cancel", cx);
            } else {
                if attempt == 1 {
                    value["future"] = 43.into();
                    std::fs::write(&path, toml::to_string(&value).unwrap()).unwrap();
                }
                window.click("sync-management-confirm", cx);
            }
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).operation.busy && !view.read(cx).busy
        })
        .await;
        if attempt < 2 {
            assert_eq!(
                std::fs::read(&legacy).unwrap(),
                b"legacy-private-fixture-password"
            );
            let stored: toml::Value =
                toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            assert_eq!(
                stored["webdav"]["password"].as_str(),
                Some("private-fixture-password")
            );
            if attempt == 1 {
                cx.read(|cx| {
                    assert_eq!(
                        view.read(cx).operation.result,
                        Some(Err(SyncError::Conflict))
                    )
                });
                view.update(cx, |this, cx| this.refresh(cx));
                cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
                    .await;
            }
        }
    }
    assert!(!legacy.exists());
    let stored: toml::Value = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(stored["future"].as_integer(), Some(43));
    assert_eq!(stored["folders"], value["folders"]);
    assert_eq!(stored["webdav"]["extension"].as_str(), Some("preserved"));
    assert_eq!(stored["webdav"]["url"].as_str(), Some(""));
    assert_eq!(stored["webdav"]["password"].as_str(), Some(""));
    assert_eq!(stored["webdav"]["enabled"].as_bool(), Some(false));
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let Some(SyncSnapshot::Present(local)) = &view.read(cx).snapshot else {
            panic!("expected refreshed projection")
        };
        assert!(!local.password_present && !local.username_present && !local.webdav_enabled);
        assert!(labels(window).contains("WebDAV account disconnected."));
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

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
        view.update(cx, |this, cx| this.suspend(window, cx));
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
        view.update(cx, |this, cx| this.suspend(window, cx));
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
        assert!(labels(window).contains("Remote sync is disabled for fixture data."));
        assert!(
            window
                .try_find("sync-webdav-connection")
                .and_then(|node| node
                    .label()
                    .map(|label| label.contains("Connection test failed")))
                .unwrap_or(false),
            "explicit test failure should be visible in the WebDAV summary after returning"
        );
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
    viewport: gpui_kit::Size<gpui_kit::Pixels>,
) -> gpui_kit::AnyWindowHandle {
    cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::centered(
                    None, viewport, cx,
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

#[gpui_kit::test]
async fn sync_page_without_a_canonical_file_can_configure_and_return_focus_at_minimum_width(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path(), size(px(960.), px(640.)));
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
        assert!(window.try_find("sync-detail").is_none());
        assert_eq!(window.find("sync-edit").label(), Some("Configure WebDAV"));
        assert!(window.find("sync-edit").bounds().size.width < px(240.));
        window.click("sync-edit", cx);
        window.render_frame(cx);
        assert!(window.find("sync-editor").visible());
        assert!(window.try_find("sync-asset-list").is_none());
        for _ in 0..3 {
            window.press("tab", cx);
        }
        window.render_frame(cx);
        assert_eq!(window.find("sync-enabled").focused(), Some(true));
        window.press("escape", cx);
        window.render_frame(cx);
        assert!(window.try_find("sync-editor").is_none());
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("sync-edit").focused(), Some(true));

        window.click("sync-edit", cx);
        window.render_frame(cx);
        window.input("https://dav.example.test/", cx);
        window.press("tab", cx);
        window.input("isolated-fixture", cx);
        for _ in 0..6 {
            window.press("tab", cx);
        }
        window.render_frame(cx);
        assert_eq!(window.find("sync-save").focused(), Some(true));
        window.press("enter", cx);
        window.render_frame(cx);
        assert!(window.try_find("sync-operation-status").is_some());
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("sync-local").is_some() && window.try_find("sync-editor").is_none()
            || window
                .try_find("sync-operation-status")
                .and_then(|node| {
                    node.label()
                        .map(|label| label.starts_with("Invalid WebDAV"))
                })
                .unwrap_or(false)
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(
            window.try_find("sync-editor").is_none(),
            "{}",
            labels(window)
        );
        assert_eq!(window.find("sync-edit").focused(), Some(true));
        assert_eq!(
            window.find("sync-edit").label(),
            Some("Edit WebDAV settings")
        );
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
    let handle = open(cx, &owner, root.path(), size(px(1280.), px(800.)));
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
        assert!(text.contains("A WebDAV password is stored."));
        assert!(text.contains("Remote access was not tested."));
        assert!(text.contains("claude-fixture"));
        assert!(text.contains("enabled"));
        assert!(!text.contains("sync-user-sentinel"));
        assert!(!text.contains("sync-password-sentinel"));
        assert!(!text.contains("dav.example.test"));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        assert!(!text.contains(" / "));
        window.scroll(
            "sync-page",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(-520.))),
            cx,
        );
        window.click(("sync-select", 0usize), cx);
        window.render_frame(cx);
        window.click("sync-pull", cx);
        window.render_frame(cx);
        assert!(window.find("sync-confirmation").visible());
        for action in ["sync-edit", "sync-read", "sync-test", "sync-assets-local"] {
            assert_eq!(window.find(action).focused(), None, "{action}");
        }
        window.scroll(
            "sync-page",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(520.))),
            cx,
        );
        window.click("sync-read", cx);
        window.render_frame(cx);
        assert!(window.find("sync-confirmation").visible());
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.press("escape", cx);
        window.render_frame(cx);
        assert!(window.try_find("sync-confirmation").is_none());
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
    let handle = open(cx, &owner, root.path(), size(px(1280.), px(800.)));
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
        assert!(window.find("sync-detail").visible());
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("sync-asset-push").focused(), Some(true));
        assert!(window.find("sync-asset-editor").visible());
        for action in ["sync-edit", "sync-read", "sync-test", "sync-folder-add"] {
            assert_eq!(
                window.try_find(action).and_then(|action| action.focused()),
                None,
                "{action}"
            );
        }
        window.scroll(
            "sync-page",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(1500.))),
            cx,
        );
        window.click("sync-read", cx);
        window.render_frame(cx);
        assert!(!view.read(cx).busy);
        window.scroll(
            "sync-page",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(-1500.))),
            cx,
        );
        window.click("sync-asset-pull", cx);
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(window.find("sync-asset-confirmation").visible());
        let input = asset_passphrase(&view, cx);
        assert!(input.read(cx).focus_handle(cx).is_focused(window));
        input.update(cx, |state, cx| {
            state.set_value("asset-passphrase-sentinel", window, cx)
        });
        window.render_frame(cx);
        assert!(
            gpui_kit::base::test_support::snapshots(window)
                .iter()
                .all(|node| {
                    [node.label(), node.value()]
                        .into_iter()
                        .flatten()
                        .all(|text| !text.contains("asset-passphrase-sentinel"))
                })
        );
        assert!(!retained.sync_operation_state().busy);
        window.click("sync-asset-change-direction", cx);
        window.render_frame(cx);
        assert!(input.read(cx).value().is_empty());
        assert!(window.try_find("sync-asset-confirmation").is_none());
        window.click("sync-asset-pull", cx);
        window.render_frame(cx);
        let input = asset_passphrase(&view, cx);
        assert!(input.read(cx).value().is_empty());
        input.update(cx, |state, cx| {
            state.set_value("second-passphrase", window, cx)
        });
        window.click("sync-asset-confirm", cx);
        assert!(view.read(cx).asset_editor.is_none());
        assert!(input.read(cx).value().is_empty());
        window.render_frame(cx);
        assert_eq!(window.find("sync-operation-status").focused(), Some(true));
        view.update(cx, |v, cx| v.suspend(window, cx));
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
        window.click("sync-asset-pull", cx);
        window.render_frame(cx);
        let input = asset_passphrase(&view, cx);
        input.update(cx, |state, cx| {
            state.set_value("populated-draft-on-leave", window, cx)
        });
        view.update(cx, |v, cx| v.suspend(window, cx));
        assert!(view.read(cx).asset_editor.is_none());
        assert!(input.read(cx).value().is_empty());
    })
    .unwrap();
    view.update(cx, |v, cx| v.refresh(cx));
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("sync-assets-all", cx);
        window.render_frame(cx);
        window.click("sync-asset-pull", cx);
        window.render_frame(cx);
        assert!(asset_passphrase(&view, cx).read(cx).value().is_empty());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

fn asset_passphrase(view: &Entity<SyncView>, cx: &App) -> Entity<InputState> {
    view.read(cx)
        .asset_editor
        .as_ref()
        .unwrap()
        .review
        .as_ref()
        .unwrap()
        .passphrase
        .as_ref()
        .unwrap()
        .clone()
}

#[gpui_kit::test]
async fn asset_transfers_keep_scope_and_report_real_loopback_results(cx: &mut TestAppContext) {
    use crate::runtime::sync::test_support::asset_server;
    use std::sync::atomic::Ordering;

    cx.executor().allow_parking();
    let server_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .worker_threads(1)
        .build()
        .unwrap();
    let server = server_runtime.block_on(asset_server());
    let root = tempfile::tempdir().unwrap();
    let settings = root.path().join("sync_folders.toml");
    let config = format!(
        "[webdav]\nenabled=true\nurl='{}dav-team/'\nusername='fixture-user'\npassword='fixture-password'\nbase_remote_path='/fixture-team/assets/'\n",
        server.url
    );
    std::fs::write(&settings, &config).unwrap();
    let local = root
        .path()
        .join("desktop-resource-fixture/home/.claude/settings.json");
    std::fs::create_dir_all(local.parent().unwrap()).unwrap();
    std::fs::write(&local, b"fixture-original").unwrap();

    let owner = RuntimeOwner::new().unwrap();
    let retained = services(&owner, root.path()).with_sync_loopback_fixture();
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(crate::locales::Language::English);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::centered(
                    None,
                    size(px(960.), px(640.)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| SyncView::new(retained.clone(), window, cx)),
        )
        .unwrap()
    });
    view.update(cx, |view, cx| view.open("Local".into(), cx));
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).snapshot.is_some()
    })
    .await;

    let review = |window: &mut Window, cx: &mut App, all, direction| {
        window.render_frame(cx);
        if all {
            window.click("sync-assets-all", cx);
        } else {
            window.click("sync-asset-details-claude-settings", cx);
            window.render_frame(cx);
            window.click(("sync-asset", 1usize), cx);
        }
        window.render_frame(cx);
        assert!(window.find("sync-detail").visible());
        assert!(window.try_find("sync-asset-list").is_none());
        window.click(direction, cx);
        window.render_frame(cx);
        assert_eq!(window.find("sync-asset-force").checked(), Some(false));
        if let Some(legacy) = window.try_find("sync-asset-v1") {
            assert_eq!(legacy.checked(), Some(false));
        }
        let input = asset_passphrase(&view, cx);
        input.update(cx, |state, cx| {
            state.set_value("fixture-operation-passphrase", window, cx)
        });
    };
    let submit = |window: &mut Window, cx: &mut App| {
        window.scroll(
            "sync-page",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(-800.))),
            cx,
        );
        window.click("sync-asset-confirm", cx);
        assert!(view.read(cx).asset_editor.is_none());
    };
    let status = |window: &mut Window, cx: &mut App, expected| {
        window.render_frame(cx);
        assert_eq!(window.find("sync-operation-status").label(), Some(expected));
    };

    cx.update_window(handle, |_, window, cx| {
        review(window, cx, false, "sync-asset-push");
        assert_eq!(
            window.find("sync-asset-transfer-webdav").label(),
            Some(format!("WebDAV address · {}dav-team/", server.url).as_str())
        );
        assert_eq!(server.requests.load(Ordering::Relaxed), 0);
        assert!(window.try_find("sync-asset-v1").is_none());
        submit(window, cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).operation.busy
    })
    .await;
    let uploaded = server.settings.lock().unwrap().clone().unwrap();
    assert!(!String::from_utf8_lossy(&uploaded).contains("fixture-original"));
    std::fs::write(&local, b"fixture-edit").unwrap();

    cx.update_window(handle, |_, window, cx| {
        status(window, cx, "Asset transfer completed · 1/1");
        review(window, cx, false, "sync-asset-sync");
        assert!(
            window
                .find("sync-asset-transfer-consequence")
                .label()
                .unwrap()
                .contains("reports a conflict")
        );
        submit(window, cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).operation.busy
    })
    .await;
    assert_eq!(
        server.settings.lock().unwrap().as_deref(),
        Some(uploaded.as_slice())
    );
    assert_eq!(std::fs::read(&local).unwrap(), b"fixture-edit");

    cx.update_window(handle, |_, window, cx| {
        status(window, cx, "Asset transfer failed · 0/1");
        assert!(
            labels(window).contains("Claude Code · settings.json · Failed · Both copies exist.")
        );
        review(window, cx, false, "sync-asset-sync");
        window.click("sync-asset-force", cx);
        window.render_frame(cx);
        assert!(
            window
                .find("sync-asset-transfer-consequence")
                .label()
                .unwrap()
                .contains("current environment overwrites WebDAV")
        );
        submit(window, cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).operation.busy
    })
    .await;
    assert_ne!(
        server.settings.lock().unwrap().as_deref(),
        Some(uploaded.as_slice())
    );
    assert_eq!(std::fs::read(&local).unwrap(), b"fixture-edit");
    *server.settings.lock().unwrap() = Some(b"fixture-legacy".to_vec());

    cx.update_window(handle, |_, window, cx| {
        status(window, cx, "Asset transfer completed · 1/1");
        review(window, cx, true, "sync-asset-pull");
        assert!(labels(window).contains("CCR Platforms"));
        assert!(labels(window).contains("Codex · AGENTS.md"));
        window.click("sync-asset-force", cx);
        window.render_frame(cx);
        assert!(labels(window).contains("WebDAV asset base path · /fixture-team/assets"));
        assert!(
            labels(window)
                .contains("When a local file is replaced, its previous contents are backed up.")
        );
        assert!(labels(window).contains(
            "CCR Platforms downloads containing profiles.db use database transactions with rollback"
        ));
        window.scroll(
            "sync-page",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(-400.))),
            cx,
        );
        window.click("sync-asset-v1", cx);
        submit(window, cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).operation.busy
    })
    .await;
    assert_eq!(std::fs::read(&local).unwrap(), b"fixture-legacy");
    assert_eq!(
        server.settings.lock().unwrap().as_deref(),
        Some(b"fixture-legacy".as_slice())
    );
    let expected_requests = server.requests.load(Ordering::Relaxed);
    cx.update_window(handle, |_, window, cx| {
        status(window, cx, "Asset transfer partially completed · 1/5");
        let state = view.read(cx);
        let Some(Ok(SyncOutcome::Assets(rows))) = &state.operation.result else {
            panic!("asset report expected");
        };
        let backup = rows
            .iter()
            .find(|row| row.id == "claude-settings")
            .unwrap()
            .result
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap();
        assert_eq!(std::fs::read(backup).unwrap(), b"fixture-edit");
        assert!(labels(window).contains("Claude Code · settings.json · Completed · Backup:"));
        review(window, cx, false, "sync-asset-push");
        std::fs::write(&settings, format!("{config}# changed elsewhere\n")).unwrap();
        submit(window, cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).operation.busy
    })
    .await;
    assert_eq!(
        retained.sync_operation_state().result,
        Some(Err(SyncError::Conflict))
    );
    assert_eq!(server.requests.load(Ordering::Relaxed), expected_requests);
    assert!(
        !std::fs::read_to_string(settings)
            .unwrap()
            .contains("fixture-operation-passphrase")
    );
    retained.select_environment(EnvironmentId::Ssh("other-test-environment".into()));
    view.update(cx, |view, cx| view.open("Other SSH host".into(), cx));
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("sync-operation-status").is_none());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
    drop(server);
    server_runtime.shutdown_background();
}

#[gpui_kit::test]
async fn assets_can_be_inspected_without_webdav_and_narrow_details_return_focus(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let nested = root.path().join("long-asset-fixture".repeat(8));
    let claude = nested
        .join("desktop-resource-fixture")
        .join("home")
        .join(".claude");
    std::fs::create_dir_all(&claude).unwrap();
    let path = claude.join("settings.json");
    std::fs::write(&path, "{}").unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, &nested, size(px(960.), px(640.)));
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
        assert!(window.try_find("sync-asset-distribution").is_none());
        window.click("sync-assets-local", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window
            .try_find("sync-asset-details-claude-settings")
            .is_some_and(|row| {
                row.label()
                    .is_some_and(|label| label.contains("Environment: present"))
            })
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("sync-asset-distribution").is_none());
        assert!(!nested.join("sync_folders.toml").exists());
        use gpui_kit::{InputEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent};
        let position = window
            .find("sync-asset-details-claude-settings")
            .bounds()
            .center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position,
                modifiers: Default::default(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        window.render_frame(cx);
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position,
                modifiers: Default::default(),
                click_count: 1,
            }
            .to_platform_input(),
            cx,
        );
        // Native GPUI drains the queue before drawing the detail tree.
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(window.try_find("sync-asset-list").is_none());
        let rendered_path = window.find("sync-asset-environment-path");
        assert_eq!(rendered_path.label(), path.to_str());
        assert!(
            rendered_path.bounds().size.width <= window.find("sync-detail").bounds().size.width
        );
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("sync-asset-back").focused(), Some(true));
        assert!(window.find("sync-asset-back").bounds().size.width < px(260.));
        assert!(window.find("sync-asset-inspect").bounds().size.width < px(260.));
        window.dispatch_keystroke(Keystroke::parse("escape").unwrap(), cx);
        // The row is also absent up to the first list draw.
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("sync-asset-details-claude-settings").focused(),
            Some(true)
        );
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}
