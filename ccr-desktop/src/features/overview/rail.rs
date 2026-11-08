//! Right rail: next actions, the seven-day comparison and the data source summary.
use super::{Geometry, Node, OverviewLink, OverviewView, activity, chart::day_total, panel};
use crate::{
    locales::{is_chinese, text},
    runtime::insights::{SessionScope, WorkbenchReport},
    shell::badge,
    theme,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{ActiveTheme, Icon, IconName};
use gpui_kit::{
    App, Context, InteractiveElement, ParentElement, StatefulInteractiveElement, Styled, div, px,
};

impl OverviewView {
    pub(super) fn rail(
        &self,
        report: Option<&WorkbenchReport>,
        geometry: Geometry,
        cx: &mut Context<Self>,
    ) -> Node {
        let rail = div()
            .id("overview-rail")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::SECTION_GAP));
        let rail = if geometry.stacked {
            rail.w_full()
        } else {
            rail.w(px(geometry.rail_w)).flex_shrink_0()
        };
        let rail = rail.child(self.actions(cx));
        // 对比来自报告：还没读到报告时不画，也不铺 0。
        let rail = match report {
            Some(report) => rail.child(activity::week_panel(report, cx)),
            None => rail,
        };
        rail.child(self.data_source(report, cx))
    }

    fn actions(&self, cx: &mut Context<Self>) -> Node {
        let mut actions = panel("overview-actions", cx)
            .flex_shrink_0()
            .gap(px(theme::CONTROL_GAP))
            .child(badge::section_heading(
                text(cx, "Next actions", "下一步"),
                None,
                cx,
            ));
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
            actions = actions.child(
                action_row(id, label, icon, cx)
                    .on_click(cx.listener(move |this, _, _, cx| this.open(link, cx))),
            );
        }
        actions
    }

    /// Data source summary from state already in this view; no extra query.
    /// This panel takes the column's spare height: readings stay at the top and
    /// the coverage footer stays at the bottom.
    fn data_source(&self, report: Option<&WorkbenchReport>, cx: &App) -> Node {
        let configured = self.requests.configured();
        let mut section = panel("overview-data-source", cx)
            .flex_1()
            .gap(px(theme::CONTROL_GAP))
            .text_size(px(theme::META_SIZE))
            .child(badge::section_heading(
                text(cx, "Data source", "数据源"),
                None,
                cx,
            ))
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
        match report {
            Some(report) => self.report_source(section, report, cx),
            None => section.child(muted_line(
                text(
                    cx,
                    "Read local usage to inspect source and archive coverage.",
                    "读取本机用量后，可查看来源与归档覆盖范围。",
                ),
                cx,
            )),
        }
    }

    fn report_source(&self, section: Node, report: &WorkbenchReport, cx: &App) -> Node {
        let zone = self.zone();
        let section = section
            .child(muted_line(
                if is_chinese(cx) {
                    format!("报告截至 {}{zone}", report.as_of)
                } else {
                    format!("Report as of {}{zone}", report.as_of)
                },
                cx,
            ))
            .child(muted_line(self.sync_line(report, cx), cx));
        // 成功同步已过期才提示较新用量可能缺失；没有成功记录时上一行已写明。
        let section = if report.freshness.stale && report.freshness.local.is_some() {
            section.child(stale_note(cx))
        } else {
            section
        };
        section.child(source_footer(report, cx))
    }
}

fn muted_line(line: impl Into<gpui_kit::SharedString>, cx: &App) -> gpui_kit::Div {
    div()
        .text_color(cx.theme().muted_foreground)
        .child(line.into())
}

fn stale_note(cx: &App) -> Node {
    let note = text(
        cx,
        "Newer usage may be missing until a sync succeeds",
        "同步成功前，较新的用量可能缺失",
    );
    div()
        .id("overview-source-stale")
        .test_support()
        .aria_label(note)
        .text_color(cx.theme().muted_foreground)
        .child(note)
}

// 脚注贴面板底部：吸收的高度只表现为读数与脚注之间的间距。
fn source_footer(report: &WorkbenchReport, cx: &App) -> gpui_kit::Div {
    let active = report.days.iter().filter(|day| day_total(day) > 0).count();
    let coverage = if report.days.is_empty() {
        text(cx, "Daily breakdown unavailable", "逐日明细不可用").to_string()
    } else if is_chinese(cx) {
        format!("图表来源在 {} 日中有 {active} 日活跃", report.days.len())
    } else {
        format!(
            "Chart sources active on {active} of {} days",
            report.days.len()
        )
    };
    let footer = div()
        .mt_auto()
        .pt(px(theme::CONTROL_GAP))
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP))
        .child(muted_line(coverage, cx));
    if matches!(
        report.session_scope,
        SessionScope::Unindexed | SessionScope::Unavailable
    ) {
        footer.child(muted_line(
            text(
                cx,
                "Session archive coverage is unknown. Browse sessions to inspect the archive.",
                "会话归档覆盖范围未知，可浏览会话检查归档。",
            ),
            cx,
        ))
    } else {
        footer
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
