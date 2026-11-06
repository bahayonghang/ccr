//! Compact controls and lists for existing record/calendar/diagnostic owners.
use super::*;
use crate::runtime::checkin::{InspectionCommand, InspectionReport, OperationError, RecordQuery};
use chrono::Datelike;

pub(super) struct InspectionUi {
    keyword: Entity<InputState>,
    status: Option<&'static str>,
    year: Entity<InputState>,
    month: Entity<InputState>,
    days: Entity<InputState>,
}
impl InspectionUi {
    pub(super) fn new(window: &mut Window, cx: &mut Context<CheckinView>) -> Self {
        let now = chrono::Local::now();
        Self {
            keyword: cx.new(|cx| InputState::new(window, cx)),
            status: None,
            year: cx.new(|cx| {
                let mut state = InputState::new(window, cx);
                state.set_value(now.year().to_string(), window, cx);
                state
            }),
            month: cx.new(|cx| {
                let mut state = InputState::new(window, cx);
                state.set_value(now.month().to_string(), window, cx);
                state
            }),
            days: cx.new(|cx| {
                let mut state = InputState::new(window, cx);
                state.set_value("30", window, cx);
                state
            }),
        }
    }
}
impl CheckinView {
    fn inspect_account(&mut self, command: InspectionCommand, cx: &mut Context<Self>) {
        self.start(Operation::Extended(ExtendedCommand::Inspect(command)), cx);
    }
    fn invalid_query(&mut self, cx: &mut Context<Self>) {
        let mut state = (*self.operation).clone();
        state.result = Some(Err(OperationError::QueryInvalid));
        self.operation = Arc::new(state);
        cx.notify();
    }
    fn query_records(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = &self.extended else {
            return;
        };
        if editor.accounts.len() > 1 {
            self.invalid_query(cx);
            return;
        }
        let query = RecordQuery {
            account: editor.accounts.first().cloned(),
            provider: editor.provider.clone(),
            status: editor.inspection.status.map(str::to_string),
            keyword: editor.inspection.keyword.read(cx).value().to_string(),
            page: 1,
        };
        self.inspect_account(InspectionCommand::Records(query), cx);
    }
    pub(super) fn render_inspection(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let Some(editor) = &self.extended else {
            return div().into_any_element();
        };
        let blocked = self.operation.busy || editor.pending.is_some();
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .flex_shrink_0();
        if editor.accounts.len() == 1 {
            let account = editor.accounts.first().cloned().unwrap_or_default();
            let connection = account.clone();
            let waf = account.clone();
            root=root.child(div().flex().gap(px(theme::CONTROL_GAP))
                .child(semantics::button(Button::new("checkin-test-connection").label(text(cx,"Test account connection","测试账户连接")).on_click(cx.listener(move|this,_,_,cx|this.inspect_account(InspectionCommand::Connection(connection.clone()),cx))),blocked,false,cx))
                .child(semantics::button(Button::new("checkin-validate-waf").label(text(cx,"Validate cached WAF access","校验已缓存 WAF 访问")).on_click(cx.listener(move|this,_,_,cx|this.inspect_account(InspectionCommand::Waf(waf.clone()),cx))),blocked,false,cx)))
                .child(text(cx,"Connection and WAF validation contact the selected account's provider using stored credentials.","连接测试与 WAF 校验会使用已保存凭据访问所选账户的提供商。"))
                .child(text(cx,"Saved account calendar and trend","已保存账户月历与趋势"))
                .child(Input::new(&editor.inspection.year).id("checkin-calendar-year").aria_label(text(cx,"Calendar year","月历年份")).disabled(blocked))
                .child(Input::new(&editor.inspection.month).id("checkin-calendar-month").aria_label(text(cx,"Calendar month","月历月份")).disabled(blocked))
                .child(Input::new(&editor.inspection.days).id("checkin-trend-days").aria_label(text(cx,"Trend days","趋势天数")).disabled(blocked))
                .child(semantics::button(Button::new("checkin-dashboard-read").label(text(cx,"Read calendar and trend","读取月历与趋势")).on_click(cx.listener(move|this,_,_,cx|{
                    let Some(editor)=&this.extended else{return;};
                    let values=(editor.inspection.year.read(cx).value().parse::<i32>(),editor.inspection.month.read(cx).value().parse::<u32>(),editor.inspection.days.read(cx).value().parse::<u32>());
                    match values { (Ok(year),Ok(month),Ok(days))=>this.inspect_account(InspectionCommand::Dashboard{account:account.clone(),year,month,days},cx),_=>this.invalid_query(cx) }
                })),blocked,false,cx));
        }
        root=root.child(text(cx,"Record filters use the selected account and provider. Leave accounts unselected to include all; select at most one account.","记录筛选使用当前所选账户和提供商。不选账户时查询全部，筛选时最多选择一个账户。"))
            .child(semantics::button(Button::new("checkin-records-all-providers").label(text(cx,"All providers","全部提供商")).ghost().selected(editor.provider.is_none()).on_click(cx.listener(|this,_,_,cx|{if let Some(editor)=&mut this.extended{editor.provider=None;}cx.notify();})),blocked,false,cx))
            .child(Input::new(&editor.inspection.keyword).id("checkin-record-keyword").aria_label(text(cx,"Record keyword","记录关键词")).disabled(blocked));
        for (id, value, en, zh) in [
            ("all", None, "All statuses", "全部状态"),
            ("success", Some("success"), "Succeeded", "成功"),
            (
                "already",
                Some("already_checked_in"),
                "Already checked in",
                "已签到",
            ),
            ("failed", Some("failed"), "Failed", "失败"),
            ("skipped", Some("skipped"), "Skipped", "已跳过"),
        ] {
            root = root.child(semantics::button(
                Button::new(SharedString::from(format!("checkin-filter-{id}")))
                    .label(text(cx, en, zh))
                    .ghost()
                    .selected(editor.inspection.status == value)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(editor) = &mut this.extended {
                            editor.inspection.status = value;
                        }
                        cx.notify();
                    })),
                blocked,
                false,
                cx,
            ));
        }
        root = root.child(semantics::button(
            Button::new("checkin-filter-records")
                .label(text(cx, "Read filtered records", "读取筛选记录"))
                .on_click(cx.listener(|this, _, _, cx| this.query_records(cx))),
            blocked || editor.accounts.len() > 1,
            false,
            cx,
        ));
        root.into_any_element()
    }
    pub(super) fn render_inspection_report(
        &self,
        report: &InspectionReport,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let mut root = div()
            .id("checkin-inspection-result")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .flex_shrink_0();
        match report {
            InspectionReport::Connection { account, success } => {
                let label = format!(
                    "{} · {}",
                    account,
                    if *success {
                        text(cx, "Connection succeeded", "连接成功")
                    } else {
                        text(
                            cx,
                            "Provider response did not confirm access",
                            "提供商响应未确认访问成功",
                        )
                    }
                );
                root = root
                    .role(gpui_kit::Role::Status)
                    .aria_label(SharedString::from(label.clone()))
                    .child(label);
            }
            InspectionReport::Waf {
                account,
                success,
                status,
                challenge,
            } => {
                let label = format!(
                    "{} · {} · {} · {}",
                    account,
                    if *success {
                        text(cx, "WAF access validated", "WAF 访问校验成功")
                    } else {
                        text(cx, "WAF access was not validated", "未能确认 WAF 访问成功")
                    },
                    status
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| text(cx, "No HTTP response", "无 HTTP 响应").into()),
                    challenge
                );
                root = root
                    .role(gpui_kit::Role::Status)
                    .aria_label(SharedString::from(label.clone()))
                    .child(label);
            }
            InspectionReport::Records { query, rows, total } => {
                let label = format!(
                    "{}: {} · {}: {total}",
                    text(cx, "Page", "页"),
                    query.page,
                    text(cx, "Matching records", "匹配记录")
                );
                root = root
                    .role(gpui_kit::Role::Status)
                    .aria_label(SharedString::from(label.clone()))
                    .child(label);
                for row in rows {
                    root = root.child(format!(
                        "{} · {} · {}",
                        row.account_id,
                        row.at,
                        super::super::operations::status(cx, row.status)
                    ));
                }
                root = root.child(format!(
                    "{} · {} · {} · {}",
                    query.account.as_deref().unwrap_or_else(|| text(
                        cx,
                        "All accounts",
                        "全部账户"
                    )),
                    query.provider.as_deref().unwrap_or_else(|| text(
                        cx,
                        "All providers",
                        "全部提供商"
                    )),
                    query
                        .status
                        .as_deref()
                        .unwrap_or_else(|| text(cx, "All statuses", "全部状态")),
                    ccr_core::core::log_redact::redact_log_text(&query.keyword)
                ));
                let previous = query.clone();
                let next = query.clone();
                root = root.child(
                    div()
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new("checkin-records-prev")
                                .label(text(cx, "Previous page", "上一页"))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    let mut query = previous.clone();
                                    query.page = query.page.saturating_sub(1).max(1);
                                    this.inspect_account(InspectionCommand::Records(query), cx);
                                })),
                            self.operation.busy || query.page <= 1,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("checkin-records-next")
                                .label(text(cx, "Next page", "下一页"))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    let mut query = next.clone();
                                    query.page += 1;
                                    this.inspect_account(InspectionCommand::Records(query), cx);
                                })),
                            self.operation.busy || query.page.saturating_mul(20) >= *total,
                            false,
                            cx,
                        )),
                );
            }
            InspectionReport::Dashboard { year, month, data } => {
                root = root.child(ccr_core::core::log_redact::redact_log_text(
                    &data.account.name,
                ));
                let label = format!(
                    "{} · {year}-{month:02} · {}: {} · {}: {}",
                    text(cx, "Saved account calendar", "已保存账户月历"),
                    text(cx, "Checked days", "签到天数"),
                    data.calendar.month_stats.checked_in_days,
                    text(cx, "Current streak", "当前连续签到"),
                    data.streak.current_streak
                );
                root = root
                    .role(gpui_kit::Role::Status)
                    .aria_label(SharedString::from(label.clone()))
                    .child(label)
                    .child(text(
                        cx,
                        "Calendar and trend are computed from saved records, not live balances.",
                        "月历与趋势根据已保存记录计算，并非实时余额。",
                    ));
                for day in &data.calendar.days {
                    let reward = day
                        .reward_amount
                        .map(|value| format!(" · {value}"))
                        .unwrap_or_default();
                    root = root.child(format!(
                        "{} · {}{reward}",
                        day.date,
                        if day.is_checked_in {
                            text(cx, "Checked in", "已签到")
                        } else {
                            text(cx, "No check-in record", "无签到记录")
                        }
                    ));
                }
                root = root.child(text(cx, "Saved trend", "已保存趋势"));
                for point in &data.trend.data_points {
                    root = root.child(format!(
                        "{} · {}: {} · {}: {}",
                        point.date,
                        text(cx, "Balance", "余额"),
                        point.current_balance,
                        text(cx, "Income", "收入"),
                        point.income_increment
                    ));
                }
            }
        }
        root.into_any_element()
    }
}
