//! Source-aware local MCP management; server process status is not inferred.
mod batch;
mod editor;
mod header;
mod list;
mod pane;
mod sources;
use crate::{
    locales::{is_chinese, text},
    runtime::{
        Services,
        mcp::{McpCatalog, McpEntry, McpScope, McpSource, McpSourceState, McpTarget},
    },
    theme,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent};
use editor::{EditMode, EditorEvent, McpEditor};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme,
    input::{InputEvent, InputState},
};
use gpui_kit::{
    AnyView, App, AppContext, Context, Div, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, Stateful, StatefulInteractiveElement, Styled, Subscription,
    Task, Window, div, px,
};

/// A test-observable stateful div; plain `Stateful<Div>` outside tests.
type Node = gpui_kit::base::ObservedElement<Stateful<Div>>;

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
    /// 窄窗 drill-in 时把焦点移入 pane 的锚点。
    pane_focus: FocusHandle,
    platform_scope: Option<Platform>,
    /// 工具栏平台胶囊的选择；`None` 表示全部平台。
    family: Option<Family>,
    /// 外壳的 lens 胶囊；只在全局 MCP 管理页的标题行渲染。
    lens: Option<AnyView>,
    /// 列表面板的焦点锚点：窄窗返回列表后把焦点还给原来的行。
    list_focus: FocusHandle,
    /// 输入框 placeholder 当前对应的语言（是否中文），语言切换后同步一次。
    placeholder_chinese: bool,
}

/// Platform group of a source that can hold entries; drives the toolbar capsule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    Claude,
    Codex,
    Antigravity,
}

impl Family {
    const ALL: [Self; 3] = [Self::Claude, Self::Codex, Self::Antigravity];

    /// OpenCode 与 Grok 从不产生条目，因此没有分组。
    fn of(source: McpSource) -> Option<Self> {
        match source {
            McpSource::ClaudeState | McpSource::ClaudeSettings | McpSource::ProjectFile => {
                Some(Self::Claude)
            }
            McpSource::CodexConfig => Some(Self::Codex),
            McpSource::AntigravityConfig => Some(Self::Antigravity),
            McpSource::OpenCode | McpSource::Grok => None,
        }
    }

    /// 平台身份色与测试 id 共用的短 id。
    fn id(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Antigravity => "antigravity",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::Antigravity => "Antigravity",
        }
    }
}

/// What the side pane shows. One value replaces the old chain of early returns.
///
/// 优先级：batch > editor > source picker > presets > 选中详情 > Empty。
enum Pane {
    Empty,
    Detail(McpEntry),
    Editor(Entity<McpEditor>),
    Presets,
    SourcePicker,
    Batch(Entity<batch::McpBatch>),
}

fn filter_placeholder(cx: &App) -> &'static str {
    text(cx, "Search name or command", "搜索名称或命令")
}

fn project_placeholder(cx: &App) -> &'static str {
    text(cx, "Absolute path (optional)", "绝对路径（可选）")
}

impl McpView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let filter = cx.new(|cx| InputState::new(window, cx).placeholder(filter_placeholder(cx)));
        let subscription = cx.subscribe(&filter, |_, _, event, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        });
        Self {
            services,
            filter,
            project: cx.new(|cx| InputState::new(window, cx).placeholder(project_placeholder(cx))),
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
            pane_focus: cx.focus_handle().tab_stop(false),
            list_focus: cx.focus_handle().tab_stop(false),
            platform_scope: None,
            family: None,
            lens: None,
            placeholder_chinese: is_chinese(cx),
        }
    }

    /// Renders the shell's lens capsule in the heading row of the global page.
    pub fn embed_lens(&mut self, lens: AnyView) {
        self.lens = Some(lens);
    }

    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        // 主区入口仍是搜索框：标题行新增的 lens 与页头操作不抢第一焦点。
        window.focus(&self.filter.read(cx).focus_handle(cx), cx);
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
        self.services = self.services.clone().scoped(None);
        self.refresh(cx);
    }
    pub fn open_platform(&mut self, platform: Platform, cx: &mut Context<Self>) {
        self.detach_batch(cx);
        self.platform_scope = Some(platform);
        self.services = self.services.clone().scoped(Some(platform));
        // 平台作用域不渲染平台胶囊，旧的胶囊选择不能继续隐藏条目。
        self.family = None;
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
        if self.platform_scope.is_some() {
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
        self.reveal_pane(window, cx);
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
                                        // 平台的最后一个条目被删除后，胶囊回到全部。
                                        if this.active_family().is_none() {
                                            this.family = None;
                                        }
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
        self.reveal_pane(window, cx);
        cx.notify();
    }

    fn add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.allows_destination(self.create_source) {
            return;
        }
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

    fn allows_destination(&self, index: usize) -> bool {
        match self.platform_scope {
            None => index <= 4,
            Some(Platform::Claude) => index <= 2,
            Some(Platform::Codex) => index == 3,
            Some(Platform::Gemini) => index == 4,
            _ => false,
        }
    }

    /// Opens the destination picker for a new, preset-based or copied server.
    fn open_sources(&mut self, mode: EditMode, window: &mut Window, cx: &mut Context<Self>) {
        self.create_mode = mode;
        self.show_presets = false;
        self.show_sources = true;
        self.reveal_pane(window, cx);
        cx.notify();
    }

    fn open_presets(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.create_mode = EditMode::Add;
        self.show_sources = false;
        self.show_presets = true;
        self.reveal_pane(window, cx);
        cx.notify();
    }

    /// 关闭预设或来源选择；保留选中条目，pane 回到详情或空态。
    fn close_picker(&mut self, cx: &mut Context<Self>) {
        self.create_mode = EditMode::Add;
        self.show_sources = false;
        self.show_presets = false;
        cx.notify();
    }

    /// pane 标题里的取消：回到选中条目的详情；没有选中时焦点回到搜索框。
    fn cancel_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_picker(cx);
        if self.selected.is_some() {
            self.reveal_pane(window, cx);
        } else {
            window.focus(&self.filter.read(cx).focus_handle(cx), cx);
        }
    }

    fn select(&mut self, id: String, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = Some(id);
        self.close_picker(cx);
        self.reveal_pane(window, cx);
    }

    /// 窄窗从 pane 返回列表：清空选中；下一帧列表重新出现后，焦点回到原来的行，
    /// 没有原行（预设、来源选择）时回到搜索框。
    fn back_to_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let row = self.selected.as_deref().and_then(|selected| {
            self.visible(cx)
                .iter()
                .position(|entry| entry.id() == selected)
        });
        self.selected = None;
        self.close_picker(cx);
        let (view, generation) = (cx.weak_entity(), self.generation);
        window.on_next_frame(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                // 离页（suspend）或重新读取后，延迟回调不再抢焦点。
                if this.generation != generation {
                    return;
                }
                match row {
                    Some(row) => {
                        window.focus(&this.list_focus, cx);
                        for _ in 0..=row {
                            window.focus_next(cx);
                        }
                    }
                    None => window.focus(&this.filter.read(cx).focus_handle(cx), cx),
                }
            });
        });
    }

    /// 窄窗 drill-in：pane 在下一帧取代列表后，把焦点移到 pane 的第一个控件。
    fn reveal_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if window.viewport_size().width >= px(theme::SPLIT_WIDTH) {
            return;
        }
        let (view, generation) = (cx.weak_entity(), self.generation);
        window.on_next_frame(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                window.focus(&this.pane_focus, cx);
                window.focus_next(cx);
            });
        });
    }

    fn sync_placeholders(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let chinese = is_chinese(cx);
        if self.placeholder_chinese == chinese {
            return;
        }
        self.placeholder_chinese = chinese;
        let (filter, project) = (filter_placeholder(cx), project_placeholder(cx));
        self.filter
            .update(cx, |state, cx| state.set_placeholder(filter, window, cx));
        self.project
            .update(cx, |state, cx| state.set_placeholder(project, window, cx));
    }

    /// Entries inside the platform scope, before the capsule and search apply.
    fn scoped(&self) -> impl Iterator<Item = &McpEntry> {
        self.catalog
            .iter()
            .flat_map(|catalog| catalog.entries.iter())
            .filter(move |entry| self.source_in_scope(entry.source))
    }

    /// Per-platform counts from the catalog, before search; empty platforms are omitted.
    fn family_counts(&self) -> Vec<(Family, usize)> {
        Family::ALL
            .into_iter()
            .map(|family| {
                let count = self
                    .scoped()
                    .filter(|entry| Family::of(entry.source) == Some(family))
                    .count();
                (family, count)
            })
            .filter(|(_, count)| *count > 0)
            .collect()
    }

    /// 胶囊选择只在全局页且该平台仍有条目时生效，否则视为全部。
    fn active_family(&self) -> Option<Family> {
        let family = self.family?;
        if self.platform_scope.is_some() {
            return None;
        }
        self.scoped()
            .any(|entry| Family::of(entry.source) == Some(family))
            .then_some(family)
    }

    fn visible(&self, cx: &App) -> Vec<McpEntry> {
        let filter = self.filter.read(cx).value().trim().to_lowercase();
        let family = self.active_family();
        self.scoped()
            .filter(|entry| family.is_none_or(|family| Family::of(entry.source) == Some(family)))
            .filter(|entry| {
                filter.is_empty()
                    || entry.name.to_lowercase().contains(&filter)
                    || entry
                        .command
                        .as_deref()
                        .is_some_and(|command| command.to_lowercase().contains(&filter))
            })
            .cloned()
            .collect()
    }

    fn pane(&self, visible: &[McpEntry]) -> Pane {
        if let Some(batch) = &self.batch {
            return Pane::Batch(batch.clone());
        }
        if let Some(editor) = &self.editor {
            return Pane::Editor(editor.clone());
        }
        if self.show_sources {
            return Pane::SourcePicker;
        }
        if self.show_presets {
            return Pane::Presets;
        }
        // 不自动选中第一项：筛选掉选中条目后详情消失，回到空态。
        match self
            .selected
            .as_deref()
            .and_then(|selected| visible.iter().find(|entry| entry.id() == selected))
        {
            Some(entry) => Pane::Detail(entry.clone()),
            None => Pane::Empty,
        }
    }

    /// 宽窗列表与 pane 3:2 并排；窄窗 pane 有内容时只渲染 pane（drill-in），否则只渲染列表。
    fn body(&self, split: bool, visible: &[McpEntry], pane: Pane, cx: &mut Context<Self>) -> Node {
        let body = div()
            .id("mcp-body")
            .test_support()
            .flex_1()
            .min_h_0()
            .flex()
            .gap(px(theme::SECTION_GAP));
        let empty = matches!(pane, Pane::Empty);
        // 没有可选条目时，宽窗也只放列表列（空态面板占满），不并排第二块空面板。
        let beside = split && (!empty || self.scoped().next().is_some());
        if beside {
            body.flex_row()
                .child(self.list_panel(visible, true, cx))
                .child(self.pane_panel(pane, true, cx))
        } else if empty {
            body.flex_col().child(self.list_panel(visible, false, cx))
        } else {
            body.flex_col().child(self.pane_panel(pane, false, cx))
        }
    }
}

/// 实色阅读面板：列表与 pane 共用同一描边、圆角与独立纵向滚动。
fn reading_panel(id: &'static str, cx: &App) -> Node {
    div()
        .id(id)
        .test_support()
        .min_h_0()
        .flex()
        .flex_col()
        .overflow_y_scroll()
        .bg(cx.theme().background)
        .border_1()
        .border_color(cx.theme().border)
        .rounded(px(theme::PANEL_RADIUS))
}

/// 平台名在中英文下相同；Claude 项目 .mcp.json 也属于 Claude Code。
fn platform_name(source: McpSource) -> (&'static str, &'static str) {
    let name = Family::of(source).map_or("MCP", Family::name);
    (name, name)
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_placeholders(window, cx);
        let width = window.viewport_size().width;
        let split = width >= px(theme::SPLIT_WIDTH);
        let visible = self.visible(cx);
        let pane = self.pane(&visible);
        let root = div()
            .id("mcp-manager")
            .test_support()
            .track_focus(&self.focus)
            .tab_stop(false)
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .p(px(theme::content_padding(width)))
            .gap(px(theme::SECTION_GAP))
            .child(self.header(cx))
            .child(self.toolbar(cx));
        let root = match self.sources(cx) {
            Some(sources) => root.child(sources),
            None => root,
        };
        root.child(self.body(split, &visible, pane, cx))
    }
}

#[cfg(test)]
mod tests;
