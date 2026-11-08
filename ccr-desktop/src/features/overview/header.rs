//! Heading row: title with the embedded lens, range capsule, refresh and status line.
use super::OverviewView;
use crate::{
    locales::{is_chinese, service_error, text},
    shell::{badge, page_header, semantics},
    theme,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    div, prelude::FluentBuilder, px,
};

impl OverviewView {
    fn status_text(&self, cx: &mut Context<Self>) -> String {
        let state = if self.loading {
            Some(text(cx, "Reading local usage…", "正在读取本机用量…"))
        } else {
            self.error.map(|error| service_error(cx, error))
        };
        let Some(report) = self.report.as_ref() else {
            return state
                .unwrap_or_else(|| text(cx, "Usage has not been read yet.", "尚未读取用量。"))
                .into();
        };
        let zone = self.zone();
        let window = if is_chinese(cx) {
            format!(
                "本机用量 · 截至 {}{zone} 的 {} 日",
                report.as_of, report.window_days
            )
        } else {
            format!(
                "Local usage · {} days ending {}{zone}",
                report.window_days, report.as_of
            )
        };
        match state {
            Some(message) => format!(
                "{message} · {} {window}",
                text(cx, "Last report:", "上次报告：")
            ),
            None => window,
        }
    }

    pub(super) fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.status_text(cx);
        let ink = if self.error.is_some() {
            cx.theme().warning
        } else {
            cx.theme().muted_foreground
        };
        let mut title = div()
            .min_w_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::SECTION_GAP))
            .child(
                div()
                    .id("overview-heading")
                    .test_support()
                    .role(gpui_kit::Role::Heading)
                    .aria_level(1)
                    .text_size(px(theme::TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "Overview", "运行概览")),
            );
        if let Some(lens) = &self.lens {
            title = title.child(lens.clone());
        }
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(theme::CONTROL_GAP))
                    .child(title)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(theme::CONTROL_GAP))
                            .child(self.windows(cx))
                            .child(semantics::button(
                                Button::new("overview-refresh")
                                    .label(text(cx, "Refresh", "刷新用量"))
                                    .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                                self.loading || !self.requests.configured(),
                                self.loading,
                                cx,
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(badge::dot(if self.error.is_some() {
                        ink
                    } else if self.report.is_some() && !self.loading {
                        cx.theme().primary
                    } else {
                        cx.theme().input
                    }))
                    .child(
                        div()
                            .id("overview-status")
                            .test_support()
                            .role(gpui_kit::Role::Status)
                            .aria_label(status.clone())
                            .text_size(px(theme::META_SIZE))
                            .text_color(ink)
                            .child(status),
                    )
                    .when(self.fixture, |row| {
                        row.child(
                            div()
                                .text_size(px(theme::META_SIZE))
                                .text_color(cx.theme().warning)
                                .child(text(cx, "Isolated demo", "隔离演示")),
                        )
                    }),
            )
    }

    /// Page-level 7D/30D/90D range capsule; it applies to the whole page.
    fn windows(&self, cx: &mut Context<Self>) -> gpui_kit::Div {
        let disabled = !self.requests.configured();
        let mut group = page_header::capsule(cx).self_center();
        for (id, days, label) in [
            ("overview-window-7", 7_u16, "7D"),
            ("overview-window-30", 30, "30D"),
            ("overview-window-90", 90, "90D"),
        ] {
            group = group.child(semantics::tab(
                semantics::button(
                    Button::new(id)
                        .label(label)
                        .ghost()
                        .on_click(cx.listener(move |this, _, _, cx| this.set_days(days, cx))),
                    disabled,
                    false,
                    cx,
                ),
                self.days == days,
                disabled,
                cx,
            ));
        }
        group
    }
}
