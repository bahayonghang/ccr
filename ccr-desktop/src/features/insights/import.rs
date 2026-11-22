use super::*;
use crate::runtime::insights::import::{Failure, Options, Snapshot, Status, ToolUpdate};
use ccr_app::runtime::EnvironmentId;

#[derive(Default)]
pub(super) struct Scope {
    options: Options,
    choose_source: bool,
    ssh_name: Option<(EnvironmentId, String)>,
}

pub(super) fn source_label(source: Option<SourceKind>, cx: &gpui_kit::App) -> &'static str {
    match source {
        None => text(cx, "All sources", "全部来源"),
        Some(SourceKind::Claude) => "Claude Code",
        Some(SourceKind::Codex) => "Codex",
        Some(SourceKind::Opencode) => "OpenCode",
        Some(SourceKind::Antigravity) => "Antigravity",
        Some(SourceKind::KimiCode) => "Kimi Code",
        Some(SourceKind::Pi) => "Pi",
        Some(SourceKind::Grok) => "Grok",
        Some(SourceKind::Zcode) => "ZCode",
        Some(SourceKind::DeepseekHarness) => "DeepSeek Harness",
    }
}

fn failure(error: Failure, cx: &gpui_kit::App) -> &'static str {
    match error {
        Failure::LocalOnly => text(
            cx,
            "This execution environment does not support usage import.",
            "当前执行环境不支持用量导入。",
        ),
        Failure::SshUnavailable => text(
            cx,
            "Usage import is unavailable. Connect the selected SSH host to continue.",
            "用量导入不可用，请先连接所选 SSH 主机。",
        ),
        Failure::Unconfigured => text(
            cx,
            "Usage import paths are unavailable. Check the configured usage and provider roots.",
            "用量导入路径不可用，请检查用量和会话来源目录。",
        ),
        Failure::FixtureDisabled => text(
            cx,
            "Usage import is disabled for fixture launches.",
            "夹具启动不允许导入本机用量。",
        ),
        Failure::SourceRoot => text(
            cx,
            "The installed importer cannot scan a custom provider home. Use the default local provider root.",
            "已安装导入器不支持自定义会话根目录，请使用默认本地来源目录。",
        ),
        Failure::Busy => text(
            cx,
            "A usage import is already running.",
            "已有用量导入正在运行。",
        ),
        Failure::MissingCli => text(
            cx,
            "llmusage is missing in this environment. Open Usage → Install llmusage.",
            "当前执行环境缺少 llmusage。请打开「用量」→「安装 llmusage」。",
        ),
        Failure::TimedOut => text(
            cx,
            "Usage import timed out after process cleanup.",
            "用量导入超时，进程清理已结束。",
        ),
        Failure::CleanupFailed => text(
            cx,
            "Import process cleanup failed. Check running processes before retrying.",
            "导入进程清理失败，请检查运行中的进程后重试。",
        ),
        Failure::Interrupted => text(
            cx,
            "Usage import stopped before a result. Retry when the application is ready.",
            "用量导入未产生结果，应用就绪后请重试。",
        ),
        Failure::Process => text(
            cx,
            "Usage import failed. Check the connection, llmusage and file access in the current execution environment. SSH snapshots also require Python3 with sqlite3.",
            "用量导入失败，请检查连接、当前执行环境中的 llmusage 和文件权限；SSH 快照还需要带 sqlite3 的 Python3。",
        ),
    }
}

impl InsightsView {
    pub fn has_retained_import(&self) -> bool {
        let snapshot = self.requests.import_snapshot();
        snapshot.run > 0 && snapshot.environment.as_ref() == Some(&self.requests.environment())
    }
    pub fn observe_retained_import(&mut self, cx: &mut Context<Self>) {
        if self.import_task.is_none() && self.has_retained_import() {
            self.observe_import(cx);
        }
    }
    pub(super) fn observe_import(&mut self, cx: &mut Context<Self>) {
        let mut updates = self.requests.observe_import();
        self.import_task = Some(cx.spawn(async move |weak, cx| {
            while updates.changed().await.is_ok() {
                // Consume the newest retained state at the shared UI budget;
                // hidden pages drop this subscription rather than poll.
                cx.background_executor()
                    .timer(ccr_app::runtime::PROGRESS_INTERVAL)
                    .await;
                let snapshot = updates.borrow_and_update().clone();
                if weak
                    .update(cx, |this, cx| {
                        if this.page == InsightsPage::Usage
                            && !this.requests.ssh_offline()
                            && snapshot.environment.as_ref() == Some(&this.requests.environment())
                            && matches!(
                                snapshot.status,
                                Status::Finished | Status::Cancelled | Status::Failed(_)
                            )
                        {
                            this.refresh(cx);
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        }));
    }
    fn effective_import_options(&self) -> Options {
        let mut options = self.import_scope.options;
        if let Some(platform) = self.platform_scope {
            options.source = crate::features::workbench::source(platform);
        }
        options
    }
    pub(crate) fn set_import_environment_name(&mut self, environment: EnvironmentId, name: String) {
        self.import_scope.ssh_name = Some((environment, name));
    }

    pub(super) fn close_import_source_menu(&mut self) {
        self.import_scope.choose_source = false;
    }

    fn import_environment_name(&self, environment: &EnvironmentId, cx: &gpui_kit::App) -> String {
        match environment {
            EnvironmentId::Local(id) => {
                let name = if self.requests.import_is_fixture() {
                    text(cx, "Fixture", "演示环境")
                } else {
                    text(cx, "Local", "本地")
                };
                if id == "local" || id == "fixture" {
                    name.into()
                } else {
                    format!("{name} · {id}")
                }
            }
            EnvironmentId::Ssh(id) => {
                let name = self
                    .import_scope
                    .ssh_name
                    .as_ref()
                    .filter(|(captured, _)| captured == environment)
                    .map_or(id.as_str(), |(_, name)| name);
                format!("SSH · {name}")
            }
            EnvironmentId::Wsl(id) => format!("WSL · {id}"),
        }
    }

    fn confirm_import(
        &mut self,
        environment: EnvironmentId,
        options: Options,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.requests.environment() != environment || self.effective_import_options() != options
        {
            return;
        }
        self.import_error = self
            .requests
            .start_import_with_options(
                RequestKey {
                    environment,
                    platform: self.platform_scope.unwrap_or(Platform::Claude),
                    generation: self.generation,
                    request_id: self.generation,
                },
                options,
            )
            .err();
        if self.import_error.is_none() {
            self.usage_controls.dismiss_panel(window, cx);
            self.close_import_source_menu();
        }
        cx.notify();
    }

    pub(super) fn import_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let environment = self.requests.environment();
        let options = self.effective_import_options();
        let active = self.requests.import_snapshot().status.active();
        let unavailable = self.requests.check_import_target(&environment).err();
        let synthetic = self.requests.import_is_synthetic();
        let scope = format!(
            "{} · {} · {}",
            self.import_environment_name(&environment, cx),
            source_label(options.source, cx),
            recent_days(options.recent_days, cx)
        );
        let mode = if synthetic {
            text(
                cx,
                "Synthetic import · Sample progress only",
                "合成导入 · 仅演示任务进度",
            )
        } else {
            text(
                cx,
                "Sync scans provider sessions with llmusage in this execution environment.",
                "同步使用当前执行环境的 llmusage 扫描会话。",
            )
        };
        let diagnostic = unavailable
            .or(self.import_error)
            .or(active.then_some(Failure::Busy))
            .map(|error| failure(error, cx));
        div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(self.import_scope_controls(active, cx))
            .child(
                div()
                    .id("insights-import-confirmation")
                    .test_support()
                    .role(gpui_kit::Role::Label)
                    .aria_label(scope.clone())
                    .child(scope),
            )
            .child(
                div()
                    .text_size(crate::theme::text_size(theme::META_SIZE))
                    .child(mode),
            )
            .children(
                diagnostic
                    .map(|message| import_message("insights-import-availability", message.into())),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("insights-import")
                            .label(text(cx, "Confirm sync", "确认同步"))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.confirm_import(environment.clone(), options, window, cx);
                            })),
                        active || unavailable.is_some(),
                        active,
                        cx,
                    ))
                    .child(semantics::ghost_button(
                        Button::new("insights-import-dismiss")
                            .label(text(cx, "Close", "关闭"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.usage_controls.dismiss_panel(window, cx);
                                this.close_import_source_menu();
                                cx.notify();
                            })),
                        false,
                        false,
                        cx,
                    )),
            )
    }

    pub(super) fn retained_import_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self.requests.import_snapshot();
        let environment = self.requests.environment();
        let run = snapshot.run;
        let belongs_here = snapshot.environment.as_ref() == Some(&environment);
        let active = snapshot.status.active() && belongs_here;
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .children(belongs_here.then(|| {
                import_message(
                    "insights-import-status",
                    self.import_status(&snapshot, &environment, cx),
                )
            }))
            .children(active.then(|| {
                semantics::button(
                    Button::new("insights-import-cancel")
                        .label(text(cx, "Cancel import", "取消导入"))
                        .ghost()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if this.requests.environment() == environment {
                                this.requests.cancel_import_run(&environment, run);
                                this.import_error = None;
                                cx.notify();
                            }
                        })),
                    snapshot.status == Status::CancelRequested,
                    snapshot.status == Status::CancelRequested,
                    cx,
                )
            }))
    }

    fn import_status(
        &self,
        snapshot: &Snapshot,
        environment: &EnvironmentId,
        cx: &gpui_kit::App,
    ) -> String {
        let partial = snapshot.source_errors > 0
            || matches!(
                snapshot.tools,
                ToolUpdate::Failed | ToolUpdate::Finished { errors: 1.., .. }
            );
        let status = match snapshot.status {
            Status::Idle => "",
            Status::Queued => text(cx, "Usage import queued", "用量导入已排队"),
            Status::Running => text(
                cx,
                "Usage import running; leaving this page keeps it running.",
                "正在导入用量，离开页面后仍继续运行。",
            ),
            Status::CancelRequested => text(
                cx,
                "Cancellation requested; waiting for process cleanup.",
                "已请求取消，正在等待进程清理。",
            ),
            Status::Cancelled => text(
                cx,
                "Usage import cancelled after process cleanup.",
                "用量导入已取消，进程清理已结束。",
            ),
            Status::Failed(error) => failure(error, cx),
            Status::Finished if partial => text(cx, "Import partially completed", "用量已部分更新"),
            Status::Finished => text(cx, "Import finished", "用量导入已完成"),
        };
        let mut status = format!(
            "{status} · {} · {} · {}",
            self.import_environment_name(environment, cx),
            source_label(snapshot.options.source, cx),
            recent_days(snapshot.options.recent_days, cx),
        );
        let files = snapshot.files_scanned;
        let inserted = snapshot.records_imported;
        let source_errors = snapshot.source_errors;
        if let Some(seen) = snapshot.total_seen {
            status.push_str(&if is_chinese(cx) {
                format!(" · 扫描 {files} 个文件，读取 {seen} 条，新增 {inserted} 条，来源错误 {source_errors}")
            } else {
                format!(" · {files} files, {seen} seen, {inserted} inserted · Source errors {source_errors}")
            });
        } else if snapshot.status == Status::Running {
            status.push_str(&if is_chinese(cx) {
                format!(" · 已扫描 {files} 个文件，新增 {inserted} 条")
            } else {
                format!(" · {files} files scanned, {inserted} inserted")
            });
        }
        match snapshot.tools {
            ToolUpdate::NotStarted => {}
            ToolUpdate::Running => status.push_str(text(
                cx,
                " · Updating Claude tool history",
                " · 正在更新 Claude 工具记录",
            )),
            ToolUpdate::Failed if snapshot.status == Status::Cancelled => status.push_str(text(
                cx,
                " · Tool history update cancelled. Usage changes already saved.",
                " · 工具记录更新已取消，已保存的用量数据会保留。",
            )),
            ToolUpdate::Failed => status.push_str(text(
                cx,
                " · Tool history update failed. Usage changes already saved.",
                " · 工具记录更新失败，已保存的用量数据会保留。",
            )),
            ToolUpdate::Finished {
                files,
                calls,
                errors,
            } => status.push_str(&if is_chinese(cx) {
                format!(" · 工具记录扫描 {files} 个文件、处理 {calls} 行、错误 {errors}")
            } else {
                format!(
                    " · Tool history scanned {files} files, processed {calls} rows, {errors} errors"
                )
            }),
        }
        status
    }
}

fn import_message(id: &'static str, message: String) -> impl IntoElement {
    div()
        .id(id)
        .test_support()
        .role(gpui_kit::Role::Status)
        .aria_label(message.clone())
        .text_size(crate::theme::text_size(theme::META_SIZE))
        .child(message)
}

fn recent_days(days: u32, cx: &gpui_kit::App) -> String {
    if is_chinese(cx) {
        format!("最近 {days} 天")
    } else {
        format!("Last {days} days")
    }
}

impl InsightsView {
    fn import_scope_controls(&self, active: bool, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut scope = div().flex().flex_col().gap(px(theme::CONTROL_GAP));
        let mut controls = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .text_size(crate::theme::text_size(theme::META_SIZE))
                    .child(text(cx, "Import scope", "导入范围")),
            )
            .child(semantics::button(
                Button::new("insights-import-source")
                    .label(source_label(self.effective_import_options().source, cx))
                    .ghost()
                    .toggled(self.import_scope.choose_source)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.import_scope.choose_source = !this.import_scope.choose_source;
                        cx.notify();
                    })),
                active || self.platform_scope.is_some(),
                false,
                cx,
            ));
        for days in [7, 30, 90] {
            controls = controls.child(semantics::button(
                Button::new(SharedString::from(format!("insights-import-days-{days}")))
                    .label(recent_days(days, cx))
                    .ghost()
                    .toggled(self.import_scope.options.recent_days == days)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.import_scope.options.recent_days = days;
                        this.import_error = None;
                        cx.notify();
                    })),
                active,
                false,
                cx,
            ));
        }
        scope = scope.child(controls);
        if self.import_scope.choose_source && self.platform_scope.is_none() {
            let mut choices = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
            for source in std::iter::once(None).chain(SourceKind::ALL.into_iter().map(Some)) {
                let id = source.map_or("all", SourceKind::as_str);
                choices = choices.child(semantics::button(
                    Button::new(SharedString::from(format!("insights-import-source-{id}")))
                        .label(source_label(source, cx))
                        .ghost()
                        .toggled(self.import_scope.options.source == source)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.import_scope.options.source = source;
                            this.import_scope.choose_source = false;
                            this.import_error = None;
                            cx.notify();
                        })),
                    active,
                    false,
                    cx,
                ));
            }
            scope = scope.child(choices);
        }
        scope.into_any_element()
    }
}
