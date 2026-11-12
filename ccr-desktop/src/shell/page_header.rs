//! Shared lens header: a content-width segmented capsule, never a full-width bar.
//!
//! 三种挂载方式，都只有一个 h1：
//! - 嵌入式（运行概览 / 全局监控、MCP 管理 / 配置转换）：外壳拥有 [`LensBar`] 视图，
//!   页面把它放进自己的标题行，标题与胶囊同一行；选中状态在渲染时从外壳读取，
//!   点击回到外壳 `go`。
//! - 平台目的地：不渲染胶囊，页头按任务分组渲染文字 tab（工作台 │ 配置方案·账户·设置 │ …），
//!   唯一 h1 是当前 tab 名，平台名只作小号上下文；页内没有跨平台选择或第二排平台导航。
//! - 上下文行（其余 lens 目的地）：页面承载多个 lens 实体，外壳在页面上方渲染
//!   “可选平台名 + 胶囊”，页面自己的标题紧随其下。
use super::{Destination, Lens, Shell, semantics};
use crate::{locales::text, theme};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    AnyElement, App, Context, Div, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, WeakEntity, Window, div, px,
};

/// 分组文字 tab 之间的目标间距（逻辑像素）。
const GROUP_GAP: f32 = 16.;

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
    /// 待恢复焦点的激活结果：(目的地, lens, 选项序号, 导航代次)。
    refocus: Option<(Destination, Lens, usize, u64)>,
    scroll: gpui_kit::ScrollHandle,
    revealed: Option<(Destination, Lens)>,
    reveal_width: Option<gpui_kit::Pixels>,
}

impl LensBar {
    pub(super) fn new(shell: WeakEntity<Shell>, cx: &mut Context<Self>) -> Self {
        Self {
            shell,
            focus: cx.focus_handle().tab_stop(false),
            refocus: None,
            scroll: gpui_kit::ScrollHandle::new(),
            revealed: None,
            reveal_width: None,
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
        let keyboard = self.focus.contains_focused(window, cx);
        self.refocus = None;
        if let Some(shell) = self.shell.upgrade() {
            let generation = shell.update(cx, |shell, cx| {
                shell.go(destination, lens, window, cx);
                shell.navigation_generation
            });
            if keyboard {
                self.refocus = Some((destination, lens, index, generation));
            }
        }
        cx.notify();
    }

    /// 新父节点中的胶囊绘制后再聚焦；离页或再次切换后的延迟回调不抢焦点。
    fn restore_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((destination, lens, index, generation)) = self.refocus.take() else {
            return;
        };
        let previous_focus = window.focused(cx);
        let view = cx.weak_entity();
        window.on_next_frame(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                let current = this.shell.upgrade().map(|shell| {
                    let shell = shell.read(cx);
                    (
                        shell.destination,
                        shell.lens,
                        shell.navigation_generation,
                        shell.remote.read(cx).is_remote(),
                    )
                });
                if current != Some((destination, lens, generation, false))
                    || window
                        .focused(cx)
                        .is_some_and(|focus| Some(focus) != previous_focus)
                {
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
            return div().into_any_element();
        };
        self.restore_focus(window, cx);
        if destination.platform().is_some() {
            let active = items.iter().position(|item| item.1 == current).unwrap_or(0);
            let mut navigation = div()
                .id("platform-tab-scroll")
                .test_support()
                .min_w_0()
                .flex()
                .items_center()
                .gap(px(4.))
                .overflow_x_scroll()
                .track_scroll(&self.scroll)
                .track_focus(&self.focus)
                .on_key_down(cx.listener(
                    move |this, event: &gpui_kit::KeyDownEvent, window, cx| {
                        let offset = match event.keystroke.key.as_str() {
                            "left" => -1,
                            "right" => 1,
                            _ => return,
                        };
                        if let Some(shell) = this.shell.upgrade() {
                            let shell = shell.read(cx);
                            let items = shell.platform_items();
                            let index = items
                                .iter()
                                .position(|item| item.1 == shell.lens)
                                .unwrap_or(0);
                            let next =
                                (index as isize + offset).rem_euclid(items.len() as isize) as usize;
                            this.activate(destination, items[next].1, next, window, cx);
                            cx.stop_propagation();
                        }
                    },
                ));
            let group_key = |lens: Lens| match lens {
                Lens::Primary => 0,
                Lens::Profiles | Lens::Auth | Lens::Accounts | Lens::Providers | Lens::Models => 1,
                Lens::Usage | Lens::Sessions => 3,
                _ => 2,
            };
            let mut selected_child = 0;
            let mut group_index = 0;
            let mut start = 0;
            while start < items.len() {
                let key = group_key(items[start].1);
                let end = (start + 1..items.len())
                    .find(|index| group_key(items[*index].1) != key)
                    .unwrap_or(items.len());
                if start > 0 {
                    // 组间距按合同为 16px：两侧 4px 容器间距 + 1px 分隔线之外的余量对半分成外边距。
                    navigation = navigation.child(
                        div()
                            .flex_shrink_0()
                            .mx(px((GROUP_GAP - 9.) / 2.))
                            .w(px(1.))
                            .h(px(16.))
                            .bg(theme::hairline(cx.theme().mode)),
                    );
                }
                let mut group = div()
                    .id(format!("platform-tab-group-{key}"))
                    .role(gpui_kit::Role::Group)
                    .aria_label(match key {
                        0 => text(cx, "Workbench", "工作台"),
                        1 => text(cx, "Profiles, accounts and settings", "方案、账户与设置"),
                        2 => text(cx, "Tools and resources", "工具与资源"),
                        _ => text(cx, "Usage and sessions", "用量与会话"),
                    })
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap(px(4.));
                for (index, (id, lens, en, zh)) in
                    items.iter().copied().enumerate().take(end).skip(start)
                {
                    group = group.child(lens_tab(
                        id,
                        current == lens,
                        text(cx, en, zh),
                        cx,
                        move |this, window, cx| this.activate(destination, lens, index, window, cx),
                    ));
                }
                if (start..end).contains(&active) {
                    selected_child = group_index * 2;
                }
                navigation = navigation.child(group);
                group_index += 1;
                start = end;
            }
            let width = window.viewport_size().width;
            if self.revealed != Some((destination, current)) || self.reveal_width != Some(width) {
                self.revealed = Some((destination, current));
                self.reveal_width = Some(width);
                self.scroll.scroll_to_item(selected_child);
            }
            let item = items[active];
            let heading = div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .id("platform-tab-heading")
                        .test_support()
                        .aria_label(text(cx, item.2, item.3))
                        .role(gpui_kit::Role::Heading)
                        .aria_level(1)
                        .text_size(px(theme::TITLE_SIZE))
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .child(text(cx, item.2, item.3)),
                )
                .child(
                    div()
                        .text_size(px(theme::META_SIZE))
                        .text_color(cx.theme().muted_foreground)
                        .child(crate::features::workbench::name(
                            destination.platform().unwrap_or(super::Platform::Claude),
                        )),
                );
            let wide = window.viewport_size().width > px(1600.);
            let row = div()
                .id("platform-context-navigation")
                .test_support()
                .flex_shrink_0()
                .flex()
                .gap(px(16.))
                .px(px(theme::CONTENT_PADDING))
                .pt(px(theme::SECTION_GAP));
            let overflow = self.scroll.max_offset().x > px(0.);
            let navigation = div()
                .min_w_0()
                .flex()
                .items_center()
                .gap(px(4.))
                .child(navigation)
                .children(overflow.then(|| {
                    // 溢出提示要有稳定 ID 与可读名称，读屏与测试都不能只看到一个箭头字符。
                    div()
                        .id("platform-tab-overflow")
                        .test_support()
                        .role(gpui_kit::Role::Label)
                        .aria_label(text(
                            cx,
                            "Tabs continue off-screen; scroll the tab row",
                            "还有 tab 在可视区外，可横向滚动",
                        ))
                        .flex_shrink_0()
                        .text_color(cx.theme().muted_foreground)
                        .child("↔")
                }));
            return if wide {
                row.items_center()
                    .child(heading)
                    .child(navigation)
                    .into_any_element()
            } else {
                row.flex_col()
                    .child(heading)
                    .child(navigation)
                    .into_any_element()
            };
        }
        let mut bar = capsule(cx).self_center().track_focus(&self.focus);
        for (index, (id, lens, en, zh)) in items.into_iter().enumerate() {
            bar = bar.child(lens_tab(id, current == lens, text(cx, en, zh), cx, {
                move |this: &mut Self, window: &mut Window, cx: &mut Context<Self>| {
                    this.activate(destination, lens, index, window, cx);
                }
            }));
        }
        bar.into_any_element()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{ConfigSource, Services};
    use ccr_app::runtime::RuntimeOwner;
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{AppContext, TestAppContext, WindowOptions};
    use std::time::Duration;

    #[gpui_kit::test]
    fn delayed_tab_restore_preserves_a_new_content_focus(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let (handle, shell) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    Shell::new(
                        Services::new(
                            owner.handle(),
                            ConfigSource::Fixture {
                                profiles: root.path().join("missing.toml"),
                                registry: root.path().join("registry.toml"),
                                delay: Duration::ZERO,
                            },
                        ),
                        window,
                        cx,
                    )
                })
            })
            .unwrap_or_else(|e| panic!("window: {e}"))
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("nav-claude-code", cx);
            window.click("tool-profiles", cx);
            let bar = shell.read(cx).lens_bar.clone();
            let generation = shell.read(cx).navigation_generation;
            // 复现回调已排队、用户随后进入内容区、下一帧才回调的顺序。
            bar.update(cx, |bar, cx| {
                window.focus(&bar.focus, cx);
                window.focus_next(cx);
                bar.refocus = Some((Destination::ClaudeCode, Lens::Profiles, 1, generation));
                bar.restore_focus(window, cx);
            });
            shell.update(cx, |shell, cx| shell.focus_main(window, cx));
            let content_focus = window.focused(cx);
            assert!(content_focus.is_some());
            window.simulate_next_frame(cx);
            assert_eq!(window.focused(cx), content_focus);
            window.render_frame(cx);
            assert_eq!(window.find("configuration-filter").focused(), Some(true));
            bar.update(cx, |bar, cx| {
                window.focus(&bar.focus, cx);
                bar.refocus = Some((Destination::ClaudeCode, Lens::Profiles, 1, generation));
                bar.restore_focus(window, cx);
            });
            // 离页再回到同一 tab，也不能让旧一代回调通过目标相等的检查。
            shell.update(cx, |shell, cx| {
                shell.go(Destination::Codex, Lens::Primary, window, cx);
                shell.go(Destination::ClaudeCode, Lens::Profiles, window, cx);
            });
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert!(bar.read(cx).focus.is_focused(window));
            // 在恢复挂载前完成离页和返回，旧请求也不能绑定到新的导航代次。
            let generation = shell.read(cx).navigation_generation;
            bar.update(cx, |bar, _| {
                bar.refocus = Some((Destination::ClaudeCode, Lens::Profiles, 1, generation));
            });
            shell.update(cx, |shell, cx| {
                shell.go(Destination::Codex, Lens::Primary, window, cx);
                shell.go(Destination::ClaudeCode, Lens::Profiles, window, cx);
            });
            bar.update(cx, |bar, cx| bar.restore_focus(window, cx));
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert!(bar.read(cx).focus.is_focused(window));
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("late tab focus: {e}"));
        owner.shutdown();
    }
}
