use super::*;
use gpui_kit::component::IconName;

impl Destination {
    pub(super) fn icon(self) -> IconName {
        match self {
            Self::Dashboard => IconName::LayoutDashboard,
            Self::AgentSessions => IconName::BookOpen,
            Self::McpManager => IconName::Network,
            Self::ClaudeCode => IconName::Bot,
            Self::Codex => IconName::Cpu,
            Self::Grok => IconName::Asterisk,
            Self::Antigravity => IconName::Globe,
            Self::OpenCode => IconName::SquareTerminal,
            Self::Commands => IconName::Play,
            Self::Checkin => IconName::Calendar,
            Self::Sync => IconName::RefreshCw,
            Self::Usage => IconName::ChartPie,
            Self::Settings => IconName::Settings2,
        }
    }
}

impl Destination {
    pub(super) fn platform(self) -> Option<Platform> {
        match self {
            Destination::ClaudeCode => Some(Platform::Claude),
            Destination::Codex => Some(Platform::Codex),
            Destination::Grok => Some(Platform::Grok),
            Destination::Antigravity => Some(Platform::Gemini),
            Destination::OpenCode => Some(Platform::OpenCode),
            _ => None,
        }
    }
}

impl Shell {
    pub(super) fn platform(&self) -> Option<Platform> {
        self.destination.platform()
    }
    pub(super) fn platform_items(&self) -> Vec<LensItem> {
        platform_items(self.platform().unwrap_or(Platform::Claude))
    }
}

pub(super) fn platform_items(platform: Platform) -> Vec<LensItem> {
    let mut items = vec![("tool-home", Lens::Primary, "Workbench", "工作台")];
    if matches!(
        platform,
        Platform::Claude | Platform::Codex | Platform::Grok
    ) {
        items.extend([
            ("tool-profiles", Lens::Profiles, "Profiles", "配置方案"),
            ("tool-auth", Lens::Auth, "Accounts", "账户"),
        ]);
    }
    items.push(("tool-settings", Lens::Accounts, "Settings", "设置"));
    if matches!(platform, Platform::Codex | Platform::OpenCode) {
        items.push(("tool-providers", Lens::Providers, "Providers", "供应商"));
    }
    if platform == Platform::Codex {
        items.push(("tool-models", Lens::Models, "Models", "模型"));
    }
    if !matches!(platform, Platform::Grok | Platform::OpenCode) {
        items.push(("tool-platform-mcp", Lens::Mcp, "MCP", "MCP"));
    }
    if platform != Platform::Grok {
        items.extend([
            ("tool-files", Lens::Files, "Instructions", "指令"),
            ("tool-extensions", Lens::Extensions, "Extensions", "扩展"),
        ]);
    }
    items.push(("tool-platform-usage", Lens::Usage, "Usage", "用量"));
    if matches!(platform, Platform::Claude | Platform::Codex) {
        items.push(("tool-platform-sessions", Lens::Sessions, "Sessions", "会话"));
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn approved_navigation_has_exact_platform_capabilities_and_ids() {
        for (platform, expected) in [
            (
                Platform::Claude,
                &[
                    "tool-home",
                    "tool-profiles",
                    "tool-auth",
                    "tool-settings",
                    "tool-platform-mcp",
                    "tool-files",
                    "tool-extensions",
                    "tool-platform-usage",
                    "tool-platform-sessions",
                ][..],
            ),
            (
                Platform::Codex,
                &[
                    "tool-home",
                    "tool-profiles",
                    "tool-auth",
                    "tool-settings",
                    "tool-providers",
                    "tool-models",
                    "tool-platform-mcp",
                    "tool-files",
                    "tool-extensions",
                    "tool-platform-usage",
                    "tool-platform-sessions",
                ][..],
            ),
            (
                Platform::Grok,
                &[
                    "tool-home",
                    "tool-profiles",
                    "tool-auth",
                    "tool-settings",
                    "tool-platform-usage",
                ][..],
            ),
            (
                Platform::Gemini,
                &[
                    "tool-home",
                    "tool-settings",
                    "tool-platform-mcp",
                    "tool-files",
                    "tool-extensions",
                    "tool-platform-usage",
                ][..],
            ),
            (
                Platform::OpenCode,
                &[
                    "tool-home",
                    "tool-settings",
                    "tool-providers",
                    "tool-files",
                    "tool-extensions",
                    "tool-platform-usage",
                ][..],
            ),
        ] {
            let ids: Vec<_> = platform_items(platform).iter().map(|item| item.0).collect();
            assert_eq!(ids, expected, "{platform:?}");
        }
    }

    #[test]
    fn removed_platform_tab_ids_do_not_appear_on_any_platform() {
        // PRD #161 第 9 节已删除或改名的 tab id，以及旧导航里 Primary 指向方案列表的 `tool-platform`。
        for platform in [
            Platform::Claude,
            Platform::Codex,
            Platform::Grok,
            Platform::Gemini,
            Platform::OpenCode,
        ] {
            for item in platform_items(platform) {
                assert!(
                    !matches!(
                        item.0,
                        "tool-accounts"
                            | "tool-agents"
                            | "tool-plugins"
                            | "tool-slash-commands"
                            | "tool-subscription"
                            | "tool-platform"
                    ),
                    "{platform:?} still lists {}",
                    item.0
                );
            }
        }
    }

    #[gpui_kit::test]
    fn platform_tabs_render_no_removed_or_global_only_ids(cx: &mut gpui_kit::TestAppContext) {
        use crate::runtime::{ConfigSource, Services};
        use ccr_app::runtime::RuntimeOwner;
        use gpui_kit::test::TestWindowExt;

        // PRD #161 第 9 节：删除类 ID 在任何平台 tab 都不出现；页内切换类 ID 只在全局视图出现。
        const ABSENT_IN_PLATFORM: &[&str] = &[
            "tool-accounts",
            "tool-agents",
            "tool-plugins",
            "tool-slash-commands",
            "tool-subscription",
            "tool-platform",
            "workbench-auth",
            "workbench-profiles",
            "workbench-settings",
            "workbench-mcp",
            "workbench-files",
            "workbench-usage",
            "workbench-sessions",
            "workbench-observer",
            "profile-policy-profiles",
            "profile-policy-configurations",
            "profile-clean-backups",
            "profile-clear-history",
            "resource-platform-claude",
            "resource-platform-codex",
            "resource-platform-gemini",
            "resource-platform-opencode",
            "resource-instructions",
            "resource-claude",
            "platform-settings",
            "platform-auth",
            "insights-usage",
            "insights-capabilities",
            "insights-observer-tab",
        ];
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let (handle, _) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(gpui_kit::WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    Shell::new(
                        Services::new(
                            owner.handle(),
                            ConfigSource::Fixture {
                                profiles: root.path().join("missing.toml"),
                                registry: root.path().join("registry.toml"),
                                delay: std::time::Duration::ZERO,
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
            for (nav, platform) in [
                ("nav-claude-code", Platform::Claude),
                ("nav-codex", Platform::Codex),
                ("nav-grok", Platform::Grok),
                ("nav-antigravity", Platform::Gemini),
                ("nav-opencode", Platform::OpenCode),
            ] {
                window.click(nav, cx);
                for (tab, ..) in platform_items(platform) {
                    window.click(tab, cx);
                    assert_eq!(window.find(tab).checked(), Some(true), "{nav} {tab}");
                    // 现状例外：OpenCode 的 tui.json 没有对应 tab，平台视图在供应商页保留
                    // 「终端界面 / 设置」子入口（features/platforms 的嵌入分支），这里如实固定。
                    let opencode_sub_entry =
                        platform == Platform::OpenCode && tab == "tool-providers";
                    for &id in ABSENT_IN_PLATFORM {
                        if opencode_sub_entry && id == "platform-settings" {
                            assert!(window.try_find(id).is_some(), "{nav} {tab} lost {id}");
                            continue;
                        }
                        assert!(window.try_find(id).is_none(), "{nav} {tab} renders {id}");
                    }
                }
            }
            // 全局用量目的地按原用途保留页内切换。
            window.click("nav-usage", cx);
            for id in [
                "insights-usage",
                "insights-capabilities",
                "insights-observer-tab",
            ] {
                assert!(window.try_find(id).is_some(), "global usage lost {id}");
            }
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("platform ids: {e}"));
        owner.shutdown();
    }

    // PRD 8.3「导航」：每个平台的每个 tab，唯一 h1 都是当前 tab 名，且平台导航只出现一次。
    #[gpui_kit::test]
    fn every_platform_heading_names_the_selected_tab(cx: &mut gpui_kit::TestAppContext) {
        use crate::runtime::{ConfigSource, Services};
        use ccr_app::runtime::RuntimeOwner;
        use gpui_kit::test::TestWindowExt;
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let (handle, _) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(gpui_kit::WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    Shell::new(
                        Services::new(
                            owner.handle(),
                            ConfigSource::Fixture {
                                profiles: root.path().join("missing.toml"),
                                registry: root.path().join("registry.toml"),
                                delay: std::time::Duration::ZERO,
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
            for (nav, platform) in [
                ("nav-claude-code", Platform::Claude),
                ("nav-codex", Platform::Codex),
                ("nav-grok", Platform::Grok),
                ("nav-antigravity", Platform::Gemini),
                ("nav-opencode", Platform::OpenCode),
            ] {
                window.click(nav, cx);
                for (tab, _, english, _) in platform_items(platform) {
                    window.click(tab, cx);
                    window.render_frame(cx);
                    let heading = window.find("platform-tab-heading");
                    assert_eq!(heading.label(), Some(english), "{nav} {tab}");
                    assert_eq!(heading.role(), Some(gpui_kit::Role::Heading), "{nav} {tab}");
                    assert!(heading.visible(), "{nav} {tab}");
                    assert!(window.find("platform-context-navigation").visible());
                    assert!(window.find(tab).checked() == Some(true), "{nav} {tab}");
                }
            }
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("headings: {e}"));
        owner.shutdown();
    }
}
