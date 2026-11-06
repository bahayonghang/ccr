//! Rendered local MCP page. Native pixels and other operating systems stay out of scope.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::runtime::ConfigSource;
use crate::runtime::mcp::FIXTURE_DIR;
use crate::shell::Shell;
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{AppContext, TestAppContext, WindowBounds, WindowOptions, px, size};
use std::fs;
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

// Textarea exposes its accessibility label on the inner Input. Its native
// accessibility_id is not a GPUI ElementId, so drive the actual input node.
fn patch_id(window: &gpui_kit::Window) -> gpui_kit::ElementId {
    gpui_kit::base::test_support::snapshots(window)
        .into_iter()
        .find(|node| node.label() == Some("MCP JSON patch"))
        .unwrap()
        .path()
        .last()
        .unwrap()
        .clone()
}
async fn wait_for_editor(cx: &mut TestAppContext, handle: gpui_kit::AnyWindowHandle) {
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window
            .try_find("mcp-editor-save")
            .is_some_and(|node| node.disabled() != Some(true))
    })
    .await;
    // Rendering the enabled Input updates its engine and queues a notification.
    // Drain that update before exercising native pointer and keyboard handlers.
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        if window.try_find("mcp-common-form").is_some() {
            window.click("mcp-editor-advanced", cx);
        }
    })
    .unwrap();
}
fn write_entries(root: &std::path::Path) {
    let home = root.join(FIXTURE_DIR).join("home");
    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::create_dir_all(home.join(".codex")).unwrap();
    fs::create_dir_all(home.join(".gemini").join("antigravity-cli")).unwrap();
    fs::write(
        home.join(".claude.json"),
        r#"{"oauth":"mcp-secret-sentinel","mcpServers":{"state-server":{"command":"fixture-mcp-command","args":["mcp-secret-sentinel"],"env":{"TOKEN":"mcp-secret-sentinel"}}},"projects":{"fixture-project":{"mcpServers":{"project-server":{"url":"https://example.test/?token=mcp-secret-sentinel"}}}}}"#,
    )
    .unwrap();
    fs::write(
        home.join(".claude").join("settings.json"),
        r#"{"apiKey":"mcp-secret-sentinel","mcpServers":{"settings-server":{"command":"fixture-mcp-command","disabled":true}}}"#,
    )
    .unwrap();
    fs::write(
        home.join(".codex").join("config.toml"),
        "[mcp_servers.codex-server]\ncommand = 'fixture-mcp-command'\nenabled = false\nbearer_token = 'mcp-secret-sentinel'\n",
    )
    .unwrap();
    fs::write(
        home.join(".gemini").join("antigravity-cli").join("mcp_config.json"),
        r#"{"mcpServers":{"agy-server":{"command":"agy","headers":{"Authorization":"mcp-secret-sentinel"}}}}"#,
    )
    .unwrap();
}

#[gpui_kit::test]
async fn mcp_page_lists_real_entries_and_hides_secret_fields(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    write_entries(root.path());
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-mcp-manager", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window.try_find("mcp-entry-4").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.find("page-mcp-manager").visible());
        assert!(window.find("tool-converter").visible());
        assert!(window.try_find("nav-converter").is_none());
        let text = labels(window);
        assert!(text.contains("state-server"));
        assert!(text.contains("project-server"));
        assert!(text.contains("settings-server"));
        assert!(text.contains("codex-server"));
        assert!(text.contains("agy-server"));
        assert!(text.contains("disabled in the file"));
        assert!(text.contains("Claude state file: 2 entries."));
        assert!(text.contains("Claude settings.json: 1 entry."));
        assert!(text.contains("Codex config.toml: 1 entry."));
        assert!(text.contains("Antigravity mcp_config.json: 1 entry."));
        assert!(text.contains("Claude project .mcp.json is not read"));
        assert!(text.contains("OpenCode MCP is not read"));
        assert!(text.contains("Grok MCP credentials are not listed as servers"));
        assert!(!text.contains("mcp-secret-sentinel"));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        assert!(!text.contains("no MCP manager connection"));
        window.click("mcp-entry-1", cx);
        window.render_frame(cx);
        let detail = labels(window);
        assert!(detail.contains("Command: fixture-mcp-command"));
        assert!(detail.contains("Arguments are set in this entry."));
        assert!(!detail.contains("mcp-secret-sentinel"));
        assert!(!detail.contains("https://example.test"));
        window.click("mcp-entry-0", cx);
        window.render_frame(cx);
        let project = labels(window);
        assert!(project.contains("fixture-project"));
        assert!(project.contains("A URL is set in this entry."));
        assert!(!project.contains("https://"));
        assert!(!project.contains("example.test"));
        assert!(!project.contains("mcp-secret-sentinel"));
        window.click("mcp-entry-2", cx);
        window.render_frame(cx);
        let settings = labels(window);
        assert!(settings.contains("disabled is true in the file."));
        assert!(!settings.contains("TOKEN"));
        assert!(!settings.contains("apiKey"));
        assert!(!settings.contains("mcp-secret-sentinel"));
        window.click("mcp-entry-3", cx);
        window.render_frame(cx);
        let codex_detail = labels(window);
        assert!(codex_detail.contains("enabled is false in the file."));
        assert!(codex_detail.contains("Command: fixture-mcp-command"));
        assert!(!codex_detail.contains("bearer"));
        assert!(!codex_detail.contains("mcp-secret-sentinel"));
        window.click("mcp-entry-4", cx);
        window.render_frame(cx);
        let agy = labels(window);
        assert!(agy.contains("Command: agy"));
        assert!(!agy.contains("Authorization"));
        assert!(!agy.contains("mcp-secret-sentinel"));
        window.click("mcp-filter", cx);
        window.input("codex-server", cx);
        window.render_frame(cx);
        assert!(
            window
                .find("mcp-entry-0")
                .label()
                .unwrap()
                .contains("codex-server")
        );
        assert!(window.try_find("mcp-entry-1").is_none());
        assert!(window.try_find("mcp-detail").is_none());
        let filtered = labels(window);
        assert!(!filtered.contains("fixture-project"));
        assert!(!filtered.contains("Command: agy"));
        assert!(!filtered.contains("https://"));
        assert!(!filtered.contains("mcp-secret-sentinel"));
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn mcp_page_with_no_files_stays_empty(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-mcp-manager", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window.try_find("mcp-empty").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let text = labels(window);
        assert!(text.contains("No MCP entries were found in the files this page reads."));
        assert!(text.contains("Claude state file: not found."));
        assert!(text.contains("Codex config.toml: not found."));
        assert!(text.contains("Antigravity mcp_config.json: not found."));
        assert!(!text.contains("fixture-mcp-command"));
        assert!(!text.contains("mcp-secret-sentinel"));
        assert!(window.try_find("mcp-entry-0").is_none());
        assert!(!root.path().join(FIXTURE_DIR).exists());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn preset_and_cross_source_copy_use_real_controls_and_keep_stored_credentials_hidden(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    let home = root.path().join(FIXTURE_DIR).join("home");
    let codex = home.join(".codex/config.toml");
    let claude = home.join(".claude.json");
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-mcp-manager", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window.try_find("mcp-empty").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("mcp-presets", cx);
        window.click("mcp-preset-fetch", cx);
        window.click("mcp-source-codex", cx);
        window.click("mcp-open-editor", cx);
    })
    .unwrap();
    wait_for_editor(cx, handle).await;
    cx.update_window(handle,|_,window,cx| {
        assert_eq!(window.find("mcp-name").value(),Some("fetch"));
        let initial:serde_json::Value=serde_json::from_str(window.find(patch_id(window)).value().unwrap()).unwrap();
        assert_eq!(initial["command"],"uvx");
        assert_eq!(initial["args"][0],"mcp-server-fetch");
        window.click(patch_id(window),cx);window.press("ctrl-a",cx);
        window.input(r#"{"command":"uvx","args":["mcp-server-fetch"],"env":{"TOKEN":"copy-secret-sentinel"}}"#,cx);
        window.click("mcp-editor-save",cx);
        assert!(!codex.exists());
        window.click("mcp-confirm-save",cx);
    }).unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window.try_find("mcp-entry-0").is_some()
    })
    .await;
    let original = fs::read(&codex).unwrap();
    cx.update_window(handle, |_, window, cx| {
        window.click("mcp-entry-0", cx);
        let button = window.find("mcp-copy").bounds();
        let viewport = window.find("mcp-manager").bounds();
        assert!(
            button.top() >= viewport.top() && button.bottom() <= viewport.bottom(),
            "copy button clipped: {button:?} in {viewport:?}"
        );
        window.click("mcp-copy", cx);
        window.click("mcp-source-user", cx);
        window.click("mcp-open-editor", cx);
    })
    .unwrap();
    wait_for_editor(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        assert!(!labels(window).contains("copy-secret-sentinel"));
        let nodes = gpui_kit::base::test_support::snapshots(window);
        assert!(
            !nodes
                .iter()
                .any(|node| node.label() == Some("MCP JSON patch"))
        );
        assert!(!nodes.iter().any(|node| {
            node.value()
                .is_some_and(|value| value.contains("copy-secret-sentinel"))
        }));
        window.click("mcp-name", cx);
        window.press("ctrl-a", cx);
        window.input("fetch-copy", cx);
        window.click("mcp-editor-save", cx);
        assert!(!claude.exists());
        window.click("mcp-confirm-cancel", cx);
        assert!(!claude.exists());
        assert_eq!(window.find("mcp-name").value(), Some("fetch-copy"));
        window.click("mcp-editor-save", cx);
        window.click("mcp-confirm-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window.try_find("mcp-entry-1").is_some()
    })
    .await;
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&claude).unwrap()).unwrap();
    assert_eq!(
        saved["mcpServers"]["fetch-copy"]["env"]["TOKEN"],
        "copy-secret-sentinel"
    );
    assert_eq!(saved["mcpServers"]["fetch-copy"]["command"], "uvx");
    assert_eq!(fs::read(&codex).unwrap(), original);
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn mcp_native_actions_write_confirm_and_keep_conflicting_drafts(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open(cx, &owner, root.path());
    let path = root.path().join(FIXTURE_DIR).join("home/.claude.json");
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-mcp-manager", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window.try_find("mcp-empty").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("mcp-add", cx);
        window.click("mcp-open-editor", cx);
    })
    .unwrap();
    wait_for_editor(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        window.click("mcp-name", cx);
        window.input("native-server", cx);
        window.click(patch_id(window), cx);
        window.press("ctrl-a", cx);
        window.input(
            r#"{"command":"fixture-tool","env":{"TOKEN":"mcp-write-secret"},"extension":7}"#,
            cx,
        );
        assert_eq!(window.find("mcp-name").value(), Some("native-server"));
        assert_eq!(
            window.find(patch_id(window)).value(),
            Some(r#"{"command":"fixture-tool","env":{"TOKEN":"mcp-write-secret"},"extension":7}"#)
        );
        window.click("mcp-editor-save", cx);
        assert!(!path.exists(), "preparing confirmation must not write");
        window.click("mcp-confirm-cancel", cx);
        assert!(
            window
                .find(patch_id(window))
                .value()
                .unwrap()
                .contains("mcp-write-secret")
        );
        window.click("mcp-editor-save", cx);
        window.click("mcp-confirm-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window.try_find("mcp-entry-0").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert!(!labels(window).contains("mcp-write-secret"));
        window.click("mcp-entry-0", cx);
        window.click("mcp-edit", cx);
    })
    .unwrap();
    wait_for_editor(cx, handle).await;
    let mut external: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    external["external"] = serde_json::json!(true);
    fs::write(&path, serde_json::to_vec(&external).unwrap()).unwrap();
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(
            window.find(patch_id(window)).value(),
            Some("{}"),
            "existing secrets must not populate patch controls"
        );
        window.click(patch_id(window), cx);
        window.press("ctrl-a", cx);
        window.input(r#"{"disabled":true}"#, cx);
        window.click("mcp-editor-save", cx);
        window.click("mcp-confirm-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        labels(window).contains("The source changed.")
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(
            window.find(patch_id(window)).value(),
            Some(r#"{"disabled":true}"#)
        );
        window.click("mcp-editor-close", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| window.click("mcp-edit", cx))
        .unwrap();
    wait_for_editor(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        window.click(patch_id(window), cx);
        window.press("ctrl-a", cx);
        window.input(r#"{"disabled":true}"#, cx);
        window.click("mcp-editor-save", cx);
        window.click("mcp-confirm-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window.try_find("mcp-entry-0").is_some()
    })
    .await;
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        saved["mcpServers"]["native-server"]["env"]["TOKEN"],
        "mcp-write-secret"
    );
    assert_eq!(saved["mcpServers"]["native-server"]["extension"], 7);
    assert_eq!(saved["mcpServers"]["native-server"]["disabled"], true);
    assert_eq!(saved["external"], true);
    cx.update_window(handle, |_, window, cx| {
        window.click("mcp-entry-0", cx);
        window.click("mcp-delete", cx);
    })
    .unwrap();
    wait_for_editor(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        window.click("mcp-editor-save", cx);
        window.click("mcp-confirm-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window.try_find("mcp-empty").is_some()
    })
    .await;
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert!(saved["mcpServers"].as_object().unwrap().is_empty());
    assert_eq!(saved["external"], true);
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    owner.shutdown();
}
