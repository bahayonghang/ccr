#![allow(clippy::unwrap_used)]
use super::*;
use crate::runtime::{ConfigSource, mcp::FIXTURE_DIR};
use ccr_app::runtime::{EnvironmentId, Platform, RuntimeOwner};
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{InputEvent, TestAppContext, WindowOptions};
use std::{fs, time::Duration};

#[gpui_kit::test]
async fn mcp_common_form_switches_http_to_stdio_without_stale_transport_type(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let fixture = root.path().canonicalize().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let home = fixture.join(FIXTURE_DIR).join("home");
    let path = home.join(".claude.json");
    fs::create_dir_all(&home).unwrap();
    fs::write(&path, serde_json::json!({"mcpServers":{"transport":{"type":"http","url":"https://synthetic.invalid/mcp","extension":7}}}).to_string()).unwrap();
    let source = ConfigSource::Fixture {
        profiles: fixture.join("profiles.toml"),
        registry: fixture.join("registry.toml"),
        delay: Duration::ZERO,
    };
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                McpEditor::new(
                    Services::new(owner.handle(), source),
                    RequestKey {
                        environment: EnvironmentId::Local("fixture".into()),
                        platform: Platform::Claude,
                        generation: 0,
                        request_id: 1,
                    },
                    McpTarget::ClaudeUser,
                    EditMode::Edit,
                    "transport",
                    window,
                    cx,
                )
            })
        })
        .unwrap()
    });
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("mcp-command", cx);
        window.input("node", cx);
        window.click("mcp-editor-save", cx);
        window.click("mcp-confirm-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let server = &saved["mcpServers"]["transport"];
    assert_eq!(server["command"], "node");
    assert!(server.get("url").is_none());
    assert!(
        server.get("type").is_none(),
        "stdio must not retain an explicit HTTP type"
    );
    assert_eq!(server["extension"], 7);
    let rows = ccr_app::claude_mcp::list_claude_mcp(&ccr_app::claude_mcp::ClaudeMcpContext {
        user_state_path: path,
        project_root: home,
        lock_directory: Some(fixture.join("locks")),
    })
    .unwrap();
    assert!(rows.servers.iter().any(|row| row.name == "transport"
        && row.command.as_deref() == Some("node")
        && row.server_type.is_none()));
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn mcp_common_form_validates_writes_http_and_retains_conflicting_private_drafts(
    cx: &mut TestAppContext,
) {
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
    let key = RequestKey {
        environment: EnvironmentId::Local("fixture".into()),
        platform: Platform::Codex,
        generation: 0,
        request_id: 1,
    };
    let path = root
        .path()
        .join(FIXTURE_DIR)
        .join("home/.codex/config.toml");
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
                    McpEditor::new(
                        services.clone(),
                        key.clone(),
                        McpTarget::Codex,
                        EditMode::Add,
                        "typed-http",
                        window,
                        cx,
                    )
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
        assert!(window.try_find("mcp-common-form").is_some());
        assert!(window.try_find("mcp-patch").is_none());
        window.click("mcp-url", cx);
        window.input("ftp://synthetic.invalid", cx);
        window.click("mcp-editor-save", cx);
        assert!(
            window
                .find("mcp-editor-message")
                .label()
                .unwrap()
                .contains("HTTP")
        );
        assert!(window.try_find("mcp-confirm-save").is_none());
        assert!(!path.exists());
        window.click("mcp-url", cx);
        window.press("ctrl-a", cx);
        window.input("https://synthetic.invalid/mcp", cx);
        window.click("mcp-header-add", cx);
        window.click("mcp-header-name-0", cx);
        window.input("X-New", cx);
        window.click("mcp-header-value-0", cx);
        window.input("replacement", cx);
        window.click("mcp-editor-save", cx);
        assert!(window.try_find("mcp-confirm-save").is_some());
        window.press("enter", cx);
        assert!(!path.exists(), "early Enter must not commit");
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("mcp-confirm-cancel").focused(), Some(true));
        window.press("tab", cx);
        assert_eq!(window.find("mcp-confirm-save").focused(), Some(true));
        window.press("tab", cx);
        assert_eq!(window.find("mcp-confirm-cancel").focused(), Some(true));
        window.press("shift-tab", cx);
        assert_eq!(window.find("mcp-confirm-save").focused(), Some(true));
        window.press("shift-tab", cx);
        assert_eq!(window.find("mcp-confirm-cancel").focused(), Some(true));
        window.press("escape", cx);
        assert!(window.try_find("mcp-confirm-save").is_none());
        assert_eq!(window.find("mcp-header-value-0").focused(), Some(true));
        window.click("mcp-editor-save", cx);
        window.click("mcp-confirm-cancel", cx);
        window.click("mcp-command", cx);
        let previous_focus = window.focused(cx);
        let position = window.find("mcp-editor-save").bounds().center();
        window.dispatch_event(
            gpui_kit::MouseDownEvent {
                button: gpui_kit::MouseButton::Left,
                position,
                modifiers: Default::default(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            gpui_kit::MouseUpEvent {
                button: gpui_kit::MouseButton::Left,
                position,
                modifiers: Default::default(),
                click_count: 1,
            }
            .to_platform_input(),
            cx,
        );
        window.simulate_next_frame(cx);
        assert_eq!(
            window.focused(cx),
            previous_focus,
            "stale entry stole focus"
        );
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("mcp-confirm-cancel").focused(), Some(true));
        window.click("mcp-confirm-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    let mut saved: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(
        saved["mcp_servers"]["typed-http"]["url"].as_str(),
        Some("https://synthetic.invalid/mcp")
    );
    assert_eq!(
        saved["mcp_servers"]["typed-http"]["http_headers"]["X-New"].as_str(),
        Some("replacement")
    );
    let server = saved["mcp_servers"]["typed-http"].as_table_mut().unwrap();
    server
        .get_mut("http_headers")
        .unwrap()
        .as_table_mut()
        .unwrap()
        .insert(
            "Authorization".into(),
            toml::Value::String("stored-secret".into()),
        );
    server.insert(
        "args".into(),
        toml::Value::Array(vec![toml::Value::String("stored-argument".into())]),
    );
    server.insert("extension".into(), toml::Value::Integer(7));
    server.insert(
        "env".into(),
        toml::Value::try_from(serde_json::json!({"TOKEN":"stored-env-secret"})).unwrap(),
    );
    fs::write(&path, toml::to_string(&saved).unwrap()).unwrap();
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| {
            *view = McpEditor::new(
                services.clone(),
                key.clone(),
                McpTarget::Codex,
                EditMode::Edit,
                "typed-http",
                window,
                cx,
            );
        });
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(view.read(cx).form.url.read(cx).value().is_empty());
        assert_eq!(
            window.find("mcp-url").value(),
            None,
            "secret inputs must omit their accessibility value"
        );
        assert!(
            gpui_kit::base::test_support::snapshots(window)
                .iter()
                .all(|node| {
                    node.value().is_none_or(|v| {
                        !v.contains("stored-secret") && !v.contains("stored-argument")
                    })
                })
        );
        window.click("mcp-header-add", cx);
        window.click("mcp-header-name-0", cx);
        window.input("Authorization", cx);
        window.click("mcp-env-add", cx);
        window.click("mcp-env-name-0", cx);
        window.input("TOKEN", cx);
        // Empty replacement values retain existing credentials; Advanced owns clearing.
        window.click("mcp-editor-save", cx);
        window.click("mcp-confirm-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    let retained: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(
        retained["mcp_servers"]["typed-http"]["http_headers"]["Authorization"].as_str(),
        Some("stored-secret")
    );
    assert_eq!(
        retained["mcp_servers"]["typed-http"]["env"]["TOKEN"].as_str(),
        Some("stored-env-secret")
    );
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| {
            *view = McpEditor::new(
                services,
                key,
                McpTarget::Codex,
                EditMode::Edit,
                "typed-http",
                window,
                cx,
            );
        });
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("mcp-header-add", cx);
        window.click("mcp-header-name-0", cx);
        window.input("X-Later", cx);
        window.click("mcp-header-value-0", cx);
        window.input("retained-private-draft", cx);
    })
    .unwrap();
    let original = fs::read_to_string(&path).unwrap();
    fs::write(&path, format!("{original}\n# external change\n")).unwrap();
    cx.update_window(handle, |_, window, cx| {
        window.click("mcp-editor-save", cx);
        window.click("mcp-confirm-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window
                .find("mcp-editor-message")
                .label()
                .unwrap()
                .contains("source changed")
        );
        assert_eq!(
            view.read(cx).form.headers[0]
                .value
                .read(cx)
                .value()
                .as_ref(),
            "retained-private-draft"
        );
        window.remove_window();
    })
    .unwrap();
    let final_bytes = fs::read_to_string(&path).unwrap();
    assert!(final_bytes.contains("stored-secret"));
    assert!(final_bytes.contains("stored-argument"));
    assert!(final_bytes.contains("extension = 7"));
    assert!(!final_bytes.contains("retained-private-draft"));
    owner.shutdown();
}
