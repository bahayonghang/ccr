//! Readable projections of the existing observer response. Raw values remain advanced.
use super::*;
use gpui_kit::component::tooltip::Tooltip;
use serde_json::Value;

fn numeric_column(key: &str) -> bool {
    !matches!(
        key,
        "date" | "key" | "tool_name" | "session_id" | "project_path"
    )
}

fn cell(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        _ => "—".into(),
    }
}
fn heading(label: &str) -> gpui_kit::Div {
    div()
        .text_size(crate::theme::text_size(theme::SECTION_TITLE_SIZE))
        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
        .child(label.to_string())
}
fn table(
    id: &'static str,
    label: &str,
    rows: Option<&Value>,
    columns: &[(&str, &str)],
    cx: &mut Context<ObserverView>,
) -> gpui_kit::AnyElement {
    let section = div()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP))
        .child(heading(label));
    let Some(rows) = rows.and_then(Value::as_array) else {
        return section
            .child(text(
                cx,
                "This projection is unavailable.",
                "此投影不可用。",
            ))
            .into_any_element();
    };
    if rows.is_empty() {
        return section
            .child(text(
                cx,
                "No stored entries in this period.",
                "此时间段没有已存条目。",
            ))
            .into_any_element();
    }
    let mut body = div()
        .id(id)
        .test_support()
        .role(gpui_kit::Role::Table)
        .aria_label(label.to_string())
        .max_h(px(theme::ROW_HEIGHT * 6.))
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .border_t_1()
        .border_color(cx.theme().border);
    let mut header = div()
        .id("header")
        .role(gpui_kit::Role::Row)
        .flex()
        .gap(px(theme::CONTROL_GAP))
        .py(px(theme::CONTROL_GAP));
    for (key, label) in columns {
        let mut cell = div()
            .id(SharedString::from(format!("header-{key}")))
            .role(gpui_kit::Role::ColumnHeader)
            .flex_1()
            .min_w_0()
            .text_size(crate::theme::text_size(theme::META_SIZE))
            .text_color(cx.theme().muted_foreground)
            .child((*label).to_string());
        if numeric_column(key) {
            cell = cell.text_right();
        }
        header = header.child(cell);
    }
    body = body.child(header);
    for (index, row) in rows.iter().take(366).enumerate() {
        let mut line = div()
            .id(SharedString::from(format!("row-{index}")))
            .role(gpui_kit::Role::Row)
            .flex_shrink_0()
            .flex()
            .gap(px(theme::CONTROL_GAP))
            .py(px(theme::CONTROL_GAP))
            .border_t_1()
            .border_color(cx.theme().border);
        for (key, _) in columns {
            let value = if *key == "cost_usd" && row.get(*key).is_none_or(Value::is_null) {
                text(cx, "Not recorded", "未记录").to_string()
            } else {
                cell(row.get(*key))
            };
            let mut cell = div()
                .id(SharedString::from(format!("cell-{key}")))
                .test_support()
                .role(gpui_kit::Role::Cell)
                .aria_label(value.clone())
                .flex_1()
                .min_w_0()
                .text_ellipsis()
                .text_size(crate::theme::text_size(theme::META_SIZE))
                .child(value.clone())
                .tooltip(move |window, cx| Tooltip::new(value.clone()).build(window, cx));
            if numeric_column(key) {
                cell = cell
                    .font_family(cx.theme().mono_font_family.clone())
                    .text_size(cx.theme().mono_font_size)
                    .text_right();
            }
            line = line.child(cell);
        }
        body = body.child(line);
    }
    section.child(body).into_any_element()
}
pub(super) fn report(view: &ObserverView, cx: &mut Context<ObserverView>) -> gpui_kit::AnyElement {
    let mut root = div()
        .id("observer-structured-report")
        .test_support()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(theme::SECTION_GAP));
    let Some(report) = &view.report else {
        return root
            .child(text(
                cx,
                "Read stored Claude data to show the report.",
                "读取已存 Claude 数据后显示报告。",
            ))
            .into_any_element();
    };
    root=root.child(div().text_size(crate::theme::text_size(theme::META_SIZE)).text_color(cx.theme().muted_foreground).child(text(cx,"Usage uses local calendar days and recorded USD costs. Tool calls come from CCR's main database; absent sources are shown separately.","用量按本地日历日和已记录美元费用统计。工具调用来自 CCR 主数据库；缺失来源分别显示。")));
    match &report.insight {
        Ok(value) => {
            let mut totals = div().flex().flex_wrap().gap(px(theme::SECTION_GAP));
            for (key, en, zh) in [
                ("today_value_usd", "Today USD", "今日美元"),
                ("month_value_usd", "This month USD", "本月美元"),
                ("total_value_usd", "All history USD", "全部历史美元"),
                ("total_sessions", "Observed sessions", "已观测会话"),
                ("total_projects", "Projects", "项目"),
                ("roi", "Monthly value / subscription", "本月费用 / 订阅费"),
            ] {
                let display = if key == "total_projects"
                    && value
                        .get("project_attribution_complete")
                        .and_then(Value::as_bool)
                        != Some(true)
                {
                    let count = value.get(key).and_then(Value::as_i64).unwrap_or(0);
                    if count == 0 {
                        text(cx, "Projects unattributed", "项目未归属").to_string()
                    } else {
                        format!(
                            "{count} · {}",
                            text(
                                cx,
                                "Attributed projects (partial coverage)",
                                "已归属项目（部分记录未归属）"
                            )
                        )
                    }
                } else {
                    cell(value.get(key))
                };
                totals = totals.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(theme::CONTROL_GAP))
                        .child(
                            div()
                                .text_size(crate::theme::text_size(theme::META_SIZE))
                                .child(text(cx, en, zh)),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!("observer-total-{key}")))
                                .test_support()
                                .aria_label(display.clone())
                                .font_family(cx.theme().mono_font_family.clone())
                                .text_size(cx.theme().mono_font_size)
                                .child(display),
                        ),
                );
            }
            root = root.child(totals);
        }
        Err(error) => root = root.child(error_text(*error, cx)),
    }
    match &report.usage {
        Ok(value) => {
            root = root.child(table(
                "observer-daily",
                text(cx, "Daily usage", "逐日用量"),
                value.get("daily"),
                &[
                    ("date", text(cx, "Date", "日期")),
                    ("cost_usd", "USD"),
                    ("input_tokens", text(cx, "Input tokens", "输入 Token")),
                    ("output_tokens", text(cx, "Output tokens", "输出 Token")),
                    ("cache_read_tokens", text(cx, "Cache read", "缓存读取")),
                ],
                cx,
            ));
            for (key, id, en, zh) in [
                ("models", "observer-models", "Models", "模型"),
                ("projects", "observer-projects", "Projects", "项目"),
            ] {
                root = root.child(table(
                    id,
                    text(cx, en, zh),
                    value.get(key),
                    &[
                        ("key", text(cx, "Name", "名称")),
                        ("cost_usd", "USD"),
                        ("tokens", "Tokens"),
                        ("count", text(cx, "Requests", "请求")),
                    ],
                    cx,
                ));
            }
        }
        Err(error) => {
            root = root.child(format!(
                "{}: {}",
                text(cx, "Usage", "用量"),
                error_text(*error, cx)
            ))
        }
    }
    match &report.tools {
        Ok(value) => {
            root = root.child(table(
                "observer-tools",
                text(cx, "Tool calls", "工具调用"),
                value.get("tools"),
                &[
                    ("tool_name", text(cx, "Tool", "工具")),
                    ("call_count", text(cx, "Calls", "调用")),
                    ("cost_usd", "USD"),
                ],
                cx,
            ));
            root = root.child(table(
                "observer-sessions",
                text(
                    cx,
                    "Observed sessions · last 30 days",
                    "已观测会话 · 最近 30 天",
                ),
                value.get("sessions"),
                &[
                    ("session_id", text(cx, "Session", "会话")),
                    ("project_path", text(cx, "Project", "项目")),
                    ("tool_call_count", text(cx, "Calls", "调用")),
                    ("cost_usd", "USD"),
                ],
                cx,
            ));
            root = root.child(table(
                "observer-tool-heatmap",
                match value.get("timezone").and_then(Value::as_str) {
                    Some("utc") => text(
                        cx,
                        "Calls by weekday and hour · UTC",
                        "星期与小时调用分布 · UTC",
                    ),
                    Some("local") => text(
                        cx,
                        "Calls by weekday and hour · local time",
                        "星期与小时调用分布 · 本地时间",
                    ),
                    _ => text(
                        cx,
                        "Calls by weekday and hour · time zone unavailable",
                        "星期与小时调用分布 · 时区不可用",
                    ),
                },
                value.get("heatmap"),
                &[
                    ("dow", text(cx, "Weekday (0 = Sunday)", "星期（0 为周日）")),
                    ("hour", text(cx, "Hour", "小时")),
                    ("count", text(cx, "Calls", "调用")),
                ],
                cx,
            ));
        }
        Err(error) => {
            root = root.child(format!(
                "{}: {}",
                text(cx, "Tool calls", "工具调用"),
                error_text(*error, cx)
            ))
        }
    }
    root.into_any_element()
}
