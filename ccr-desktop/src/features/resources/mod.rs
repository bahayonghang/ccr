//! Existing fixed instruction files, with a scoped plaintext edit session.
use crate::{
    locales::text,
    runtime::{
        Services,
        resources::{PromptRow, PromptSession, ResourceCommand, ResourceResult},
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    FocusTrapElement,
    button::{Button, ButtonVariants},
    input::{Textarea, TextareaState},
};
use gpui_kit::{
    AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, Task, Window, div, px,
};
use std::sync::Arc;

#[derive(Clone, Copy)]
enum Decision {
    Open(&'static str),
    Reload,
}

pub struct ResourcesView {
    services: Services,
    platform: Platform,
    rows: Vec<PromptRow>,
    session: Option<Arc<PromptSession>>,
    editor: Entity<TextareaState>,
    pending_content: Option<String>,
    editor_entry_pending: bool,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    busy: bool,
    message: Option<&'static str>,
    decision: Option<Decision>,
    decision_generation: u64,
    decision_entry_pending: bool,
    confirmation_focus: FocusHandle,
    focus: FocusHandle,
    decision_return: Option<FocusHandle>,
    editor_return: Option<FocusHandle>,
}
impl ResourcesView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            services,
            platform: Platform::Claude,
            rows: Vec::new(),
            session: None,
            editor: cx.new(|cx| TextareaState::new(window, cx)),
            pending_content: None,
            editor_entry_pending: false,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            busy: false,
            message: None,
            decision: None,
            decision_generation: 0,
            decision_entry_pending: false,
            confirmation_focus: cx.focus_handle().tab_stop(false),
            focus: cx.focus_handle().tab_stop(false),
            decision_return: None,
            editor_return: None,
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.session.is_some() {
            window.focus(&self.editor.read(cx).focus_handle(cx), cx);
        } else {
            window.focus(&self.focus, cx);
            window.focus_next(cx);
        }
    }
    pub fn load(&mut self, cx: &mut Context<Self>) {
        self.request(ResourceCommand::ListPrompts, cx);
    }
    pub fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.generation += 1;
        self.session = None;
        self.pending_content = None;
        self.editor_entry_pending = false;
        self.decision = None;
        self.decision_entry_pending = false;
        self.decision_generation += 1;
        self.decision_return = None;
        self.editor_return = None;
        self.busy = false;
        self.message = None;
        self.editor
            .update(cx, |state, cx| state.set_value("", window, cx));
        cx.notify();
    }
    fn close_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let target = self.editor_return.take();
        self.leave(window, cx);
        if let Some(target) = target {
            window.focus(&target, cx);
        }
        self.load(cx);
    }
    fn request(&mut self, command: ResourceCommand, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.generation += 1;
        let key = RequestKey {
            environment: EnvironmentId::Local(
                if self.services.is_fixture() {
                    "fixture"
                } else {
                    "local"
                }
                .into(),
            ),
            platform: self.platform,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        self.busy = true;
        self.message = None;
        match self.services.resource_request(key.clone(), command) {
            Ok(mut subscription) => self.task = Some(cx.spawn(async move |weak, cx| {
                while let Some(event) = subscription.next().await {
                    if let TaskEvent::Terminal(result) = event {
                        let _ = weak.update(cx, |this, cx| {
                            if !this.gate.accepts(&key) { return; }
                            this.busy = false;
                            match result.as_ref() {
                                Completion::Success(ResourceResult::Prompts(rows)) => this.rows = rows.clone(),
                                Completion::Success(ResourceResult::Opened(session)) => {
                                    this.pending_content = Some(session.snapshot.content.clone());
                                    this.editor_entry_pending = true;
                                    this.session = Some(session.clone());
                                }
                                Completion::Success(ResourceResult::Saved { conflict: true, .. }) => {
                                    this.message = Some("File changed. Draft and original revision retained. Discard and reload before retrying. / 文件已变化，保留草稿及原始版本，请丢弃并重读。");
                                }
                                Completion::Success(ResourceResult::Saved { size_warning, .. }) => {
                                    this.session = None;
                                    this.rows.clear();
                                    this.pending_content = Some(String::new());
                                    this.message = Some(if *size_warning {
                                        "Saved. This file exceeds64KiB; reload to review its size warning. / 已保存。文件超过64KiB，请重读并留意大小警告。"
                                    } else { "Saved. Reload to begin a new edit session. / 已保存，请重读后开始新编辑。" });
                                }
                                Completion::Failed(error) | Completion::CleanupFailed(error) => this.message = Some(error.message()),
                                Completion::Cancelled => this.message = Some("Operation did not start. / 操作尚未开始。"),
                                Completion::WorkerFailed => this.message = Some("Worker stopped without a result. Reload before retrying. / 后台未返回结果，请重读后重试。"),
                            }
                            cx.notify();
                        });
                        break;
                    }
                }
            })),
            Err(_) => { self.busy = false; self.message = Some("Queue unavailable. Retry. / 操作队列不可用，请重试。"); }
        }
        cx.notify();
    }
    fn decide(&mut self, decision: Decision, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.decision.is_some() {
            return;
        }
        self.decision_return = window.focused(cx);
        if matches!(decision, Decision::Open(_)) && self.session.is_none() {
            self.editor_return = self.decision_return.clone();
        }
        self.decision = Some(decision);
        self.decision_generation += 1;
        self.decision_entry_pending = true;
        cx.notify();
    }
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.decision = None;
        self.decision_generation += 1;
        self.decision_entry_pending = false;
        if let Some(target) = self.decision_return.take() {
            window.focus(&target, cx);
        }
        cx.notify();
    }
    fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(decision) = self.decision else {
            return;
        };
        let id = match decision {
            Decision::Open(id) => id,
            Decision::Reload => {
                let Some(session) = self.session.as_ref() else {
                    self.dismiss(window, cx);
                    return;
                };
                session.spec.id
            }
        };
        self.dismiss(window, cx);
        self.session = None;
        self.editor
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.pending_content = None;
        self.request(ResourceCommand::OpenPrompt(id), cx);
    }
}
impl Render for ResourcesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(content) = self.pending_content.take() {
            self.editor
                .update(cx, |state, cx| state.set_value(content, window, cx));
        }
        if std::mem::take(&mut self.editor_entry_pending) {
            let generation = self.generation;
            let view = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = view.update(cx, |this, cx| {
                    if this.session.is_some()
                        && this.generation == generation
                        && this.decision.is_none()
                    {
                        this.focus_main(window, cx);
                    }
                });
            });
        }
        let blocked = self.busy || self.decision.is_some();
        let mut root = div().id("resources-view").test_support().track_focus(&self.focus).tab_stop(false).flex().flex_col().size_full()
            .p(px(theme::CONTENT_PADDING)).gap(px(theme::SECTION_GAP))
            .on_key_down(cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" && this.decision.is_some() { this.dismiss(window, cx); cx.stop_propagation(); }
            }))
            .child(div().text_size(px(theme::TITLE_SIZE)).child(text(cx, "User instruction files", "用户指令文件")))
            .child(text(cx, "Fixed user files, original revision and managed backup. Remote environments do not fall back to Local.", "固定用户文件、原始版本与托管备份。远程环境不会回退本地。"));
        let mut platforms = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        for (platform, name) in [
            (Platform::Claude, "Claude Code"),
            (Platform::Codex, "Codex"),
            (Platform::Gemini, "Antigravity"),
            (Platform::OpenCode, "OpenCode"),
        ] {
            platforms = platforms.child(semantics::button(
                Button::new(format!("resource-platform-{}", platform.short_name()))
                    .label(name)
                    .ghost()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.leave(window, cx);
                        this.platform = platform;
                        this.load(cx);
                    })),
                blocked || self.session.is_some(),
                false,
                cx,
            ));
        }
        root = root.child(platforms);
        if let Some(decision) = self.decision {
            if std::mem::take(&mut self.decision_entry_pending) {
                let generation = self.decision_generation;
                let view = cx.weak_entity();
                window.on_next_frame(move |window, cx| {
                    let _ = view.update(cx, |this, cx| {
                        if this.decision.is_some() && this.decision_generation == generation {
                            window.focus(&this.confirmation_focus, cx);
                            window.focus_next(cx);
                        }
                    });
                });
            }
            root = root.child(div().id("resource-confirmation").test_support().role(gpui_kit::Role::AlertDialog)
                .aria_label("Confirm scoped resource editor")
                .child(match decision {
                    Decision::Open(_) => "Open plaintext instruction content? It remains in this scoped editor. Do not include private text in screenshots or logs. / 打开明文指令内容？仅保留在临时编辑会话，请勿将私密文本记录或截图。",
                    Decision::Reload => "Discard this draft and reload the current file? / 丢弃当前草稿并重新读取文件？",
                }))
                .child(div().tab_group().tab_stop(false).flex().gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(Button::new("resource-decision-cancel").label("Cancel / 取消").ghost()
                        .on_click(cx.listener(|this, _, window, cx| this.dismiss(window, cx))), false, false, cx))
                    .child(semantics::button(Button::new("resource-decision-confirm").label("Confirm / 确认")
                        .on_click(cx.listener(|this, _, window, cx| this.confirm(window, cx))), false, false, cx))
                    .focus_trap("resource-decision-trap", &self.confirmation_focus));
        }
        if let Some(session) = self.session.clone() {
            root = root
                .child(
                    div()
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new("resource-editor-close")
                                .label("Cancel / back / 取消返回")
                                .ghost()
                                .on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.close_editor(window, cx)
                                    }),
                                ),
                            blocked,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("resource-reload")
                                .label("Discard and reload / 丢弃并重读")
                                .ghost()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.decide(Decision::Reload, window, cx)
                                })),
                            blocked,
                            false,
                            cx,
                        )),
                )
                .child(format!(
                    "{} · {}",
                    session.spec.filename,
                    if session.snapshot.exists {
                        "Existing / 已存在"
                    } else {
                        "Missing / 尚未创建"
                    }
                ))
                .child(
                    Textarea::new(&self.editor)
                        .accessibility_id("resource-content")
                        .aria_label("Scoped instruction file / 临时指令文件")
                        .flex_1()
                        .readonly(blocked),
                )
                .child(semantics::button(
                    Button::new("resource-save")
                        .label("Save original revision / 按原始版本保存")
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(session) = this.session.clone() {
                                this.request(
                                    ResourceCommand::SavePrompt {
                                        session,
                                        content: this.editor.read(cx).value().to_string(),
                                    },
                                    cx,
                                );
                            }
                        })),
                    blocked,
                    self.busy,
                    cx,
                ));
            if !session.snapshot.exists {
                root = root.child(semantics::button(
                    Button::new("resource-create")
                        .label("Create empty file / 创建空文件")
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(session) = this.session.clone() {
                                this.request(ResourceCommand::CreatePrompt { session }, cx);
                            }
                        })),
                    blocked,
                    false,
                    cx,
                ));
            }
            if session.spec.platform == "codex" {
                root = root.child("Codex reads at most32KiB of user instructions. / Codex 用户指令读取上限为32KiB。");
            }
        } else {
            root = root.child(semantics::button(
                Button::new("resources-refresh")
                    .label("Refresh / 刷新")
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| this.load(cx))),
                blocked,
                self.busy,
                cx,
            ));
            for row in &self.rows {
                let id = row.id;
                root = root.child(
                    div()
                        .h(px(theme::ROW_HEIGHT))
                        .flex()
                        .items_center()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new(format!("resource-open-{id}"))
                                .label(format!(
                                    "{} · {}",
                                    row.filename,
                                    if row.exists {
                                        "Existing / 已存在"
                                    } else {
                                        "Missing / 尚未创建"
                                    }
                                ))
                                .ghost()
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.decide(Decision::Open(id), window, cx)
                                })),
                            blocked,
                            false,
                            cx,
                        ))
                        .child(
                            row.size
                                .map(|size| format!("{size} bytes"))
                                .unwrap_or_default(),
                        ),
                );
            }
        }
        if self.busy {
            root = root.child("Working on background worker / 正在后台处理");
        }
        if let Some(message) = self.message {
            root = root.child(
                div()
                    .id("resources-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .child(message),
            );
        }
        root
    }
}

#[cfg(test)]
mod tests;
