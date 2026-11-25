//! Native command workbench: catalog list, command editor and app-owned jobs.
//!
//! 分类、搜索、分词与可运行判定都来自 [`model`]；视图只消费结果。
mod collections;
mod ledger;
pub mod model;
mod render;

use self::collections::{Collection, Records};
use self::model::{Blocked, Catalog, Filter};
use crate::{
    locales::text,
    runtime::{Services, jobs::CommandRequests},
};
#[cfg(test)]
use ccr_app::runtime::EnvironmentId;
use ccr_app::{
    jobs::commands::Snapshot,
    runtime::{Platform, RequestKey},
};
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, ScrollStrategy, Subscription, Task,
    UniformListScrollHandle, Window,
};

const NOT_FROM_CATALOG: &str =
    "That command cannot be started from this catalog. / 不能从当前目录启动这条命令";

pub struct InstallRequested;
impl gpui_kit::EventEmitter<InstallRequested> for JobsView {}

pub struct JobsView {
    requests: CommandRequests,
    environment_name: Option<String>,
    catalog: Catalog,
    collection: Collection,
    records: Records,
    filter: Filter,
    search: Entity<InputState>,
    /// The command loaded into the editor.
    selected: String,
    /// The highlighted catalog row; Enter loads it.
    cursor: Option<String>,
    args: Entity<InputState>,
    /// Argument text as it was when the command was loaded.
    loaded_args: String,
    list_focus: FocusHandle,
    work_focus: FocusHandle,
    list_scroll: UniformListScrollHandle,
    narrow_detail: bool,
    focus_generation: u64,
    snapshot: Option<Snapshot>,
    job_id: Option<String>,
    task: Option<Task<()>>,
    generation: u64,
    busy: bool,
    message: Option<&'static str>,
    observe_error: Option<&'static str>,
    log_scroll: UniformListScrollHandle,
    _inputs: [Subscription; 2],
}
impl JobsView {
    /// Only argument text that differs from what was loaded is a draft.
    pub fn has_environment_draft(&self, cx: &gpui_kit::App) -> bool {
        self.args.read(cx).value().as_str() != self.loaded_args
    }
    pub fn clear_environment(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.suspend(cx);
        self.environment_name = None;
        self.narrow_detail = false;
        self.job_id = None;
        self.snapshot = None;
        self.message = None;
        self.observe_error = None;
        self.reset_log_scroll();
        let loaded = self.loaded_args.clone();
        self.args
            .update(cx, |state, cx| state.set_value(loaded, window, cx));
    }
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let requests = CommandRequests::new(&services);
        let search = cx.new(|cx| {
            InputState::new(window, cx).placeholder(text(
                cx,
                "Search name or description",
                "搜索命令名或说明",
            ))
        });
        let args = cx.new(|cx| {
            InputState::new(window, cx).placeholder(text(
                cx,
                "Arguments separated by spaces",
                "参数，用空格分隔",
            ))
        });
        let inputs = [
            cx.subscribe(&search, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
            cx.subscribe(&args, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
        ];
        Self {
            catalog: Catalog::new(requests.catalog()),
            environment_name: None,
            collection: Collection::default(),
            records: Records::new(&services, cx),
            requests,
            filter: Filter::default(),
            search,
            selected: "status".into(),
            cursor: Some("status".into()),
            args,
            loaded_args: String::new(),
            list_focus: cx.focus_handle(),
            work_focus: cx.focus_handle().tab_stop(false),
            list_scroll: UniformListScrollHandle::new(),
            narrow_detail: false,
            focus_generation: 0,
            snapshot: None,
            job_id: None,
            task: None,
            generation: 0,
            busy: false,
            message: None,
            observe_error: None,
            log_scroll: UniformListScrollHandle::new(),
            _inputs: inputs,
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        if window.viewport_size().width < gpui_kit::px(crate::theme::SPLIT_WIDTH)
            && self.narrow_detail
        {
            self.focus_work(window, cx);
        } else {
            self.focus_list(window, cx);
        }
    }
    pub fn focus_main_end(&self, window: &mut Window, cx: &mut Context<Self>) {
        if window.viewport_size().width >= gpui_kit::px(crate::theme::SPLIT_WIDTH)
            || self.narrow_detail
        {
            self.focus_work(window, cx);
        } else {
            self.focus_list(window, cx);
        }
    }
    pub fn focus_list(&self, window: &mut Window, cx: &mut Context<Self>) {
        let has_rows = match self.collection {
            Collection::Catalog => !self.visible(cx).is_empty(),
            Collection::Favorites | Collection::Recent => !self.visible_records(cx).is_empty(),
        };
        if has_rows {
            window.focus(&self.list_focus, cx);
        } else {
            window.focus(&self.records.region_focus, cx);
            window.focus_next(cx);
        }
    }
    pub fn focus_work(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.work_focus, cx);
        window.focus_next(cx);
    }
    /// Handles F6 / Shift+F6 inside wide split layout (list <-> work column).
    /// Returns true when focus moved within the page; false lets the shell move
    /// to navigation or the status bar.
    pub fn focus_region(
        &self,
        backward: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if window.viewport_size().width < gpui_kit::px(crate::theme::SPLIT_WIDTH) {
            return false;
        }
        let in_list = self.records.region_focus.contains_focused(window, cx);
        let in_work = self.work_focus.contains_focused(window, cx);
        if !backward && in_list {
            self.focus_work(window, cx);
            true
        } else if backward && in_work {
            self.focus_list(window, cx);
            true
        } else {
            false
        }
    }
    pub(super) fn reveal_work(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if window.viewport_size().width >= gpui_kit::px(crate::theme::SPLIT_WIDTH) {
            return;
        }
        self.narrow_detail = true;
        self.focus_generation += 1;
        let (view, guard) = (cx.weak_entity(), self.focus_generation);
        window.on_next_frame(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                if this.focus_generation != guard || !this.narrow_detail {
                    return;
                }
                this.focus_work(window, cx);
            });
        });
        cx.notify();
    }
    pub(super) fn back_to_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.narrow_detail = false;
        let row = match self.collection {
            Collection::Catalog => self
                .cursor
                .as_deref()
                .and_then(|name| self.visible(cx).iter().position(|entry| *entry == name)),
            Collection::Favorites | Collection::Recent => {
                self.records.cursor.as_deref().and_then(|key| {
                    self.visible_records(cx)
                        .iter()
                        .position(|row| row.key() == key)
                })
            }
        };
        if let Some(index) = row {
            self.list_scroll.scroll_to_item(index, ScrollStrategy::Top);
        }
        self.focus_generation += 1;
        let (view, guard) = (cx.weak_entity(), self.focus_generation);
        window.on_next_frame(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                if this.focus_generation != guard || this.narrow_detail {
                    return;
                }
                this.focus_list(window, cx);
            });
        });
        cx.notify();
    }
    pub fn set_environment_name(&mut self, name: String) {
        self.environment_name = Some(name);
    }
    pub fn open(&mut self, cx: &mut Context<Self>) {
        self.activate_records(cx);
        self.refresh(cx);
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        // App-owned runner keeps its terminal and bounded logs; view only detaches.
        self.suspend_records();
        self.task = None;
        self.generation += 1;
        self.focus_generation += 1;
        self.busy = false;
        cx.notify();
    }
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.job_id.clone() else {
            return;
        };
        if self.task.is_some() {
            match self.requests.status(&id) {
                Ok(snapshot) => {
                    self.observe_error = None;
                    self.accept_snapshot(self.generation, snapshot);
                }
                Err(error) => {
                    self.observe_error = Some(error.message());
                    self.busy = false;
                    self.task = None;
                }
            }
        } else {
            match self.requests.observe(&id) {
                Ok((snapshot, mut observation)) => {
                    self.observe_error = None;
                    self.accept_snapshot(self.generation, snapshot);
                    if self.busy {
                        let generation = self.generation;
                        self.task = Some(cx.spawn(async move |weak, cx| {
                            loop {
                                // Broadcast deltas can arrive faster than the
                                // UI budget. Read the latest retained snapshot
                                // at most ten times a second, including on lag.
                                cx.background_executor()
                                    .timer(std::time::Duration::from_millis(100))
                                    .await;
                                match observation.next().await {
                                    Ok(Some(snapshot)) => {
                                        let terminal = snapshot.status.is_terminal();
                                        if weak
                                            .update(cx, |this, cx| {
                                                this.accept_snapshot(generation, snapshot);
                                                if terminal && this.generation == generation {
                                                    this.task = None;
                                                }
                                                cx.notify();
                                            })
                                            .is_err()
                                        {
                                            break;
                                        }
                                        if terminal {
                                            break;
                                        }
                                    }
                                    Ok(None) => {
                                        let _ = weak.update(cx, |this, _| {
                                            if this.generation == generation {
                                                this.task = None;
                                            }
                                        });
                                        break;
                                    }
                                    Err(error) => {
                                        let _ = weak.update(cx, |this, cx| {
                                            if this.generation == generation {
                                                this.observe_error = Some(error.message());
                                                this.busy = false;
                                                this.task = None;
                                                cx.notify();
                                            }
                                        });
                                        break;
                                    }
                                }
                            }
                        }));
                    }
                }
                Err(error) => {
                    self.observe_error = Some(error.message());
                    self.busy = false;
                    self.task = None;
                }
            }
        }
        cx.notify();
    }
    fn accept_snapshot(&mut self, generation: u64, snapshot: Snapshot) {
        if self.generation != generation || self.job_id.as_deref() != Some(&snapshot.job_id) {
            return;
        }
        if self
            .snapshot
            .as_ref()
            .is_some_and(|previous| previous.status.is_terminal())
        {
            return;
        }
        self.apply_snapshot(snapshot);
    }
    fn apply_snapshot(&mut self, snapshot: Snapshot) {
        let follow = self.snapshot.is_none() || self.log_pinned();
        let next_count = ledger::line_count(&snapshot);
        self.observe_error = None;
        self.busy = !snapshot.status.is_terminal();
        self.snapshot = Some(snapshot);
        if follow && next_count > 0 {
            self.log_scroll
                .scroll_to_item_strict(next_count - 1, ScrollStrategy::Bottom);
        }
    }
    fn log_pinned(&self) -> bool {
        let state = self.log_scroll.0.borrow();
        ledger::pinned(
            f32::from(state.base_handle.offset().y),
            f32::from(state.base_handle.max_offset().y),
        )
    }
    fn reset_log_scroll(&self) {
        self.log_scroll
            .0
            .borrow()
            .base_handle
            .set_offset(gpui_kit::point(gpui_kit::px(0.), gpui_kit::px(0.)));
    }

    /// Catalog rows for the current filter and search, in catalog order.
    fn visible(&self, cx: &App) -> Vec<&'static str> {
        let query = self.search.read(cx).value();
        self.catalog
            .visible(self.filter, &query)
            .into_iter()
            .map(|entry| entry.name)
            .collect()
    }
    fn set_filter(&mut self, filter: Filter, cx: &mut Context<Self>) {
        self.filter = filter;
        cx.notify();
    }
    fn clear_search_and_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.filter = Filter::All;
        self.search
            .update(cx, |state, cx| state.set_value("", window, cx));
        cx.notify();
    }
    fn move_cursor(&mut self, down: bool, cx: &mut Context<Self>) {
        let rows = self.visible(cx);
        let Some(last) = rows.len().checked_sub(1) else {
            return;
        };
        let current = rows
            .iter()
            .position(|name| self.cursor.as_deref() == Some(*name));
        let next = match current {
            Some(index) if down => (index + 1).min(last),
            Some(index) => index.saturating_sub(1),
            None if down => 0,
            None => last,
        };
        self.cursor = rows.get(next).map(|name| (*name).to_string());
        self.list_scroll.scroll_to_item(next, ScrollStrategy::Top);
        cx.notify();
    }
    /// Loads the highlighted catalog cursor only if it is currently visible.
    fn load_cursor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(name) = self.cursor.clone() else {
            return;
        };
        if self.visible(cx).iter().any(|entry| *entry == name) {
            self.load(&name, window, cx);
        }
    }
    /// Loads a catalog row into the editor. The loaded command stays fixed while
    /// a job runs; reloading the current command keeps its argument text.
    fn load(&mut self, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.cursor = Some(name.to_string());
        if self.selected != name {
            self.selected = name.to_string();
            self.loaded_args.clear();
            self.args
                .update(cx, |state, cx| state.set_value("", window, cx));
        }
        self.reveal_work(window, cx);
        cx.notify();
    }
    fn run_if_ready(&mut self, cx: &mut Context<Self>) {
        let draft = self.args.read(cx).value().to_string();
        if self
            .catalog
            .readiness(&self.selected, &draft, self.busy)
            .is_ok()
        {
            self.start(cx);
        }
    }
    fn next_key(&mut self) -> RequestKey {
        self.generation += 1;
        RequestKey {
            environment: self.requests.environment(),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        }
    }
    /// Runs the loaded command as a background job and starts observing it.
    fn start(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let draft = self.args.read(cx).value().to_string();
        let args = match self.catalog.readiness(&self.selected, &draft, self.busy) {
            Ok(args) => args,
            Err(Blocked::UnknownCommand | Blocked::NotRunnable(_)) => {
                self.message = Some(NOT_FROM_CATALOG);
                cx.notify();
                return;
            }
            // The editor already states the argument problem next to the input.
            Err(_) => return,
        };
        let key = self.next_key();
        let started = self.requests.start(key, self.selected.clone(), args);
        self.admit(started, cx);
    }
    /// Help is available for every catalog command, runnable or not.
    fn help(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        if self.catalog.get(&self.selected).is_none() {
            self.message = Some(NOT_FROM_CATALOG);
            cx.notify();
            return;
        }
        let key = self.next_key();
        let started = self.requests.help(key, &self.selected);
        self.admit(started, cx);
    }
    fn admit(
        &mut self,
        started: Result<crate::runtime::jobs::Started, &'static str>,
        cx: &mut Context<Self>,
    ) {
        match started {
            Ok(started) => {
                self.message = None;
                self.observe_error = None;
                self.task = None;
                self.job_id = Some(started.snapshot.job_id.clone());
                self.snapshot = None;
                self.reset_log_scroll();
                // Wait for the runner's terminal write so Recent refreshes automatically.
                self.watch_finalized(started.subscription, cx);
                self.refresh(cx);
            }
            Err(message) => self.message = Some(message),
        }
        cx.notify();
    }
    fn cancel(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.job_id {
            match self.requests.cancel(id) {
                Ok(snapshot) => self.apply_snapshot(snapshot),
                Err(error) => self.message = Some(error.message()),
            }
        }
        cx.notify();
    }
    fn copy_output(&self, cx: &mut Context<Self>) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return;
        };
        let copied = ledger::copy_text(snapshot);
        if !copied.is_empty() {
            cx.write_to_clipboard(gpui_kit::ClipboardItem::new_string(copied));
        }
    }
    fn clear_ledger(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.job_id = None;
        self.snapshot = None;
        self.observe_error = None;
        self.message = None;
        self.reset_log_scroll();
        cx.notify();
    }
    fn retry_observe(&mut self, cx: &mut Context<Self>) {
        self.observe_error = None;
        self.task = None;
        self.refresh(cx);
    }
}

#[cfg(test)]
mod tests;
