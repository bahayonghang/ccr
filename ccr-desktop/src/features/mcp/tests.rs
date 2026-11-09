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

/// 掩码合同：URL 值、参数值、env/header 的键名和值都不得出现在任何快照的
/// label 或 value 中。
const SECRETS: [&str; 7] = [
    "mcp-secret-sentinel",
    "TOKEN",
    "Authorization",
    "bearer",
    "apiKey",
    "https://",
    "example.test",
];

fn assert_masked(window: &gpui_kit::Window, state: &str) {
    let exposed = gpui_kit::base::test_support::snapshots(window)
        .into_iter()
        .flat_map(|node| {
            [
                node.label().map(str::to_string),
                node.value().map(str::to_string),
            ]
        })
        .flatten()
        .collect::<Vec<_>>()
        .join("\n");
    for secret in SECRETS {
        assert!(!exposed.contains(secret), "{state} exposes {secret}");
    }
}

fn assert_absent(window: &gpui_kit::Window, state: &str, ids: &[&'static str]) {
    for id in ids.iter().copied() {
        assert!(window.try_find(id).is_none(), "{state} still shows {id}");
    }
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
/// 列表在编辑器打开时仍然可见；保存后要等编辑器关闭且重新读取出 `id`，
/// 才说明写入已经完成并刷新了列表。
async fn wait_for_reload(
    cx: &mut TestAppContext,
    handle: gpui_kit::AnyWindowHandle,
    id: &'static str,
) {
    cx.wait_for(handle, Duration::from_secs(5), move |window, _| {
        window.try_find("mcp-editor").is_none() && window.try_find(id).is_some()
    })
    .await;
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

/// 特征测试：pane 优先级为 batch > editor > source picker > presets > 选中详情，
/// 且每种状态下的快照都不含凭据。
#[gpui_kit::test]
async fn pane_priority_runs_batch_editor_sources_presets_then_detail_and_stays_masked(
    cx: &mut TestAppContext,
) {
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
        assert_masked(window, "list");
        assert!(window.try_find("mcp-detail").is_none(), "no auto selection");
        window.click("mcp-entry-1", cx);
        window.render_frame(cx);
        assert!(window.try_find("mcp-detail").is_some());
        assert_absent(window, "detail", &["mcp-preset-fetch", "mcp-source-user"]);
        assert_masked(window, "detail");
        window.click("mcp-presets", cx);
        window.render_frame(cx);
        assert!(window.try_find("mcp-preset-fetch").is_some());
        assert_absent(window, "presets", &["mcp-source-user", "mcp-detail"]);
        assert_masked(window, "presets");
        window.click("mcp-preset-fetch", cx);
        window.render_frame(cx);
        assert!(window.try_find("mcp-source-user").is_some());
        assert_absent(window, "source picker", &["mcp-preset-fetch", "mcp-detail"]);
        assert_masked(window, "source picker");
        window.click("mcp-open-editor", cx);
    })
    .unwrap();
    wait_for_editor(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        assert!(window.try_find("mcp-editor").is_some());
        // 宽窗分栏：编辑器占据 pane，列表仍在左侧。
        assert!(window.try_find("mcp-entry-0").is_some());
        let hidden = [
            "mcp-source-user",
            "mcp-preset-fetch",
            "mcp-detail",
            "mcp-batch",
        ];
        assert_absent(window, "editor", &hidden);
        // 编辑器打开时，预设与批量入口不可用，pane 不被替换。
        window.click("mcp-presets", cx);
        window.click("mcp-batch-open", cx);
        window.render_frame(cx);
        assert_absent(window, "editor", &hidden);
        assert_masked(window, "editor");
        window.click("mcp-editor-close", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("mcp-editor").is_none());
        assert!(
            window.try_find("mcp-detail").is_some(),
            "selection survives"
        );
        window.click("mcp-batch-open", cx);
        window.render_frame(cx);
        assert!(window.try_find("mcp-batch").is_some());
        assert!(window.try_find("mcp-entry-0").is_some());
        let hidden = [
            "mcp-editor",
            "mcp-source-user",
            "mcp-preset-fetch",
            "mcp-detail",
        ];
        assert_absent(window, "batch", &hidden);
        // 批量打开时，添加与预设入口不可用。
        window.click("mcp-presets", cx);
        window.click("mcp-add", cx);
        window.render_frame(cx);
        assert!(window.try_find("mcp-batch").is_some());
        assert_absent(window, "batch", &hidden);
        assert_masked(window, "batch");
        // 退出批量的按钮在标题行，1280×800 的分栏 pane 里不落到折叠线以下。
        inside(window, "mcp-batch-close", "mcp-pane");
        window.click("mcp-batch-close", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window.try_find("mcp-batch").is_none() && window.try_find("mcp-entry-4").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window.try_find("mcp-detail").is_none(),
            "reload clears selection"
        );
        assert_masked(window, "after batch");
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

fn open_at(
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
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::new(
                    gpui_kit::point(px(0.), px(0.)),
                    size(px(width), px(height)),
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

fn same_row(window: &gpui_kit::Window, left: &'static str, right: &'static str) {
    let (a, b) = (window.find(left).bounds(), window.find(right).bounds());
    assert!(
        a.top() < b.bottom() && b.top() < a.bottom(),
        "{left} {a:?} and {right} {b:?} are not on one row"
    );
}

fn inside(window: &gpui_kit::Window, id: &'static str, container: &'static str) {
    let (node, frame) = (window.find(id).bounds(), window.find(container).bounds());
    assert!(
        node.left() >= frame.left()
            && node.right() <= frame.right()
            && node.top() >= frame.top()
            && node.bottom() <= frame.bottom(),
        "{id} {node:?} is clipped by {container} {frame:?}"
    );
}

/// 标题与 lens 同一行；工具栏、平台计数与异常标签都来自 catalog。
#[gpui_kit::test]
async fn heading_lens_toolbar_counts_and_tags_follow_the_catalog(cx: &mut TestAppContext) {
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
        // 嵌入式 lens：外壳不再渲染上方的胶囊行。
        assert!(window.try_find("lens-navigation").is_none());
        assert_eq!(
            window.find("mcp-heading").role(),
            Some(gpui_kit::Role::Heading)
        );
        same_row(window, "mcp-heading", "tool-mcp");
        same_row(window, "mcp-heading", "tool-converter");
        same_row(window, "mcp-heading", "mcp-add");
        assert_eq!(window.find("tool-mcp").checked(), Some(true));
        // 工具栏一行：搜索在左，批量入口在右；没有全宽按钮。
        same_row(window, "mcp-filter", "mcp-batch-open");
        let toolbar = window.find("mcp-toolbar").bounds();
        let batch = window.find("mcp-batch-open").bounds();
        assert!(
            batch.size.width < toolbar.size.width / 2.,
            "batch button stretches"
        );
        for (id, label) in [
            ("mcp-platform-all", "All 5"),
            ("mcp-platform-claude", "Claude Code 3"),
            ("mcp-platform-codex", "Codex 1"),
            ("mcp-platform-antigravity", "Antigravity 1"),
        ] {
            assert_eq!(window.find(id).label(), Some(label));
        }
        assert_eq!(window.find("mcp-platform-all").checked(), Some(true));
        // 两行行高 56；标签只在异常时出现。
        assert_eq!(
            window.find("mcp-entry-1").bounds().size.height,
            px(theme::MCP_ROW_HEIGHT)
        );
        for id in [
            "mcp-entry-2-disabled",
            "mcp-entry-2-read-only",
            "mcp-entry-3-disabled",
        ] {
            assert!(window.try_find(id).is_some(), "{id} missing");
        }
        assert_absent(
            window,
            "healthy rows",
            &[
                "mcp-entry-0-disabled",
                "mcp-entry-0-read-only",
                "mcp-entry-1-disabled",
                "mcp-entry-1-read-only",
                "mcp-entry-3-read-only",
                "mcp-entry-4-disabled",
                "mcp-entry-4-read-only",
            ],
        );
        let row = window.find("mcp-entry-0").label().unwrap().to_string();
        assert!(
            row.contains("project-server") && row.contains("URL set (hidden)"),
            "{row}"
        );
        assert!(
            window
                .find("mcp-entry-1")
                .label()
                .unwrap()
                .contains("fixture-mcp-command")
        );
        // 来源健康条：chip 只写短读数，原句在无障碍标签里。
        assert_eq!(
            window.find("mcp-source-0").label(),
            Some("Claude state file: 2 entries.")
        );
        let text = labels(window);
        for invented in ["Running", "Connected", "running", "connected"] {
            assert!(
                !text.contains(invented),
                "invented process state {invented}"
            );
        }
        assert_masked(window, "list");
        // 配置转换：同一胶囊，标题有 heading 语义。
        window.click("tool-converter", cx);
        window.render_frame(cx);
        assert!(window.try_find("lens-navigation").is_none());
        assert_eq!(
            window.find("converter-heading").role(),
            Some(gpui_kit::Role::Heading)
        );
        same_row(window, "converter-heading", "tool-converter");
        assert_eq!(window.find("tool-converter").checked(), Some(true));
        // Claude 工作台复用的 MCP 视图不带嵌入胶囊，平台工具栏不变。
        window.click("nav-claude-code", cx);
        window.click("tool-platform-mcp", cx);
        window.render_frame(cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window.try_find("mcp-entry-2").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.find("platform-context-navigation").visible());
        assert_eq!(window.find("tool-platform-mcp").checked(), Some(true));
        assert_absent(
            window,
            "platform MCP",
            &[
                "tool-mcp",
                "tool-converter",
                "mcp-platform-all",
                "mcp-entry-3",
            ],
        );
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

/// 搜索同时匹配名称和命令，平台胶囊与搜索取交集，计数在搜索之前计算。
#[gpui_kit::test]
async fn search_matches_commands_and_the_platform_capsule_intersects_it(cx: &mut TestAppContext) {
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
        window.click("mcp-filter", cx);
        window.input("FIXTURE-MCP", cx);
        window.render_frame(cx);
        for (id, name) in [
            ("mcp-entry-0", "state-server"),
            ("mcp-entry-1", "settings-server"),
            ("mcp-entry-2", "codex-server"),
        ] {
            assert!(window.find(id).label().unwrap().contains(name), "{id}");
        }
        assert!(window.try_find("mcp-entry-3").is_none());
        assert_eq!(window.find("mcp-platform-all").label(), Some("All 5"));
        window.click("mcp-platform-codex", cx);
        window.render_frame(cx);
        assert_eq!(window.find("mcp-platform-codex").checked(), Some(true));
        assert_eq!(window.find("mcp-platform-all").checked(), Some(false));
        assert!(
            window
                .find("mcp-entry-0")
                .label()
                .unwrap()
                .contains("codex-server")
        );
        assert!(window.try_find("mcp-entry-1").is_none());
        window.click("mcp-filter", cx);
        window.press("ctrl-a", cx);
        window.input("agy", cx);
        window.render_frame(cx);
        assert!(window.try_find("mcp-entry-0").is_none());
        assert!(window.try_find("mcp-filter-empty").is_some());
        window.click("mcp-filter-clear", cx);
        window.render_frame(cx);
        assert_eq!(window.find("mcp-filter").value(), Some(""));
        assert_eq!(window.find("mcp-platform-all").checked(), Some(true));
        assert!(window.try_find("mcp-entry-4").is_some());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

/// 宽窗列表与 pane 3:2 并排；960×640 进入 drill-in，返回后回到列表。
#[gpui_kit::test]
async fn wide_windows_split_three_to_two_and_narrow_windows_drill_in(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    write_entries(root.path());
    let owner = RuntimeOwner::new().unwrap();
    for (width, height) in [(1280., 800.), (2000., 1080.)] {
        let handle = open_at(cx, &owner, root.path(), width, height);
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
            // 未选中时空态 pane 与详情 pane 用同一列宽规则。
            let list = window.find("mcp-list").bounds();
            let empty = window.find("mcp-pane-empty").bounds();
            let body = window.find("mcp-body").bounds();
            assert!(
                list.bottom() + px(48.) < body.bottom(),
                "few rows must hug content"
            );
            assert!(
                empty.bottom() < body.bottom(),
                "empty pane must hug content"
            );
            let ratio = f32::from(list.size.width) / f32::from(empty.size.width);
            assert!((ratio - 1.5).abs() < 0.01, "list/empty pane ratio {ratio}");
            window.click("mcp-entry-1", cx);
            window.render_frame(cx);
            let list = window.find("mcp-list").bounds();
            let pane = window.find("mcp-pane").bounds();
            println!(
                "{width}x{height}: mcp-list {} wide, mcp-pane {} wide",
                f32::from(list.size.width),
                f32::from(pane.size.width)
            );
            assert_eq!(list.top(), pane.top());
            assert_eq!(pane.left() - list.right(), px(theme::SECTION_GAP));
            assert!(list.size.width >= px(theme::LIST_MIN_WIDTH));
            assert!(pane.size.width >= px(theme::DETAIL_MIN_WIDTH));
            let ratio = f32::from(list.size.width) / f32::from(pane.size.width);
            assert!((ratio - 1.5).abs() < 0.01, "list/pane ratio {ratio}");
            for id in ["mcp-copy", "mcp-edit", "mcp-delete"] {
                inside(window, id, "mcp-pane");
                inside(window, id, "mcp-manager");
            }
            assert!(window.try_find("mcp-pane-back").is_none());
            window.remove_window();
        })
        .unwrap();
    }
    let handle = open_at(cx, &owner, root.path(), 960., 640.);
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
        assert!(window.try_find("mcp-pane").is_none());
        assert!(window.try_find("mcp-pane-empty").is_none());
        // 健康条在条内横滚，不撑出页面。
        inside(window, "mcp-sources", "mcp-manager");
        inside(window, "mcp-toolbar", "mcp-manager");
        window.click("mcp-entry-1", cx);
        window.render_frame(cx);
        assert!(window.try_find("mcp-list").is_none());
        assert!(window.try_find("mcp-detail").is_some());
        for id in ["mcp-pane-back", "mcp-copy", "mcp-edit", "mcp-delete"] {
            inside(window, id, "mcp-pane");
        }
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("mcp-pane-back").focused(), Some(true));
        window.click("mcp-pane-back", cx);
        window.render_frame(cx);
        assert!(window.try_find("mcp-pane").is_none());
        assert!(window.try_find("mcp-detail").is_none());
        assert!(window.try_find("mcp-entry-1").is_some());
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("mcp-entry-1").focused(), Some(true));
        // 离页后，延迟的 drill-in 焦点回调不抢走侧栏焦点。
        window.click("mcp-entry-2", cx);
        window.click("nav-checkin", cx);
        assert_eq!(window.find("nav-checkin").focused(), Some(true));
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("nav-checkin").focused(), Some(true));
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

/// 1280×800 分栏 pane 较矮：批量的退出、检查与确认控件都在 pane 视口内，
/// 确认区挂载后焦点落在可见的“继续编辑”上，Esc 回到可见的“检查批量更改”。
#[gpui_kit::test]
async fn batch_decisions_stay_inside_the_split_pane(cx: &mut TestAppContext) {
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
        window.click("mcp-batch-open", cx);
        window.render_frame(cx);
        for id in ["mcp-batch-close", "mcp-batch-review", "mcp-batch-mode-sync"] {
            inside(window, id, "mcp-pane");
        }
        window.click("mcp-batch-mode-sync", cx);
        window.render_frame(cx);
        inside(window, "mcp-batch-review", "mcp-pane");
        window.click("mcp-batch-review", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("mcp-batch-confirm").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("mcp-batch-back").focused(), Some(true));
        for id in ["mcp-batch-back", "mcp-batch-confirm"] {
            inside(window, id, "mcp-pane");
        }
        // 确认区挂载时只有确认区自己的返回。
        assert!(window.try_find("mcp-batch-close").is_none());
        assert_masked(window, "batch confirmation");
        window.press("escape", cx);
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("mcp-batch-review").focused(), Some(true));
        inside(window, "mcp-batch-review", "mcp-pane");
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

/// 搜索框与项目根输入框有 placeholder，并随界面语言切换。
#[gpui_kit::test]
fn inputs_carry_placeholders_that_follow_the_language(cx: &mut TestAppContext) {
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let services = services(&owner, root.path());
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| McpView::new(services, window, cx))
        })
        .unwrap()
    });
    let placeholders = |cx: &mut TestAppContext| {
        cx.update(|cx| {
            let view = view.read(cx);
            (
                view.filter
                    .read(cx)
                    .presentation()
                    .placeholder()
                    .to_string(),
                view.project
                    .read(cx)
                    .presentation()
                    .placeholder()
                    .to_string(),
            )
        })
    };
    assert_eq!(
        placeholders(cx),
        (
            "Search name or command".to_string(),
            "Absolute path (optional)".to_string()
        )
    );
    cx.update(|cx| cx.set_global(crate::locales::Language::Chinese));
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |_, cx| cx.notify());
        window.render_frame(cx);
    })
    .unwrap();
    assert_eq!(
        placeholders(cx),
        ("搜索名称或命令".to_string(), "绝对路径（可选）".to_string())
    );
    cx.update(|cx| cx.set_global(crate::locales::Language::English));
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("mcp-add", cx);
        window.render_frame(cx);
        // 项目根输入框有可见标签，标签在输入框正上方。
        let label = window.find("mcp-project-root-label");
        assert_eq!(label.label(), Some("Project directory"));
        let input = window.find("mcp-project-root").bounds();
        assert!(label.bounds().bottom() <= input.top());
        assert_eq!(
            window.find("mcp-destination-label").label(),
            Some("Destination")
        );
        assert_eq!(window.find("mcp-source-user").checked(), Some(true));
        window.click("mcp-pane-cancel", cx);
        window.render_frame(cx);
        assert!(window.try_find("mcp-source-user").is_none());
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
        window.render_frame(cx);
        assert_eq!(window.find("mcp-source-codex").checked(), Some(true));
        assert_eq!(window.find("mcp-source-user").checked(), Some(false));
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
    wait_for_reload(cx, handle, "mcp-entry-0").await;
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
    wait_for_reload(cx, handle, "mcp-entry-1").await;
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
    wait_for_reload(cx, handle, "mcp-entry-0").await;
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
    wait_for_reload(cx, handle, "mcp-entry-0").await;
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

/// 长列表在列内滚动，阅读面不超过主体底部。
#[gpui_kit::test]
async fn many_mcp_entries_scroll_inside_the_content_sized_panel(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    write_entries(root.path());
    let entries: serde_json::Map<String, serde_json::Value> = (0..100)
        .map(|i| {
            (
                format!("server-{i:03}"),
                serde_json::json!({"command": "fixture-command"}),
            )
        })
        .collect();
    fs::write(
        root.path().join(FIXTURE_DIR).join("home/.claude.json"),
        serde_json::json!({"mcpServers": entries}).to_string(),
    )
    .unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let handle = open_at(cx, &owner, root.path(), 2000., 1080.);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-mcp-manager", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window.try_find("mcp-entry-99").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let list = window.find("mcp-list").bounds();
        let body = window.find("mcp-body").bounds();
        assert!(list.bottom() <= body.bottom());
        assert!(list.size.height < px(100. * theme::MCP_ROW_HEIGHT));
        assert!(window.find("mcp-pane-empty").bounds().bottom() < body.bottom());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}
