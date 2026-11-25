//! Read results belong to the query that requested them. Returning from an
//! account editor or detail restores the record page and its invoking button.
use super::*;
use crate::runtime::checkin::{
    AccountHistory, HistoryQuery, HistoryResult, RecordPage, RecordQuery,
};
use chrono::{Datelike, Local, NaiveDate};
use gpui_kit::component::input::Input;

mod calendar;
mod records;

pub(super) struct HistoryUi {
    filter: RecordQuery,
    picker: Option<records::Pick>,
    keyword: Entity<InputState>,
    _changed: Subscription,
    pub(super) month: NaiveDate,
    trend_days: u32,
    range_editor: bool,
    range_invalid: bool,
    month_field: Entity<InputState>,
    days_field: Entity<InputState>,
    selected_day: Option<NaiveDate>,
    show_values: bool,
    pub(super) show_samples: bool,
    generation: u64,
    active: Option<HistoryQuery>,
    records: Option<(RecordQuery, Result<Option<RecordPage>, CheckinError>)>,
    detail: Option<(HistoryQuery, Result<Option<AccountHistory>, CheckinError>)>,
    task: Option<Task<()>>,
}

fn record_search_hint(cx: &App) -> &'static str {
    text(
        cx,
        "Search account, provider or message",
        "搜索账户、提供商或消息",
    )
}

impl HistoryUi {
    pub(super) fn new(window: &mut Window, cx: &mut Context<CheckinView>) -> Self {
        let hint = record_search_hint(cx);
        let keyword = cx.new(|cx| InputState::new(window, cx).placeholder(hint));
        let changed = cx.subscribe(&keyword, |this, input, event, cx| {
            if matches!(event, InputEvent::Change) {
                let value = input.read(cx).value().trim().to_owned();
                if this.history.filter.keyword != value {
                    this.history.filter.keyword = value;
                    this.history.filter.page = 1;
                    cx.notify();
                }
            }
        });
        let today = Local::now().date_naive();
        Self {
            picker: None,
            filter: RecordQuery::default(),
            keyword,
            _changed: changed,
            month: today.with_day(1).unwrap_or(today),
            trend_days: 0,
            range_editor: false,
            range_invalid: false,
            month_field: cx.new(|cx| InputState::new(window, cx)),
            days_field: cx.new(|cx| InputState::new(window, cx)),
            selected_day: None,
            show_values: false,
            show_samples: false,
            generation: 0,
            active: None,
            records: None,
            detail: None,
            task: None,
        }
    }

    pub(super) fn update_language(&mut self, window: &mut Window, cx: &mut Context<CheckinView>) {
        let hint = record_search_hint(cx);
        self.keyword
            .update(cx, |input, cx| input.set_placeholder(hint, window, cx));
    }

    pub(super) fn invalidate(&mut self) {
        self.records = None;
        self.detail = None;
        self.active = None;
        self.task = None;
    }
}

impl CheckinView {
    pub(super) fn open_records(
        &mut self,
        account: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.switch_tab(CheckinTab::Records, window, cx);
        self.clear_record_filters(window, cx);
        self.history.filter.account = account;
    }

    fn clear_record_filters(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.history.picker = None;
        self.history.filter = RecordQuery::default();
        self.history
            .keyword
            .update(cx, |s, cx| s.set_value("", window, cx));
        cx.notify();
    }

    pub(super) fn read_visible_history(&mut self, cx: &mut Context<Self>) {
        let state = &mut self.history;
        let desired = if self.editor.is_some()
            || !matches!(self.snapshot, Some(CheckinSnapshot::Accounts(_)))
        {
            None
        } else if let Some(account) = &self.selected {
            Some(HistoryQuery::Account {
                account: account.clone(),
                month: state.month,
                trend_days: state.trend_days,
            })
        } else if self.tab == CheckinTab::Records {
            Some(HistoryQuery::Records(state.filter.clone()))
        } else {
            None
        };
        if state.active == desired {
            return;
        }
        state.task = None;
        state.active = desired.clone();
        let Some(query) = desired else { return };
        if match &query {
            HistoryQuery::Records(q) => state.records.as_ref().is_some_and(|(k, _)| k == q),
            HistoryQuery::Account { .. } => state.detail.as_ref().is_some_and(|(k, _)| k == &query),
        } {
            return;
        }
        self.generation += 1;
        let generation = self.generation;
        state.generation = generation;
        let key = RequestKey {
            environment: EnvironmentId::Local("checkin-history".into()),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        match self.services.checkin_history_request(key, query.clone()) {
            Ok(mut sub) => {
                state.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = sub.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if this.history.generation == generation
                                    && this.history.active.as_ref() == Some(&query)
                                {
                                    this.finish_history(
                                        query,
                                        match result.as_ref() {
                                            Completion::Success(v) => Ok(v.clone()),
                                            Completion::Failed(e)
                                            | Completion::CleanupFailed(e) => Err(*e),
                                            _ => Err(CheckinError::Storage),
                                        },
                                    );
                                    cx.notify();
                                }
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => self.finish_history(query, Err(CheckinError::Storage)),
        }
    }

    fn finish_history(&mut self, query: HistoryQuery, result: Result<HistoryResult, CheckinError>) {
        match query {
            HistoryQuery::Records(q) => {
                self.history.records = Some((
                    q,
                    result.and_then(|v| match v {
                        HistoryResult::Records(page) => Ok(page),
                        _ => Err(CheckinError::Storage),
                    }),
                ));
            }
            HistoryQuery::Account { .. } => {
                self.history.detail = Some((
                    query,
                    result.and_then(|v| match v {
                        HistoryResult::Account(data) => Ok(data),
                        _ => Err(CheckinError::Storage),
                    }),
                ));
            }
        }
    }
}

fn note(id: impl Into<SharedString>, value: impl Into<SharedString>) -> impl IntoElement {
    let value = value.into();
    div()
        .id(id.into())
        .test_support()
        .aria_label(value.clone())
        .child(value)
}
