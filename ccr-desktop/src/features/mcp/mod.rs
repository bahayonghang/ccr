//! Local MCP catalog. Reads existing files and does not invent servers or status.
use crate::{
    locales::{is_chinese, service_error, text},
    runtime::{
        Services,
        mcp::{McpCatalog, McpEntry, McpScope, McpSource, McpSourceState},
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent};
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
    _filter: Subscription,
    catalog: Option<McpCatalog>,
    selected: Option<String>,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    busy: bool,
    message: Option<&'static str>,
    focus: FocusHandle,
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
            _filter: subscription,
            catalog: None,
            selected: None,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            busy: false,
            message: None,
            focus: cx.focus_handle().tab_stop(false),
        }
    }

    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        window.focus_next(cx);
    }

    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.generation += 1;
        self.busy = false;
        cx.notify();
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.request(cx);
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
        self.catalog = None;
        self.selected = None;
        match self.services.mcp_request(key.clone()) {
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

    fn visible(&self, cx: &App) -> Vec<McpEntry> {
        let Some(catalog) = &self.catalog else {
            return Vec::new();
        };
        let filter = self.filter.read(cx).value().trim().to_lowercase();
        catalog
            .entries
            .iter()
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
                "OpenCode MCP is not read by this page.",
                "此页不读取 OpenCode MCP。",
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
                "Reads MCP entries stored in local files. It does not add, remove, or report whether a server is running. Environment values, headers, arguments, URLs, and tokens stay in the file.",
                "读取已经写在本地文件里的 MCP 条目。不会新增、删除，也不会报告服务器是否在运行。环境变量、请求头、参数、URL 和令牌仍留在文件中。",
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
                    .child(semantics::button(
                        Button::new("mcp-read")
                            .label(if self.busy {
                                text(cx, "Reading", "读取中")
                            } else {
                                text(cx, "Read local files", "读取本地文件")
                            })
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                        self.busy,
                        self.busy,
                        cx,
                    )),
            );
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
