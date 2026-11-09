//! Side pane: selected detail, presets, destination picker, editor or batch.
use super::{
    EditMode, McpView, Node, Pane, detail_lines,
    list::{column_size, scope_short, transport_icon},
    platform_name, reading_panel, source_title,
};
use crate::{
    locales::{is_chinese, text},
    runtime::mcp::{McpEntry, McpScope, McpTarget},
    shell::{badge, page_header, semantics},
    theme::{self, Tone},
};
use ccr_skills::McpPreset;
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, Icon, IconName,
    button::{Button, ButtonVariants},
    input::Input,
    tooltip::Tooltip,
};
use gpui_kit::{
    AnyElement, App, Context, Div, FontWeight, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, div, px, relative,
};

impl McpView {
    /// The right column in split mode, or the whole body during a narrow drill-in.
    pub(super) fn pane_panel(
        &self,
        pane: Pane,
        beside: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if matches!(pane, Pane::Empty) {
            return column_size(self.empty_pane(cx), beside, 2., theme::DETAIL_MIN_WIDTH)
                .into_any_element();
        }
        let panel = reading_panel("mcp-pane", cx)
            .track_focus(&self.pane_focus)
            .tab_stop(false)
            .p(px(theme::SECTION_GAP))
            .gap(px(theme::SECTION_GAP));
        // 窄窗详情 drill-in 需要明确的返回；编辑器与批量用各自的关闭按钮。
        let panel = if !beside && matches!(pane, Pane::Detail(_)) {
            panel.child(
                semantics::button(
                    Button::new("mcp-pane-back")
                        .icon(IconName::ArrowLeft)
                        .label(text(cx, "Back to list", "返回列表"))
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| this.back_to_list(window, cx))),
                    false,
                    false,
                    cx,
                )
                .self_start(),
            )
        } else {
            panel
        };
        let panel = match pane {
            Pane::Empty => panel,
            Pane::Detail(entry) => panel.child(self.detail(&entry, cx)),
            Pane::Editor(editor) => panel.child(editor),
            Pane::Presets => panel.child(self.presets(cx)),
            Pane::SourcePicker => panel.child(self.source_picker(cx)),
            Pane::Batch(batch) => panel.child(batch),
        };
        column_size(panel, beside, 2., theme::DETAIL_MIN_WIDTH).into_any_element()
    }

    /// 宽窗未选中：说明下一步，并只用 catalog 的数字汇总读到的条目。
    fn empty_pane(&self, cx: &mut Context<Self>) -> Node {
        let count = self.scoped().count();
        let read = if is_chinese(cx) {
            format!("已从本地文件读取 {count} 条记录。")
        } else if count == 1 {
            "1 entry was read from local files.".to_string()
        } else {
            format!("{count} entries were read from local files.")
        };
        let body = format!(
            "{} {read} {}",
            text(
                cx,
                "Select a server to see its source, scope and actions.",
                "选择一个服务器，查看它的来源、作用域与操作。",
            ),
            text(
                cx,
                "This page does not start servers, and stored secrets stay hidden.",
                "此页不会启动服务器，已保存的密钥保持隐藏。",
            )
        );
        badge::empty_panel(
            "mcp-pane-empty",
            IconName::Info,
            text(cx, "No server selected", "未选择服务器"),
            body,
            None,
            cx,
        )
        .test_support()
    }

    fn detail(&self, entry: &McpEntry, cx: &mut Context<Self>) -> Node {
        let (platform_en, platform_zh) = platform_name(entry.source);
        let (source_en, source_zh) = source_title(entry.source);
        let lines = detail_lines(cx, entry, platform_en, platform_zh, source_en, source_zh);
        div()
            .id("mcp-detail")
            .test_support()
            .aria_label(SharedString::from(lines.join(" ")))
            .flex()
            .flex_col()
            .gap(px(theme::SECTION_GAP))
            .child(detail_head(entry, cx))
            .child(self.detail_actions(entry, cx))
            .child(detail_grid(entry, cx))
    }

    fn detail_actions(&self, entry: &McpEntry, cx: &mut Context<Self>) -> AnyElement {
        let Ok(target) = McpTarget::for_entry(entry) else {
            return read_only_note(cx).into_any_element();
        };
        let copy = entry.clone();
        let edit = (target.clone(), entry.name.clone());
        let delete = (target, entry.name.clone());
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(semantics::button(
                Button::new("mcp-copy")
                    .icon(IconName::Copy)
                    .label(text(cx, "Copy to…", "复制到…"))
                    .tooltip(text(cx, "Copy to another source", "复制到其他来源"))
                    .ghost()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_sources(EditMode::Copy(copy.clone()), window, cx)
                    })),
                self.busy,
                false,
                cx,
            ))
            .child(semantics::button(
                Button::new("mcp-edit")
                    .label(text(cx, "Edit", "编辑"))
                    .ghost()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.start_editor(edit.0.clone(), EditMode::Edit, &edit.1, window, cx)
                    })),
                self.busy,
                false,
                cx,
            ))
            .child(semantics::button(
                Button::new("mcp-delete")
                    .label(text(cx, "Delete", "删除"))
                    .ghost()
                    .text_color(cx.theme().danger)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.start_editor(delete.0.clone(), EditMode::Delete, &delete.1, window, cx)
                    })),
                self.busy,
                false,
                cx,
            ))
            .into_any_element()
    }

    /// 16/600 pane title with the picker's Cancel action on the right.
    fn pane_heading(&self, title: &'static str, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .min_w_0()
                    .text_size(px(theme::SECTION_TITLE_SIZE))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
            .child(semantics::button(
                Button::new("mcp-pane-cancel")
                    .icon(IconName::Close)
                    .label(text(cx, "Cancel", "取消"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| this.cancel_picker(window, cx))),
                false,
                false,
                cx,
            ))
    }

    fn presets(&self, cx: &mut Context<Self>) -> Node {
        let hairline = theme::hairline(cx.theme().mode);
        let mut list = div().flex().flex_col();
        for (index, preset) in ccr_skills::get_builtin_presets().into_iter().enumerate() {
            if index > 0 {
                list = list.child(
                    div()
                        .flex_shrink_0()
                        .mx(px(theme::CONTROL_GAP))
                        .h(px(1.))
                        .bg(hairline),
                );
            }
            list = list.child(preset_row(preset, cx));
        }
        div()
            .id("mcp-preset-list")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP + 4.))
            .child(self.pane_heading(text(cx, "Built-in presets", "内置预设"), cx))
            .child(muted_line(
                text(
                    cx,
                    "Choose a template, then a destination. Review the command and fill required credentials before confirming. Nothing is installed or started.",
                    "先选择模板，再选择目标。确认前请检查命令并填写所需凭据。不会安装或启动服务器。",
                ),
                cx,
            ))
            .child(list)
    }

    fn source_picker(&self, cx: &mut Context<Self>) -> Node {
        let (title, context) = match &self.create_mode {
            EditMode::Preset(id) => (
                text(cx, "Add preset", "添加预设"),
                format!("{}: {id}", text(cx, "Preset", "预设")),
            ),
            EditMode::Copy(entry) => {
                let (en, zh) = source_title(entry.source);
                (
                    text(cx, "Copy server", "复制服务器"),
                    format!(
                        "{}: {} · {}",
                        text(cx, "Copy from", "复制来源"),
                        text(cx, en, zh),
                        entry.name
                    ),
                )
            }
            _ => (
                text(cx, "Add server", "添加服务器"),
                text(
                    cx,
                    "Choose the source for the new server.",
                    "选择新服务器的来源。",
                )
                .to_string(),
            ),
        };
        div()
            .id("mcp-source-picker")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP + 4.))
            .child(self.pane_heading(title, cx))
            .child(muted_line(context, cx))
            .child(self.destinations(cx))
            .child(self.project_field(cx))
            .child(
                div().flex().child(semantics::button(
                    Button::new("mcp-open-editor")
                        .label(text(cx, "Open editor", "打开编辑器"))
                        .primary()
                        .on_click(cx.listener(|this, _, window, cx| this.add(window, cx))),
                    self.busy,
                    false,
                    cx,
                )),
            )
    }

    /// Visible label plus the existing five destination tabs, filtered by platform scope.
    fn destinations(&self, cx: &mut Context<Self>) -> Div {
        let mut targets = page_header::capsule(cx);
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
            let allowed = match self.platform_scope {
                None => true,
                Some(ccr_app::runtime::Platform::Claude) => index <= 2,
                Some(ccr_app::runtime::Platform::Codex) => index == 3,
                Some(ccr_app::runtime::Platform::Gemini) => index == 4,
                _ => false,
            };
            if allowed {
                targets = targets.child(destination_tab(
                    index,
                    id,
                    text(cx, en, zh),
                    self.create_source == index,
                    self.busy,
                    cx,
                ));
            }
        }
        field(
            "mcp-destination-label",
            text(cx, "Destination", "目标来源"),
            targets,
            cx,
        )
    }

    fn project_field(&self, cx: &mut Context<Self>) -> Div {
        field(
            "mcp-project-root-label",
            text(cx, "Project directory", "项目目录"),
            Input::new(&self.project)
                .id("mcp-project-root")
                .aria_label(text(
                    cx,
                    "Absolute project directory (optional)",
                    "项目绝对路径（可选）",
                )),
            cx,
        )
        .child(muted_line(
            text(
                cx,
                "Required for Claude local and project-file scope. Reload to include that project's .mcp.json. User scope applies across projects.",
                "Claude 项目本地与项目文件作用域需要项目目录。重新读取时会包含其 .mcp.json。用户作用域适用于所有项目。",
            ),
            cx,
        ))
    }
}

// 选项构建放在独立函数中，避免增大 render 栈帧（Windows UI 线程只有 1 MiB）。
fn destination_tab(
    index: usize,
    id: &'static str,
    label: &'static str,
    selected: bool,
    busy: bool,
    cx: &mut Context<McpView>,
) -> Button {
    semantics::tab(
        semantics::button(
            Button::new(id)
                .label(label)
                .ghost()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.create_source = index;
                    cx.notify();
                })),
            busy,
            false,
            cx,
        ),
        selected,
        busy,
        cx,
    )
}

/// 12/500 可见字段标签在上，控件在下。
fn field(id: &'static str, label: &'static str, control: impl IntoElement, cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(theme::SELECTION_RADIUS + 2.))
        .child(
            div()
                .id(id)
                .test_support()
                .role(gpui_kit::Role::Label)
                .aria_label(label)
                .text_size(px(theme::META_SIZE))
                .font_weight(FontWeight::MEDIUM)
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .child(control)
}

fn muted_line(line: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_size(px(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .child(line.into())
}

fn read_only_note(cx: &App) -> Div {
    let muted = cx.theme().muted_foreground;
    div()
        .flex()
        .items_start()
        .gap(px(theme::CONTROL_GAP))
        .text_size(px(theme::META_SIZE))
        .text_color(muted)
        .child(
            Icon::new(IconName::Eye)
                .size(px(theme::NAV_ICON_SIZE))
                .text_color(muted),
        )
        .child(div().flex_1().min_w_0().child(text(
            cx,
            "This legacy source is read-only here. Manage it through the platform settings source editor; new Claude entries use explicit user/local/project scope.",
            "此旧版来源在这里为只读。请通过平台设置源码编辑器管理；新的 Claude 条目使用明确的用户、项目本地或项目文件作用域。",
        )))
}

/// 40px 传输图标方块 + 眉题（平台 · 来源文件 · 作用域）+ 20/600 名称。
fn detail_head(entry: &McpEntry, cx: &App) -> Div {
    let (platform_en, platform_zh) = platform_name(entry.source);
    let (source_en, source_zh) = source_title(entry.source);
    let eyebrow = format!(
        "{} · {} · {}",
        text(cx, platform_en, platform_zh),
        text(cx, source_en, source_zh),
        scope_short(cx, &entry.scope)
    );
    let primary = cx.theme().primary;
    div()
        .flex()
        .items_center()
        .gap(px(theme::CONTROL_GAP + 4.))
        .child(
            div()
                .flex_shrink_0()
                .size(px(40.))
                .rounded(px(theme::INPUT_RADIUS))
                .bg(primary.opacity(0.1))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Icon::new(transport_icon(entry))
                        .size(px(20.))
                        .text_color(primary),
                ),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(muted_line(eyebrow, cx))
                .child(
                    div()
                        .text_size(px(theme::TITLE_SIZE))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(entry.name.clone()),
                ),
        )
}

/// 键值网格：标签 12 muted 在上，值 14 在下；两列，窄时自动单列。
fn detail_grid(entry: &McpEntry, cx: &App) -> Div {
    let not_set = text(cx, "Not set", "未设置");
    let hidden = text(cx, "Set (hidden)", "已设置（隐藏）");
    let transport = match (&entry.command, entry.has_url) {
        (Some(_), _) => "stdio",
        (None, true) => "URL",
        (None, false) => not_set,
    };
    let (source_en, source_zh) = source_title(entry.source);
    let (scope, project) = match &entry.scope {
        McpScope::User => (text(cx, "User", "用户").to_string(), false),
        McpScope::Project(key) => (key.clone(), true),
    };
    let mut grid = div().flex().flex_wrap().gap(px(theme::SECTION_GAP));
    // 命令常是长路径：独占整行，避免在半宽单元格里折成多行把后续字段挤出 pane。
    // State 只有在引用文件原文（disabled = …）时用 mono，说明句用正文字体。
    let state = state_value(cx, entry);
    let state_is_raw = entry.disabled.is_some() || entry.enabled.is_some();
    for (label, value, mono, wide) in [
        (
            text(cx, "Transport", "传输"),
            transport.to_string(),
            false,
            false,
        ),
        (
            text(cx, "Source", "来源"),
            text(cx, source_en, source_zh).to_string(),
            false,
            false,
        ),
        (
            text(cx, "Command", "命令"),
            entry.command.clone().unwrap_or_else(|| not_set.to_string()),
            entry.command.is_some(),
            entry.command.is_some(),
        ),
        (
            "URL",
            if entry.has_url { hidden } else { not_set }.to_string(),
            false,
            false,
        ),
        (
            text(cx, "Arguments", "参数"),
            if entry.has_args { hidden } else { not_set }.to_string(),
            false,
            false,
        ),
        (text(cx, "Scope", "作用域"), scope, project, project),
        (text(cx, "State", "状态"), state, state_is_raw, false),
    ] {
        grid = grid.child(cell(label, value, mono, wide, cx));
    }
    grid
}

/// 文件里 enabled / disabled 的原文语义；未设置时写明未设置。
fn state_value(cx: &App, entry: &McpEntry) -> String {
    let mut parts = Vec::new();
    if let Some(disabled) = entry.disabled {
        parts.push(format!("disabled = {disabled}"));
    }
    if let Some(enabled) = entry.enabled {
        parts.push(format!("enabled = {enabled}"));
    }
    if parts.is_empty() {
        text(cx, "Not set in the file", "文件中未设置").to_string()
    } else {
        parts.join(" · ")
    }
}

fn cell(label: &'static str, value: String, mono: bool, wide: bool, cx: &App) -> Div {
    let value = div().min_w_0().text_size(px(theme::BODY_SIZE)).child(value);
    let value = if mono {
        value.font_family(cx.theme().mono_font_family.clone())
    } else {
        value
    };
    div()
        .flex_grow(1.)
        .flex_basis(relative(if wide { 1. } else { 0.4 }))
        .min_w(px(140.))
        .flex()
        .flex_col()
        .gap(px(theme::SELECTION_RADIUS))
        .child(muted_line(label, cx))
        .child(value)
}

/// English copy for the built-in presets; the crate only ships Chinese text.
fn preset_description(cx: &App, preset: &McpPreset) -> String {
    let english = match preset.id.as_str() {
        "fetch" => Some("Fetch web pages and convert HTML to Markdown"),
        "context7" => Some("Search current library documentation for context"),
        "sequential-thinking" => Some("Step-by-step reasoning for complex problems"),
        "exa" => Some("AI search engine with semantic search and content extraction"),
        "serena" => Some("Semantic code analysis across many languages"),
        _ => None,
    };
    match english {
        Some(english) if !is_chinese(cx) => english.to_string(),
        _ => preset.description.clone(),
    }
}

/// 预设行：名称 + 描述与标签（单行省略，tooltip 补全）；需要密钥时显示环境变量名。
fn preset_row(preset: McpPreset, cx: &mut Context<McpView>) -> gpui_kit::base::Button {
    let description = preset_description(cx, &preset);
    let tags = preset
        .tags
        .iter()
        .filter(|tag| tag.is_ascii())
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let meta: SharedString = if tags.is_empty() {
        description.clone()
    } else {
        format!("{description} · {tags}")
    }
    .into();
    let tip = meta.clone();
    let mut label = format!("{}. {description}", preset.name);
    if let Some(key) = &preset.api_key_env {
        label.push_str(&format!(". {} {key}", text(cx, "Requires", "需要")));
    }
    let id = preset.id.clone();
    let colors = cx.theme();
    let row = gpui_kit::base::Button::new(SharedString::from(format!("mcp-preset-{id}")))
        .accessibility_label(label)
        .flex_shrink_0()
        .w_full()
        .h(px(theme::MCP_ROW_HEIGHT))
        .px(px(theme::CONTROL_GAP))
        .gap(px(theme::CONTROL_GAP + 4.))
        .justify_start()
        .rounded(px(theme::SELECTION_RADIUS))
        .border_2()
        .border_color(gpui_kit::transparent_black())
        .hover(|style| style.bg(colors.muted))
        .focus_visible(|style| style.border_color(colors.ring))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .truncate()
                        .text_size(px(theme::BODY_SIZE))
                        .line_height(px(20.))
                        .font_weight(FontWeight::MEDIUM)
                        .child(preset.name.clone()),
                )
                .child(
                    div()
                        .id(SharedString::from(format!("mcp-preset-{id}-meta")))
                        .truncate()
                        .text_size(px(theme::META_SIZE))
                        .line_height(px(16.))
                        .text_color(colors.muted_foreground)
                        .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
                        .child(meta),
                ),
        );
    let row = match &preset.api_key_env {
        Some(key) => row.child(badge::tag(
            SharedString::from(format!("mcp-preset-{id}-key")),
            key.clone(),
            Tone::Warning,
            cx,
        )),
        None => row,
    };
    row.on_click(cx.listener(move |this, _, window, cx| {
        this.open_sources(EditMode::Preset(id.clone()), window, cx)
    }))
}
