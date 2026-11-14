use super::*;
use crate::runtime::insights::import::{Failure, Options, Status, ToolUpdate};

#[derive(Default)]
pub(super) struct Scope {
    options: Options,
    choose_source: bool,
}

fn source_label(source: Option<SourceKind>, cx: &gpui_kit::App) -> &'static str {
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
            "Usage import is available in Local only.",
            "用量导入仅支持本地环境。",
        ),
        Failure::Unconfigured => text(
            cx,
            "Usage import paths are unavailable.",
            "用量导入路径不可用。",
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
            "Install the llmusage CLI before importing.",
            "请先安装 llmusage CLI 再导入。",
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
            "Usage import failed. Check the installed llmusage CLI and local file access.",
            "用量导入失败，请检查 llmusage CLI 和本地文件权限。",
        ),
    }
}

impl InsightsView {
    pub(super) fn observe_import(&mut self, cx: &mut Context<Self>) {
        let mut updates = self.requests.observe_import();
        self.import_task = Some(cx.spawn(async move |weak, cx| {
            while updates.changed().await.is_ok() {
                // Consume the newest retained state at the shared UI budget;
                // hidden pages drop this subscription rather than poll.
                cx.background_executor()
                    .timer(ccr_app::runtime::PROGRESS_INTERVAL)
                    .await;
                let status = updates.borrow_and_update().status;
                if weak
                    .update(cx, |this, cx| {
                        if this.page == InsightsPage::Usage
                            && matches!(
                                status,
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
    fn start_import(&mut self, cx: &mut Context<Self>) {
        let result = self.requests.start_import_with_options(
            RequestKey {
                environment: EnvironmentId::Local("insights-explicit".into()),
                platform: Platform::Claude,
                generation: self.generation,
                request_id: self.generation,
            },
            self.effective_import_options(),
        );
        self.import_error = result.err();
        cx.notify();
    }
    pub(super) fn import_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self.requests.import_snapshot();
        let active = snapshot.status.active();
        let mut status = if let Some(error) = self.import_error {
            failure(error, cx).to_string()
        } else {
            match snapshot.status {
                Status::Idle => text(cx, "Import uses the installed llmusage CLI and default local provider directories.", "导入使用已安装的 llmusage CLI 和默认本地会话来源目录。").to_string(),
                Status::Queued => text(cx, "Usage import queued", "用量导入已排队").to_string(),
                Status::Running => text(cx, "Usage import running; leaving this page keeps it running.", "正在导入用量，离开页面后仍继续运行。").to_string(),
                Status::CancelRequested => text(cx, "Cancellation requested; waiting for process cleanup.", "已请求取消，正在等待进程清理。").to_string(),
                Status::Cancelled => text(cx, "Usage import cancelled after process cleanup.", "用量导入已取消，进程清理已结束。").to_string(),
                Status::Failed(error) => failure(error, cx).to_string(),
                Status::Finished => if is_chinese(cx) {
                    format!("导入完成：扫描 {} 个文件，读取 {} 条，新增 {} 条，来源错误 {}", snapshot.files_scanned, snapshot.total_seen.unwrap_or(0), snapshot.records_imported, snapshot.source_errors)
                } else {
                    format!("Import finished: {} files, {} seen, {} inserted, {} source errors", snapshot.files_scanned, snapshot.total_seen.unwrap_or(0), snapshot.records_imported, snapshot.source_errors)
                },
            }
        };
        match snapshot.tools {
            ToolUpdate::NotStarted => {}
            ToolUpdate::Running => status.push_str(text(
                cx,
                " Updating Claude tool history.",
                " 正在更新 Claude 工具记录。",
            )),
            ToolUpdate::Failed => status.push_str(text(
                cx,
                " Tool history did not finish; completed usage changes remain.",
                " 工具记录更新未完成，已完成的用量变更仍保留。",
            )),
            ToolUpdate::Finished {
                files,
                calls,
                errors,
            } => status.push_str(&if is_chinese(cx) {
                format!(" 工具记录：扫描 {files} 个文件、处理 {calls} 行、错误 {errors}。")
            } else {
                format!(" Tool history: {files} files, {calls} rows processed, {errors} errors.")
            }),
        }
        if snapshot.status == Status::Cancelled && snapshot.total_seen.is_some() {
            status.push_str(&if is_chinese(cx) {
                format!(" 用量阶段已写入 {} 条，工具更新已取消；未回滚已完成变更。",snapshot.records_imported)
            } else { format!(" Usage already inserted {} records; tool update was cancelled. Completed changes were not rolled back.",snapshot.records_imported) });
        }
        if snapshot.run > 0 {
            status.push_str(&if is_chinese(cx) {
                format!(
                    " 本次来源：{}，最近 {} 天。",
                    source_label(snapshot.options.source, cx),
                    snapshot.options.recent_days
                )
            } else {
                format!(
                    " Run scope: {}, last {} days.",
                    source_label(snapshot.options.source, cx),
                    snapshot.options.recent_days
                )
            });
        }
        div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(self.import_scope_controls(active, cx))
            .child(
                div()
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("insights-import")
                            .label(text(cx, "Import local usage", "导入本地用量"))
                            .on_click(cx.listener(|this, _, _, cx| this.start_import(cx))),
                        active,
                        active,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("insights-import-cancel")
                            .label(text(cx, "Cancel import", "取消导入"))
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.requests.cancel_import();
                                this.import_error = None;
                                cx.notify();
                            })),
                        !active || snapshot.status == Status::CancelRequested,
                        false,
                        cx,
                    )),
            )
            .child(
                div()
                    .id("insights-import-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .text_size(crate::theme::text_size(theme::META_SIZE))
                    .child(status),
            )
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
                    .label(if is_chinese(cx) {
                        format!("最近 {days} 天")
                    } else {
                        format!("Last {days} days")
                    })
                    .ghost()
                    .toggled(self.import_scope.options.recent_days == days)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.import_scope.options.recent_days = days;
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
