//! Native command catalog, app-owned execution and bounded log reader.
use crate::{
    locales::text,
    runtime::{Services, jobs::CommandRequests},
    shell::semantics,
};
use ccr_app::{
    jobs::{commands::Snapshot, policy::CommandInfo},
    runtime::{EnvironmentId, Platform, RequestKey},
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    AppContext, Context, Entity, Focusable, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Task, UniformListScrollHandle, Window, div,
    px, uniform_list,
};

enum CommandAction {
    Background,
    Foreground,
    Help,
}

pub struct JobsView {
    requests: CommandRequests,
    catalog: Vec<CommandInfo>,
    selected: String,
    args: Entity<InputState>,
    snapshot: Option<Snapshot>,
    job_id: Option<String>,
    task: Option<Task<()>>,
    generation: u64,
    busy: bool,
    message: Option<&'static str>,
    log_scroll: UniformListScrollHandle,
}
impl JobsView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let requests = CommandRequests::new(&services);
        Self {
            catalog: requests.catalog(),
            requests,
            selected: "status".into(),
            args: cx.new(|cx| {
                let mut state = InputState::new(window, cx);
                state.set_value("[\"--json\"]", window, cx);
                state
            }),
            snapshot: None,
            job_id: None,
            task: None,
            generation: 0,
            busy: false,
            message: None,
            log_scroll: UniformListScrollHandle::new(),
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.args.read(cx).focus_handle(cx), cx);
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
    fn start(&mut self, cx: &mut Context<Self>) {
        self.start_action(CommandAction::Background, cx);
    }
    fn start_action(&mut self, action: CommandAction, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let args = if matches!(action, CommandAction::Help) {
            Vec::new()
        } else {
            let Ok(args) = serde_json::from_str::<Vec<String>>(&self.args.read(cx).value()) else {
                self.message =
                    Some("Arguments must be a JSON string array. / 参数必须为 JSON 字符串数组");
                cx.notify();
                return;
            };
            args
        };
        self.generation += 1;
        let key = RequestKey {
            environment: EnvironmentId::Local(
                if self.requests.is_fixture() {
                    "fixture"
                } else {
                    "local"
                }
                .into(),
            ),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        let started = match action {
            CommandAction::Background => self.requests.start(key, self.selected.clone(), args),
            CommandAction::Foreground => self.requests.foreground(key, self.selected.clone(), args),
            CommandAction::Help => self.requests.help(key, &self.selected),
        };
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
        Some(format!(
            "{channel}: {}{}",
            &line[..end],
            if end < line.len() {
                "… / clipped"
            } else {
                ""
            }
        ))
    }
}
impl Render for JobsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut commands = div().flex().flex_wrap().gap_1();
        for entry in &self.catalog {
            let command = entry.name;
            let enabled = entry.executable && !self.busy;
            commands = commands.child(semantics::button(
                Button::new(SharedString::from(format!("job-command-{command}")))
                    .label(command)
                    .ghost()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if !this.busy {
                            this.selected = command.into();
                            cx.notify();
                        }
                    })),
                !enabled,
                false,
                cx,
            ));
        }
        let selected = self
            .catalog
            .iter()
            .find(|entry| entry.name == self.selected);
        let metadata = selected
            .map(|entry| format!("{} · {} · {}", entry.title, entry.risk, entry.usage))
            .unwrap_or_default();
        let state = self
            .snapshot
            .as_ref()
            .map(|snapshot| {
                format!(
                    "{:?}{} · exit {:?} · dropped {} · truncated {}",
                    snapshot.status,
                    if snapshot.cancel_requested {
                        " / cancel_requested"
                    } else {
                        ""
                    },
                    snapshot.exit_code,
                    snapshot.dropped_lines,
                    snapshot.truncated
                )
            })
            .unwrap_or_else(|| text(cx, "No job yet", "尚无任务").to_string());
        let count = self
            .snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot.stdout_lines.len()
                    + snapshot.stderr_lines.len()
                    + snapshot.system_lines.len()
            })
            .unwrap_or(0);
        let logs = uniform_list(
            "command-job-logs",
            count,
            cx.processor(|this, range: std::ops::Range<usize>, _, _| {
                range
                    .filter_map(|index| {
                        this.log_line(index).map(|line| {
                            div()
                                .id(SharedString::from(format!(
                                    "command-log-{}-{index}",
                                    this.job_id.as_deref().unwrap_or("none")
                                )))
                                .test_support()
                                .h(px(36.))
                                .px_2()
                                .child(line)
                        })
                    })
                    .collect::<Vec<_>>()
            }),
        )
        .h_full()
        .track_scroll(&self.log_scroll);
        div().id("native-command-workbench").test_support().flex().flex_col().size_full().p_4().gap_2().bg(cx.theme().background).text_color(cx.theme().foreground)
            .child(div().text_size(px(20.)).child(text(cx,"Commands and jobs","命令与任务")))
            .when(self.requests.is_fixture(), |view| {
                view.child(
                    div()
                        .id("jobs-fixture-warning")
                        .test_support()
                        .aria_label("Synthetic owned-process fixture. No CCR command or real configuration is accessed.")
                        .text_color(cx.theme().warning)
                        .child(text(
                            cx,
                            "Synthetic owned-process fixture. No CCR command or real configuration is accessed.",
                            "合成进程生命周期验证，不执行真实 CCR 命令",
                        )),
                )
            })
            .child(commands).child(metadata)
            .child(div().child(text(cx,"Arguments as JSON string array","参数 JSON 字符串数组")))
            .child(Input::new(&self.args))
            .child(div().flex().gap_2()
                .child(semantics::button(Button::new("command-start").label(text(cx,"Start background job","启动后台任务")).on_click(cx.listener(|this,_,_,cx|this.start(cx))), self.busy, self.busy, cx))
                .child(semantics::button(Button::new("command-foreground").label(text(cx,"Run","执行")).on_click(cx.listener(|this,_,_,cx|this.start_action(CommandAction::Foreground,cx))), self.busy, self.busy, cx))
                .child(semantics::button(Button::new("command-help").label(text(cx,"Command help","命令帮助")).ghost().on_click(cx.listener(|this,_,_,cx|this.start_action(CommandAction::Help,cx))), self.busy, false, cx))
                .child(semantics::button(Button::new("command-cancel").label(text(cx,"Request cancel","请求取消")).ghost().on_click(cx.listener(|this,_,_,cx|this.cancel(cx))), !self.busy, false, cx))
                .child(semantics::button(Button::new("command-status").label(text(cx,"Refresh status","刷新状态")).ghost().on_click(cx.listener(|this,_,_,cx|this.refresh(cx))), self.job_id.is_none(), false, cx)))
            .child(div().child(state))
            .when_some(self.message, |view, message| view.child(div().text_color(cx.theme().danger).child(crate::locales::localized_line(cx, message))))
            .child(div().flex_1().min_h_0().bg(cx.theme().background).child(logs))
    }
}

#[cfg(test)]
mod tests;
