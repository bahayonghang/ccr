//! Core rendered shell interactions. Native OS IME/UIA are separate evidence.
use ccr_app::runtime::RuntimeOwner;
use ccr_gpui::{
    locales::Language,
    runtime::{ConfigSource, Services, profiles::context},
    shell::Shell,
};
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{AppContext, TestAppContext, WindowOptions};
use std::time::{Duration, Instant};

fn wait_for_preferences(cx: &mut TestAppContext, handle: gpui_kit::AnyWindowHandle) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-settings", cx);
    })
    .unwrap_or_else(|error| panic!("preferences: {error}"));
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        cx.run_until_parked();
        let ready = cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                window.find("theme-dark").disabled() != Some(true)
            })
            .unwrap_or_else(|error| panic!("preferences: {error}"));
        if ready {
            break;
        }
        assert!(Instant::now() < deadline, "preferences startup timed out");
        cx.executor().advance_clock(Duration::from_millis(10));
        std::thread::yield_now();
    }
}

#[gpui_kit::test]
fn actual_theme_controls_keep_base_component_palette_and_language_in_sync(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
    let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                Shell::new(
                    Services::new(
                        owner.handle(),
                        ConfigSource::Fixture {
                            profiles: root.path().join("profiles.toml"),
                            registry: root.path().join("config.toml"),
                            delay: Duration::ZERO,
                        },
                    ),
                    window,
                    cx,
                )
            })
        })
        .unwrap_or_else(|e| panic!("window: {e}"))
    });
    wait_for_preferences(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-claude-code", cx);
        window.click("tool-profiles", cx);
        window.click("configuration-filter", cx);
        window.input("retained query", cx);
        window.click("nav-settings", cx);
        window.press("f6", cx);
        assert_eq!(window.find("settings-nav-appearance").focused(), Some(true));
        window.press("tab", cx);
        window.press("enter", cx);
        assert_eq!(window.find("settings-nav-language").checked(), Some(true));
        window.press("tab", cx);
        assert_eq!(window.find("language-en").focused(), Some(true));
        for (nav, first_option) in [
            ("settings-nav-language", "language-en"),
            ("settings-nav-workbench", "close-confirm"),
            ("settings-nav-environment", "env-notify"),
            ("settings-nav-appearance", "theme-light"),
        ] {
            window.click(nav, cx);
            assert_eq!(window.find(nav).checked(), Some(true));
            window.press("tab", cx);
            assert_eq!(window.find(first_option).focused(), Some(true), "{nav}");
            let option = window.find(first_option).bounds();
            assert!(
                option.top() >= window.find("settings-nav-environment").bounds().bottom()
                    && option.bottom() <= window.find("shell-status").bounds().top(),
                "{nav} leaves its focused option outside the settings content: {option:?}"
            );
        }
        window.click("theme-dark", cx);
        assert_eq!(Theme::global(cx).mode, ThemeMode::Dark);
        assert_eq!(
            window.find("nav-settings-summary").label(),
            Some("Dark · English")
        );
        assert_eq!(
            Theme::global(cx).colors.background,
            gpui_kit::Hsla::from(gpui_kit::rgb(0x1d292c))
        );
        assert_eq!(
            gpui_kit::base::Theme::global(cx).tokens.colors,
            Theme::global(cx).semantic_tokens().colors
        );
        assert_eq!(
            Theme::global(cx).primary,
            gpui_kit::Hsla::from(gpui_kit::rgb(0x85d5b1))
        );
        window.click("theme-light", cx);
        assert_eq!(Theme::global(cx).mode, ThemeMode::Light);
        assert_eq!(
            Theme::global(cx).colors.background,
            gpui_kit::Hsla::from(gpui_kit::rgb(0xffffff))
        );
        assert_eq!(
            gpui_kit::base::Theme::global(cx).tokens.colors,
            Theme::global(cx).semantic_tokens().colors
        );
        window.click("language-zh", cx);
        assert!(matches!(cx.global::<Language>(), Language::Chinese));
        assert_eq!(
            window.find("nav-settings-summary").label(),
            Some("浅色 · 中文")
        );
        assert_eq!(window.find("nav-dashboard").label(), Some("运行概览"));
        assert_eq!(window.find("theme-system").label(), Some("跟随系统"));
        assert_eq!(window.find("language-zh").checked(), Some(true));
        assert!(
            window
                .find("settings-summary")
                .label()
                .is_some_and(|label| label.ends_with("浅色  ·  中文"))
        );
        window.click("language-en", cx);
        assert_eq!(window.find("nav-dashboard").label(), Some("Overview"));
        window.click("theme-system", cx);
        assert_eq!(
            gpui_kit::base::Theme::global(cx).tokens.colors,
            Theme::global(cx).semantic_tokens().colors
        );
        window.click("nav-claude-code", cx);
        window.click("tool-profiles", cx);
        assert_eq!(
            window.find("configuration-filter").value(),
            Some("retained query")
        );
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("interaction: {e}"));
    owner.shutdown();
}

#[gpui_kit::test]
fn f6_tab_navigation_does_not_intercept_search_editing(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
    let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                Shell::new(
                    Services::new(
                        owner.handle(),
                        ConfigSource::Fixture {
                            profiles: root.path().join("profiles.toml"),
                            registry: root.path().join("config.toml"),
                            delay: Duration::ZERO,
                        },
                    ),
                    window,
                    cx,
                )
            })
        })
        .unwrap_or_else(|e| panic!("window: {e}"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-claude-code", cx);
        window.click("tool-profiles", cx);
        window.click("configuration-filter", cx);
        window.input("原生读取", cx);
        window.press("left", cx);
        window.press("backspace", cx);
        assert_eq!(window.find("configuration-filter").value(), Some("原生取"));
        window.press("f6", cx); // main -> status
        window.press("f6", cx); // status -> navigation
        assert_eq!(window.find("nav-dashboard").focused(), Some(true));
        window.press("down", cx);
        assert_eq!(window.find("nav-agent-sessions").focused(), Some(true));
        window.press("up", cx);
        assert_eq!(window.find("nav-dashboard").focused(), Some(true));
        window.press("f6", cx); // navigation -> main
        assert_eq!(window.find("configuration-filter").focused(), Some(true));
        window.press("shift-f6", cx);
        assert_eq!(window.find("nav-dashboard").focused(), Some(true));
        window.press("tab", cx);
        assert_eq!(window.find("nav-agent-sessions").focused(), Some(true));
        window.press("shift-tab", cx);
        assert_eq!(window.find("nav-dashboard").focused(), Some(true));
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("interaction: {e}"));
    owner.shutdown();
}

#[gpui_kit::test]
fn navigation_clears_converter_plaintext_and_acknowledgement(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                Shell::new(
                    Services::new(
                        owner.handle(),
                        ConfigSource::Fixture {
                            profiles: root.path().join("profiles.toml"),
                            registry: root.path().join("registry.toml"),
                            delay: Duration::ZERO,
                        },
                    ),
                    window,
                    cx,
                )
            })
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle,|_,window,cx|{
        window.render_frame(cx);window.click("nav-mcp-manager",cx);window.click("tool-converter",cx);window.click("converter-acknowledge",cx);
        let input=gpui_kit::base::test_support::snapshots(window).into_iter().find(|node|node.label()==Some("Source configuration")).unwrap_or_else(||panic!("source input"));
        window.click(input.path().last().cloned().unwrap_or_else(||panic!("input identity")),cx);
        window.input("{\"mcpServers\":{\"fixture\":{\"command\":\"fixture-command\",\"env\":{\"KEY\":\"converter-secret-sentinel\"}}}}",cx);
        window.click("converter-run",cx);window.click("nav-claude-code",cx);
        window.click("nav-mcp-manager",cx);window.click("tool-converter",cx);assert!(window.find("converter-acknowledge").visible());
        window.click("converter-acknowledge",cx);
        let nodes=gpui_kit::base::test_support::snapshots(window);
        assert_eq!(nodes.iter().find(|node|node.label()==Some("Source configuration")).and_then(|node|node.value()),Some(""));
        assert_eq!(nodes.iter().find(|node|node.label()==Some("Converted preview")).and_then(|node|node.value()),Some(""));
        window.remove_window();
    }).unwrap_or_else(|_|panic!("navigation"));
    owner.shutdown();
}

#[gpui_kit::test]
fn opaque_surface_quads_preserve_the_paper_corners(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
    let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(gpui_kit::WindowBounds::Windowed(
                    gpui_kit::Bounds::centered(
                        None,
                        gpui_kit::size(gpui_kit::px(1280.), gpui_kit::px(800.)),
                        cx,
                    ),
                )),
                ..Default::default()
            },
            cx,
            |window, cx| {
                cx.new(|cx| {
                    Shell::new(
                        Services::new(
                            owner.handle(),
                            ConfigSource::Fixture {
                                profiles: root.path().join("profiles.toml"),
                                registry: root.path().join("registry.toml"),
                                delay: Duration::ZERO,
                            },
                        ),
                        window,
                        cx,
                    )
                })
            },
        )
        .unwrap_or_else(|e| panic!("window: {e}"))
    });
    wait_for_preferences(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        for mode in ["theme-light", "theme-dark"] {
            window.click("nav-settings", cx);
            window.click(mode, cx);
            for page in [
                "nav-dashboard",
                "nav-agent-sessions",
                "nav-mcp-manager",
                "nav-claude-code",
                "nav-codex",
                "nav-grok",
                "nav-antigravity",
                "nav-opencode",
                "nav-commands",
                "nav-checkin",
                "nav-sync",
                "nav-usage",
                "nav-settings",
            ] {
                window.click(page, cx);
                if page == "nav-claude-code" {
                    window.click("tool-profiles", cx);
                }
                let quads = window.painted_quads();
                let scale = window.scale_factor();
                let paper = quads
                    .iter()
                    .find(|quad| {
                        quad.background
                            == gpui_kit::Background::from(gpui_kit::rgb(
                                ccr_gpui::theme::palette(Theme::global(cx).mode).sheet,
                            ))
                            && quad.bounds.size.width.as_f32() > 500. * scale
                            && quad.bounds.size.height.as_f32() > 100. * scale
                    })
                    .unwrap_or_else(|| panic!("painted paper on {page}"));
                let sidebar = quads
                    .iter()
                    .find(|quad| {
                        quad.background == gpui_kit::Background::from(Theme::global(cx).sidebar)
                            && quad.bounds.size.height.as_f32() > 100. * scale
                    })
                    .unwrap_or_else(|| panic!("painted sidebar on {page}"));
                assert!(
                    sidebar.corner_radii.top_left.as_f32() > 0.,
                    "sidebar covers paper corner on {page}"
                );
                assert_eq!(
                    paper.corner_radii.bottom_left.as_f32(),
                    ccr_gpui::theme::SHEET_RADIUS * scale
                );
                assert_eq!(
                    paper.corner_radii.bottom_right.as_f32(),
                    ccr_gpui::theme::SHEET_RADIUS * scale
                );
                assert!(
                    !quads.iter().any(|quad| {
                        quad.background == gpui_kit::Background::from(Theme::global(cx).status_bar)
                            && quad.bounds.size.width.as_f32() > 500. * scale
                            && quad.bounds.size.height.as_f32() < 40. * scale
                            && quad.bounds.origin.y.as_f32()
                                > paper.bounds.bottom_right().y.as_f32() - 40. * scale
                    }),
                    "status background covers paper bottom corners on {page}"
                );
                if page == "nav-claude-code" {
                    let reading = quads
                        .iter()
                        .find(|quad| {
                            quad.background
                                == gpui_kit::Background::from(Theme::global(cx).background)
                                && quad.bounds.size.width.as_f32() > 500. * scale
                                && quad.bounds.size.height.as_f32() > 100. * scale
                                && quad.bounds.origin.x > paper.bounds.origin.x
                        })
                        .unwrap_or_else(|| panic!("painted configurations"));
                    assert!(reading.corner_radii.top_right.as_f32() > 0.);
                }
            }
        }
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("painted corners: {e}"));
    owner.shutdown();
}

#[gpui_kit::test]
fn overview_states_missing_usage_without_sample_numbers(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
    let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                Shell::new(
                    Services::new(
                        owner.handle(),
                        ConfigSource::Fixture {
                            profiles: root.path().join("profiles.toml"),
                            registry: root.path().join("config.toml"),
                            delay: Duration::ZERO,
                        },
                    ),
                    window,
                    cx,
                )
            })
        })
        .unwrap_or_else(|e| panic!("window: {e}"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let status = window
            .find("overview-status")
            .label()
            .unwrap_or_default()
            .to_string();
        assert!(
            status.contains("No local usage source"),
            "missing source must be distinct from a zero-usage report: {status}"
        );
        assert_eq!(window.find("nav-dashboard").label(), Some("Overview"));
        let text = gpui_kit::base::test_support::snapshots(window)
            .into_iter()
            .filter_map(|node| node.label().map(str::to_string))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!text.contains("Synthetic owned-process"));
        assert!(!text.contains("264.98"));
        assert!(!text.contains("$0.00"));
        window.click("nav-commands", cx);
        let warning = window
            .find("jobs-fixture-warning")
            .label()
            .unwrap_or_default()
            .to_string();
        assert!(
            warning.contains("Synthetic owned-process"),
            "fixture command page keeps its closed synthetic warning"
        );
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("overview: {e}"));
    owner.shutdown();
}

#[gpui_kit::test]
fn workbench_navigation_exposes_thirteen_destinations(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
    let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(gpui_kit::WindowBounds::Windowed(
                    gpui_kit::Bounds::centered(
                        None,
                        gpui_kit::size(gpui_kit::px(1280.), gpui_kit::px(800.)),
                        cx,
                    ),
                )),
                ..Default::default()
            },
            cx,
            |window, cx| {
                cx.new(|cx| {
                    Shell::new(
                        Services::new(
                            owner.handle(),
                            ConfigSource::Fixture {
                                profiles: root.path().join("profiles.toml"),
                                registry: root.path().join("config.toml"),
                                delay: Duration::ZERO,
                            },
                        ),
                        window,
                        cx,
                    )
                })
            },
        )
        .unwrap_or_else(|e| panic!("window: {e}"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let settings = window.find("nav-settings").bounds();
        let status = window.find("shell-status").bounds();
        assert!(
            settings.size.height >= gpui_kit::px(24.),
            "settings dock is clipped: {settings:?}"
        );
        assert!(
            settings.bottom() <= status.top(),
            "settings dock overlaps the status line: {settings:?} {status:?}"
        );
        for id in [
            "nav-overview",
            "nav-configurations",
            "nav-sessions",
            "nav-resources",
            "nav-platforms",
            "nav-favorites",
            "nav-recent",
            "nav-converter",
            "nav-insights",
            "nav-jobs",
            "nav-install",
        ] {
            assert!(
                window.try_find(id).is_none(),
                "{id} is still a sidebar entry"
            );
        }
        for (nav, page, label) in [
            ("nav-dashboard", "page-dashboard", "Overview"),
            (
                "nav-agent-sessions",
                "page-agent-sessions",
                "Agent sessions",
            ),
            ("nav-mcp-manager", "page-mcp-manager", "MCP Manager"),
            ("nav-claude-code", "page-claude-code", "Claude Code"),
            ("nav-codex", "page-codex", "Codex"),
            ("nav-grok", "page-grok", "Grok"),
            ("nav-antigravity", "page-antigravity", "Antigravity"),
            ("nav-opencode", "page-opencode", "OpenCode"),
            ("nav-commands", "page-commands", "Commands"),
            ("nav-checkin", "page-checkin", "Check-ins"),
            ("nav-sync", "page-sync", "Sync"),
            ("nav-usage", "page-usage", "Usage"),
            ("nav-settings", "page-settings", "Settings"),
        ] {
            window.click(nav, cx);
            assert_eq!(window.find(nav).label(), Some(label));
            assert_eq!(window.find(nav).checked(), Some(true), "{nav} selected");
            if nav != "nav-dashboard" {
                assert_eq!(
                    window.find("nav-dashboard").checked(),
                    Some(false),
                    "{nav} left Overview selected"
                );
            }
            assert!(window.find(page).visible(), "{page} missing");
            let text = gpui_kit::base::test_support::snapshots(window)
                .into_iter()
                .filter_map(|node| node.label().map(str::to_string))
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                !text.contains("Home insights"),
                "{nav} shows the insights bench"
            );
            assert!(
                !text.contains("Apply / refresh"),
                "{nav} shows refresh debug copy"
            );
        }
        window.click("nav-claude-code", cx);
        window.click("tool-profiles", cx);
        window.click("configuration-filter", cx);
        window.press("f6", cx);
        window.press("f6", cx);
        assert_eq!(window.find("nav-dashboard").focused(), Some(true));
        window.press("up", cx);
        assert_eq!(window.find("nav-settings").focused(), Some(true));
        assert_eq!(window.find("nav-claude-code").checked(), Some(true));
        window.press("down", cx);
        assert_eq!(window.find("nav-dashboard").focused(), Some(true));
        window.press("down", cx);
        assert_eq!(window.find("nav-agent-sessions").focused(), Some(true));
        assert_eq!(window.find("nav-claude-code").checked(), Some(true));
        assert_eq!(window.find("nav-agent-sessions").checked(), Some(false));
        window.press("enter", cx);
        assert!(window.find("page-agent-sessions").visible());
        assert_eq!(window.find("nav-agent-sessions").checked(), Some(true));
        window.press("tab", cx);
        assert_eq!(window.find("nav-mcp-manager").focused(), Some(true));
        assert_eq!(window.find("nav-agent-sessions").checked(), Some(true));
        window.press("enter", cx);
        assert!(window.find("page-mcp-manager").visible());
        window.click("nav-claude-code", cx);
        window.click("tool-profiles", cx);
        window.click("configuration-filter", cx);
        window.press("f6", cx);
        window.press("f6", cx);
        assert_eq!(window.find("nav-dashboard").focused(), Some(true));
        window.click("nav-checkin", cx);
        assert_eq!(window.find("nav-checkin").checked(), Some(true));
        assert_eq!(window.find("nav-dashboard").checked(), Some(false));
        assert_eq!(
            window.find("nav-checkin").focused(),
            Some(true),
            "Check-ins did not take the sidebar focus ring"
        );
        assert_ne!(
            window.find("nav-dashboard").focused(),
            Some(true),
            "Overview keeps the sidebar focus ring"
        );
        window.click("nav-settings", cx);
        assert_eq!(window.find("nav-settings").checked(), Some(true));
        assert_eq!(window.find("nav-settings").focused(), Some(true));
        assert_eq!(window.find("nav-dashboard").focused(), Some(false));
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("navigation: {e}"));
    owner.shutdown();
}

fn seed_catalog(source: &ConfigSource, platform: ccr_config::Platform, name: &str, current: bool) {
    let context = context(source, platform).unwrap_or_else(|_| panic!("context"));
    context
        .run(|| {
            let mut config = ccr_config::CcsConfig {
                default_config: name.into(),
                current_config: if current { name.into() } else { String::new() },
                settings: Default::default(),
                sections: Default::default(),
            };
            config.sections.insert(
                name.into(),
                ccr_config::ConfigSection {
                    description: Some("Fixture profile".into()),
                    base_url: Some("https://fixture.invalid".into()),
                    auth_token: Some(ccr_core::Secret::new("sk-native-profile-secret-sentinel")),
                    model: Some("fixture-model".into()),
                    enabled: Some(true),
                    ..Default::default()
                },
            );
            context.manager().save(&config)
        })
        .unwrap_or_else(|_| panic!("seed"));
}

fn page_labels(window: &gpui_kit::Window) -> String {
    gpui_kit::base::test_support::snapshots(window)
        .into_iter()
        .filter_map(|node| node.label().map(str::to_string))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Explicit QA preparation only. The supplied new directory must be beneath
/// this checkout's outputs/desktop; no user roots or existing fixture are read.
#[test]
#[ignore = "explicit isolated native workbench preparation"]
fn prepare_native_workbench_fixture() {
    use ccr_app::platform_settings::{Document, Format, SaveOutcome};
    use ccr_app::runtime::{EnvironmentId, Platform, RequestKey};
    use ccr_app::usage::observer::subscription::SubscriptionDto;
    use ccr_core::core::BackupPolicy;
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| panic!("repo"));
    let output = repo
        .join("outputs/desktop")
        .canonicalize()
        .unwrap_or_else(|e| panic!("output: {e}"));
    let root = std::path::PathBuf::from(
        std::env::var_os("CCR_NATIVE_FIXTURE_ROOT")
            .unwrap_or_else(|| panic!("set CCR_NATIVE_FIXTURE_ROOT")),
    );
    assert!(root.is_absolute());
    let parent = root
        .parent()
        .unwrap_or_else(|| panic!("fixture parent"))
        .canonicalize()
        .unwrap_or_else(|e| panic!("parent: {e}"));
    assert!(
        parent.starts_with(&output),
        "fixture must stay in outputs/desktop"
    );
    assert!(!root.exists(), "refuse existing fixture");
    std::fs::create_dir(&root).unwrap_or_else(|e| panic!("fixture: {e}"));
    std::fs::write(
        root.join("CCR_NATIVE_DEMO_ONLY"),
        "Synthetic native workbench fixture. No real credentials.\n",
    )
    .unwrap_or_else(|e| panic!("sentinel: {e}"));
    let source = ConfigSource::Fixture {
        profiles: root.join("profiles.toml"),
        registry: root.join("registry.toml"),
        delay: Duration::ZERO,
    };
    for (platform, name) in [
        (ccr_config::Platform::Claude, "demo-claude"),
        (ccr_config::Platform::Codex, "demo-codex"),
        (ccr_config::Platform::Grok, "demo-grok"),
    ] {
        seed_catalog(&source, platform, name, true);
        let context = context(&source, platform).unwrap_or_else(|_| panic!("fixture context"));
        context
            .run(|| {
                let mut config = context.manager().load()?;
                let template = config.sections.get(name).cloned().unwrap_or_default();
                config.sections.insert(format!("{name}-backup"), template);
                context.manager().save(&config)
            })
            .unwrap_or_else(|_| panic!("fixture profiles"));
    }
    let save = |path: std::path::PathBuf, content: &str, format: Format| {
        let document = Document::new(
            path,
            None,
            BackupPolicy::Dir {
                dir: root.join("fixture-backups"),
                prefix: "native-qa".into(),
            },
            Some(root.join("fixture-locks")),
            format,
        );
        let snapshot = document.read().unwrap_or_else(|_| panic!("fixture read"));
        assert!(matches!(
            document.save_text(content, &snapshot),
            Ok(SaveOutcome::Saved { .. })
        ));
    };
    // Each existing adapter deliberately captures a separate fixture root.
    // Seed all of them explicitly, instead of falling through to a user's HOME.
    for directory in [
        "desktop-platform-fixture",
        "desktop-resource-fixture",
        "desktop-mcp-fixture",
    ] {
        let roots =
            ccr_cli::application::platform_roots::PlatformRoots::fixture(&root.join(directory))
                .unwrap_or_else(|_| panic!("captured roots"));
        save(
            roots.claude.settings_file,
            r#"{"model":"demo-sonnet","agents":[{"name":"reviewer","model":"demo-sonnet","tools":[]}],"plugins":[{"id":"demo-plugin","name":"demo-plugin","version":"0.0.0","enabled":false}],"slashCommands":[{"name":"review","description":"Synthetic command","command":"fixture-do-not-run"}],"mcpServers":{"demo-files":{"command":"fixture-do-not-run"}}}"#,
            Format::Json,
        );
        save(
            roots.codex_runtime.join("config.toml"),
            "model = 'demo-codex-model'\n[mcp_servers.demo]\ncommand = 'fixture-do-not-run'\n",
            Format::Toml,
        );
        save(
            roots.grok_runtime.join("config.toml"),
            "[models]\ndefault = 'demo-grok-model'\n",
            Format::Toml,
        );
        save(
            roots.home.join(".config/opencode/opencode.json"),
            r#"{"model":"demo/provider","provider":{"demo":{"name":"Synthetic provider"}}}"#,
            Format::Json,
        );
    }
    let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
    let services = Services::new(owner.handle(), source);
    let requests = ccr_gpui::runtime::insights::InsightsRequests::new(&services);
    requests
        .observer_save(
            RequestKey {
                environment: EnvironmentId::Local("native-demo".into()),
                platform: Platform::Claude,
                generation: 1,
                request_id: 1,
            },
            None,
            SubscriptionDto {
                mode: "subscription".into(),
                plan: "custom".into(),
                monthly_usd: 20.0,
            },
        )
        .unwrap_or_else(|_| panic!("fixture subscription"));
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match requests.observer_update() {
            ccr_gpui::runtime::insights::observer::Update::Finished(Ok(_)) => break,
            ccr_gpui::runtime::insights::observer::Update::Finished(Err(_)) => {
                panic!("fixture observer write")
            }
            _ => {}
        }
        assert!(Instant::now() < deadline, "fixture observer timeout");
        std::thread::sleep(Duration::from_millis(10));
    }
    owner.shutdown();
    std::fs::write(root.join("fixture.json"),serde_json::to_string_pretty(&serde_json::json!({"demo":true,"profiles":root.join("profiles.toml"),"registry":root.join("registry.toml"),"observer_db":root.join("observer.db")})).unwrap_or_else(|e|panic!("manifest: {e}"))).unwrap_or_else(|e|panic!("manifest: {e}"));
}

#[gpui_kit::test]
async fn platform_workbench_is_default_and_scoped_navigation_keeps_platform(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
    let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
    let source = ConfigSource::Fixture {
        profiles: root.path().join("profiles.toml"),
        registry: root.path().join("registry.toml"),
        delay: Duration::ZERO,
    };
    seed_catalog(
        &source,
        ccr_config::Platform::Claude,
        "claude-workbench",
        true,
    );
    seed_catalog(
        &source,
        ccr_config::Platform::Codex,
        "codex-workbench",
        false,
    );
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| Shell::new(Services::new(owner.handle(), source), window, cx))
        })
        .unwrap_or_else(|e| panic!("window: {e}"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-claude-code", cx);
        assert!(window.find("platform-workbench").visible());
        assert!(window.try_find("configuration-filter").is_none());
        assert_eq!(window.find("workbench-title").label(), Some("Claude Code"));
        for id in [
            "tool-profiles",
            "tool-auth",
            "tool-platform-mcp",
            "tool-agents",
            "tool-plugins",
            "tool-slash-commands",
            "tool-files",
            "tool-platform-usage",
            "tool-subscription",
        ] {
            assert!(window.find(id).visible(), "missing scoped link {id}");
        }
    })
    .unwrap_or_else(|e| panic!("default: {e}"));
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window
            .find("workbench-current-profile")
            .label()
            .is_some_and(|label| label.contains("claude-workbench"))
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("tool-agents", cx);
        assert!(window.find("page-claude-code").visible());
        assert!(window.try_find("claude-resource-agents").is_some());
        window.click("tool-profiles", cx);
    })
    .unwrap_or_else(|e| panic!("scoped agents: {e}"));
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("configuration:claude-workbench").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-codex", cx);
        assert!(window.find("platform-workbench").visible());
        assert_eq!(window.find("workbench-title").label(), Some("Codex"));
        assert!(!page_labels(window).contains("claude-workbench"));
        window.click("tool-platform-usage", cx);
        assert!(window.find("page-codex").visible());
        assert_eq!(window.find("insights-filter-0").value(), Some("codex"));
        window.click("nav-usage", cx);
        assert_eq!(window.find("insights-filter-0").value(), Some(""));
        window.click("nav-grok", cx);
        assert!(window.try_find("tool-platform-mcp").is_none());
        assert!(window.try_find("workbench-mcp").is_none());
        assert!(window.try_find("tool-plugins").is_none());
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("platform isolation: {e}"));
    owner.shutdown();
}

#[gpui_kit::test]
async fn profile_pages_show_only_their_catalog(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
    let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
    let source = ConfigSource::Fixture {
        profiles: root.path().join("profiles.toml"),
        registry: root.path().join("registry.toml"),
        delay: Duration::ZERO,
    };
    seed_catalog(&source, ccr_config::Platform::Claude, "claude-kept", true);
    seed_catalog(&source, ccr_config::Platform::Codex, "codex-kept", false);
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(gpui_kit::WindowBounds::Windowed(
                    gpui_kit::Bounds::centered(
                        None,
                        gpui_kit::size(gpui_kit::px(1280.), gpui_kit::px(800.)),
                        cx,
                    ),
                )),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| Shell::new(Services::new(owner.handle(), source), window, cx)),
        )
        .unwrap_or_else(|e| panic!("window: {e}"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("nav-converter").is_none());
        window.click("nav-claude-code", cx);
        window.click("tool-profiles", cx);
    })
    .unwrap_or_else(|e| panic!("open claude: {e}"));
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("configuration:claude-kept").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.find("page-claude-code").visible());
        let text = page_labels(window);
        assert_eq!(
            window.find("configuration:claude-kept").label(),
            Some("claude-kept, current")
        );
        assert_eq!(
            window.find("configuration-status").label(),
            Some("1 profile")
        );
        assert!(!text.contains("codex-kept"));
        assert!(!text.contains("sk-native-profile-secret-sentinel"));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        window.click("configuration:claude-kept", cx);
        window.render_frame(cx);
        let credential = window
            .find("configuration-credential")
            .label()
            .unwrap_or_default()
            .to_string();
        assert!(credential.contains("sk-n...inel"), "{credential}");
        assert!(!credential.contains("sk-native-profile-secret-sentinel"));
        assert!(!page_labels(window).contains("sk-native-profile-secret-sentinel"));
        window.click("nav-codex", cx);
        window.click("tool-profiles", cx);
    })
    .unwrap_or_else(|e| panic!("open codex: {e}"));
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("configuration:codex-kept").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.find("page-codex").visible());
        let text = page_labels(window);
        assert_eq!(
            window.find("configuration:codex-kept").label(),
            Some("codex-kept")
        );
        assert_eq!(
            window.find("configuration-status").label(),
            Some("1 profile")
        );
        assert!(!text.contains("claude-kept"));
        assert!(!text.contains("sk-native-profile-secret-sentinel"));
        window.click("nav-grok", cx);
        window.click("tool-profiles", cx);
    })
    .unwrap_or_else(|e| panic!("open grok: {e}"));
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window
            .try_find("configuration-status")
            .and_then(|node| {
                node.label()
                    .map(|label| label.contains("Catalog or profile not found"))
            })
            .unwrap_or(false)
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.find("page-grok").visible());
        let text = page_labels(window);
        assert!(text.contains("Catalog or profile not found"));
        assert!(!text.contains("claude-kept"));
        assert!(!text.contains("codex-kept"));
        assert!(window.try_find("configuration:claude-kept").is_none());
        assert!(window.try_find("configuration:codex-kept").is_none());
        assert!(!text.contains("sk-native-profile-secret-sentinel"));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("grok: {e}"));
    owner.shutdown();
}

fn write_fixture(path: &std::path::Path, body: &str) {
    std::fs::create_dir_all(path.parent().unwrap_or_else(|| panic!("parent")))
        .unwrap_or_else(|e| panic!("dir: {e}"));
    std::fs::write(path, body).unwrap_or_else(|e| panic!("write: {e}"));
}

#[gpui_kit::test]
async fn platform_routes_show_supported_local_status_without_secrets(cx: &mut TestAppContext) {
    // A newly mounted button may omit its disabled snapshot. Wait for a visible
    // read result instead; successful values are asserted separately below.
    async fn loaded(cx: &mut TestAppContext, handle: gpui_kit::AnyWindowHandle) {
        cx.wait_for(handle, Duration::from_secs(10), |window, _| {
            window.try_find("platform-file-status").is_some()
                || window.try_find("platform-operation-status").is_some()
        })
        .await;
    }
    fn safe_state(window: &gpui_kit::Window) -> String {
        let nodes = gpui_kit::base::test_support::snapshots(window);
        let ids: Vec<_> = nodes
            .iter()
            .filter_map(|node| {
                let id = format!("{:?}", node.path().last()?);
                id.contains("platform-").then_some(id)
            })
            .collect();
        let statuses: Vec<_> = [
            "platform-file-status",
            "platform-auth-status",
            "platform-provider-status",
            "platform-operation-status",
        ]
        .into_iter()
        .filter_map(|id| {
            window
                .try_find(id)
                .map(|node| (id, node.label().map(str::to_owned)))
        })
        .collect();
        format!("field paths={ids:?}; safe statuses={statuses:?}")
    }
    fn private_projection(window: &gpui_kit::Window, secret: &str, foreign_models: &[&str]) {
        for node in gpui_kit::base::test_support::snapshots(window) {
            for value in [node.label(), node.value()].into_iter().flatten() {
                assert!(
                    !value.contains(secret),
                    "stored secret reached a visible projection"
                );
                for model in foreign_models {
                    assert!(
                        !value.contains(model),
                        "another platform's model reached this projection"
                    );
                }
            }
        }
    }
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
    let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
    let home = root.path().join("desktop-platform-fixture").join("home");
    let secret = "sk-platform-route-sentinel";
    write_fixture(
        &home.join(".claude/settings.json"),
        &format!(r#"{{"model":"sonnet","env":{{"TOKEN":"{secret}"}}}}"#),
    );
    write_fixture(
        &home.join(".codex/config.toml"),
        "model = \"gpt-5\"\ncli_auth_credentials_store = \"file\"\n",
    );
    write_fixture(
        &home.join(".grok/config.toml"),
        "[models]\ndefault = \"grok-3\"\n",
    );
    let scope = "https://auth.x.ai::fixture-client";
    let grok_auth = home.join(".grok/auth.json");
    write_fixture(
        &grok_auth,
        &serde_json::json!({(scope): {
            "auth_mode": "oidc",
            "oidc_issuer": "https://auth.x.ai",
            "oidc_client_id": "fixture-client",
            "key": secret,
            "user_id": "synthetic-grok-user",
            "create_time": "2026-01-01T00:00:00Z"
        }})
        .to_string(),
    );
    let captured = ccr_cli::application::platform_roots::PlatformRoots::fixture(
        &root
            .path()
            .join("desktop-platform-fixture")
            .canonicalize()
            .unwrap_or_else(|error| panic!("fixture root: {error}")),
    )
    .unwrap_or_else(|error| panic!("captured roots: {error}"));
    let grok = ccr_cli::services::GrokAuthService::from_roots(captured);
    let revision = grok
        .read_snapshot()
        .unwrap_or_else(|error| panic!("fixture Grok state: {error}"))
        .revision;
    grok.save_current("grok-fixture-only", scope, &revision, false)
        .unwrap_or_else(|error| panic!("fixture Grok account: {error}"));
    // A saved fixture account must be visible even without runtime credentials.
    std::fs::remove_file(&grok_auth)
        .unwrap_or_else(|error| panic!("clear fixture runtime: {error}"));
    write_fixture(
        &home.join(".config/opencode/opencode.json"),
        &format!(
            r#"{{"model":"opencode-model","provider":{{"fixture-provider":{{"apiKey":"{secret}"}}}}}}"#
        ),
    );
    write_fixture(
        &home.join(".gemini/antigravity-cli/settings.json"),
        &format!(r#"{{"apiKey":"{secret}","theme":"dark"}}"#),
    );
    let original_sources: Vec<_> = [
        ".claude/settings.json",
        ".codex/config.toml",
        ".grok/config.toml",
        ".config/opencode/opencode.json",
        ".gemini/antigravity-cli/settings.json",
    ]
    .into_iter()
    .map(|relative| {
        let path = home.join(relative);
        let bytes = std::fs::read(&path).unwrap_or_else(|_| panic!("fixture source"));
        (path, bytes)
    })
    .collect();
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                Shell::new(
                    Services::new(
                        owner.handle(),
                        ConfigSource::Fixture {
                            profiles: root.path().join("profiles.toml"),
                            registry: root.path().join("registry.toml"),
                            delay: Duration::ZERO,
                        },
                    ),
                    window,
                    cx,
                )
            })
        })
        .unwrap_or_else(|e| panic!("window: {e}"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-claude-code", cx);
        window.click("tool-profiles", cx);
    })
    .unwrap_or_else(|e| panic!("claude catalog: {e}"));
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window
            .try_find("configuration-status")
            .and_then(|node| node.label().map(|label| label.contains("not found")))
            .unwrap_or(false)
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.find("page-claude-code").visible());
        assert!(!page_labels(window).contains("model: sonnet"));
        window.click("tool-accounts", cx);
    })
    .unwrap_or_else(|e| panic!("claude accounts: {e}"));
    loaded(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window
                .try_find("platform-setting-model")
                .and_then(|node| node.value().map(str::to_owned)),
            Some("sonnet".into()),
            "{}",
            safe_state(window)
        );
        assert!(window.try_find("platform-common-form").is_some());
        assert!(window.try_find("platform-field-model").is_none());
        private_projection(window, secret, &["gpt-5", "grok-3", "opencode-model"]);
        window.click("tool-auth", cx);
    })
    .unwrap_or_else(|e| panic!("claude settings: {e}"));
    loaded(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window
                .try_find("platform-field-model")
                .and_then(|node| node.label().map(str::to_owned)),
            Some("model: sonnet".into()),
            "{}",
            safe_state(window)
        );
        let text = page_labels(window);
        assert!(text.contains("settings.json was found"));
        assert!(text.contains("No runtime credential was detected."));
        assert!(text.contains("No saved accounts."));
        assert!(!text.contains(secret));
        assert!(!text.contains("TOKEN"));
        assert!(!text.contains(" / "));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        private_projection(window, secret, &["gpt-5", "grok-3", "opencode-model"]);
        window.click("nav-codex", cx);
        window.click("tool-profiles", cx);
        window.click("tool-accounts", cx);
    })
    .unwrap_or_else(|e| panic!("codex accounts: {e}"));
    loaded(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window
                .try_find("platform-setting-model")
                .and_then(|node| node.value().map(str::to_owned)),
            Some("gpt-5".into()),
            "{}",
            safe_state(window)
        );
        assert!(window.try_find("platform-common-form").is_some());
        private_projection(window, secret, &["sonnet", "grok-3", "opencode-model"]);
        window.click("tool-auth", cx);
    })
    .unwrap_or_else(|e| panic!("codex settings: {e}"));
    loaded(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window
                .try_find("platform-field-model")
                .and_then(|node| node.label().map(str::to_owned)),
            Some("model: gpt-5".into()),
            "{}",
            safe_state(window)
        );
        let text = page_labels(window);
        assert!(window.find("page-codex").visible());
        assert!(text.contains("No runtime credential was detected."));
        assert!(text.contains("No saved accounts."));
        assert!(!text.contains(secret));
        private_projection(window, secret, &["sonnet", "grok-3", "opencode-model"]);
        window.click("nav-grok", cx);
        window.click("tool-profiles", cx);
        window.click("tool-accounts", cx);
    })
    .unwrap_or_else(|e| panic!("grok accounts: {e}"));
    loaded(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window
                .try_find("platform-setting-default_model")
                .and_then(|node| node.value().map(str::to_owned)),
            Some("grok-3".into()),
            "{}",
            safe_state(window)
        );
        assert!(window.try_find("platform-common-form").is_some());
        private_projection(window, secret, &["sonnet", "gpt-5", "opencode-model"]);
        window.click("tool-auth", cx);
    })
    .unwrap_or_else(|e| panic!("grok settings: {e}"));
    loaded(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window
                .try_find("platform-field-default_model")
                .and_then(|node| node.label().map(str::to_owned)),
            Some("default_model: grok-3".into()),
            "{}",
            safe_state(window)
        );
        let text = page_labels(window);
        assert!(window.find("page-grok").visible());
        assert!(text.contains("No runtime credential was detected."));
        assert_eq!(
            window.find("platform-account-grok-fixture-only").label(),
            Some("grok-fixture-only")
        );
        assert!(!text.contains("No saved accounts."));
        assert!(!text.contains("logged in"));
        assert!(!text.contains(secret));
        private_projection(window, secret, &["sonnet", "gpt-5", "opencode-model"]);
        window.click("nav-opencode", cx);
        window.click("tool-accounts", cx);
    })
    .unwrap_or_else(|e| panic!("opencode: {e}"));
    loaded(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let text = page_labels(window);
        assert!(window.find("page-opencode").visible());
        assert_eq!(
            window
                .try_find("platform-setting-model")
                .and_then(|node| node.value().map(str::to_owned)),
            Some("opencode-model".into()),
            "{}",
            safe_state(window)
        );
        assert!(
            window
                .find("platform-provider-status")
                .label()
                .is_some_and(|label| label.contains("fixture-provider"))
        );
        assert!(window.try_find("platform-common-form").is_some());
        assert!(window.try_find("tool-auth").is_none());
        assert!(text.contains("Accounts are not supported. Login state is not reported."));
        assert!(!text.contains(secret));
        assert!(!text.contains("apiKey"));
        private_projection(window, secret, &["sonnet", "gpt-5", "grok-3"]);
        window.click("nav-antigravity", cx);
        window.click("tool-accounts", cx);
    })
    .unwrap_or_else(|e| panic!("antigravity: {e}"));
    loaded(cx, handle).await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let text = page_labels(window);
        assert!(window.find("page-antigravity").visible());
        assert!(
            window
                .find("platform-file-status")
                .label()
                .is_some_and(|label| label.contains("Typed settings are not supported")),
            "{}",
            safe_state(window)
        );
        assert!(text.contains("Accounts are not supported. Login state is not reported."));
        assert!(window.try_find("platform-save").is_none());
        assert!(window.try_find("platform-common-form").is_none());
        assert!(window.try_find("platform-setting-model").is_none());
        assert!(window.try_find("tool-auth").is_none());
        assert!(!text.contains(secret));
        assert!(!text.contains("theme"));
        assert!(!text.contains("dark"));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        assert!(window.try_find("nav-settings").is_some());
        private_projection(
            window,
            secret,
            &["sonnet", "gpt-5", "grok-3", "opencode-model"],
        );
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("status: {e}"));
    owner.shutdown();
    for (path, original) in original_sources {
        assert!(
            std::fs::read(path).unwrap_or_else(|_| panic!("fixture source remains readable"))
                == original,
            "read-only platform navigation changed source bytes"
        );
    }
    assert!(
        !grok_auth.exists(),
        "observing a saved account must not create runtime credentials"
    );
}
