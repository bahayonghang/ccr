//! Favorites and Recent in the command list: segment capsule, record rows,
//! the editor's favorite toggle and the inline confirmation for removals.
//!
//! 记录由 UI 状态服务持久化，跨执行环境共享；最近由运行器在终态写入，视图只读取。
use super::{JobsView, model};
use crate::{
    locales::{is_chinese, text},
    runtime::{
        Services,
        ui_state::{
            AddFavoriteRequest, FavoriteRow, HistoryRow, UiStateError, UiStateRequests,
            UiStateSnapshot,
        },
    },
    shell::{badge, page_header, semantics},
    theme::{self, Tone},
};
use ccr_app::jobs::commands::{Error, Snapshot};
use ccr_app::runtime::{
    AdmissionError, Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent,
    TaskSubscription,
};
use ccr_core::core::log_redact::redact_log_text;
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, IconName,
    button::{Button, ButtonVariants},
    tooltip::Tooltip,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    AnyElement, App, Context, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, Role, ScrollStrategy, SharedString, StatefulInteractiveElement,
    Styled, Task, Window, div, px, uniform_list,
};
use std::sync::Arc;

/// Which list the command list shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Collection {
    #[default]
    Catalog,
    Favorites,
    Recent,
}

/// Argument words as editor text that splits back into the same words.
pub fn args_text(args: &[String]) -> String {
    args.iter()
        .map(|word| {
            if word.is_empty() || word.chars().any(char::is_whitespace) {
                format!("\"{word}\"")
            } else {
                word.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The favorite saved for exactly this command and these argument words.
pub fn find_favorite<'a>(
    favorites: &'a [FavoriteRow],
    command: &str,
    args: &[String],
) -> Option<&'a FavoriteRow> {
    favorites
        .iter()
        .find(|row| row.command == command && row.args == args)
}

/// `ccr <command> <args>` for a favorite, redacted for display.
pub fn favorite_line(row: &FavoriteRow) -> String {
    redact_log_text(&model::preview(&row.command, &args_text(&row.args)))
}

/// A favorite's display name, or its command when it has none.
pub fn favorite_name(row: &FavoriteRow) -> String {
    row.display_name
        .clone()
        .unwrap_or_else(|| row.command.clone())
}

/// Recent keeps the command name only; arguments are never stored at rest.
pub fn history_line(row: &HistoryRow) -> String {
    format!("ccr {}", row.command)
}

/// `MM-DD HH:mm · N ms · 成功/失败`, with the time shown in `zone`.
pub fn history_meta<Tz: chrono::TimeZone>(row: &HistoryRow, zone: &Tz, chinese: bool) -> String
where
    Tz::Offset: std::fmt::Display,
{
    let when = chrono::DateTime::parse_from_rfc3339(&row.executed_at)
        .map(|time| time.with_timezone(zone).format("%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| "–".to_string());
    let outcome = match (row.success, chinese) {
        (true, true) => "成功",
        (false, true) => "失败",
        (true, false) => "Succeeded",
        (false, false) => "Failed",
    };
    format!("{when} · {} ms · {outcome}", row.duration_ms)
}

/// Case-insensitive substring search over a record's visible texts.
pub fn matches(query: &str, texts: &[&str]) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || texts
            .iter()
            .any(|text| text.to_lowercase().contains(&query))
}

#[derive(Clone)]
pub(super) enum Confirmation {
    RemoveFavorite { id: String, name: String },
    ClearHistory,
}

/// One favorite or recent row as the list shows it.
#[derive(Clone)]
enum Record {
    Favorite(FavoriteRow),
    History(HistoryRow),
}
impl Record {
    fn key(&self) -> String {
        match self {
            Self::Favorite(row) => format!("favorite:{}", row.id),
            Self::History(row) => format!("history:{}", row.id),
        }
    }
    fn command(&self) -> &str {
        match self {
            Self::Favorite(row) => &row.command,
            Self::History(row) => &row.command,
        }
    }
    fn args_text(&self) -> String {
        match self {
            Self::Favorite(row) => args_text(&row.args),
            Self::History(_) => String::new(),
        }
    }
}

/// Persisted favorites and recent records plus their pending reads and writes.
pub(super) struct Records {
    requests: UiStateRequests,
    pub(super) snapshot: Arc<UiStateSnapshot>,
    generation: u64,
    gate: ResponseGate,
    read: Option<Task<()>>,
    mutation: Option<Task<()>>,
    /// Waits for the runner to persist the latest job's Recent record.
    finalized: Option<Task<()>>,
    pub(super) loaded: bool,
    pub(super) loading: bool,
    pub(super) saving: bool,
    pub(super) error: bool,
    pub(super) active: bool,
    pub(super) confirmation: Option<Confirmation>,
    confirmation_focus: FocusHandle,
    confirmation_return: Option<FocusHandle>,
    /// The list panel; focus stays here after a confirmed removal.
    pub(super) region_focus: FocusHandle,
    cursor: Option<String>,
}

impl Records {
    pub(super) fn new(services: &Services, cx: &mut App) -> Self {
        Self {
            requests: UiStateRequests::new(services),
            snapshot: Arc::new(UiStateSnapshot::default()),
            generation: 0,
            gate: ResponseGate::default(),
            read: None,
            mutation: None,
            finalized: None,
            loaded: false,
            loading: false,
            saving: false,
            error: false,
            active: false,
            confirmation: None,
            confirmation_focus: cx.focus_handle().tab_stop(false),
            confirmation_return: None,
            region_focus: cx.focus_handle().tab_stop(false),
            cursor: None,
        }
    }
    // UI state is admitted only for the application-local environment, even
    // when commands run on an SSH host: records are shared across environments.
    fn key(&mut self) -> RequestKey {
        self.generation += 1;
        RequestKey {
            environment: EnvironmentId::Local("application".into()),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        }
    }
    fn locked(&self) -> bool {
        self.loading || self.saving || self.error || self.confirmation.is_some()
    }
}

impl JobsView {
    pub(super) fn activate_records(&mut self, cx: &mut Context<Self>) {
        self.records.active = true;
        self.reload_records(cx);
    }

    pub(super) fn suspend_records(&mut self) {
        let records = &mut self.records;
        records.generation += 1;
        records.active = false;
        records.confirmation = None;
        records.confirmation_return = None;
        records.read = None;
        records.gate.clear();
        records.loading = false;
    }

    pub(super) fn reload_records(&mut self, cx: &mut Context<Self>) {
        self.records.read = None;
        let key = self.records.key();
        self.records.gate.replace(key.clone());
        self.records.loading = true;
        self.records.error = false;
        match self.records.requests.load(key.clone()) {
            Ok(mut read) => {
                self.records.read = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = read.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.records.gate.accepts(&key) {
                                    return;
                                }
                                this.records.loading = false;
                                match result.as_ref() {
                                    Completion::Success(snapshot) => {
                                        this.records.snapshot = snapshot.clone();
                                        this.records.loaded = true;
                                    }
                                    _ => this.records.error = true,
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => {
                self.records.loading = false;
                self.records.error = true;
            }
        }
        cx.notify();
    }

    fn observe_mutation(
        &mut self,
        result: Result<TaskSubscription<(), UiStateError>, AdmissionError>,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(mut job) => {
                self.records.saving = true;
                self.records.error = false;
                self.records.mutation = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = job.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                this.records.saving = false;
                                match result.as_ref() {
                                    Completion::Success(()) => {
                                        if this.records.active {
                                            this.reload_records(cx);
                                        }
                                    }
                                    _ => this.records.error = true,
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => self.records.error = true,
        }
        cx.notify();
    }

    /// The runner writes Recent before it publishes the job's completion, so
    /// rereading after completion shows the new record. The view never writes it.
    pub(super) fn watch_finalized(
        &mut self,
        subscription: Option<TaskSubscription<Snapshot, Error>>,
        cx: &mut Context<Self>,
    ) {
        let Some(mut subscription) = subscription else {
            return;
        };
        self.records.finalized = Some(cx.spawn(async move |weak, cx| {
            while let Some(event) = subscription.next().await {
                if matches!(event, TaskEvent::Terminal(_)) {
                    let _ = weak.update(cx, |this, cx| {
                        if this.records.active {
                            this.reload_records(cx);
                        }
                    });
                    break;
                }
            }
        }));
    }

    fn set_collection(&mut self, collection: Collection, cx: &mut Context<Self>) {
        self.collection = collection;
        self.records.confirmation = None;
        self.records.confirmation_return = None;
        if collection != Collection::Catalog && !self.records.active {
            self.activate_records(cx);
            return;
        }
        cx.notify();
    }

    /// The saved favorite matching the editor's command and argument words.
    fn editor_favorite(&self, cx: &App) -> Option<&FavoriteRow> {
        let words = model::tokenize(self.args.read(cx).value().as_str()).ok()?;
        find_favorite(&self.records.snapshot.favorites, &self.selected, &words)
    }

    fn toggle_favorite(&mut self, cx: &mut Context<Self>) {
        if !self.records.loaded || self.records.locked() {
            return;
        }
        let Ok(words) = model::tokenize(self.args.read(cx).value().as_str()) else {
            return;
        };
        let existing = self.editor_favorite(cx).map(|row| row.id.clone());
        let key = self.records.key();
        let result = match existing {
            Some(id) => self.records.requests.remove_favorite(key, id),
            None => self.records.requests.add_favorite(
                key,
                AddFavoriteRequest {
                    command: self.selected.clone(),
                    args: words,
                    display_name: self
                        .catalog
                        .get(&self.selected)
                        .map(|entry| entry.title.to_string()),
                    module: "commands".into(),
                },
            ),
        };
        self.observe_mutation(result, cx);
    }

    fn visible_records(&self, cx: &App) -> Vec<Record> {
        let query = self.search.read(cx).value();
        let snapshot = &self.records.snapshot;
        match self.collection {
            Collection::Catalog => Vec::new(),
            Collection::Favorites => snapshot
                .favorites
                .iter()
                .filter(|row| matches(&query, &[&favorite_name(row), &favorite_line(row)]))
                .cloned()
                .map(Record::Favorite)
                .collect(),
            Collection::Recent => snapshot
                .history
                .iter()
                .filter(|row| matches(&query, &[&history_line(row)]))
                .cloned()
                .map(Record::History)
                .collect(),
        }
    }

    fn move_record_cursor(&mut self, down: bool, cx: &mut Context<Self>) {
        let rows = self.visible_records(cx);
        let Some(last) = rows.len().checked_sub(1) else {
            return;
        };
        let current = rows
            .iter()
            .position(|row| self.records.cursor.as_deref() == Some(row.key().as_str()));
        let next = match current {
            Some(index) if down => (index + 1).min(last),
            Some(index) => index.saturating_sub(1),
            None if down => 0,
            None => last,
        };
        self.records.cursor = rows.get(next).map(Record::key);
        self.list_scroll.scroll_to_item(next, ScrollStrategy::Top);
        cx.notify();
    }

    /// Loads a record's command and arguments into the editor; the segment stays.
    /// Commands no longer in the catalog still load, and the editor says why
    /// they cannot run.
    fn load_record(&mut self, key: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let Some(record) = self
            .visible_records(cx)
            .into_iter()
            .find(|row| row.key() == key)
        else {
            return;
        };
        self.records.cursor = Some(key.to_string());
        let command = record.command().to_string();
        let args = record.args_text();
        if self.catalog.get(&command).is_some() {
            self.cursor = Some(command.clone());
        }
        self.selected = command;
        self.loaded_args = args.clone();
        self.args
            .update(cx, |state, cx| state.set_value(args, window, cx));
        cx.notify();
    }

    fn ask_confirmation(
        &mut self,
        action: Confirmation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.records.locked() {
            return;
        }
        self.records.confirmation_return = window.focused(cx);
        self.records.confirmation = Some(action);
        // The Cancel button renders next frame; start keyboard focus on it.
        let (weak, generation) = (cx.weak_entity(), self.records.generation);
        window.on_next_frame(move |window, cx| {
            let _ = weak.update(cx, |this, cx| {
                if this.records.confirmation.is_some() && this.records.generation == generation {
                    window.focus(&this.records.confirmation_focus, cx);
                    window.focus_next(cx);
                }
            });
        });
        cx.notify();
    }

    fn cancel_confirmation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.records.confirmation = None;
        if let Some(focus) = self.records.confirmation_return.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }

    fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.records.loading || self.records.saving || self.records.error {
            return;
        }
        let Some(action) = self.records.confirmation.take() else {
            return;
        };
        self.records.confirmation_return = None;
        let key = self.records.key();
        let result = match action {
            Confirmation::RemoveFavorite { id, .. } => {
                self.records.requests.remove_favorite(key, id)
            }
            Confirmation::ClearHistory => self.records.requests.clear_history(key),
        };
        self.observe_mutation(result, cx);
        // The trigger row may disappear; keep keyboard focus in the list.
        window.focus(&self.records.region_focus, cx);
    }

    /// The segment capsule, and Clear recent while the Recent segment shows.
    pub(super) fn collection_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let snapshot = self.records.snapshot.clone();
        let tabs = [
            (
                "command-collection-catalog",
                Collection::Catalog,
                text(cx, "Catalog", "目录"),
                self.catalog.counts().all,
            ),
            (
                "command-collection-favorites",
                Collection::Favorites,
                text(cx, "Favorites", "收藏"),
                snapshot.favorites.len(),
            ),
            (
                "command-collection-recent",
                Collection::Recent,
                text(cx, "Recent", "最近"),
                snapshot.history.len(),
            ),
        ];
        let mut capsule = page_header::capsule(cx);
        for (id, collection, label, count) in tabs {
            capsule = capsule.child(semantics::tab(
                semantics::button(
                    Button::new(id)
                        .label(format!("{label} {count}"))
                        .ghost()
                        .on_click(
                            cx.listener(move |this, _, _, cx| this.set_collection(collection, cx)),
                        ),
                    false,
                    false,
                    cx,
                ),
                self.collection == collection,
                false,
                cx,
            ));
        }
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .id("command-collections")
                    .test_support()
                    .role(Role::Group)
                    .aria_label(text(cx, "Command list", "命令列表"))
                    .child(capsule),
            )
            .when(self.collection == Collection::Recent, |row| {
                row.child(semantics::ghost_button(
                    Button::new("command-history-clear")
                        .label(text(cx, "Clear recent", "清空最近"))
                        .on_click(cx.listener(|this, _, window, cx| {
                            if this.records.snapshot.history.is_empty() {
                                return;
                            }
                            this.ask_confirmation(Confirmation::ClearHistory, window, cx);
                        })),
                    self.records.locked() || snapshot.history.is_empty(),
                    self.records.saving,
                    cx,
                ))
            })
            .into_any_element()
    }

    /// Favorites or Recent below the search field: notes, confirmation and rows.
    pub(super) fn records_panel(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut parts = Vec::new();
        if self.collection == Collection::Favorites {
            let note = text(
                cx,
                "Saved on this desktop and shared across environments. Do not save credentials in arguments.",
                "保存在此桌面并跨环境共享。不要在参数里保存凭据。",
            );
            parts.push(
                div()
                    .id("command-favorites-note")
                    .test_support()
                    .aria_label(note)
                    .text_size(theme::text_size(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(note)
                    .into_any_element(),
            );
        }
        if self.records.error {
            let line = text(
                cx,
                "Cannot access command records. Retry loading; a failed save is not replayed.",
                "无法访问命令记录。请重试读取；失败的保存不会自动重放。",
            );
            parts.push(
                div()
                    .id("command-records-error")
                    .test_support()
                    .role(Role::Status)
                    .aria_label(line)
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .text_color(cx.theme().danger)
                    .child(line)
                    .child(semantics::ghost_button(
                        Button::new("command-records-retry")
                            .label(text(cx, "Retry", "重试"))
                            .on_click(cx.listener(|this, _, _, cx| this.reload_records(cx))),
                        self.records.loading,
                        self.records.loading,
                        cx,
                    ))
                    .into_any_element(),
            );
        }
        if let Some(confirmation) = self.confirmation_block(cx) {
            parts.push(confirmation);
        }
        parts.push(self.records_body(cx));
        parts
    }

    fn confirmation_block(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let consequence = match self.records.confirmation.as_ref()? {
            Confirmation::RemoveFavorite { name, .. } => format!(
                "{}{name}{}",
                text(cx, "Remove favorite: ", "移除收藏："),
                text(
                    cx,
                    ". Its saved arguments will also be deleted. This cannot be undone.",
                    "。保存的参数也会删除，此操作无法撤销。",
                )
            ),
            Confirmation::ClearHistory => text(
                cx,
                "Clear all recent command records stored on this desktop, including records outside this list. This cannot be undone.",
                "清空保存在此桌面的全部最近命令记录，包括列表外的记录。此操作无法撤销。",
            )
            .to_string(),
        };
        Some(
            div()
                .id("command-confirmation")
                .test_support()
                .role(Role::Group)
                .aria_label(consequence.clone())
                .track_focus(&self.records.confirmation_focus)
                .tab_stop(false)
                .flex()
                .flex_col()
                .gap(px(theme::CONTROL_GAP))
                .p(px(theme::CONTROL_GAP))
                .rounded(px(theme::INPUT_RADIUS))
                .bg(cx.theme().secondary)
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" {
                        this.cancel_confirmation(window, cx);
                        cx.stop_propagation();
                    }
                }))
                .child(consequence)
                .child(
                    div()
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::ghost_button(
                            Button::new("command-confirm-cancel")
                                .label(text(cx, "Cancel", "取消"))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.cancel_confirmation(window, cx)
                                })),
                            false,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("command-confirm-apply")
                                .label(text(cx, "Confirm deletion", "确认删除"))
                                .primary()
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.confirm(window, cx)),
                                ),
                            false,
                            false,
                            cx,
                        )),
                )
                .into_any_element(),
        )
    }

    fn records_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let rows = self.visible_records(cx);
        if rows.is_empty() {
            return self.empty_records(cx);
        }
        let total = rows.len();
        let list = uniform_list(
            "command-records",
            total,
            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                let rows = this.visible_records(cx);
                let locked = this.records.locked();
                let saving = this.records.saving;
                range
                    .filter_map(|index| {
                        let row = rows.get(index)?.clone();
                        let highlighted =
                            this.records.cursor.as_deref() == Some(row.key().as_str());
                        Some(record_row(
                            index,
                            total,
                            row,
                            highlighted,
                            locked,
                            saving,
                            cx,
                        ))
                    })
                    .collect::<Vec<_>>()
            }),
        )
        .track_scroll(&self.list_scroll)
        .flex_1()
        .min_h_0();
        let label = match self.collection {
            Collection::Favorites => text(cx, "Favorite commands", "收藏命令"),
            _ => text(cx, "Recent commands", "最近命令"),
        };
        div()
            .id("command-record-listbox")
            .test_support()
            .track_focus(&self.list_focus)
            .tab_stop(true)
            .role(Role::ListBox)
            .aria_label(label)
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "down" => this.move_record_cursor(true, cx),
                    "up" => this.move_record_cursor(false, cx),
                    "enter" => {
                        if let Some(key) = this.records.cursor.clone() {
                            this.load_record(&key, window, cx);
                        }
                    }
                    _ => return,
                }
                cx.stop_propagation();
            }))
            .child(list)
            .into_any_element()
    }

    fn empty_records(&self, cx: &mut Context<Self>) -> AnyElement {
        let query = self.search.read(cx).value();
        if !query.trim().is_empty() {
            let clear = Button::new("command-clear-search")
                .label(text(cx, "Clear search", "清除搜索"))
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.search
                        .update(cx, |state, cx| state.set_value("", window, cx));
                    cx.notify();
                }))
                .into_any_element();
            return badge::empty_panel(
                "command-empty",
                IconName::Search,
                text(cx, "No matching records", "没有匹配的记录"),
                text(
                    cx,
                    "No record in this segment matches the search.",
                    "当前分段中没有记录匹配搜索。",
                ),
                Some(clear),
                cx,
            )
            .test_support()
            .into_any_element();
        }
        let (id, icon, title, body) = match self.collection {
            Collection::Favorites => (
                "command-favorites-empty",
                IconName::Star,
                text(cx, "No favorite commands yet", "还没有收藏命令"),
                text(
                    cx,
                    "Click Favorite in the editor to save the current command and arguments.",
                    "在编排区点“收藏”保存当前命令和参数。",
                ),
            ),
            _ => (
                "command-recent-empty",
                IconName::Inbox,
                text(cx, "No recent commands yet", "还没有最近命令"),
                text(
                    cx,
                    "Commands you run appear here.",
                    "运行过的命令会出现在这里。",
                ),
            ),
        };
        badge::empty_panel(id, icon, title, body, None, cx)
            .test_support()
            .into_any_element()
    }

    /// Favorite / Unfavorite for the editor's command and argument words.
    pub(super) fn favorite_toggle(&self, cx: &mut Context<Self>) -> Button {
        let draft = self.args.read(cx).value().to_string();
        let favorited = self.editor_favorite(cx).is_some();
        let label = if favorited {
            text(cx, "Unfavorite", "取消收藏")
        } else {
            text(cx, "Favorite", "收藏")
        };
        semantics::ghost_button(
            Button::new("command-favorite-toggle")
                .label(label)
                .icon(IconName::Star)
                .on_click(cx.listener(|this, _, _, cx| this.toggle_favorite(cx))),
            !self.records.loaded || self.records.locked() || model::tokenize(&draft).is_err(),
            self.records.saving,
            cx,
        )
    }
}

// 56px 两行：收藏行是显示名 / mono 完整命令 + 行尾「移除」；最近行是 mono 命令（失败加
// danger 标签）/ tnum 时间、耗时与结果。
fn record_row(
    index: usize,
    total: usize,
    record: Record,
    highlighted: bool,
    locked: bool,
    saving: bool,
    cx: &mut Context<JobsView>,
) -> AnyElement {
    let key = record.key();
    let chinese = is_chinese(cx);
    let (selected_bg, hover_bg, muted, mono) = {
        let colors = cx.theme();
        (
            colors.secondary,
            colors.muted,
            colors.muted_foreground,
            colors.mono_font_family.clone(),
        )
    };
    let (label, first, second, remove) = match &record {
        Record::Favorite(row) => {
            let name = favorite_name(row);
            let line = favorite_line(row);
            let first = div()
                .truncate()
                .text_size(theme::text_size(theme::BODY_SIZE))
                .font_weight(FontWeight::MEDIUM)
                .child(name.clone())
                .into_any_element();
            let tip = SharedString::from(line.clone());
            let second = div()
                .id(SharedString::from(format!("favorite-line:{}", row.id)))
                .truncate()
                .font_family(mono)
                .text_size(theme::text_size(theme::META_SIZE))
                .text_color(muted)
                .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
                .child(line.clone())
                .into_any_element();
            let id = row.id.clone();
            let remove = semantics::ghost_button(
                Button::new(SharedString::from(format!("remove-favorite:{id}")))
                    .label(text(cx, "Remove", "移除"))
                    .on_click(cx.listener({
                        let name = name.clone();
                        move |this, _, window, cx| {
                            this.ask_confirmation(
                                Confirmation::RemoveFavorite {
                                    id: id.clone(),
                                    name: name.clone(),
                                },
                                window,
                                cx,
                            );
                        }
                    })),
                locked,
                saving,
                cx,
            );
            (format!("{name} · {line}"), first, second, Some(remove))
        }
        Record::History(row) => {
            let line = history_line(row);
            let meta = history_meta(row, &chrono::Local, chinese);
            let mut first = div()
                .flex()
                .items_center()
                .gap(px(theme::CONTROL_GAP))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .font_family(mono)
                        .text_size(theme::text_size(theme::BODY_SIZE))
                        .font_weight(FontWeight::MEDIUM)
                        .child(line.clone()),
                );
            if !row.success {
                first = first.child(
                    badge::tag(
                        SharedString::from(format!("history-failed:{}", row.id)),
                        text(cx, "Failed", "失败"),
                        Tone::Danger,
                        cx,
                    )
                    .test_support(),
                );
            }
            let second = div()
                .truncate()
                .font_features(theme::tabular_figures())
                .text_size(theme::text_size(theme::META_SIZE))
                .text_color(muted)
                .child(meta.clone())
                .into_any_element();
            (
                format!("{line} · {meta}"),
                first.into_any_element(),
                second,
                None,
            )
        }
    };
    let weak = cx.weak_entity();
    let a11y_key = key.clone();
    let click_key = key.clone();
    div()
        .id(SharedString::from(key))
        .test_support()
        .role(Role::ListBoxOption)
        .aria_label(label)
        .aria_selected(highlighted)
        .aria_position_in_set(index + 1)
        .aria_size_of_set(total)
        .on_a11y_action(gpui_kit::AccessibleAction::Click, move |_, window, cx| {
            let _ = weak.update(cx, |this, cx| this.load_record(&a11y_key, window, cx));
        })
        .h(theme::data_row_height(theme::MCP_ROW_HEIGHT, cx))
        .w_full()
        .flex()
        .items_center()
        .gap(px(theme::CONTROL_GAP))
        .px(px(theme::CONTROL_GAP))
        .rounded(px(theme::SELECTION_RADIUS))
        .bg(if highlighted {
            selected_bg
        } else {
            gpui_kit::transparent_black()
        })
        .hover(move |style| style.bg(hover_bg))
        .child(
            // The load target excludes the row's own Remove button.
            div()
                .id(SharedString::from(format!("record-load:{click_key}")))
                .flex_1()
                .min_w_0()
                .h_full()
                .flex()
                .flex_col()
                .justify_center()
                .gap(px(2.))
                .cursor_pointer()
                .on_click(
                    cx.listener(move |this, _, window, cx| {
                        this.load_record(&click_key, window, cx)
                    }),
                )
                .child(first)
                .child(second),
        )
        .children(remove)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn favorite(command: &str, args: &[&str], display_name: Option<&str>) -> FavoriteRow {
        FavoriteRow {
            id: "fav-1".into(),
            command: command.into(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
            display_name: display_name.map(str::to_string),
            module: "commands".into(),
            created_at: "2026-11-25T00:00:00Z".into(),
        }
    }

    #[test]
    fn args_text_quotes_words_with_spaces_or_empty_values() {
        assert_eq!(
            args_text(&["-l".into(), "20".into(), "two words".into(), String::new(),]),
            r#"-l 20 "two words" """#
        );
        assert_eq!(args_text(&[]), "");
    }

    #[test]
    fn find_favorite_matches_exact_command_and_argument_words() {
        let rows = [
            favorite("history", &["-l", "20"], Some("History")),
            favorite("status", &[], None),
        ];
        assert_eq!(
            find_favorite(&rows, "history", &["-l".into(), "20".into()]).map(|r| r.id.as_str()),
            Some("fav-1")
        );
        assert!(find_favorite(&rows, "history", &["-l".into(), "10".into()]).is_none());
        assert!(find_favorite(&rows, "validate", &[]).is_none());
    }

    #[test]
    fn favorite_line_redacts_secrets_and_name_falls_back_to_command() {
        let row = favorite("status", &["--token", "sk-1234567890abcdef123456"], None);
        let shown = favorite_line(&row);
        assert!(!shown.contains("sk-1234567890abcdef123456"));
        assert_eq!(favorite_name(&row), "status");
        let named = favorite("status", &[], Some("Status overview"));
        assert_eq!(favorite_name(&named), "Status overview");
    }

    #[test]
    fn history_line_never_includes_arguments_and_meta_formats_outcome() {
        let row = HistoryRow {
            id: "hist-1".into(),
            command: "status".into(),
            success: false,
            duration_ms: 42,
            executed_at: "2026-11-25T08:05:00Z".into(),
        };
        assert_eq!(history_line(&row), "ccr status");
        assert_eq!(
            history_meta(&row, &chrono::Utc, false),
            "11-25 08:05 · 42 ms · Failed"
        );
        assert_eq!(
            history_meta(&row, &chrono::Utc, true),
            "11-25 08:05 · 42 ms · 失败"
        );
    }

    #[test]
    fn matches_checks_trimmed_case_insensitive_substrings() {
        assert!(matches("  HIST ", &["Show history", "ccr history -l 20"]));
        assert!(!matches("missing", &["Show history", "ccr history -l 20"]));
        assert!(matches("", &[]));
    }
}
