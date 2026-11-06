//! Native install adapter; the existing CLI service owns plans and child cleanup.
use super::Services;
use ccr_app::runtime::{AdmissionError, Completion, EnvironmentId, RequestKey, TaskSubscription};
use ccr_cli::services::{
    install_service::InstallService,
    install_types::{
        DetectionResult, HostCapabilities, InstallEvent, InstallFlowError,
        InstallPlanConsumeErrorCode, ManualCatalog, PlanId, PlanOutcome, ProgressStage,
        RingBufferSnapshot,
    },
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::sync::watch;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Unsupported,
    Busy,
    Confirmation,
    PlanUnavailable,
    HostChanged,
    Operation,
    Interrupted,
}
impl Error {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unsupported => {
                "Installation is local-only and disabled in fixture mode. / 安装仅支持本地环境，演示模式禁用"
            }
            Self::Busy => "An installation is already queued or running. / 已有安装任务等待或运行",
            Self::Confirmation => {
                "Review and acknowledge this install plan first. / 请先核对并确认安装计划"
            }
            Self::PlanUnavailable => {
                "The install plan expired or was already used. Check and plan again. / 安装计划已失效或已使用，请重新检测并规划"
            }
            Self::HostChanged => {
                "Host detection changed. Check and plan again. / 主机检测结果已变化，请重新检测并规划"
            }
            Self::Operation => {
                "The install operation failed. Check the retained status and retry. / 安装操作失败，请检查保留状态后重试"
            }
            Self::Interrupted => {
                "The install runner ended without a terminal result. Success is unconfirmed. / 安装执行器结束但无终态，尚未确认成功"
            }
        }
    }
}
impl From<InstallFlowError> for Error {
    fn from(error: InstallFlowError) -> Self {
        match error {
            InstallFlowError::AlreadyRunning => Self::Busy,
            InstallFlowError::PlanUnavailable {
                code: InstallPlanConsumeErrorCode::HostMismatch,
                ..
            }
            | InstallFlowError::InvalidPayload { .. } => Self::HostChanged,
            InstallFlowError::PlanUnavailable { .. } => Self::PlanUnavailable,
            _ => Self::Operation,
        }
    }
}

#[derive(Clone)]
pub struct Snapshot {
    pub detection: Option<DetectionResult>,
    pub capabilities: Option<HostCapabilities>,
    pub plan: Option<PlanOutcome>,
    pub manual: Option<ManualCatalog>,
    pub recent: RingBufferSnapshot,
    pub queued: bool,
    pub running: bool,
    pub cancel_requested: bool,
    pub stage: Option<ProgressStage>,
    pub error: Option<Error>,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            detection: None,
            capabilities: None,
            plan: None,
            manual: None,
            recent: RingBufferSnapshot {
                attempt_id: None,
                logs: Vec::new(),
                terminal: None,
            },
            queued: false,
            running: false,
            cancel_requested: false,
            stage: None,
            error: None,
        }
    }
}
impl Snapshot {
    pub fn busy(&self) -> bool {
        self.queued || self.running
    }
}

/// One application-lifetime projection, never a second plan or process owner.
pub struct InstallOwner {
    service: Arc<InstallService>,
    state: Mutex<Snapshot>,
    updates: watch::Sender<Arc<Snapshot>>,
    query_generation: AtomicU64,
    #[cfg(test)]
    synthetic: bool,
}
impl Default for InstallOwner {
    fn default() -> Self {
        Self::with_service(InstallService::new())
    }
}
impl InstallOwner {
    fn with_service(service: InstallService) -> Self {
        Self {
            service: Arc::new(service),
            state: Mutex::new(Snapshot::default()),
            updates: watch::channel(Arc::new(Snapshot::default())).0,
            query_generation: AtomicU64::new(0),
            #[cfg(test)]
            synthetic: false,
        }
    }
    fn publish(&self, change: impl FnOnce(&mut Snapshot)) -> Arc<Snapshot> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        change(&mut state);
        let snapshot = Arc::new(state.clone());
        self.updates.send_replace(snapshot.clone());
        snapshot
    }
    pub fn observe(&self) -> watch::Receiver<Arc<Snapshot>> {
        self.updates.subscribe()
    }
    pub fn snapshot(&self) -> Arc<Snapshot> {
        self.updates.borrow().clone()
    }
    fn publish_query(
        &self,
        generation: u64,
        change: impl FnOnce(&mut Snapshot),
    ) -> Result<Arc<Snapshot>, Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.query_generation.load(Ordering::SeqCst) != generation {
            return Err(Error::Interrupted);
        }
        change(&mut state);
        let snapshot = Arc::new(state.clone());
        self.updates.send_replace(snapshot.clone());
        Ok(snapshot)
    }
    fn recent(&self, state: &mut Snapshot) {
        state.recent = self.service.recent_events();
        for event in &mut state.recent.logs {
            if let InstallEvent::Log { line, .. } = event {
                *line = safe_line(line);
            }
        }
        if let Some(InstallEvent::Failed {
            stderr_excerpt,
            error_message,
            ..
        }) = &mut state.recent.terminal
        {
            *stderr_excerpt = None;
            *error_message = Error::Operation.message().into();
        }
    }
}
fn safe_line(line: &str) -> String {
    let line = ccr_core::core::log_redact::redact_log_text(line);
    let mut end = line.len().min(8 * 1024);
    while !line.is_char_boundary(end) {
        end -= 1;
    }
    line[..end].to_owned()
}

#[derive(Clone)]
pub struct InstallRequests(Services);
pub enum Query {
    Check,
    Plan,
    Manual,
}
impl InstallRequests {
    pub fn new(services: &Services) -> Self {
        Self(services.clone())
    }
    pub fn is_fixture(&self) -> bool {
        self.0.is_fixture()
    }
    pub fn observe(&self) -> watch::Receiver<Arc<Snapshot>> {
        self.0.install.observe()
    }
    pub fn snapshot(&self) -> Arc<Snapshot> {
        self.0.install.snapshot()
    }
    fn supported(&self, key: &RequestKey) -> bool {
        if self.0.source.is_benchmark() {
            return false;
        }
        let local = matches!(key.environment, EnvironmentId::Local(_));
        #[cfg(test)]
        if self.0.install.synthetic {
            return local;
        }
        local && !self.0.is_fixture()
    }
    pub fn query(
        &self,
        key: RequestKey,
        query: Query,
    ) -> Result<TaskSubscription<Arc<Snapshot>, Error>, AdmissionError> {
        let supported = self.supported(&key);
        let owner = self.0.install.clone();
        let generation = owner.query_generation.fetch_add(1, Ordering::SeqCst) + 1;
        self.0.executor.read(key, move |cancel, _| {
            if !supported {
                return Err(Error::Unsupported);
            }
            let handle = tokio::runtime::Handle::try_current().map_err(|_| Error::Operation)?;
            handle.block_on(async {
                if cancel.is_cancelled() {
                    return Err(Error::Interrupted);
                }
                match query {
                    Query::Check => {
                        let detection = owner.service.detect().await.map_err(Error::from)?;
                        let capabilities = owner.service.probe_capabilities();
                        if cancel.is_cancelled() {
                            return Err(Error::Interrupted);
                        }
                        owner.publish_query(generation, |state| {
                            state.detection = Some(detection);
                            state.capabilities = Some(capabilities);
                            state.plan = None;
                            state.error = None;
                        })
                    }
                    Query::Plan => {
                        let current = owner.snapshot();
                        let (Some(detection), Some(capabilities)) =
                            (&current.detection, &current.capabilities)
                        else {
                            return Err(Error::HostChanged);
                        };
                        let plan = owner
                            .service
                            .plan(detection, capabilities)
                            .await
                            .map_err(Error::from)?;
                        if cancel.is_cancelled() {
                            return Err(Error::Interrupted);
                        }
                        owner.publish_query(generation, |state| {
                            state.plan = Some(plan);
                            state.error = None;
                        })
                    }
                    Query::Manual => {
                        let manual = owner.service.manual_catalog().map_err(Error::from)?;
                        if cancel.is_cancelled() {
                            return Err(Error::Interrupted);
                        }
                        owner.publish_query(generation, |state| state.manual = Some(manual))
                    }
                }
            })
        })
    }
    pub fn execute(
        &self,
        key: RequestKey,
        plan_id: PlanId,
        acknowledged: bool,
    ) -> Result<TaskSubscription<Arc<Snapshot>, Error>, Error> {
        if !self.supported(&key) {
            return Err(Error::Unsupported);
        }
        if !acknowledged {
            return Err(Error::Confirmation);
        }
        let owner = self.0.install.clone();
        {
            let mut state = owner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.busy() {
                return Err(Error::Busy);
            }
            state.queued = true;
            state.cancel_requested = false;
            state.error = None;
        }
        owner.publish(|_| {});
        let guard = Queued {
            owner: owner.clone(),
            started: false,
        };
        self.0.executor.job(key, move |cancel, _| {
            let mut guard = guard;
            let Ok(handle) = tokio::runtime::Handle::try_current() else { return Completion::Failed(Error::Operation); };
            handle.block_on(async {
                if owner.snapshot().cancel_requested || cancel.is_cancelled() { return Completion::Cancelled; }
                let attempt = match owner.service.execute(plan_id).await {
                    Ok(attempt) => attempt,
                    Err(error) => {
                        let error = Error::from(error);
                        owner.publish(|state| state.error = Some(error));
                        return Completion::Failed(error);
                    }
                };
                guard.started = true;
                let id = attempt.attempt_id;
                let mut events = attempt.events;
                owner.publish(|state| {
                    state.queued = false; state.running = true; state.plan = None; state.stage = None;
                    state.recent = RingBufferSnapshot { attempt_id: Some(id), logs: Vec::new(), terminal: None };
                });
                let mut tick = tokio::time::interval(Duration::from_millis(100));
                let mut cancel_sent = false;
                let mut open = true;
                loop {
                    tokio::select! {
                        _ = cancel.cancelled(), if !cancel_sent => {
                            let _ = owner.service.cancel(id).await;
                            cancel_sent = true;
                            owner.publish(|state| state.cancel_requested = true);
                        }
                        event = events.recv(), if open => {
                            match event {
                                Some(InstallEvent::Progress { stage, .. }) => { owner.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).stage = Some(stage); }
                                Some(_) => {},
                                None => open = false,
                            }
                        }
                        _ = tick.tick() => {
                            if !cancel_sent && owner.snapshot().cancel_requested {
                                let _ = owner.service.cancel(id).await;
                                cancel_sent = true;
                            }
                            let active = owner.service.current_attempt_id().await == Some(id);
                            owner.publish(|state| { owner.recent(state); state.running = active; if active { state.recent.terminal = None; } });
                            if !active { break; }
                        }
                    }
                }
                let snapshot = owner.publish(|state| {
                    owner.recent(state); state.running = false; state.queued = false; state.stage = None;
                    if state.recent.terminal.is_none() { state.error = Some(Error::Interrupted); }
                });
                if snapshot.recent.terminal.is_none() { Completion::Failed(Error::Interrupted) }
                else { Completion::Success(snapshot) }
            })
        }).map_err(|_| Error::Operation)
    }
    pub fn cancel(&self, key: RequestKey) -> Result<Arc<Snapshot>, Error> {
        if !self.supported(&key) {
            return Err(Error::Unsupported);
        }
        // Cancellation remains reachable without another worker/admission slot.
        // The retained runner forwards this request and owns cleanup/terminal.
        Ok(self.0.install.publish(|state| {
            if state.busy() {
                state.cancel_requested = true;
            }
        }))
    }
}
struct Queued {
    owner: Arc<InstallOwner>,
    started: bool,
}
impl Drop for Queued {
    fn drop(&mut self) {
        if !self.started {
            self.owner.publish(|state| {
                state.queued = false;
                if state.error.is_none() {
                    state.error = Some(Error::Interrupted);
                }
            });
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
pub(crate) mod tests;
