//! Explicit S2-only runtime adapter. A completed run remains a process tombstone.
use crate::{
    events::EventLogEvictionTracker, services::benchmark::BenchmarkS2SnapshotDto, state::AppState,
};
use ccr_app::benchmark::{
    Seed,
    s2::{self, Ack, Owner, Snapshot},
};
use ccr_db::services::log_persistence::{LogPersistenceState, LogPersistenceTicket};
use std::sync::{
    Arc, Mutex, MutexGuard,
    atomic::{AtomicBool, Ordering},
};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex as AsyncMutex, OwnedSemaphorePermit};
use tokio_util::sync::CancellationToken;
const CHANNEL: &str = "benchmark:s2";
fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
pub fn seed() -> Result<&'static Seed, String> {
    crate::fixture::admit("benchmark_s2_get_status")?;
    crate::fixture::current()
        .and_then(|layout| layout.benchmark.as_ref())
        .ok_or_else(|| "benchmark_s2_unavailable".into())
}
pub fn adapter(state: &AppState) -> &Arc<Adapter> {
    state
        .benchmark_s2
        .get_or_init(|| Arc::new(Adapter::default()))
}
#[derive(Default)]
pub struct Adapter {
    pub core: Arc<Owner>,
    worker: Mutex<Option<tokio::task::JoinHandle<()>>>,
    exit_waiting: AtomicBool,
}
struct Pending {
    tickets: Vec<LogPersistenceTicket>,
    tracker: Option<EventLogEvictionTracker>,
    evicted: u64,
}
enum SinkContext {
    App(AppHandle),
    #[cfg(test)]
    Headless {
        event_log: crate::events::EventLog,
        logs: ccr_db::services::log_persistence::LogPersistenceService,
    },
}
struct Sink {
    context: SinkContext,
    core: Arc<Owner>,
    pending: AsyncMutex<Pending>,
}
impl SinkContext {
    async fn record(
        &self,
        entry: ccr_types::MonitoringEntry,
    ) -> crate::monitoring::CheckedMonitoringReceipt {
        match self {
            Self::App(app) => {
                crate::monitoring::record_monitoring_entry_checked(app, entry, true).await
            }
            #[cfg(test)]
            Self::Headless { event_log, logs } => {
                let (ring, ticket) =
                    crate::monitoring::append_monitoring_checked(event_log, logs, &entry).await;
                crate::monitoring::CheckedMonitoringReceipt {
                    ring,
                    ticket: Some(ticket),
                    emitted: true,
                }
            }
        }
    }
    async fn flush_logs(&self) {
        match self {
            Self::App(app) => {
                let _ = app
                    .state::<AppState>()
                    .monitoring_logs
                    .force_flush_checked()
                    .await;
            }
            #[cfg(test)]
            Self::Headless { logs, .. } => {
                let _ = logs.force_flush_checked().await;
            }
        }
    }
    async fn finish_tracker(&self, tracker: EventLogEvictionTracker) -> Option<u64> {
        match self {
            Self::App(app) => app
                .state::<AppState>()
                .event_log
                .finish_eviction_tracker(tracker)
                .await
                .ok(),
            #[cfg(test)]
            Self::Headless { event_log, .. } => {
                event_log.finish_eviction_tracker(tracker).await.ok()
            }
        }
    }
}
fn harvest(pending: &mut Pending) -> Ack {
    let mut ack = Ack::default();
    pending.tickets.retain(|ticket| match ticket.state() {
        LogPersistenceState::Buffered | LogPersistenceState::Attempting => true,
        LogPersistenceState::Committed => {
            ack.db_attempted += 1;
            ack.db_committed += 1;
            false
        }
        LogPersistenceState::Failed { attempted } => {
            ack.db_attempted += u64::from(attempted);
            ack.db_failed += 1;
            false
        }
    });
    ack.db_pending = pending.tickets.len() as u64;
    ack
}
impl Sink {
    async fn delivery(&self, entry: ccr_types::MonitoringEntry) -> Ack {
        // Keep at most the same 128 checked tickets. Ordinary flushers may own an
        // inflight batch, so buffer length alone is never treated as completion.
        loop {
            if self.pending.lock().await.tickets.len() < s2::CAPACITY {
                break;
            }
            self.context.flush_logs().await;
            let ack = harvest(&mut *self.pending.lock().await);
            self.core.apply(ack);
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        let receipt = self.context.record(entry).await;
        let mut pending = self.pending.lock().await;
        if let Some(ticket) = receipt.ticket {
            pending.tickets.push(ticket);
        }
        let mut ack = harvest(&mut pending);
        ack.ring = Some(receipt.ring.accepted);
        ack.emit = Some(receipt.emitted);
        if let Some(tracker) = &pending.tracker {
            let total = tracker.evicted();
            ack.evicted = total.saturating_sub(pending.evicted);
            pending.evicted = total;
        }
        ack
    }
    async fn flush(&self, detach: bool) -> Ack {
        self.context.flush_logs().await;
        let mut total = Ack::default();
        loop {
            let mut pending = self.pending.lock().await;
            let ack = harvest(&mut pending);
            total.db_attempted += ack.db_attempted;
            total.db_committed += ack.db_committed;
            total.db_failed += ack.db_failed;
            total.db_pending = ack.db_pending;
            if pending.tickets.is_empty() {
                break;
            }
            drop(pending);
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        let mut pending = self.pending.lock().await;
        let count = if detach {
            if let Some(tracker) = pending.tracker.take() {
                self.context.finish_tracker(tracker).await
            } else {
                None
            }
        } else {
            pending
                .tracker
                .as_ref()
                .map(EventLogEvictionTracker::evicted)
        };
        if let Some(count) = count {
            total.evicted = count.saturating_sub(pending.evicted);
            pending.evicted = count;
        }
        total
    }
}
impl Adapter {
    pub fn start(
        self: &Arc<Self>,
        app: AppHandle,
        permit: OwnedSemaphorePermit,
    ) -> Result<Snapshot, String> {
        let run = self.core.admit(seed()?).map_err(str::to_string)?;
        let adapter = self.clone();
        // The transferred registry permit stays with this outer owner through
        // blocking driver join, recovery, final flush, tracker detach and emit.
        let worker = tokio::spawn(async move {
            let _permit = permit;
            let tracker = app
                .state::<AppState>()
                .event_log
                .register_eviction_tracker(s2::EVENT_PREFIX)
                .await;
            let Ok(tracker) = tracker else {
                drop(run);
                let _ = adapter
                    .core
                    .complete_joined(Some("benchmark_s2_delivery_failed"));
                return;
            };
            let sink = Arc::new(Sink {
                context: SinkContext::App(app.clone()),
                core: adapter.core.clone(),
                pending: AsyncMutex::new(Pending {
                    tickets: Vec::with_capacity(s2::CAPACITY),
                    tracker: Some(tracker),
                    evicted: 0,
                }),
            });
            let deliver = sink.clone();
            let flush = sink.clone();
            let runtime = tokio::runtime::Handle::current();
            let mut driver = tokio::task::spawn_blocking(move || {
                runtime.block_on(run.drive(
                    CancellationToken::new(),
                    move |entry| {
                        let sink = deliver.clone();
                        async move { sink.delivery(entry).await }
                    },
                    move || {
                        let sink = flush.clone();
                        async move { sink.flush(false).await }
                    },
                ))
            });
            let mut changes = adapter.core.observe();
            let outcome = loop {
                tokio::select! {
                    result=&mut driver=>break result,
                    changed=changes.changed()=>{if changed.is_ok(){let _=app.emit(CHANNEL,BenchmarkS2SnapshotDto::from(adapter.core.snapshot()));}}
                }
            };
            let error = match outcome {
                Ok(error) => error,
                Err(_) => {
                    adapter.core.recover_worker().await;
                    Some("benchmark_s2_worker_failed")
                }
            };
            // Even recovery uses a blocking worker: the existing DB flush body
            // performs synchronous IO and must not block the event-loop thread.
            let final_sink = sink.clone();
            let runtime = tokio::runtime::Handle::current();
            match tokio::task::spawn_blocking(move || runtime.block_on(final_sink.flush(true)))
                .await
            {
                Ok(ack) => adapter.core.apply(ack),
                Err(_) => return, // Cleaning is truthful if cleanup itself cannot join.
            }
            if let Ok(snapshot) = adapter.core.complete_joined(error) {
                let _ = app.emit(CHANNEL, BenchmarkS2SnapshotDto::from(snapshot));
            }
        });
        *lock(&self.worker) = Some(worker);
        Ok(self.core.snapshot())
    }
    pub async fn joined(&self) {
        let worker = lock(&self.worker).take();
        if let Some(worker) = worker {
            let _ = worker.await;
        } else {
            self.core.joined().await;
        }
    }
}
/// Only S2 delays database shutdown. The same 500ms budget is observed, never
/// extended or used to detach already-started storage IO.
pub fn defer_exit(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();
    let Some(adapter) = state.benchmark_s2.get() else {
        return false;
    };
    let phase = adapter.core.snapshot().phase;
    if phase == s2::Phase::NotStarted {
        return false;
    }
    if phase.terminal()
        && lock(&adapter.worker)
            .as_ref()
            .is_none_or(tokio::task::JoinHandle::is_finished)
    {
        return false;
    }
    adapter.core.request_shutdown();
    if !adapter.exit_waiting.swap(true, Ordering::SeqCst) {
        let adapter = adapter.clone();
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let joining = adapter.joined();
            tokio::pin!(joining);
            if tokio::time::timeout(s2::CLEANUP_BUDGET, &mut joining)
                .await
                .is_err()
            {
                joining.await;
            }
            // A cleanup panic remains Cleaning; it cannot authorize DB shutdown.
            if adapter.core.snapshot().phase.terminal() {
                app.exit(0);
            }
        });
    }
    true
}
#[cfg(test)]
mod tests;
