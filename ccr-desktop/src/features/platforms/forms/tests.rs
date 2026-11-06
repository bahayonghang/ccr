#![allow(clippy::unwrap_used)]
use super::*;
use crate::runtime::ConfigSource;
use ccr_app::runtime::RuntimeOwner;
use ccr_cli::application::platform_roots::PlatformRoots;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{TestAppContext, WindowOptions};
use std::{fs, time::Duration};

#[gpui_kit::test]
async fn ordinary_settings_forms_write_dirty_fields_keep_conflicts_and_clear(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let fixture = root.path().canonicalize().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let source = ConfigSource::Fixture {
        profiles: fixture.join("profiles.toml"),
        registry: fixture.join("registry.toml"),
        delay: Duration::ZERO,
    };
    let roots = PlatformRoots::fixture(&fixture.join("desktop-platform-fixture")).unwrap();
    let cases = [
        (
            Platform::Claude,
            roots.claude.settings_file.clone(),
            r#"{"model":"synthetic-old","extension":7,"env":{"TOKEN":"settings-secret"}}"#,
        ),
        (
            Platform::Codex,
            roots.codex_runtime.join("config.toml"),
            "model='synthetic-old'\nextension=7\nsecret='settings-secret'\n",
        ),
        (
            Platform::Grok,
            roots.grok_runtime.join("config.toml"),
            "extension=7\nsecret='settings-secret'\n[models]\ndefault='synthetic-old'\n",
        ),
        (
            Platform::OpenCode,
            roots.home.join(".config/opencode/opencode.json"),
            r#"{"model":"synthetic-old","extension":7,"secret":"settings-secret","snapshot":false}"#,
        ),
    ];
    cx.update(gpui_kit::init);
    for (platform, path, original) in cases {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, original).unwrap();
        let services = Services::new(owner.handle(), source.clone());
        let (handle, view) = cx.update(|cx| {
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(gpui_kit::WindowBounds::Windowed(
                        gpui_kit::Bounds::centered(None, gpui_kit::size(px(1280.), px(800.)), cx),
                    )),
                    ..Default::default()
                },
                cx,
                |window, cx| {
                    cx.new(|cx| {
                        let mut view = PlatformsView::new(services, window, cx);
                        view.open(platform, Surface::Settings, window, cx);
                        view
                    })
                },
            )
            .unwrap()
        });
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        let id = if platform == Platform::Grok {
            "platform-setting-default_model"
        } else {
            "platform-setting-model"
        };
        let attempted = cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                assert!(window.try_find("platform-common-form").is_some());
                assert_eq!(window.find(id).value(), Some("synthetic-old"));
                assert!(window.try_find("platform-content").is_none());
                window.click(id, cx);
                window.press("ctrl-a", cx);
                window.input("synthetic-new", cx);
                assert_eq!(
                    window.find(id).value(),
                    Some("synthetic-new"),
                    "{platform:?} field bounds {:?}, viewport {:?}",
                    window.find(id).bounds(),
                    window.viewport_size()
                );
                if platform == Platform::OpenCode {
                    window.click("platform-setting-snapshot-true", cx);
                }
                let attempted = (
                    window.find("platform-save").bounds(),
                    window.viewport_size(),
                );
                window.click("platform-save", cx);
                attempted
            })
            .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        let saved = fs::read_to_string(&path).unwrap();
        let state = cx.update(|cx| view.read(cx).message.clone());
        assert!(
            saved.contains("synthetic-new"),
            "{platform:?}: {saved}; status {state:?}; Save/viewport {attempted:?}"
        );
        assert!(saved.contains("settings-secret"));
        assert!(saved.contains('7'));
        if platform == Platform::OpenCode {
            assert_eq!(
                serde_json::from_str::<Value>(&saved).unwrap()["snapshot"],
                true
            );
        }
        cx.update(|cx| view.update(cx, |view, cx| view.load(cx)));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        fs::write(&path, format!("{saved}\n")).unwrap();
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click(id, cx);
            window.press("ctrl-a", cx);
            window.input("retained-draft", cx);
            window.click("platform-save", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(
                view.read(cx)
                    .message
                    .as_ref()
                    .unwrap()
                    .contains("Source changed")
            );
            assert_eq!(window.find(id).value(), Some("retained-draft"));
            view.update(cx, |view, cx| view.leave(window, cx));
            assert!(view.read(cx).form.fields.is_empty());
            assert!(view.read(cx).content.read(cx).value().is_empty());
            window.remove_window();
        })
        .unwrap();
        assert!(
            !fs::read_to_string(&path)
                .unwrap()
                .contains("retained-draft")
        );
    }
    owner.shutdown();
}

#[gpui_kit::test]
async fn provider_common_fields_keep_stored_keys_and_metadata_omissions(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let fixture = root.path().canonicalize().unwrap();
    let roots = PlatformRoots::fixture(&fixture.join("desktop-platform-fixture")).unwrap();
    fs::create_dir_all(&roots.codex_store).unwrap();
    let timestamp = "2026-11-06T12:00:00Z";
    let initial = json!({"version":1,"providers":[{"id":"synthetic","name":"Synthetic","base_url":"https://synthetic.invalid/v1","website_url":"https://synthetic.invalid","api_keys":[{"id":"key","name":"Synthetic","api_key":"provider-stored-secret","created_at":timestamp,"updated_at":timestamp}],"created_at":timestamp,"updated_at":timestamp}]});
    let path = roots.codex_store.join("model_providers.json");
    fs::write(&path, initial.to_string()).unwrap();
    let source = ConfigSource::Fixture {
        profiles: fixture.join("profiles.toml"),
        registry: fixture.join("registry.toml"),
        delay: Duration::ZERO,
    };
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(gpui_kit::WindowBounds::Windowed(
                    gpui_kit::Bounds::centered(None, gpui_kit::size(px(1280.), px(800.)), cx),
                )),
                ..Default::default()
            },
            cx,
            |window, cx| {
                cx.new(|cx| {
                    let mut view =
                        PlatformsView::new(Services::new(owner.handle(), source), window, cx);
                    view.open(Platform::Codex, Surface::Providers, window, cx);
                    view
                })
            },
        )
        .unwrap()
    });
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("platform-provider-id", cx);
        window.input("synthetic", cx);
        window.click("platform-provider-load-fields", cx);
        assert_eq!(
            window.find("platform-setting-name").value(),
            Some("Synthetic")
        );
        assert_eq!(
            window.find("platform-setting-base_url").value(),
            Some("https://synthetic.invalid/v1")
        );
        assert!(
            view.read(cx)
                .form
                .fields
                .iter()
                .find(|field| field.key == "api_key")
                .unwrap()
                .input
                .read(cx)
                .value()
                .is_empty()
        );
        assert!(
            gpui_kit::base::test_support::snapshots(window)
                .iter()
                .all(|node| node
                    .value()
                    .is_none_or(|v| !v.contains("provider-stored-secret")))
        );
    })
    .unwrap();
    // Newly mounted inputs schedule their engine synchronization after render.
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("platform-setting-name", cx);
        window.press("ctrl-a", cx);
        window.input("Renamed", cx);
        assert_eq!(
            window.find("platform-setting-name").value(),
            Some("Renamed")
        );
        window.click("platform-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    let stored: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let state = cx.update(|cx| view.read(cx).message.clone());
    assert_eq!(stored["providers"][0]["name"], "Renamed", "{state:?}");
    assert_eq!(
        stored["providers"][0]["base_url"],
        initial["providers"][0]["base_url"]
    );
    assert_eq!(
        stored["providers"][0]["website_url"],
        initial["providers"][0]["website_url"]
    );
    assert_eq!(
        stored["providers"][0]["api_keys"][0]["api_key"],
        "provider-stored-secret"
    );
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| view.leave(window, cx));
        assert!(view.read(cx).form.fields.is_empty());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}
