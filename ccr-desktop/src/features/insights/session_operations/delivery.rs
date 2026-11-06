//! Explicit native file delivery and metadata-only recoverable trash UI.
use super::*;
use gpui_kit::component::tooltip::Tooltip;

const TRASH_PAGE_SIZE: usize = 6;

impl InsightsView {
    pub(super) fn save_session_markdown(&mut self, cx: &mut Context<Self>) {
        if self.session_ops.busy || self.session_ops.state.busy {
            return;
        }
        let (Some(preview), Some(id), Some(directory)) = (
            self.session_ops.preview.clone(),
            self.archive_id.clone(),
            self.requests.session_save_directory(),
        ) else {
            return;
        };
        let generation = self.generation;
        let decision = self.session_ops.decision_generation;
        let selected = cx.prompt_for_new_path(&directory, Some(&preview.file_name));
        self.session_ops.busy = true;
        self.session_ops.task = Some(cx.spawn(async move |weak, cx| {
            let selected = selected.await;
            let _ = weak.update(cx, |this, cx| {
                if this.generation != generation
                    || this.session_ops.decision_generation != decision
                    || this.archive_id.as_deref() != Some(&id)
                {
                    return;
                }
                this.session_ops.busy = false;
                match selected {
                    Ok(Ok(Some(path))) => {
                        this.session_ops.error = this
                            .requests
                            .start_session_save(this.session_key(), id, preview, path)
                            .err();
                        this.observe_session_operations(cx);
                    }
                    Ok(Ok(None)) => (), // Cancel is not a failure and never admits a writer.
                    _ => this.session_ops.error = Some(Failure::Storage),
                }
                cx.notify();
            });
        }));
        cx.notify();
    }
    fn load_session_trash(&mut self, cx: &mut Context<Self>) {
        if self.session_ops.busy || self.session_ops.state.busy {
            return;
        }
        self.clear_session_draft();
        self.session_ops.busy = true;
        let generation = self.generation;
        let decision = self.session_ops.decision_generation;
        match self.requests.list_session_trash(self.session_key()) {
            Ok(mut request) => {
                self.session_ops.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = request.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if this.generation != generation
                                    || this.session_ops.decision_generation != decision
                                {
                                    return;
                                }
                                this.session_ops.busy = false;
                                match result.as_ref() {
                                    Completion::Success(items) => {
                                        this.session_ops.trash = Some(items.clone())
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
    fn prepare_session_restore(
        &mut self,
        item: crate::runtime::insights::session_operations::TrashItem,
        cx: &mut Context<Self>,
    ) {
        if self.session_ops.busy || self.session_ops.state.busy {
            return;
        }
        self.session_ops.busy = true;
        self.session_ops.restore_preparing = true;
        self.session_ops.error = None;
        self.session_ops.decision_generation += 1;
        let decision = self.session_ops.decision_generation;
        let generation = self.generation;
        match self
            .requests
            .prepare_session_restore(self.session_key(), item)
        {
            Ok(mut request) => {
                self.session_ops.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = request.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if this.generation != generation
                                    || this.session_ops.decision_generation != decision
                                {
                                    return;
                                }
                                this.session_ops.busy = false;
                                this.session_ops.restore_preparing = false;
                                match result.as_ref() {
                                    Completion::Success(prepared) => {
                                        this.session_ops.restore = Some(prepared.clone());
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
                self.session_ops.restore_preparing = false;
                self.session_ops.error = Some(Failure::Interrupted);
            }
        }
        cx.notify();
    }
    fn cancel_restore_preparation(&mut self, cx: &mut Context<Self>) {
        self.session_ops.task = None;
        self.session_ops.busy = false;
        self.session_ops.restore_preparing = false;
        self.session_ops.decision_generation += 1;
        cx.notify();
    }
    pub(super) fn render_session_trash(&mut self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let active = self.session_ops.busy || self.session_ops.state.busy;
        let mut root = div()
            .id("session-trash-panel")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(semantics::button(
                Button::new("session-open-trash")
                    .label(text(
                        cx,
                        "Open / refresh CCR trash",
                        "打开 / 刷新 CCR 废纸篓",
                    ))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| this.load_session_trash(cx))),
                active,
                false,
                cx,
            ));
        if self.session_ops.restore_preparing {
            root = root.child(semantics::button(
                Button::new("session-restore-prepare-cancel")
                    .label(text(cx, "Cancel restore review", "取消恢复核对"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| this.cancel_restore_preparation(cx))),
                false,
                false,
                cx,
            ));
        }
        if let Some(items) = &self.session_ops.trash {
            root = root.child(text(cx,"Recoverable Codex sessions from this captured local home. Existing rollout files and index identities are never overwritten.","此本地 Codex 目录中的可恢复会话。不会覆盖已有记录文件或索引标识。"));
            if items.is_empty() {
                root = root.child(text(cx, "CCR trash is empty.", "CCR 废纸篓为空。"));
            }
            let pages = items.len().div_ceil(TRASH_PAGE_SIZE).max(1);
            let page = self.session_ops.trash_page.min(pages - 1);
            root = root.child(self.render_session_trash_pagination(page, pages, active, cx));
            for (index, item) in items
                .iter()
                .enumerate()
                .skip(page * TRASH_PAGE_SIZE)
                .take(TRASH_PAGE_SIZE)
            {
                root = root.child(self.render_session_trash_row(index, item.clone(), active, cx));
            }
        }
        root.into_any_element()
    }
    // Optional builders need separate frames on the native 1 MiB UI stack.
    fn render_session_trash_pagination(
        &self,
        page: usize,
        pages: usize,
        active: bool,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        div()
            .flex()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(semantics::button(
                Button::new("session-trash-prev")
                    .label(text(cx, "Previous", "上一页"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.session_ops.trash_page = this.session_ops.trash_page.saturating_sub(1);
                        cx.notify();
                    })),
                active || page == 0,
                false,
                cx,
            ))
            .child(
                div()
                    .id("session-trash-page")
                    .test_support()
                    .child(format!("{} / {pages}", page + 1)),
            )
            .child(semantics::button(
                Button::new("session-trash-next")
                    .label(text(cx, "Next", "下一页"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.session_ops.trash_page += 1;
                        cx.notify();
                    })),
                active || page + 1 >= pages,
                false,
                cx,
            ))
            .into_any_element()
    }
    fn render_session_trash_row(
        &self,
        index: usize,
        item: crate::runtime::insights::session_operations::TrashItem,
        active: bool,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let title = item.title.clone();
        let detail = format!("{} · {}", item.session_id, item.deleted_at);
        div()
            .id(SharedString::from(format!(
                "session-trash-row:{}:{}:{}",
                item.session_id, item.deleted_at, item.relative_path
            )))
            .test_support()
            .h(px(theme::ROW_HEIGHT * 2.))
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .id("title")
                            .text_ellipsis()
                            .child(title.clone())
                            .tooltip(move |window, cx| {
                                Tooltip::new(title.clone()).build(window, cx)
                            }),
                    )
                    .child(
                        div()
                            .id("metadata")
                            .text_size(px(theme::META_SIZE))
                            .font_family(cx.theme().mono_font_family.clone())
                            .text_ellipsis()
                            .child(detail.clone())
                            .tooltip(move |window, cx| {
                                Tooltip::new(detail.clone()).build(window, cx)
                            }),
                    ),
            )
            .child(semantics::button(
                Button::new(SharedString::from(format!("session-restore:{index}")))
                    .label(text(cx, "Review restore…", "确认恢复…"))
                    .ghost()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.prepare_session_restore(item.clone(), cx);
                    })),
                active,
                false,
                cx,
            ))
            .into_any_element()
    }
    pub(super) fn render_session_restore_confirmation(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let Some(prepared) = self.session_ops.restore.clone() else {
            return div().into_any_element();
        };
        let item = &prepared.item;
        let focus = self
            .session_ops
            .focus
            .get_or_insert_with(|| cx.focus_handle().tab_stop(false))
            .clone();
        if std::mem::take(&mut self.session_ops.entry_pending) {
            self.session_ops.restore_return_focus = window.focused(cx);
            let decision = self.session_ops.decision_generation;
            let weak = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    if this.session_ops.decision_generation == decision
                        && this.session_ops.restore.is_some()
                        && let Some(focus) = &this.session_ops.focus
                    {
                        window.focus(focus, cx);
                        window.focus_next(cx);
                    }
                });
            });
        }
        div().id("session-restore-confirmation").test_support().role(gpui_kit::Role::AlertDialog).track_focus(&focus).tab_stop(false).flex().flex_col().gap(px(theme::CONTROL_GAP))
            .on_key_down(cx.listener(|this,event:&gpui_kit::KeyDownEvent,window,cx|{if event.keystroke.key=="escape" {this.dismiss_session_restore(window,cx);cx.stop_propagation();}}))
            .child(text(cx,"Restore this reviewed trash entry? A changed trash payload or an existing active session stops restoration.","恢复此已核对的废纸篓条目？副本变化或活跃会话已存在时会停止恢复。"))
            .child(format!("{} · {}",item.title,item.session_id)).child(item.relative_path.clone()).child(item.codex_home.display().to_string())
            .child(div().tab_group().tab_stop(false)
                .child(semantics::button(Button::new("session-restore-cancel").label(text(cx,"Cancel","取消")).ghost().on_click(cx.listener(|this,_,window,cx|this.dismiss_session_restore(window,cx))),false,false,cx))
                .child(semantics::button(Button::new("session-restore-confirm").label(text(cx,"Restore session","恢复会话")).on_click(cx.listener(|this,_,window,cx|{
                    if let Some(item)=this.session_ops.restore.take(){this.session_ops.error=this.requests.start_session_restore(this.session_key(),item).err();this.session_ops.trash=None;this.observe_session_operations(cx);}this.dismiss_session_restore(window,cx);
                })),false,false,cx)).focus_trap("session-restore-trap",&focus)).into_any_element()
    }
    fn dismiss_session_restore(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.session_ops.restore = None;
        self.session_ops.decision_generation += 1;
        if let Some(focus) = self.session_ops.restore_return_focus.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }
}
