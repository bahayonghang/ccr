//! Native command workbench: catalog list, command editor and app-owned jobs.
//!
//! 分类、搜索、分词与可运行判定都来自 [`model`]；视图只消费结果。
pub mod model;
mod render;

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
    catalog: Catalog,
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
    list_scroll: UniformListScrollHandle,
    snapshot: Option<Snapshot>,
    job_id: Option<String>,
    task: Option<Task<()>>,
    generation: u64,
    busy: bool,
    message: Option<&'static str>,
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
        self.job_id = None;
        self.snapshot = None;
        self.message = None;
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
            requests,
            filter: Filter::default(),
            search,
            selected: "status".into(),
            cursor: Some("status".into()),
            args,
            loaded_args: String::new(),
            list_focus: cx.focus_handle(),
            list_scroll: UniformListScrollHandle::new(),
            snapshot: None,
            job_id: None,
            task: None,
            generation: 0,
            busy: false,
            message: None,
            log_scroll: UniformListScrollHandle::new(),
            _inputs: inputs,
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.list_focus, cx);
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        // App-owned runner keeps its terminal and bounded logs; view only detaches.
        self.task = None;
        self.generation += 1;
        self.busy = false;
        cx.notify();
    }
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.job_id.clone() else {
            return;
        };
        if self.task.is_some() {
            match self.requests.status(&id) {
                Ok(snapshot) => self.accept_snapshot(self.generation, snapshot),
                Err(error) => self.message = Some(error.message()),
            }
        } else {
            match self.requests.observe(&id) {
                Ok((snapshot, mut observation)) => {
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
                                    Ok(None) => break,
                                    Err(error) => {
                                        let _ = weak.update(cx, |this, cx| {
                                            if this.generation == generation {
                                                this.message = Some(error.message());
                                                this.busy = false;
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
                    self.message = Some(error.message());
                    self.busy = false;
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
        self.busy = !snapshot.status.is_terminal();
        self.snapshot = Some(snapshot);
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
        cx.notify();
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
                self.task = None;
                self.job_id = Some(started.snapshot.job_id.clone());
                self.snapshot = None;
                // Detaching the executor subscription keeps the app-owned runner.
                drop(started.subscription);
                self.refresh(cx);
            }
            Err(message) => self.message = Some(message),
        }
        cx.notify();
    }
    fn cancel(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.job_id {
            match self.requests.cancel(id) {
                Ok(snapshot) => {
                    self.busy = !snapshot.status.is_terminal();
                    self.snapshot = Some(snapshot);
                }
                Err(error) => self.message = Some(error.message()),
            }
        }
        cx.notify();
    }
    fn log_line(&self, index: usize) -> Option<String> {
        let snapshot = self.snapshot.as_ref()?;
        let (channel, line) = if index < snapshot.stdout_lines.len() {
            ("stdout", snapshot.stdout_lines.get(index)?)
        } else if index < snapshot.stdout_lines.len() + snapshot.stderr_lines.len() {
            (
                "stderr",
                snapshot
                    .stderr_lines
                    .get(index - snapshot.stdout_lines.len())?,
            )
        } else {
            (
                "system",
                snapshot
                    .system_lines
                    .get(index - snapshot.stdout_lines.len() - snapshot.stderr_lines.len())?,
            )
        };
        let mut end = line.len().min(8 * 1024);
        while !line.is_char_boundary(end) {
            end -= 1;
        }
        let suffix = if end < line.len() { "…" } else { "" };
        Some(format!("{channel}: {}{suffix}", &line[..end]))
    }
}

#[cfg(test)]
mod tests;
