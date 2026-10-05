//! Secret-free interaction and serialized background account operations.

use std::sync::{Arc, mpsc};

use ccr_cli::platforms::grok::{GrokActivationState, GrokPlatform};
use ccr_cli::services::grok_auth_service::{GrokAuthRevision, GrokAuthService, GrokAuthSnapshot};
use ccr_core::core::error::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{Frame, layout::Rect};

use crate::tui::CompletedAction;
use crate::tui::pagination::{page_for_index, visible_page_size};
use crate::tui::runtime::{AsyncTaskExecutor, TuiApp};

#[derive(Clone)]
pub(super) struct Request {
    pub action: CompletedAction,
    pub name: String,
    pub scope: String,
    pub revision: GrokAuthRevision,
    pub replace: bool,
}
pub(super) enum Modal {
    Source {
        selected: usize,
        revision: GrokAuthRevision,
    },
    Name {
        value: String,
        scope: String,
        revision: GrokAuthRevision,
    },
    Confirm(Request),
}
pub(super) struct Loaded {
    pub snapshot: GrokAuthSnapshot,
    pub activation: Option<GrokActivationState>,
}
// Tests inject this service seam without opening home files.
pub(super) trait Backend: Send + Sync {
    fn read(&self) -> std::result::Result<Loaded, String>;
    fn execute(&self, request: &Request) -> std::result::Result<Vec<String>, String>;
}
struct ServiceBackend;
impl Backend for ServiceBackend {
    fn read(&self) -> std::result::Result<Loaded, String> {
        let snapshot = GrokAuthService::new()
            .read_snapshot()
            .map_err(|e| e.to_string())?;
        let activation = GrokPlatform::new()
            .and_then(|p| p.inspect_activation_state())
            .ok();
        Ok(Loaded {
            snapshot,
            activation,
        })
    }
    fn execute(&self, r: &Request) -> std::result::Result<Vec<String>, String> {
        let service = GrokAuthService::new();
        let result = match r.action {
            CompletedAction::Save => {
                service.save_current(&r.name, &r.scope, &r.revision, r.replace)
            }
            CompletedAction::Switch => service.switch_account(&r.name, &r.revision),
            CompletedAction::Delete => service.delete_account(&r.name, &r.revision),
            CompletedAction::GrokLogout => {
                return service
                    .off_checked(&r.revision)
                    .map(|r| r.warnings)
                    .map_err(|e| e.to_string());
            }
            _ => return Err("Unsupported Grok action".into()),
        };
        result.map(|r| r.warnings).map_err(|e| e.to_string())
    }
}
struct Response {
    action: Option<Request>,
    outcome: std::result::Result<Vec<String>, String>,
    loaded: std::result::Result<Loaded, String>,
}
pub struct GrokAuthApp {
    pub toasts: crate::tui::toast::ToastManager,
    pub(super) snapshot: GrokAuthSnapshot,
    pub(super) selected: usize,
    pub(super) page_size: usize,
    pub(super) list_area: Rect,
    pub(super) modal: Option<Modal>,
    pub(super) stale: bool,
    pub(super) error: Option<String>,
    pub(super) warnings: Vec<String>,
    pub(super) activation: Option<GrokActivationState>,
    pub(super) notice: Option<&'static str>,
    pub last_action: Option<(CompletedAction, String, bool, Option<String>)>,
    executor: AsyncTaskExecutor,
    backend: Arc<dyn Backend>,
    pending: Option<mpsc::Receiver<Response>>,
    initialized: bool,
}
impl GrokAuthApp {
    pub fn new() -> Result<Self> {
        Self::with_task_executor(AsyncTaskExecutor::from_current_or_test())
    }
    pub fn with_task_executor(executor: AsyncTaskExecutor) -> Result<Self> {
        let mut app = Self::empty(executor, Arc::new(ServiceBackend));
        app.refresh();
        Ok(app)
    }
    fn empty(executor: AsyncTaskExecutor, backend: Arc<dyn Backend>) -> Self {
        Self {
            toasts: crate::tui::toast::ToastManager::new(),
            snapshot: GrokAuthSnapshot::default(),
            selected: 0,
            page_size: 1,
            list_area: Rect::default(),
            modal: None,
            stale: false,
            error: None,
            warnings: Vec::new(),
            activation: None,
            notice: None,
            last_action: None,
            executor,
            backend,
            pending: None,
            initialized: false,
        }
    }
    pub fn is_busy(&self) -> bool {
        self.pending.is_some()
    }
    pub fn outcome_unknown(&self) -> bool {
        self.notice == Some("unknown")
    }
    pub fn operation_warnings(&self) -> &[String] {
        &self.warnings
    }
    pub fn on_deactivated(&mut self) {
        if !self.is_busy() {
            self.modal = None;
        }
    }
    pub fn on_activated(&mut self) {
        if !self.is_busy() {
            self.refresh();
        }
    }
    pub(super) fn selected_name(&self) -> Option<&str> {
        self.snapshot
            .accounts
            .get(self.selected)
            .map(|a| a.name.as_str())
    }
    pub(super) fn page(&self) -> usize {
        page_for_index(self.selected, self.page_size)
    }
    pub(super) fn resize_list(&mut self, area: Rect) {
        self.list_area = area;
        self.page_size = visible_page_size(area.height.saturating_sub(2));
    }
    fn select_by(&mut self, delta: isize) {
        self.selected = self
            .selected
            .saturating_add_signed(delta)
            .min(self.snapshot.accounts.len().saturating_sub(1));
    }
    fn refresh(&mut self) {
        self.submit(None);
    }
    fn submit(&mut self, request: Option<Request>) {
        if self.is_busy() {
            return;
        }
        if matches!(self.executor, AsyncTaskExecutor::Disabled) {
            self.notice = Some("executor");
            return;
        }
        let backend = Arc::clone(&self.backend);
        let (tx, rx) = mpsc::channel();
        self.pending = Some(rx);
        self.notice = None;
        self.executor.spawn_blocking(move || {
            let outcome = request
                .as_ref()
                .map_or_else(|| Ok(Vec::new()), |r| backend.execute(r));
            let loaded = backend.read();
            let _ = tx.send(Response {
                action: request,
                outcome,
                loaded,
            });
        });
    }
    fn consume(&mut self, response: Response) {
        let selected = self.selected_name().map(str::to_owned);
        if let Some(request) = response.action {
            let success = response.outcome.is_ok();
            let error = response.outcome.as_ref().err().cloned();
            self.last_action = Some((request.action, request.name, success, error));
            self.warnings = response.outcome.unwrap_or_default();
        }
        match response.loaded {
            Ok(loaded) => {
                self.snapshot = loaded.snapshot;
                self.activation = loaded.activation;
                self.selected = selected
                    .and_then(|name| self.snapshot.accounts.iter().position(|a| a.name == name))
                    .unwrap_or(
                        self.selected
                            .min(self.snapshot.accounts.len().saturating_sub(1)),
                    );
                self.error = None;
                self.stale = false;
                self.initialized = true;
            }
            Err(error) => {
                self.error = Some(error);
                self.stale = true;
            }
        }
    }
    fn request(&self, action: CompletedAction, name: String) -> Request {
        let scope = self
            .snapshot
            .accounts
            .iter()
            .find(|account| account.name == name)
            .map(|account| account.scope.clone())
            .unwrap_or_default();
        Request {
            action,
            name,
            scope,
            revision: self.snapshot.revision.clone(),
            replace: false,
        }
    }
    fn save(&mut self) {
        match self.snapshot.sources.as_slice() {
            [] => self.notice = Some("login"),
            [source] => {
                self.modal = Some(Modal::Name {
                    value: source.matched_account.clone().unwrap_or_default(),
                    scope: source.scope.clone(),
                    revision: self.snapshot.revision.clone(),
                })
            }
            _ => {
                self.modal = Some(Modal::Source {
                    selected: 0,
                    revision: self.snapshot.revision.clone(),
                })
            }
        }
    }
    fn modal_key(&mut self, key: KeyEvent, modal: Modal) {
        if key.code == KeyCode::Esc {
            return;
        }
        match modal {
            Modal::Confirm(request) => match key.code {
                KeyCode::Char('y' | 'Y') => self.submit(Some(request)),
                KeyCode::Enter | KeyCode::Char('n' | 'N') => {}
                _ => self.modal = Some(Modal::Confirm(request)),
            },
            Modal::Source {
                mut selected,
                revision,
            } => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => selected = selected.saturating_sub(1),
                    KeyCode::Down | KeyCode::Char('j') => {
                        selected = (selected + 1).min(self.snapshot.sources.len().saturating_sub(1))
                    }
                    KeyCode::Enter => {
                        if let Some(source) = self.snapshot.sources.get(selected) {
                            self.modal = Some(Modal::Name {
                                value: source.matched_account.clone().unwrap_or_default(),
                                scope: source.scope.clone(),
                                revision,
                            });
                        }
                        return;
                    }
                    _ => {}
                }
                self.modal = Some(Modal::Source { selected, revision });
            }
            Modal::Name {
                mut value,
                scope,
                revision,
            } => {
                match key.code {
                    KeyCode::Char(c)
                        if value.len() < 32
                            && (c.is_ascii_alphanumeric() || c == '_' || c == '-') =>
                    {
                        value.push(c);
                    }
                    KeyCode::Backspace => {
                        value.pop();
                    }
                    KeyCode::Enter if !value.trim().is_empty() => {
                        let name = value.trim().to_owned();
                        let replace = self.snapshot.accounts.iter().any(|a| a.name == name);
                        let request = Request {
                            action: CompletedAction::Save,
                            name,
                            scope,
                            revision,
                            replace,
                        };
                        if replace {
                            self.modal = Some(Modal::Confirm(request));
                        } else {
                            self.submit(Some(request));
                        }
                        return;
                    }
                    _ => {}
                }
                self.modal = Some(Modal::Name {
                    value,
                    scope,
                    revision,
                });
            }
        }
    }
}
impl TuiApp for GrokAuthApp {
    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if self.is_busy() {
            return Ok(false);
        }
        if let Some(modal) = self.modal.take() {
            self.modal_key(key, modal);
            return Ok(false);
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Up | KeyCode::Char('k') => self.select_by(-1),
            KeyCode::Down | KeyCode::Char('j') => self.select_by(1),
            KeyCode::PageUp => self.select_by(-(self.page_size as isize)),
            KeyCode::PageDown => self.select_by(self.page_size as isize),
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('s') if self.initialized && !self.stale => self.save(),
            KeyCode::Enter | KeyCode::Char('d') if self.initialized && !self.stale => {
                if let Some(name) = self.selected_name() {
                    let action = if key.code == KeyCode::Enter {
                        CompletedAction::Switch
                    } else {
                        CompletedAction::Delete
                    };
                    self.modal = Some(Modal::Confirm(self.request(action, name.to_owned())));
                }
            }
            KeyCode::Char('o' | 'O') if self.initialized && !self.stale => {
                self.modal = Some(Modal::Confirm(
                    self.request(CompletedAction::GrokLogout, String::new()),
                ));
            }
            _ => {}
        }
        Ok(false)
    }
    fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<bool> {
        if self.is_busy() || self.modal.is_some() {
            return Ok(false);
        }
        match mouse.kind {
            MouseEventKind::ScrollUp => self.select_by(-1),
            MouseEventKind::ScrollDown => self.select_by(1),
            MouseEventKind::Down(MouseButton::Left)
                if self.list_area.contains((mouse.column, mouse.row).into()) =>
            {
                if let Some(row) =
                    crate::tui::app::list_hit_test(self.list_area, mouse.row, self.page_size)
                {
                    self.selected = (self.page() * self.page_size + row)
                        .min(self.snapshot.accounts.len().saturating_sub(1));
                }
            }
            _ => {}
        }
        Ok(false)
    }
    fn on_tick(&mut self) -> bool {
        let toast_redraw = self.toasts.tick();
        let Some(receiver) = self.pending.as_ref() else {
            return toast_redraw;
        };
        match receiver.try_recv() {
            Ok(response) => {
                self.pending = None;
                self.consume(response);
                true
            }
            Err(mpsc::TryRecvError::Empty) => toast_redraw,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.pending = None;
                self.last_action = None;
                self.warnings.clear();
                self.stale = true;
                if self.notice != Some("unknown") {
                    self.refresh();
                }
                self.notice = Some("unknown");
                true
            }
        }
    }
    fn render(&mut self, frame: &mut Frame) {
        super::ui::draw(frame, self);
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
