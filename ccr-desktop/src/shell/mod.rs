//! Native capability shell: one workbench destination per sidebar entry.
pub mod badge;
mod close;
#[cfg(test)]
mod close_tests;
mod command_state;
mod navigation_guard;
pub(crate) mod page_header;
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
    ActiveTheme, IconNamed,
    button::{Button, ButtonVariants},
};
use gpui_kit::prelude::FluentBuilder;
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

/// One lens option: test id, lens, English and Chinese labels.
type LensItem = (&'static str, Lens, &'static str, &'static str);

/// In-page capability. These are not sidebar entries.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lens {
    Primary,
    Profiles,
    Auth,
    Mcp,
    Usage,
    Sessions,
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
    typography: crate::runtime::preferences::TypographyPreferences,
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
    /// 嵌入式目的地放进自己标题行的 lens 胶囊（外壳唯一拥有 lens 状态）。
    lens_bar: Entity<page_header::LensBar>,
    navigation_focus: FocusHandle,
    main_focus: FocusHandle,
    navigation_decision: Option<navigation_guard::Decision>,
    navigation_generation: u64,
    leave_focus: FocusHandle,
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
                theme::apply_typography(&this.typography, cx);
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
        let weak_shell = cx.weak_entity();
        let lens_bar = cx.new(|cx| page_header::LensBar::new(weak_shell, cx));
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
            settings_nav: settings::Navigation::new(window, cx),
            preference: Preference::Light,
            typography: Default::default(),
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
            lens_bar,
            navigation_decision: None,
            navigation_generation: 0,
            leave_focus: cx.focus_handle().tab_stop(false),
            navigation_focus: cx.focus_handle().tab_stop(false),
            main_focus: cx.focus_handle().tab_stop(false),
            status_focus: cx.focus_handle(),
            _appearance: appearance,
            _activation: activation,
        };
        let lens: gpui_kit::AnyView = shell.lens_bar.clone().into();
        shell
            .overview
            .update(cx, |view, _| view.embed_lens(lens.clone()));
        shell
            .monitoring
            .update(cx, |view, _| view.embed_lens(lens.clone()));
        shell
            .mcp
            .update(cx, |view, _| view.embed_lens(lens.clone()));
        shell.converter.update(cx, |view, _| view.embed_lens(lens));
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

    #[cfg(test)]
    pub(crate) fn insights_view(&self) -> Entity<crate::features::insights::InsightsView> {
        self.insights.clone()
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
        self.navigation_generation += 1;
        if self.lens == Lens::Profiles && self.configurations.read(cx).has_dirty_draft(cx) {
            self.navigation_decision = Some(navigation_guard::Decision {
                destination,
                lens,
                generation: self.navigation_generation,
                enter: true,
                return_focus: window.focused(cx),
            });
            cx.notify();
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
            Lens::Usage | Lens::Sessions => {
                if let Some(platform) = self.platform() {
                    let page = if self.lens == Lens::Sessions {
                        InsightsPage::Sessions
                    } else {
                        InsightsPage::Usage
                    };
                    self.insights.update(cx, |view, cx| {
                        view.open_platform(page, platform, window, cx)
                    });
                }
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
        match self.destination {
            // 平台目的地的首个 tab 一律是工作台。
            Destination::ClaudeCode
            | Destination::Codex
            | Destination::Grok
            | Destination::OpenCode
            | Destination::Antigravity => {
                if let Some(platform) = self.platform() {
                    self.workbench
                        .update(cx, |view, cx| view.open(platform, cx));
                }
            }
            Destination::Dashboard => self.overview.update(cx, |view, cx| view.refresh(cx)),
            Destination::AgentSessions => self.insights.update(cx, |view, cx| {
                view.open_global(InsightsPage::Sessions, window, cx)
            }),
            Destination::Usage => self.insights.update(cx, |view, cx| {
                view.open_global(InsightsPage::Usage, window, cx)
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
            Lens::Usage | Lens::Sessions => self
                .insights
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
        let nav = theme::nav_colors(colors.mode);
        // 平台图标默认中性，hover / 选中时显示平台身份色；非平台图标选中时为主色。
        let identity = destination
            .platform()
            .map(|p| theme::platform_color(p.short_name(), colors.mode));
        let icon_ink = match (selected, identity) {
            (true, Some(identity)) => identity,
            (true, None) => colors.primary,
            (false, _) => colors.muted_foreground,
        };
        let icon_hover = identity.unwrap_or(if selected {
            colors.primary
        } else {
            colors.sidebar_foreground
        });
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
                    .text_size(crate::theme::text_size(theme::META_SIZE))
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
            .group(id)
            .w_full()
            .flex_shrink_0()
            .h(theme::text_size(if settings {
                50.
            } else {
                theme::NAV_ROW_HEIGHT
            }))
            .px(px(theme::CONTROL_GAP))
            .rounded(px(theme::PILL_RADIUS))
            // 2px 透明边框只为键盘焦点环预留位置，选中态不画边框。
            .border_2()
            .border_color(gpui_kit::transparent_black())
            .bg(if selected {
                nav.selected
            } else {
                gpui_kit::transparent_black()
            })
            .when(selected, |el| el.shadow(vec![nav.rim.clone()]))
            .text_color(if selected {
                colors.primary
            } else {
                colors.sidebar_foreground
            })
            .font_weight(if selected {
                gpui_kit::FontWeight::SEMIBOLD
            } else {
                gpui_kit::FontWeight::NORMAL
            })
            .hover(|style| style.bg(if selected { nav.selected } else { nav.hover }))
            .active(|style| style.bg(nav.pressed))
            .focus_visible(|style| style.border_color(colors.ring))
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_start()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        gpui_kit::svg()
                            .path(destination.icon().path())
                            .flex_shrink_0()
                            .size(px(theme::NAV_ICON_SIZE))
                            .text_color(icon_ink)
                            .group_hover(id, |style| style.text_color(icon_hover)),
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
        // 与上一组间距 12px（8px + 列间距 4px），标题到首行 4px。
        div()
            .pt(px(theme::CONTROL_GAP))
            .px(px(theme::CONTROL_GAP))
            .line_height(gpui_kit::relative(1.))
            .flex()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .text_size(crate::theme::text_size(theme::META_SIZE))
            .font_weight(gpui_kit::FontWeight::MEDIUM)
            .text_color(cx.theme().muted_foreground)
            .child(text(cx, en, zh))
            .child(div().flex_1().h(px(1.)).bg(cx.theme().border))
    }

    /// 嵌入式目的地：页面把外壳的 [`page_header::LensBar`] 放进自己的标题行。
    /// MCP 管理的两个 lens（MCP 管理 / 配置转换）都有嵌入槽；平台目的地不在此列，
    /// 它们由页头按任务分组渲染自己的文字 tab。
    fn embeds_lens(&self) -> bool {
        matches!(
            self.destination,
            Destination::Dashboard | Destination::McpManager
        )
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.remote.read(cx).is_remote() {
            return None;
        }
        if self.platform().is_some() {
            return Some(self.lens_bar.clone().into_any_element());
        }
        if self.embeds_lens() {
            return None;
        }
        let mut capsule = page_header::capsule(cx);
        for (id, lens, en, zh) in self.lens_items(cx)? {
            capsule = capsule.child(self.lens_button(id, lens, en, zh, cx));
        }
        Some(page_header::row("lens-navigation", None, capsule, cx))
    }

    /// Lens options for the current destination: grouped platform tabs, or the
    /// capsule options of a non-platform destination.
    fn lens_items(&self, cx: &gpui_kit::App) -> Option<Vec<LensItem>> {
        let remote = self.remote.read(cx).is_remote();
        Some(match self.destination {
            // 平台目的地在远程环境下也保留分组 tab；其余目的地远程时不渲染胶囊。
            Destination::ClaudeCode
            | Destination::Codex
            | Destination::Grok
            | Destination::OpenCode
            | Destination::Antigravity => self.platform_items(),
            _ if remote => return None,
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
        })
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
            Lens::Usage | Lens::Sessions => self.insights.clone().into_any_element(),
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
        window.set_rem_size(cx.theme().font_size);
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
        let material = theme::material(mode, window.viewport_size());
        let sidebar_width = theme::sidebar_width(window.viewport_size().width);
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
            // chrome 导轨：与纸面并列（不嵌套），左侧 20px 外角，右侧亮边即分隔线。
            .bg(material.chrome)
            .rounded_l(px(theme::SHEET_RADIUS))
            .border_1()
            .border_color(material.highlight)
            .shadow(vec![material.rim.clone()])
            .child(
                div()
                    .h(px(theme::NAV_BRAND_HEIGHT))
                    .mb(px(theme::CONTROL_GAP))
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
                                    .text_size(crate::theme::text_size(16.))
                                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                    .child("CCR"),
                            )
                            .child(
                                div()
                                    .text_color(cx.theme().muted_foreground)
                                    .text_size(crate::theme::text_size(theme::META_SIZE))
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
                .text_size(crate::theme::text_size(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(format!("CCR {}", env!("CARGO_PKG_VERSION"))),
        );
        let tools = self.toolbar(cx);
        let body = if self.navigation_decision.is_some() {
            self.confirm_navigation(window, cx)
        } else {
            self.body(cx)
        };
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
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            // sheet 材质：背后只有场景；侧栏位于左侧并列，因此左边不画亮边。
            .bg(material.sheet)
            .rounded_r(px(theme::SHEET_RADIUS))
            .border_t(px(theme::SHEET_RIM))
            .border_r(px(theme::SHEET_RIM))
            .border_b(px(theme::SHEET_RIM))
            .border_color(material.highlight)
            .shadow(vec![material.rim.clone()])
            .child(
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
                    .text_size(crate::theme::text_size(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(self.status_contents(status, cx)),
            );
        let frame = div()
            .id("application-frame")
            .size_full()
            .flex()
            .rounded(px(theme::SHEET_RADIUS))
            .shadow(material.shadow.to_vec())
            .text_color(cx.theme().foreground)
            .font(theme::body_font(cx))
            .text_size(cx.theme().font_size)
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "f6" {
                        this.region(event.keystroke.modifiers.shift, window, cx);
                        cx.stop_propagation();
                    }
                }),
            )
            .child(navigation)
            .child(workspace);
        div()
            .id("ccr-shell")
            .relative()
            .size_full()
            .p(material.inset)
            .bg(material.scene)
            .children(material.glows.iter().map(scene_glow))
            .child(frame)
    }
}

/// 静态柔光斑：透明圆加同色大模糊阴影；无 id/role，不进入无障碍树，也不接收指针。
fn scene_glow(glow: &theme::Glow) -> gpui_kit::Div {
    div()
        .absolute()
        .left(glow.origin.x)
        .top(glow.origin.y)
        .size(glow.diameter)
        .rounded_full()
        .shadow(vec![glow.shadow.clone()])
}
