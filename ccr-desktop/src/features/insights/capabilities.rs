use super::*;
use crate::runtime::insights::capabilities::Report;
use ccr_usage::UnsupportedReason;

pub(super) struct CapabilitiesView {
    requests: InsightsRequests,
    report: Option<Report>,
    task: Option<Task<()>>,
    generation: u64,
    error: Option<&'static str>,
}

impl CapabilitiesView {
    pub fn new(requests: InsightsRequests, _: &mut Context<Self>) -> Self {
        Self {
            requests,
            report: None,
            task: None,
            generation: 0,
            error: None,
        }
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        self.task = None;
        self.report = None;
        self.error = None;
        cx.notify();
    }
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.suspend(cx);
        let generation = self.generation;
        let request = self.requests.usage_capabilities(RequestKey {
            environment: EnvironmentId::Local("usage-capabilities".into()),
            platform: Platform::Claude,
            generation,
            request_id: generation,
        });
        self.task = Some(cx.spawn(async move |weak, cx| {
            let result = match request {
                Ok(mut subscription) => loop {
                    match subscription.next().await {
                        Some(TaskEvent::Terminal(value)) => {
                            break match value.as_ref() {
                                Completion::Success(report) => Ok(report.clone()),
                                Completion::Failed(error) | Completion::CleanupFailed(error) => {
                                    Err(error.message())
                                }
                                _ => Err(
                                    "Capability read interrupted. Retry. / 能力读取已中断，请重试",
                                ),
                            };
                        }
                        Some(_) => {}
                        None => {
                            break Err(
                                "Capability read interrupted. Retry. / 能力读取已中断，请重试",
                            );
                        }
                    }
                },
                Err(_) => Err("Read queue busy. Retry. / 读取队列繁忙，请重试"),
            };
            let _ = weak.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                match result {
                    Ok(report) => this.report = Some(report),
                    Err(error) => this.error = Some(error),
                }
                this.task = None;
                cx.notify();
            });
        }));
    }
    fn rows(&self, report: &Report, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut rows = div().flex().flex_col().gap(px(theme::CONTROL_GAP));
        for (key, en, zh) in [
            ("overview", "Overview", "概览"),
            ("daily_trends", "Daily trends", "逐日趋势"),
            ("model_breakdown", "Models", "模型"),
            ("provider_breakdown", "Providers", "提供商"),
            ("project_breakdown", "Projects", "项目"),
            ("heatmap", "Heatmap", "热力数据"),
            ("logs", "Records", "记录"),
            ("diagnostics", "Diagnostics", "诊断"),
            ("home_overview", "Combined overview", "联合概览"),
            ("insights", "Insights", "洞察"),
            ("sync_json_events", "CLI streaming import", "CLI 流式导入"),
            ("cancel", "Upstream graceful cancellation", "上游优雅取消"),
        ] {
            let Some(capability) = report.features.get(key) else {
                continue;
            };
            let status =
                if key == "sync_json_events" && !self.requests.capability_cli_probe_enabled() {
                    text(cx, "Disabled in fixture", "fixture 中禁用")
                } else if capability.supported {
                    text(cx, "Available", "可用")
                } else {
                    match capability.reason {
                        Some(UnsupportedReason::CliMissing) => {
                            text(cx, "Installed CLI unavailable", "已安装 CLI 不可用")
                        }
                        Some(UnsupportedReason::DbMissing) => {
                            text(cx, "Database missing", "数据库不存在")
                        }
                        Some(UnsupportedReason::DbUnreadable) => {
                            text(cx, "Database unreadable", "数据库不可读")
                        }
                        Some(UnsupportedReason::SchemaUnsupported) => {
                            text(cx, "Schema version unsupported", "数据库版本不支持")
                        }
                        Some(UnsupportedReason::MissingTable) => {
                            text(cx, "Required table missing", "缺少所需数据表")
                        }
                        Some(UnsupportedReason::MissingColumn) => {
                            text(cx, "Required column missing", "缺少所需字段")
                        }
                        Some(UnsupportedReason::WaitingForLlmusage) => {
                            text(cx, "Not provided by upstream", "上游尚未提供")
                        }
                        None => text(cx, "Unavailable", "不可用"),
                    }
                };
            rows = rows.child(
                div()
                    .id(SharedString::from(format!("usage-capability-{key}")))
                    .test_support()
                    .aria_label(format!("{}: {status}", text(cx, en, zh)))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(theme::SECTION_GAP))
                    .min_h(px(theme::ROW_HEIGHT))
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(text(cx, en, zh))
                    .child(status),
            );
        }
        rows.into_any_element()
    }
}

impl Render for CapabilitiesView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root=div().id("usage-capabilities").test_support().flex_1().min_h_0().overflow_y_scroll()
            .flex().flex_col().gap(px(theme::CONTROL_GAP))
            .child(div().text_size(px(theme::SECTION_TITLE_SIZE)).font_weight(gpui_kit::FontWeight::SEMIBOLD)
                .child(text(cx,"Usage storage and CLI capabilities","用量存储与 CLI 能力")))
            .child(text(cx,"Capabilities describe the complete captured database, independent of report filters. No import or database migration is started.","能力描述整份捕获的数据库，与报表筛选无关。读取不会启动导入或迁移。"))
            .child(semantics::button(Button::new("usage-capabilities-refresh")
                .label(text(cx,"Refresh capabilities","刷新能力")).ghost()
                .on_click(cx.listener(|this,_,_,cx|this.refresh(cx))),self.task.is_some(),self.task.is_some(),cx));
        if let Some(error) = self.error {
            root = root.child(service_error(cx, error));
        }
        if let Some(report) = &self.report {
            let cli = if !self.requests.capability_cli_probe_enabled() {
                text(
                    cx,
                    "CLI probe is disabled for this fixture.",
                    "此 fixture 禁止探测真实 CLI。",
                )
                .to_string()
            } else {
                report.cli_version.clone().unwrap_or_else(|| {
                    text(cx, "Installed CLI unavailable", "已安装 CLI 不可用").into()
                })
            };
            root=root.child(div().id("usage-capabilities-cli").test_support().aria_label(cli.clone()).child(cli))
                .child(format!("{}: {}",text(cx,"Schema version","数据库版本"),report.schema_version.map(|v|v.to_string()).unwrap_or_else(||"—".into())))
                .child(self.rows(report,cx))
                .child(text(cx,"Local job cancellation remains available for running imports; it does not imply an upstream graceful-cancel command.","运行中的导入仍可请求本地任务取消；这不代表上游提供优雅取消命令。"));
        } else if self.task.is_some() {
            root = root.child(text(cx, "Reading capabilities…", "正在读取能力…"));
        }
        root
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_app::runtime::RuntimeOwner;
    use gpui_kit::{
        TestAppContext, WindowOptions,
        test::{TestAppContextExt, TestWindowExt},
    };
    use std::time::Duration;

    #[gpui_kit::test]
    async fn capability_controls_show_missing_and_old_schema_without_host_probe_or_creation(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let services = Services::new(
            owner.handle(),
            crate::runtime::ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("config.toml"),
                delay: Duration::ZERO,
            },
        )
        .with_insights_paths(crate::runtime::insights::InsightsPaths {
            usage_root: root.path().join("llmusage"),
            archive_db: root.path().join("archive.db"),
            provider_home: root.path().join("home"),
        });
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    let mut view = InsightsView::new(services, window, cx);
                    view.open_product(InsightsPage::Usage, cx);
                    view
                })
            })
            .unwrap()
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("insights-capabilities", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).capabilities.read(cx).report.is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(
                window
                    .find("usage-capability-overview")
                    .label()
                    .unwrap()
                    .contains("missing")
            );
            assert!(
                window
                    .find("usage-capabilities-cli")
                    .label()
                    .unwrap()
                    .contains("disabled")
            );
        })
        .unwrap();
        assert!(!root.path().join("llmusage").exists());
        let paths = ccr_usage::AppPaths::from_root(root.path().join("llmusage"));
        std::fs::create_dir_all(paths.db_path.parent().unwrap()).unwrap();
        rusqlite::Connection::open(&paths.db_path).unwrap().execute_batch("CREATE TABLE meta(key TEXT PRIMARY KEY,value TEXT NOT NULL); INSERT INTO meta VALUES('schema_version','9');").unwrap();
        cx.update_window(handle, |_, window, cx| {
            window.click("usage-capabilities-refresh", cx)
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx)
                .capabilities
                .read(cx)
                .report
                .as_ref()
                .is_some_and(|r| r.schema_version == Some(9))
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(
                window
                    .find("usage-capability-overview")
                    .label()
                    .unwrap()
                    .contains("unsupported")
            );
            window.click("insights-usage", cx);
            window.render_frame(cx);
            assert!(window.try_find("usage-capabilities").is_none());
            assert!(view.read(cx).capabilities.read(cx).report.is_none());
            window.remove_window();
        })
        .unwrap();
        assert!(!root.path().join("archive.db").exists());
        owner.shutdown();
    }
}
