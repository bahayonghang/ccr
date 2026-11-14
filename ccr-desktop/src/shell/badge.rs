//! Shared status tags, filter chips, dots, search field and empty-state panel.
//!
//! 所有页面共用这一套实现，颜色只来自 `theme`，不用 Kit 的 Tag/Badge（它们读 Kit
//! 主题色，圆角和字号也不在本项目的 token 体系内）。可选子树由调用方先构造好再
//! 传入，避免增大 render 栈帧（Windows UI 线程只有 1 MiB）。
use super::semantics;
use crate::theme::{self, Tone};
use gpui_kit::component::{
    ActiveTheme, Icon, IconName, Selectable, Sizable,
    button::{Button, ButtonVariants},
    input::Input,
};
use gpui_kit::{
    AnyElement, App, Div, ElementId, FontWeight, Hsla, InteractiveElement, ParentElement, Role,
    SharedString, Stateful, StatefulInteractiveElement, Styled, div, px, rgb, transparent_black,
};

/// A non-interactive status tag: a 22px, 4px-radius rectangle with 12/500 text.
pub fn tag(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    tone: Tone,
    cx: &App,
) -> Stateful<Div> {
    let label = label.into();
    tag_frame(id, label.clone(), tone, cx).child(label)
}

/// A status tag whose label follows a 12px glyph in the same tone, such as a
/// ban sign before "Disabled". The text still carries the state on its own.
pub fn icon_tag(
    id: impl Into<ElementId>,
    icon: IconName,
    label: impl Into<SharedString>,
    tone: Tone,
    cx: &App,
) -> Stateful<Div> {
    let label = label.into();
    let ink = theme::tone_colors(tone, cx.theme().mode).ink;
    tag_frame(id, label.clone(), tone, cx)
        .gap(px(theme::SELECTION_RADIUS))
        .child(Icon::new(icon).size(px(theme::META_SIZE)).text_color(ink))
        .child(label)
}

// 标签外框：语调色、尺寸与无障碍标签；内容由调用方追加。
fn tag_frame(id: impl Into<ElementId>, label: SharedString, tone: Tone, cx: &App) -> Stateful<Div> {
    let colors = theme::tone_colors(tone, cx.theme().mode);
    div()
        .id(id)
        .role(Role::Label)
        .aria_label(label)
        .flex_shrink_0()
        .h(crate::theme::text_size(theme::TAG_HEIGHT))
        .flex()
        .items_center()
        .px(px(theme::CONTROL_GAP))
        .rounded(px(theme::SELECTION_RADIUS))
        .border_1()
        .border_color(colors.edge)
        .bg(colors.fill)
        .text_color(colors.ink)
        .text_size(crate::theme::text_size(theme::META_SIZE))
        .font_weight(FontWeight::MEDIUM)
        .whitespace_nowrap()
}

/// A solid 8px dot. Always sits beside text that names what it marks.
pub fn dot(color: Hsla) -> Div {
    div()
        .flex_shrink_0()
        .size(px(theme::DOT_SIZE))
        .rounded_full()
        .bg(color)
}

/// A hollow 8px dot for states that were not found or not read.
pub fn hollow_dot(color: Hsla) -> Div {
    div()
        .flex_shrink_0()
        .size(px(theme::DOT_SIZE))
        .rounded_full()
        .border_1()
        .border_color(color)
}

/// A platform identity: its 8px dot followed by the platform name.
pub fn identity(platform_id: &str, label: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .flex()
        .flex_shrink_0()
        .items_center()
        .gap(px(theme::CONTROL_GAP))
        .whitespace_nowrap()
        .child(dot(theme::platform_color(platform_id, cx.theme().mode)))
        .child(label.into())
}

/// Decorates a toggleable filter chip: a 28px pill reporting its pressed state.
///
/// 未选时透明底加 hairline 边；选中时是 tab-selected 浮起面加主色文字。禁用态保留
/// muted 色，选中样式不得覆盖禁用色，否则忙碌或不可选的 chip 看起来仍可操作。
pub fn filter_chip(button: Button, pressed: bool, disabled: bool, cx: &App) -> Button {
    let colors = cx.theme();
    let tabs = theme::tab_colors(colors.mode);
    let raised = pressed && !disabled;
    let button = semantics::button(button.ghost().xsmall(), disabled, false, cx)
        .selected(pressed)
        .toggled(pressed)
        .h(crate::theme::text_size(theme::FILTER_CHIP_HEIGHT))
        .px(px(theme::CONTROL_GAP + 4.))
        .border_1()
        .bg(if raised {
            tabs.selected
        } else {
            transparent_black()
        })
        .text_color(if disabled {
            colors.muted_foreground
        } else if pressed {
            tabs.selected_foreground
        } else {
            tabs.foreground
        })
        .border_color(if raised {
            transparent_black()
        } else {
            colors.border
        })
        .font_weight(if pressed {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        });
    if raised {
        button.shadow(vec![tabs.selected_shadow])
    } else {
        button
    }
}

/// Decorates a search input with its prefix icon and fixed width. The caller
/// supplies the id, accessibility label and placeholder.
pub fn search_field(input: Input, cx: &App) -> Input {
    input
        .prefix(
            Icon::new(IconName::Search)
                .size(px(theme::NAV_ICON_SIZE))
                .text_color(cx.theme().muted_foreground),
        )
        .flex_shrink_0()
        .w(px(theme::SEARCH_WIDTH))
        .max_w_full()
        .rounded(px(theme::INPUT_RADIUS))
}

/// A dashed panel stating why there is nothing to show and what to do next.
///
/// `action` is a real control built by the caller; the panel never invents one.
pub fn empty_panel(
    id: impl Into<ElementId>,
    icon: IconName,
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
    action: Option<AnyElement>,
    cx: &App,
) -> Stateful<Div> {
    let colors = cx.theme();
    let (title, body) = (title.into(), body.into());
    let panel = div()
        .id(id)
        .role(Role::Group)
        .aria_label(title.clone())
        .aria_description(body.clone())
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(theme::CONTROL_GAP))
        .p(px(theme::CONTENT_PADDING))
        .rounded(px(theme::INPUT_RADIUS))
        .border_1()
        .border_dashed()
        .border_color(rgb(theme::palette(colors.mode).subtle))
        .text_center()
        .child(
            Icon::new(icon)
                .size(px(theme::EMPTY_ICON_SIZE))
                .text_color(colors.muted_foreground),
        )
        .child(
            div()
                .text_size(crate::theme::text_size(theme::BODY_SIZE))
                .font_weight(FontWeight::MEDIUM)
                .child(title),
        )
        .child(
            div()
                .text_size(crate::theme::text_size(theme::META_SIZE))
                .text_color(colors.muted_foreground)
                .child(body),
        );
    match action {
        Some(action) => panel.child(action),
        None => panel,
    }
}

/// A 16/600 section title with an optional 12px muted scope, such as "All history".
pub fn section_heading(
    title: impl Into<SharedString>,
    scope: Option<SharedString>,
    cx: &App,
) -> Div {
    let heading = div()
        .flex()
        .flex_wrap()
        .items_baseline()
        .gap(px(theme::CONTROL_GAP))
        .child(
            div()
                .text_size(crate::theme::text_size(theme::SECTION_TITLE_SIZE))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.into()),
        );
    match scope {
        Some(scope) => heading.child(
            div()
                .text_size(crate::theme::text_size(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(scope),
        ),
        None => heading,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use gpui_kit::base::TestSupportExt;
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{
        AnyWindowHandle, AppContext, ClickEvent, Context, Entity, FocusHandle, IntoElement, Render,
        TestAppContext, Window, WindowOptions,
    };
    use std::{cell::Cell, rc::Rc};

    /// 每个控件被真实激活的次数；禁用控件必须保持 0。
    #[derive(Clone, Default)]
    struct Clicks {
        off: Rc<Cell<usize>>,
        locked: Rc<Cell<usize>>,
        action: Rc<Cell<usize>>,
    }

    struct Harness {
        clicks: Clicks,
        focus: FocusHandle,
    }

    fn count(cell: &Rc<Cell<usize>>) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        let cell = cell.clone();
        move |_, _, _| cell.set(cell.get() + 1)
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            let chip = |id: &'static str, label: &'static str| Button::new(id).label(label);
            div()
                .track_focus(&self.focus)
                .size_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(theme::CONTROL_GAP))
                .child(tag("tag-partial", "Partial", Tone::Warning, cx).test_support())
                .child(
                    icon_tag("tag-disabled", IconName::Ban, "Disabled", Tone::Warning, cx)
                        .test_support(),
                )
                .child(filter_chip(chip("chip-on", "Claude"), true, false, cx))
                .child(filter_chip(
                    chip("chip-off", "Codex").on_click(count(&clicks.off)),
                    false,
                    false,
                    cx,
                ))
                .child(filter_chip(
                    chip("chip-locked", "Grok").on_click(count(&clicks.locked)),
                    false,
                    true,
                    cx,
                ))
                .child(
                    empty_panel(
                        "panel-empty",
                        IconName::Inbox,
                        "No sessions",
                        "Refresh the index to read sessions.",
                        Some(
                            Button::new("panel-action")
                                .label("Refresh index")
                                .on_click(count(&clicks.action))
                                .into_any_element(),
                        ),
                        cx,
                    )
                    .test_support(),
                )
        }
    }

    fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Harness>, Clicks) {
        let clicks = Clicks::default();
        let shared = clicks.clone();
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|cx| Harness {
                    clicks: shared,
                    focus: cx.focus_handle(),
                })
            })
            .unwrap()
        });
        (handle, view, clicks)
    }

    #[gpui_kit::test]
    fn tag_renders_its_label_at_the_token_height(cx: &mut TestAppContext) {
        let (handle, _, _) = open(cx);
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let tag = window.find("tag-partial");
            assert!(tag.visible());
            assert_eq!(tag.role(), Some(Role::Label));
            assert_eq!(tag.label(), Some("Partial"));
            assert_eq!(tag.bounds().size.height, px(theme::TAG_HEIGHT));
            // 带图标的标签保持同一高度与标签语义，图标不进入无障碍名称。
            let icon = window.find("tag-disabled");
            assert_eq!(icon.role(), Some(Role::Label));
            assert_eq!(icon.label(), Some("Disabled"));
            assert_eq!(icon.bounds().size.height, px(theme::TAG_HEIGHT));
            assert!(icon.bounds().size.width > tag.bounds().size.width);
            window.remove_window();
        })
        .unwrap();
    }

    #[gpui_kit::test]
    fn filter_chip_reports_toggled_state_and_a_disabled_chip_stays_inert(cx: &mut TestAppContext) {
        let (handle, _, clicks) = open(cx);
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert_eq!(window.find("chip-on").checked(), Some(true));
            assert_eq!(window.find("chip-off").checked(), Some(false));
            assert_eq!(window.find("chip-locked").checked(), Some(false));
            assert_eq!(
                window.find("chip-off").bounds().size.height,
                px(theme::FILTER_CHIP_HEIGHT)
            );
            // Kit 的快照读不到 disabled 标记，用真实行为验证：启用的 chip 有焦点绑定，
            // 禁用的没有，激活也不会触发。
            assert_eq!(window.find("chip-off").focused(), Some(false));
            assert_eq!(window.find("chip-locked").focused(), None);
            window.click("chip-off", cx);
            window.click("chip-locked", cx);
            window.remove_window();
        })
        .unwrap();
        assert_eq!(clicks.off.get(), 1);
        assert_eq!(clicks.locked.get(), 0);
    }

    #[gpui_kit::test]
    fn empty_panel_keeps_its_action_reachable_by_keyboard_and_pointer(cx: &mut TestAppContext) {
        let (handle, view, clicks) = open(cx);
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert_eq!(window.find("panel-empty").label(), Some("No sessions"));
            assert_eq!(window.find("panel-action").focused(), Some(false));
            let focus = view.read(cx).focus.clone();
            window.focus(&focus, cx);
            // 从视图根开始按 Tab：跳过禁用 chip，最终落到面板内的真实操作上。
            let mut reached = false;
            for _ in 0..4 {
                window.press("tab", cx);
                if window.find("panel-action").focused() == Some(true) {
                    reached = true;
                    break;
                }
            }
            assert!(reached, "Tab reaches the empty panel action");
            window.click("panel-action", cx);
            window.remove_window();
        })
        .unwrap();
        assert_eq!(clicks.action.get(), 1);
    }
}
