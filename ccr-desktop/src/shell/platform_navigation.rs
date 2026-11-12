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
        for (platform, count) in [
            (Platform::Claude, 9),
            (Platform::Codex, 11),
            (Platform::Grok, 5),
            (Platform::Gemini, 6),
            (Platform::OpenCode, 6),
        ] {
            let items = platform_items(platform);
            assert_eq!(items.len(), count);
            assert!(items.iter().any(|item| item.0 == "tool-settings"));
            assert!(!items.iter().any(|item| matches!(
                item.0,
                "tool-accounts"
                    | "tool-agents"
                    | "tool-plugins"
                    | "tool-subscription"
                    | "tool-slash-commands"
            )));
        }
    }
}
