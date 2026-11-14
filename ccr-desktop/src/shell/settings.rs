use super::{Destination, Lens, Shell, semantics};
use crate::{
    locales::{Language, text},
    runtime::preferences::{DesktopPreferences, TypographyPreferences},
    theme::{self, Preference},
};
use gpui_kit::{
    App, AppContext, Context, Div, Entity, FocusHandle, FontWeight, InteractiveElement,
    IntoElement, ParentElement, ScrollHandle, SharedString, StatefulInteractiveElement, Styled,
    Window,
    accesskit::Toggled,
    assets::IconName,
    base::{Button as Control, TestSupportExt},
    component::{
        ActiveTheme, Icon, IndexPath,
        button::{Button, ButtonVariants},
        select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    },
    div,
    prelude::FluentBuilder,
    px,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Appearance,
    Typography,
    Language,
    Workbench,
    Environment,
}

impl Section {
    const ALL: [Self; 5] = [
        Self::Appearance,
        Self::Typography,
        Self::Language,
        Self::Workbench,
        Self::Environment,
    ];

    fn label(self, cx: &App) -> &'static str {
        match self {
            Self::Appearance => text(cx, "Appearance", "外观"),
            Self::Typography => text(cx, "Fonts & sizes", "字体与字号"),
            Self::Language => text(cx, "Language", "语言"),
            Self::Workbench => text(cx, "Workbench preferences", "工作台偏好"),
            Self::Environment => text(cx, "Diagnostics & environment", "诊断与环境"),
        }
    }

    fn icon(self) -> IconName {
        match self {
            Self::Appearance => IconName::Sun,
            Self::Typography => IconName::Type,
            Self::Language => IconName::Globe,
            Self::Workbench => IconName::Settings2,
            Self::Environment => IconName::Info,
        }
    }

    fn nav_id(self) -> &'static str {
        match self {
            Self::Appearance => "settings-nav-appearance",
            Self::Typography => "settings-nav-typography",
            Self::Language => "settings-nav-language",
            Self::Workbench => "settings-nav-workbench",
            Self::Environment => "settings-nav-environment",
        }
    }
}

pub(super) struct Navigation {
    active: Section,
    scroll: ScrollHandle,
    anchors: [FocusHandle; 5],
    fonts: [Entity<SelectState<SearchableVec<FontChoice>>>; 2],
    language: Language,
}

#[derive(Clone)]
struct FontChoice(Option<String>, SharedString);
impl SelectItem for FontChoice {
    type Value = Option<String>;
    fn title(&self) -> SharedString {
        self.0
            .clone()
            .map(SharedString::from)
            .unwrap_or_else(|| self.1.clone())
    }
    fn value(&self) -> &Self::Value {
        &self.0
    }
    fn render(&self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let title = self.title();
        div()
            .id(title.clone())
            .role(gpui_kit::Role::Label)
            .aria_label(title.clone())
            .child(title)
    }
}

fn font_choices(cx: &App) -> Vec<FontChoice> {
    std::iter::once(FontChoice(
        None,
        text(cx, "System default", "系统默认").into(),
    ))
    .chain(
        cx.global::<theme::FontCatalog>()
            .0
            .iter()
            .cloned()
            .map(|name| FontChoice(Some(name), SharedString::default())),
    )
    .collect()
}

impl Navigation {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Shell>) -> Self {
        let items = font_choices(cx);
        let fonts = std::array::from_fn(|index| {
            let select = cx.new(|cx| {
                SelectState::new(
                    SearchableVec::new(items.clone()),
                    Some(IndexPath::default()),
                    window,
                    cx,
                )
                .searchable(true)
            });
            cx.subscribe_in(&select, window, move |this, _, event, window, cx| {
                let SelectEvent::Confirm(Some(font)) = event else {
                    return;
                };
                let mut value = this.preference_snapshot(cx);
                if index == 0 {
                    value.typography.ui_font = font.clone();
                } else {
                    value.typography.mono_font = font.clone();
                }
                this.save_preferences(value, window, cx);
            })
            .detach();
            select
        });
        Self {
            active: Section::Appearance,
            scroll: ScrollHandle::new(),
            anchors: std::array::from_fn(|_| cx.focus_handle().tab_stop(false)),
            fonts,
            language: *cx.global::<Language>(),
        }
    }

    pub(super) fn sync_fonts(
        &mut self,
        value: &TypographyPreferences,
        window: &mut Window,
        cx: &mut Context<Shell>,
    ) {
        let language_changed = self.language != *cx.global::<Language>();
        if language_changed {
            self.language = *cx.global::<Language>();
            let items = font_choices(cx);
            for select in &self.fonts {
                select.update(cx, |state, cx| {
                    state.set_items(SearchableVec::new(items.clone()), window, cx)
                });
            }
        }
        for (select, selected) in self.fonts.iter().zip([&value.ui_font, &value.mono_font]) {
            if language_changed || select.read(cx).selected_value() != Some(selected) {
                select.update(cx, |state, cx| {
                    state.set_selected_value(selected, window, cx)
                });
            }
        }
    }
}

impl Shell {
    fn follow_settings_scroll(&self, window: &mut Window, cx: &mut Context<Self>) {
        let weak = cx.weak_entity();
        window.on_next_frame(move |_, cx| {
            let _ = weak.update(cx, |this, cx| {
                if this.destination != Destination::Settings {
                    return;
                }
                let scroll = &this.settings_nav.scroll;
                let index = if scroll.max_offset().y > px(0.)
                    && scroll.offset().y <= -scroll.max_offset().y + px(1.)
                {
                    Section::ALL.len() - 1
                } else {
                    scroll.top_item().min(Section::ALL.len() - 1)
                };
                if this.settings_nav.active != Section::ALL[index] {
                    this.settings_nav.active = Section::ALL[index];
                    cx.notify();
                }
            });
        });
    }

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
            .w(px(theme::SETTINGS_NAV_WIDTH))
            .flex_shrink_0()
            .gap(px(theme::CONTROL_GAP));
        for section in Section::ALL {
            let active = self.settings_nav.active == section;
            nav = nav.child(
                control(section.nav_id(), section.label(cx), active, cx)
                    .min_h(px(40.))
                    .justify_start()
                    .px(px(theme::SECTION_GAP))
                    .rounded(px(theme::INPUT_RADIUS))
                    .border_color(gpui_kit::transparent_black())
                    .child(Icon::new(section.icon()).size(px(16.)))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .whitespace_normal()
                            .child(section.label(cx)),
                    )
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.locate(section, window, cx)),
                    ),
            );
        }
        let mut content = div()
            .id("settings-body")
            .test_support()
            .min_h_0()
            .min_w_0()
            .flex_1()
            .overflow_y_scroll()
            .track_scroll(&self.settings_nav.scroll)
            .flex()
            .flex_col()
            .gap(px(theme::SECTION_GAP))
            .on_scroll_wheel(
                cx.listener(|this, _, window, cx| this.follow_settings_scroll(window, cx)),
            )
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if matches!(
                        event.keystroke.key.as_str(),
                        "up" | "down" | "pageup" | "pagedown" | "home" | "end"
                    ) {
                        this.follow_settings_scroll(window, cx);
                    }
                }),
            );
        for section in Section::ALL {
            let mut panel = div()
                .track_focus(&self.settings_nav.anchors[section as usize])
                .flex_shrink_0()
                .min_w_0()
                .flex()
                .flex_col()
                .p(px(theme::SECTION_GAP))
                .gap(px(theme::SECTION_GAP))
                .border_b_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .text_size(crate::theme::text_size(theme::SECTION_TITLE_SIZE))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(section.label(cx)),
                );
            panel = match section {
                Section::Appearance => panel.child(self.theme_choices(cx)),
                Section::Typography => panel.child(self.typography_controls(cx)),
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
                            .text_size(crate::theme::text_size(theme::DISPLAY_SIZE))
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
                            .text_size(crate::theme::text_size(theme::META_SIZE))
                            .text_color(cx.theme().muted_foreground)
                            .child(meta),
                    ),
            )
            .when(self.preferences_busy || self.preferences_failed, |el| {
                el.child(self.preference_status(cx))
            })
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .gap(px(theme::SECTION_GAP))
                    .child(nav)
                    .child(
                        content
                            .rounded(px(theme::PANEL_RADIUS))
                            .bg(cx.theme().background),
                    ),
            )
    }

    fn typography_controls(&self, cx: &mut Context<Self>) -> Div {
        let mut content = div().flex().flex_col().gap(px(theme::SECTION_GAP));
        for (index, id, en, zh, size, range) in [
            (
                0,
                "settings-ui-font",
                "Interface font",
                "界面字体",
                self.typography.ui_size,
                12..=20,
            ),
            (
                1,
                "settings-mono-font",
                "Code / monospace font",
                "等宽字体",
                self.typography.mono_size,
                11..=20,
            ),
        ] {
            let family = if index == 0 {
                &self.typography.ui_font
            } else {
                &self.typography.mono_font
            };
            let mut row = div()
                .flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap(px(theme::CONTROL_GAP))
                .child(div().min_w(px(140.)).child(text(cx, en, zh)))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(theme::CONTROL_GAP))
                        .child(
                            div().w(px(240.)).child(
                                Select::new(&self.settings_nav.fonts[index])
                                    .placeholder(text(cx, "System default", "系统默认"))
                                    .id(id)
                                    .accessibility_label(text(cx, en, zh))
                                    .search_placeholder(text(
                                        cx,
                                        "Search installed fonts",
                                        "搜索本机字体",
                                    ))
                                    .disabled(!self.preferences_loaded || self.preferences_closing),
                            ),
                        )
                        .child(self.font_size_control(index, size, range, cx)),
                );
            if let Some(missing) = family
                .as_ref()
                .filter(|name| !cx.global::<theme::FontCatalog>().0.contains(name))
            {
                row = row.child(
                    div()
                        .text_size(theme::text_size(theme::META_SIZE))
                        .text_color(cx.theme().warning)
                        .child(format!(
                            "{}: {missing}",
                            text(
                                cx,
                                "Font unavailable; using system default",
                                "字体不可用，已回退到系统默认"
                            )
                        )),
                );
            }
            content = content.child(row);
            let preview = if index == 0 {
                div()
                    .id("settings-ui-preview")
                    .test_support()
                    .font(theme::body_font(cx))
                    .text_size(cx.theme().font_size)
                    .child(text(
                        cx,
                        "A readable workbench · 中文 / English · 0123456789",
                        "清晰的工作台 · 中文 / English · 0123456789",
                    ))
            } else {
                div()
                    .id("settings-mono-preview")
                    .test_support()
                    .font_family(cx.theme().mono_font_family.clone())
                    .text_size(cx.theme().mono_font_size)
                    .child("ccr status --platform claude\n0123456789  il1  O0  {} [] =>")
            };
            content = content.child(
                preview
                    .p(px(theme::SECTION_GAP))
                    .rounded(px(theme::INPUT_RADIUS))
                    .bg(cx.theme().muted),
            );
        }
        content
            .child(hint(
                text(
                    cx,
                    "Fonts are read from this machine. Code font selection is independent; preview its spacing before choosing.",
                    "字体来自本机。代码字体独立选择，请通过预览确认字符对齐。",
                ),
                cx,
            ))
            .child(semantics::button(
                Button::new("settings-fonts-reset")
                    .label(text(cx, "Reset fonts and sizes", "恢复默认字体与字号"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        let mut value = this.preference_snapshot(cx);
                        value.typography = Default::default();
                        this.save_preferences(value, window, cx);
                    })),
                !self.preferences_loaded || self.preferences_closing,
                false,
                cx,
            ))
    }

    fn font_size_control(
        &self,
        index: usize,
        size: u8,
        range: std::ops::RangeInclusive<u8>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut controls = div()
            .id(if index == 0 {
                "settings-ui-size-controls"
            } else {
                "settings-mono-size-controls"
            })
            .role(gpui_kit::Role::Group)
            .aria_label(if index == 0 {
                text(cx, "Interface font size", "界面字号")
            } else {
                text(cx, "Code font size", "等宽字号")
            })
            .flex()
            .items_center()
            .gap(px(theme::CONTROL_GAP));
        for (increase, id) in [
            (
                false,
                if index == 0 {
                    "settings-ui-smaller"
                } else {
                    "settings-mono-smaller"
                },
            ),
            (
                true,
                if index == 0 {
                    "settings-ui-larger"
                } else {
                    "settings-mono-larger"
                },
            ),
        ] {
            let disabled = !self.preferences_loaded
                || self.preferences_closing
                || if increase {
                    size == *range.end()
                } else {
                    size == *range.start()
                };
            controls = controls.child(
                Control::new(id)
                    .disabled(disabled)
                    .a11y_synthetic_children(move |tree| {
                        if disabled {
                            tree.parent_node().set_disabled();
                        } else {
                            tree.parent_node().clear_disabled();
                        }
                    })
                    .accessibility_label(if increase {
                        text(cx, "Increase font size", "增大字号")
                    } else {
                        text(cx, "Decrease font size", "减小字号")
                    })
                    .w(px(36.))
                    .h(px(36.))
                    .rounded(px(theme::PILL_RADIUS))
                    .border_1()
                    .border_color(cx.theme().input)
                    .flex()
                    .items_center()
                    .justify_center()
                    .focus_visible(|style| style.border_color(cx.theme().ring))
                    .child(
                        Icon::new(if increase {
                            IconName::Plus
                        } else {
                            IconName::Minus
                        })
                        .size(px(16.)),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        let mut value = this.preference_snapshot(cx);
                        let current = if index == 0 {
                            &mut value.typography.ui_size
                        } else {
                            &mut value.typography.mono_size
                        };
                        *current = (if increase {
                            current.saturating_add(1)
                        } else {
                            current.saturating_sub(1)
                        })
                        .clamp(if index == 0 { 12 } else { 11 }, 20);
                        this.save_preferences(value, window, cx);
                    })),
            );
            if !increase {
                controls = controls.child(
                    div()
                        .id(if index == 0 {
                            "settings-ui-size"
                        } else {
                            "settings-mono-size"
                        })
                        .test_support()
                        .role(gpui_kit::Role::Label)
                        .aria_label(format!("{size} px"))
                        .aria_value(format!("{size}"))
                        .child(format!("{size} px")),
                );
            }
        }
        controls
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
            .text_size(crate::theme::text_size(theme::META_SIZE))
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
        .text_size(crate::theme::text_size(theme::META_SIZE))
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
