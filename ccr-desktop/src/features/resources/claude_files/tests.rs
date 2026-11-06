#![allow(clippy::unwrap_used)]
use crate::{
    runtime::{ConfigSource, Services},
    shell::Shell,
};
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    AppContext, Bounds, ElementId, TestAppContext, Window, WindowBounds, WindowOptions, px, size,
};
use std::{
    fs,
    time::{Duration, Instant},
};

fn wait(
    cx: &mut TestAppContext,
    handle: gpui_kit::AnyWindowHandle,
    mut ready: impl FnMut(&mut Window) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        cx.run_until_parked();
        let found = cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                ready(window)
            })
            .unwrap();
        if found {
            // Input's render queues its engine update separately from the outer control.
            cx.run_until_parked();
            cx.update_window(handle, |_, window, cx| window.render_frame(cx))
                .unwrap();
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Claude resource UI did not reach the expected state"
        );
        cx.executor().advance_clock(Duration::from_millis(10));
        std::thread::yield_now();
    }
}
fn content(window: &Window) -> gpui_kit::test::ElementSnapshot {
    gpui_kit::base::test_support::snapshots(window)
        .into_iter()
        .find(|node| node.label() == Some("Claude resource content"))
        .unwrap()
}
fn content_id(window: &Window) -> ElementId {
    content(window).path().last().unwrap().clone()
}
fn replace(window: &mut Window, value: &str, cx: &mut gpui_kit::App) {
    window.click(content_id(window), cx);
    window.press(
        if cfg!(target_os = "macos") {
            "cmd-a"
        } else {
            "ctrl-a"
        },
        cx,
    );
    window.input(value, cx);
    assert_eq!(content(window).value(), Some(value));
}
fn editor_ready(window: &mut Window) -> bool {
    window
        .try_find("claude-resource-save")
        .is_some_and(|node| node.disabled() != Some(true))
}

#[gpui_kit::test]
fn shell_claude_resources_write_confirm_conflict_and_keep_private_fields(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let services = Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        },
    );
    let source = root.path().join("desktop-resource-fixture/home/.claude");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("settings.json"), r#"{"env":{"KEEP":"private"},"plugins":[{"name":"fixture-plugin","id":"fixture","version":"1","config":{"token":"resource-secret"},"unknown":7}]}"#).unwrap();
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1280.), px(1000.)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| Shell::new(services, window, cx)),
        )
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-claude-code", cx);
        window.click("tool-files", cx);
    })
    .unwrap();
    wait(cx, handle, |window| {
        window
            .try_find("resource-claude")
            .is_some_and(|node| node.disabled() != Some(true))
    });
    cx.update_window(handle, |_, window, cx| window.click("resource-claude", cx))
        .unwrap();
    wait(cx, handle, |window| {
        window
            .try_find("claude-resource-create")
            .is_some_and(|node| node.disabled() != Some(true))
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("claude-resource-name", cx);
        window.input("fixture.md", cx);
        window.click("claude-resource-create", cx);
    })
    .unwrap();
    wait(cx, handle, editor_ready);
    let rule = source.join("rules/fixture.md");
    cx.update_window(handle, |_, window, cx| {
        replace(window, "original private rule", cx);
        window.click("claude-resource-save", cx);
        assert!(!rule.exists());
        window.click("claude-resource-cancel", cx);
        assert_eq!(content(window).value(), Some("original private rule"));
        window.click("claude-resource-save", cx);
        window.click("claude-resource-confirm", cx);
    })
    .unwrap();
    wait(cx, handle, |window| {
        window
            .try_find("claude-resource-row-0")
            .is_some_and(|node| node.disabled() != Some(true))
    });
    assert_eq!(fs::read_to_string(&rule).unwrap(), "original private rule");
    cx.update_window(handle, |_, window, cx| {
        window.click("claude-resource-row-0", cx)
    })
    .unwrap();
    wait(cx, handle, editor_ready);
    fs::write(&rule, "external rule").unwrap();
    cx.update_window(handle, |_, window, cx| {
        replace(window, "retained draft", cx);
        window.click("claude-resource-save", cx);
        window.click("claude-resource-confirm", cx);
    })
    .unwrap();
    wait(cx, handle, |window| {
        window
            .try_find("claude-resource-status")
            .and_then(|node| node.label().map(str::to_owned))
            .is_some_and(|label| label.contains("The source changed."))
    });
    assert_eq!(fs::read_to_string(&rule).unwrap(), "external rule");
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(content(window).value(), Some("retained draft"));
        window.click("resource-instructions", cx);
        assert!(window.try_find("claude-resource-save").is_some());
        assert_eq!(content(window).value(), Some("retained draft"));
        window.click("claude-resource-discard", cx);
        window.click("claude-resource-confirm", cx);
    })
    .unwrap();
    wait(cx, handle, |window| {
        window
            .try_find("claude-resource-row-0")
            .is_some_and(|node| node.disabled() != Some(true))
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("claude-resource-row-0", cx)
    })
    .unwrap();
    wait(cx, handle, editor_ready);
    cx.update_window(handle, |_, window, cx| {
        window.click("claude-resource-delete", cx);
        window.click("claude-resource-confirm", cx);
    })
    .unwrap();
    wait(cx, handle, |window| {
        window.try_find("claude-resource-empty").is_some()
    });
    assert!(!rule.exists());
    assert!(
        fs::read_dir(
            root.path()
                .join("desktop-resource-fixture/ccr/backups/claude-resources")
        )
        .unwrap()
        .next()
        .is_some()
    );

    cx.update_window(handle, |_, window, cx| {
        window.click("claude-resource-plugins", cx)
    })
    .unwrap();
    wait(cx, handle, |window| {
        window
            .try_find("claude-resource-row-0")
            .is_some_and(|node| node.disabled() != Some(true))
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("claude-resource-row-0", cx)
    })
    .unwrap();
    wait(cx, handle, editor_ready);
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(
            content(window).value(),
            Some("{}"),
            "stored private values must not enter patch controls"
        );
        replace(window, r#"{"enabled":false}"#, cx);
        window.click("claude-resource-save", cx);
        window.click("claude-resource-confirm", cx);
    })
    .unwrap();
    wait(cx, handle, |window| {
        window
            .try_find("claude-resource-row-0")
            .is_some_and(|node| node.disabled() != Some(true))
    });
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(source.join("settings.json")).unwrap()).unwrap();
    assert_eq!(saved["plugins"][0]["enabled"], false);
    assert_eq!(saved["plugins"][0]["config"]["token"], "resource-secret");
    assert_eq!(saved["plugins"][0]["unknown"], 7);
    assert_eq!(saved["env"]["KEEP"], "private");
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    owner.shutdown();
}
