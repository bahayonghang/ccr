//! Status-only clock updates avoid invalidating the workbench every frame.
use super::*;

pub(super) struct Clock {
    value: String,
    _tick: gpui_kit::Task<()>,
}

impl Clock {
    pub(super) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            value: chrono::Local::now().format("%H:%M").to_string(),
            _tick: cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_secs(30))
                        .await;
                    if this
                        .update(cx, |this, cx| {
                            this.value = chrono::Local::now().format("%H:%M").to_string();
                            cx.notify();
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }),
        }
    }
}

impl Render for Clock {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("status-clock")
            .test_support()
            .aria_label(self.value.clone())
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(cx.theme().mono_font_size)
            .child(self.value.clone())
    }
}

impl Shell {
    pub(super) fn status_contents(&self, status: &str, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme();
        let menu_was_open = self.active_environment.menu;
        let state = if !self.preferences_loaded {
            colors.muted_foreground
        } else {
            match self.active_environment.state {
                super::environment::ConnectionState::Connected => colors.success,
                super::environment::ConnectionState::Connecting => colors.info,
                _ => colors.warning,
            }
        };
        let mut routes = div().flex().items_center().gap(px(theme::NAV_ROW_GAP));
        for (id, destination, en, zh) in [
            (
                "status-overview",
                Destination::Dashboard,
                "01 Overview",
                "01 概览",
            ),
            ("status-usage", Destination::Usage, "02 Usage", "02 用量"),
            (
                "status-sessions",
                Destination::AgentSessions,
                "03 Sessions",
                "03 会话",
            ),
            (
                "status-commands",
                Destination::Commands,
                "04 Commands",
                "04 命令",
            ),
        ] {
            routes = routes.child(
                gpui_kit::base::Button::new(id)
                    .accessibility_label(text(cx, en, zh))
                    .h(px(22.))
                    .px(px(4.))
                    .rounded(px(theme::SELECTION_RADIUS))
                    .border_2()
                    .border_color(gpui_kit::transparent_black())
                    .hover(|style| style.bg(colors.secondary_hover))
                    .active(|style| style.bg(colors.secondary_active))
                    .focus_visible(|style| style.border_color(colors.ring))
                    .child(text(cx, en, zh))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.go(destination, Lens::Primary, window, cx)
                    })),
            );
        }
        div()
            .w_full()
            .min_w_0()
            .flex()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(div().size(px(6.)).flex_shrink_0().rounded_full().bg(state))
            .child(
                gpui_kit::base::Button::new("environment-switcher")
                    .track_focus(&self.active_environment.trigger_focus)
                    .capture_any_mouse_down(cx.listener(move |this, _, _, _| {
                        this.active_environment.menu_press = menu_was_open
                    }))
                    .accessibility_label(status.to_owned())
                    .flex_1()
                    .min_w_0()
                    .justify_start()
                    .h(px(24.))
                    .px(px(4.))
                    .rounded(px(theme::SELECTION_RADIUS))
                    .border_2()
                    .border_color(gpui_kit::transparent_black())
                    .hover(|style| style.bg(colors.secondary_hover))
                    .focus_visible(|style| style.border_color(colors.ring))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.))
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .id("environment-current-name")
                                    .test_support()
                                    .aria_label(self.environment_name(cx))
                                    .min_w_0()
                                    .max_w(px(320.))
                                    .text_ellipsis()
                                    .child(self.environment_name(cx)),
                            )
                            .child(
                                div()
                                    .id("environment-current-state")
                                    .test_support()
                                    .aria_label(self.environment_connection_label(cx))
                                    .flex_shrink_0()
                                    .whitespace_nowrap()
                                    .child(format!("· {}", self.environment_connection_label(cx))),
                            ),
                    )
                    .on_click(
                        cx.listener(|this, _, window, cx| this.toggle_environment_menu(window, cx)),
                    ),
            )
            .child(div().h(px(14.)).border_l_1().border_color(colors.border))
            .child(routes)
            .child(div().h(px(14.)).border_l_1().border_color(colors.border))
            .child(self.clock.clone())
    }
}
