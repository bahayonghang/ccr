use super::*;
use crate::runtime::checkin::{HistoryRecord, RECORDS_PER_PAGE};
use crate::shell::badge;
use gpui_kit::component::ActiveTheme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Pick {
    Account,
    Provider,
}

impl CheckinView {
    pub(in crate::features::checkin) fn records_panel(
        &self,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let mut panel = div().flex().flex_col().gap(px(theme::CONTROL_GAP));
        if matches!(self.snapshot, Some(CheckinSnapshot::NoDatabase)) {
            return panel
                .child(note(
                    "checkin-records-empty",
                    text(
                        cx,
                        "No local database or check-in records.",
                        "尚无本机数据库和签到记录。",
                    ),
                ))
                .into_any_element();
        }
        let filter = &self.history.filter;
        let mut status = div().flex().flex_wrap().items_center().gap(px(4.));
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
            ("skipped", Some("skipped"), "Skipped", "跳过"),
        ] {
            status = status.child(badge::filter_chip(
                Button::new(format!("checkin-filter-{id}"))
                    .label(text(cx, en, zh))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.history.filter.status = value.map(str::to_owned);
                        this.history.filter.page = 1;
                        cx.notify();
                    })),
                filter.status.as_deref() == value,
                false,
                cx,
            ));
        }
        let keyword = text(
            cx,
            "Search account, provider or message",
            "搜索账户、提供商或消息",
        );
        let mut tools = div().flex().flex_wrap().items_center().gap(px(6.)).child(
            div().w(px(230.)).child(
                Input::new(&self.history.keyword)
                    .id("checkin-record-keyword")
                    .aria_label(keyword),
            ),
        );
        for (kind, id, en, zh) in [
            (Pick::Provider, "provider", "All providers", "全部提供商"),
            (Pick::Account, "account", "All accounts", "全部账户"),
        ] {
            let name = match kind {
                Pick::Account => filter
                    .account
                    .as_ref()
                    .and_then(|id| self.rows().iter().find(|r| &r.id == id).map(|r| &r.name)),
                Pick::Provider => self.data().and_then(|d| {
                    d.providers
                        .iter()
                        .find(|r| Some(&r.id) == filter.provider.as_ref())
                        .map(|r| &r.name)
                }),
            };
            tools = tools.child(semantics::ghost_button(
                Button::new(format!("checkin-record-choose-{id}"))
                    .label(name.cloned().unwrap_or_else(|| text(cx, en, zh).into()))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.history.picker = if this.history.picker == Some(kind) {
                            None
                        } else {
                            Some(kind)
                        };
                        cx.notify();
                    })),
                false,
                false,
                cx,
            ));
        }
        tools = tools.child(semantics::ghost_button(
            Button::new("checkin-record-clear")
                .label(text(cx, "Clear filters", "清除筛选"))
                .on_click(cx.listener(|this, _, w, cx| this.clear_record_filters(w, cx))),
            !filter.filtered(),
            false,
            cx,
        ));
        panel = panel.child(tools).child(status);
        if let (Some(kind), Some(data)) = (self.history.picker, self.data()) {
            let mut choices = div().flex().flex_wrap().gap(px(4.));
            let (all, items): (_, Vec<_>) = match kind {
                Pick::Account => (
                    text(cx, "All accounts", "全部账户"),
                    data.accounts.iter().map(|r| (&r.id, &r.name)).collect(),
                ),
                Pick::Provider => (
                    text(cx, "All providers", "全部提供商"),
                    data.providers.iter().map(|r| (&r.id, &r.name)).collect(),
                ),
            };
            for (index, (value, label)) in std::iter::once((None, all))
                .chain(
                    items
                        .into_iter()
                        .map(|(id, name)| (Some(id.clone()), name.as_str())),
                )
                .enumerate()
            {
                choices = choices.child(semantics::ghost_button(
                    Button::new(format!("checkin-record-option-{index}"))
                        .label(label.to_owned())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let filter = &mut this.history.filter;
                            match kind {
                                Pick::Account => filter.account = value.clone(),
                                Pick::Provider => filter.provider = value.clone(),
                            }
                            filter.page = 1;
                            this.history.picker = None;
                            cx.notify();
                        })),
                    false,
                    false,
                    cx,
                ));
            }
            panel = panel.child(choices);
        }
        let page = match self
            .history
            .records
            .as_ref()
            .filter(|(q, _)| q == filter)
            .map(|(_, v)| v)
        {
            Some(Ok(Some(v))) => v,
            Some(Ok(None)) => {
                return panel
                    .child(note(
                        "checkin-records-empty",
                        text(cx, "No local check-in database.", "尚无本机签到数据库。"),
                    ))
                    .into_any_element();
            }
            Some(Err(_)) => return panel.child(self.history_error(cx)).into_any_element(),
            None => {
                return panel
                    .child(text(cx, "Reading records…", "正在读取记录…"))
                    .into_any_element();
            }
        };
        let pending = page.enabled - page.checked - page.failed;
        let today = if is_chinese(cx) {
            format!(
                "今日 · 启用账户 {} · 已签到 {} · 待签到 {pending} · 失败 {}",
                page.enabled, page.checked, page.failed
            )
        } else {
            format!(
                "Today · {} enabled · {} checked in · {pending} pending · {} failed",
                page.enabled, page.checked, page.failed
            )
        };
        panel = panel.child(note("checkin-today-summary", today));
        if page.rows.is_empty() {
            let (id, label) = if filter.filtered() && page.has_records {
                (
                    "checkin-record-no-match",
                    text(cx, "No matching records.", "没有匹配的签到记录。"),
                )
            } else {
                (
                    "checkin-records-empty",
                    text(cx, "No saved check-in records.", "尚无签到记录。"),
                )
            };
            return panel.child(note(id, label)).into_any_element();
        }
        let last = page.total.div_ceil(RECORDS_PER_PAGE);
        let label = format!(
            "{}: {} · {} {}/{}",
            text(cx, "Matching records", "匹配记录"),
            page.total,
            text(cx, "Page", "页"),
            page.page,
            last
        );
        let mut pages = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(8.))
            .child(note("checkin-record-count", label));
        for (id, en, zh, target, disabled) in [
            (
                "prev",
                "Previous",
                "上一页",
                page.page.saturating_sub(1),
                page.page <= 1,
            ),
            ("next", "Next", "下一页", page.page + 1, page.page >= last),
        ] {
            pages = pages.child(semantics::ghost_button(
                Button::new(format!("checkin-records-{id}"))
                    .label(text(cx, en, zh))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.history.filter.page = target;
                        cx.notify();
                    })),
                disabled,
                false,
                cx,
            ));
        }
        panel = panel.child(pages);
        for row in &page.rows {
            panel = panel.child(self.record_row(row, cx));
        }
        panel.into_any_element()
    }

    fn record_row(&self, row: &HistoryRecord, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let title = format!(
            "{} · {} · {} · {}",
            row.account.as_deref().unwrap_or(&row.account_id),
            row.provider
                .as_deref()
                .unwrap_or(text(cx, "Unknown provider", "未知提供商")),
            operations::status(cx, row.status),
            row.at
        );
        let mut result = div()
            .flex()
            .flex_col()
            .gap(px(3.))
            .border_b_1()
            .border_color(cx.theme().border)
            .py(px(6.))
            .child(note(format!("checkin-record-row-{}", row.id), title));
        let mut fields = Vec::new();
        for (label, value) in [
            (text(cx, "Reward", "奖励"), row.reward.as_ref()),
            (text(cx, "Before", "签到前余额"), row.before.as_ref()),
            (text(cx, "After", "签到后余额"), row.after.as_ref()),
            (text(cx, "Message", "原因"), row.reason.as_ref()),
            (text(cx, "Code", "错误码"), row.error_code.as_ref()),
        ] {
            if let Some(v) = value {
                fields.push(format!("{label} {v}"));
            }
        }
        if !fields.is_empty() {
            result = result.child(note(
                format!("checkin-record-fields-{}", row.id),
                fields.join(" · "),
            ));
        }
        if row.account.is_some() {
            let mut actions = div().flex().flex_wrap().gap(px(6.));
            for repair in [false, true] {
                if repair && !row.needs_credentials() {
                    continue;
                }
                let account = row.account_id.clone();
                let kind = if repair { "repair" } else { "detail" };
                let key = format!("checkin-record-{kind}-{}", row.id);
                let origin = key.clone();
                actions = actions.child(
                    self.origin_control(
                        &key,
                        semantics::ghost_button(
                            Button::new(key.clone())
                                .label(if repair {
                                    text(cx, "Repair credentials", "修复凭据")
                                } else {
                                    text(cx, "Account details", "账户详情")
                                })
                                .on_click(cx.listener(move |this, _, w, cx| {
                                    if repair {
                                        this.open_account_credentials(&account, &origin, w, cx)
                                    } else {
                                        this.open_account_detail(&account, &origin, cx)
                                    }
                                })),
                            repair && self.operation.busy,
                            false,
                            cx,
                        ),
                    ),
                );
            }
            result = result.child(actions);
        }
        result
    }
}
