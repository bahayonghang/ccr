//! Existing fixed instruction files, with a scoped plaintext edit session.
pub mod claude_extensions;
pub mod claude_files;
pub mod codex_agents;
pub mod platform_files;
pub mod templates;
use crate::{
    locales::{service_error, text},
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
    ParentElement, Render, StatefulInteractiveElement, Styled, Subscription, Task, Window, div, px,
};
use std::sync::Arc;

#[derive(Clone, Copy)]
enum Decision {
    Open(&'static str),
    Reload,
    /// 含凭据的旧版 Claude 数组完整副本。
    LegacyFullExport,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ResourceSurface {
    ClaudeExtensions,
    OpenCode,
    Antigravity,
    Instructions,
    Claude,
    Codex,
}

pub struct ResourcesView {
    services: Services,
    platform: Platform,
    rows: Vec<PromptRow>,
    extension_sources: Vec<ccr_app::resources::claude_extensions::Source>,
    /// 旧版条目的临时导出：只在本视图内显示，离页即清空。
    legacy_output: Entity<TextareaState>,
    legacy_pending: Option<String>,
    /// `Some(true)` 表示当前临时输出含凭据。
    legacy_full: Option<bool>,
    pinned: bool,
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
    surface: ResourceSurface,
    opencode: Entity<platform_files::PlatformFilesView>,
    antigravity: Entity<platform_files::PlatformFilesView>,
    _opencode_observation: Subscription,
    _antigravity_observation: Subscription,
    claude: Entity<claude_files::ClaudeResourcesView>,
    codex: Entity<codex_agents::CodexAgentsView>,
    _claude_observation: Subscription,
    _codex_observation: Subscription,
    templates: Entity<templates::TemplatesView>,
    _template_events: Subscription,
    templates_open: bool,
    template_cleanup: bool,
}
impl ResourcesView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let opencode = cx.new(|cx| {
            platform_files::PlatformFilesView::new(
                services.clone(),
                crate::runtime::resources::platform_files::Kind::OpenCodeAgent,
                window,
                cx,
            )
        });
        let antigravity = cx.new(|cx| {
            platform_files::PlatformFilesView::new(
                services.clone(),
                crate::runtime::resources::platform_files::Kind::AntigravityCommand,
                window,
                cx,
            )
        });
        let opencode_observation = cx.observe(&opencode, |_, _, cx| cx.notify());
        let antigravity_observation = cx.observe(&antigravity, |_, _, cx| cx.notify());
        let claude =
            cx.new(|cx| claude_files::ClaudeResourcesView::new(services.clone(), window, cx));
        let codex = cx.new(|cx| codex_agents::CodexAgentsView::new(services.clone(), window, cx));
        let claude_observation = cx.observe(&claude, |_, _, cx| cx.notify());
        let codex_observation = cx.observe(&codex, |_, _, cx| cx.notify());
        let templates = cx.new(|cx| templates::TemplatesView::new(services.clone(), window, cx));
        let template_events = cx.subscribe(&templates, |this, _, event, cx| {
            if let templates::TemplateEvent::Use(content) = event {
                this.pending_content = Some(content.clone());
                this.editor_entry_pending = true;
            }
            this.templates_open = false;
            this.template_cleanup = true;
            cx.notify();
        });
        Self {
            services,
            platform: Platform::Claude,
            rows: Vec::new(),
            extension_sources: Vec::new(),
            legacy_output: cx.new(|cx| TextareaState::new(window, cx)),
            legacy_pending: None,
            legacy_full: None,
            pinned: false,
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
            surface: ResourceSurface::Instructions,
            opencode,
            antigravity,
            _opencode_observation: opencode_observation,
            _antigravity_observation: antigravity_observation,
            claude,
            codex,
            _claude_observation: claude_observation,
            _codex_observation: codex_observation,
            templates,
            _template_events: template_events,
            templates_open: false,
            template_cleanup: false,
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.templates_open {
            self.templates
                .update(cx, |view, cx| view.focus_main(window, cx));
            return;
        }
        if self.surface == ResourceSurface::OpenCode {
            self.opencode
                .update(cx, |view, cx| view.focus_main(window, cx));
            return;
        }
        if self.surface == ResourceSurface::Antigravity {
            self.antigravity
                .update(cx, |view, cx| view.focus_main(window, cx));
            return;
        }
        if self.surface == ResourceSurface::Claude {
            self.claude
                .update(cx, |view, cx| view.focus_main(window, cx));
            return;
        }
        if self.surface == ResourceSurface::Codex {
            self.codex
                .update(cx, |view, cx| view.focus_main(window, cx));
            return;
        }
        if self.session.is_some() {
            window.focus(&self.editor.read(cx).focus_handle(cx), cx);
        } else {
            window.focus(&self.focus, cx);
            window.focus_next(cx);
        }
    }
    pub fn load(&mut self, cx: &mut Context<Self>) {
        match self.surface {
            ResourceSurface::ClaudeExtensions => {
                self.request(ResourceCommand::ListClaudeExtensions, cx);
                return;
            }
            ResourceSurface::OpenCode => {
                self.opencode.update(cx, |view, cx| view.load(cx));
                return;
            }
            ResourceSurface::Antigravity => {
                self.antigravity.update(cx, |view, cx| view.load(cx));
                return;
            }
            ResourceSurface::Claude => {
                self.claude.update(cx, |view, cx| view.load(cx));
                return;
            }
            ResourceSurface::Codex => {
                self.codex.update(cx, |view, cx| view.activate(cx));
                return;
            }
            ResourceSurface::Instructions => {}
        }
        self.request(ResourceCommand::ListPrompts, cx);
    }
    pub fn open_platform(
        &mut self,
        platform: Platform,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.leave(window, cx);
        self.pinned = true;
        self.platform = platform;
        self.load(cx);
    }
    pub fn open_claude(
        &mut self,
        kind: crate::runtime::resources::claude_files::Kind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.leave(window, cx);
        self.platform = Platform::Claude;
        self.surface = ResourceSurface::Claude;
        self.claude
            .update(cx, |view, cx| view.open_kind(kind, window, cx));
    }
    pub fn open_extensions(
        &mut self,
        platform: Platform,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.leave(window, cx);
        self.platform = platform;
        self.pinned = true;
        self.surface = match platform {
            Platform::Codex => ResourceSurface::Codex,
            Platform::Gemini => ResourceSurface::Antigravity,
            Platform::OpenCode => ResourceSurface::OpenCode,
            _ => ResourceSurface::ClaudeExtensions,
        };
        self.load(cx);
    }
    pub fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.templates.update(cx, |view, cx| view.leave(window, cx));
        self.templates_open = false;
        self.template_cleanup = false;
        self.opencode.update(cx, |view, cx| view.leave(window, cx));
        self.antigravity
            .update(cx, |view, cx| view.leave(window, cx));
        self.claude.update(cx, |view, cx| view.leave(window, cx));
        self.codex.update(cx, |view, cx| view.suspend(window, cx));
        self.surface = ResourceSurface::Instructions;
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
        self.clear_legacy_output(window, cx);
        cx.notify();
    }
    fn blocks_switch(&self, cx: &Context<Self>) -> bool {
        self.busy
            || self.session.is_some()
            || self.decision.is_some()
            || match self.surface {
                ResourceSurface::OpenCode => self.opencode.read(cx).blocks_switch(),
                ResourceSurface::Antigravity => self.antigravity.read(cx).blocks_switch(),
                ResourceSurface::Claude => self.claude.read(cx).blocks_switch(),
                ResourceSurface::Codex => self.codex.read(cx).blocks_switch(),
                ResourceSurface::Instructions | ResourceSurface::ClaudeExtensions => false,
            }
    }
    fn select_surface(
        &mut self,
        surface: ResourceSurface,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.blocks_switch(cx) {
            return;
        }
        self.leave(window, cx);
        self.surface = surface;
        self.load(cx);
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
                                Completion::Success(ResourceResult::Extensions(sources)) => this.extension_sources=sources.clone(),
                                Completion::Success(ResourceResult::LegacyExport { content, full }) => this.accept_legacy_export(content, *full),
                                Completion::Success(ResourceResult::ExportSaved { conflict }) => this.message = Some(if *conflict {
                                    "A file already exists there; nothing was written. Choose a new file. / 该位置已有文件，未写入，请选择新文件。"
                                } else { "Export file saved; Claude settings unchanged. / 导出文件已保存，Claude 设置未改动。" }),
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
            Decision::LegacyFullExport => {
                self.dismiss(window, cx);
                self.request(ResourceCommand::ExportClaudeLegacy { full: true }, cx);
                return;
            }
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
        if std::mem::take(&mut self.template_cleanup) {
            self.templates.update(cx, |view, cx| view.leave(window, cx));
        }
        if let Some(content) = self.pending_content.take() {
            self.editor
                .update(cx, |state, cx| state.set_value(content, window, cx));
        }
        if let Some(content) = self.legacy_pending.take() {
            self.legacy_output
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
        let switch_blocked = self.blocks_switch(cx);
        let mut root = div().id("resources-view").test_support().track_focus(&self.focus).tab_stop(false).flex().flex_col().size_full()
            .p(px(theme::CONTENT_PADDING)).gap(px(theme::SECTION_GAP))
            .on_key_down(cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" && this.decision.is_some() { this.dismiss(window, cx); cx.stop_propagation(); }
            }))

            .child(div().flex_shrink_0().child(text(cx, "Local resources, original revision and managed backup. Remote environments do not fall back to Local.", "本地资源、原始版本与托管备份。远程环境不会回退本地。")));
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
                switch_blocked,
                false,
                cx,
            ));
        }
        if !self.pinned {
            root = root.child(platforms.flex_shrink_0());
        }
        let mut surfaces = div()
            .flex()
            .flex_wrap()
            .flex_shrink_0()
            .gap(px(theme::CONTROL_GAP))
            .child(semantics::button(
                Button::new("resource-instructions")
                    .label(text(cx, "Instruction files", "指令文件"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.select_surface(ResourceSurface::Instructions, window, cx)
                    })),
                switch_blocked,
                false,
                cx,
            ));
        if !self.pinned {
            let (surface, id, label) = match self.platform {
                Platform::OpenCode => (
                    ResourceSurface::OpenCode,
                    "resource-opencode",
                    "OpenCode files",
                ),
                Platform::Gemini => (
                    ResourceSurface::Antigravity,
                    "resource-antigravity",
                    "Antigravity compatibility files",
                ),
                Platform::Codex => (
                    ResourceSurface::Codex,
                    "resource-codex-agents",
                    "Codex agents and sources",
                ),
                _ => (
                    ResourceSurface::Claude,
                    "resource-claude",
                    "Claude rules and output styles",
                ),
            };
            surfaces =
                surfaces.child(semantics::ghost_button(
                    Button::new(id).label(label).ghost().on_click(cx.listener(
                        move |this, _, window, cx| this.select_surface(surface, window, cx),
                    )),
                    switch_blocked,
                    false,
                    cx,
                ));
            root = root.child(surfaces);
        } else if matches!(
            self.surface,
            ResourceSurface::Instructions | ResourceSurface::Claude
        ) && self.platform == Platform::Claude
        {
            root = root.child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .child(semantics::ghost_button(
                        Button::new("claude-instructions-files")
                            .label(text(cx, "Instruction files", "指令文件"))
                            .ghost()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.select_surface(ResourceSurface::Instructions, window, cx)
                            })),
                        switch_blocked,
                        false,
                        cx,
                    ))
                    .child(semantics::ghost_button(
                        Button::new("claude-instructions-rules")
                            .label(text(cx, "Rules", "规则"))
                            .ghost()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_claude(
                                    crate::runtime::resources::claude_files::Kind::Rules,
                                    window,
                                    cx,
                                )
                            })),
                        switch_blocked,
                        false,
                        cx,
                    ))
                    .child(semantics::ghost_button(
                        Button::new("claude-instructions-output-styles")
                            .label(text(cx, "Output styles", "输出样式"))
                            .ghost()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_claude(
                                    crate::runtime::resources::claude_files::Kind::OutputStyles,
                                    window,
                                    cx,
                                )
                            })),
                        switch_blocked,
                        false,
                        cx,
                    )),
            );
        }
        match self.surface {
            ResourceSurface::ClaudeExtensions => {
                let content = self.render_extensions(window, cx);
                return root.child(content).overflow_y_scroll();
            }
            ResourceSurface::OpenCode => {
                return root.child(div().flex_1().min_h_0().child(self.opencode.clone()));
            }
            ResourceSurface::Antigravity => {
                return root.child(div().flex_1().min_h_0().child(self.antigravity.clone()));
            }
            ResourceSurface::Claude => {
                return root.child(div().flex_1().min_h_0().child(self.claude.clone()));
            }
            ResourceSurface::Codex => {
                return root.child(div().flex_1().min_h_0().child(self.codex.clone()));
            }
            ResourceSurface::Instructions => {}
        }
        if self.templates_open {
            return root.child(div().flex_1().min_h_0().child(self.templates.clone()));
        }
        if let Some(decision) = self.decision {
            self.schedule_decision_focus(window, cx);
            root = root.child(div().id("resource-confirmation").test_support().role(gpui_kit::Role::AlertDialog)
                .aria_label(text(cx, "Confirm scoped resource editor", "确认临时资源编辑"))
                .child(match decision {
                    Decision::LegacyFullExport => Self::legacy_confirm_text(cx),
                    Decision::Open(_) => text(cx, "Open plaintext instruction content? It remains in this scoped editor. Do not include private text in screenshots or logs.", "打开明文指令内容？仅保留在临时编辑会话，请勿将私密文本记录或截图。"),
                    Decision::Reload => text(cx, "Discard this draft and reload the current file?", "丢弃当前草稿并重新读取文件？"),
                }))
                .child(div().tab_group().tab_stop(false).flex().gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(Button::new("resource-decision-cancel").label(text(cx, "Cancel", "取消")).ghost()
                        .on_click(cx.listener(|this, _, window, cx| this.dismiss(window, cx))), false, false, cx))
                    .child(semantics::button(Button::new("resource-decision-confirm").label(text(cx, "Confirm", "确认"))
                        .on_click(cx.listener(|this, _, window, cx| this.confirm(window, cx))), false, false, cx))
                    .focus_trap("resource-decision-trap", &self.confirmation_focus));
        }
        if self.session.is_some() && self.platform == Platform::OpenCode {
            root = root.child(text(cx, "Saved prompt presets support Claude, Codex and Gemini. Edit this instruction file directly.", "提示词预设支持 Claude、Codex 和 Gemini，请直接编辑当前指令文件。"));
        }
        if let Some(session) = self.session.clone() {
            root = root
                .child(
                    div()
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new("resource-editor-close")
                                .label(text(cx, "Back", "返回"))
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
                            Button::new("resource-templates")
                                .label(text(cx, "Templates and presets", "模板与预设"))
                                .ghost()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    if !matches!(
                                        this.platform,
                                        Platform::Claude | Platform::Codex | Platform::Gemini
                                    ) {
                                        return;
                                    }
                                    let Some(session) = this.session.as_ref() else {
                                        return;
                                    };
                                    let target = match session.spec.filename {
                                        "CLAUDE.md" => ccr_skills::PromptTarget::Claude,
                                        "GEMINI.md" => ccr_skills::PromptTarget::Gemini,
                                        _ => ccr_skills::PromptTarget::Agents,
                                    };
                                    this.templates_open = true;
                                    this.templates.update(cx, |view, cx| {
                                        view.open(this.platform, target, window, cx)
                                    });
                                    cx.notify();
                                })),
                            blocked
                                || !matches!(
                                    self.platform,
                                    Platform::Claude | Platform::Codex | Platform::Gemini
                                ),
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("resource-reload")
                                .label(text(cx, "Discard and reload", "丢弃并重读"))
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
                        text(cx, "Existing", "已存在")
                    } else {
                        text(cx, "Missing", "尚未创建")
                    }
                ))
                .child(
                    Textarea::new(&self.editor)
                        .accessibility_id("resource-content")
                        .aria_label(text(cx, "Scoped instruction file", "临时指令文件"))
                        .flex_1()
                        .readonly(blocked),
                )
                .child(semantics::button(
                    Button::new("resource-save")
                        .label(text(cx, "Save original revision", "按原始版本保存"))
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
                        .label(text(cx, "Create empty file", "创建空文件"))
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
                root = root.child(text(
                    cx,
                    "Codex reads at most 32KiB of user instructions.",
                    "Codex 用户指令读取上限为 32KiB。",
                ));
            }
        } else {
            root = root.child(semantics::button(
                Button::new("resources-refresh")
                    .label(text(cx, "Refresh", "刷新"))
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
                        .h(crate::theme::text_size(theme::ROW_HEIGHT))
                        .flex()
                        .items_center()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new(format!("resource-open-{id}"))
                                .label(format!(
                                    "{} · {}",
                                    row.filename,
                                    if row.exists {
                                        text(cx, "Existing", "已存在")
                                    } else {
                                        text(cx, "Missing", "尚未创建")
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
            root = root.child(text(
                cx,
                "Working on a background worker.",
                "正在后台处理。",
            ));
        }
        if let Some(message) = self.message {
            root = root.child(
                div()
                    .id("resources-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .child(service_error(cx, message)),
            );
        }
        root
    }
}

#[cfg(test)]
mod tests;
