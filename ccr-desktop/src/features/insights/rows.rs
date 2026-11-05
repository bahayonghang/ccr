use super::*;
pub(super) struct Row {
    pub key: String,
    pub label: String,
    pub left: String,
    pub right: String,
}
impl InsightsView {
    pub(super) fn row_count(&self) -> usize {
        match self.snapshot.as_deref() {
            Some(InsightsSnapshot::Home(home)) => home.agents.len(),
            Some(InsightsSnapshot::Usage(report)) => match self.section {
                Section::Sources => report.sources.len(),
                Section::Trends => report.trends.len(),
                Section::Models => report.models.len(),
                Section::Providers => report.providers.len(),
                Section::Projects => report.projects.len(),
                Section::Heatmap => report.heatmap.as_ref().map_or(0, Vec::len),
                Section::Logs => 0,
            },
            Some(InsightsSnapshot::Logs(page)) => page.records.len(),
            Some(InsightsSnapshot::Sessions { page, .. }) => page.items.len(),
            Some(InsightsSnapshot::Detail(detail)) => detail.page.messages.len(),
            None => 0,
        }
    }
    pub(super) fn row_key(&self, index: usize) -> Option<&str> {
        match self.snapshot.as_deref()? {
            InsightsSnapshot::Home(home) => home.agents.get(index).map(|row| row.source.as_str()),
            InsightsSnapshot::Usage(report) => match self.section {
                Section::Sources => report.sources.get(index).map(|row| row.source.as_str()),
                Section::Trends => report.trends.get(index).map(|row| row.date.as_str()),
                Section::Models => report.models.get(index).map(|row| row.model.as_str()),
                Section::Providers => report
                    .providers
                    .get(index)
                    .map(|row| row.provider.as_deref().unwrap_or("")),
                Section::Projects => report
                    .projects
                    .get(index)
                    .map(|row| row.project_hash.as_str()),
                Section::Heatmap => report
                    .heatmap
                    .as_ref()?
                    .get(index)
                    .map(|row| row.date.as_str()),
                Section::Logs => None,
            },
            InsightsSnapshot::Logs(page) => page.records.get(index).map(|row| row.id.as_str()),
            InsightsSnapshot::Sessions { page, .. } => {
                page.items.get(index).map(|row| row.archive_id.as_str())
            }
            InsightsSnapshot::Detail(detail) => {
                detail.page.messages.get(index).map(|row| row.key.as_str())
            }
        }
    }
    pub(super) fn row(&self, index: usize) -> Option<Row> {
        let key = self.row_key(index)?.to_string();
        let (label, left, right) = match self.snapshot.as_deref()? {
            InsightsSnapshot::Home(home) => {
                let row = home.agents.get(index)?;
                (
                    row.source.clone(),
                    row.requests.to_string(),
                    if home.sessions_indexed {
                        row.sessions.to_string()
                    } else {
                        "not indexed".into()
                    },
                )
            }
            InsightsSnapshot::Usage(report) => match self.section {
                Section::Sources => {
                    let r = report.sources.get(index)?;
                    (
                        r.source.clone(),
                        r.total_tokens.to_string(),
                        r.event_count.to_string(),
                    )
                }
                Section::Trends => {
                    let r = report.trends.get(index)?;
                    (
                        r.date.clone(),
                        r.total_tokens.to_string(),
                        r.request_count.to_string(),
                    )
                }
                Section::Models => {
                    let r = report.models.get(index)?;
                    (
                        r.model.clone(),
                        r.total_tokens.to_string(),
                        r.event_count.to_string(),
                    )
                }
                Section::Providers => {
                    let r = report.providers.get(index)?;
                    (
                        r.display_provider().to_string(),
                        r.total_tokens.to_string(),
                        r.request_count.to_string(),
                    )
                }
                Section::Projects => {
                    let r = report.projects.get(index)?;
                    (
                        r.project_label.clone(),
                        r.total_tokens.to_string(),
                        r.event_count.to_string(),
                    )
                }
                Section::Heatmap => {
                    let r = report.heatmap.as_ref()?.get(index)?;
                    (r.date.clone(), r.event_count.to_string(), String::new())
                }
                Section::Logs => return None,
            },
            InsightsSnapshot::Logs(page) => {
                let r = page.records.get(index)?;
                (
                    format!(
                        "{} · {} · {}",
                        r.recorded_at,
                        r.platform,
                        r.model.as_deref().unwrap_or("unknown")
                    ),
                    r.input_tokens.to_string(),
                    r.output_tokens.to_string(),
                )
            }
            InsightsSnapshot::Sessions { page, .. } => {
                let r = page.items.get(index)?;
                (
                    format!(
                        "{} · {}",
                        r.platform,
                        r.title.as_deref().unwrap_or(&r.session_id)
                    ),
                    r.message_count.to_string(),
                    format!("{} / {}", r.source_state.as_str(), r.source_fidelity),
                )
            }
            InsightsSnapshot::Detail(detail) => {
                let r = detail.page.messages.get(index)?;
                (
                    format!(
                        "{} · {}{}",
                        r.ordinal,
                        r.role,
                        if r.clipped {
                            " · clipped / 已裁剪"
                        } else {
                            ""
                        }
                    ),
                    r.tool_name.clone().unwrap_or_default(),
                    r.timestamp.map(|t| t.to_rfc3339()).unwrap_or_default(),
                )
            }
        };
        Some(Row {
            key,
            label,
            left,
            right,
        })
    }
    pub(super) fn details(&self, index: usize) -> Vec<String> {
        let Some(snapshot) = self.snapshot.as_deref() else {
            return Vec::new();
        };
        match snapshot {
            InsightsSnapshot::Home(home) => {
                let r = &home.agents[index];
                vec![
                    format!(
                        "Source / 来源: {}{}",
                        r.source,
                        if r.unmapped {
                            " (unmapped / 未映射)"
                        } else {
                            ""
                        }
                    ),
                    format!(
                        "Requests / 请求: {} · Tokens / 令牌: {}",
                        r.requests, r.tokens
                    ),
                    format!(
                        "Sessions / 会话: {}",
                        if home.sessions_indexed {
                            r.sessions.to_string()
                        } else {
                            "not indexed".into()
                        }
                    ),
                ]
            }
            InsightsSnapshot::Usage(report) => match self.section {
                Section::Models => {
                    let r = &report.models[index];
                    vec![
                        r.model.clone(),
                        format!(
                            "Input {} · Output {} · Reasoning {}",
                            r.input_tokens, r.output_tokens, r.reasoning_output_tokens
                        ),
                        format!(
                            "Cache read {} · Cache creation {} · Total {} · Requests {}",
                            r.cache_read_tokens,
                            r.cache_creation_tokens,
                            r.total_tokens,
                            r.event_count
                        ),
                        format!(
                            "USD with cache {} · without cache {} · savings {}",
                            r.cost_with_cache_usd, r.cost_without_cache_usd, r.cache_savings_usd
                        ),
                        format!(
                            "Pricing status {} · source {} · rate {}",
                            r.pricing_status,
                            r.pricing_source.as_deref().unwrap_or("unknown"),
                            r.pricing_rate.as_deref().unwrap_or("unknown")
                        ),
                    ]
                }
                Section::Providers => {
                    let r = &report.providers[index];
                    vec![
                        r.display_provider().to_string(),
                        format!(
                            "Input {} · Output {} · Reasoning {}",
                            r.input_tokens, r.output_tokens, r.reasoning_output_tokens
                        ),
                        format!(
                            "Cache read {} · Cache creation {} · Total {} · Requests {}",
                            r.cache_read_tokens,
                            r.cache_creation_tokens,
                            r.total_tokens,
                            r.request_count
                        ),
                        format!(
                            "USD with cache {} · without cache {}",
                            r.cost_with_cache_usd, r.cost_without_cache_usd
                        ),
                    ]
                }
                Section::Trends => {
                    let r = &report.trends[index];
                    vec![
                        r.date.clone(),
                        format!(
                            "Input {} · Output including reasoning {} · Reasoning {}",
                            r.input_tokens, r.output_tokens, r.reasoning_output_tokens
                        ),
                        format!(
                            "Cache read {} · Cache creation {} · Total {} · Requests {} · USD {}",
                            r.cache_read_tokens,
                            r.cache_creation_tokens,
                            r.total_tokens,
                            r.request_count,
                            r.cost_usd
                        ),
                    ]
                }
                Section::Sources => {
                    let r = &report.sources[index];
                    vec![
                        r.source.clone(),
                        format!(
                            "Tokens {} · Requests {} · Active days {}",
                            r.total_tokens, r.event_count, r.active_days
                        ),
                        format!(
                            "USD {} · Token share {} · Cost share {}",
                            r.total_cost, r.share_tokens, r.share_cost
                        ),
                    ]
                }
                Section::Projects => {
                    let r = &report.projects[index];
                    vec![
                        format!("Project {} · {}", r.project_label, r.project_hash),
                        format!(
                            "Tokens {} · Requests {} · USD {}",
                            r.total_tokens, r.event_count, r.total_cost_usd
                        ),
                    ]
                }
                Section::Heatmap => report
                    .heatmap
                    .as_ref()
                    .and_then(|rows| rows.get(index))
                    .map(|r| vec![format!("{} · Requests {}", r.date, r.event_count)])
                    .unwrap_or_default(),
                Section::Logs => Vec::new(),
            },
            InsightsSnapshot::Logs(page) => {
                let r = &page.records[index];
                vec![
                    format!(
                        "{} · {} · {}",
                        r.recorded_at,
                        r.platform,
                        r.model.as_deref().unwrap_or("unknown")
                    ),
                    format!(
                        "Input {} · Output including reasoning {} · Cache read {} · Cache creation {}",
                        r.input_tokens,
                        r.output_tokens,
                        r.cache_read_tokens,
                        r.cache_creation_tokens
                    ),
                    format!(
                        "USD with cache {} · without cache {} · Pricing {} / {}",
                        r.cost_with_cache_usd,
                        r.cost_without_cache_usd,
                        r.pricing_status,
                        r.pricing_source.as_deref().unwrap_or("unknown")
                    ),
                ]
            }
            InsightsSnapshot::Detail(detail) => {
                let r = &detail.page.messages[index];
                vec![
                    format!(
                        "{} · {} · fidelity {} · clipped {}",
                        r.ordinal,
                        r.role,
                        detail.page.fidelity.as_str(),
                        r.clipped
                    ),
                    r.content.clone(),
                ]
            }
            InsightsSnapshot::Sessions { .. } => Vec::new(),
        }
    }
    pub(super) fn summary(&self) -> String {
        match self.snapshot.as_deref() {
            Some(InsightsSnapshot::Home(h)) => format!(
                "{} · Requests {} · Tokens {} · Sessions {} · Last 7 days: requests {}, sessions {} · Previous 7 days: requests {}, sessions {}",
                h.as_of,
                h.totals.requests,
                h.totals.tokens,
                if h.sessions_indexed {
                    h.totals.sessions.to_string()
                } else {
                    "not indexed".into()
                },
                h.last_7_days.requests,
                if h.sessions_indexed {
                    h.last_7_days.sessions.to_string()
                } else {
                    "not indexed".into()
                },
                h.previous_7_days.requests,
                if h.sessions_indexed {
                    h.previous_7_days.sessions.to_string()
                } else {
                    "not indexed".into()
                }
            ),
            Some(InsightsSnapshot::Usage(r)) => format!(
                "Requests {} · Tokens {} · USD {} · Last sync {} · {}",
                r.overview.total_events,
                r.overview.total.total_tokens,
                r.overview.total_cost_usd,
                r.overview.last_sync_at.as_deref().unwrap_or("not recorded"),
                if r.providers_supported {
                    "Provider dimension available"
                } else {
                    "Provider dimension unsupported, not measured zero"
                }
            ),
            Some(InsightsSnapshot::Logs(p)) => format!(
                "{} rows on this page · {} matching records",
                p.records.len(),
                p.total
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "not counted".into())
            ),
            Some(InsightsSnapshot::Sessions { page, .. }) => {
                format!("{} archived summaries on this page", page.items.len())
            }
            Some(InsightsSnapshot::Detail(d)) => format!(
                "{} · {} · {} · {} messages on this page",
                d.agent.as_str(),
                d.variant,
                d.page.fidelity.as_str(),
                d.page.messages.len()
            ),
            None => "This report has not been read.".into(),
        }
    }
    pub(super) fn comparison(&self, cx: &Context<Self>) -> gpui_kit::Div {
        let rows: Vec<(&str, i64)> = match self.snapshot.as_deref() {
            Some(InsightsSnapshot::Home(h)) => h
                .agents
                .iter()
                .take(4)
                .map(|r| (r.source.as_str(), r.requests))
                .collect(),
            Some(InsightsSnapshot::Usage(r)) => r
                .sources
                .iter()
                .take(4)
                .map(|r| (r.source.as_str(), r.total_tokens))
                .collect(),
            _ => Vec::new(),
        };
        let max = rows
            .iter()
            .map(|(_, value)| *value)
            .max()
            .unwrap_or(1)
            .max(1) as f32;
        let mut chart = div().flex().flex_col().gap(px(4.)).bg(cx.theme().secondary);
        if !rows.is_empty() {
            chart = chart
                .h(px(160.))
                .p(px(8.))
                .child(div().text_size(px(theme::META_SIZE)).child(text(
                    cx,
                    "Source comparison (first four); all exact values below",
                    "来源比较（前四项）；全部精确数值见下表",
                )));
        }
        for (label, value) in rows {
            chart = chart.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(16.))
                    .child(div().w(px(120.)).child(label.to_string()))
                    .child(
                        div().flex_1().child(
                            div()
                                .h(px(8.))
                                .w(gpui_kit::relative(
                                    (value.max(0) as f32 / max).clamp(0., 1.),
                                ))
                                .bg(cx.theme().primary),
                        ),
                    )
                    .child(div().w(px(100.)).text_right().child(value.to_string())),
            );
        }
        chart
    }
}
