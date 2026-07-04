use std::sync::Arc;

use ccr_core::core::error::Result;
use ccr_usage::{QueryFilter, SourceKind, TaggedProviderBreakdown, UsageError};
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, error::TryRecvError};

use crate::tui::runtime::{AsyncTaskExecutor, TuiApp};
use crate::tui::toast::{Toast, ToastManager};

#[derive(Debug, Clone, PartialEq)]
pub struct UsageDataset {
    pub rows: Vec<TaggedProviderBreakdown>,
}

impl UsageDataset {
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn platform_rows(
        &self,
        platform: SourceKind,
    ) -> impl Iterator<Item = &TaggedProviderBreakdown> {
        self.rows.iter().filter(move |row| row.source == platform)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UsageLoadState {
    Idle,
    Loading,
    Loaded(UsageDataset),
    Empty,
    Unsupported(String),
    Error(String),
}

/// 可注入的数据加载 seam：生产走 llmusage 共享投影，测试注入假 loader
/// 即可驱动完整状态机，无需真实 ~/.llmusage 数据库。
pub type UsageLoader = Arc<dyn Fn() -> std::result::Result<UsageDataset, UsageError> + Send + Sync>;

enum UsageTaskMessage {
    Loaded(std::result::Result<UsageDataset, UsageError>),
}

pub struct UsageApp {
    pub state: UsageLoadState,
    pub toasts: ToastManager,
    task_executor: AsyncTaskExecutor,
    loader: UsageLoader,
    task_tx: UnboundedSender<UsageTaskMessage>,
    task_rx: UnboundedReceiver<UsageTaskMessage>,
    task_active: bool,
    activation_delay_ticks: Option<u32>,
}

impl UsageApp {
    pub fn with_task_executor(task_executor: AsyncTaskExecutor) -> Self {
        Self::with_loader(task_executor, Arc::new(load_usage_dataset))
    }

    pub fn with_loader(task_executor: AsyncTaskExecutor, loader: UsageLoader) -> Self {
        let (task_tx, task_rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            state: UsageLoadState::Idle,
            toasts: ToastManager::new(),
            task_executor,
            loader,
            task_tx,
            task_rx,
            task_active: false,
            activation_delay_ticks: None,
        }
    }

    pub fn on_activated(&mut self) {
        if matches!(self.state, UsageLoadState::Idle) {
            self.activation_delay_ticks = Some(1);
        }
    }

    pub fn refresh(&mut self) {
        self.start_fetch();
    }

    fn start_fetch(&mut self) {
        if self.task_active {
            return;
        }

        self.state = UsageLoadState::Loading;
        self.task_active = true;
        let tx = self.task_tx.clone();
        let loader = self.loader.clone();

        self.task_executor.spawn_blocking(move || {
            let result = loader();
            let _ = tx.send(UsageTaskMessage::Loaded(result));
        });
    }

    fn drain_task_messages(&mut self) -> bool {
        let mut changed = false;
        loop {
            match self.task_rx.try_recv() {
                Ok(UsageTaskMessage::Loaded(result)) => {
                    self.state = state_from_load_result(result);
                    self.task_active = false;
                    changed = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        changed
    }
}

// 错误分类唯一出口：SchemaUnsupported / FeatureUnavailable 属于"llmusage 版本
// 不够新"的可自助恢复态（Unsupported），其余为真实错误（Error）。
fn state_from_load_result(result: std::result::Result<UsageDataset, UsageError>) -> UsageLoadState {
    match result {
        Ok(dataset) => {
            if dataset.is_empty() {
                UsageLoadState::Empty
            } else {
                UsageLoadState::Loaded(dataset)
            }
        }
        Err(
            error @ (UsageError::SchemaUnsupported { .. } | UsageError::FeatureUnavailable { .. }),
        ) => UsageLoadState::Unsupported(format!("{error}")),
        Err(other) => UsageLoadState::Error(format!("{other}")),
    }
}

fn load_usage_dataset() -> std::result::Result<UsageDataset, UsageError> {
    let paths = ccr_usage::discover_llmusage_paths().map_err(UsageError::DbUnreadable)?;
    let dashboard = ccr_usage::open_dashboard(paths)?;
    let rows = dashboard.provider_breakdown_by_source(
        &[SourceKind::Claude, SourceKind::Codex],
        &QueryFilter::default(),
    )?;
    Ok(UsageDataset { rows })
}

impl TuiApp for UsageApp {
    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(true),
            KeyCode::Char('r') => {
                self.refresh();
                self.toasts.push(Toast::info("正在刷新用量统计"));
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn on_tick(&mut self) -> bool {
        let mut needs_redraw = self.toasts.tick();
        if let Some(remaining) = self.activation_delay_ticks.as_mut() {
            needs_redraw = true;
            if *remaining > 0 {
                *remaining -= 1;
            }
            if *remaining == 0 {
                self.activation_delay_ticks = None;
                self.start_fetch();
            }
        }
        needs_redraw |= self.drain_task_messages();
        needs_redraw
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        super::ui::draw(frame, self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccr_usage::ProviderBreakdownDto;

    fn sample_row(
        source: SourceKind,
        provider: Option<&str>,
        total_tokens: i64,
    ) -> TaggedProviderBreakdown {
        TaggedProviderBreakdown {
            source,
            breakdown: ProviderBreakdownDto {
                provider: provider.map(str::to_string),
                request_count: 1,
                input_tokens: 10,
                cache_read_tokens: 2,
                cache_creation_tokens: 3,
                output_tokens: 4,
                reasoning_output_tokens: 5,
                total_tokens,
                cost_with_cache_usd: 0.12,
                cost_without_cache_usd: 0.15,
            },
        }
    }

    fn sample_dataset() -> UsageDataset {
        UsageDataset {
            rows: vec![
                sample_row(SourceKind::Claude, Some("anthropic"), 50),
                sample_row(SourceKind::Codex, Some("openai"), 120),
                sample_row(SourceKind::Codex, None, 30),
            ],
        }
    }

    #[test]
    fn dataset_filters_rows_by_platform_tag() {
        let dataset = sample_dataset();

        let claude: Vec<_> = dataset.platform_rows(SourceKind::Claude).collect();
        let codex: Vec<_> = dataset.platform_rows(SourceKind::Codex).collect();

        assert_eq!(claude.len(), 1);
        assert_eq!(claude[0].breakdown.provider.as_deref(), Some("anthropic"));
        assert_eq!(codex.len(), 2);
        assert!(codex.iter().any(|row| row.breakdown.provider.is_none()));
    }

    #[test]
    fn load_result_classifies_success_and_empty() {
        assert!(matches!(
            state_from_load_result(Ok(sample_dataset())),
            UsageLoadState::Loaded(dataset) if dataset.rows.len() == 3
        ));
        assert_eq!(
            state_from_load_result(Ok(UsageDataset { rows: Vec::new() })),
            UsageLoadState::Empty
        );
    }

    #[test]
    fn load_result_classifies_schema_and_feature_errors_as_unsupported() {
        let schema = state_from_load_result(Err(UsageError::SchemaUnsupported {
            expected: 14,
            actual: Some(13),
        }));
        assert!(
            matches!(&schema, UsageLoadState::Unsupported(message) if message.contains("expected schema >= 14"))
        );

        let feature = state_from_load_result(Err(UsageError::FeatureUnavailable {
            feature: "provider_breakdown",
            reason: "missing column".to_string(),
        }));
        assert!(
            matches!(&feature, UsageLoadState::Unsupported(message) if message.contains("provider_breakdown"))
        );
    }

    #[test]
    fn load_result_classifies_remaining_errors_as_error() {
        for error in [
            UsageError::DbMissing(std::path::PathBuf::from("/tmp/llmusage.db")),
            UsageError::DbUnreadable("io".to_string()),
            UsageError::Query("boom".to_string()),
        ] {
            assert!(matches!(
                state_from_load_result(Err(error)),
                UsageLoadState::Error(_)
            ));
        }
    }

    async fn drain_until_settled(app: &mut UsageApp) {
        for _ in 0..200 {
            if app.drain_task_messages() {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        panic!("usage task message should arrive before the polling budget runs out");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn injected_loader_drives_state_machine_to_loaded() {
        let dataset = sample_dataset();
        let mut app = UsageApp::with_loader(
            AsyncTaskExecutor::from_current_or_test(),
            Arc::new(move || Ok(dataset.clone())),
        );

        assert_eq!(app.state, UsageLoadState::Idle);
        app.refresh();
        assert_eq!(app.state, UsageLoadState::Loading);

        drain_until_settled(&mut app).await;

        assert!(matches!(&app.state, UsageLoadState::Loaded(dataset) if dataset.rows.len() == 3));

        // 任务结束后允许再次刷新（task_active 已复位）
        app.refresh();
        assert_eq!(app.state, UsageLoadState::Loading);
        drain_until_settled(&mut app).await;
        assert!(matches!(app.state, UsageLoadState::Loaded(_)));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn injected_loader_classifies_schema_error_as_unsupported() {
        let mut app = UsageApp::with_loader(
            AsyncTaskExecutor::from_current_or_test(),
            Arc::new(|| {
                Err(UsageError::SchemaUnsupported {
                    expected: 14,
                    actual: Some(13),
                })
            }),
        );

        app.refresh();
        drain_until_settled(&mut app).await;

        assert!(matches!(app.state, UsageLoadState::Unsupported(_)));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn injected_loader_reports_empty_dataset() {
        let mut app = UsageApp::with_loader(
            AsyncTaskExecutor::from_current_or_test(),
            Arc::new(|| Ok(UsageDataset { rows: Vec::new() })),
        );

        app.refresh();
        drain_until_settled(&mut app).await;

        assert_eq!(app.state, UsageLoadState::Empty);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn injected_loader_reports_db_error() {
        let mut app = UsageApp::with_loader(
            AsyncTaskExecutor::from_current_or_test(),
            Arc::new(|| Err(UsageError::Query("boom".to_string()))),
        );

        app.refresh();
        drain_until_settled(&mut app).await;

        assert!(matches!(&app.state, UsageLoadState::Error(message) if message.contains("boom")));
    }
}
