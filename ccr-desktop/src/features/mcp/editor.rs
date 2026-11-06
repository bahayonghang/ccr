//! Scoped patch editor: stored credentials never enter editable controls.
use crate::{
    locales::{service_error, text},
    runtime::{
        Services,
        mcp::{
            McpCopySession, McpEditSession, McpEntry, McpError, McpScope, McpSource, McpTarget,
            preset_patch,
        },
    },
    shell::semantics,
    theme,
};
use ccr_app::{
    mcp_document::EntryMutation,
    runtime::{Completion, RequestKey, TaskEvent},
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    FocusTrapElement,
    button::{Button, ButtonVariants},
    input::{Input, InputState, Textarea, TextareaState},
};
use gpui_kit::{
    AppContext, Context, Entity, EventEmitter, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, Task, Window, div, px,
};
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq)]
pub(super) enum EditMode {
    Add,
    Edit,
    Delete,
    Preset(String),
    Copy(McpEntry),
}

impl EditMode {
    fn creates(&self) -> bool {
        matches!(self, Self::Add | Self::Preset(_) | Self::Copy(_))
    }
}

pub(super) enum EditorEvent {
    Saved,
    Closed,
}

pub(super) struct McpEditor {
    services: Services,
    key: RequestKey,
    mode: EditMode,
    title: String,
    name: Entity<InputState>,
    patch: Entity<TextareaState>,
    session: Option<Arc<McpEditSession>>,
    copy_session: Option<Arc<McpCopySession>>,
    required_env: Option<String>,
    task: Option<Task<()>>,
    busy: bool,
    message: Option<&'static str>,
    pending: Option<(String, Option<EntryMutation>)>,
    focus: FocusHandle,
    confirmation_focus: FocusHandle,
    enter_confirmation: bool,
}
impl EventEmitter<EditorEvent> for McpEditor {}

impl McpEditor {
    pub fn new(
        services: Services,
        key: RequestKey,
        target: McpTarget,
        mode: EditMode,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (source, scope) = match &target {
            McpTarget::ClaudeUser => (McpSource::ClaudeState, None),
            McpTarget::ClaudeLocal(path) => (McpSource::ClaudeState, Some(path)),
            McpTarget::ClaudeProject(path) => (McpSource::ProjectFile, Some(path)),
            McpTarget::Codex => (McpSource::CodexConfig, None),
            McpTarget::Antigravity => (McpSource::AntigravityConfig, None),
        };
        let (en, zh) = super::source_title(source);
        let scope = scope
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| text(cx, "User", "用户").to_string());
        let mut title = format!("{} · {scope}", text(cx, en, zh));
        if !name.is_empty() {
            title.push_str(&format!(" · {name}"));
        }
        let name_input = cx.new(|cx| InputState::new(window, cx));
        name_input.update(cx, |state, cx| {
            state.set_value(name.to_string(), window, cx)
        });
        let patch = cx.new(|cx| TextareaState::new(window, cx));
        let initial_patch = match &mode {
            EditMode::Preset(id) => preset_patch(id, &target),
            _ => Ok(serde_json::json!({})),
        };
        let required_env = match &mode {
            EditMode::Preset(id) => ccr_skills::get_builtin_presets()
                .into_iter()
                .find(|preset| preset.id == *id)
                .and_then(|preset| preset.api_key_env),
            _ => None,
        };
        if let Ok(value) = &initial_patch
            && let Ok(value) = serde_json::to_string_pretty(value)
        {
            patch.update(cx, |state, cx| state.set_value(value, window, cx));
        }
        if let EditMode::Copy(source) = &mode {
            let (en, zh) = super::source_title(source.source);
            let scope = match &source.scope {
                McpScope::User => text(cx, "User", "用户").to_string(),
                McpScope::Project(path) => path.clone(),
            };
            title = format!("{} · {scope} · {} → {title}", text(cx, en, zh), source.name);
        }
        let mut this = Self {
            services,
            key,
            mode,
            title,
            name: name_input,
            patch,
            session: None,
            copy_session: None,
            required_env,
            task: None,
            busy: true,
            message: None,
            pending: None,
            focus: cx.focus_handle().tab_stop(false),
            confirmation_focus: cx.focus_handle(),
            enter_confirmation: false,
        };
        if let Err(error) = initial_patch {
            this.busy = false;
            this.message = Some(error.message());
            return this;
        }
        if let EditMode::Copy(source) = &this.mode {
            match this
                .services
                .mcp_copy_prepare_request(this.key.clone(), source.clone(), target)
            {
                Ok(mut subscription) => {
                    this.task = Some(cx.spawn(async move |weak, cx| {
                        while let Some(event) = subscription.next().await {
                            if let TaskEvent::Terminal(result) = event {
                                let _ = weak.update(cx, |this, cx| {
                                    this.busy = false;
                                    match result.as_ref() {
                                        Completion::Success(session) => {
                                            this.copy_session = Some(session.clone())
                                        }
                                        Completion::Failed(error)
                                        | Completion::CleanupFailed(error) => {
                                            this.message = Some(error.message())
                                        }
                                        _ => {
                                            this.message = Some(text(
                                                cx,
                                                "MCP copy preparation did not complete.",
                                                "MCP 复制准备未完成。",
                                            ))
                                        }
                                    }
                                    cx.notify();
                                });
                                break;
                            }
                        }
                    }));
                }
                Err(_) => {
                    this.busy = false;
                    this.message = Some(text(
                        cx,
                        "Queue unavailable. Retry.",
                        "队列不可用，请重试。",
                    ));
                }
            }
            return this;
        }
        match this.services.mcp_edit_request(this.key.clone(), target) {
            Ok(mut subscription) => {
                this.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                this.busy = false;
                                match result.as_ref() {
                                    Completion::Success(session) => this.session = Some(session.clone()),
                                    Completion::Failed(error) | Completion::CleanupFailed(error) => this.message = Some(error.message()),
                                    _ => this.message = Some("MCP source read did not complete. / MCP 来源读取未完成"),
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => {
                this.busy = false;
                this.message = Some("Queue unavailable. Retry. / 队列不可用，请重试");
            }
        }
        this
    }

    fn prepare(&mut self, cx: &mut Context<Self>) {
        if self.busy || (self.session.is_none() && self.copy_session.is_none()) {
            return;
        }
        let name = self.name.read(cx).value().to_string();
        if ccr_app::mcp_document::validate_name(&name).is_err() {
            self.message = Some(
                "Enter a nonempty name without surrounding whitespace. / 请输入不含首尾空白的非空名称",
            );
            cx.notify();
            return;
        }
        let mutation = if matches!(self.mode, EditMode::Copy(_)) {
            None
        } else if self.mode == EditMode::Delete {
            Some(EntryMutation::Delete)
        } else {
            let value: serde_json::Value = match serde_json::from_str::<serde_json::Value>(
                self.patch.read(cx).value().as_ref(),
            ) {
                Ok(value) if value.is_object() => value,
                _ => {
                    self.message = Some(McpError::Invalid.message());
                    cx.notify();
                    return;
                }
            };
            if self.required_env.as_ref().is_some_and(|key| {
                value
                    .get("env")
                    .and_then(|env| env.get(key))
                    .and_then(serde_json::Value::as_str)
                    .is_none_or(|value| value.trim().is_empty())
            }) {
                self.message = Some(text(
                    cx,
                    "Fill in the preset's required API key before saving.",
                    "保存前请填写预设需要的 API 密钥。",
                ));
                cx.notify();
                return;
            }
            if self.mode.creates() {
                Some(EntryMutation::Add(value))
            } else {
                Some(EntryMutation::Patch(value))
            }
        };
        self.pending = Some((name, mutation));
        self.enter_confirmation = true;
        self.message = None;
        cx.notify();
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let Some((name, mutation)) = self.pending.take() else {
            return;
        };
        self.key.request_id = self.key.request_id.saturating_add(1);
        self.busy = true;
        let request = if let Some(session) = self.copy_session.clone() {
            self.services
                .mcp_copy_save_request(self.key.clone(), session, name)
        } else if let (Some(session), Some(mutation)) = (self.session.clone(), mutation) {
            self.services
                .mcp_mutate_request(self.key.clone(), session, name, mutation)
        } else {
            self.busy = false;
            return;
        };
        match request {
            Ok(mut subscription) => {
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                this.busy = false;
                                match result.as_ref() {
                                    Completion::Success(()) => cx.emit(EditorEvent::Saved),
                                    Completion::Failed(error) | Completion::CleanupFailed(error) => this.message = Some(error.message()),
                                    _ => this.message = Some("MCP write did not complete. Your draft is retained. / MCP 写入未完成，草稿已保留"),
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => {
                self.busy = false;
                self.message = Some("Queue unavailable. Retry. / 队列不可用，请重试");
            }
        }
        cx.notify();
    }
}

impl Render for McpEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div().id("mcp-editor").test_support().track_focus(&self.focus).tab_stop(false).flex().flex_col().gap(px(theme::CONTROL_GAP))
            .child(self.title.clone())
            .child(text(cx, "The selected source and scope stay fixed until this editor closes. Existing secret values are not loaded. Only submitted fields change; omitted fields are retained. Leaving discards the draft; a save already started continues.", "编辑器关闭前，所选来源与作用域保持固定。已有密钥不会载入。仅修改提交的字段，省略字段保持不变。离开会丢弃草稿；已开始的保存会继续。"))
            .child(Input::new(&self.name).id("mcp-name").aria_label(text(cx,"Server name","服务器名称")).disabled(!self.mode.creates() || self.busy || self.pending.is_some()));
        if matches!(self.mode, EditMode::Copy(_)) {
            root = root.child(text(cx,"Copy the selected server, including stored environment values and supported HTTP headers, into this source. Stored values stay hidden. Existing entries are never overwritten; use a different name if needed.","将所选服务器连同已保存的环境变量与支持的 HTTP 请求头复制到此来源。已保存的值保持隐藏。不会覆盖已有条目；如有重名，请使用其他名称。"));
        } else if self.mode != EditMode::Delete {
            root = root.child(text(cx, "JSON patch. A new server requires command or url. Enter new credentials only when replacing them. null removes a field; Claude uses its existing patch rules.", "JSON 补丁。新服务器必须填写 command 或 url；仅在替换时输入新密钥。null 删除字段；Claude 沿用既有补丁规则。"))
                .child(Textarea::new(&self.patch).accessibility_id("mcp-patch").aria_label(text(cx,"MCP JSON patch","MCP JSON 补丁")).disabled(self.busy || self.pending.is_some()).h(px(180.)));
        }
        if let Some(message) = self.message {
            root = root.child(
                div()
                    .id("mcp-editor-message")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(service_error(cx, message))
                    .child(service_error(cx, message)),
            );
        }
        if self.pending.is_some() {
            if self.enter_confirmation {
                window.focus(&self.confirmation_focus, cx);
                self.enter_confirmation = false;
            }
            root = root.child(div().id("mcp-confirmation").test_support().on_key_down(cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| { if event.keystroke.key == "escape" { this.pending = None; window.focus(&this.focus, cx); cx.stop_propagation(); cx.notify(); } })).flex().flex_col().gap(px(theme::CONTROL_GAP))
                .child(text(cx,"Commit this change to the selected MCP source? Removing or changing a server can affect clients using it. No server process will be started.","将更改写入所选 MCP 来源？删除或修改服务器可能影响使用它的客户端。此操作不会启动服务器进程。"))
                .child(div().flex().gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(Button::new("mcp-confirm-cancel").label(text(cx,"Keep editing","继续编辑")).ghost().on_click(cx.listener(|this,_,window,cx|{this.pending=None;window.focus(&this.focus,cx);cx.notify();})),false,false,cx))
                    .child(semantics::button(Button::new("mcp-confirm-save").label(text(cx,"Confirm change","确认更改")).primary().on_click(cx.listener(|this,_,_,cx|this.save(cx))),false,false,cx)).focus_trap("mcp-confirmation-trap", &self.confirmation_focus)));
        } else {
            root = root.child(
                div()
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("mcp-editor-close")
                            .label(text(cx, "Discard draft and close", "丢弃草稿并关闭"))
                            .ghost()
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(EditorEvent::Closed))),
                        false,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("mcp-editor-save")
                            .label(if self.mode == EditMode::Delete {
                                text(cx, "Delete server", "删除服务器")
                            } else {
                                text(cx, "Save changes", "保存更改")
                            })
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| this.prepare(cx))),
                        self.busy || (self.session.is_none() && self.copy_session.is_none()),
                        self.busy,
                        cx,
                    )),
            );
        }
        root
    }
}
