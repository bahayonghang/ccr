//! Persisted command favorites and completed-command history; no execution authority.
use super::semantics;
use crate::{
    locales::text,
    runtime::{
        Services,
        ui_state::{AddFavoriteRequest, UiStateError, UiStateRequests, UiStateSnapshot},
    },
    theme,
};
use ccr_app::runtime::{
    AdmissionError, Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent,
    TaskSubscription,
};
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState, Textarea, TextareaState},
};
use gpui_kit::{
    AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Subscription, Task,
    Window, div, px, uniform_list,
};
use std::sync::Arc;

#[derive(Clone)]
enum Confirmation {
    RemoveFavorite { id: String, name: String },
    ClearHistory,
}

pub struct CommandState {
    requests: UiStateRequests,
    fixture: bool,
    history: bool,
    snapshot: Arc<UiStateSnapshot>,
    generation: u64,
    gate: ResponseGate,
    read: Option<Task<()>>,
    mutation: Option<Task<()>>,
    loading: bool,
    saving: bool,
    active: bool,
    error: bool,
    saved: bool,
    confirmation: Option<Confirmation>,
    content_focus: FocusHandle,
    confirmation_focus: FocusHandle,
    confirmation_return: Option<FocusHandle>,
    command: Entity<InputState>,
    name: Entity<InputState>,
    module: Entity<InputState>,
    args: Entity<TextareaState>,
    _command_subscription: Subscription,
}
impl CommandState {
    pub fn new(services: &Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let command = cx.new(|cx| InputState::new(window, cx));
        let subscription = cx.subscribe(&command, |_, _, event, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        });
        Self {
            requests: UiStateRequests::new(services),
            fixture: services.is_fixture(),
            history: false,
            snapshot: Arc::new(UiStateSnapshot::default()),
            generation: 0,
            gate: ResponseGate::default(),
            read: None,
            mutation: None,
            loading: false,
            saving: false,
            active: false,
            error: false,
            saved: false,
            confirmation: None,
            content_focus: cx.focus_handle().tab_stop(false),
            confirmation_focus: cx.focus_handle().tab_stop(false),
            confirmation_return: None,
            command,
            name: cx.new(|cx| InputState::new(window, cx)),
            module: cx.new(|cx| InputState::new(window, cx)),
            args: cx.new(|cx| TextareaState::new(window, cx)),
            _command_subscription: subscription,
        }
    }
    fn key(&mut self) -> RequestKey {
        self.generation += 1;
        RequestKey {
            environment: EnvironmentId::Local(
                if self.fixture { "fixture" } else { "local" }.into(),
            ),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        }
    }
    pub fn open(&mut self, history: bool, cx: &mut Context<Self>) {
        self.history = history;
        self.active = true;
        self.reload(cx);
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.active = false;
        self.confirmation = None;
        self.confirmation_return = None;
        self.read = None;
        self.gate.clear();
        self.loading = false;
        cx.notify();
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.history {
            window.focus_next(cx);
        } else {
            window.focus(&self.command.read(cx).focus_handle(cx), cx);
        }
    }
    fn reload(&mut self, cx: &mut Context<Self>) {
        self.read = None;
        let key = self.key();
        self.gate.replace(key.clone());
        self.loading = true;
        self.error = false;
        match self.requests.load(key.clone()) {
            Ok(mut read) => {
                self.read = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = read.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) {
                                    return;
                                }
                                this.loading = false;
                                match result.as_ref() {
                                    Completion::Success(snapshot) => {
                                        this.snapshot = snapshot.clone()
                                    }
                                    _ => this.error = true,
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }))
            }
            Err(_) => {
                self.loading = false;
                self.error = true;
            }
        }
        cx.notify();
    }
    fn observe_mutation(
        &mut self,
        result: Result<TaskSubscription<(), UiStateError>, AdmissionError>,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(mut job) => {
                self.saving = true;
                self.saved = false;
                self.error = false;
                self.mutation = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = job.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                this.saving = false;
                                match result.as_ref() {
                                    Completion::Success(()) => {
                                        this.saved = true;
                                        if this.active {
                                            this.reload(cx);
                                        }
                                    }
                                    _ => this.error = true,
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => self.error = true,
        }
        cx.notify();
    }
    fn add(&mut self, cx: &mut Context<Self>) {
        let command = self.command.read(cx).value().to_string();
        if self.saving || self.confirmation.is_some() || command.trim().is_empty() {
            return;
        }
        let name = self.name.read(cx).value().to_string();
        let args = self.args.read(cx).value().to_string();
        let key = self.key();
        let result = self.requests.add_favorite(
            key,
            AddFavoriteRequest {
                command,
                args: if args.is_empty() {
                    Vec::new()
                } else {
                    args.lines().map(str::to_string).collect()
                },
                display_name: if name.is_empty() { None } else { Some(name) },
                module: self.module.read(cx).value().to_string(),
            },
        );
        self.observe_mutation(result, cx);
    }

    fn ask_confirmation(
        &mut self,
        action: Confirmation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.loading || self.saving || self.error || self.confirmation.is_some() {
            return;
        }
        self.confirmation_return = window.focused(cx);
        self.confirmation = Some(action);
        window.focus(&self.confirmation_focus, cx);
        cx.notify();
    }

    fn cancel_confirmation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.confirmation = None;
        if let Some(focus) = self.confirmation_return.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }

    fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.loading || self.saving || self.error {
            return;
        }
        let Some(action) = self.confirmation.take() else {
            return;
        };
        self.confirmation_return = None;
        let key = self.key();
        let result = match action {
            Confirmation::RemoveFavorite { id, .. } => self.requests.remove_favorite(key, id),
            Confirmation::ClearHistory => self.requests.clear_history(key),
        };
        self.observe_mutation(result, cx);
        // The confirmation (and possibly its trigger row) disappears. Keep
        // keyboard focus in a rendered scope while the mutation completes.
        window.focus(&self.content_focus, cx);
    }
}
impl Render for CommandState {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let history = self.history;
        let snapshot = self.snapshot.clone();
        let count = if history {
            snapshot.history.len()
        } else {
            snapshot.favorites.len()
        };
        let mut content=div().id("command-state").role(gpui_kit::Role::Region).aria_label(if history{text(cx,"Recent commands","最近命令")}else{text(cx,"Favorite commands","收藏命令")}).track_focus(&self.content_focus).tab_stop(false).flex().flex_col().size_full().min_w_0().p(px(theme::CONTENT_PADDING)).gap(px(theme::SECTION_GAP))
            .child(div().text_size(px(theme::TITLE_SIZE)).font_weight(gpui_kit::FontWeight::SEMIBOLD).child(if history{text(cx,"Recent commands","最近命令")}else{text(cx,"Favorite commands","收藏命令")}))
            .child(div().text_color(muted).child(text(cx,"Stored on this desktop, shared across environments. Execution is not connected yet.","记录保存在此桌面，跨环境共享。命令执行尚未接入。")))
            .child(div().flex().gap(px(theme::CONTROL_GAP))
                .child(semantics::button(Button::new("command-state-refresh").label(text(cx,"Refresh","刷新")).ghost().on_click(cx.listener(|this,_,_,cx|this.reload(cx))),self.loading||self.saving||self.confirmation.is_some(),self.loading,cx))
                .child(semantics::button(Button::new("command-history-clear").label(text(cx,"Clear recent commands","清空最近命令")).ghost().on_click(cx.listener(|this,_,window,cx|{if !this.history||this.snapshot.history.is_empty(){return;}this.ask_confirmation(Confirmation::ClearHistory,window,cx);})),!history||self.saving||self.loading||self.error||snapshot.history.is_empty()||self.confirmation.is_some(),self.saving,cx)));
        if !history {
            content = content
                .child(
                    div()
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(
                            div().flex_1().child(text(cx, "Command", "命令")).child(
                                Input::new(&self.command)
                                    .id("favorite-command")
                                    .aria_label(text(cx, "Command name", "命令名称")),
                            ),
                        )
                        .child(
                            div()
                                .flex_1()
                                .child(text(cx, "Display name (optional)", "显示名称（可选）"))
                                .child(
                                    Input::new(&self.name).id("favorite-name").aria_label(text(
                                        cx,
                                        "Favorite display name",
                                        "收藏显示名称",
                                    )),
                                ),
                        )
                        .child(
                            div().flex_1().child(text(cx, "Module", "模块")).child(
                                Input::new(&self.module)
                                    .id("favorite-module")
                                    .aria_label(text(cx, "Command module", "命令模块")),
                            ),
                        ),
                )
                .child(
                    div()
                        .child(text(
                            cx,
                            "Arguments, one per line. Do not store credentials here.",
                            "参数，每行一个。请勿在此保存凭据。",
                        ))
                        .child(
                            Textarea::new(&self.args)
                                .h(px(theme::ROW_HEIGHT * 2.))
                                .accessibility_id("favorite-args")
                                .aria_label(text(cx, "Arguments, one per line", "参数，每行一个")),
                        ),
                )
                .child(semantics::button(
                    Button::new("favorite-add")
                        .label(text(cx, "Add favorite", "添加收藏"))
                        .primary()
                        .on_click(cx.listener(|this, _, _, cx| this.add(cx))),
                    self.saving
                        || self.confirmation.is_some()
                        || self.command.read(cx).value().trim().is_empty(),
                    self.saving,
                    cx,
                ));
        }
        if let Some(action) = &self.confirmation {
            let consequence = match action {
                Confirmation::RemoveFavorite { name, .. } => format!("{}: {name}. {}",text(cx,"Remove favorite","移除收藏"),text(cx,"Its saved arguments will also be deleted. This cannot be undone.","保存的参数也会删除，此操作无法撤销。")),
                Confirmation::ClearHistory => text(cx,"Clear all recent command records stored on this desktop, including records outside this list. This cannot be undone.","清空保存在此桌面的全部最近命令记录，包括列表外的记录。此操作无法撤销。").to_owned(),
            };
            content = content.child(
                div()
                    .id("command-confirmation")
                    .role(gpui_kit::Role::Group)
                    .aria_label(consequence.clone())
                    .track_focus(&self.confirmation_focus)
                    .tab_stop(false)
                    .flex()
                    .flex_col()
                    .gap(px(theme::CONTROL_GAP))
                    .on_key_down(
                        cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                            if event.keystroke.key == "escape" {
                                this.cancel_confirmation(window, cx);
                                cx.stop_propagation();
                            }
                        }),
                    )
                    .child(consequence)
                    .child(
                        div()
                            .flex()
                            .gap(px(theme::CONTROL_GAP))
                            .child(semantics::button(
                                Button::new("command-confirm-cancel")
                                    .label(text(cx, "Cancel", "取消"))
                                    .ghost()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.cancel_confirmation(window, cx)
                                    })),
                                false,
                                false,
                                cx,
                            ))
                            .child(semantics::button(
                                Button::new("command-confirm-apply")
                                    .label(text(cx, "Confirm deletion", "确认删除"))
                                    .primary()
                                    .on_click(
                                        cx.listener(|this, _, window, cx| this.confirm(window, cx)),
                                    ),
                                false,
                                false,
                                cx,
                            )),
                    ),
            );
        }
        let status = if self.saving {
            text(cx, "Saving…", "正在保存…")
        } else if self.loading {
            text(cx, "Reading…", "正在读取…")
        } else if self.error {
            text(
                cx,
                "Cannot access command records. Retry loading; a failed save is not replayed.",
                "无法访问命令记录。请重试读取；失败的保存不会自动重放。",
            )
        } else if self.saved {
            text(cx, "Saved", "已保存")
        } else if count == 0 {
            text(cx, "No command records yet", "暂无命令记录")
        } else {
            ""
        };
        content = content.child(
            div()
                .id("command-state-status")
                .role(gpui_kit::Role::Status)
                .aria_label(status)
                .a11y_synthetic_children(|tree| {
                    tree.parent_node()
                        .set_live(gpui_kit::accesskit::Live::Polite)
                })
                .text_color(if self.error { cx.theme().danger } else { muted })
                .child(status),
        );
        let list = uniform_list(
            "command-records",
            count,
            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                range
                    .map(|index| {
                        if history {
                            let row = &snapshot.history[index];
                            div()
                                .id(SharedString::from(format!("history:{}", row.id)))
                                .role(gpui_kit::Role::ListItem)
                                .aria_label(row.command.clone())
                                .h(px(theme::ROW_HEIGHT))
                                .flex()
                                .items_center()
                                .gap(px(theme::SECTION_GAP))
                                .child(div().flex_1().child(row.command.clone()))
                                .child(if row.success {
                                    text(cx, "Succeeded", "成功")
                                } else {
                                    text(cx, "Failed", "失败")
                                })
                                .child(
                                    div()
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .child(format!("{} ms", row.duration_ms)),
                                )
                                .child(
                                    div()
                                        .text_color(muted)
                                        .text_size(px(theme::META_SIZE))
                                        .child(row.executed_at.clone()),
                                )
                                .into_any_element()
                        } else {
                            let row = &snapshot.favorites[index];
                            let id = row.id.clone();
                            let name = row.display_name.as_ref().unwrap_or(&row.command).clone();
                            div()
                                .id(SharedString::from(format!("favorite:{}", row.id)))
                                .role(gpui_kit::Role::ListItem)
                                .aria_label(
                                    row.display_name.as_ref().unwrap_or(&row.command).clone(),
                                )
                                .h(px(theme::ROW_HEIGHT))
                                .flex()
                                .items_center()
                                .gap(px(theme::SECTION_GAP))
                                .child(div().flex_1().child(
                                    row.display_name.as_ref().unwrap_or(&row.command).clone(),
                                ))
                                .child(div().text_color(muted).child(row.command.clone()))
                                .child(semantics::button(
                                    Button::new(SharedString::from(format!(
                                        "remove-favorite:{}",
                                        row.id
                                    )))
                                    .label(text(cx, "Remove", "移除"))
                                    .ghost()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.ask_confirmation(
                                            Confirmation::RemoveFavorite {
                                                id: id.clone(),
                                                name: name.clone(),
                                            },
                                            window,
                                            cx,
                                        );
                                    })),
                                    this.saving
                                        || this.loading
                                        || this.error
                                        || this.confirmation.is_some(),
                                    this.saving,
                                    cx,
                                ))
                                .into_any_element()
                        }
                    })
                    .collect()
            }),
        )
        .flex_1()
        .size_full();
        content.child(div().flex_1().min_h_0().child(list))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::ConfigSource;
    use ccr_app::runtime::RuntimeOwner;
    use gpui_kit::test::{TestAppContextExt, TestWindowExt};
    use gpui_kit::{TestAppContext, WindowOptions};
    use std::time::Duration;

    #[gpui_kit::test]
    async fn real_favorite_controls_persist_reopen_and_remove_without_execution(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("config.toml"),
                delay: Duration::ZERO,
            },
        );
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    let mut view = CommandState::new(&services, window, cx);
                    view.open(false, cx);
                    view
                })
            })
            .unwrap_or_else(|e| panic!("window: {e}"))
        });
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            assert!(!view.read(cx).error);
            window.click("favorite-command", cx);
            window.input("list", cx);
            window.click("favorite-name", cx);
            window.input("Native fixture favorite", cx);
            window.click("favorite-module", cx);
            window.input("configuration", cx);
            window.click("favorite-add", cx);
        })
        .unwrap_or_else(|e| panic!("interaction: {e}"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).snapshot.favorites.len() == 1 && !view.read(cx).loading
        })
        .await;
        let id = cx.read(|cx| view.read(cx).snapshot.favorites[0].id.clone());
        cx.update_window(handle, |_, window, cx| {
            window.click("command-state-refresh", cx);
        })
        .unwrap_or_else(|e| panic!("interaction: {e}"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            assert_eq!(view.read(cx).snapshot.favorites[0].id, id);
            assert_eq!(view.read(cx).snapshot.favorites[0].command, "list");
            assert!(
                view.read(cx).snapshot.history.is_empty(),
                "saving a favorite must never invent execution history"
            );
            window.click(SharedString::from(format!("remove-favorite:{id}")), cx);
            assert!(window.find("command-confirm-apply").visible());
            assert_eq!(view.read(cx).snapshot.favorites.len(), 1);
            window.press("escape", cx);
            assert!(view.read(cx).confirmation.is_none());
            assert_eq!(view.read(cx).snapshot.favorites.len(), 1);
            window.click(SharedString::from(format!("remove-favorite:{id}")), cx);
            window.click("command-confirm-cancel", cx);
            assert!(view.read(cx).confirmation.is_none());
            assert_eq!(view.read(cx).snapshot.favorites.len(), 1);
            window.click(SharedString::from(format!("remove-favorite:{id}")), cx);
            window.click("command-confirm-apply", cx);
        })
        .unwrap_or_else(|e| panic!("interaction: {e}"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).snapshot.favorites.is_empty()
                && !view.read(cx).loading
                && !view.read(cx).saving
        })
        .await;
        cx.update_window(handle, |_, window, _| window.remove_window())
            .unwrap_or_else(|e| panic!("close: {e}"));
        owner.shutdown();
        assert!(root.path().join("ui-state.db").exists());
    }

    #[gpui_kit::test]
    async fn clear_history_requires_confirmation_and_escape_restores_trigger_focus(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        // Seed only through the domain owner before the application opens its pool.
        {
            use ccr_db::{database, managers::ui_state::UiStateManager};
            let home = root.path().join("seed-home");
            std::fs::create_dir(&home).unwrap_or_else(|e| panic!("fixture home: {e}"));
            let pool = database::create_pool(&root.path().join("ui-state.db"), None)
                .unwrap_or_else(|e| panic!("fixture pool: {e}"));
            {
                let conn = pool
                    .get()
                    .unwrap_or_else(|e| panic!("fixture connection: {e}"));
                database::migrations::run_all_migrations(&conn, &home)
                    .unwrap_or_else(|e| panic!("fixture migrations: {e}"));
            }
            let manager = UiStateManager::with_db(database::DbAccess::Pool(pool));
            manager
                .add_history("list".into(), vec![], true, 1)
                .unwrap_or_else(|e| panic!("fixture history: {e}"));
        }
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("config.toml"),
                delay: Duration::ZERO,
            },
        );
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    let mut view = CommandState::new(&services, window, cx);
                    view.open(true, cx);
                    view
                })
            })
            .unwrap_or_else(|e| panic!("window: {e}"))
        });
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert_eq!(view.read(cx).snapshot.history.len(), 1);
            // Standalone feature tests have no shell activation observer to
            // establish initial focus. Start on the actual first control.
            window.focus_next(cx);
            window.render_frame(cx);
            for _ in 0..3 {
                if window.find("command-history-clear").focused() == Some(true) {
                    break;
                }
                window.press("tab", cx);
            }
            assert_eq!(window.find("command-history-clear").focused(), Some(true));
            window.press("enter", cx);
            assert!(window.find("command-confirm-apply").visible());
            assert!(!view.read(cx).saving);
            window.press("escape", cx);
            assert!(view.read(cx).confirmation.is_none());
            assert_eq!(window.find("command-history-clear").focused(), Some(true));
            assert_eq!(view.read(cx).snapshot.history.len(), 1);
            window.press("enter", cx);
            window.click("command-confirm-cancel", cx);
            assert_eq!(view.read(cx).snapshot.history.len(), 1);
            window.press("enter", cx);
            window.click("command-confirm-apply", cx);
        })
        .unwrap_or_else(|e| panic!("confirmation: {e}"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            let view = view.read(cx);
            !view.loading && !view.saving && view.snapshot.history.is_empty()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            assert!(!view.read(cx).error);
            assert!(view.read(cx).content_focus.is_focused(window));
            window.press("tab", cx);
            assert_eq!(window.find("command-state-refresh").focused(), Some(true));
            window.click("command-state-refresh", cx);
        })
        .unwrap_or_else(|e| panic!("reread: {e}"));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            assert!(view.read(cx).snapshot.history.is_empty());
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("close: {e}"));
        owner.shutdown();
    }
}
