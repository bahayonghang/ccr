use super::{Shell, close, semantics};
use crate::{
    locales::{Language, text},
    runtime::preferences::{DesktopPreferences, PreferencesError},
    theme::{self, Preference},
};
use ccr_app::runtime::{Completion, TaskEvent, TaskSubscription};
use gpui_kit::component::{
    ActiveTheme, Selectable,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    AppContext, Context, InteractiveElement, IntoElement, ParentElement, Styled, Window, div, px,
};

impl Shell {
    pub(super) fn load_preferences(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.preferences_busy || self.preferences_closing {
            return;
        }
        match self.preferences_requests.load() {
            Ok(subscription) => self.watch_preferences(subscription, true, window, cx),
            Err(_) => {
                self.preferences_failed = true;
                self.reveal_preferences_after_save = false;
                cx.notify();
            }
        }
    }

    fn save_preferences(
        &mut self,
        value: DesktopPreferences,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.preferences_loaded || self.preferences_closing {
            return;
        }
        self.apply_preferences(value, window, cx);
        self.persist_preferences(value, window, cx);
    }

    pub(super) fn persist_preferences(
        &mut self,
        value: DesktopPreferences,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        self.preferences_generation = self.preferences_generation.saturating_add(1);
        match self
            .preferences_requests
            .save(value, self.preferences_generation)
        {
            Ok(subscription) => self.watch_preferences(subscription, false, window, cx),
            Err(_) => {
                self.preferences_failed = true;
                self.reveal_preferences_after_save = false;
                cx.notify();
            }
        }
    }

    pub(super) fn accept_loaded_preferences(
        &mut self,
        result: Result<DesktopPreferences, PreferencesError>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(value) => {
                self.preferences_loaded = true;
                self.preferences_failed = false;
                self.window_placement =
                    Some(super::window_state::placement(window.window_bounds()));
                self.apply_preferences(value, window, cx);
            }
            Err(_) => {
                self.preferences_failed = true;
                self.reveal_preferences_after_save = false;
            }
        }
        cx.notify();
    }

    fn apply_preferences(
        &mut self,
        value: DesktopPreferences,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.preference = value.theme;
        self.close_guard.confirm.set(value.confirm_quit);
        cx.set_global(value.language);
        theme::apply(value.theme, window, cx);
        // Menu labels follow the same language owner as the window.
        cx.set_menus(vec![gpui_kit::Menu::new("CCR").items([
            gpui_kit::MenuItem::action(text(cx, "Quit CCR", "退出 CCR"), close::Quit),
        ])]);
        cx.refresh_windows();
    }

    fn watch_preferences(
        &mut self,
        mut subscription: TaskSubscription<DesktopPreferences, PreferencesError>,
        loading: bool,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        self.preferences_busy = true;
        self.preferences_failed = false;
        let handle = window.window_handle();
        self.preferences_task = Some(cx.spawn(async move |weak, cx| {
            while let Some(event) = subscription.next().await {
                if let TaskEvent::Terminal(result) = event {
                    let _ = cx.update_window(handle, |_, window, cx| {
                        let _ = weak.update(cx, |this, cx| {
                            this.preferences_busy = false;
                            match result.as_ref() {
                                Completion::Success(value) => {
                                    this.preferences_loaded = true;
                                    if loading {
                                        this.accept_loaded_preferences(Ok(*value), window, cx);
                                    }
                                    if this.reveal_preferences_after_save {
                                        this.reveal_preferences_after_save = false;
                                        if let Some(path) = this.preferences_requests.path() {
                                            cx.reveal_path(path);
                                        }
                                    }
                                }
                                _ => {
                                    this.preferences_failed = true;
                                    this.reveal_preferences_after_save = false;
                                }
                            }
                            cx.notify();
                        });
                    });
                    break;
                }
            }
        }));
        cx.notify();
    }

    pub(super) fn preference_snapshot(&self, cx: &Context<Self>) -> DesktopPreferences {
        DesktopPreferences {
            theme: self.preference,
            language: *cx.global::<Language>(),
            confirm_quit: self.close_guard.confirm.get(),
            window: self.window_placement,
        }
    }

    pub(super) fn choose_theme(
        &mut self,
        preference: Preference,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.save_preferences(
            DesktopPreferences {
                theme: preference,
                ..self.preference_snapshot(cx)
            },
            window,
            cx,
        );
    }

    pub(super) fn preference_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let disabled = !self.preferences_loaded || self.preferences_closing;
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
                disabled,
                false,
                cx,
            ));
        }
        let language = cx.global::<Language>();
        let mut languages = div().flex().gap(px(theme::CONTROL_GAP));
        for (id, label, value) in [
            ("language-en", "English", Language::English),
            ("language-zh", "中文", Language::Chinese),
        ] {
            languages = languages.child(semantics::button(
                Button::new(id)
                    .label(label)
                    .ghost()
                    .selected(*language == value)
                    .toggled(*language == value)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.save_preferences(
                            DesktopPreferences {
                                language: value,
                                ..this.preference_snapshot(cx)
                            },
                            window,
                            cx,
                        )
                    })),
                disabled,
                false,
                cx,
            ));
        }
        let mut close_choices = div().flex().gap(px(theme::CONTROL_GAP));
        for (id, confirm, en, zh) in [
            ("close-confirm", true, "Ask before quitting", "退出前询问"),
            ("close-direct", false, "Quit directly", "直接退出"),
        ] {
            close_choices = close_choices.child(semantics::button(
                Button::new(id)
                    .label(text(cx, en, zh))
                    .ghost()
                    .selected(self.close_guard.confirm.get() == confirm)
                    .toggled(self.close_guard.confirm.get() == confirm)
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
                disabled,
                false,
                cx,
            ));
        }
        let status = if self.preferences_failed {
            text(
                cx,
                "Preferences could not be loaded or saved. Existing files were preserved; changes may apply only to this session.",
                "无法读取或保存偏好。已有文件保持不变，更改可能仅在当前会话生效。",
            )
        } else if self.preferences_busy {
            text(cx, "Updating desktop preferences…", "正在更新桌面偏好…")
        } else {
            text(
                cx,
                "Preferences are stored separately for this desktop app.",
                "偏好独立保存在此桌面应用的专用目录中。",
            )
        };
        let mut controls = div().flex().flex_col().gap(px(theme::SECTION_GAP))
            .child(div().text_size(px(theme::TITLE_SIZE)).font_weight(gpui_kit::FontWeight::SEMIBOLD).child(text(cx,"Appearance and language","外观与语言")))
            .child(text(cx,"Theme","主题")).child(choices)
            .child(text(cx,"Language","语言")).child(languages)
            .child(text(cx,"Closing the window or quitting","关闭窗口与退出")).child(close_choices)
            .child(div().text_color(cx.theme().muted_foreground).child(text(cx,"Quitting discards unsaved changes and requests cancellation of running tasks. Direct quit does not ask for confirmation.","退出会丢弃未保存的更改并请求取消运行中的任务。直接退出不会再次确认。")))
            .child(div().id("preferences-status").text_color(cx.theme().muted_foreground).child(status));
        if self.preferences_failed {
            controls = controls.child(semantics::button(
                Button::new("preferences-retry")
                    .label(text(cx, "Reload preferences", "重新读取偏好"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| this.load_preferences(window, cx))),
                self.preferences_busy,
                false,
                cx,
            ));
        }
        controls.child(
            div()
                .flex()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("preferences-reveal")
                        .label(text(cx, "Show preferences file", "显示偏好文件"))
                        .ghost()
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
                        .ghost()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.close_guard.request(window, cx)),
                        ),
                    false,
                    false,
                    cx,
                )),
        )
    }
}
