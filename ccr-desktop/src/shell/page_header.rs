//! Shared lens header: a content-width segmented capsule, never a full-width bar.
//!
//! 两种挂载方式，都只有一个 h1：
//! - 嵌入式（运行概览 / 全局监控、MCP 管理 / 配置转换）：外壳拥有 [`LensBar`] 视图，
//!   页面把它放进自己的标题行，标题与胶囊同一行；选中状态在渲染时从外壳读取，
//!   点击回到外壳 `go`。
//! - 上下文行（其余 lens 目的地）：页面承载多个 lens 实体或多达十项的平台 tab，
//!   外壳在页面上方渲染“可选平台名 + 胶囊”，页面自己的标题紧随其下。
use super::{Destination, Lens, Shell, semantics};
use crate::{locales::text, theme};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    AnyElement, App, Context, Div, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, Styled, WeakEntity, Window, div, px,
};

/// 内容宽度的分段胶囊容器；选项由 `semantics::tab` 装饰。
pub(crate) fn capsule(cx: &App) -> Div {
    let tabs = theme::tab_colors(cx.theme().mode);
    div()
        .self_start()
        .min_w_0()
        .max_w_full()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(px(2.))
        .p(px(theme::CAPSULE_PADDING))
        .rounded(px(theme::CAPSULE_RADIUS))
        .border_1()
        .border_color(tabs.rim)
        .bg(tabs.background)
}

/// 页头行：可选上下文标签 + 胶囊。
pub(crate) fn row(
    id: &'static str,
    context: Option<SharedString>,
    capsule: Div,
    cx: &App,
) -> AnyElement {
    let mut row = div()
        .id(id)
        .test_support()
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(theme::CONTROL_GAP + 4.))
        .px(px(theme::CONTENT_PADDING))
        .pt(px(theme::SECTION_GAP));
    if let Some(context) = context {
        row = row.child(
            div()
                .flex_shrink_0()
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(context),
        );
    }
    row.child(capsule).into_any_element()
}

/// Shell-owned lens capsule that an embedding page renders inside its own
/// heading row, so the title and the capsule share one line.
pub(crate) struct LensBar {
    shell: WeakEntity<Shell>,
    /// 胶囊容器的稳定焦点锚点：页面切换会换掉胶囊的父节点，Kit 按钮按元素路径
    /// 保存的焦点状态随之丢失，因此用此锚点把焦点恢复到刚激活的选项。
    focus: FocusHandle,
    /// 待恢复焦点的激活结果：(目的地, lens, 选项序号)。
    refocus: Option<(Destination, Lens, usize)>,
}

impl LensBar {
    pub(super) fn new(shell: WeakEntity<Shell>, cx: &mut Context<Self>) -> Self {
        Self {
            shell,
            focus: cx.focus_handle().tab_stop(false),
            refocus: None,
        }
    }

    fn activate(
        &mut self,
        destination: Destination,
        lens: Lens,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 只在焦点原本就在胶囊内（键盘激活）时恢复；指针点击不聚焦按钮，也不移动焦点。
        if self.focus.contains_focused(window, cx) {
            self.refocus = Some((destination, lens, index));
        }
        if let Some(shell) = self.shell.upgrade() {
            shell.update(cx, |shell, cx| shell.go(destination, lens, window, cx));
        }
        cx.notify();
    }

    /// 新父节点中的胶囊绘制后再聚焦；离页或再次切换后的延迟回调不抢焦点。
    fn restore_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((destination, lens, index)) = self.refocus.take() else {
            return;
        };
        let view = cx.weak_entity();
        window.on_next_frame(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                let current = this
                    .shell
                    .upgrade()
                    .map(|shell| (shell.read(cx).destination, shell.read(cx).lens));
                if current != Some((destination, lens)) {
                    return;
                }
                window.focus(&this.focus, cx);
                for _ in 0..=index {
                    window.focus_next(cx);
                }
            });
        });
    }
}

impl Render for LensBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 子视图在外壳 render 返回后才渲染，此时读取外壳不会重入借用。
        let state = self.shell.upgrade().map(|shell| {
            let shell = shell.read(cx);
            (shell.destination, shell.lens, shell.lens_items(cx))
        });
        // 没有 lens 选项（如 SSH 环境）时不画空胶囊。
        let Some((destination, current, Some(items))) = state else {
            return div();
        };
        self.restore_focus(window, cx);
        let mut bar = capsule(cx).self_center().track_focus(&self.focus);
        for (index, (id, lens, en, zh)) in items.into_iter().enumerate() {
            bar = bar.child(lens_tab(id, current == lens, text(cx, en, zh), cx, {
                move |this: &mut Self, window: &mut Window, cx: &mut Context<Self>| {
                    this.activate(destination, lens, index, window, cx);
                }
            }));
        }
        bar
    }
}

// 选项构建放在独立函数中，避免增大 render 栈帧（Windows UI 线程只有 1 MiB）。
fn lens_tab(
    id: &'static str,
    selected: bool,
    label: &'static str,
    cx: &mut Context<LensBar>,
    open: impl Fn(&mut LensBar, &mut Window, &mut Context<LensBar>) + 'static,
) -> Button {
    semantics::tab(
        semantics::button(
            Button::new(id)
                .label(label)
                .ghost()
                .on_click(cx.listener(move |this, _, window, cx| open(this, window, cx))),
            false,
            false,
            cx,
        ),
        selected,
        false,
        cx,
    )
}
