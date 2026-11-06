//! Local Claude resource operations with a source-bound edit session.
use crate::{
    locales::text,
    runtime::{
        Services,
        resources::claude_files::{Command, Error, Kind, Outcome, Row, Session},
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    FocusTrapElement,
    button::{Button, ButtonVariants},
    input::{Input, InputState, Textarea, TextareaState},
};
use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Subscription, Task,
    Window, div, px,
};
use std::sync::Arc;

const PAGE_SIZE: usize = 40;
enum Intent {
    Save(String),
    Delete,
    Discard,
}
enum Notice {
    Error(Error),
    Queue,
    Cancelled,
    Saved,
}

pub struct ClaudeResourcesView {
    services: Services,
    kind: Kind,
    rows: Vec<Row>,
    filter: Entity<InputState>,
    name: Entity<InputState>,
    editor: Entity<TextareaState>,
    _filter_observation: Subscription,
    page: usize,
    session: Option<Arc<Session>>,
    pending_text: Option<String>,
    enter_editor: bool,
    intent: Option<Intent>,
    enter_confirmation: bool,
    busy: bool,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    notice: Option<Notice>,
    focus: FocusHandle,
    confirmation_focus: FocusHandle,
}
impl ClaudeResourcesView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let filter = cx.new(|cx| InputState::new(window, cx));
        let observation = cx.observe(&filter, |this, _, cx| {
            this.page = 0;
            cx.notify();
        });
        Self {
            services,
            kind: Kind::Rules,
            rows: Vec::new(),
            filter,
            name: cx.new(|cx| InputState::new(window, cx)),
            editor: cx.new(|cx| TextareaState::new(window, cx)),
            _filter_observation: observation,
            page: 0,
            session: None,
            pending_text: None,
            enter_editor: false,
            intent: None,
            enter_confirmation: false,
            busy: false,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            notice: None,
            focus: cx.focus_handle().tab_stop(false),
            confirmation_focus: cx.focus_handle(),
        }
    }
    pub fn load(&mut self, cx: &mut Context<Self>) {
        self.request(Command::List(self.kind), cx);
    }
    pub fn blocks_switch(&self) -> bool {
        self.busy || self.session.is_some() || self.intent.is_some()
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.session.is_some() {
            window.focus(&self.editor.read(cx).focus_handle(cx), cx);
        } else {
            window.focus(&self.focus, cx);
            window.focus_next(cx);
        }
    }
    pub fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.generation = self.generation.saturating_add(1);
        self.session = None;
        self.intent = None;
        self.pending_text = None;
        self.enter_editor = false;
        self.enter_confirmation = false;
        self.rows.clear();
        self.busy = false;
        self.notice = None;
        self.editor
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.name
            .update(cx, |state, cx| state.set_value("", window, cx));
        cx.notify();
    }
    fn request(&mut self, command: Command, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.generation = self.generation.saturating_add(1);
        let key = RequestKey {
            environment: EnvironmentId::Local(
                if self.services.is_fixture() {
                    "fixture"
                } else {
                    "local"
                }
                .into(),
            ),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        self.busy = true;
        self.notice = None;
        match self.services.claude_resource_request(key.clone(), command) {
            Ok(mut subscription) => {
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) {
                                    return;
                                }
                                this.busy = false;
                                match result.as_ref() {
                                    Completion::Success(Outcome::Listed(rows)) => {
                                        this.rows = rows.clone()
                                    }
                                    Completion::Success(Outcome::Opened(session)) => {
                                        this.pending_text = Some(session.initial_text());
                                        this.enter_editor = true;
                                        this.session = Some(session.clone());
                                    }
                                    Completion::Success(Outcome::Committed) => {
                                        this.session = None;
                                        this.pending_text = Some(String::new());
                                        this.rows.clear();
                                        this.request(Command::List(this.kind), cx);
                                        this.notice = Some(Notice::Saved);
                                    }
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => {
                                        this.notice = Some(Notice::Error(*error))
                                    }
                                    _ => this.notice = Some(Notice::Cancelled),
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
                self.notice = Some(Notice::Queue);
            }
        }
        cx.notify();
    }
    fn open(&mut self, name: String, creating: bool, cx: &mut Context<Self>) {
        if self.session.is_some() || self.busy {
            return;
        }
        self.request(
            Command::Open {
                kind: self.kind,
                name,
                creating,
            },
            cx,
        );
    }
    fn prepare(&mut self, intent: Intent, cx: &mut Context<Self>) {
        if self.busy || self.session.is_none() {
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
            self.editor
                .update(cx, |state, cx| state.set_value("", window, cx));
            window.focus(&self.focus, cx);
            self.load(cx);
            return;
        }
        if let Some(session) = self.session.clone() {
            let content = match intent {
                Intent::Save(content) => Some(content),
                _ => None,
            };
            self.request(Command::Commit { session, content }, cx);
        }
    }
}

fn title(kind: Kind, cx: &App) -> &'static str {
    match kind {
        Kind::Rules => text(cx, "Rules", "规则"),
        Kind::OutputStyles => text(cx, "Output styles", "输出样式"),
        Kind::Agents => text(cx, "Agents", "代理配置"),
        Kind::SlashCommands => text(cx, "Slash commands", "斜杠命令"),
        Kind::Plugins => text(cx, "Plugins", "插件配置"),
    }
}
fn notice_text(notice: &Notice, cx: &App) -> &'static str {
    match notice {
        Notice::Queue => text(
            cx,
            "The operation queue is unavailable. Try again.",
            "操作队列不可用，请重试。",
        ),
        Notice::Cancelled => text(
            cx,
            "The operation did not complete. Your draft is retained.",
            "操作未完成，草稿已保留。",
        ),
        Notice::Saved => text(
            cx,
            "Change saved. The resource list has been refreshed.",
            "更改已保存，资源列表已刷新。",
        ),
        Notice::Error(error) => match error {
            Error::Unsupported => text(
                cx,
                "These Claude resources are available only in Local.",
                "这些 Claude 资源仅在本地环境可用。",
            ),
            Error::Invalid => text(
                cx,
                "Check the name and document format. File names must end in .md; settings patches must satisfy the selected resource schema.",
                "请检查名称与内容格式。文件名须以 .md 结尾；设置补丁须符合所选资源结构。",
            ),
            Error::UnsafePath => text(
                cx,
                "This path contains an unsupported link or escapes the resource directory.",
                "此路径包含不支持的链接或超出资源目录。",
            ),
            Error::Storage => text(
                cx,
                "Cannot read or write the source. Your draft is retained.",
                "无法读取或写入来源，草稿已保留。",
            ),
            Error::TooLarge => text(
                cx,
                "The resource exceeds the editor's file or directory limits.",
                "资源超过编辑器的文件或目录限制。",
            ),
            Error::Conflict => text(
                cx,
                "The source changed. Your draft and original version are retained. Discard and reopen before retrying.",
                "来源已被修改，草稿与原始版本已保留。请丢弃草稿并重新打开后重试。",
            ),
            Error::Missing => text(
                cx,
                "The selected resource no longer exists. Reload the list.",
                "所选资源已不存在，请重新读取列表。",
            ),
            Error::Duplicate => text(
                cx,
                "This name already exists or is ambiguous. Choose a unique name.",
                "名称已存在或存在重复，请使用唯一名称。",
            ),
        },
    }
}

impl Render for ClaudeResourcesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(content) = self.pending_text.take() {
            self.editor
                .update(cx, |state, cx| state.set_value(content, window, cx));
        }
        if self.enter_editor {
            self.enter_editor = false;
            window.focus(&self.editor.read(cx).focus_handle(cx), cx);
        }
        let mut root = div()
            .id("claude-resources")
            .test_support()
            .track_focus(&self.focus)
            .tab_stop(false)
            .size_full()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .gap(px(theme::CONTROL_GAP));
        let locked = self.busy || self.session.is_some();
        let mut kinds = div()
            .flex()
            .flex_wrap()
            .flex_shrink_0()
            .gap(px(theme::CONTROL_GAP));
        for (id, kind) in [
            ("claude-resource-rules", Kind::Rules),
            ("claude-resource-styles", Kind::OutputStyles),
            ("claude-resource-agents", Kind::Agents),
            ("claude-resource-commands", Kind::SlashCommands),
            ("claude-resource-plugins", Kind::Plugins),
        ] {
            kinds = kinds.child(semantics::button(
                Button::new(id)
                    .label(title(kind, cx))
                    .ghost()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.kind = kind;
                        this.page = 0;
                        this.rows.clear();
                        this.load(cx);
                    })),
                locked,
                false,
                cx,
            ));
        }
        root = root.child(kinds).child(div().flex_shrink_0().child(if self.kind.is_file() {
            text(cx, "Markdown files in the existing Claude resource directory. Frontmatter and text are preserved.", "现有 Claude 资源目录中的 Markdown 文件，保留前置元数据与正文。")
        } else {
            text(cx, "Named entries in the existing Claude settings.json. Editing a configuration does not install or run a plugin, agent, or command.", "现有 Claude settings.json 中的命名条目。编辑配置不会安装或运行插件、代理或命令。")
        }));
        if let Some(notice) = &self.notice {
            let message = notice_text(notice, cx);
            root = root.child(
                div()
                    .id("claude-resource-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(message)
                    .flex_shrink_0()
                    .child(message),
            );
        }
        if let Some(session) = self.session.clone() {
            root = root.child(div().flex_shrink_0().child(format!("{} · {}", title(session.kind, cx), session.name)))
                .child(div().flex_shrink_0().child(session.source.to_string_lossy().into_owned()))
                .child(div().flex_shrink_0().child(if session.kind.is_file() {
                    text(cx, "Editing the full file. Leaving this page discards the draft; a write already started continues.", "正在编辑完整文件。离开此页会丢弃草稿；已经开始的写入将继续。")
                } else {
                    text(cx, "Submit a JSON object patch. Omitted fields and existing credentials are retained; stored credentials are never loaded into this editor.", "提交 JSON 对象补丁。省略的字段与已有凭据保持不变；已有凭据不会载入编辑器。")
                }))
                .child(div().flex_shrink_0().child(Textarea::new(&self.editor).aria_label(text(cx, "Claude resource content", "Claude 资源内容"))
                    .accessibility_id("claude-resource-content").disabled(self.busy || self.intent.is_some()).h(px(260.))));
            if self.intent.is_some() {
                if self.enter_confirmation {
                    self.enter_confirmation = false;
                    window.focus(&self.confirmation_focus, cx);
                }
                let description = match self.intent {
                    Some(Intent::Delete) => text(
                        cx,
                        "Delete this resource from its original source? A backup is kept for files.",
                        "从原始来源删除此资源？文件会保留备份。",
                    ),
                    Some(Intent::Discard) => text(
                        cx,
                        "Discard this draft and close the editor?",
                        "丢弃草稿并关闭编辑器？",
                    ),
                    _ => text(
                        cx,
                        "Save this change to the selected source?",
                        "将更改保存到所选来源？",
                    ),
                };
                root = root.child(
                    div()
                        .id("claude-resource-confirmation")
                        .test_support()
                        .flex_shrink_0()
                        .flex()
                        .flex_col()
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
                                    Button::new("claude-resource-cancel")
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
                                    Button::new("claude-resource-confirm")
                                        .label(text(cx, "Confirm", "确认"))
                                        .primary()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.confirm(window, cx)
                                        })),
                                    false,
                                    false,
                                    cx,
                                ))
                                .focus_trap(
                                    "claude-resource-confirmation-trap",
                                    &self.confirmation_focus,
                                ),
                        ),
                );
            } else {
                let mut actions = div()
                    .flex()
                    .flex_wrap()
                    .flex_shrink_0()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("claude-resource-save")
                            .label(text(cx, "Save", "保存"))
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.prepare(
                                    Intent::Save(this.editor.read(cx).value().to_string()),
                                    cx,
                                )
                            })),
                        self.busy,
                        self.busy,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("claude-resource-discard")
                            .label(text(cx, "Discard and close", "丢弃并关闭"))
                            .ghost()
                            .on_click(
                                cx.listener(|this, _, _, cx| this.prepare(Intent::Discard, cx)),
                            ),
                        self.busy,
                        false,
                        cx,
                    ));
                if !session.creating {
                    actions = actions.child(semantics::button(
                        Button::new("claude-resource-delete")
                            .label(text(cx, "Delete", "删除"))
                            .ghost()
                            .on_click(
                                cx.listener(|this, _, _, cx| this.prepare(Intent::Delete, cx)),
                            ),
                        self.busy,
                        false,
                        cx,
                    ));
                }
                root = root.child(actions);
            }
            return root;
        }
        root = root
            .child(
                div().flex_shrink_0().child(
                    Input::new(&self.filter)
                        .id("claude-resource-filter")
                        .aria_label(text(cx, "Filter resource names", "筛选资源名称")),
                ),
            )
            .child(
                div()
                    .flex()
                    .flex_shrink_0()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        div().flex_1().child(
                            Input::new(&self.name)
                                .id("claude-resource-name")
                                .aria_label(if self.kind.is_file() {
                                    text(cx, "New file name ending in .md", "新文件名，以 .md 结尾")
                                } else {
                                    text(cx, "New resource name", "新资源名称")
                                }),
                        ),
                    )
                    .child(semantics::button(
                        Button::new("claude-resource-create")
                            .label(text(cx, "Create", "创建"))
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.open(this.name.read(cx).value().to_string(), true, cx)
                            })),
                        self.busy,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("claude-resource-reload")
                            .label(text(cx, "Reload", "重新读取"))
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| this.load(cx))),
                        self.busy,
                        self.busy,
                        cx,
                    )),
            );
        let filter = self.filter.read(cx).value().to_lowercase();
        let filtered: Vec<_> = self
            .rows
            .iter()
            .filter(|row| row.name.to_lowercase().contains(&filter))
            .collect();
        let pages = filtered.len().div_ceil(PAGE_SIZE).max(1);
        self.page = self.page.min(pages - 1);
        for (index, row) in filtered
            .iter()
            .skip(self.page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .enumerate()
        {
            let name = row.name.clone();
            let label = match row.enabled {
                Some(false) => format!("{} · {}", row.name, text(cx, "disabled", "已禁用")),
                _ => row.name.clone(),
            };
            root = root.child(
                div().flex_shrink_0().child(semantics::button(
                    Button::new(SharedString::from(format!("claude-resource-row-{index}")))
                        .label(label)
                        .ghost()
                        .on_click(
                            cx.listener(move |this, _, _, cx| this.open(name.clone(), false, cx)),
                        ),
                    self.busy,
                    false,
                    cx,
                )),
            );
        }
        if filtered.is_empty() && !self.busy {
            let label = text(
                cx,
                "No resources match this view.",
                "此视图没有匹配的资源。",
            );
            root = root.child(
                div()
                    .id("claude-resource-empty")
                    .test_support()
                    .aria_label(label)
                    .flex_shrink_0()
                    .child(label),
            );
        }
        root.child(
            div()
                .flex()
                .flex_shrink_0()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("claude-resource-prev")
                        .label(text(cx, "Previous", "上一页"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.page = this.page.saturating_sub(1);
                            cx.notify();
                        })),
                    self.page == 0 || self.busy,
                    false,
                    cx,
                ))
                .child(format!("{} / {}", self.page + 1, pages))
                .child(semantics::button(
                    Button::new("claude-resource-next")
                        .label(text(cx, "Next", "下一页"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.page += 1;
                            cx.notify();
                        })),
                    self.page + 1 >= pages || self.busy,
                    false,
                    cx,
                )),
        )
    }
}

#[cfg(test)]
mod tests;
