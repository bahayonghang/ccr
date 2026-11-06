//! Codex agents and tracked source installs within the existing Resources surface.
use crate::{
    locales::text,
    runtime::{
        Services,
        resources::codex_agents::{
            self as runtime, AgentRow, Mutation, OperationState, Read, ReadResult, Scope, Session,
            SourceAction,
        },
    },
    shell::semantics,
    theme,
};
use ccr_app::{
    codex_agents::sources::{
        CodexAgentSourceCatalogResponse, CodexAgentSourceInstallRequest, CodexAgentSourceRecord,
        CodexAgentSourceSyncRequest,
    },
    runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent},
};
use ccr_core::core::log_redact::redact_log_text;
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    Selectable,
    button::{Button, ButtonVariants},
    input::{Input, InputState, Textarea, TextareaState},
};
use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, StatefulInteractiveElement, Styled, Task, Window, div, px,
};
use std::sync::Arc;

#[derive(Clone)]
enum Confirm {
    Delete,
    RemoveSource(String),
    Install { agent: String, replace: bool },
    Sync { install: String, force: bool },
    AcceptLocal(String),
    Untrack(String),
}
pub struct CodexAgentsView {
    services: Services,
    scope: Scope,
    sources_mode: bool,
    rows: Vec<AgentRow>,
    sources: Vec<CodexAgentSourceRecord>,
    catalog: Option<Box<CodexAgentSourceCatalogResponse>>,
    selected_source: Option<String>,
    session: Option<Arc<Session>>,
    editing: bool,
    submitted: bool,
    preview: Option<String>,
    pending_text: Option<String>,
    editor: Entity<TextareaState>,
    name: Entity<InputState>,
    project: Entity<InputState>,
    repo: Entity<InputState>,
    confirmation: Option<Confirm>,
    generation: u64,
    gate: ResponseGate,
    read_task: Option<Task<()>>,
    observation: Option<Task<()>>,
    operation: Arc<OperationState>,
    busy: bool,
    page: usize,
    message: Option<runtime::Error>,
    focus: FocusHandle,
}
impl CodexAgentsView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            operation: services.codex_agents_state(),
            services,
            scope: Scope::Global,
            sources_mode: false,
            rows: Vec::new(),
            sources: Vec::new(),
            catalog: None,
            selected_source: None,
            session: None,
            editing: false,
            submitted: false,
            preview: None,
            pending_text: None,
            editor: cx.new(|cx| TextareaState::new(window, cx)),
            name: cx.new(|cx| InputState::new(window, cx)),
            project: cx.new(|cx| InputState::new(window, cx)),
            repo: cx.new(|cx| InputState::new(window, cx)),
            confirmation: None,
            generation: 0,
            gate: ResponseGate::default(),
            read_task: None,
            observation: None,
            busy: false,
            page: 0,
            message: None,
            focus: cx.focus_handle().tab_stop(false),
        }
    }
    pub fn blocks_switch(&self) -> bool {
        self.editing || self.operation.busy || self.confirmation.is_some()
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        window.focus_next(cx);
    }
    pub fn suspend(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.session = None;
        self.preview = None;
        self.catalog = None;
        self.pending_text = None;
        self.editing = false;
        self.submitted = false;
        self.editor
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.read_task = None;
        self.observation = None;
        self.gate.clear();
        self.generation += 1;
        self.busy = false;
        self.confirmation = None;
        cx.notify();
    }
    pub fn activate(&mut self, cx: &mut Context<Self>) {
        self.observe(cx);
        self.reload(cx);
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
            platform: Platform::Codex,
            generation: self.generation,
            request_id: self.generation,
        }
    }
    fn accept_operation(&mut self, state: Arc<OperationState>) {
        if self.submitted && !state.busy && state.result.is_some() {
            if state.result.is_some_and(|r| r.is_ok()) {
                self.editing = false;
                self.session = None;
                self.pending_text = Some(String::new());
            }
            self.submitted = false;
        }
        self.operation = state;
    }
    fn observe(&mut self, cx: &mut Context<Self>) {
        if self.observation.is_some() {
            return;
        }
        let mut updates = self.services.observe_codex_agents();
        self.accept_operation(updates.borrow_and_update().clone());
        self.observation = Some(cx.spawn(async move |weak, cx| {
            while updates.changed().await.is_ok() {
                let state = updates.borrow_and_update().clone();
                if weak
                    .update(cx, |this, cx| {
                        let done = !state.busy && state.result.is_some();
                        this.accept_operation(state);
                        if done {
                            this.reload(cx);
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
    fn reload(&mut self, cx: &mut Context<Self>) {
        if self.sources_mode {
            if let Some(id) = self.selected_source.clone() {
                self.request(Read::Catalog(id), cx);
            } else {
                self.request(Read::Sources, cx);
            }
        } else {
            self.request(Read::List(self.scope.clone()), cx);
        }
    }
    fn request(&mut self, request: Read, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let key = self.key();
        self.gate.replace(key.clone());
        self.busy = true;
        self.message = None;
        self.confirmation = None;
        // A failed load must not leave rows or previews from a previous scope
        // actionable under the newly selected source.
        match &request {
            Read::List(_) => {
                self.rows.clear();
                self.page = 0;
            }
            Read::Sources => {
                self.sources.clear();
                self.catalog = None;
                self.preview = None;
                self.page = 0;
            }
            Read::Catalog(_) => {
                self.catalog = None;
                self.preview = None;
                self.page = 0;
            }
            Read::Open(_, _) => {}
        }
        match self.services.codex_agents_read(key.clone(), request) {
            Ok(mut subscription) => {
                self.read_task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) {
                                    return;
                                }
                                this.busy = false;
                                match result.as_ref() {
                                    Completion::Success(ReadResult::Agents(rows)) => {
                                        this.rows = rows.clone()
                                    }
                                    Completion::Success(ReadResult::Opened(session)) => {
                                        this.session = Some(session.clone());
                                        this.pending_text = Some(session.snapshot.content.clone());
                                        this.editing = true;
                                    }
                                    Completion::Success(ReadResult::Sources(rows)) => {
                                        this.sources = rows.clone()
                                    }
                                    Completion::Success(ReadResult::Catalog(catalog)) => {
                                        this.catalog = Some(catalog.clone())
                                    }
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => {
                                        this.message = Some(*error)
                                    }
                                    _ => this.message = Some(runtime::Error::Interrupted),
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
                self.message = Some(runtime::Error::Interrupted);
            }
        }
        cx.notify();
    }
    fn start(&mut self, command: Mutation, cx: &mut Context<Self>) {
        if self.operation.busy {
            return;
        }
        self.observe(cx);
        let key = self.key();
        self.confirmation = None;
        self.message = None;
        let local_edit = !matches!(&command, Mutation::Source(_));
        match self.services.start_codex_agents(key, command) {
            Ok(()) => self.submitted = local_edit,
            Err(error) => self.message = Some(error),
        }
        self.operation = self.services.codex_agents_state();
        cx.notify();
    }
    fn new_agent(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.editing || self.operation.busy {
            return;
        }
        self.session = None;
        self.editing = true;
        self.name.update(cx, |s, cx| s.set_value("", window, cx));
        self.editor.update(cx, |s, cx| {
            s.set_value(
                "name = ''\ndescription = ''\ndeveloper_instructions = ''\n",
                window,
                cx,
            )
        });
        cx.notify();
    }
    fn save(&mut self, cx: &mut Context<Self>) {
        let content = self.editor.read(cx).value().to_string();
        let command = match &self.session {
            Some(session) => Mutation::Save {
                session: session.clone(),
                content,
            },
            None => Mutation::Add {
                scope: self.scope.clone(),
                name: self.name.read(cx).value().to_string(),
                content,
            },
        };
        self.start(command, cx);
    }
    fn confirm(&mut self, cx: &mut Context<Self>) {
        let Some(action) = self.confirmation.take() else {
            return;
        };
        let source = match action {
            Confirm::Delete => {
                if let Some(session) = &self.session {
                    self.start(Mutation::Delete(session.clone()), cx);
                }
                return;
            }
            Confirm::RemoveSource(id) => {
                self.selected_source = None;
                self.catalog = None;
                SourceAction::Remove(id)
            }
            Confirm::Install { agent, replace } => {
                let Some(source) = self.selected_source.clone() else {
                    return;
                };
                SourceAction::Install(CodexAgentSourceInstallRequest {
                    source_id: source,
                    agent_id: agent,
                    target_name: Some(self.name.read(cx).value().to_string()),
                    conflict_mode: replace.then(|| "replace".into()),
                })
            }
            Confirm::Sync { install, force } => SourceAction::Sync(CodexAgentSourceSyncRequest {
                install_id: install,
                force: Some(force),
            }),
            Confirm::AcceptLocal(id) => SourceAction::AcceptLocal(id),
            Confirm::Untrack(id) => SourceAction::Untrack(id),
        };
        self.start(Mutation::Source(source), cx);
    }
}
fn error(cx: &App, value: runtime::Error) -> &'static str {
    match value {
        runtime::Error::Unsupported => text(
            cx,
            "Codex agents require the Local environment.",
            "Codex Agents 仅支持本地环境。",
        ),
        runtime::Error::Conflict => text(
            cx,
            "The target exists or changed. Draft retained; reload before deciding to replace it.",
            "目标已存在或已被修改。草稿已保留，请重新读取后决定是否替换。",
        ),
        runtime::Error::FixtureNetwork => text(
            cx,
            "Remote agent requests are disabled for fixture data.",
            "演示数据禁止远程 Agent 请求。",
        ),
        runtime::Error::Invalid => text(
            cx,
            "Check the agent name, required TOML fields and project directory.",
            "请检查 Agent 名称、必需 TOML 字段及项目目录。",
        ),
        runtime::Error::Busy => text(cx, "An agent operation is running.", "Agent 操作正在运行。"),
        runtime::Error::Interrupted => text(
            cx,
            "Operation interrupted; read saved data before retrying.",
            "操作中断，重试前请读取保存的数据。",
        ),
        runtime::Error::Source => text(
            cx,
            "Source request or cache is unavailable. Refresh the selected source and check GitHub access.",
            "源请求或缓存不可用。请刷新所选源并检查 GitHub 访问。",
        ),
        runtime::Error::Storage => text(
            cx,
            "Agent storage failed. Draft retained; check paths and permissions.",
            "Agent 存储失败。草稿已保留，请检查路径及权限。",
        ),
    }
}
impl Render for CodexAgentsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(value) = self.pending_text.take() {
            self.editor
                .update(cx, |state, cx| state.set_value(value, window, cx));
        }
        let locked = self.busy || self.operation.busy;
        let mut root = div()
            .id("codex-agents")
            .track_focus(&self.focus)
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scroll()
            .gap(px(theme::CONTROL_GAP));
        root = root.child(
            div()
                .flex()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("codex-agents-local")
                        .label(text(cx, "Local agents", "本地 Agents"))
                        .selected(!self.sources_mode)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.page = 0;
                            this.sources_mode = false;
                            this.confirmation = None;
                            this.reload(cx);
                        })),
                    locked || self.editing,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("codex-agents-sources")
                        .label(text(cx, "Source agents", "源 Agents"))
                        .selected(self.sources_mode)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.page = 0;
                            this.sources_mode = true;
                            this.confirmation = None;
                            this.reload(cx);
                        })),
                    locked || self.editing,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("codex-agents-read")
                        .label(text(cx, "Read saved data", "读取已存数据"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| this.reload(cx))),
                    locked || self.editing,
                    false,
                    cx,
                )),
        );
        if let Some(value) = self
            .message
            .or_else(|| self.operation.result.and_then(Result::err))
        {
            root = root.child(
                div()
                    .id("codex-agents-error")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(error(cx, value))
                    .child(error(cx, value)),
            );
        }
        if self.operation.busy {
            root=root.child(text(cx,"Operation continues when you leave this page. Active source requests cannot be cancelled.","离开页面后操作继续。进行中的源请求无法取消。"));
        } else if self.operation.result.is_some_and(|r| r.is_ok()) {
            root = root.child(div().id("codex-agents-result").test_support().child(text(
                cx,
                "Agent operation completed.",
                "Agent 操作已完成。",
            )));
        }
        if self.sources_mode {
            root=root.child(text(cx,"Cached catalogs do not contact GitHub. Adding or refreshing a source and syncing an install contact GitHub. Review remote instructions before installing.","读取缓存目录不会连接 GitHub。添加或刷新源、同步安装会连接 GitHub。安装前请审阅远程指令。"))
                .child(Input::new(&self.repo).aria_label(text(cx,"GitHub repository URL","GitHub 仓库 URL")).accessibility_id("codex-agent-repo"))
                .child(semantics::button(Button::new("codex-agent-source-add").label(text(cx,"Add source","添加源")).on_click(cx.listener(|this,_,_,cx|{let url=this.repo.read(cx).value().to_string();this.selected_source=None;this.start(Mutation::Source(SourceAction::Add(url)),cx);})),locked,false,cx));
            for (index, source) in self
                .sources
                .iter()
                .skip(self.page * 100)
                .take(100)
                .enumerate()
            {
                let id = source.id.clone();
                let refresh = id.clone();
                let remove = id.clone();
                root = root.child(
                    div()
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new(("codex-agent-source", index))
                                .label(format!(
                                    "{} · {}",
                                    redact_log_text(&source.repo_url),
                                    redact_log_text(&source.status)
                                ))
                                .ghost()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.page = 0;
                                    this.selected_source = Some(id.clone());
                                    this.catalog = None;
                                    this.request(Read::Catalog(id.clone()), cx);
                                })),
                            locked,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new(("codex-agent-source-refresh", index))
                                .label(text(cx, "Refresh source", "刷新源"))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.selected_source = Some(refresh.clone());
                                    this.start(
                                        Mutation::Source(SourceAction::Refresh(refresh.clone())),
                                        cx,
                                    );
                                })),
                            locked,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new(("codex-agent-source-remove", index))
                                .label(text(cx, "Remove source", "移除源"))
                                .ghost()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.confirmation = Some(Confirm::RemoveSource(remove.clone()));
                                    cx.notify();
                                })),
                            locked,
                            false,
                            cx,
                        )),
                );
            }
            if let Some(catalog) = &self.catalog {
                root = root
                    .child(format!(
                        "{} · {}",
                        redact_log_text(&catalog.source.repo_url),
                        redact_log_text(&catalog.source.status)
                    ))
                    .child(
                        Input::new(&self.name)
                            .aria_label(text(
                                cx,
                                "Install name (blank uses agent name)",
                                "安装名称（留空使用 Agent 名称）",
                            ))
                            .accessibility_id("codex-agent-install-name"),
                    );
                for (index, agent) in catalog
                    .agents
                    .iter()
                    .skip(self.page * 100)
                    .take(100)
                    .enumerate()
                {
                    let id = agent.id.clone();
                    let raw = agent.raw_toml.clone();
                    let replacement = id.clone();
                    root = root.child(
                        div()
                            .flex()
                            .gap(px(theme::CONTROL_GAP))
                            .child(redact_log_text(&agent.name))
                            .child(semantics::button(
                                Button::new(("codex-agent-preview", index))
                                    .label(text(cx, "Preview instructions", "预览指令"))
                                    .ghost()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.preview = Some(raw.clone());
                                        cx.notify();
                                    })),
                                locked,
                                false,
                                cx,
                            ))
                            .child(semantics::button(
                                Button::new(("codex-agent-install", index))
                                    .label(text(cx, "Install", "安装"))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.confirmation = Some(Confirm::Install {
                                            agent: id.clone(),
                                            replace: false,
                                        });
                                        cx.notify();
                                    })),
                                locked || agent.parse_error.is_some(),
                                false,
                                cx,
                            ))
                            .child(semantics::button(
                                Button::new(("codex-agent-replace", index))
                                    .label(text(cx, "Replace tracked install", "替换已跟踪安装"))
                                    .ghost()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.confirmation = Some(Confirm::Install {
                                            agent: replacement.clone(),
                                            replace: true,
                                        });
                                        cx.notify();
                                    })),
                                locked || agent.parse_error.is_some(),
                                false,
                                cx,
                            )),
                    );
                }
                for (index, install) in catalog
                    .installs
                    .iter()
                    .skip(self.page * 100)
                    .take(100)
                    .enumerate()
                {
                    let sync = install.id.clone();
                    let force = sync.clone();
                    let accept = sync.clone();
                    let untrack = sync.clone();
                    root = root.child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(theme::CONTROL_GAP))
                            .child(format!(
                                "{} · {}",
                                redact_log_text(&install.installed_name),
                                redact_log_text(&install.status)
                            ))
                            .child(semantics::button(
                                Button::new(("codex-agent-sync", index))
                                    .label(text(cx, "Sync install", "同步安装"))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.confirmation = Some(Confirm::Sync {
                                            install: sync.clone(),
                                            force: false,
                                        });
                                        cx.notify();
                                    })),
                                locked,
                                false,
                                cx,
                            ))
                            .child(semantics::button(
                                Button::new(("codex-agent-force", index))
                                    .label(text(cx, "Replace with upstream", "用上游替换"))
                                    .ghost()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.confirmation = Some(Confirm::Sync {
                                            install: force.clone(),
                                            force: true,
                                        });
                                        cx.notify();
                                    })),
                                locked,
                                false,
                                cx,
                            ))
                            .child(semantics::button(
                                Button::new(("codex-agent-accept", index))
                                    .label(text(cx, "Keep local changes", "保留本地修改"))
                                    .ghost()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.confirmation =
                                            Some(Confirm::AcceptLocal(accept.clone()));
                                        cx.notify();
                                    })),
                                locked,
                                false,
                                cx,
                            ))
                            .child(semantics::button(
                                Button::new(("codex-agent-untrack", index))
                                    .label(text(cx, "Stop tracking", "停止跟踪"))
                                    .ghost()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.confirmation = Some(Confirm::Untrack(untrack.clone()));
                                        cx.notify();
                                    })),
                                locked,
                                false,
                                cx,
                            )),
                    );
                }
                if catalog.agents.len() > 100 || catalog.installs.len() > 100 {
                    root = root.child(text(
                        cx,
                        "Showing up to 100 entries per page.",
                        "每页显示最多 100 项。",
                    ));
                }
            }
            if let Some(preview) = &self.preview {
                root = root
                    .child(div().id("codex-agent-preview-text").child(preview.clone()))
                    .child(semantics::button(
                        Button::new("codex-agent-preview-close")
                            .label(text(cx, "Close preview", "关闭预览"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.preview = None;
                                cx.notify();
                            })),
                        false,
                        false,
                        cx,
                    ));
            }
        } else {
            root = root
                .child(
                    div()
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new("codex-agent-global")
                                .label(text(cx, "Global scope", "全局范围"))
                                .selected(self.scope == Scope::Global)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.scope = Scope::Global;
                                    this.reload(cx);
                                })),
                            locked || self.editing,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("codex-agent-project")
                                .label(text(cx, "Use project directory", "使用项目目录"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.scope = Scope::Project(
                                        this.project.read(cx).value().to_string().into(),
                                    );
                                    this.reload(cx);
                                })),
                            locked || self.editing,
                            false,
                            cx,
                        )),
                )
                .child(
                    Input::new(&self.project)
                        .aria_label(text(cx, "Absolute project directory", "项目绝对目录"))
                        .accessibility_id("codex-agent-project-path"),
                )
                .child(semantics::button(
                    Button::new("codex-agent-new")
                        .label(text(cx, "New agent", "新增 Agent"))
                        .on_click(cx.listener(|this, _, window, cx| this.new_agent(window, cx))),
                    locked || self.editing,
                    false,
                    cx,
                ));
            if !self.editing {
                for (index, agent) in self.rows.iter().skip(self.page * 100).take(100).enumerate() {
                    let id = agent.key.clone();
                    root = root.child(semantics::button(
                        Button::new(("codex-agent-open", index))
                            .label(format!(
                                "{}{}",
                                agent.name,
                                if agent.malformed {
                                    text(cx, " (invalid TOML)", "（TOML 无效）")
                                } else {
                                    ""
                                }
                            ))
                            .ghost()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.request(Read::Open(this.scope.clone(), id.clone()), cx)
                            })),
                        locked,
                        false,
                        cx,
                    ));
                }
                if self.rows.len() > 100 {
                    root = root.child(text(
                        cx,
                        "Showing up to 100 agents per page.",
                        "每页显示最多 100 个 Agents。",
                    ));
                }
            }
            if self.editing {
                root = root
                    .child(
                        Input::new(&self.name)
                            .aria_label(text(
                                cx,
                                "Name for creation, rename or copy",
                                "新增、重命名或复制的名称",
                            ))
                            .accessibility_id("codex-agent-name"),
                    )
                    .child(Textarea::new(&self.editor).h(px(250.)))
                    .child(
                        div()
                            .flex()
                            .gap(px(theme::CONTROL_GAP))
                            .child(semantics::button(
                                Button::new("codex-agent-save")
                                    .label(text(cx, "Save agent", "保存 Agent"))
                                    .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
                                locked,
                                false,
                                cx,
                            ))
                            .child(semantics::button(
                                Button::new("codex-agent-discard")
                                    .label(text(cx, "Discard draft", "丢弃草稿"))
                                    .ghost()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.editing = false;
                                        this.session = None;
                                        this.editor.update(cx, |s, cx| s.set_value("", window, cx));
                                        cx.notify();
                                    })),
                                locked,
                                false,
                                cx,
                            )),
                    );
                if self.session.is_some() {
                    root = root.child(
                        div()
                            .flex()
                            .gap(px(theme::CONTROL_GAP))
                            .child(semantics::button(
                                Button::new("codex-agent-rename")
                                    .label(text(cx, "Rename saved agent", "重命名已存 Agent"))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if let Some(session) = &this.session {
                                            this.start(
                                                Mutation::Rename {
                                                    session: session.clone(),
                                                    name: this.name.read(cx).value().to_string(),
                                                },
                                                cx,
                                            );
                                        }
                                    })),
                                locked,
                                false,
                                cx,
                            ))
                            .child(semantics::button(
                                Button::new("codex-agent-copy")
                                    .label(text(
                                        cx,
                                        "Copy saved agent to this scope",
                                        "复制已存 Agent 到当前范围",
                                    ))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if let Some(session) = &this.session {
                                            this.start(
                                                Mutation::Copy {
                                                    session: session.clone(),
                                                    target: this.scope.clone(),
                                                    name: this.name.read(cx).value().to_string(),
                                                },
                                                cx,
                                            );
                                        }
                                    })),
                                locked,
                                false,
                                cx,
                            ))
                            .child(semantics::button(
                                Button::new("codex-agent-delete")
                                    .label(text(cx, "Delete agent", "删除 Agent"))
                                    .ghost()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.confirmation = Some(Confirm::Delete);
                                        cx.notify();
                                    })),
                                locked,
                                false,
                                cx,
                            )),
                    );
                }
            }
        }
        if let Some(action) = &self.confirmation {
            let warning = match action {
                Confirm::Delete => text(cx, "Delete this agent file?", "删除此 Agent 文件？"),
                Confirm::RemoveSource(_) => text(
                    cx,
                    "Remove this source and its cached catalog? Installed agents remain.",
                    "移除此源及缓存目录？已安装的 Agents 保留。",
                ),
                Confirm::Install { replace: true, .. } | Confirm::Sync { force: true, .. } => text(
                    cx,
                    "Replace the tracked local file with upstream instructions? Local modifications may be overwritten.",
                    "用上游指令替换已跟踪的本地文件？本地修改可能被覆盖。",
                ),
                Confirm::Install { .. } => text(
                    cx,
                    "Install these remote instructions into global Codex agents? This does not execute them.",
                    "将这些远程指令安装为全局 Codex Agent？此操作不会执行指令。",
                ),
                Confirm::Sync { .. } => text(
                    cx,
                    "Download the latest agent from GitHub and update this tracked install?",
                    "从 GitHub 下载最新 Agent 并更新此跟踪安装？",
                ),
                Confirm::AcceptLocal(_) => text(
                    cx,
                    "Accept the saved local file as the new tracking baseline?",
                    "将已存本地文件接受为新的跟踪基线？",
                ),
                Confirm::Untrack(_) => text(
                    cx,
                    "Stop tracking this install while keeping its local file?",
                    "停止跟踪此安装并保留本地文件？",
                ),
            };
            root = root.child(
                div()
                    .id("codex-agent-confirmation")
                    .test_support()
                    .child(warning)
                    .child(semantics::button(
                        Button::new("codex-agent-confirm")
                            .label(text(cx, "Confirm", "确认"))
                            .on_click(cx.listener(|this, _, _, cx| this.confirm(cx))),
                        locked,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("codex-agent-cancel")
                            .label(text(cx, "Cancel", "取消"))
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.confirmation = None;
                                cx.notify();
                            })),
                        locked,
                        false,
                        cx,
                    )),
            );
        }
        let total = if self.sources_mode {
            self.catalog
                .as_ref()
                .map(|c| c.agents.len().max(c.installs.len()))
                .unwrap_or(self.sources.len())
        } else {
            self.rows.len()
        };
        if total > 100 || self.page > 0 {
            root = root.child(
                div()
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("codex-agents-previous")
                            .label(text(cx, "Previous page", "上一页"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.page = this.page.saturating_sub(1);
                                cx.notify();
                            })),
                        locked || self.editing || self.page == 0,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("codex-agents-next")
                            .label(text(cx, "Next page", "下一页"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.page += 1;
                                cx.notify();
                            })),
                        locked || self.editing || (self.page + 1) * 100 >= total,
                        false,
                        cx,
                    )),
            );
        }
        root
    }
}

#[cfg(test)]
mod tests;
