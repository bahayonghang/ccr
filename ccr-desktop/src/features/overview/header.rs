//! Heading row: title with the embedded lens, range capsule, refresh and status line.
use super::{Node, OverviewView};
use crate::{
    locales::{is_chinese, service_error, text},
    runtime::insights::WorkbenchReport,
    shell::{badge, page_header, semantics},
    theme::{self, Tone},
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, IconName,
    button::{Button, ButtonVariants},
    tooltip::Tooltip,
};
use gpui_kit::{
    App, Context, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, div, prelude::FluentBuilder, px,
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

    /// `Last successful sync MM-DD HH:mm`, or why there is no time to show.
    ///
    /// llmusage 只记录“成功”的同步时间；失败的同步仍可能导入了部分数据，所以这里
    /// 只陈述成功时间，不据此推断哪些日期“未导入”。
    pub(super) fn sync_line(&self, report: &WorkbenchReport, cx: &App) -> String {
        match (&report.freshness.local, &report.last_sync_at) {
            (Some(local), _) if is_chinese(cx) => format!("上次成功同步 {local}{}", self.zone()),
            (Some(local), _) => format!("Last successful sync {local}{}", self.zone()),
            (None, Some(_)) => text(cx, "Sync time unreadable", "同步时间不可读").into(),
            (None, None) => text(cx, "No successful sync recorded", "未记录成功同步").into(),
        }
    }

    // 过期时圆点旁必须有文字说明它标记的是什么；无障碍标签同样写明。
    fn last_sync(&self, cx: &App) -> Option<Node> {
        let report = self.report.as_ref()?;
        let line = self.sync_line(report, cx);
        let stale = report.freshness.stale;
        let description = if stale && report.freshness.local.is_some() {
            format!(
                "{line}{}",
                text(
                    cx,
                    ". Newer usage may be missing until a sync succeeds",
                    "。同步成功前，较新的用量可能缺失"
                )
            )
        } else {
            line.clone()
        };
        let row = div()
            .id("overview-last-sync")
            .test_support()
            .role(gpui_kit::Role::Label)
            .aria_label(description.clone())
            .flex()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .text_size(px(theme::META_SIZE))
            .text_color(cx.theme().muted_foreground)
            .child("·");
        let row = if stale {
            row.child(
                badge::dot(theme::tone_colors(Tone::Warning, cx.theme().mode).ink)
                    .id("overview-last-sync-stale")
                    .tooltip(move |window, cx| Tooltip::new(description.clone()).build(window, cx))
                    .test_support(),
            )
        } else {
            row
        };
        Some(row.child(line))
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
                            .child(semantics::ghost_button(
                                Button::new("overview-refresh")
                                    .icon(IconName::RefreshCw)
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
                    .children(self.last_sync(cx))
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
