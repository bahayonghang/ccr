//! Heading row with the embedded lens and page actions, then the one-row toolbar.
use super::{EditMode, Family, McpView, Node};
use crate::{
    locales::text,
    shell::{badge, page_header, semantics},
    theme,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, IconName,
    button::{Button, ButtonVariants},
    input::Input,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    Context, Div, InteractiveElement, ParentElement, SharedString, StatefulInteractiveElement,
    Styled, div, px,
};

impl McpView {
    /// h1 + lens on the left, Reload and Add server on the right, one muted line below.
    pub(super) fn header(&self, cx: &mut Context<Self>) -> Div {
        let mut title = div()
            .min_w_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::SECTION_GAP))
            .when(self.platform_scope.is_none(), |title| {
                title.child(
                    div()
                        .id("mcp-heading")
                        .test_support()
                        .role(gpui_kit::Role::Heading)
                        .aria_level(1)
                        .text_size(crate::theme::text_size(theme::TITLE_SIZE))
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .child(text(cx, "MCP Manager", "MCP 管理")),
                )
            });
        // Claude 工作台复用本视图时不带胶囊，平台 tab 仍由外壳工具栏承载。
        if self.platform_scope.is_none()
            && let Some(lens) = &self.lens
        {
            title = title.child(lens.clone());
        }
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .id("mcp-header")
                    .test_support()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(theme::CONTROL_GAP))
                    .child(title)
                    .child(self.header_actions(cx)),
            )
            .child(
                div()
                    .text_size(crate::theme::text_size(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(text(
                        cx,
                        "MCP servers configured in local files, listed by source and scope.",
                        "本地文件中配置的 MCP 服务器，按来源与作用域列出。",
                    )),
            )
    }

    fn header_actions(&self, cx: &mut Context<Self>) -> Div {
        let locked = self.busy || self.editor.is_some() || self.batch.is_some();
        div()
            .flex()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(semantics::ghost_button(
                Button::new("mcp-read")
                    .icon(IconName::RefreshCw)
                    .label(if self.busy {
                        text(cx, "Reading", "读取中")
                    } else {
                        text(cx, "Reload", "重新读取")
                    })
                    .tooltip(text(
                        cx,
                        "Read the local MCP files again",
                        "重新读取本地 MCP 文件",
                    ))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                locked,
                self.busy,
                cx,
            ))
            .child(semantics::button(
                Button::new("mcp-add")
                    .icon(IconName::Plus)
                    .label(text(cx, "Add server", "添加服务器"))
                    .primary()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_sources(EditMode::Add, window, cx)
                    })),
                locked,
                false,
                cx,
            ))
    }

    /// Search and platform filter on the left, presets and batch on the right.
    pub(super) fn toolbar(&self, cx: &mut Context<Self>) -> Node {
        let filters = div()
            .min_w_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(badge::search_field(
                Input::new(&self.filter).id("mcp-filter").aria_label(text(
                    cx,
                    "Filter MCP entries by name or command",
                    "按名称或命令筛选 MCP 条目",
                )),
                cx,
            ));
        let filters = match self.family_capsule(cx) {
            Some(capsule) => filters.child(capsule),
            None => filters,
        };
        div()
            .id("mcp-toolbar")
            .test_support()
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(px(theme::CONTROL_GAP))
            .child(filters)
            .child(self.toolbar_actions(cx))
    }

    fn toolbar_actions(&self, cx: &mut Context<Self>) -> Div {
        let locked = self.busy || self.editor.is_some() || self.batch.is_some();
        div()
            .flex()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(semantics::ghost_button(
                Button::new("mcp-presets")
                    .icon(IconName::Star)
                    .label(text(cx, "Presets", "预设"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| this.open_presets(window, cx))),
                locked,
                false,
                cx,
            ))
            .when(self.platform_scope.is_none(), |row| {
                row.child(semantics::ghost_button(
                    Button::new("mcp-batch-open")
                        .icon(IconName::GalleryVerticalEnd)
                        .label(text(cx, "Batch & sync", "批量与同步"))
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| this.open_batch(window, cx))),
                    locked,
                    false,
                    cx,
                ))
            })
    }

    /// `All n | Claude Code n | …`; counts come from the catalog before search.
    fn family_capsule(&self, cx: &mut Context<Self>) -> Option<Div> {
        if self.platform_scope.is_some() {
            return None;
        }
        let counts = self.family_counts();
        if counts.is_empty() {
            return None;
        }
        let total: usize = counts.iter().map(|(_, count)| count).sum();
        let active = self.active_family();
        let all = format!("{} {total}", text(cx, "All", "全部"));
        let mut capsule = page_header::capsule(cx).self_center().child(family_tab(
            "mcp-platform-all".into(),
            all,
            active.is_none(),
            None,
            cx,
        ));
        for (family, count) in counts {
            capsule = capsule.child(family_tab(
                format!("mcp-platform-{}", family.id()).into(),
                format!("{} {count}", family.name()),
                active == Some(family),
                Some(family),
                cx,
            ));
        }
        Some(capsule)
    }
}

// 选项构建放在独立函数中，避免增大 render 栈帧（Windows UI 线程只有 1 MiB）。
fn family_tab(
    id: SharedString,
    label: String,
    selected: bool,
    family: Option<Family>,
    cx: &mut Context<McpView>,
) -> Button {
    semantics::tab(
        semantics::button(
            Button::new(id)
                .label(label)
                .ghost()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.family = family;
                    cx.notify();
                })),
            false,
            false,
            cx,
        ),
        selected,
        false,
        cx,
    )
}
