//! Right rail: next actions and the data source summary.
use super::{Node, OverviewLink, OverviewView, chart::day_total, heading};
use crate::{
    locales::{is_chinese, text},
    runtime::insights::{SessionScope, WorkbenchReport},
    shell::badge,
    theme,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{ActiveTheme, Icon, IconName};
use gpui_kit::{
    App, Context, InteractiveElement, IntoElement, ParentElement, Styled, div,
    prelude::FluentBuilder, px,
};

impl OverviewView {
    pub(super) fn rail(
        &self,
        report: Option<&WorkbenchReport>,
        stacked: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut rail = div()
            .id("overview-rail")
            .test_support()
            .when(stacked, |el| el.w_full())
            .when(!stacked, |el| {
                el.w(px(theme::OVERVIEW_RAIL_WIDTH)).flex_shrink_0()
            })
            .flex()
            .flex_col()
            .p(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(theme::PANEL_RADIUS))
            .child(heading(text(cx, "Next actions", "下一步")));
        for (id, label, link, icon) in [
            (
                "overview-open-usage",
                text(cx, "View usage report", "查看用量报告"),
                OverviewLink::Usage,
                IconName::ChartPie,
            ),
            (
                "overview-open-commands",
                text(cx, "Run a command", "运行命令"),
                OverviewLink::Commands,
                IconName::Play,
            ),
            (
                "overview-open-sessions",
                text(cx, "Browse sessions", "浏览会话"),
                OverviewLink::Sessions,
                IconName::BookOpen,
            ),
            (
                "overview-open-install",
                text(cx, "Install llmusage", "安装 llmusage"),
                OverviewLink::Install,
                IconName::SquareTerminal,
            ),
        ] {
            rail = rail.child(
                action_row(id, label, icon, cx)
                    .on_click(cx.listener(move |this, _, _, cx| this.open(link, cx))),
            );
        }
        rail.child(self.data_source(report, cx))
    }

    /// Data source summary from state already in this view; no extra query.
    fn data_source(&self, report: Option<&WorkbenchReport>, cx: &App) -> Node {
        let configured = self.requests.configured();
        let muted = cx.theme().muted_foreground;
        let mut section = div()
            .id("overview-insights")
            .test_support()
            .mt(px(theme::SECTION_GAP))
            .pt(px(theme::SECTION_GAP))
            .border_t_1()
            .border_color(cx.theme().border)
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .text_size(px(theme::META_SIZE))
            .child(
                div()
                    .text_size(px(theme::BODY_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "Data source", "数据源")),
            )
            .child(
                div()
                    .id("overview-source-state")
                    .test_support()
                    .flex()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(badge::dot(if configured {
                        cx.theme().primary
                    } else {
                        cx.theme().input
                    }))
                    .child(if configured {
                        text(cx, "Local usage source configured", "已配置本机用量来源")
                    } else {
                        text(cx, "No local usage source configured", "未配置本机用量来源")
                    }),
            );
        if self.fixture {
            section = section.child(div().text_color(cx.theme().warning).child(text(
                cx,
                "Isolated demo",
                "隔离演示",
            )));
        }
        let Some(report) = report else {
            return section.child(div().text_color(muted).child(text(
                cx,
                "Read local usage to inspect source and archive coverage.",
                "读取本机用量后，可查看来源与归档覆盖范围。",
            )));
        };
        let zone = self.zone();
        section = section.child(div().text_color(muted).child(if is_chinese(cx) {
            format!("报告截至 {}{zone}", report.as_of)
        } else {
            format!("Report as of {}{zone}", report.as_of)
        }));
        let active = report.days.iter().filter(|day| day_total(day) > 0).count();
        let coverage = if report.days.is_empty() {
            text(cx, "Daily breakdown unavailable", "逐日明细不可用").to_string()
        } else if is_chinese(cx) {
            format!(
                "图表的四个来源在 {} 个日历日中有 {active} 个活跃日；其他来源不计入活跃日。",
                report.days.len()
            )
        } else {
            format!(
                "The four chart sources were active on {active} of {} calendar days; other sources are excluded from active days.",
                report.days.len()
            )
        };
        section = section.child(div().text_color(muted).child(coverage));
        if matches!(
            report.session_scope,
            SessionScope::Unindexed | SessionScope::Unavailable
        ) {
            section = section.child(div().text_color(muted).child(text(
                cx,
                "Session archive coverage is unknown. Browse sessions to inspect the archive.",
                "会话归档覆盖范围未知，可浏览会话检查归档。",
            )));
        }
        section
    }
}

fn action_row(
    id: &'static str,
    label: &'static str,
    icon: IconName,
    cx: &App,
) -> gpui_kit::base::Button {
    gpui_kit::base::Button::new(id)
        .accessibility_label(label)
        .mt(px(theme::CONTROL_GAP))
        .w_full()
        .px(px(theme::CONTROL_GAP))
        .h(px(theme::ROW_HEIGHT))
        .rounded(px(theme::SELECTION_RADIUS))
        .border_2()
        .border_color(gpui_kit::transparent_black())
        .hover(|style| style.bg(cx.theme().muted))
        .focus_visible(|style| style.border_color(cx.theme().ring))
        .child(
            div()
                .w_full()
                .flex()
                .items_center()
                .gap(px(theme::CONTROL_GAP))
                .child(
                    Icon::new(icon)
                        .size(px(16.))
                        .text_color(cx.theme().muted_foreground),
                )
                .child(div().flex_1().child(label)),
        )
}
