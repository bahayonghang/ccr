use super::*;
use crate::features::workbench::Link;
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

pub(super) fn link_lens(link: Link) -> Lens {
    match link {
        Link::Profiles => Lens::Profiles,
        Link::Auth => Lens::Auth,
        Link::Settings => Lens::Accounts,
        Link::Mcp => Lens::Mcp,
        Link::Files => Lens::Files,
        Link::Agents => Lens::Agents,
        Link::Plugins => Lens::Plugins,
        Link::Commands => Lens::SlashCommands,
        Link::Usage => Lens::Usage,
        Link::Observer => Lens::Observer,
        Link::Sessions => Lens::Sessions,
    }
}

impl Shell {
    pub(super) fn platform(&self) -> Option<Platform> {
        match self.destination {
            Destination::ClaudeCode => Some(Platform::Claude),
            Destination::Codex => Some(Platform::Codex),
            Destination::Grok => Some(Platform::Grok),
            Destination::Antigravity => Some(Platform::Gemini),
            Destination::OpenCode => Some(Platform::OpenCode),
            _ => None,
        }
    }
    pub(super) fn platform_toolbar(&self, cx: &mut Context<Self>) -> AnyElement {
        let platform = self.platform().unwrap_or(Platform::Claude);
        let mut items = vec![("tool-home", Lens::Primary, "Workbench", "工作台")];
        if matches!(
            platform,
            Platform::Claude | Platform::Codex | Platform::Grok
        ) {
            items.push(("tool-profiles", Lens::Profiles, "Profiles", "Profiles"));
            items.push(("tool-auth", Lens::Auth, "Authentication", "认证"));
        }
        items.push((
            "tool-accounts",
            Lens::Accounts,
            "Settings and sources",
            "设置与来源",
        ));
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
            items.push(("tool-files", Lens::Files, "Instruction files", "指令文件"));
        }
        match platform {
            Platform::Codex => items.push((
                "tool-extensions",
                Lens::Extensions,
                "Agents and sources",
                "代理与来源",
            )),
            Platform::OpenCode => items.push((
                "tool-extensions",
                Lens::Extensions,
                "Agents and plugins",
                "代理与插件",
            )),
            Platform::Gemini => {
                items.push(("tool-extensions", Lens::Extensions, "Commands", "Commands"))
            }
            _ => {}
        }
        if platform == Platform::Claude {
            items.extend([
                ("tool-agents", Lens::Agents, "Agents", "Agents"),
                ("tool-plugins", Lens::Plugins, "Plugins", "Plugins"),
                (
                    "tool-slash-commands",
                    Lens::SlashCommands,
                    "Commands",
                    "Commands",
                ),
                (
                    "tool-subscription",
                    Lens::Observer,
                    "Subscription and tools",
                    "订阅与工具",
                ),
            ]);
        }
        if matches!(platform, Platform::Claude | Platform::Codex) {
            items.push(("tool-platform-sessions", Lens::Sessions, "Sessions", "会话"));
        }
        items.push(("tool-platform-usage", Lens::Usage, "Usage", "用量"));
        let mut row = div().flex().flex_wrap().gap(px(theme::NAV_ROW_GAP));
        for (id, lens, en, zh) in items {
            row = row.child(self.lens_button(id, lens, en, zh, cx));
        }
        div()
            .id("platform-context-navigation")
            .test_support()
            .flex_shrink_0()
            .px(px(theme::CONTENT_PADDING))
            .py(px(theme::CONTROL_GAP))
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(crate::features::workbench::name(platform)),
            )
            .child(row)
            .into_any_element()
    }
}
