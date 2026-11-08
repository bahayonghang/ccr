//! Source-aware local MCP management; server process status is not inferred.
mod batch;
mod editor;
use crate::{
    locales::{is_chinese, service_error, text},
    runtime::{
        Services,
        mcp::{McpCatalog, McpEntry, McpScope, McpSource, McpSourceState, McpTarget},
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent};
use editor::{EditMode, EditorEvent, McpEditor};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    Selectable,
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
};
use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Subscription, Task, Window, div, px,
};

pub struct McpView {
    services: Services,
    filter: Entity<InputState>,
    project: Entity<InputState>,
    create_source: usize,
    create_mode: EditMode,
    show_presets: bool,
    show_sources: bool,
    editor: Option<Entity<McpEditor>>,
    editor_events: Option<Subscription>,
    batch: Option<Entity<batch::McpBatch>>,
    batch_events: Option<Subscription>,
    last_batch: Option<std::sync::Arc<crate::runtime::mcp::BatchOperation>>,
    _filter: Subscription,
    catalog: Option<McpCatalog>,
    selected: Option<String>,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    busy: bool,
    message: Option<&'static str>,
    focus: FocusHandle,
    platform_scope: Option<Platform>,
}

impl McpView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let filter = cx.new(|cx| InputState::new(window, cx));
        let subscription = cx.subscribe(&filter, |_, _, event, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        });
        Self {
            services,
            filter,
            project: cx.new(|cx| InputState::new(window, cx)),
            create_source: 0,
            create_mode: EditMode::Add,
            show_presets: false,
            show_sources: false,
            editor: None,
            editor_events: None,
            batch: None,
            batch_events: None,
            last_batch: None,
            _filter: subscription,
            catalog: None,
            selected: None,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            busy: false,
            message: None,
            focus: cx.focus_handle().tab_stop(false),
            platform_scope: None,
        }
    }

    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        window.focus_next(cx);
    }

    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.detach_batch(cx);
        self.create_mode = EditMode::Add;
        self.show_sources = false;
        self.show_presets = false;
        self.editor = None;
        self.editor_events = None;
        self.task = None;
        self.gate.clear();
        self.generation += 1;
        self.busy = false;
        cx.notify();
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.request(cx);
    }
    pub fn open_global(&mut self, cx: &mut Context<Self>) {
        self.detach_batch(cx);
        self.platform_scope = None;
        self.refresh(cx);
    }
    pub fn open_platform(&mut self, platform: Platform, cx: &mut Context<Self>) {
        self.detach_batch(cx);
        self.platform_scope = Some(platform);
        self.create_source = match platform {
            Platform::Codex => 3,
            Platform::Gemini => 4,
            _ => 0,
        };
        self.refresh(cx);
    }
    fn source_in_scope(&self, source: McpSource) -> bool {
        match self.platform_scope {
            None => true,
            Some(Platform::Claude) => matches!(
                source,
                McpSource::ClaudeState | McpSource::ClaudeSettings | McpSource::ProjectFile
            ),
            Some(Platform::Codex) => source == McpSource::CodexConfig,
            Some(Platform::Gemini) => source == McpSource::AntigravityConfig,
            Some(Platform::OpenCode) => source == McpSource::OpenCode,
            Some(Platform::Grok) => source == McpSource::Grok,
            _ => false,
        }
    }

    fn detach_batch(&mut self, cx: &mut Context<Self>) {
        if let Some(batch) = self.batch.take() {
            self.last_batch = batch.read(cx).operation.clone();
        }
        self.batch_events = None;
    }

    fn open_batch(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
            platform: self.platform_scope.unwrap_or(Platform::Claude),
            generation: self.generation,
            request_id: self.generation,
        };
        let batch = cx.new(|cx| {
            batch::McpBatch::new(
                self.services.clone(),
                key,
                self.last_batch.clone(),
                window,
                cx,
            )
        });
        self.batch_events = Some(cx.subscribe(&batch, |this, _, _: &batch::Closed, cx| {
            this.detach_batch(cx);
            this.request(cx);
            cx.notify();
        }));
        self.show_sources = false;
        self.show_presets = false;
        self.batch = Some(batch);
        cx.notify();
    }

    fn request(&mut self, cx: &mut Context<Self>) {
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
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        self.busy = true;
        self.message = None;
        self.create_mode = EditMode::Add;
        self.show_sources = false;
        self.show_presets = false;
        self.catalog = None;
        self.selected = None;
        let project = self.project.read(cx).value().trim().to_string();
        let project = if project.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(project))
        };
        match self.services.mcp_project_request(key.clone(), project) {
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
                                    Completion::Success(catalog) => {
                                        this.catalog = Some(catalog.clone());
                                    }
                                    Completion::Failed(error) | Completion::CleanupFailed(error) => {
                                        this.message = Some(error.message());
                                    }
                                    Completion::Cancelled => {
                                        this.message = Some(
                                            "The MCP read did not finish. / MCP 读取没有完成",
                                        );
                                    }
                                    Completion::WorkerFailed => {
                                        this.message = Some(
                                            "The MCP reader stopped without a result. / MCP 读取没有返回结果",
                                        );
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
                self.busy = false;
                self.message = Some("Queue unavailable. Retry. / 读取队列不可用，请重试");
            }
        }
        cx.notify();
    }

    fn start_editor(
        &mut self,
        target: McpTarget,
        mode: EditMode,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        let editor =
            cx.new(|cx| McpEditor::new(self.services.clone(), key, target, mode, name, window, cx));
        self.editor_events = Some(cx.subscribe(&editor, |this, _, event, cx| {
            this.editor = None;
            this.editor_events = None;
            if matches!(event, EditorEvent::Saved) {
                this.request(cx);
            }
            cx.notify();
        }));
        self.show_sources = false;
        self.show_presets = false;
        self.editor = Some(editor);
        cx.notify();
    }

    fn add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let project = std::path::PathBuf::from(self.project.read(cx).value().trim());
        let target = match self.create_source {
            1 => McpTarget::ClaudeLocal(project),
            2 => McpTarget::ClaudeProject(project),
            3 => McpTarget::Codex,
            4 => McpTarget::Antigravity,
            _ => McpTarget::ClaudeUser,
        };
        let name = match &self.create_mode {
            EditMode::Preset(id) => id.clone(),
            EditMode::Copy(entry) => entry.name.clone(),
            _ => String::new(),
        };
        self.start_editor(target, self.create_mode.clone(), &name, window, cx);
    }

    fn visible(&self, cx: &App) -> Vec<McpEntry> {
        let Some(catalog) = &self.catalog else {
            return Vec::new();
        };
        let filter = self.filter.read(cx).value().trim().to_lowercase();
        catalog
            .entries
            .iter()
            .filter(|entry| self.source_in_scope(entry.source))
            .filter(|entry| filter.is_empty() || entry.name.to_lowercase().contains(&filter))
            .cloned()
            .collect()
    }
}

fn platform_name(source: McpSource) -> (&'static str, &'static str) {
    match source {
        McpSource::ClaudeState | McpSource::ClaudeSettings => ("Claude Code", "Claude Code"),
        McpSource::CodexConfig => ("Codex", "Codex"),
        McpSource::AntigravityConfig => ("Antigravity", "Antigravity"),
        McpSource::ProjectFile | McpSource::OpenCode | McpSource::Grok => ("MCP", "MCP"),
    }
}

fn source_title(source: McpSource) -> (&'static str, &'static str) {
    match source {
        McpSource::ClaudeState => ("Claude state file", "Claude 状态文件"),
        McpSource::ClaudeSettings => ("Claude settings.json", "Claude settings.json"),
        McpSource::CodexConfig => ("Codex config.toml", "Codex config.toml"),
        McpSource::AntigravityConfig => {
            ("Antigravity mcp_config.json", "Antigravity mcp_config.json")
        }
        McpSource::ProjectFile => ("Claude project .mcp.json", "Claude 项目 .mcp.json"),
        McpSource::OpenCode => ("OpenCode", "OpenCode"),
        McpSource::Grok => ("Grok", "Grok"),
    }
}

fn note_line(cx: &App, source: McpSource, state: McpSourceState) -> String {
    let (title_en, title_zh) = source_title(source);
    let title = text(cx, title_en, title_zh);
    match state {
        McpSourceState::Missing => {
            text(cx, "{title}: not found.", "{title}：未找到。").replace("{title}", title)
        }
        McpSourceState::Empty | McpSourceState::Entries(0) => {
            text(cx, "{title}: no MCP entries.", "{title}：没有 MCP 条目。")
                .replace("{title}", title)
        }
        McpSourceState::Entries(count) => {
            if is_chinese(cx) {
                format!("{title}：{count} 条记录。")
            } else if count == 1 {
                format!("{title}: 1 entry.")
            } else {
                format!("{title}: {count} entries.")
            }
        }
        McpSourceState::Invalid => text(
            cx,
            "{title}: not valid MCP configuration. No entries were taken from this file.",
            "{title}：不是有效的 MCP 配置。没有采用这个文件里的条目。",
        )
        .replace("{title}", title),
        McpSourceState::Unreadable => {
            text(cx, "{title}: could not be read.", "{title}：无法读取。").replace("{title}", title)
        }
        McpSourceState::NotRead => match source {
            McpSource::ProjectFile => text(
                cx,
                "Claude project .mcp.json is not read. This window has no project root.",
                "未读取 Claude 项目 .mcp.json。这个窗口没有项目根目录。",
            )
            .to_string(),
            McpSource::OpenCode => text(
                cx,
                "OpenCode MCP is not read by this page. Use the OpenCode settings source editor.",
                "此页不读取 OpenCode MCP，请使用 OpenCode 设置源码编辑器。",
            )
            .to_string(),
            McpSource::Grok => text(
                cx,
                "Grok MCP credentials are not listed as servers.",
                "Grok 的 MCP 凭据不会被列成服务器。",
            )
            .to_string(),
            McpSource::ClaudeState
            | McpSource::ClaudeSettings
            | McpSource::CodexConfig
            | McpSource::AntigravityConfig => {
                text(cx, "{title}: not read.", "{title}：未读取。").replace("{title}", title)
            }
        },
    }
}

fn bool_word(chinese: bool, value: bool) -> &'static str {
    match (chinese, value) {
        (false, true) => "true",
        (false, false) => "false",
        (true, true) => "是",
        (true, false) => "否",
    }
}

fn detail_lines(
    cx: &App,
    entry: &McpEntry,
    platform_en: &'static str,
    platform_zh: &'static str,
    source_en: &'static str,
    source_zh: &'static str,
) -> [String; 7] {
    [
        format!("{} · {}", text(cx, platform_en, platform_zh), entry.name),
        text(cx, source_en, source_zh).to_string(),
        match &entry.scope {
            McpScope::User => text(cx, "User entry.", "用户条目。").to_string(),
            McpScope::Project(key) => {
                if is_chinese(cx) {
                    format!("Claude 状态文件中的项目键：{key}")
                } else {
                    format!("Project key stored in the Claude state file: {key}")
                }
            }
        },
        match entry.command.as_deref() {
            Some(command) => {
                if is_chinese(cx) {
                    format!("命令：{command}")
                } else {
                    format!("Command: {command}")
                }
            }
            None => text(
                cx,
                "No command is set in this entry.",
                "这个条目没有设置命令。",
            )
            .to_string(),
        },
        if entry.has_url {
            text(cx, "A URL is set in this entry.", "这个条目设置了 URL。").to_string()
        } else {
            text(cx, "No URL is set in this entry.", "这个条目没有设置 URL。").to_string()
        },
        if entry.has_args {
            text(
                cx,
                "Arguments are set in this entry.",
                "这个条目设置了参数。",
            )
            .to_string()
        } else {
            text(
                cx,
                "No arguments are set in this entry.",
                "这个条目没有设置参数。",
            )
            .to_string()
        },
        flag_line(cx, entry),
    ]
}

fn flag_line(cx: &App, entry: &McpEntry) -> String {
    match (entry.disabled, entry.enabled) {
        (None, None) => text(
            cx,
            "The file does not set enabled or disabled.",
            "文件没有设置 enabled 或 disabled。",
        )
        .to_string(),
        (disabled, enabled) => {
            let chinese = is_chinese(cx);
            let mut parts = Vec::new();
            if let Some(disabled) = disabled {
                let word = bool_word(chinese, disabled);
                parts.push(if chinese {
                    format!("文件中的 disabled 为{word}。")
                } else {
                    format!("disabled is {word} in the file.")
                });
            }
            if let Some(enabled) = enabled {
                let word = bool_word(chinese, enabled);
                parts.push(if chinese {
                    format!("文件中的 enabled 为{word}。")
                } else {
                    format!("enabled is {word} in the file.")
                });
            }
            parts.join(" ")
        }
    }
}

impl Render for McpView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let visible = self.visible(cx);
        let mut root = div()
            .id("mcp-manager")
            .test_support()
            .track_focus(&self.focus)
            .tab_stop(false)
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scroll()
            .p(px(theme::CONTENT_PADDING))
            .gap(px(theme::SECTION_GAP))
            .child(
                div()
                    .id("mcp-heading")
                    .test_support()
                    .role(gpui_kit::Role::Heading)
                    .aria_level(1)
                    .text_size(px(theme::TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "MCP Manager", "MCP 管理")),
            )
            .child(text(
                cx,
                "Manage MCP entries in their selected local source and scope. Existing secret values stay out of the list and patch editor. This page does not start servers or infer process status.",
                "管理所选本地来源与作用域中的 MCP 条目。已有密钥不会出现在列表或补丁编辑器中。此页不会启动服务器或推断进程状态。",
            ))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        div().flex_1().child(
                            Input::new(&self.filter)
                                .id("mcp-filter")
                                .aria_label(text(cx, "Filter MCP entries by name", "按名称筛选 MCP 条目")),
                        ),
                    )
                    .child(semantics::button(Button::new("mcp-add").label(text(cx,"Add server and select source","添加服务器与选择来源")).ghost()
                        .on_click(cx.listener(|this,_,_,cx|{this.create_mode=EditMode::Add;this.show_presets=false;this.show_sources = !this.show_sources;cx.notify();})),self.busy||self.editor.is_some()||self.batch.is_some(),false,cx))
                    .child(semantics::button(Button::new("mcp-presets").label(text(cx,"Built-in presets","内置预设")).ghost()
                        .on_click(cx.listener(|this,_,_,cx|{this.show_sources=false;this.create_mode=EditMode::Add;this.show_presets = !this.show_presets;cx.notify();})),self.busy||self.editor.is_some()||self.batch.is_some(),false,cx))
                    .child(semantics::button(
                        Button::new("mcp-read")
                            .label(if self.busy {
                                text(cx, "Reading", "读取中")
                            } else {
                                text(cx, "Read local files", "读取本地文件")
                            })
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                        self.busy || self.editor.is_some() || self.batch.is_some(),
                        self.busy,
                        cx,
                    )),
            );
        if let Some(batch) = &self.batch {
            return root.child(batch.clone());
        }
        root = root.child(semantics::button(
            Button::new("mcp-batch-open")
                .label(text(
                    cx,
                    "Batch presets and synchronization",
                    "批量预设与同步",
                ))
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| this.open_batch(window, cx))),
            self.busy || self.editor.is_some(),
            false,
            cx,
        ));
        if let Some(editor) = &self.editor {
            return root.child(editor.clone());
        }
        if self.show_presets {
            let mut presets = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
            for preset in ccr_skills::get_builtin_presets() {
                let id = preset.id.clone();
                presets = presets.child(semantics::button(
                    Button::new(format!("mcp-preset-{id}"))
                        .label(preset.name)
                        .ghost()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.create_mode = EditMode::Preset(id.clone());
                            this.show_presets = false;
                            this.show_sources = true;
                            cx.notify();
                        })),
                    false,
                    false,
                    cx,
                ));
            }
            root = root.child(text(cx,"Choose a template, then a destination. Review commands and fill required credentials before confirming. This does not install or start a server.","先选择模板，再选择目标。确认前请检查命令并填写所需凭据。此操作不会安装或启动服务器。" )).child(presets);
        }
        if self.show_sources {
            root = root.child(match &self.create_mode {
                EditMode::Preset(id) => format!("{}: {id}", text(cx, "Preset", "预设")),
                EditMode::Copy(entry) => {
                    let (en, zh) = source_title(entry.source);
                    format!(
                        "{}: {} · {}",
                        text(cx, "Copy from", "复制来源"),
                        text(cx, en, zh),
                        entry.name
                    )
                }
                _ => text(
                    cx,
                    "Choose the source for the new server.",
                    "选择新服务器的来源。",
                )
                .to_string(),
            });
            let mut targets = crate::shell::page_header::capsule(cx);
            for (index, id, en, zh) in [
                (0, "mcp-source-user", "Claude user", "Claude 用户"),
                (
                    1,
                    "mcp-source-local",
                    "Claude local project",
                    "Claude 项目本地",
                ),
                (
                    2,
                    "mcp-source-project",
                    "Claude project file",
                    "Claude 项目文件",
                ),
                (3, "mcp-source-codex", "Codex", "Codex"),
                (4, "mcp-source-antigravity", "Antigravity", "Antigravity"),
            ] {
                if self.platform_scope.is_some()
                    && !match self.platform_scope {
                        Some(Platform::Claude) => index <= 2,
                        Some(Platform::Codex) => index == 3,
                        Some(Platform::Gemini) => index == 4,
                        _ => false,
                    }
                {
                    continue;
                }
                targets = targets.child(semantics::tab(
                    semantics::button(
                        Button::new(id)
                            .label(text(cx, en, zh))
                            .ghost()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.create_source = index;
                                cx.notify();
                            })),
                        self.busy,
                        false,
                        cx,
                    ),
                    self.create_source == index,
                    self.busy,
                    cx,
                ));
            }
            root = root.child(targets)
            .child(Input::new(&self.project).id("mcp-project-root").aria_label(text(cx,"Absolute project directory (optional)","项目绝对路径（可选）")))
            .child(text(cx,"A project directory is required for Claude local/project scope. Read local files to include its .mcp.json. User scope applies across projects.","Claude 项目本地与项目文件作用域需要项目目录。读取本地文件时会包含其 .mcp.json。用户作用域适用于所有项目。"))
            .child(semantics::button(Button::new("mcp-open-editor").label(text(cx,"Add server to selected source","向所选来源添加服务器")).primary().on_click(cx.listener(|this,_,window,cx|this.add(window,cx))),self.busy,false,cx));
        }
        if self.busy {
            root = root.child(
                div()
                    .id("mcp-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .child(text(
                        cx,
                        "Reading local MCP files.",
                        "正在读取本地 MCP 文件。",
                    )),
            );
        }
        if let Some(message) = self.message {
            root = root.child(
                div()
                    .id("mcp-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .child(service_error(cx, message)),
            );
        }
        if let Some(catalog) = self.catalog.clone() {
            let mut sources = div()
                .id("mcp-sources")
                .test_support()
                .flex()
                .flex_col()
                .gap(px(theme::CONTROL_GAP));
            for (index, item) in catalog.notes.iter().enumerate() {
                if !self.source_in_scope(item.source) {
                    continue;
                }
                let line = note_line(cx, item.source, item.state);
                sources = sources.child(
                    div()
                        .id(SharedString::from(format!("mcp-source-{index}")))
                        .test_support()
                        .aria_label(SharedString::from(line.clone()))
                        .child(line),
                );
            }
            root = root.child(sources);
            if catalog.entries.is_empty() {
                let empty = text(
                    cx,
                    "No MCP entries were found in the files this page reads.",
                    "此页读取的文件里没有 MCP 条目。",
                );
                root = root.child(
                    div()
                        .id("mcp-empty")
                        .test_support()
                        .aria_label(empty)
                        .child(empty),
                );
            } else if visible.is_empty() {
                root = root.child(div().id("mcp-filter-empty").test_support().child(text(
                    cx,
                    "No MCP entries match this filter.",
                    "没有 MCP 条目匹配这个筛选。",
                )));
            } else {
                for (index, entry) in visible.iter().enumerate() {
                    let entry_id = entry.id();
                    let selected = self.selected.as_deref() == Some(entry_id.as_str());
                    let (platform_en, platform_zh) = platform_name(entry.source);
                    let label = if entry.disabled == Some(true) {
                        format!(
                            "{} · {}{}",
                            text(cx, platform_en, platform_zh),
                            entry.name,
                            text(cx, " · disabled in the file", " · 文件中已禁用")
                        )
                    } else {
                        format!("{} · {}", text(cx, platform_en, platform_zh), entry.name)
                    };
                    let clicked = entry_id.clone();
                    root = root.child(
                        div().h(px(theme::ROW_HEIGHT)).flex().items_center().child(
                            semantics::button(
                                Button::new(format!("mcp-entry-{index}"))
                                    .label(label)
                                    .ghost()
                                    .selected(selected)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.selected = Some(clicked.clone());
                                        this.show_sources = false;
                                        this.show_presets = false;
                                        this.create_mode = EditMode::Add;
                                        cx.notify();
                                    })),
                                false,
                                false,
                                cx,
                            ),
                        ),
                    );
                }
            }
            if let Some(selected) = self.selected.clone()
                && let Some(entry) = visible.iter().find(|entry| entry.id() == selected)
            {
                let (platform_en, platform_zh) = platform_name(entry.source);
                let (source_en, source_zh) = source_title(entry.source);
                let lines = detail_lines(cx, entry, platform_en, platform_zh, source_en, source_zh);
                let summary = lines.join(" ");
                let mut detail = div()
                    .id("mcp-detail")
                    .test_support()
                    .aria_label(SharedString::from(summary))
                    .flex()
                    .flex_col()
                    .gap(px(theme::CONTROL_GAP));
                match McpTarget::for_entry(entry) {
                    Ok(target) => {
                        let edit_target = target.clone();
                        let edit_name = entry.name.clone();
                        let delete_name = entry.name.clone();
                        let copy_entry = entry.clone();
                        detail = detail.child(
                            div()
                                .flex()
                                .gap(px(theme::CONTROL_GAP))
                                .child(semantics::button(
                                    Button::new("mcp-copy")
                                        .label(text(cx, "Copy to another source", "复制到其他来源"))
                                        .ghost()
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.create_mode = EditMode::Copy(copy_entry.clone());
                                            this.show_sources = true;
                                            this.show_presets = false;
                                            cx.notify();
                                        })),
                                    self.busy,
                                    false,
                                    cx,
                                ))
                                .child(semantics::button(
                                    Button::new("mcp-edit")
                                        .label(text(cx, "Edit server", "编辑服务器"))
                                        .ghost()
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.start_editor(
                                                edit_target.clone(),
                                                EditMode::Edit,
                                                &edit_name,
                                                window,
                                                cx,
                                            )
                                        })),
                                    self.busy,
                                    false,
                                    cx,
                                ))
                                .child(semantics::button(
                                    Button::new("mcp-delete")
                                        .label(text(cx, "Delete server", "删除服务器"))
                                        .ghost()
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.start_editor(
                                                target.clone(),
                                                EditMode::Delete,
                                                &delete_name,
                                                window,
                                                cx,
                                            )
                                        })),
                                    self.busy,
                                    false,
                                    cx,
                                )),
                        );
                    }
                    Err(_) => {
                        detail = detail.child(text(cx,"This legacy source is read-only here. Manage it through the platform settings source editor; new Claude entries use explicit user/local/project scope.","此旧版来源在这里为只读。请通过平台设置源码编辑器管理；新的 Claude 条目使用明确的用户、项目本地或项目文件作用域。"));
                    }
                }
                for line in lines {
                    detail = detail.child(line);
                }
                root = root.child(detail);
            }
        }
        root
    }
}

#[cfg(test)]
mod tests;
