//! Native capability shell: one workbench destination per sidebar entry.
mod close;
#[cfg(test)]
mod close_tests;
mod command_state;
mod platform_navigation;
mod preferences;
pub(crate) mod semantics;
mod settings;
mod status;
pub(crate) mod window_state;
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
    ActiveTheme,
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
    Profiles,
    Auth,
    Mcp,
    Usage,
    Observer,
    Sessions,
    Agents,
    Plugins,
    SlashCommands,
    Extensions,
    Providers,
    Models,
    Accounts,
    Files,
    Converter,
    Favorites,
    Recent,
    Install,
    Monitoring,
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
    monitoring: Entity<crate::features::monitoring::MonitoringView>,
    checkin: Entity<crate::features::checkin::CheckinView>,
    sync: Entity<crate::features::sync::SyncView>,
    overview: Entity<crate::features::overview::OverviewView>,
    workbench: Entity<crate::features::workbench::WorkbenchView>,
    environments: Entity<crate::features::environments::EnvironmentView>,
    remote: Entity<crate::features::environments::remote::RemoteView>,
    _remote_observation: Subscription,
    destination: Destination,
    lens: Lens,
    settings_nav: settings::Navigation,
    preference: Preference,
    close_guard: close::CloseGuard,
    preferences_requests: crate::runtime::preferences::PreferencesRequests,
    preferences_task: Option<gpui_kit::Task<()>>,
    preferences_busy: bool,
    preferences_loaded: bool,
    preferences_failed: bool,
    preferences_generation: u64,
    preferences_closing: bool,
    window_placement: Option<crate::runtime::preferences::WindowPlacement>,
    window_state_task: Option<gpui_kit::Task<()>>,
    _window_bounds: Subscription,
    reveal_preferences_after_save: bool,
    fixture: bool,
    slow_fixture: bool,
    brand: std::sync::Arc<gpui_kit::Image>,
    clock: Entity<status::Clock>,
    navigation_focus: FocusHandle,
    main_focus: FocusHandle,
    status_focus: FocusHandle,
    _appearance: Subscription,
    _activation: Subscription,
}

impl Shell {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::build(services, None, window, cx)
    }

    pub(crate) fn new_with_preferences(
        services: Services,
        preferences: Result<
            crate::runtime::preferences::DesktopPreferences,
            crate::runtime::preferences::PreferencesError,
        >,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::build(services, Some(preferences), window, cx)
    }

    fn build(
        services: Services,
        preferences: Option<
            Result<
                crate::runtime::preferences::DesktopPreferences,
                crate::runtime::preferences::PreferencesError,
            >,
        >,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let shell = cx.weak_entity();
        let close_guard = close::install(window, cx, move |window, cx| {
            let shell = shell.clone();
            // A close action can originate inside a Shell listener. Defer the
            // final snapshot so the entity is no longer borrowed by that action.
            window.defer(cx, move |window, cx| {
                let _ = shell.update(cx, |this, cx| this.finish_close(window, cx));
            });
        });
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
        let remote = cx.new(|cx| {
            crate::features::environments::remote::RemoteView::new(services.clone(), window, cx)
        });
        let remote_observation = cx.observe(&remote, |_, _, cx| cx.notify());
        let window_bounds = cx.observe_window_bounds(window, |this, window, cx| {
            this.window_bounds_changed(window, cx);
        });
        let mut shell = Self {
            remote,
            _remote_observation: remote_observation,
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
            monitoring: cx.new(|cx| {
                crate::features::monitoring::MonitoringView::new(services.clone(), window, cx)
            }),
            checkin: cx
                .new(|cx| crate::features::checkin::CheckinView::new(services.clone(), window, cx)),
            sync: cx.new(|cx| crate::features::sync::SyncView::new(services.clone(), window, cx)),
            overview: cx
                .new(|cx| crate::features::overview::OverviewView::new(services.clone(), cx)),
            workbench: cx
                .new(|cx| crate::features::workbench::WorkbenchView::new(services.clone(), cx)),
            environments: cx.new(|cx| {
                crate::features::environments::EnvironmentView::new(services.clone(), cx)
            }),
            destination: Destination::Dashboard,
            lens: Lens::Primary,
            settings_nav: settings::Navigation::new(cx),
            preference: Preference::Light,
            close_guard,
            preferences_requests: crate::runtime::preferences::PreferencesRequests::new(&services),
            preferences_task: None,
            preferences_busy: false,
            preferences_loaded: false,
            preferences_failed: false,
            preferences_generation: 0,
            preferences_closing: false,
            window_placement: None,
            window_state_task: None,
            _window_bounds: window_bounds,
            reveal_preferences_after_save: false,
            fixture: services.is_fixture(),
            slow_fixture: services.is_slow_fixture(),
            brand: std::sync::Arc::new(gpui_kit::Image::from_bytes(
                gpui_kit::ImageFormat::Png,
                include_bytes!("../../../ccr-ui/public/icons/icon.png").to_vec(),
            )),
            clock: cx.new(status::Clock::new),
            navigation_focus: cx.focus_handle().tab_stop(false),
            main_focus: cx.focus_handle().tab_stop(false),
            status_focus: cx.focus_handle(),
            _appearance: appearance,
            _activation: activation,
        };
        match preferences {
            Some(preferences) => shell.accept_loaded_preferences(preferences, window, cx),
            None => shell.load_preferences(window, cx),
        }
        shell.overview.update(cx, |view, cx| view.refresh(cx));
        shell
    }

    #[cfg(test)]
    pub(crate) fn environment_probe(
        &self,
        cx: &Context<Self>,
    ) -> crate::features::environments::EnvironmentProbe {
        self.environments.read(cx).probe()
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
        if self.destination == destination && self.lens == lens {
            return;
        }
        if self.destination == Destination::Settings && destination != Destination::Settings {
            self.remote
                .update(cx, |view, cx| view.clear_cli_on_leave(cx));
        }
        self.destination = destination;
        self.lens = lens;
        if !self.remote.read(cx).is_remote() {
            self.park(window, cx);
        }
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
        self.monitoring.update(cx, |view, cx| view.suspend(cx));
        self.checkin.update(cx, |view, cx| view.suspend(cx));
        self.sync.update(cx, |view, cx| view.suspend(cx));
        self.overview.update(cx, |view, cx| view.suspend(cx));
        self.workbench.update(cx, |view, cx| view.suspend(cx));
        self.environments.update(cx, |view, cx| view.suspend(cx));
    }

    fn activate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.destination == Destination::Settings {
            self.remote.update(cx, |view, cx| view.refresh(cx));
        }
        if self.remote.read(cx).is_remote() {
            return;
        }
        match self.lens {
            Lens::Extensions => {
                if let Some(platform) = self.platform() {
                    self.resources
                        .update(cx, |view, cx| view.open_extensions(platform, window, cx));
                }
            }
            Lens::Providers | Lens::Models => {
                if let Some(platform) = self.platform() {
                    let surface = if self.lens == Lens::Providers {
                        Surface::Providers
                    } else {
                        Surface::Models
                    };
                    self.platforms
                        .update(cx, |view, cx| view.open(platform, surface, window, cx));
                }
            }
            Lens::Profiles => {
                if let Some(platform) = self.platform() {
                    self.configurations
                        .update(cx, |view, cx| view.open_platform(platform, window, cx));
                }
            }
            Lens::Auth => {
                if let Some(platform) = self.platform() {
                    self.platforms.update(cx, |view, cx| {
                        view.open(platform, Surface::Auth, window, cx)
                    });
                }
            }
            Lens::Mcp => {
                if let Some(platform) = self.platform() {
                    self.mcp
                        .update(cx, |view, cx| view.open_platform(platform, cx));
                }
            }
            Lens::Usage | Lens::Sessions | Lens::Observer => {
                if let Some(platform) = self.platform() {
                    let page = match self.lens {
                        Lens::Sessions => InsightsPage::Sessions,
                        Lens::Observer => InsightsPage::Observer,
                        _ => InsightsPage::Usage,
                    };
                    self.insights.update(cx, |view, cx| {
                        view.open_platform(page, platform, window, cx)
                    });
                }
            }
            Lens::Agents | Lens::Plugins | Lens::SlashCommands => {
                use crate::runtime::resources::claude_files::Kind;
                let kind = match self.lens {
                    Lens::Agents => Kind::Agents,
                    Lens::Plugins => Kind::Plugins,
                    _ => Kind::SlashCommands,
                };
                self.resources
                    .update(cx, |view, cx| view.open_claude(kind, window, cx));
            }
            Lens::Converter => {}
            Lens::Favorites => self.commands.update(cx, |view, cx| view.open(false, cx)),
            Lens::Recent => self.commands.update(cx, |view, cx| view.open(true, cx)),
            Lens::Install => self.install.update(cx, |view, cx| view.refresh(cx)),
            Lens::Monitoring => self.monitoring.update(cx, |view, cx| view.refresh(cx)),
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
        if let Some(platform) = self.platform() {
            self.workbench
                .update(cx, |view, cx| view.open(platform, cx));
            return;
        }
        match self.destination {
            Destination::Dashboard => self.overview.update(cx, |view, cx| view.refresh(cx)),
            Destination::AgentSessions => self.insights.update(cx, |view, cx| {
                view.open_global(InsightsPage::Sessions, window, cx)
            }),
            Destination::Usage => self.insights.update(cx, |view, cx| {
                view.open_global(InsightsPage::Usage, window, cx)
            }),
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
            Destination::McpManager => self.mcp.update(cx, |view, cx| view.open_global(cx)),
            Destination::Checkin => self.checkin.update(cx, |view, cx| view.refresh(cx)),
            Destination::Sync => self.sync.update(cx, |view, cx| view.refresh(cx)),
            Destination::Settings => {
                self.environments.update(cx, |view, cx| view.refresh(cx));
            }
        }
    }

    fn account_target(&self) -> Option<(Platform, Surface)> {
        match self.destination {
            Destination::ClaudeCode => Some((Platform::Claude, Surface::Settings)),
            Destination::Codex => Some((Platform::Codex, Surface::Settings)),
            Destination::Grok => Some((Platform::Grok, Surface::Settings)),
            Destination::OpenCode => Some((Platform::OpenCode, Surface::Settings)),
            Destination::Antigravity => Some((Platform::Gemini, Surface::LegacySettings)),
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
        if self.remote.read(cx).is_remote() {
            window.focus(&self.main_focus, cx);
            window.focus_next(cx);
            return;
        }
        match self.lens {
            Lens::Extensions => self
                .resources
                .update(cx, |view, cx| view.focus_main(window, cx)),
            Lens::Providers | Lens::Models => self
                .platforms
                .update(cx, |view, cx| view.focus_main(window, cx)),
            Lens::Profiles => self
                .configurations
                .update(cx, |view, cx| view.focus_main(window, cx)),
            Lens::Auth => self
                .platforms
                .update(cx, |view, cx| view.focus_main(window, cx)),
            Lens::Mcp => self.mcp.update(cx, |view, cx| view.focus_main(window, cx)),
            Lens::Usage | Lens::Sessions | Lens::Observer => self
                .insights
                .update(cx, |view, cx| view.focus_main(window, cx)),
            Lens::Agents | Lens::Plugins | Lens::SlashCommands => self
                .resources
                .update(cx, |view, cx| view.focus_main(window, cx)),
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
            Lens::Monitoring => self
                .monitoring
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
                Destination::ClaudeCode
                | Destination::Codex
                | Destination::Grok
                | Destination::OpenCode
                | Destination::Antigravity => self
                    .workbench
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
            }
            1 => self.focus_main(window, cx),
            _ => window.focus(&self.status_focus, cx),
        }
        cx.notify();
    }

    fn nav_button(
        &self,
        id: &'static str,
        destination: Destination,
        en: &'static str,
        zh: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.destination == destination;
        let colors = cx.theme();
        let settings = destination == Destination::Settings;
        let selection = colors.sidebar.blend(colors.primary.opacity(0.14));
        let hover =
            colors
                .sidebar
                .blend(colors.primary.opacity(if selected { 0.18 } else { 0.07 }));
        let pressed = colors.sidebar.blend(colors.primary.opacity(0.22));
        let ink = destination
            .platform()
            .map(|p| theme::platform_color(p.short_name(), colors.mode))
            .unwrap_or(colors.sidebar_foreground);
        let mut copy = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(3.))
            .child(div().text_ellipsis().child(text(cx, en, zh)));
        if settings {
            let mode = if colors.mode.is_dark() {
                text(cx, "Dark", "深色")
            } else {
                text(cx, "Light", "浅色")
            };
            let summary = format!("{mode} · {}", text(cx, "English", "中文"));
            copy = copy.child(
                div()
                    .id("nav-settings-summary")
                    .test_support()
                    .aria_label(summary.clone())
                    .text_size(px(theme::META_SIZE))
                    .font_weight(gpui_kit::FontWeight::NORMAL)
                    .text_color(colors.muted_foreground)
                    .child(summary),
            );
        }
        gpui_kit::base::Button::new(id)
            .accessibility_label(text(cx, en, zh))
            .aria_toggled(if selected {
                gpui_kit::accesskit::Toggled::True
            } else {
                gpui_kit::accesskit::Toggled::False
            })
            .w_full()
            .flex_shrink_0()
            .h(px(if settings { 50. } else { theme::NAV_ROW_HEIGHT }))
            .px(px(theme::CONTROL_GAP))
            .rounded(px(theme::INPUT_RADIUS))
            .border_2()
            .border_color(if selected {
                colors.primary.opacity(0.24)
            } else {
                gpui_kit::transparent_black()
            })
            .bg(if selected {
                selection
            } else {
                gpui_kit::transparent_black()
            })
            .font_weight(if selected {
                gpui_kit::FontWeight::SEMIBOLD
            } else {
                gpui_kit::FontWeight::NORMAL
            })
            .hover(|style| style.bg(hover))
            .active(|style| style.bg(pressed))
            .focus_visible(|style| style.border_color(colors.ring))
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_start()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        gpui_kit::component::Icon::new(destination.icon())
                            .size(px(theme::NAV_ICON_SIZE))
                            .text_color(ink),
                    )
                    .child(copy),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.go(destination, Lens::Primary, window, cx)
            }))
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
        semantics::tab(
            semantics::button(
                Button::new(id).label(text(cx, en, zh)).ghost().on_click(
                    cx.listener(move |this, _, window, cx| this.go(destination, lens, window, cx)),
                ),
                false,
                false,
                cx,
            ),
            self.lens == lens,
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
            .pt(px(theme::NAV_PADDING))
            .px(px(theme::CONTROL_GAP))
            .line_height(gpui_kit::relative(1.))
            .flex()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .text_size(px(theme::META_SIZE))
            .font_weight(gpui_kit::FontWeight::MEDIUM)
            .text_color(cx.theme().muted_foreground)
            .child(text(cx, en, zh))
            .child(div().flex_1().h(px(1.)).bg(cx.theme().border))
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.remote.read(cx).is_remote() {
            return None;
        }
        if self.platform().is_some() {
            return Some(self.platform_toolbar(cx));
        }
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
            Destination::Dashboard => vec![
                ("tool-overview", Lens::Primary, "Overview", "运行概览"),
                (
                    "tool-monitoring",
                    Lens::Monitoring,
                    "Monitoring",
                    "全局监控",
                ),
            ],
            Destination::AgentSessions
            | Destination::Checkin
            | Destination::Sync
            | Destination::Settings => return None,
        };
        let mut row = div()
            .flex()
            .flex_wrap()
            .gap(px(2.))
            .p(px(2.))
            .rounded(px(theme::INPUT_RADIUS))
            .border_1()
            .border_color(theme::tab_colors(cx.theme().mode).border)
            .bg(theme::tab_colors(cx.theme().mode).background);
        for (id, lens, en, zh) in items {
            row = row.child(self.lens_button(id, lens, en, zh, cx));
        }
        Some(
            div()
                .flex_shrink_0()
                .px(px(theme::CONTENT_PADDING))
                .py(px(theme::CONTROL_GAP))
                .border_b_1()
                .border_color(cx.theme().border)
                .child(row)
                .into_any_element(),
        )
    }

    fn body(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.remote.read(cx).is_remote() && self.destination != Destination::Settings {
            return div().p(px(theme::CONTENT_PADDING)).flex().flex_col().gap(px(theme::CONTROL_GAP))
                .child(self.remote.read(cx).status())
                .child(text(cx,"This page uses local CCR services and is unavailable in SSH environments. Open Settings to edit remote configuration over SFTP or return to Local.","此页面依赖本地 CCR 服务，在 SSH 环境不可用。请打开设置，通过 SFTP 编辑远程配置，或返回本地环境。"))
                .child(semantics::button(Button::new("remote-open-settings").label(text(cx,"Open environment settings","打开环境设置")).ghost().on_click(cx.listener(|this,_,window,cx|this.open_environment_settings(window,cx))),false,false,cx)).into_any_element();
        }

        match self.lens {
            Lens::Extensions => self.resources.clone().into_any_element(),
            Lens::Providers | Lens::Models => self.platforms.clone().into_any_element(),
            Lens::Profiles => self.configurations.clone().into_any_element(),
            Lens::Auth => self.platforms.clone().into_any_element(),
            Lens::Mcp => self.mcp.clone().into_any_element(),
            Lens::Usage | Lens::Observer | Lens::Sessions => {
                self.insights.clone().into_any_element()
            }
            Lens::Agents | Lens::Plugins | Lens::SlashCommands => {
                self.resources.clone().into_any_element()
            }
            Lens::Converter => self.converter.clone().into_any_element(),
            Lens::Favorites | Lens::Recent => self.commands.clone().into_any_element(),
            Lens::Install => self.install.clone().into_any_element(),
            Lens::Monitoring => self.monitoring.clone().into_any_element(),
            Lens::Files => self.resources.clone().into_any_element(),
            Lens::Accounts => self.platforms.clone().into_any_element(),
            Lens::Primary => match self.destination {
                Destination::Dashboard => self.overview.clone().into_any_element(),
                Destination::AgentSessions | Destination::Usage => {
                    self.insights.clone().into_any_element()
                }
                Destination::ClaudeCode
                | Destination::Codex
                | Destination::Grok
                | Destination::OpenCode
                | Destination::Antigravity => self.workbench.clone().into_any_element(),
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
        if self.workbench.read(cx).has_pending() {
            let shell = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = shell.update(cx, |this, cx| {
                    if let Some(link) = this.workbench.update(cx, |view, _| view.take_pending()) {
                        let lens = platform_navigation::link_lens(link);
                        let open_daily =
                            lens == Lens::Usage && this.destination == Destination::ClaudeCode;
                        if open_daily {
                            this.insights
                                .update(cx, |view, _| view.select_daily_report());
                        }
                        this.go(this.destination, lens, window, cx);
                        if open_daily {
                            this.insights
                                .update(cx, |view, cx| view.focus_main(window, cx));
                        }
                    }
                });
            });
        }
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
            .test_support()
            .role(gpui_kit::Role::Navigation)
            .aria_label(text(cx, "Navigation", "导航"))
            .track_focus(&self.navigation_focus)
            .tab_stop(false)
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    match event.keystroke.key.as_str() {
                        "down" => {
                            window.focus_next(cx);
                            if !this.navigation_focus.contains_focused(window, cx) {
                                window.focus(&this.navigation_focus, cx);
                                window.focus_next(cx);
                            }
                        }
                        "up" => {
                            window.focus_prev(cx);
                            if !this.navigation_focus.contains_focused(window, cx) {
                                window.focus(&this.main_focus, cx);
                                window.focus_prev(cx);
                            }
                        }
                        _ => return,
                    }
                    cx.stop_propagation();
                }),
            )
            .w(px(sidebar_width))
            .flex_shrink_0()
            .h_full()
            .overflow_hidden()
            .flex()
            .flex_col()
            .p(px(theme::NAV_PADDING))
            .gap(px(theme::NAV_ROW_GAP))
            .bg(cx.theme().sidebar)
            .rounded_tl(px(theme::SHEET_CONTENT_RADIUS))
            .child(
                div()
                    .h(px(theme::NAV_BRAND_HEIGHT))
                    .line_height(gpui_kit::relative(1.3))
                    .flex()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        div()
                            .id("brand-mark")
                            .test_support()
                            .role(gpui_kit::Role::Image)
                            .aria_label("CCR")
                            .size(px(28.))
                            .flex_shrink_0()
                            .child(gpui_kit::img(self.brand.clone()).size_full()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
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
                    ),
            );
        let items = Self::nav_items();
        let (primary, pinned) = items.split_at(12);
        let mut routes = div()
            .id("shell-nav-routes")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(theme::NAV_ROW_GAP));
        for (index, (id, destination, en, zh)) in primary.iter().copied().enumerate() {
            if index == 1 {
                routes = routes.child(self.section_title("Configuration", "配置中心", cx));
            } else if index == 3 {
                routes = routes.child(self.section_title("Platforms", "平台", cx));
            } else if index == 8 {
                routes = routes.child(self.section_title("Tools", "工具", cx));
            }
            routes = routes.child(self.nav_button(id, destination, en, zh, cx));
        }
        navigation = navigation.child(routes);
        for (id, destination, en, zh) in pinned.iter().copied() {
            navigation = navigation.child(
                div()
                    .flex_shrink_0()
                    .border_t_1()
                    .border_color(cx.theme().sidebar_border)
                    .pt(px(theme::NAV_ROW_GAP))
                    .child(self.nav_button(id, destination, en, zh, cx)),
            );
        }
        navigation = navigation.child(
            div()
                .flex_shrink_0()
                .px(px(theme::CONTROL_GAP))
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(format!("CCR {}", env!("CARGO_PKG_VERSION"))),
        );
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
        let remote_status = self.remote.read(cx).status();
        let status = if self.remote.read(cx).is_remote() {
            remote_status.as_str()
        } else if self.slow_fixture {
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
                "Local · Usage source configured",
                "本地 · 已配置用量来源",
            )
        } else {
            text(
                cx,
                "Local · No usage source configured",
                "本地 · 未配置用量来源",
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
            .font(theme::body_font(cx))
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
                    .test_support()
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
                    .child(self.status_contents(status, cx)),
            );
        div()
            .id("ccr-shell")
            .size_full()
            .p(sheet.inset)
            .bg(theme::field(mode))
            .child(workspace)
    }
}
