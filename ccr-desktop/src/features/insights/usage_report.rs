use super::*;
use ccr_app::usage::read::UsageSnapshot;
use gpui_kit::{AnyElement, App, Div, Role, ScrollHandle, prelude::FluentBuilder};

mod heatmap;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Page {
    #[default]
    Overview,
    Details,
    Records,
}

pub(super) struct Navigation {
    page: Page,
    ranking: Option<Section>,
    analysis: Option<Arc<InsightsSnapshot>>,
    pub(super) earlier_cursors: Vec<Option<String>>,
    pub(super) scroll: ScrollHandle,
    main_focus: FocusHandle,
    detail_focus: FocusHandle,
    heatmap: heatmap::State,
}

impl Navigation {
    pub(super) fn new(cx: &mut Context<InsightsView>) -> Self {
        Self {
            page: Page::Overview,
            ranking: None,
            analysis: None,
            earlier_cursors: Vec::new(),
            scroll: ScrollHandle::new(),
            main_focus: cx.focus_handle().tab_stop(false),
            detail_focus: cx.focus_handle().tab_stop(false),
            heatmap: heatmap::State::new(cx),
        }
    }

    pub(super) fn suspend(&mut self) {
        self.page = Page::Overview;
        self.ranking = None;
        self.analysis = None;
        self.earlier_cursors.clear();
        self.heatmap.reset();
        self.scroll_to_start();
    }

    fn scroll_to_start(&self) {
        self.scroll.set_offset(gpui_kit::point(px(0.), px(0.)));
    }
}

impl InsightsView {
    pub(super) fn focus_usage_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.usage_report.main_focus, cx);
        window.focus_next(cx);
    }

    pub(super) fn reset_usage_data(&mut self, filter: &QueryFilter) {
        let source = filter.source;
        if self
            .submitted_usage
            .as_ref()
            .is_some_and(|filter| filter.source != source)
        {
            self.usage_report.ranking = None;
        }
        self.usage_report.analysis = None;
        self.usage_report.earlier_cursors.clear();
        self.usage_report.heatmap.reset();
        if self.usage_report.page != Page::Records {
            self.section = self.usage_report.ranking.unwrap_or(if source.is_some() {
                Section::Models
            } else {
                Section::Sources
            });
        }
    }

    fn default_usage_ranking(&self) -> Section {
        if self
            .submitted_usage
            .as_ref()
            .is_some_and(|filter| filter.source.is_some())
        {
            Section::Models
        } else {
            Section::Sources
        }
    }

    fn open_usage_section(&mut self, page: Page, cx: &mut Context<Self>) {
        let current = self.usage_report.page;
        if current == page {
            return;
        }
        self.usage_report.page = page;
        self.detail_open = false;
        self.selected = None;
        self.usage_report.scroll_to_start();
        self.scroll.scroll_to_item(0, gpui_kit::ScrollStrategy::Top);
        match (current, page) {
            (_, Page::Records) => {
                self.usage_report.analysis = self.snapshot.take();
                self.section = Section::Logs;
                self.cursor = None;
                self.usage_report.earlier_cursors.clear();
                self.submit(cx);
            }
            (Page::Records, _) => {
                self.section = self
                    .usage_report
                    .ranking
                    .unwrap_or_else(|| self.default_usage_ranking());
                self.cursor = None;
                self.usage_report.earlier_cursors.clear();
                if let Some(analysis) = self.usage_report.analysis.take() {
                    self.gate.clear();
                    self.task = None;
                    self.generation += 1;
                    self.loading = false;
                    self.error = None;
                    self.snapshot = Some(analysis);
                    cx.notify();
                } else {
                    self.submit(cx);
                }
            }
            _ => {
                self.section = self
                    .usage_report
                    .ranking
                    .unwrap_or_else(|| self.default_usage_ranking());
                cx.notify();
            }
        }
    }

    pub(super) fn render_usage_page(
        &self,
        root: gpui_kit::base::ObservedElement<gpui_kit::Stateful<Div>>,
        auxiliary: Div,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let report = match self.snapshot.as_deref() {
            Some(InsightsSnapshot::Usage(report)) => Some(report),
            _ => None,
        };
        let context_report = report.or(match self.usage_report.analysis.as_deref() {
            Some(InsightsSnapshot::Usage(report)) => Some(report),
            _ => None,
        });
        let records_context = match self.snapshot.as_deref() {
            Some(InsightsSnapshot::Logs(_, context)) => context.as_ref(),
            _ => None,
        };
        root.overflow_y_scroll()
            .track_focus(&self.usage_report.main_focus)
            .tab_stop(false)
            .track_scroll(&self.usage_report.scroll)
            .child(self.report_toolbar(window, cx))
            .children(
                self.submitted_usage
                    .as_ref()
                    .map(|filter| usage::context_row(filter, context_report, records_context, cx)),
            )
            .children(self.platform_scope.is_none().then_some(auxiliary))
            .child(self.usage_navigation(cx))
            .children(self.usage_report_status(report, cx))
            .child(match self.usage_report.page {
                Page::Overview => self.usage_overview(report, window, cx).into_any_element(),
                Page::Details => self.usage_details(report, cx).into_any_element(),
                Page::Records => self.usage_records(cx).into_any_element(),
            })
            .into_any_element()
    }

    fn usage_navigation(&self, cx: &mut Context<Self>) -> Div {
        let mut tabs = crate::shell::page_header::capsule(cx);
        for (page, id, en, zh) in [
            (Page::Overview, "usage-view-overview", "Overview", "概览"),
            (Page::Details, "usage-view-details", "Details", "明细"),
            (Page::Records, "usage-view-records", "Records", "记录"),
        ] {
            tabs = tabs.child(semantics::tab(
                semantics::ghost_button(
                    Button::new(id).label(text(cx, en, zh)).on_click(
                        cx.listener(move |this, _, _, cx| this.open_usage_section(page, cx)),
                    ),
                    false,
                    false,
                    cx,
                ),
                self.usage_report.page == page,
                false,
                cx,
            ));
        }
        tabs
    }

    fn usage_report_status(&self, report: Option<&UsageSnapshot>, cx: &App) -> Option<AnyElement> {
        let status = if self.loading {
            text(cx, "Reading on background worker", "正在后台读取")
        } else if let Some(error) = self.error {
            service_error(cx, error)
        } else if report.is_some_and(|report| report.overview.bucket_count == 0)
            || matches!(self.snapshot.as_deref(), Some(InsightsSnapshot::Logs(page, _)) if page.records.is_empty())
        {
            text(cx, "No records in this range.", "此区间没有记录。")
        } else {
            return None;
        };
        Some(
            div()
                .id("insights-status")
                .test_support()
                .role(Role::Status)
                .aria_label(status)
                .text_color(cx.theme().muted_foreground)
                .child(status)
                .into_any_element(),
        )
    }

    fn usage_overview(
        &self,
        report: Option<&UsageSnapshot>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let readings = report.filter(|report| report.overview.bucket_count > 0);
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(px(theme::SECTION_GAP))
            .children((readings.is_some() || self.loading).then(|| {
                usage::readings(
                    readings,
                    window.viewport_size().width < px(theme::SPLIT_WIDTH),
                    cx,
                )
            }))
            .children(readings.map(|report| self.usage_trend(report, cx)))
            .children(readings.map(|_| {
                self.usage_trend
                    .token_composition(window.viewport_size().width < px(theme::SPLIT_WIDTH), cx)
            }))
            .children(readings.map(|report| self.usage_ranking(report, cx)))
            .children((readings.is_none() && self.loading).then(|| {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(theme::SECTION_GAP))
                    .child(loading_panel(
                        "usage-trend-loading",
                        text(cx, "Daily trend", "逐日趋势"),
                        250.,
                        cx,
                    ))
                    .child(loading_panel(
                        "usage-composition-loading",
                        text(cx, "Token composition", "Token 构成"),
                        110.,
                        cx,
                    ))
                    .child(loading_panel(
                        "usage-rank-loading",
                        self.ranking_title(cx),
                        180.,
                        cx,
                    ))
            }))
    }

    fn ranking_title(&self, cx: &App) -> &'static str {
        if self.default_usage_ranking() == Section::Sources {
            text(cx, "Top sources", "来源排行")
        } else {
            text(cx, "Top models", "模型排行")
        }
    }

    fn usage_ranking(&self, report: &UsageSnapshot, cx: &mut Context<Self>) -> Div {
        let sources = self.default_usage_ranking() == Section::Sources;
        let title = self.ranking_title(cx);
        let mut rows = div();
        if sources {
            for source in report.sources.iter().take(5) {
                let name = SourceKind::parse_id(&source.source)
                    .map(|kind| import::source_label(Some(kind), cx))
                    .unwrap_or(&source.source);
                rows = rows.child(rank_row(
                    &source.source,
                    name,
                    source.total_tokens,
                    source.event_count,
                    cx,
                ));
            }
        } else {
            for model in report.models.iter().take(5) {
                rows = rows.child(rank_row(
                    &model.model,
                    &model.model,
                    model.total_tokens,
                    model.event_count,
                    cx,
                ));
            }
        }
        report_panel(cx)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .child(heading(title, cx))
                    .child(semantics::ghost_button(
                        Button::new("usage-open-ranking")
                            .label(text(cx, "View full ranking", "查看完整排行"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.usage_report.ranking = None;
                                this.open_usage_section(Page::Details, cx);
                            })),
                        false,
                        false,
                        cx,
                    )),
            )
            .child(report_columns(
                text(cx, "Name", "名称"),
                text(cx, "Tokens", "Token"),
                text(cx, "Requests", "请求"),
                cx,
            ))
            .child(rows)
    }

    fn usage_details(&self, report: Option<&UsageSnapshot>, cx: &mut Context<Self>) -> Div {
        let Some(report) = report else {
            return if self.loading {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(theme::SECTION_GAP))
                    .child(loading_panel(
                        "usage-ranking-loading",
                        self.ranking_title(cx),
                        190.,
                        cx,
                    ))
                    .child(loading_panel(
                        "usage-heatmap-loading",
                        text(cx, "Activity heatmap", "活动热力图"),
                        160.,
                        cx,
                    ))
            } else {
                div()
            };
        };
        let mut tabs = crate::shell::page_header::capsule(cx);
        for (section, en, zh) in [
            (Section::Sources, "Sources", "来源"),
            (Section::Models, "Models", "模型"),
            (Section::Providers, "Providers", "提供商"),
            (Section::Projects, "Projects", "项目"),
        ] {
            tabs = tabs.child(semantics::tab(
                semantics::ghost_button(
                    Button::new(SharedString::from(format!("insights-section-{en}")))
                        .label(text(cx, en, zh))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.section = section;
                            this.usage_report.ranking = Some(section);
                            this.detail_open = false;
                            this.selected = None;
                            this.scroll.scroll_to_item(0, gpui_kit::ScrollStrategy::Top);
                            cx.notify();
                        })),
                    false,
                    false,
                    cx,
                ),
                self.section == section,
                false,
                cx,
            ));
        }
        let table = if self.section == Section::Providers && !report.providers_supported {
            Some(status_line(
                "usage-provider-unsupported",
                text(
                    cx,
                    "Provider breakdown unavailable for this database.",
                    "当前数据库不支持提供商明细。",
                ),
                cx,
            ))
        } else if self.row_count() == 0 {
            Some(status_line(
                "usage-ranking-empty",
                text(
                    cx,
                    "No matching rows for this dimension.",
                    "此维度没有匹配的记录。",
                ),
                cx,
            ))
        } else {
            Some(self.usage_table(cx).into_any_element())
        };
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(px(theme::SECTION_GAP))
            .child(
                report_panel(cx)
                    .id("usage-ranking-panel")
                    .test_support()
                    .child(tabs)
                    .children(table),
            )
            .children(
                report
                    .heatmap
                    .as_deref()
                    .and_then(|points| self.usage_heatmap(points, cx)),
            )
    }

    fn usage_records(&self, cx: &mut Context<Self>) -> Div {
        let Some(InsightsSnapshot::Logs(page, _)) = self.snapshot.as_deref() else {
            return if self.loading {
                loading_panel(
                    "usage-records-loading",
                    text(cx, "Records", "记录"),
                    310.,
                    cx,
                )
            } else {
                div()
            };
        };
        if page.records.is_empty() {
            return div();
        }
        let page_count = page.records.len();
        let count = page.total.map_or_else(
            || {
                format!(
                    "{page_count} {}",
                    text(cx, "records on this page", "条本页记录")
                )
            },
            |total| {
                format!(
                    "{page_count} / {total} {}",
                    text(cx, "matching records", "条匹配记录")
                )
            },
        );
        let newer = !self.usage_report.earlier_cursors.is_empty();
        let older = self.has_next();
        report_panel(cx)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(status_line("usage-record-count", count, cx))
                    .children(newer.then(|| {
                        semantics::ghost_button(
                            Button::new("usage-newer-page")
                                .label(text(cx, "Newer page", "更新一页"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if let Some(cursor) = this.usage_report.earlier_cursors.pop() {
                                        this.cursor = cursor;
                                        this.submit(cx);
                                    }
                                })),
                            false,
                            false,
                            cx,
                        )
                    }))
                    .children(older.then(|| {
                        semantics::ghost_button(
                            Button::new("insights-next")
                                .label(text(cx, "Older page", "更早一页"))
                                .on_click(cx.listener(|this, _, _, cx| this.next(cx))),
                            false,
                            false,
                            cx,
                        )
                    })),
            )
            .child(self.usage_table(cx))
    }

    fn usage_table(&self, cx: &mut Context<Self>) -> Div {
        if self.detail_open
            && let Some(index) =
                (0..self.row_count()).find(|&index| self.row_key(index) == self.selected.as_deref())
        {
            return div().child(self.usage_exact_detail(index, cx));
        }
        let headers = self.header_text(cx);
        let count = self.row_count();
        let rows = uniform_list(
            "usage-exact-rows",
            count,
            cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                range
                    .filter_map(|index| this.row(index).map(|row| (index, row)))
                    .map(|(index, row)| this.usage_table_row(index, row, cx))
                    .collect()
            }),
        )
        .track_scroll(&self.scroll)
        .h(theme::text_size(theme::ROW_HEIGHT) * count.min(6) as f32)
        .w_full();
        div()
            .child(
                report_columns(headers.0, headers.1, headers.2, cx)
                    .id("usage-table-header")
                    .test_support(),
            )
            .child(
                div()
                    .id("insights-list-focus")
                    .test_support()
                    .role(Role::ListBox)
                    .aria_label(text(cx, "Exact-value report rows", "精确数值报告行"))
                    .track_focus(&self.focus)
                    .tab_stop(true)
                    .border_2()
                    .border_color(gpui_kit::transparent_black())
                    .focus_visible(|style| style.border_color(cx.theme().ring))
                    .on_key_down(
                        cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                            match event.keystroke.key.as_str() {
                                "down" => this.select_offset(true, cx),
                                "up" => this.select_offset(false, cx),
                                "enter" => {
                                    let index = (0..this.row_count())
                                        .find(|&i| this.row_key(i) == this.selected.as_deref())
                                        .unwrap_or(0);
                                    this.open_usage_detail(index, window, cx);
                                }
                                _ => return,
                            }
                            cx.stop_propagation();
                        }),
                    )
                    .child(rows),
            )
    }

    fn usage_table_row(&self, index: usize, row: rows::Row, cx: &mut Context<Self>) -> AnyElement {
        let label = row.label;
        let description = format!("{label} · {} · {}", row.left, row.right);
        let selected = self.selected.as_deref() == Some(&row.key);
        let weak = cx.weak_entity();
        div()
            .id(SharedString::from(format!("insights-row:{}", row.key)))
            .test_support()
            .role(Role::ListBoxOption)
            .aria_label(description)
            .aria_selected(selected)
            .aria_position_in_set(index + 1)
            .aria_size_of_set(self.row_count())
            .on_a11y_action(gpui_kit::AccessibleAction::Click, move |_, window, cx| {
                let _ = weak.update(cx, |this, cx| this.open_usage_detail(index, window, cx));
            })
            .h(theme::text_size(theme::ROW_HEIGHT))
            .border_b_1()
            .border_color(cx.theme().border)
            .when(selected, |row| row.bg(cx.theme().secondary))
            .hover(|style| style.bg(cx.theme().muted))
            .on_click(
                cx.listener(move |this, _, window, cx| this.open_usage_detail(index, window, cx)),
            )
            .child(report_columns(label, row.left, row.right, cx).h_full())
            .into_any_element()
    }

    fn usage_exact_detail(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let mut lines = self.details(index);
        if let Some(InsightsSnapshot::Logs(page, _)) = self.snapshot.as_deref()
            && let Some(record) = page.records.get(index)
            && !record.project_path.is_empty()
        {
            lines.push(format!(
                "{} · {}",
                text(cx, "Project", "项目"),
                record.project_path,
            ));
        }
        let exact = lines.join(" · ");
        div()
            .id("usage-row-detail")
            .track_focus(&self.usage_report.detail_focus)
            .tab_stop(false)
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" {
                        this.close_usage_detail(window, cx);
                        cx.stop_propagation();
                    }
                }),
            )
            .child(semantics::ghost_button(
                Button::new("usage-return-to-table")
                    .label(text(cx, "Back to table", "返回表格"))
                    .on_click(
                        cx.listener(|this, _, window, cx| this.close_usage_detail(window, cx)),
                    ),
                false,
                false,
                cx,
            ))
            .child(
                div()
                    .id("insights-exact-detail")
                    .test_support()
                    .role(Role::Label)
                    .aria_label(exact.clone())
                    .aria_value(exact)
                    .p(px(theme::CONTROL_GAP))
                    .children(lines.into_iter().map(|line| div().child(line))),
            )
    }

    fn open_usage_detail(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.row_key(index).is_none() {
            return;
        }
        self.select(index, cx);
        self.restore_usage_focus(true, window, cx);
    }

    fn close_usage_detail(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.detail_open = false;
        if let Some(index) =
            (0..self.row_count()).find(|&index| self.row_key(index) == self.selected.as_deref())
        {
            self.scroll
                .scroll_to_item(index, gpui_kit::ScrollStrategy::Top);
        }
        self.restore_usage_focus(false, window, cx);
        cx.notify();
    }

    fn restore_usage_focus(&self, detail: bool, window: &mut Window, cx: &Context<Self>) {
        let guard = (self.generation, self.usage_report.page, self.section);
        let view = cx.weak_entity();
        window.on_next_frame(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                if guard != (this.generation, this.usage_report.page, this.section)
                    || this.detail_open != detail
                {
                    return;
                }
                if detail {
                    window.focus(&this.usage_report.detail_focus, cx);
                    window.focus_next(cx);
                } else {
                    window.focus(&this.focus, cx);
                }
            });
        });
    }
}

fn report_panel(cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .gap(px(theme::CONTROL_GAP))
        .rounded(px(theme::PANEL_RADIUS))
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().popover)
        .p(px(theme::SECTION_GAP))
}

fn loading_panel(id: &'static str, title: &'static str, height: f32, cx: &App) -> Div {
    report_panel(cx)
        .min_h(px(height))
        .child(div().id(id).test_support().child(heading(title, cx)))
}

fn heading(title: &'static str, cx: &App) -> Div {
    div()
        .text_size(theme::text_size(theme::SECTION_TITLE_SIZE))
        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
        .text_color(cx.theme().foreground)
        .child(title)
}

fn status_line(id: &'static str, value: impl Into<SharedString>, cx: &App) -> AnyElement {
    let value = value.into();
    div()
        .id(id)
        .test_support()
        .role(Role::Status)
        .aria_label(value.clone())
        .aria_value(value.clone())
        .text_size(theme::text_size(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .child(value)
        .into_any_element()
}

fn report_columns(
    name: impl Into<SharedString>,
    left: impl Into<SharedString>,
    right: impl Into<SharedString>,
    cx: &App,
) -> Div {
    div()
        .w_full()
        .px(px(theme::CONTROL_GAP))
        .flex()
        .items_center()
        .gap(px(theme::CONTROL_GAP))
        .child(div().flex_1().min_w_0().truncate().child(name.into()))
        .child(numeric_column(left, cx))
        .child(numeric_column(right, cx))
}

fn numeric_column(value: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .w(gpui_kit::relative(0.20))
        .min_w(px(70.))
        .text_right()
        .font_features(theme::tabular_figures())
        .text_color(cx.theme().foreground)
        .child(value.into())
}

fn rank_row(key: &str, label: &str, tokens: i64, requests: i64, cx: &App) -> AnyElement {
    let exact = format!(
        "{label} · {} {tokens} · {} {requests}",
        text(cx, "Tokens", "Token"),
        text(cx, "Requests", "请求"),
    );
    div()
        .id(SharedString::from(format!("usage-rank:{key}")))
        .test_support()
        .role(Role::Label)
        .aria_label(exact.clone())
        .aria_value(exact)
        .border_t_1()
        .border_color(cx.theme().border)
        .py(px(6.))
        .child(report_columns(
            label.to_string(),
            tokens.to_string(),
            requests.to_string(),
            cx,
        ))
        .into_any_element()
}
