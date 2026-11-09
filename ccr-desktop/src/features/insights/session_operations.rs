//! Selected-session operations; transcript exports stay only in this view.
//!
//! 操作绑定到 Sessions 产品面的详情槽（`detail.archive_id` / `detail.generation`），
//! 不随列表翻页或筛选失效；废纸篓独立于列表与详情，只由 decision generation 守卫。
use super::*;
use crate::runtime::insights::session_operations::{Action, Failure, Prepared, State};
use ccr_codex::CodexSessionExport;
use gpui_kit::component::FocusTrapElement;
mod delivery;

#[derive(Default)]
pub(super) struct ViewState {
    task: Option<Task<()>>,
    observation: Option<Task<()>>,
    pending: Option<(Action, Arc<Prepared>)>,
    preview: Option<Arc<CodexSessionExport>>,
    state: Arc<State>,
    busy: bool,
    error: Option<Failure>,
    copied: bool,
    entry_pending: bool,
    decision_generation: u64,
    focus: Option<FocusHandle>,
    /// 废纸篓面板是否打开（包括正在读取、恢复核对与确认）。
    trash_open: bool,
    trash: Option<Arc<Vec<crate::runtime::insights::session_operations::TrashItem>>>,
    trash_page: usize,
    restore: Option<Arc<crate::runtime::insights::session_operations::PreparedRestore>>,
    restore_preparing: bool,
    restore_return_focus: Option<FocusHandle>,
}
impl InsightsView {
    pub(super) fn session_ops_trash_open(&self) -> bool {
        self.session_ops.trash_open
    }
    #[cfg(test)]
    pub(super) fn session_trash_loaded(&self) -> bool {
        self.session_ops.trash.is_some()
    }

    pub(super) fn session_trash_empty(&self) -> bool {
        self.session_ops
            .trash
            .as_ref()
            .is_some_and(|items| items.is_empty())
    }
    #[cfg(test)]
    pub(super) fn session_restore_pending(&self) -> bool {
        self.session_ops.restore.is_some()
    }
    /// 把已读到的第一条废纸篓条目复制成 `count` 条，用于长列表版式测试。
    #[cfg(test)]
    pub(super) fn repeat_session_trash(&mut self, count: usize) {
        let Some(first) = self
            .session_ops
            .trash
            .as_ref()
            .and_then(|items| items.first().cloned())
        else {
            return;
        };
        self.session_ops.trash = Some(Arc::new(
            (0..count)
                .map(|index| {
                    let mut item = first.clone();
                    item.session_id = format!("synthetic-{index}");
                    item
                })
                .collect(),
        ));
    }
    /// A session operation or trash read is running.
    pub(super) fn session_ops_active(&self) -> bool {
        self.session_ops.busy || self.session_ops.state.busy
    }
    pub(super) fn clear_session_draft(&mut self) {
        self.session_ops.task = None;
        self.session_ops.pending = None;
        self.session_ops.preview = None;
        self.session_ops.trash_open = false;
        self.session_ops.trash = None;
        self.session_ops.trash_page = 0;
        self.session_ops.restore = None;
        self.session_ops.restore_preparing = false;
        self.session_ops.restore_return_focus = None;
        self.session_ops.busy = false;
        self.session_ops.error = None;
        self.session_ops.copied = false;
        self.session_ops.entry_pending = false;
        self.session_ops.decision_generation += 1;
    }
    /// Closes the trash panel; the opened detail, if any, takes the pane again.
    pub(super) fn close_session_trash(&mut self, cx: &mut Context<Self>) {
        self.clear_session_draft();
        cx.notify();
    }
    pub(super) fn observe_session_operations(&mut self, cx: &mut Context<Self>) {
        self.session_ops.state = self.requests.session_operation_snapshot();
        if self.session_ops.observation.is_some() {
            return;
        }
        let mut updates = self.requests.observe_session_operations();
        self.session_ops.observation = Some(cx.spawn(async move |weak, cx| {
            while updates.changed().await.is_ok() {
                let state = updates.borrow_and_update().clone();
                if weak
                    .update(cx, |this, cx| {
                        let removed = state
                            .result
                            .as_ref()
                            .and_then(|v| v.as_ref().ok())
                            .is_some_and(|outcome| {
                                outcome.action == Action::Restore
                                    || (outcome.action == Action::Trash
                                        && this.opened_session()
                                            == Some(outcome.archive_id.as_str()))
                            });
                        this.session_ops.state = state;
                        if removed {
                            this.refresh(cx);
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        }));
    }
    fn session_key(&self) -> RequestKey {
        RequestKey {
            environment: EnvironmentId::Local("insights-explicit".into()),
            platform: Platform::Codex,
            generation: self.generation,
            request_id: self.generation,
        }
    }
    /// 详情操作的作用域：打开的会话与它的详情 generation。
    fn session_scope(&self) -> Option<(String, u64)> {
        let id = self.opened_session()?.to_string();
        Some((id, self.detail.generation))
    }
    fn session_scope_matches(&self, id: &str, generation: u64) -> bool {
        self.detail.generation == generation && self.opened_session() == Some(id)
    }
    fn prepare_session_action(&mut self, action: Action, cx: &mut Context<Self>) {
        if self.session_ops_active() || self.detail.loading {
            return;
        }
        let Some((id, generation)) = self.session_scope() else {
            return;
        };
        self.clear_session_draft();
        self.session_ops.busy = true;
        match self
            .requests
            .prepare_session_operation(self.session_key(), id.clone())
        {
            Ok(mut request) => {
                self.session_ops.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = request.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.session_scope_matches(&id, generation) {
                                    return;
                                }
                                this.session_ops.busy = false;
                                match result.as_ref() {
                                    Completion::Success(prepared) => {
                                        this.session_ops.pending = Some((action, prepared.clone()));
                                        this.session_ops.decision_generation += 1;
                                        this.session_ops.entry_pending = true;
                                    }
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => {
                                        this.session_ops.error = Some(*error)
                                    }
                                    _ => this.session_ops.error = Some(Failure::Interrupted),
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }))
            }
            Err(_) => {
                self.session_ops.busy = false;
                self.session_ops.error = Some(Failure::Interrupted);
            }
        }
        cx.notify();
    }
    fn export_selected_session(&mut self, cx: &mut Context<Self>) {
        if self.session_ops_active() || self.detail.loading {
            return;
        }
        let Some((id, generation)) = self.session_scope() else {
            return;
        };
        self.clear_session_draft();
        self.session_ops.busy = true;
        match self.requests.export_session(self.session_key(), id.clone()) {
            Ok(mut request) => {
                self.session_ops.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = request.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.session_scope_matches(&id, generation) {
                                    return;
                                }
                                this.session_ops.busy = false;
                                match result.as_ref() {
                                    Completion::Success(export) => {
                                        this.session_ops.preview = Some(export.clone())
                                    }
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => {
                                        this.session_ops.error = Some(*error)
                                    }
                                    _ => this.session_ops.error = Some(Failure::Interrupted),
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }))
            }
            Err(_) => {
                self.session_ops.busy = false;
                self.session_ops.error = Some(Failure::Interrupted);
            }
        }
        cx.notify();
    }
    /// Clone / trash confirmation for the opened session, mounted above the transcript.
    pub(super) fn render_session_confirmation(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<gpui_kit::AnyElement> {
        let action = self
            .session_ops
            .pending
            .as_ref()
            .map(|(action, _)| *action)?;
        let focus = self
            .session_ops
            .focus
            .get_or_insert_with(|| cx.focus_handle().tab_stop(false))
            .clone();
        if std::mem::take(&mut self.session_ops.entry_pending) {
            let generation = self.session_ops.decision_generation;
            let weak = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    if this.session_ops.decision_generation == generation
                        && this.session_ops.pending.is_some()
                        && let Some(focus) = &this.session_ops.focus
                    {
                        window.focus(focus, cx);
                        window.focus_next(cx);
                    }
                });
            });
        }
        let root = div()
            .id("session-action-confirmation")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP));
        Some(root.role(gpui_kit::Role::AlertDialog).track_focus(&focus).tab_stop(false)
            .on_key_down(cx.listener(|this,event:&gpui_kit::KeyDownEvent,_,cx|{if event.keystroke.key=="escape"{this.session_ops.pending=None;cx.stop_propagation();cx.notify();}}))
            .child(if action==Action::Trash{text(cx,"Move this Codex session to CCR trash? The original rollout and session-index entry are removed after a recoverable copy is saved. Open CCR trash to restore it.","将此 Codex 会话移入 CCR 废纸篓？保存可恢复副本后，移除原始记录和会话索引条目。可打开 CCR 废纸篓恢复。")}else{text(cx,"Clone this Codex session with a new identity? The original file stays unchanged.","以新标识克隆此 Codex 会话？原始文件保持不变。")})
            .child(div().flex().gap(px(theme::CONTROL_GAP)).tab_group().tab_stop(false)
                .child(semantics::button(Button::new("session-action-cancel").label(text(cx,"Cancel","取消")).ghost().on_click(cx.listener(|this,_,_,cx|{this.session_ops.pending=None;cx.notify();})),false,false,cx))
                .child(semantics::button(Button::new("session-action-confirm").label(text(cx,"Confirm","确认")).on_click(cx.listener(|this,_,_,cx|{
                    if let Some((action,prepared))=this.session_ops.pending.take(){this.session_ops.error=this.requests.start_session_operation(this.session_key(),prepared,action).err();this.observe_session_operations(cx);}cx.notify();
                })),false,false,cx)).focus_trap("session-action-trap",&focus)).into_any_element())
    }
    /// Export, clone and trash for a live Codex session; nothing for other providers.
    pub(super) fn render_session_actions(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Option<gpui_kit::AnyElement> {
        let supported = self.session_detail().is_some_and(|detail| {
            detail.agent == AgentSessionAgentId::Codex && detail.variant == "codex-live"
        });
        if !supported {
            return None;
        }
        let active = self.session_ops_active() || self.detail.loading;
        Some(
            div()
                .flex()
                .flex_wrap()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("session-export")
                        .label(text(cx, "Export Markdown", "导出 Markdown"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| this.export_selected_session(cx))),
                    active,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("session-clone")
                        .label(text(cx, "Clone session", "克隆会话"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.prepare_session_action(Action::Clone, cx)
                        })),
                    active,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("session-trash")
                        .label(text(cx, "Move to CCR trash", "移入 CCR 废纸篓"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.prepare_session_action(Action::Trash, cx)
                        })),
                    active,
                    false,
                    cx,
                ))
                .into_any_element(),
        )
    }
    /// Running, failure and outcome lines of session operations; `None` when idle.
    pub(super) fn render_session_feedback(
        &self,
        cx: &mut Context<Self>,
    ) -> Option<gpui_kit::AnyElement> {
        let error = self.session_ops.error.or_else(|| {
            self.session_ops
                .state
                .result
                .as_ref()
                .and_then(|r| r.as_ref().err())
                .copied()
        });
        let outcome = match &self.session_ops.state.result {
            Some(Ok(outcome)) => Some(outcome),
            _ => None,
        };
        if !self.session_ops_active() && error.is_none() && outcome.is_none() {
            return None;
        }
        let mut root = div()
            .id("session-feedback")
            .test_support()
            .role(gpui_kit::Role::Status)
            .flex()
            .flex_col()
            .gap(px(theme::SELECTION_RADIUS))
            .text_size(px(theme::META_SIZE))
            .text_color(cx.theme().muted_foreground);
        if self.session_ops_active() {
            root = root.child(text(
                cx,
                "Session operation is running.",
                "会话操作正在运行。",
            ));
        }
        if let Some(error) = error {
            root = root.child(
                div()
                    .text_color(theme::tone_colors(theme::Tone::Danger, cx.theme().mode).ink)
                    .child(failure(error, cx)),
            );
        }
        if let Some(outcome) = outcome {
            let line = match outcome.action {
                Action::Clone => text(cx, "Session cloned.", "会话已克隆。"),
                Action::Save => text(
                    cx,
                    "Markdown saved to the selected new file.",
                    "Markdown 已保存到所选新文件。",
                ),
                Action::Restore => text(
                    cx,
                    "Session restored from CCR trash.",
                    "会话已从 CCR 废纸篓恢复。",
                ),
                Action::Trash => text(
                    cx,
                    "Session moved to CCR trash. Open CCR trash to restore it.",
                    "会话已移入 CCR 废纸篓，可打开废纸篓恢复。",
                ),
            };
            root = root.child(format!("{line} {}", outcome.session_id));
            if outcome.action == Action::Trash {
                root = root.child(format!(
                    "{}: {}",
                    text(cx, "Codex home", "Codex 目录"),
                    outcome.codex_home.display()
                ));
            }
            if !outcome.archive_updated {
                root=root.child(text(cx,"Source operation completed, but CCR archive update failed. Refresh the index before continuing.","源文件操作已完成，但 CCR 归档更新失败。请先刷新索引再继续。"));
            }
        }
        Some(root.into_any_element())
    }
    /// The Markdown export preview with save and copy; `None` without a preview.
    pub(super) fn render_session_export(
        &self,
        cx: &mut Context<Self>,
    ) -> Option<gpui_kit::AnyElement> {
        let preview = self.session_ops.preview.as_ref()?;
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(format!(
                "{} · {}",
                preview.file_name,
                if preview.truncated {
                    text(
                        cx,
                        "Truncated to the domain export limit",
                        "已按领域导出上限截断",
                    )
                } else {
                    text(cx, "Within the export message limit", "未超出导出消息上限")
                }
            ))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("session-save-markdown")
                            .label(text(
                                cx,
                                "Save as new Markdown file…",
                                "另存为新的 Markdown 文件…",
                            ))
                            .on_click(cx.listener(|this, _, _, cx| this.save_session_markdown(cx))),
                        self.session_ops_active(),
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("session-copy-markdown")
                            .label(text(cx, "Copy Markdown", "复制 Markdown"))
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                if let Some(preview) = &this.session_ops.preview {
                                    cx.write_to_clipboard(gpui_kit::ClipboardItem::new_string(
                                        preview.content.clone(),
                                    ));
                                    this.session_ops.copied = true;
                                    cx.notify();
                                }
                            })),
                        false,
                        false,
                        cx,
                    )),
            )
            .child(
                div()
                    .id("session-markdown-preview")
                    .test_support()
                    .max_h(px(240.))
                    .overflow_y_scroll()
                    .p(px(theme::CONTROL_GAP))
                    .rounded(px(theme::INPUT_RADIUS))
                    .bg(cx.theme().secondary)
                    .font_family(cx.theme().mono_font_family.clone())
                    .text_size(px(theme::META_SIZE))
                    .child(preview.content.clone()),
            );
        if self.session_ops.copied {
            root = root.child(text(cx, "Markdown copied.", "Markdown 已复制。"));
        }
        Some(root.into_any_element())
    }
}
fn failure(error: Failure, cx: &gpui_kit::App) -> &'static str {
    match error {
        Failure::LocalOnly => text(
            cx,
            "Session operations are local-only.",
            "会话操作仅支持本地环境。",
        ),
        Failure::Unsupported => text(
            cx,
            "Operations support live Codex sessions only.",
            "操作仅支持 Codex 活跃会话。",
        ),
        Failure::Conflict => text(
            cx,
            "The session or trash changed, or the destination already exists. Read again or choose a new file.",
            "会话或废纸篓已变化，或目标文件已存在。请重新读取或选择新文件。",
        ),
        Failure::Partial => text(
            cx,
            "The session operation did not finish cleanly; files may have changed. Refresh the source and CCR archive before retrying.",
            "会话操作未完整结束，文件可能已变化。重试前请刷新源文件和 CCR 归档。",
        ),
        Failure::Busy => text(
            cx,
            "A session operation is already running.",
            "已有会话操作正在运行。",
        ),
        _ => text(
            cx,
            "Cannot complete the session operation. Check the source and archive, then retry.",
            "无法完成会话操作，请检查来源与归档后重试。",
        ),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use gpui_kit::{
        TestAppContext, WindowOptions,
        test::{TestAppContextExt, TestWindowExt},
    };
    use std::time::Duration;

    /// 详情已读到（无论存放在哪个状态槽）。
    fn detail_loaded(view: &InsightsView) -> bool {
        view.session_detail().is_some()
            || matches!(view.snapshot.as_deref(), Some(InsightsSnapshot::Detail(_)))
    }

    /// 视图里是否还持有会话正文或导出明文。
    fn holds_plaintext(view: &InsightsView) -> bool {
        let held = |snapshot: Option<&InsightsSnapshot>| match snapshot {
            Some(InsightsSnapshot::Detail(detail)) => detail
                .page
                .messages
                .iter()
                .any(|message| message.content.contains("private synthetic transcript")),
            _ => false,
        };
        let transcript = held(view.snapshot.as_deref()) || held(view.detail.detail.as_deref());
        transcript || view.session_ops.preview.is_some()
    }

    fn open_sessions(
        cx: &mut TestAppContext,
        services: Services,
    ) -> (gpui_kit::AnyWindowHandle, Entity<InsightsView>) {
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| InsightsView::new(services, window, cx))
            })
            .unwrap()
        });
        view.update(cx, |this, cx| this.open_product(InsightsPage::Sessions, cx));
        (handle, view)
    }

    #[gpui_kit::test]
    async fn leaving_sessions_clears_transcript_and_export_plaintext(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let (_root, owner, services, _, id) =
            crate::runtime::insights::session_operations::tests::fixture();
        let (handle, view) = open_sessions(cx, services);
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click(SharedString::from(format!("insights-row:{id}")), cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            detail_loaded(view.read(cx))
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-export", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).session_ops.preview.is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(holds_plaintext(view.read(cx)));
            view.update(cx, |this, cx| this.suspend(cx));
            window.render_frame(cx);
            assert!(!holds_plaintext(view.read(cx)));
            assert!(!detail_loaded(view.read(cx)));
            assert!(view.read(cx).session_ops.task.is_none());
            assert!(view.read(cx).task.is_none());
            assert!(view.read(cx).detail.task.is_none());
            assert!(view.read(cx).search_task.is_none());
            assert!(
                gpui_kit::base::test_support::snapshots(window)
                    .iter()
                    .all(|node| !node
                        .label()
                        .is_some_and(|label| label.contains("private synthetic")))
            );
            window.remove_window();
        })
        .unwrap();
        owner.shutdown();
    }

    #[gpui_kit::test]
    async fn open_trash_keeps_the_search_field_visible_and_clickable(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let (root, owner, services, _, _) =
            crate::runtime::insights::session_operations::tests::fixture();
        let trash =
            ccr_codex::CodexSessionTrashService::with_codex_home(root.path().join("home/.codex"));
        ccr_core::core::LockManager::with_scoped_directory(&root.path().join("locks"), || {
            trash.trash_sessions(["synthetic-session"])
        })
        .unwrap();
        let (handle, view) = open_sessions(cx, services);
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-open-trash", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).session_ops.trash.is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(window.find("session-restore:0").visible());
            assert!(window.find("insights-filter-6").visible());
            window.click("insights-filter-6", cx);
            assert_eq!(window.find("insights-filter-6").focused(), Some(true));
            assert!(window.try_find("session-restore:0").is_some());
            window.remove_window();
        })
        .unwrap();
        owner.shutdown();
    }

    #[gpui_kit::test]
    async fn session_trash_pagination_bounds_large_collections_and_keeps_controls_reachable(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let (root, owner, services, _, _) =
            crate::runtime::insights::session_operations::tests::fixture();
        let trash =
            ccr_codex::CodexSessionTrashService::with_codex_home(root.path().join("home/.codex"));
        ccr_core::core::LockManager::with_scoped_directory(&root.path().join("locks"), || {
            trash.trash_sessions(["synthetic-session"])
        })
        .unwrap();
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| InsightsView::new(services, window, cx))
            })
            .unwrap()
        });
        view.update(cx, |this, cx| this.open_product(InsightsPage::Sessions, cx));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-open-trash", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).session_ops.trash.is_some()
        })
        .await;
        view.update(cx, |this, cx| {
            let item = this.session_ops.trash.as_ref().unwrap()[0].clone();
            this.session_ops.trash = Some(Arc::new(
                (0..10_000)
                    .map(|index| {
                        let mut item = item.clone();
                        item.session_id = format!("synthetic-{index}");
                        item
                    })
                    .collect(),
            ));
            cx.notify();
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let count = |window: &Window| {
                gpui_kit::base::test_support::snapshots(window)
                    .iter()
                    .filter(|node| node.label() == Some("Review restore…"))
                    .count()
            };
            assert_eq!(count(window), 6);
            assert!(window.try_find("session-restore:5").is_some());
            assert!(window.try_find("session-restore:6").is_none());
            // Disabled state is emitted through synthetic accessibility nodes,
            // outside Kit's control snapshot; assert the actual boundary action.
            window.click("session-trash-prev", cx);
            assert_eq!(view.read(cx).session_ops.trash_page, 0);
            window.click("session-trash-next", cx);
            assert_eq!(count(window), 6);
            assert!(window.try_find("session-restore:0").is_none());
            assert!(window.try_find("session-restore:6").is_some());
            assert_eq!(view.read(cx).session_ops.trash_page, 1);
            window.click("session-trash-prev", cx);
            assert!(window.try_find("session-restore:0").is_some());
            view.update(cx, |this, cx| {
                this.clear_session_draft();
                cx.notify();
            });
            assert_eq!(view.read(cx).session_ops.trash_page, 0);
            window.remove_window();
        })
        .unwrap();
        owner.shutdown();
    }

    #[gpui_kit::test]
    async fn selected_codex_controls_export_cancel_and_clear_plaintext(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let (root, owner, services, path, id) =
            crate::runtime::insights::session_operations::tests::fixture();
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| InsightsView::new(services, window, cx))
            })
            .unwrap()
        });
        view.update(cx, |this, cx| this.open_product(InsightsPage::Sessions, cx));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click(SharedString::from(format!("insights-row:{id}")), cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            detail_loaded(view.read(cx))
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-export", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).session_ops.preview.is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-save-markdown", cx);
        })
        .unwrap();
        assert!(cx.did_prompt_for_new_path());
        cx.simulate_new_path_selection(|_| None);
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).session_ops.busy
        })
        .await;
        assert!(!root.path().join("saved.md").exists());
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-save-markdown", cx);
        })
        .unwrap();
        let destination = root.path().join("saved.md");
        cx.simulate_new_path_selection(|_| Some(destination.clone()));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            matches!(&view.read(cx).session_ops.state.result,Some(Ok(value)) if value.action==Action::Save)
        }).await;
        assert!(
            std::fs::read_to_string(&destination)
                .unwrap()
                .contains("private synthetic transcript")
        );
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-copy-markdown", cx);
            assert!(
                cx.read_from_clipboard()
                    .unwrap()
                    .text()
                    .unwrap()
                    .contains("private synthetic transcript")
            );
            window.click("session-trash", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).session_ops.pending.is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-action-cancel", cx);
            assert!(path.exists());
        })
        .unwrap();
        // The restore UI remains reachable after the active row disappears.
        let requests = view.read_with(cx, |this, _| this.requests.clone());
        let trash =
            ccr_codex::CodexSessionTrashService::with_codex_home(root.path().join("home/.codex"));
        ccr_core::core::LockManager::with_scoped_directory(&root.path().join("locks"), || {
            trash.trash_sessions(["synthetic-session"])
        })
        .unwrap();
        view.update(cx, |this, cx| {
            this.archive_id = None;
            this.snapshot = None;
            this.session_ops.preview = None;
            this.refresh(cx);
        });
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-open-trash", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).session_ops.trash.is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-restore:0", cx);
            assert!(view.read(cx).session_ops.restore_preparing);
            view.update(cx, |this, cx| {
                this.clear_session_draft();
                cx.notify();
            });
            assert!(view.read(cx).session_ops.task.is_none());
            assert!(view.read(cx).session_ops.restore.is_none());
            assert!(!view.read(cx).session_ops.restore_preparing);
            window.render_frame(cx);
            window.click("session-open-trash", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).session_ops.trash.is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("insights-filter-6", cx);
            window.click("session-restore:0", cx);
            window.render_frame(cx);
            window.click("session-restore-prepare-cancel", cx);
            assert!(!view.read(cx).session_ops.restore_preparing);
            assert!(view.read(cx).session_ops.restore.is_none());
            assert!(!path.exists());
            window.render_frame(cx);
            window.click("insights-filter-6", cx);
            window.click("session-restore:0", cx);
            window.press("enter", cx);
            assert!(!path.exists(), "Enter before focus entry must not restore");
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).session_ops.restore.is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("session-restore-cancel").focused(), Some(true));
            window.press("tab", cx);
            assert_eq!(window.find("session-restore-confirm").focused(), Some(true));
            window.press("shift-tab", cx);
            assert_eq!(window.find("session-restore-cancel").focused(), Some(true));
            window.press("escape", cx);
            assert!(window.try_find("session-restore-confirm").is_none());
            assert_eq!(window.find("insights-filter-6").focused(), Some(true));
        })
        .unwrap();
        assert!(!path.exists());
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-restore:0", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).session_ops.restore.is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-restore-confirm", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            matches!(&view.read(cx).session_ops.state.result,Some(Ok(value)) if value.action==Action::Restore)
        }).await;
        assert!(path.exists());
        assert!(!requests.session_operation_snapshot().busy);
        view.update(cx, |this, cx| this.suspend(cx));
        cx.update_window(handle, |_, window, cx| {
            assert!(view.read(cx).session_ops.preview.is_none());
            assert!(view.read(cx).session_ops.pending.is_none());
            assert!(view.read(cx).snapshot.is_none());
            window.remove_window();
        })
        .unwrap();
        owner.shutdown();
    }
}
