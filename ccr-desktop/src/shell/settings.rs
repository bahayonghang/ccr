use super::{Destination, Lens, Shell, semantics};
use crate::{
    locales::{Language, text},
    runtime::preferences::DesktopPreferences,
    theme::{self, Preference},
};
use gpui_kit::{
    App, Context, Div, FocusHandle, FontWeight, InteractiveElement, IntoElement, ParentElement,
    ScrollHandle, StatefulInteractiveElement, Styled, Window,
    accesskit::Toggled,
    assets::IconName,
    base::{Button as Control, TestSupportExt},
    component::{ActiveTheme, Icon, button::Button},
    div,
    prelude::FluentBuilder,
    px,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Appearance,
    Language,
    Workbench,
    Environment,
}

impl Section {
    const ALL: [Self; 4] = [
        Self::Appearance,
        Self::Language,
        Self::Workbench,
        Self::Environment,
    ];

    fn label(self, cx: &App) -> &'static str {
        match self {
            Self::Appearance => text(cx, "Appearance", "外观"),
            Self::Language => text(cx, "Language", "语言"),
            Self::Workbench => text(cx, "Workbench preferences", "工作台偏好"),
            Self::Environment => text(cx, "Diagnostics & environment", "诊断与环境"),
        }
    }

    fn icon(self) -> IconName {
        match self {
            Self::Appearance => IconName::Sun,
            Self::Language => IconName::Globe,
            Self::Workbench => IconName::Settings2,
            Self::Environment => IconName::Info,
        }
    }

    fn nav_id(self) -> &'static str {
        match self {
            Self::Appearance => "settings-nav-appearance",
            Self::Language => "settings-nav-language",
            Self::Workbench => "settings-nav-workbench",
            Self::Environment => "settings-nav-environment",
        }
    }
}

pub(super) struct Navigation {
    active: Section,
    scroll: ScrollHandle,
    anchors: [FocusHandle; 4],
}

impl Navigation {
    pub(super) fn new(cx: &mut Context<Shell>) -> Self {
        Self {
            active: Section::Appearance,
            scroll: ScrollHandle::new(),
            anchors: std::array::from_fn(|_| cx.focus_handle().tab_stop(false)),
        }
    }
}

impl Shell {
    fn locate(&mut self, section: Section, window: &mut Window, cx: &mut Context<Self>) {
        self.settings_nav.active = section;
        self.settings_nav
            .scroll
            .scroll_to_top_of_item(section as usize);
        window.focus(&self.settings_nav.anchors[section as usize], cx);
        cx.notify();
    }

    pub(super) fn open_environment_settings(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.go(Destination::Settings, Lens::Primary, window, cx);
        let shell = cx.weak_entity();
        window.on_next_frame(move |window, cx| {
            let _ = shell.update(cx, |this, cx| {
                if this.destination == Destination::Settings {
                    this.locate(Section::Environment, window, cx);
                }
            });
        });
    }

    pub(super) fn settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let effective = if cx.theme().mode.is_dark() {
            text(cx, "Dark", "深色")
        } else {
            text(cx, "Light", "浅色")
        };
        let mode = if self.preference == Preference::System {
            format!("{} ({effective})", text(cx, "System", "跟随系统"))
        } else {
            effective.to_string()
        };
        let meta = format!(
            "GPUI  ·  v{}  ·  {mode}  ·  {}",
            env!("CARGO_PKG_VERSION"),
            text(cx, "English", "中文")
        );
        let mut nav = div()
            .flex()
            .flex_col()
            .gap(px(1.))
            .rounded(px(theme::INPUT_RADIUS))
            .overflow_hidden()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().border);
        for pair in Section::ALL.chunks(2) {
            let mut row = div().flex().gap(px(1.));
            for &section in pair {
                let active = self.settings_nav.active == section;
                row =
                    row.child(
                        control(section.nav_id(), section.label(cx), active, cx)
                            .flex_1()
                            .h(px(40.))
                            .justify_start()
                            .px(px(theme::SECTION_GAP))
                            .rounded(px(0.))
                            .border_color(gpui_kit::transparent_black())
                            .child(Icon::new(section.icon()).size(px(16.)))
                            .child(section.label(cx))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.locate(section, window, cx)
                            })),
                    );
            }
            nav = nav.child(row);
        }
        let mut content = div()
            .id("settings-body")
            .min_h_0()
            .flex_1()
            .overflow_y_scroll()
            .track_scroll(&self.settings_nav.scroll)
            .flex()
            .flex_col()
            .gap(px(theme::SECTION_GAP));
        for section in Section::ALL {
            let mut panel = div()
                .track_focus(&self.settings_nav.anchors[section as usize])
                .flex_shrink_0()
                .flex()
                .flex_col()
                .p(px(theme::SECTION_GAP))
                .gap(px(theme::SECTION_GAP))
                .rounded(px(theme::PANEL_RADIUS))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().background)
                .child(
                    div()
                        .text_size(px(theme::SECTION_TITLE_SIZE))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(section.label(cx)),
                );
            panel = match section {
                Section::Appearance => panel.child(self.theme_choices(cx)),
                Section::Language => panel.child(self.language_choices(cx)),
                Section::Workbench => panel.child(self.workbench_controls(cx)),
                Section::Environment => panel
                    .child(self.environments.clone())
                    .child(self.remote.clone()),
            };
            content = content.child(panel);
        }
        div()
            .size_full()
            .flex()
            .flex_col()
            .p(px(theme::CONTENT_PADDING))
            .gap(px(theme::SECTION_GAP))
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .flex_wrap()
                    .items_end()
                    .justify_between()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        div()
                            .text_size(px(theme::DISPLAY_SIZE))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(text(cx, "Global settings", "全局设置")),
                    )
                    .child(
                        div()
                            .id("settings-summary")
                            .test_support()
                            .role(gpui_kit::Role::Label)
                            .aria_label(meta.clone())
                            .aria_value(meta.clone())
                            .text_size(px(theme::META_SIZE))
                            .text_color(cx.theme().muted_foreground)
                            .child(meta),
                    ),
            )
            .child(nav.flex_shrink_0())
            .when(self.preferences_busy || self.preferences_failed, |el| {
                el.child(self.preference_status(cx))
            })
            .child(content)
    }

    fn theme_choices(&self, cx: &mut Context<Self>) -> Div {
        let mut row = choice_row();
        for (id, preference, icon, en, zh) in [
            (
                "theme-light",
                Preference::Light,
                IconName::Sun,
                "Light",
                "浅色",
            ),
            (
                "theme-dark",
                Preference::Dark,
                IconName::Moon,
                "Dark",
                "深色",
            ),
            (
                "theme-system",
                Preference::System,
                IconName::Settings,
                "System",
                "跟随系统",
            ),
        ] {
            row = row.child(
                self.option(id, text(cx, en, zh), self.preference == preference, cx)
                    .child(Icon::new(icon).size(px(16.)))
                    .child(text(cx, en, zh))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.choose_theme(preference, window, cx)
                    })),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(hint(
                text(
                    cx,
                    "Follow your system appearance or choose a fixed theme.",
                    "跟随系统明暗，或固定使用浅色或深色。",
                ),
                cx,
            ))
            .child(row)
    }

    fn language_choices(&self, cx: &mut Context<Self>) -> Div {
        let mut row = choice_row();
        for (id, label, language) in [
            ("language-en", "English", Language::English),
            ("language-zh", "中文", Language::Chinese),
        ] {
            row = row.child(
                self.option(id, label, *cx.global::<Language>() == language, cx)
                    .child(label)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.save_preferences(
                            DesktopPreferences {
                                language,
                                ..this.preference_snapshot(cx)
                            },
                            window,
                            cx,
                        )
                    })),
            );
        }
        row
    }

    fn workbench_controls(&self, cx: &mut Context<Self>) -> Div {
        let mut row = choice_row();
        for (id, confirm, en, zh) in [
            ("close-confirm", true, "Ask before quitting", "退出前询问"),
            ("close-direct", false, "Quit directly", "直接退出"),
        ] {
            row = row.child(
                self.option(
                    id,
                    text(cx, en, zh),
                    self.close_guard.confirm.get() == confirm,
                    cx,
                )
                .child(text(cx, en, zh))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.save_preferences(
                        DesktopPreferences {
                            confirm_quit: confirm,
                            ..this.preference_snapshot(cx)
                        },
                        window,
                        cx,
                    )
                })),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(text(cx, "When closing the window", "关闭窗口时"))
            .child(row)
            .child(hint(
                text(
                    cx,
                    "Quitting discards unsaved changes and requests cancellation of running tasks.",
                    "退出会丢弃未保存的更改，并请求取消运行中的任务。",
                ),
                cx,
            ))
            .child(
                div()
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .mt(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("preferences-reveal")
                            .label(text(cx, "Show preferences file", "显示偏好文件"))
                            .outline()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.reveal_preferences_after_save = true;
                                this.save_preferences(this.preference_snapshot(cx), window, cx);
                            })),
                        !self.preferences_loaded || self.preferences_busy,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("app-quit")
                            .label(text(cx, "Quit CCR", "退出 CCR"))
                            .outline()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.close_guard.request(window, cx)
                            })),
                        false,
                        false,
                        cx,
                    )),
            )
    }

    fn option(
        &self,
        id: &'static str,
        label: &'static str,
        selected: bool,
        cx: &Context<Self>,
    ) -> Control {
        control(id, label, selected, cx)
            .flex_1()
            .min_w(px(120.))
            .h(px(44.))
            .disabled(!self.preferences_loaded || self.preferences_closing)
            .child(div().w(px(16.)).flex_shrink_0().when(selected, |el| {
                el.child(Icon::new(IconName::Check).size(px(16.)))
            }))
    }

    fn preference_status(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let (status, retry_label) = match (self.preferences_failed, self.preferences_loaded) {
            (true, true) => (
                text(
                    cx,
                    "Preferences could not be saved. Changes apply to this session.",
                    "无法保存偏好，更改仅在当前会话生效。",
                ),
                text(cx, "Retry save", "重试保存"),
            ),
            (true, false) => (
                text(
                    cx,
                    "Preferences could not be read. The existing file was preserved.",
                    "无法读取偏好，已有文件已保留。",
                ),
                text(cx, "Reload preferences", "重新读取偏好"),
            ),
            (false, _) => (
                text(cx, "Updating desktop preferences…", "正在更新桌面偏好…"),
                "",
            ),
        };
        div()
            .id("preferences-status")
            .test_support()
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_between()
            .role(gpui_kit::Role::Status)
            .aria_label(status)
            .text_size(px(theme::META_SIZE))
            .text_color(if self.preferences_failed {
                cx.theme().warning
            } else {
                cx.theme().muted_foreground
            })
            .child(status)
            .when(self.preferences_failed, |el| {
                el.child(semantics::button(
                    Button::new("preferences-retry")
                        .label(retry_label)
                        .outline()
                        .on_click(cx.listener(|this, _, window, cx| {
                            if this.preferences_loaded {
                                this.persist_preferences(this.preference_snapshot(cx), window, cx);
                            } else {
                                this.load_preferences(window, cx);
                            }
                        })),
                    self.preferences_busy,
                    false,
                    cx,
                ))
            })
    }
}

fn choice_row() -> Div {
    div().flex().flex_wrap().gap(px(theme::CONTROL_GAP))
}

fn hint(label: &'static str, cx: &App) -> Div {
    div()
        .text_color(cx.theme().muted_foreground)
        .text_size(px(theme::META_SIZE))
        .child(label)
}

fn control(id: &'static str, label: &'static str, selected: bool, cx: &App) -> Control {
    let colors = cx.theme();
    Control::new(id)
        .accessibility_label(label)
        .aria_toggled(if selected {
            Toggled::True
        } else {
            Toggled::False
        })
        .flex()
        .items_center()
        .gap(px(theme::CONTROL_GAP))
        .px(px(12.))
        .rounded(px(theme::SELECTION_RADIUS))
        .border_2()
        .border_color(if selected {
            colors.list_active
        } else {
            colors.border
        })
        .bg(if selected {
            colors.sidebar.blend(colors.primary.opacity(0.14))
        } else {
            colors.background
        })
        .font_weight(if selected {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        })
        .hover(|s| s.bg(colors.sidebar))
        .active(|s| s.bg(colors.list_active))
        .focus_visible(|s| s.border_color(colors.ring))
}
