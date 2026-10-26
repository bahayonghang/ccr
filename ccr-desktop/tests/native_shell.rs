//! Core rendered shell interactions. Native OS IME/UIA are separate evidence.
use ccr_app::runtime::RuntimeOwner;
use ccr_gpui::{
    locales::Language,
    runtime::{ConfigSource, Services},
    shell::Shell,
};
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::test::TestWindowExt;
use gpui_kit::{AppContext, TestAppContext, WindowOptions};
use std::time::Duration;

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
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-settings", cx);
        window.click("theme-dark", cx);
        assert_eq!(Theme::global(cx).mode, ThemeMode::Dark);
        assert_eq!(
            Theme::global(cx).colors.background,
            gpui_kit::Hsla::from(gpui_kit::rgb(0x151617))
        );
        assert_eq!(
            gpui_kit::base::Theme::global(cx).tokens.colors,
            Theme::global(cx).semantic_tokens().colors
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
        assert_eq!(window.find("nav-configurations").label(), Some("配置"));
        assert_eq!(window.find("theme-system").label(), Some("跟随系统"));
        window.click("language-en", cx);
        assert_eq!(
            window.find("nav-configurations").label(),
            Some("Configurations")
        );
        window.click("theme-system", cx);
        assert_eq!(
            gpui_kit::base::Theme::global(cx).tokens.colors,
            Theme::global(cx).semantic_tokens().colors
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
        window.click("configuration-filter", cx);
        window.input("原生读取", cx);
        window.press("left", cx);
        window.press("backspace", cx);
        assert_eq!(window.find("configuration-filter").value(), Some("原生取"));
        window.press("f6", cx); // main -> status
        window.press("f6", cx); // status -> navigation
        assert_eq!(window.find("nav-configurations").focused(), Some(true));
        window.press("down", cx);
        assert_eq!(window.find("nav-favorites").focused(), Some(true));
        window.press("up", cx);
        assert_eq!(window.find("nav-configurations").focused(), Some(true));
        window.press("f6", cx); // navigation -> main
        assert_eq!(window.find("configuration-filter").focused(), Some(true));
        window.press("shift-f6", cx);
        assert_eq!(window.find("nav-configurations").focused(), Some(true));
        window.press("tab", cx);
        assert_eq!(window.find("nav-favorites").focused(), Some(true));
        window.press("shift-tab", cx);
        assert_eq!(window.find("nav-configurations").focused(), Some(true));
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
        window.render_frame(cx);window.click("nav-converter",cx);window.click("converter-acknowledge",cx);
        let input=gpui_kit::base::test_support::snapshots(window).into_iter().find(|node|node.label()==Some("Source configuration")).unwrap_or_else(||panic!("source input"));
        window.click(input.path().last().cloned().unwrap_or_else(||panic!("input identity")),cx);
        window.input("{\"mcpServers\":{\"fixture\":{\"command\":\"fixture-command\",\"env\":{\"KEY\":\"converter-secret-sentinel\"}}}}",cx);
        window.click("converter-run",cx);window.click("nav-configurations",cx);
        window.click("nav-converter",cx);assert!(window.find("converter-acknowledge").visible());
        window.click("converter-acknowledge",cx);
        let nodes=gpui_kit::base::test_support::snapshots(window);
        assert_eq!(nodes.iter().find(|node|node.label()==Some("Source configuration")).and_then(|node|node.value()),Some(""));
        assert_eq!(nodes.iter().find(|node|node.label()==Some("Converted preview")).and_then(|node|node.value()),Some(""));
        window.remove_window();
    }).unwrap_or_else(|_|panic!("navigation"));
    owner.shutdown();
}
