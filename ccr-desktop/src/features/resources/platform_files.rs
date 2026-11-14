//! OpenCode Markdown and Antigravity TOML directory resources.
use crate::{
    locales::text,
    runtime::{
        Services,
        resources::platform_files::{
            self as runtime, Error, Kind, OperationState, Read, ReadResult, Row, Scope, Session,
        },
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, FocusTrapElement,
    button::{Button, ButtonVariants},
    input::{Input, InputState, Textarea, TextareaState},
};
use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, Subscription, Task, Window, div, px,
};
use std::{path::PathBuf, sync::Arc};
const PAGE_SIZE: usize = 40;
enum Intent {
    Save(String),
    Delete,
    Discard,
}
pub struct PlatformFilesView {
    services: Services,
    platform: Platform,
    kind: Kind,
    scope: Scope,
    project: Entity<InputState>,
    name: Entity<InputState>,
    filter: Entity<InputState>,
    _filter_observation: Subscription,
    rows: Vec<Row>,
    page: usize,
    session: Option<Arc<Session>>,
    editor: Entity<TextareaState>,
    pending_text: Option<String>,
    intent: Option<Intent>,
    enter_confirmation: bool,
    busy: bool,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    operation: Arc<OperationState>,
    observation: Option<Task<()>>,
    submitted: bool,
    message: Option<Error>,
    focus: FocusHandle,
    confirmation_focus: FocusHandle,
}
impl PlatformFilesView {
    pub fn new(
        services: Services,
        kind: Kind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let filter = cx.new(|cx| InputState::new(window, cx));
        let observation = cx.observe(&filter, |this, _, cx| {
            this.page = 0;
            cx.notify();
        });
        Self {
            operation: services.platform_files_state(),
            services,
            platform: runtime::platform(kind),
            kind,
            scope: Scope::User,
            project: cx.new(|cx| InputState::new(window, cx)),
            name: cx.new(|cx| InputState::new(window, cx)),
            filter,
            _filter_observation: observation,
            rows: Vec::new(),
            page: 0,
            session: None,
            editor: cx.new(|cx| TextareaState::new(window, cx)),
            pending_text: None,
            intent: None,
            enter_confirmation: false,
            busy: false,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            observation: None,
            submitted: false,
            message: None,
            focus: cx.focus_handle().tab_stop(false),
            confirmation_focus: cx.focus_handle(),
        }
    }
    pub fn blocks_switch(&self) -> bool {
        self.busy || self.operation.busy || self.session.is_some() || self.intent.is_some()
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        window.focus_next(cx);
    }
    pub fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.task = None;
        self.observation = None;
        self.gate.clear();
        self.generation += 1;
        self.session = None;
        self.intent = None;
        self.pending_text = None;
        self.submitted = false;
        self.enter_confirmation = false;
        self.rows.clear();
        self.busy = false;
        self.message = None;
        self.editor
            .update(cx, |state, cx| state.set_value("", window, cx));
        cx.notify();
    }
    pub fn load(&mut self, cx: &mut Context<Self>) {
        self.observe(cx);
        if self.session.is_none() {
            self.request(
                Read::List {
                    kind: self.kind,
                    scope: self.scope.clone(),
                },
                cx,
            );
        }
    }
    fn key(&mut self) -> RequestKey {
        self.generation += 1;
        RequestKey {
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
        }
    }
    fn accept_operation(&mut self, state: Arc<OperationState>) -> bool {
        let done = self.submitted && !state.busy && state.result.is_some();
        let saved = done && state.result.is_some_and(|r| r.is_ok());
        if done {
            self.submitted = false;
            if saved {
                self.session = None;
                self.pending_text = Some(String::new());
            }
        }
        self.operation = state;
        saved
    }
    fn observe(&mut self, cx: &mut Context<Self>) {
        if self.observation.is_some() {
            return;
        }
        let mut updates = self.services.observe_platform_files();
        self.accept_operation(updates.borrow_and_update().clone());
        self.observation = Some(cx.spawn(async move |weak, cx| {
            while updates.changed().await.is_ok() {
                let state = updates.borrow_and_update().clone();
                if weak
                    .update(cx, |this, cx| {
                        let saved = this.accept_operation(state);
                        if saved {
                            this.load(cx);
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
    fn request(&mut self, request: Read, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let key = self.key();
        self.gate.replace(key.clone());
        self.busy = true;
        self.message = None;
        match self.services.platform_files_read(key.clone(), request) {
            Ok(mut sub) => {
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = sub.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) {
                                    return;
                                }
                                this.busy = false;
                                match result.as_ref() {
                                    Completion::Success(ReadResult::Listed(rows)) => {
                                        this.rows = rows.clone()
                                    }
                                    Completion::Success(ReadResult::Opened(session)) => {
                                        this.pending_text = Some(session.initial_text());
                                        this.session = Some(session.clone());
                                    }
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => {
                                        this.message = Some(*error)
                                    }
                                    _ => this.message = Some(Error::Interrupted),
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }))
            }
            Err(_) => {
                self.busy = false;
                self.message = Some(Error::Busy);
            }
        }
        cx.notify();
    }
    fn open(&mut self, name: String, creating: bool, cx: &mut Context<Self>) {
        if self.blocks_switch() {
            return;
        }
        self.request(
            Read::Open {
                kind: self.kind,
                scope: self.scope.clone(),
                name,
                creating,
            },
            cx,
        );
    }
    fn prepare(&mut self, intent: Intent, cx: &mut Context<Self>) {
        if self.busy || self.operation.busy || self.session.is_none() {
            return;
        }
        self.intent = Some(intent);
        self.enter_confirmation = true;
        cx.notify();
    }
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.intent = None;
        self.enter_confirmation = false;
        window.focus(&self.editor.read(cx).focus_handle(cx), cx);
        cx.notify();
    }
    fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(intent) = self.intent.take() else {
            return;
        };
        self.enter_confirmation = false;
        if matches!(intent, Intent::Discard) {
            self.session = None;
            self.editor.update(cx, |s, cx| s.set_value("", window, cx));
            self.load(cx);
            return;
        }
        let Some(session) = self.session.clone() else {
            return;
        };
        let content = match intent {
            Intent::Save(value) => Some(value),
            _ => None,
        };
        let key = self.key();
        match self.services.commit_platform_file(key, session, content) {
            Ok(()) => {
                self.submitted = true;
                self.operation = self.services.platform_files_state();
            }
            Err(error) => self.message = Some(error),
        }
        self.observe(cx);
        cx.notify();
    }
}
fn title(kind: Kind, cx: &App) -> &'static str {
    match kind {
        Kind::OpenCodeAgent => text(cx, "Agents", "代理配置"),
        Kind::OpenCodeCommand => text(cx, "Commands", "命令"),
        Kind::OpenCodePlugin => text(cx, "Local plugins", "本地插件"),
        Kind::AntigravityCommand => text(cx, "Slash commands", "斜杠命令"),
        Kind::Extension => text(cx, "Extensions", "扩展"),
    }
}
fn error_text(error: Error, cx: &App) -> &'static str {
    match error {
        Error::Unsupported => text(
            cx,
            "This resource or scope is unavailable. Extensions are read-only and Local only.",
            "此资源或作用域不可用。扩展只读，仅支持本地环境。",
        ),
        Error::Invalid => text(
            cx,
            "Check the file name and format. Markdown needs a description; command TOML needs a prompt.",
            "请检查文件名与格式。Markdown 需要 description，命令 TOML 需要 prompt。",
        ),
        Error::Conflict => text(
            cx,
            "The source changed elsewhere. Your original version and draft remain. Discard and reopen before retrying.",
            "来源已被其他客户端修改，原始版本与草稿已保留。请丢弃并重新打开后重试。",
        ),
        Error::UnsafePath => text(
            cx,
            "Linked or out-of-scope paths are unsupported.",
            "不支持链接路径或范围外的路径。",
        ),
        Error::TooLarge => text(
            cx,
            "The file or directory exceeds the editor limits.",
            "文件或目录超过编辑器限制。",
        ),
        Error::Missing => text(
            cx,
            "This file is missing. An extension without a manifest can only be listed.",
            "文件不存在。没有 manifest 的扩展仅可查看清单。",
        ),
        Error::Duplicate => text(
            cx,
            "A file with this name already exists.",
            "同名文件已存在。",
        ),
        Error::Busy => text(
            cx,
            "The resource worker is busy or unavailable.",
            "资源执行器忙碌或不可用。",
        ),
        Error::Interrupted => text(
            cx,
            "The write did not report completion. Reopen the source to check its state.",
            "写入未报告完成，请重新打开来源确认状态。",
        ),
        Error::Storage => text(
            cx,
            "The resource could not be read or written. Your draft remains.",
            "无法读取或写入资源，草稿已保留。",
        ),
    }
}
impl Render for PlatformFilesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(value) = self.pending_text.take() {
            self.editor
                .update(cx, |s, cx| s.set_value(value, window, cx));
        }
        let locked = self.blocks_switch();
        let operation_visible = self.operation.platform == Some(self.platform)
            && self.operation.environment
                == Some(EnvironmentId::Local(
                    if self.services.is_fixture() {
                        "fixture"
                    } else {
                        "local"
                    }
                    .into(),
                ));
        let mut root = div()
            .id("platform-files")
            .test_support()
            .track_focus(&self.focus)
            .tab_stop(false)
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scroll()
            .gap(px(theme::CONTROL_GAP));
        let kinds = if self.platform == Platform::OpenCode {
            vec![
                Kind::OpenCodeAgent,
                Kind::OpenCodeCommand,
                Kind::OpenCodePlugin,
            ]
        } else {
            vec![Kind::AntigravityCommand, Kind::Extension]
        };
        root = root.child(
            div()
                .flex()
                .flex_shrink_0()
                .gap(px(theme::CONTROL_GAP))
                .children(kinds.into_iter().enumerate().map(|(index, kind)| {
                    semantics::button(
                        Button::new(("platform-file-kind", index))
                            .label(title(kind, cx))
                            .ghost()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.kind = kind;
                                if kind == Kind::Extension {
                                    this.scope = Scope::User;
                                }
                                this.page = 0;
                                this.rows.clear();
                                this.load(cx);
                            })),
                        locked,
                        false,
                        cx,
                    )
                })),
        );
        root=root.child(div().flex_shrink_0().child(match self.kind{Kind::OpenCodeAgent|Kind::OpenCodeCommand=>text(cx,"Global OpenCode resources use ~/.config/opencode. Project resources use .opencode. Full Markdown preserves frontmatter and body.","OpenCode 全局资源位于 ~/.config/opencode，项目资源位于 .opencode。完整 Markdown 保留前置元数据与正文。"),Kind::OpenCodePlugin=>text(cx,"Local plugin names and paths only. Plugin code is not opened or executed.","仅列出本地插件名称与路径，不读取或执行插件代码。"),Kind::AntigravityCommand=>text(cx,"Antigravity slash commands retain ~/.gemini/commands and project .gemini/commands. Nested TOML file names are supported.","Antigravity 斜杠命令沿用 ~/.gemini/commands 与项目 .gemini/commands，支持子目录 TOML 文件名。"),Kind::Extension=>text(cx,"Existing extensions in ~/.gemini/antigravity-cli/extensions. Manifests are read-only; this page does not install or uninstall extensions.","查看 ~/.gemini/antigravity-cli/extensions 中已有扩展。manifest 只读，此页不安装或卸载扩展。")}));
        if self.kind != Kind::Extension {
            root = root.child(
                div()
                    .flex()
                    .flex_wrap()
                    .flex_shrink_0()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("platform-file-user")
                            .label(text(cx, "User directory", "用户目录"))
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.scope = Scope::User;
                                this.page = 0;
                                this.rows.clear();
                                this.load(cx);
                            })),
                        locked,
                        false,
                        cx,
                    ))
                    .child(
                        Input::new(&self.project)
                            .aria_label(text(cx, "Absolute project directory", "项目绝对路径"))
                            .accessibility_id("platform-file-project-path")
                            .disabled(locked)
                            .w(px(320.)),
                    )
                    .child(semantics::button(
                        Button::new("platform-file-project")
                            .label(text(cx, "Use project directory", "使用项目目录"))
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.scope = Scope::Project(PathBuf::from(
                                    this.project.read(cx).value().to_string(),
                                ));
                                this.page = 0;
                                this.rows.clear();
                                this.load(cx);
                            })),
                        locked,
                        false,
                        cx,
                    )),
            );
        }
        root = root.child(div().flex_shrink_0().child(match &self.scope {
            Scope::User => text(cx, "Scope: user", "作用域：用户").to_string(),
            Scope::Project(path) => format!("{}: {}", text(cx, "Project", "项目"), path.display()),
        }));
        if let Some(error) = self.message.or_else(|| {
            if operation_visible {
                self.operation.result.and_then(Result::err)
            } else {
                None
            }
        }) {
            let message = error_text(error, cx);
            root = root.child(
                div()
                    .id("platform-file-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .flex_shrink_0()
                    .aria_label(message)
                    .child(message),
            );
        } else if operation_visible {
            if self.operation.busy {
                root = root.child(text(
                    cx,
                    "Saving to the original source. This write continues if you leave.",
                    "正在保存到原始来源，离页后写入仍会继续。",
                ));
            } else if self.operation.result.is_some_and(|result| result.is_ok()) {
                root = root.child(
                    div()
                        .id("platform-file-result")
                        .test_support()
                        .child(format!(
                            "{} · {}",
                            text(cx, "Change completed", "更改已完成"),
                            self.operation.target.as_deref().unwrap_or("")
                        )),
                );
            }
        }
        if let Some(session) = self.session.clone() {
            root = root
                .child(div().flex_shrink_0().child(session.source_label()))
                .child(
                    div().flex_shrink_0().child(
                        Textarea::new(&self.editor)
                            .font_family(cx.theme().mono_font_family.clone())
                            .text_size(cx.theme().mono_font_size)
                            .aria_label(text(cx, "Resource source", "资源源码"))
                            .accessibility_id("platform-file-content")
                            .readonly(!session.kind.writable())
                            .disabled(self.operation.busy || self.intent.is_some())
                            .h(px(280.)),
                    ),
                );
            if self.intent.is_some() {
                if self.enter_confirmation {
                    self.enter_confirmation = false;
                    window.focus(&self.confirmation_focus, cx);
                }
                let description = match self.intent {
                    Some(Intent::Delete) => text(
                        cx,
                        "Delete this file from its original source? A backup is kept.",
                        "从原始来源删除此文件？会保留备份。",
                    ),
                    Some(Intent::Discard) => text(
                        cx,
                        "Discard the draft and close the source?",
                        "丢弃草稿并关闭源码？",
                    ),
                    _ => text(
                        cx,
                        "Save the file to its original source?",
                        "将文件保存到原始来源？",
                    ),
                };
                root = root.child(
                    div()
                        .id("platform-file-confirmation")
                        .test_support()
                        .flex()
                        .flex_col()
                        .flex_shrink_0()
                        .gap(px(theme::CONTROL_GAP))
                        .on_key_down(cx.listener(
                            |this, event: &gpui_kit::KeyDownEvent, window, cx| {
                                if event.keystroke.key == "escape" {
                                    this.dismiss(window, cx);
                                    cx.stop_propagation();
                                }
                            },
                        ))
                        .child(description)
                        .child(
                            div()
                                .flex()
                                .gap(px(theme::CONTROL_GAP))
                                .child(semantics::button(
                                    Button::new("platform-file-cancel")
                                        .label(text(cx, "Keep editing", "继续编辑"))
                                        .ghost()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.dismiss(window, cx)
                                        })),
                                    false,
                                    false,
                                    cx,
                                ))
                                .child(semantics::button(
                                    Button::new("platform-file-confirm")
                                        .label(text(cx, "Confirm", "确认"))
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.confirm(window, cx)
                                        })),
                                    false,
                                    false,
                                    cx,
                                ))
                                .focus_trap(
                                    "platform-file-confirmation-trap",
                                    &self.confirmation_focus,
                                ),
                        ),
                );
            } else {
                let mut actions = div()
                    .flex()
                    .flex_wrap()
                    .flex_shrink_0()
                    .gap(px(theme::CONTROL_GAP));
                if session.kind.writable() {
                    actions = actions.child(semantics::button(
                        Button::new("platform-file-save")
                            .label(text(cx, "Save", "保存"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.prepare(
                                    Intent::Save(this.editor.read(cx).value().to_string()),
                                    cx,
                                )
                            })),
                        self.operation.busy,
                        false,
                        cx,
                    ));
                    if !session.creating {
                        actions = actions.child(semantics::button(
                            Button::new("platform-file-delete")
                                .label(text(cx, "Delete", "删除"))
                                .ghost()
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.prepare(Intent::Delete, cx)),
                                ),
                            self.operation.busy,
                            false,
                            cx,
                        ));
                    }
                }
                root = root.child(
                    actions.child(semantics::button(
                        Button::new("platform-file-close")
                            .label(text(cx, "Close source", "关闭源码"))
                            .ghost()
                            .on_click(
                                cx.listener(|this, _, _, cx| this.prepare(Intent::Discard, cx)),
                            ),
                        self.operation.busy,
                        false,
                        cx,
                    )),
                );
            }
            return root;
        }
        root = root.child(
            div()
                .flex()
                .flex_wrap()
                .flex_shrink_0()
                .gap(px(theme::CONTROL_GAP))
                .child(
                    Input::new(&self.filter)
                        .aria_label(text(cx, "Filter resource files", "筛选资源文件"))
                        .accessibility_id("platform-file-filter")
                        .w(px(260.)),
                )
                .child(semantics::button(
                    Button::new("platform-file-reload")
                        .label(text(cx, "Read directory", "读取目录"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| this.load(cx))),
                    locked,
                    false,
                    cx,
                )),
        );
        if self.kind.writable() {
            root = root.child(
                div()
                    .flex()
                    .flex_wrap()
                    .flex_shrink_0()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        Input::new(&self.name)
                            .aria_label(text(
                                cx,
                                "New file name including extension",
                                "含扩展名的新文件名",
                            ))
                            .accessibility_id("platform-file-name")
                            .w(px(320.)),
                    )
                    .child(semantics::button(
                        Button::new("platform-file-new")
                            .label(text(cx, "Create file", "创建文件"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.open(this.name.read(cx).value().to_string(), true, cx)
                            })),
                        locked,
                        false,
                        cx,
                    )),
            );
        }
        let filter = self.filter.read(cx).value().to_lowercase();
        let rows: Vec<_> = self
            .rows
            .iter()
            .filter(|row| row.key.to_lowercase().contains(&filter))
            .cloned()
            .collect();
        let pages = rows.len().div_ceil(PAGE_SIZE).max(1);
        self.page = self.page.min(pages - 1);
        if rows.is_empty() && !self.busy {
            root = root.child(text(
                cx,
                "No resource files in this directory.",
                "此目录中没有资源文件。",
            ));
        }
        for (index, row) in rows
            .into_iter()
            .skip(self.page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .enumerate()
        {
            let key = row.key.clone();
            let mut item = div()
                .id(("platform-file-row", index))
                .test_support()
                .flex()
                .flex_shrink_0()
                .gap(px(theme::CONTROL_GAP))
                .child(format!(
                    "{}{}{}",
                    row.key,
                    row.description
                        .map(|v| format!(" · {v}"))
                        .unwrap_or_default(),
                    row.version.map(|v| format!(" · {v}")).unwrap_or_default()
                ))
                .child(if row.malformed {
                    text(cx, "Unrecognized document", "无法识别的文档")
                } else {
                    ""
                });
            if self.kind == Kind::OpenCodePlugin {
                item = item.child(format!("{} · {} bytes", row.path, row.bytes));
            } else {
                item = item.child(semantics::button(
                    Button::new(("platform-file-open", index))
                        .label(text(cx, "Open source", "打开源码"))
                        .ghost()
                        .on_click(
                            cx.listener(move |this, _, _, cx| this.open(key.clone(), false, cx)),
                        ),
                    locked,
                    false,
                    cx,
                ));
            }
            root = root.child(item);
        }
        root.child(
            div()
                .flex()
                .flex_shrink_0()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("platform-file-prev")
                        .label(text(cx, "Previous", "上一页"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.page = this.page.saturating_sub(1);
                            cx.notify();
                        })),
                    self.page == 0,
                    false,
                    cx,
                ))
                .child(format!("{} / {}", self.page + 1, pages))
                .child(semantics::button(
                    Button::new("platform-file-next")
                        .label(text(cx, "Next", "下一页"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.page += 1;
                            cx.notify();
                        })),
                    self.page + 1 >= pages,
                    false,
                    cx,
                )),
        )
    }
}
#[cfg(test)]
mod tests;
