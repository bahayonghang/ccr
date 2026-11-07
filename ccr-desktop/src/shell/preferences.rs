use super::{Shell, close};
use crate::{
    locales::{Language, text},
    runtime::preferences::{DesktopPreferences, PreferencesError},
    theme::{self, Preference},
};
use ccr_app::runtime::{Completion, TaskEvent, TaskSubscription};
use gpui_kit::{AppContext, Context, Window};

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

    pub(super) fn save_preferences(
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
}
