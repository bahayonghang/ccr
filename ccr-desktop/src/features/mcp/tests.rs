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
