use ccr_core::core::error::Result;
use ccr_usage::{ProviderBreakdownDto, QueryFilter, SourceKind, UsageError};
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, error::TryRecvError};

use crate::tui::runtime::{AsyncTaskExecutor, TuiApp};
use crate::tui::toast::{Toast, ToastManager};

#[derive(Debug, Clone, PartialEq)]
pub struct UsageProviderRow {
    pub platform: SourceKind,
    pub provider: Option<String>,
    pub request_count: i64,
    pub input_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
    pub cost_with_cache_usd: f64,
}

impl UsageProviderRow {
    pub fn display_provider(&self) -> &str {
        self.provider.as_deref().unwrap_or("unattributed")
    }

    pub fn output_tokens_total(&self) -> i64 {
        self.output_tokens + self.reasoning_output_tokens
    }

    pub fn cache_tokens_total(&self) -> i64 {
        self.cache_read_tokens + self.cache_creation_tokens
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageDataset {
    pub rows: Vec<UsageProviderRow>,
}

impl UsageDataset {
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn platform_rows(&self, platform: SourceKind) -> impl Iterator<Item = &UsageProviderRow> {
        self.rows.iter().filter(move |row| row.platform == platform)
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

enum UsageTaskMessage {
    Loaded(std::result::Result<UsageDataset, UsageError>),
}

pub struct UsageApp {
    pub state: UsageLoadState,
    pub toasts: ToastManager,
    task_executor: AsyncTaskExecutor,
    task_tx: UnboundedSender<UsageTaskMessage>,
    task_rx: UnboundedReceiver<UsageTaskMessage>,
    task_active: bool,
    activation_delay_ticks: Option<u32>,
}

impl UsageApp {
    pub fn with_task_executor(task_executor: AsyncTaskExecutor) -> Self {
        let (task_tx, task_rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            state: UsageLoadState::Idle,
            toasts: ToastManager::new(),
            task_executor,
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

        self.task_executor.spawn_blocking(move || {
            let result = load_usage_dataset();
            let _ = tx.send(UsageTaskMessage::Loaded(result));
        });
    }

    fn drain_task_messages(&mut self) -> bool {
        let mut changed = false;
        loop {
            match self.task_rx.try_recv() {
                Ok(UsageTaskMessage::Loaded(Ok(dataset))) => {
                    self.state = if dataset.is_empty() {
                        UsageLoadState::Empty
                    } else {
                        UsageLoadState::Loaded(dataset)
                    };
                    self.task_active = false;
                    changed = true;
                }
                Ok(UsageTaskMessage::Loaded(Err(error))) => {
                    self.state = match error {
                        UsageError::SchemaUnsupported { .. }
                        | UsageError::FeatureUnavailable { .. } => {
                            UsageLoadState::Unsupported(format!("{error}"))
                        }
                        other => UsageLoadState::Error(format!("{other}")),
                    };
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

fn load_usage_dataset() -> std::result::Result<UsageDataset, UsageError> {
    let paths = ccr_usage::discover_llmusage_paths().map_err(UsageError::DbUnreadable)?;
    let dashboard = ccr_usage::open_dashboard(paths)?;
    let mut rows = Vec::new();

    for platform in [SourceKind::Claude, SourceKind::Codex] {
        let provider_rows = dashboard.provider_breakdown(&QueryFilter {
            source: Some(platform),
            ..QueryFilter::default()
        })?;
        rows.extend(
            provider_rows
                .into_iter()
                .map(|row| provider_row_from_shared(platform, row)),
        );
    }

    Ok(UsageDataset { rows })
}

fn provider_row_from_shared(platform: SourceKind, row: ProviderBreakdownDto) -> UsageProviderRow {
    UsageProviderRow {
        platform,
        provider: row.provider,
        request_count: row.request_count,
        input_tokens: row.input_tokens,
        cache_read_tokens: row.cache_read_tokens,
        cache_creation_tokens: row.cache_creation_tokens,
        output_tokens: row.output_tokens,
        reasoning_output_tokens: row.reasoning_output_tokens,
        total_tokens: row.total_tokens,
        cost_with_cache_usd: row.cost_with_cache_usd,
    }
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

    #[test]
    fn provider_row_sums_output_and_cache_tokens() {
        let row = UsageProviderRow {
            platform: SourceKind::Codex,
            provider: None,
            request_count: 1,
            input_tokens: 10,
            cache_read_tokens: 2,
            cache_creation_tokens: 3,
            output_tokens: 4,
            reasoning_output_tokens: 5,
            total_tokens: 24,
            cost_with_cache_usd: 0.12,
        };

        assert_eq!(row.display_provider(), "unattributed");
        assert_eq!(row.output_tokens_total(), 9);
        assert_eq!(row.cache_tokens_total(), 5);
    }
}
