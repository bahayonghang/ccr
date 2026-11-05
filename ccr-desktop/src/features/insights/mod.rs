//! Native read-only Insights workbench, using the shared bounded request adapter.
mod policies;
mod product;
mod rows;
use crate::{
    locales::{is_chinese, localized_line, service_error, text},
    runtime::{
        Services,
        insights::{
            InsightsError, InsightsQuery, InsightsRequests, InsightsSnapshot, RefreshStatus,
        },
    },
    shell::semantics,
    theme,
};
use ccr_app::{
    runtime::{
        AdmissionError, Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent,
    },
    sessions::SessionListRequest,
};
use ccr_store::sessions::AgentSessionAgentId;
use ccr_usage::{QueryFilter, ReportTimezone, SourceKind};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};
use gpui_kit::{
    AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Task, UniformListScrollHandle,
    Window, div, px, uniform_list,
};
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InsightsPage {
    Home,
    Usage,
    Sessions,
    Policies,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Sources,
    Trends,
    Models,
    Providers,
    Projects,
    Heatmap,
    Logs,
}
impl Section {
    const ALL: [(Self, &'static str, &'static str); 7] = [
        (Self::Sources, "Sources", "来源"),
        (Self::Trends, "Daily", "逐日"),
        (Self::Models, "Models", "模型"),
        (Self::Providers, "Providers", "提供商"),
        (Self::Projects, "Projects", "项目"),
        (Self::Heatmap, "Heatmap values", "热力数值"),
        (Self::Logs, "Records", "记录"),
    ];
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Surface {
    Workbench,
    Sessions,
    Usage,
}

pub struct InsightsView {
    requests: InsightsRequests,
    policies: Entity<policies::PoliciesView>,
    page: InsightsPage,
    surface: Surface,
    section: Section,
    snapshot: Option<Arc<InsightsSnapshot>>,
    fields: Vec<Entity<InputState>>,
    utc: bool,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    refresh_task: Option<Task<()>>,
    loading: bool,
    error: Option<&'static str>,
    selected: Option<String>,
    detail_open: bool,
    archive_id: Option<String>,
    cursor: Option<String>,
    before: Option<u32>,
    focus: FocusHandle,
    scroll: UniformListScrollHandle,
    #[cfg(test)]
    rendered_rows: usize,
}
impl InsightsView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let requests = InsightsRequests::new(&services);
        let policies = cx.new(|cx| policies::PoliciesView::new(requests.clone(), window, cx));
        Self {
            error: (!requests.configured()).then_some(InsightsError::Unconfigured.message()),
            requests,
            policies,
            page: InsightsPage::Home,
            surface: Surface::Workbench,
            section: Section::Sources,
            snapshot: None,
            fields: (0..8)
                .map(|_| cx.new(|cx| InputState::new(window, cx)))
                .collect(),
            utc: false,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            refresh_task: None,
            loading: false,
            selected: None,
            detail_open: false,
            archive_id: None,
            cursor: None,
            before: None,
            focus: cx.focus_handle(),
            scroll: UniformListScrollHandle::new(),
            #[cfg(test)]
            rendered_rows: 0,
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.policies.update(cx, |this, cx| this.suspend(cx));
        self.task = None;
        self.refresh_task = None;
        self.gate.clear();
        self.generation += 1;
        self.loading = false;
        self.snapshot = None;
        self.selected = None;
        self.archive_id = None;
        self.detail_open = false;
        self.cursor = None;
        self.before = None;
        cx.notify();
    }
    pub fn open_product(&mut self, page: InsightsPage, cx: &mut Context<Self>) {
        self.surface = if page == InsightsPage::Sessions {
            Surface::Sessions
        } else {
            Surface::Usage
        };
        let page = if page == InsightsPage::Home {
            InsightsPage::Usage
        } else {
            page
        };
        self.set_page(page, cx);
    }

    pub fn set_page(&mut self, page: InsightsPage, cx: &mut Context<Self>) {
        self.suspend(cx);
        self.page = page;
        if page == InsightsPage::Policies {
            self.policies.update(cx, |this, cx| {
                this.load(
                    crate::runtime::insights::policies::PolicyCommand::Budgets,
                    cx,
                )
            });
            return;
        }
        // Refresh writes archive summaries. Opening the page only reads what is already indexed.
        self.refresh(cx);
    }
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.cursor = None;
        self.before = None;
        self.archive_id = None;
        self.submit(cx);
    }
    fn start_archive_refresh(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        let generation = self.generation;
        let key = RequestKey {
            environment: EnvironmentId::Local("insights-explicit".into()),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        match self.requests.refresh_archive(key) {
            Ok(Some(mut subscription)) => {
                self.refresh_task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(_) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if this.page == InsightsPage::Sessions
                                    && this.archive_id.is_none()
                                    && this.generation == generation
                                {
                                    this.refresh(cx);
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Ok(None) => {}
            Err(error) => {
                self.error = Some(match error {
                    AdmissionError::Full => "Refresh queue busy. Retry. / 刷新队列繁忙，请重试",
                    AdmissionError::Stopped => "Application is shutting down / 应用正在退出",
                });
            }
        }
        cx.notify();
    }
    fn filter(&self, cx: &Context<Self>) -> Result<QueryFilter, InsightsError> {
        let value = |i: usize| self.fields[i].read(cx).value().trim().to_string();
        let optional = |i| {
            let value = value(i);
            (!value.is_empty()).then_some(value)
        };
        let mut filter = ccr_usage::build_filter(None, optional(1), optional(4), optional(5))
            .map_err(|_| InsightsError::Query)?;
        filter.source = optional(0)
            .map(|value| SourceKind::parse_id(&value).ok_or(InsightsError::Query))
            .transpose()?;
        filter.provider = optional(2);
        filter.project_hash = optional(3);
        filter.timezone = if self.utc {
            ReportTimezone::Utc
        } else {
            ReportTimezone::Local
        };
        if filter
            .since
            .zip(filter.until)
            .is_some_and(|(start, end)| start > end)
        {
            return Err(InsightsError::Query);
        }
        Ok(filter)
    }
    fn query(&self, cx: &Context<Self>) -> Result<InsightsQuery, InsightsError> {
        match self.page {
            InsightsPage::Policies => Err(InsightsError::Query),
            InsightsPage::Home => {
                let generated_at = chrono::Utc::now();
                Ok(InsightsQuery::Home {
                    as_of: generated_at.with_timezone(&chrono::Local).date_naive(),
                    generated_at,
                })
            }
            InsightsPage::Usage => {
                let filter = self.filter(cx)?;
                Ok(if self.section == Section::Logs {
                    InsightsQuery::Logs {
                        filter,
                        cursor: self.cursor.clone(),
                    }
                } else {
                    InsightsQuery::Usage(filter)
                })
            }
            InsightsPage::Sessions => {
                if let Some(archive_id) = &self.archive_id {
                    return Ok(InsightsQuery::Detail {
                        archive_id: archive_id.clone(),
                        before: self.before,
                    });
                }
                let family = self.fields[7].read(cx).value().trim().to_string();
                let agents = if family.is_empty() {
                    None
                } else {
                    Some(vec![
                        AgentSessionAgentId::parse(&family).ok_or(InsightsError::Query)?,
                    ])
                };
                let query = self.fields[6].read(cx).value().trim().to_string();
                Ok(InsightsQuery::Sessions(SessionListRequest {
                    agents,
                    query: (!query.is_empty()).then_some(query),
                    cursor: self.cursor.clone(),
                    limit: Some(100),
                    ..Default::default()
                }))
            }
        }
    }
    fn submit(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.generation += 1;
        self.snapshot = None;
        self.selected = None;
        self.detail_open = false;
        self.error = None;
        let query = match self.query(cx) {
            Ok(query) => query,
            Err(error) => {
                self.error = Some(error.message());
                self.loading = false;
                cx.notify();
                return;
            }
        };
        let key = RequestKey {
            environment: EnvironmentId::Local("insights-explicit".into()),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        self.loading = true;
        match self.requests.load(key.clone(), query) {
            Ok(mut subscription) => {
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) {
                                    return;
                                }
                                this.loading = false;
                                match result.as_ref() {
                                    Completion::Success(snapshot) => {
                                        this.snapshot = Some(snapshot.clone())
                                    }
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => {
                                        this.error = Some(error.message())
                                    }
                                    Completion::Cancelled => {
                                        this.error = Some("Read cancelled / 已取消读取")
                                    }
                                    Completion::WorkerFailed => {
                                        this.error = Some(
                                            "Read worker stopped. Retry. / 读取线程已停止，请重试",
                                        )
                                    }
                                }
                                this.scroll.scroll_to_item(0, gpui_kit::ScrollStrategy::Top);
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(error) => {
                self.loading = false;
                self.error = Some(match error {
                    AdmissionError::Full => "Read queue busy. Retry. / 读取队列繁忙，请重试",
                    AdmissionError::Stopped => "Application is shutting down / 应用正在退出",
                });
            }
        }
        cx.notify();
    }
    fn next(&mut self, cx: &mut Context<Self>) {
        match self.snapshot.as_deref() {
            Some(InsightsSnapshot::Logs(page)) => {
                let Some(cursor) = page.next_cursor.clone() else {
                    return;
                };
                self.cursor = Some(cursor);
            }
            Some(InsightsSnapshot::Sessions { page, .. }) => {
                let Some(cursor) = page.next_cursor.clone() else {
                    return;
                };
                self.cursor = Some(cursor);
            }
            Some(InsightsSnapshot::Detail(detail)) => {
                let Some(before) = detail.page.next_before else {
                    return;
                };
                self.before = Some(before);
            }
            _ => return,
        }
        self.submit(cx);
    }
    fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(InsightsSnapshot::Sessions { page, .. }) = self.snapshot.as_deref() {
            self.archive_id = Some(page.items[index].archive_id.clone());
            self.before = None;
            self.submit(cx);
            return;
        }
        self.selected = self.row(index).map(|row| row.key);
        self.detail_open = true;
        cx.notify();
    }
    fn select_offset(&mut self, down: bool, cx: &mut Context<Self>) {
        let count = self.row_count();
        if count == 0 {
            return;
        }
        let index = (0..count).find(|&index| self.row_key(index) == self.selected.as_deref());
        let next = match (index, down) {
            (Some(i), true) => (i + 1).min(count - 1),
            (Some(i), false) => i.saturating_sub(1),
            _ => 0,
        };
        self.selected = self.row(next).map(|row| row.key);
        self.scroll
            .scroll_to_item(next, gpui_kit::ScrollStrategy::Top);
        cx.notify();
    }
    fn has_next(&self) -> bool {
        match self.snapshot.as_deref() {
            Some(InsightsSnapshot::Logs(page)) => page.next_cursor.is_some(),
            Some(InsightsSnapshot::Sessions { page, .. }) => page.next_cursor.is_some(),
            Some(InsightsSnapshot::Detail(detail)) => detail.page.has_older,
            _ => false,
        }
    }
}
impl InsightsView {
    fn product_status(&self, cx: &Context<Self>) -> String {
        let Some(InsightsSnapshot::Usage(report)) = self.snapshot.as_deref() else {
            return text(cx, "This report has not been read.", "还没有读到这份报告。").to_string();
        };
        let sync =
            report
                .overview
                .last_sync_at
                .as_deref()
                .unwrap_or(text(cx, "not recorded", "未记录"));
        let providers = if report.providers_supported {
            text(cx, "Provider dimension available", "提供商维度可用")
        } else {
            text(
                cx,
                "Provider dimension unsupported, not measured zero",
                "提供商维度不支持，不是实测零值",
            )
        };
        if is_chinese(cx) {
            format!(
                "请求 {} · 令牌 {} · 美元 {} · 最近同步 {} · {}",
                report.overview.total_events,
                report.overview.total.total_tokens,
                report.overview.total_cost_usd,
                sync,
                providers
            )
        } else {
            format!(
                "Requests {} · Tokens {} · USD {} · Last sync {} · {}",
                report.overview.total_events,
                report.overview.total.total_tokens,
                report.overview.total_cost_usd,
                sync,
                providers
            )
        }
    }

    fn header_text(&self, cx: &Context<Self>) -> (String, String, String) {
        match self.section {
            Section::Heatmap => (
                text(cx, "Date", "日期").into(),
                text(cx, "Requests", "请求").into(),
                String::new(),
            ),
            Section::Logs => (
                text(cx, "Time", "时间").into(),
                text(cx, "Input", "输入").into(),
                text(cx, "Output", "输出").into(),
            ),
            _ => (
                text(cx, "Label", "名称").into(),
                text(cx, "Tokens", "令牌").into(),
                text(cx, "Requests", "请求").into(),
            ),
        }
    }
}

impl Render for InsightsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.surface == Surface::Sessions {
            return product::sessions(self, cx).into_any_element();
        }
        let mut root = div()
            .id("insights-root")
            .test_support()
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .gap(px(theme::CONTROL_GAP))
            .p(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .rounded_tr(px(theme::SHEET_CONTENT_RADIUS));
        let mut tabs = div().flex().gap(px(theme::CONTROL_GAP));
        if self.surface == Surface::Usage {
            root = root
                .child(
                    div()
                        .text_size(px(theme::TITLE_SIZE))
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .child(text(cx, "Usage", "用量")),
                )
                .child(
                    div()
                        .text_size(px(theme::META_SIZE))
                        .text_color(cx.theme().muted_foreground)
                        .child(text(
                            cx,
                            "Figures already stored in the local usage database.",
                            "这些数字来自本机用量库，没有示例填充。",
                        )),
                );
        }
        if self.surface == Surface::Workbench {
            for (page, id, en, zh) in [
                (InsightsPage::Usage, "insights-usage", "Usage", "用量"),
                (
                    InsightsPage::Sessions,
                    "insights-sessions",
                    "Sessions",
                    "会话",
                ),
                (
                    InsightsPage::Policies,
                    "insights-policies-tab",
                    "Budget / pricing",
                    "预算 / 定价",
                ),
            ] {
                tabs = tabs.child(semantics::button(
                    Button::new(id)
                        .label(text(cx, en, zh))
                        .ghost()
                        .toggled(self.page == page)
                        .on_click(cx.listener(move |this, _, _, cx| this.set_page(page, cx))),
                    false,
                    false,
                    cx,
                ));
            }
        }
        if self.page == InsightsPage::Policies {
            return root
                .child(tabs)
                .child(self.policies.clone())
                .into_any_element();
        }
        tabs = tabs.child(semantics::button(
            Button::new("insights-refresh")
                .label(text(cx, "Refresh", "刷新"))
                .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
            self.loading,
            self.loading,
            cx,
        ));
        root = root.child(tabs);
        if self.page == InsightsPage::Sessions {
            let status = self.requests.refresh_status();
            let running = matches!(status, RefreshStatus::Running(_));
            let chinese = is_chinese(cx);
            let status_text = match status {
                RefreshStatus::Idle => {
                    text(cx, "Archive refresh has not run.", "尚未刷新归档。").to_string()
                }
                RefreshStatus::Running(id) => {
                    if chinese {
                        format!("归档刷新 {id} 进行中。离开页面后仍由应用持有。")
                    } else {
                        format!(
                            "Archive refresh {id} is running. Leaving this page keeps it with the application."
                        )
                    }
                }
                RefreshStatus::Finished(id, report) => {
                    if chinese {
                        format!(
                            "刷新 {id}：发现 {}，未变 {}，已解析 {}，写入 {}，部分 {}，锁定 {}，错误 {}",
                            report.total.discovered,
                            report.total.unchanged,
                            report.total.parsed,
                            report.total.upserted,
                            report.total.partial,
                            report.total.locked,
                            report.total.errors
                        )
                    } else {
                        format!(
                            "Refresh {id}: discovered {}, unchanged {}, parsed {}, upserted {}, partial {}, locked {}, errors {}",
                            report.total.discovered,
                            report.total.unchanged,
                            report.total.parsed,
                            report.total.upserted,
                            report.total.partial,
                            report.total.locked,
                            report.total.errors
                        )
                    }
                }
                RefreshStatus::Failed(id, error) => format!(
                    "Refresh {id}: {}",
                    crate::locales::localized_line(cx, error.message())
                ),
                RefreshStatus::Interrupted(id) => {
                    if chinese {
                        format!("刷新 {id} 没有结果。请重试。")
                    } else {
                        format!("Refresh {id} stopped before a result. Retry.")
                    }
                }
            };
            root = root.child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("insights-index-refresh")
                            .label(text(cx, "Refresh archive", "刷新归档"))
                            .on_click(cx.listener(|this, _, _, cx| this.start_archive_refresh(cx))),
                        running,
                        running,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("insights-index-status")
                            .label(text(cx, "Refresh status", "刷新状态"))
                            .ghost()
                            .on_click(cx.listener(|_, _, _, cx| cx.notify())),
                        false,
                        false,
                        cx,
                    ))
                    .child(
                        div()
                            .id("insights-index-status-text")
                            .test_support()
                            .text_size(px(theme::META_SIZE))
                            .child(status_text),
                    ),
            );
        }
        if self.page != InsightsPage::Home {
            let mut filters = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
            let indices: &[usize] = if self.page == InsightsPage::Usage {
                &[0, 1, 2, 3, 4, 5]
            } else {
                &[7, 6]
            };
            for &index in indices {
                let label = match index {
                    0 => text(cx, "Source id (blank = all)", "来源 ID（空为全部）"),
                    7 => text(
                        cx,
                        "Agent id (pi and omp distinct)",
                        "会话家族（pi 与 omp 独立）",
                    ),
                    1 => text(cx, "Model", "模型"),
                    2 => text(cx, "Provider", "提供商"),
                    3 => text(cx, "Project hash", "项目哈希"),
                    4 => text(cx, "Since YYYY-MM-DD", "开始 YYYY-MM-DD"),
                    5 => text(cx, "Until YYYY-MM-DD", "结束 YYYY-MM-DD"),
                    _ => text(cx, "Session title search", "会话标题搜索"),
                };
                filters = filters.child(
                    div()
                        .w(px(184.))
                        .child(div().text_size(px(theme::META_SIZE)).child(label))
                        .child(
                            Input::new(&self.fields[index])
                                .id(SharedString::from(format!("insights-filter-{index}")))
                                .aria_label(label),
                        ),
                );
            }
            root = root.child(filters);
        }
        if self.page == InsightsPage::Usage {
            let mut sections = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
            for (section, en, zh) in Section::ALL {
                sections = sections.child(semantics::button(
                    Button::new(SharedString::from(format!("insights-section-{en}")))
                        .label(text(cx, en, zh))
                        .ghost()
                        .toggled(self.section == section)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.section = section;
                            this.refresh(cx);
                        })),
                    self.loading,
                    false,
                    cx,
                ));
            }
            sections = sections.child(semantics::button(
                Button::new("insights-timezone")
                    .label(if self.utc {
                        "UTC"
                    } else {
                        text(cx, "Local", "本地")
                    })
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.utc = !this.utc;
                        this.refresh(cx);
                    })),
                self.loading,
                false,
                cx,
            ));
            root=root.child(sections).child(div().text_size(px(theme::META_SIZE)).child(text(cx,"Source IDs: claude, codex, opencode, antigravity, kimi_code, pi, grok, zcode, deepseek_harness. Inclusive dates in selected reporting zone.","来源为九个规范 ID；日期按所选报告时区包含首尾日。")));
        }
        if self.page == InsightsPage::Home {
            root=root.child(div().text_size(px(theme::META_SIZE)).child(text(cx,"All-history usage and indexed sessions; seven-day windows use the local reporting date. Usage filters do not affect this join.","全历史用量与已索引会话；七日窗口按本地报告日期。用量筛选不影响此联合报告。")));
        }
        if self.page == InsightsPage::Sessions {
            root=root.child(div().text_size(px(theme::META_SIZE)).child(text(cx,"Archive refresh reads explicit provider roots into CCR summaries; no usage sync/import is started. Transcript plaintext stays scoped to this view; leave to clear it.","归档刷新只读指定提供方并更新 CCR 摘要，不启动用量同步或导入。正文仅在本页临时显示，不写日志，离页清除。")));
        }
        root = root.child(
            div()
                .id("insights-status")
                .test_support()
                .role(gpui_kit::Role::Status)
                .child(if self.loading {
                    text(cx, "Reading on background worker", "正在后台读取").to_string()
                } else if let Some(error) = self.error {
                    service_error(cx, error).to_string()
                } else if self.surface == Surface::Workbench {
                    self.summary()
                } else {
                    self.product_status(cx)
                }),
        );
        if let Some(InsightsSnapshot::Sessions { statuses, .. }) = self.snapshot.as_deref() {
            root = root.child(
                div().text_size(px(theme::META_SIZE)).child(
                    statuses
                        .iter()
                        .map(|status| {
                            format!(
                                "{}: {:?}/{}",
                                status.agent.as_str(),
                                status.availability,
                                status
                                    .fidelity
                                    .map(|f| f.as_str())
                                    .unwrap_or("not measured")
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" · "),
                ),
            );
        }
        if self.page == InsightsPage::Home
            || (self.page == InsightsPage::Usage && self.section == Section::Sources)
        {
            root = root.child(self.comparison(cx));
        }
        let count = self.row_count();
        let list = uniform_list(
            "insights-rows",
            count,
            cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                #[cfg(test)]
                {
                    this.rendered_rows = range.len();
                }
                range
                    .filter_map(|index| this.row(index).map(|row| (index, row)))
                    .map(|(index, row)| {
                        let label = localized_line(cx, &row.label);
                        let selected = this.selected.as_ref() == Some(&row.key);
                        let weak = cx.weak_entity();
                        div()
                            .id(SharedString::from(format!("insights-row:{}", row.key)))
                            .test_support()
                            .role(gpui_kit::Role::ListBoxOption)
                            .aria_label(format!("{label} {} {}", row.left, row.right))
                            .aria_selected(selected)
                            .aria_position_in_set(index + 1)
                            .aria_size_of_set(this.row_count())
                            .on_a11y_action(gpui_kit::AccessibleAction::Click, move |_, _, cx| {
                                let _ = weak.update(cx, |this, cx| this.select(index, cx));
                            })
                            .w_full()
                            .h(px(theme::ROW_HEIGHT))
                            .px(px(theme::CONTROL_GAP))
                            .flex()
                            .items_center()
                            .gap(px(theme::SECTION_GAP))
                            .bg(if selected {
                                cx.theme().secondary
                            } else {
                                cx.theme().background
                            })
                            .hover(|style| style.bg(cx.theme().muted))
                            .on_click(cx.listener(move |this, _, _, cx| this.select(index, cx)))
                            .child(div().flex_1().overflow_hidden().child(label))
                            .child(
                                div()
                                    .min_w(px(100.))
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_right()
                                    .child(row.left),
                            )
                            .child(
                                div()
                                    .min_w(px(100.))
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_right()
                                    .child(row.right),
                            )
                            .into_any_element()
                    })
                    .collect()
            }),
        )
        .track_scroll(&self.scroll)
        .flex_1()
        .size_full();
        let headers = self.header_text(cx);
        root = root.child(
            div()
                .flex()
                .gap(px(theme::SECTION_GAP))
                .text_size(px(theme::META_SIZE))
                .child(div().flex_1().child(headers.0.clone()))
                .child(div().min_w(px(100.)).text_right().child(headers.1.clone()))
                .child(div().min_w(px(100.)).text_right().child(headers.2.clone())),
        );
        root = root.child(
            div()
                .id("insights-list-focus")
                .test_support()
                .track_focus(&self.focus)
                .tab_stop(true)
                .role(gpui_kit::Role::ListBox)
                .aria_label(text(cx, "Exact-value report rows", "精确数值报告行"))
                .flex_1()
                .min_h_0()
                .on_key_down(cx.listener(|this, event: &gpui_kit::KeyDownEvent, _, cx| {
                    match event.keystroke.key.as_str() {
                        "down" => this.select_offset(true, cx),
                        "up" => this.select_offset(false, cx),
                        "enter" => {
                            if let Some(index) = (0..this.row_count())
                                .find(|&i| this.row_key(i) == this.selected.as_deref())
                            {
                                this.select(index, cx);
                            }
                        }
                        "escape" => {
                            this.detail_open = false;
                            this.selected = None;
                            cx.notify();
                        }
                        _ => return,
                    }
                    cx.stop_propagation();
                }))
                .child(list),
        );
        if !self.loading && self.error.is_none() && self.snapshot.is_some() && count == 0 {
            root = root.child(if self.surface == Surface::Usage {
                text(
                    cx,
                    "This report has no rows.",
                    "这份报告没有记录。",
                )
            } else {
                text(
                    cx,
                    "No rows in this report. This is not proof that history or sessions were imported.",
                    "此报告没有记录，不代表历史或会话已完成导入。",
                )
            });
        }
        if self.detail_open
            && let Some(index) =
                (0..count).find(|&index| self.row_key(index) == self.selected.as_deref())
        {
            root = root.child(
                div()
                    .id("insights-exact-detail")
                    .test_support()
                    .h(px(160.))
                    .overflow_y_scroll()
                    .bg(cx.theme().secondary)
                    .p(px(theme::CONTROL_GAP))
                    .children(
                        self.details(index)
                            .into_iter()
                            .map(|line| div().child(localized_line(cx, &line))),
                    ),
            );
        }
        let mut paging = div().flex().gap(px(theme::CONTROL_GAP));
        if self.archive_id.is_some() {
            paging = paging.child(semantics::button(
                Button::new("insights-back")
                    .label(text(cx, "Back to archive", "返回归档"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                self.loading,
                false,
                cx,
            ));
        }
        if self.has_next() {
            paging = paging.child(semantics::button(
                Button::new("insights-next")
                    .label(text(cx, "Older page", "更早一页"))
                    .on_click(cx.listener(|this, _, _, cx| this.next(cx))),
                self.loading,
                false,
                cx,
            ));
        }
        root.child(paging).into_any_element()
    }
}
#[cfg(test)]
mod tests;
