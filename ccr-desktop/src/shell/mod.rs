//! Native capability shell: one workbench destination per sidebar entry.
mod command_state;
pub(crate) mod semantics;
use crate::{
    app::ConfigurationApp,
    features::{insights::InsightsPage, overview::OverviewLink},
    locales::{Language, text},
    runtime::{Services, platforms::Surface},
    theme::{self, Preference},
};
use ccr_app::runtime::Platform;
use command_state::CommandState;
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, Selectable,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    AnyElement, AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, Subscription, Window, div, px,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Destination {
    Dashboard,
    AgentSessions,
    McpManager,
    ClaudeCode,
    Codex,
    Grok,
    Antigravity,
    OpenCode,
    Commands,
    Checkin,
    Sync,
    Usage,
    Settings,
}

impl Destination {
    fn page_id(self) -> &'static str {
        match self {
            Self::Dashboard => "page-dashboard",
            Self::AgentSessions => "page-agent-sessions",
            Self::McpManager => "page-mcp-manager",
            Self::ClaudeCode => "page-claude-code",
            Self::Codex => "page-codex",
            Self::Grok => "page-grok",
            Self::Antigravity => "page-antigravity",
            Self::OpenCode => "page-opencode",
            Self::Commands => "page-commands",
            Self::Checkin => "page-checkin",
            Self::Sync => "page-sync",
            Self::Usage => "page-usage",
            Self::Settings => "page-settings",
        }
    }
}

/// In-page capability. These are not sidebar entries.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lens {
    Primary,
    Accounts,
    Files,
    Converter,
    Favorites,
    Recent,
    Install,
}

pub struct Shell {
    configurations: Entity<ConfigurationApp>,
    commands: Entity<CommandState>,
    converter: Entity<crate::features::converter::Converter>,
    platforms: Entity<crate::features::platforms::PlatformsView>,
    insights: Entity<crate::features::insights::InsightsView>,
    resources: Entity<crate::features::resources::ResourcesView>,
    mcp: Entity<crate::features::mcp::McpView>,
    jobs: Entity<crate::features::jobs::JobsView>,
    install: Entity<crate::features::install::InstallView>,
    checkin: Entity<crate::features::checkin::CheckinView>,
    sync: Entity<crate::features::sync::SyncView>,
    overview: Entity<crate::features::overview::OverviewView>,
    destination: Destination,
    lens: Lens,
    focused_nav: usize,
    preference: Preference,
    fixture: bool,
    slow_fixture: bool,
    navigation_focus: FocusHandle,
    main_focus: FocusHandle,
    status_focus: FocusHandle,
    _appearance: Subscription,
    _activation: Subscription,
}

impl Shell {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.set_global(Language::English);
        theme::apply(Preference::Light, window, cx);
        let appearance = cx.observe_window_appearance(window, |this, window, cx| {
            if this.preference == Preference::System {
                theme::apply(this.preference, window, cx);
                cx.notify();
            }
        });
        let activation = cx.observe_window_activation(window, |this, window, cx| {
            if window.is_window_active() && window.focused(cx).is_none() {
                this.focus_main(window, cx);
            }
        });
        let shell = Self {
            configurations: cx.new(|cx| ConfigurationApp::new(services.clone(), window, cx)),
            commands: cx.new(|cx| CommandState::new(&services, window, cx)),
            converter: cx
                .new(|cx| crate::features::converter::Converter::new(services.clone(), window, cx)),
            platforms: cx.new(|cx| {
                crate::features::platforms::PlatformsView::new(services.clone(), window, cx)
            }),
            insights: cx.new(|cx| {
                crate::features::insights::InsightsView::new(services.clone(), window, cx)
            }),
            resources: cx.new(|cx| {
                crate::features::resources::ResourcesView::new(services.clone(), window, cx)
            }),
            mcp: cx.new(|cx| crate::features::mcp::McpView::new(services.clone(), window, cx)),
            jobs: cx.new(|cx| crate::features::jobs::JobsView::new(services.clone(), window, cx)),
            install: cx
                .new(|cx| crate::features::install::InstallView::new(services.clone(), window, cx)),
            checkin: cx
                .new(|cx| crate::features::checkin::CheckinView::new(services.clone(), window, cx)),
            sync: cx.new(|cx| crate::features::sync::SyncView::new(services.clone(), window, cx)),
            overview: cx
                .new(|cx| crate::features::overview::OverviewView::new(services.clone(), cx)),
            destination: Destination::Dashboard,
            lens: Lens::Primary,
            focused_nav: 0,
            preference: Preference::Light,
            fixture: services.is_fixture(),
            slow_fixture: services.is_slow_fixture(),
            navigation_focus: cx.focus_handle().tab_stop(false),
            main_focus: cx.focus_handle().tab_stop(false),
            status_focus: cx.focus_handle(),
            _appearance: appearance,
            _activation: activation,
        };
        shell.overview.update(cx, |view, cx| view.refresh(cx));
        shell
    }

    fn nav_items() -> [(&'static str, Destination, &'static str, &'static str); 13] {
        [
            (
                "nav-dashboard",
                Destination::Dashboard,
                "Overview",
                "运行概览",
            ),
            (
                "nav-agent-sessions",
                Destination::AgentSessions,
                "Agent sessions",
                "Agent 会话",
            ),
            (
                "nav-mcp-manager",
                Destination::McpManager,
                "MCP Manager",
                "MCP 管理",
            ),
            (
                "nav-claude-code",
                Destination::ClaudeCode,
                "Claude Code",
                "Claude Code",
            ),
            ("nav-codex", Destination::Codex, "Codex", "Codex"),
            ("nav-grok", Destination::Grok, "Grok", "Grok"),
            (
                "nav-antigravity",
                Destination::Antigravity,
                "Antigravity",
                "Antigravity",
            ),
            (
                "nav-opencode",
                Destination::OpenCode,
                "OpenCode",
                "OpenCode",
            ),
            ("nav-commands", Destination::Commands, "Commands", "命令"),
            ("nav-checkin", Destination::Checkin, "Check-ins", "签到管理"),
            ("nav-sync", Destination::Sync, "Sync", "同步"),
            ("nav-usage", Destination::Usage, "Usage", "用量"),
            ("nav-settings", Destination::Settings, "Settings", "设置"),
        ]
    }

    fn go(
        &mut self,
        destination: Destination,
        lens: Lens,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(index) = Self::nav_items()
            .iter()
            .position(|item| item.1 == destination)
        {
            self.focused_nav = index;
        }
        if self.destination == destination && self.lens == lens {
            return;
        }
        self.destination = destination;
        self.lens = lens;
        self.park(window, cx);
        self.activate(window, cx);
        cx.notify();
    }

    fn park(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.configurations
            .update(cx, |view, cx| view.leave(window, cx));
        self.commands.update(cx, |view, cx| view.suspend(cx));
        self.converter.update(cx, |view, cx| view.leave(window, cx));
        self.platforms.update(cx, |view, cx| view.leave(window, cx));
        self.insights.update(cx, |view, cx| view.suspend(cx));
        self.resources.update(cx, |view, cx| view.leave(window, cx));
        self.mcp.update(cx, |view, cx| view.suspend(cx));
        self.jobs.update(cx, |view, cx| view.suspend(cx));
        self.install.update(cx, |view, cx| view.suspend(cx));
        self.checkin.update(cx, |view, cx| view.suspend(cx));
        self.sync.update(cx, |view, cx| view.suspend(cx));
        self.overview.update(cx, |view, cx| view.suspend(cx));
    }

    fn activate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.lens {
            Lens::Converter => {}
            Lens::Favorites => self.commands.update(cx, |view, cx| view.open(false, cx)),
            Lens::Recent => self.commands.update(cx, |view, cx| view.open(true, cx)),
            Lens::Install => self.install.update(cx, |view, cx| view.refresh(cx)),
            Lens::Files => {
                if let Some(platform) = self.files_platform() {
                    self.resources
                        .update(cx, |view, cx| view.open_platform(platform, window, cx));
                }
            }
            Lens::Accounts => {
                if let Some((platform, surface)) = self.account_target() {
                    self.platforms
                        .update(cx, |view, cx| view.open(platform, surface, window, cx));
                }
            }
            Lens::Primary => self.activate_primary(window, cx),
        }
    }

    fn activate_primary(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.destination {
            Destination::Dashboard => self.overview.update(cx, |view, cx| view.refresh(cx)),
            Destination::AgentSessions => self
                .insights
                .update(cx, |view, cx| view.open_product(InsightsPage::Sessions, cx)),
            Destination::Usage => self
                .insights
                .update(cx, |view, cx| view.open_product(InsightsPage::Usage, cx)),
            Destination::ClaudeCode => self.configurations.update(cx, |view, cx| {
                view.open_platform(Platform::Claude, window, cx)
            }),
            Destination::Codex => self.configurations.update(cx, |view, cx| {
                view.open_platform(Platform::Codex, window, cx)
            }),
            Destination::Grok => self.configurations.update(cx, |view, cx| {
                view.open_platform(Platform::Grok, window, cx)
            }),
            Destination::OpenCode => self.platforms.update(cx, |view, cx| {
                view.open(Platform::OpenCode, Surface::Settings, window, cx)
            }),
            Destination::Antigravity => self.platforms.update(cx, |view, cx| {
                view.open(Platform::Gemini, Surface::LegacySettings, window, cx)
            }),
            Destination::Commands => self.jobs.update(cx, |view, cx| view.refresh(cx)),
            Destination::McpManager => self.mcp.update(cx, |view, cx| view.refresh(cx)),
            Destination::Checkin => self.checkin.update(cx, |view, cx| view.refresh(cx)),
            Destination::Sync => self.sync.update(cx, |view, cx| view.refresh(cx)),
            Destination::Settings => {}
        }
    }

    fn account_target(&self) -> Option<(Platform, Surface)> {
        match self.destination {
            Destination::ClaudeCode => Some((Platform::Claude, Surface::Settings)),
            Destination::Codex => Some((Platform::Codex, Surface::Settings)),
            Destination::Grok => Some((Platform::Grok, Surface::Settings)),
            _ => None,
        }
    }

    fn files_platform(&self) -> Option<Platform> {
        match self.destination {
            Destination::ClaudeCode => Some(Platform::Claude),
            Destination::Codex => Some(Platform::Codex),
            Destination::Antigravity => Some(Platform::Gemini),
            Destination::OpenCode => Some(Platform::OpenCode),
            _ => None,
        }
    }

    fn focus_main(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.lens {
            Lens::Favorites | Lens::Recent => {
                self.commands
                    .update(cx, |view, cx| view.focus_main(window, cx));
            }
            Lens::Converter => self
                .converter
                .update(cx, |view, cx| view.focus_main(window, cx)),
            Lens::Install => self
                .install
                .update(cx, |view, cx| view.focus_main(window, cx)),
            Lens::Files => self
                .resources
                .update(cx, |view, cx| view.focus_main(window, cx)),
            Lens::Accounts => self
                .platforms
                .update(cx, |view, cx| view.focus_main(window, cx)),
            Lens::Primary => match self.destination {
                Destination::Dashboard => self
                    .overview
                    .update(cx, |view, cx| view.focus_main(window, cx)),
                Destination::AgentSessions | Destination::Usage => self
                    .insights
                    .update(cx, |view, cx| view.focus_main(window, cx)),
                Destination::ClaudeCode | Destination::Codex | Destination::Grok => self
                    .configurations
                    .update(cx, |view, cx| view.focus_main(window, cx)),
                Destination::OpenCode | Destination::Antigravity => self
                    .platforms
                    .update(cx, |view, cx| view.focus_main(window, cx)),
                Destination::Commands => {
                    self.jobs.update(cx, |view, cx| view.focus_main(window, cx));
                }
                Destination::McpManager => {
                    self.mcp.update(cx, |view, cx| view.focus_main(window, cx));
                }
                Destination::Checkin => {
                    self.checkin
                        .update(cx, |view, cx| view.focus_main(window, cx));
                }
                Destination::Sync => {
                    self.sync.update(cx, |view, cx| view.focus_main(window, cx));
                }
                Destination::Settings => {
                    window.focus(&self.main_focus, cx);
                    window.focus_next(cx);
                }
            },
        }
    }

    fn region(&mut self, backward: bool, window: &mut Window, cx: &mut Context<Self>) {
        let in_nav = self.navigation_focus.contains_focused(window, cx);
        let in_status = self.status_focus.contains_focused(window, cx);
        let target = if backward {
            if in_nav {
                2
            } else if in_status {
                1
            } else {
                0
            }
        } else if in_nav {
            1
        } else if in_status {
            0
        } else {
            2
        };
        match target {
            0 => {
                window.focus(&self.navigation_focus, cx);
                window.focus_next(cx);
                self.focused_nav = 0;
            }
            1 => self.focus_main(window, cx),
            _ => window.focus(&self.status_focus, cx),
        }
        cx.notify();
    }

    fn choose_theme(
        &mut self,
        preference: Preference,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.preference = preference;
        theme::apply(preference, window, cx);
        cx.notify();
    }

    fn settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut choices = div().flex().gap(px(theme::CONTROL_GAP));
        for (id, preference, en, zh) in [
            ("theme-system", Preference::System, "System", "跟随系统"),
            ("theme-light", Preference::Light, "Light", "浅色"),
            ("theme-dark", Preference::Dark, "Dark", "深色"),
        ] {
            choices = choices.child(semantics::button(
                Button::new(id)
                    .label(text(cx, en, zh))
                    .ghost()
                    .selected(self.preference == preference)
                    .toggled(self.preference == preference)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.choose_theme(preference, window, cx)
                    })),
                false,
                false,
                cx,
            ));
        }
        let language = cx.global::<Language>();
        div().flex().flex_col().p(px(theme::CONTENT_PADDING)).gap(px(theme::SECTION_GAP))
            .child(div().text_size(px(theme::TITLE_SIZE)).font_weight(gpui_kit::FontWeight::SEMIBOLD).child(text(cx,"Appearance and language","外观与语言")))
            .child(text(cx,"Theme","主题")).child(choices)
            .child(text(cx,"Language","语言"))
            .child(div().flex().gap(px(theme::CONTROL_GAP))
                .child(semantics::button(Button::new("language-en").label("English").ghost().selected(*language==Language::English).toggled(*language==Language::English).on_click(cx.listener(|_,_,_,cx|{cx.set_global(Language::English);cx.refresh_windows();})),false,false,cx))
                .child(semantics::button(Button::new("language-zh").label("中文").ghost().selected(*language==Language::Chinese).toggled(*language==Language::Chinese).on_click(cx.listener(|_,_,_,cx|{cx.set_global(Language::Chinese);cx.refresh_windows();})),false,false,cx)))
            .child(div().text_color(cx.theme().muted_foreground).child(text(cx,"Preferences apply to this window session. Keyboard actions are immediate; decorative motion is disabled.","设置在当前窗口会话内生效。键盘操作即时响应，装饰动画已关闭。")))
    }

    fn nav_button(
        &self,
        id: &'static str,
        destination: Destination,
        en: &'static str,
        zh: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        semantics::button(
            Button::new(id)
                .label(text(cx, en, zh))
                .ghost()
                .selected(self.destination == destination)
                .toggled(self.destination == destination)
                .w_full()
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.go(destination, Lens::Primary, window, cx)
                })),
            false,
            false,
            cx,
        )
    }

    fn lens_button(
        &self,
        id: &'static str,
        lens: Lens,
        en: &'static str,
        zh: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let destination = self.destination;
        semantics::button(
            Button::new(id)
                .label(text(cx, en, zh))
                .ghost()
                .selected(self.lens == lens)
                .toggled(self.lens == lens)
                .on_click(
                    cx.listener(move |this, _, window, cx| this.go(destination, lens, window, cx)),
                ),
            false,
            false,
            cx,
        )
    }

    fn section_title(
        &self,
        en: &'static str,
        zh: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .pt(px(theme::CONTROL_GAP))
            .text_size(px(theme::META_SIZE))
            .text_color(cx.theme().muted_foreground)
            .child(text(cx, en, zh))
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let items: Vec<(&str, Lens, &str, &str)> = match self.destination {
            Destination::ClaudeCode | Destination::Codex => vec![
                ("tool-profiles", Lens::Primary, "Profiles", "配置列表"),
                (
                    "tool-accounts",
                    Lens::Accounts,
                    "Settings and accounts",
                    "设置与账户",
                ),
                ("tool-files", Lens::Files, "Instruction files", "指令文件"),
            ],
            Destination::Grok => vec![
                ("tool-profiles", Lens::Primary, "Profiles", "配置列表"),
                (
                    "tool-accounts",
                    Lens::Accounts,
                    "Settings and accounts",
                    "设置与账户",
                ),
            ],
            Destination::OpenCode => vec![
                ("tool-platform", Lens::Primary, "OpenCode", "OpenCode"),
                ("tool-files", Lens::Files, "Instruction files", "指令文件"),
            ],
            Destination::Antigravity => vec![
                ("tool-platform", Lens::Primary, "Antigravity", "Antigravity"),
                ("tool-files", Lens::Files, "Instruction files", "指令文件"),
            ],
            Destination::Commands => vec![
                ("tool-catalog", Lens::Primary, "Command catalog", "命令目录"),
                (
                    "tool-favorites",
                    Lens::Favorites,
                    "Favorite commands",
                    "收藏命令",
                ),
                ("tool-recent", Lens::Recent, "Recent commands", "最近命令"),
            ],
            Destination::Usage => vec![
                ("tool-report", Lens::Primary, "Usage", "用量"),
                (
                    "tool-install",
                    Lens::Install,
                    "Install llmusage",
                    "安装 llmusage",
                ),
            ],
            Destination::McpManager => vec![
                ("tool-mcp", Lens::Primary, "MCP Manager", "MCP 管理"),
                (
                    "tool-converter",
                    Lens::Converter,
                    "Configuration converter",
                    "配置转换",
                ),
            ],
            Destination::Dashboard
            | Destination::AgentSessions
            | Destination::Checkin
            | Destination::Sync
            | Destination::Settings => return None,
        };
        let mut row = div()
            .flex()
            .flex_wrap()
            .gap(px(theme::CONTROL_GAP))
            .px(px(theme::CONTENT_PADDING))
            .pt(px(theme::CONTROL_GAP));
        for (id, lens, en, zh) in items {
            row = row.child(self.lens_button(id, lens, en, zh, cx));
        }
        Some(row.into_any_element())
    }

    fn body(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.lens {
            Lens::Converter => self.converter.clone().into_any_element(),
            Lens::Favorites | Lens::Recent => self.commands.clone().into_any_element(),
            Lens::Install => self.install.clone().into_any_element(),
            Lens::Files => self.resources.clone().into_any_element(),
            Lens::Accounts => self.platforms.clone().into_any_element(),
            Lens::Primary => match self.destination {
                Destination::Dashboard => self.overview.clone().into_any_element(),
                Destination::AgentSessions | Destination::Usage => {
                    self.insights.clone().into_any_element()
                }
                Destination::ClaudeCode | Destination::Codex | Destination::Grok => {
                    self.configurations.clone().into_any_element()
                }
                Destination::OpenCode | Destination::Antigravity => {
                    self.platforms.clone().into_any_element()
                }
                Destination::Commands => self.jobs.clone().into_any_element(),
                Destination::Settings => self.settings(cx).into_any_element(),
                Destination::McpManager => self.mcp.clone().into_any_element(),
                Destination::Checkin => self.checkin.clone().into_any_element(),
                Destination::Sync => self.sync.clone().into_any_element(),
            },
        }
    }
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.overview.read(cx).has_pending() {
            let shell = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = shell.update(cx, |this, cx| {
                    let Some(link) = this.overview.update(cx, |view, _| view.take_pending()) else {
                        return;
                    };
                    let (destination, lens) = match link {
                        OverviewLink::Usage => (Destination::Usage, Lens::Primary),
                        OverviewLink::Commands => (Destination::Commands, Lens::Primary),
                        OverviewLink::Install => (Destination::Usage, Lens::Install),
                        OverviewLink::Sessions => (Destination::AgentSessions, Lens::Primary),
                    };
                    this.go(destination, lens, window, cx);
                });
            });
        }
        let mode = cx.theme().mode;
        let sheet = theme::solid_sheet(mode, window.viewport_size());
        let narrow = window.viewport_size().width < px(theme::NARROW_WIDTH);
        let sidebar_width = if narrow {
            theme::NARROW_SIDEBAR
        } else {
            theme::SIDEBAR_WIDTH
        };
        let mut navigation = div()
            .id("shell-navigation")
            .role(gpui_kit::Role::Navigation)
            .aria_label(text(cx, "Navigation", "导航"))
            .track_focus(&self.navigation_focus)
            .tab_stop(false)
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    let count = Self::nav_items().len();
                    match event.keystroke.key.as_str() {
                        "down" => {
                            this.focused_nav = (this.focused_nav + 1) % count;
                            window.focus_next(cx);
                            if !this.navigation_focus.contains_focused(window, cx) {
                                window.focus(&this.navigation_focus, cx);
                                window.focus_next(cx);
                                this.focused_nav = 0;
                            }
                        }
                        "up" => {
                            this.focused_nav = (this.focused_nav + count - 1) % count;
                            window.focus_prev(cx);
                            if !this.navigation_focus.contains_focused(window, cx) {
                                window.focus(&this.main_focus, cx);
                                window.focus_prev(cx);
                            }
                        }
                        "enter" => {
                            let destination = Self::nav_items()[this.focused_nav].1;
                            this.go(destination, Lens::Primary, window, cx);
                        }
                        _ => return,
                    }
                    cx.stop_propagation();
                }),
            )
            .w(px(sidebar_width))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .p(px(theme::SECTION_GAP))
            .gap(px(theme::CONTROL_GAP))
            .bg(cx.theme().sidebar)
            .rounded_tl(px(theme::SHEET_CONTENT_RADIUS))
            .child(
                div()
                    .h(px(theme::TOOLBAR_HEIGHT))
                    .flex()
                    .flex_col()
                    .justify_center()
                    .child(
                        div()
                            .text_size(px(16.))
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .child("CCR"),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .text_size(px(theme::META_SIZE))
                            .child(if self.fixture {
                                text(cx, "Demo data", "演示数据")
                            } else {
                                text(cx, "Workbench", "运行工作台")
                            }),
                    ),
            );
        for (index, (id, destination, en, zh)) in Self::nav_items().into_iter().enumerate() {
            if index == 1 {
                navigation = navigation.child(self.section_title("Configuration", "配置中心", cx));
            } else if index == 3 {
                navigation = navigation.child(self.section_title("Platforms", "平台", cx));
            } else if index == 8 {
                navigation = navigation.child(self.section_title("Tools", "工具", cx));
            } else if index == 12 {
                navigation = navigation.child(div().flex_1());
            }
            navigation = navigation.child(self.nav_button(id, destination, en, zh, cx));
        }
        let tools = self.toolbar(cx);
        let body = self.body(cx);
        let mut page = div()
            .id(self.destination.page_id())
            .test_support()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col();
        if let Some(tools) = tools {
            page = page.child(tools);
        }
        let content = page.child(div().flex_1().min_h_0().child(body));
        let status = if self.slow_fixture {
            text(
                cx,
                "Local · Demo data · Controlled slow read",
                "本地 · 演示数据 · 受控慢读取",
            )
        } else if self.fixture {
            text(cx, "Local · Demo data", "本地 · 演示数据")
        } else if self.overview.read(cx).requests_configured() {
            text(
                cx,
                "Local · Usage database connected",
                "本地 · 已连接用量库",
            )
        } else {
            text(
                cx,
                "Local · Usage database not ready",
                "本地 · 用量库未就绪",
            )
        };
        let workspace = div()
            .id("application-sheet")
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(sheet.background)
            .rounded(px(theme::SHEET_RADIUS))
            .border_1()
            .border_color(cx.theme().border)
            .shadow(vec![sheet.shadow])
            .text_color(cx.theme().foreground)
            .font_family(cx.theme().font_family.clone())
            .text_size(px(theme::BODY_SIZE))
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "f6" {
                        this.region(event.keystroke.modifiers.shift, window, cx);
                        cx.stop_propagation();
                    }
                }),
            )
            .child(
                div().flex().flex_1().min_h_0().child(navigation).child(
                    div()
                        .id("shell-main")
                        .role(gpui_kit::Role::Main)
                        .aria_label(text(cx, "Workspace", "工作区"))
                        .track_focus(&self.main_focus)
                        .tab_stop(false)
                        .flex_1()
                        .min_w_0()
                        .min_h_0()
                        .child(content),
                ),
            )
            .child(
                div()
                    .id("shell-status")
                    .role(gpui_kit::Role::Status)
                    .aria_label(status)
                    .track_focus(&self.status_focus)
                    .tab_stop(true)
                    .h(px(theme::STATUS_HEIGHT))
                    .flex_shrink_0()
                    .px(px(theme::SECTION_GAP))
                    .flex()
                    .items_center()
                    .border_t_1()
                    .border_color(cx.theme().status_bar_border)
                    .focus(|style| style.border_color(cx.theme().ring))
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(status),
            );
        div()
            .id("ccr-shell")
            .size_full()
            .p(sheet.inset)
            .bg(theme::field(mode))
            .child(workspace)
    }
}
