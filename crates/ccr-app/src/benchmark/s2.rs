//! The one fixed S2 workload. Report timestamps are data; pacing uses real Instant.
use super::Seed;
use ccr_types::{MonitoringEntry, MonitoringLevel};
use std::{
    future::Future,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::watch;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub const RATE: u64 = 1000;
pub const DURATION_MS: u64 = 60_000;
pub const SLOTS: u64 = 60_000;
pub const CAPACITY: usize = 128;
pub const CLEANUP_BUDGET: Duration = Duration::from_millis(500);
pub const EVENT_PREFIX: &str = "benchmark-s2-v1-";
static USED: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
const ACTIVE_SLOTS: u64 = 300;
#[cfg(not(test))]
const ACTIVE_SLOTS: u64 = SLOTS;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    NotStarted,
    Queued,
    Running,
    Cleaning,
    Completed,
    Cancelled,
    Failed,
}
impl Phase {
    pub fn terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Failed)
    }
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub phase: Phase,
    pub run_id: Option<String>,
    pub fixture_id: Option<String>,
    pub logical_sha256: Option<String>,
    pub report_as_of_utc: Option<String>,
    pub revision: u64,
    pub requested_rate: u64,
    pub requested_duration_ms: u64,
    pub requested_slots: u64,
    pub queue_capacity: usize,
    pub scheduled_slots: u64,
    pub attempted: u64,
    pub scheduler_skipped: u64,
    pub queue_accepted: u64,
    pub queue_dropped: u64,
    pub queue_abandoned: u64,
    pub queue_pending: u64,
    pub ring_accepted: u64,
    pub ring_rejected: u64,
    pub ring_evicted: u64,
    pub db_attempted: u64,
    pub db_committed: u64,
    pub db_failed: u64,
    pub db_pending: u64,
    pub emit_attempted: u64,
    pub emit_failed: u64,
    pub unacknowledged: u64,
    pub elapsed_ms: Option<f64>,
    pub source_elapsed_ms: Option<f64>,
    pub actual_attempted_per_second: Option<f64>,
    pub actual_delivered_per_second: Option<f64>,
    pub actual_committed_per_second: Option<f64>,
    pub cleanup_over_budget: bool,
    pub failure_code: Option<String>,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            phase: Phase::NotStarted,
            run_id: None,
            fixture_id: None,
            logical_sha256: None,
            report_as_of_utc: None,
            revision: 0,
            requested_rate: RATE,
            requested_duration_ms: DURATION_MS,
            requested_slots: SLOTS,
            queue_capacity: CAPACITY,
            scheduled_slots: 0,
            attempted: 0,
            scheduler_skipped: 0,
            queue_accepted: 0,
            queue_dropped: 0,
            queue_abandoned: 0,
            queue_pending: 0,
            ring_accepted: 0,
            ring_rejected: 0,
            ring_evicted: 0,
            db_attempted: 0,
            db_committed: 0,
            db_failed: 0,
            db_pending: 0,
            emit_attempted: 0,
            emit_failed: 0,
            unacknowledged: 0,
            elapsed_ms: None,
            source_elapsed_ms: None,
            actual_attempted_per_second: None,
            actual_delivered_per_second: None,
            actual_committed_per_second: None,
            cleanup_over_budget: false,
            failure_code: None,
        }
    }
}
/// Counts acknowledged at their actual stage, never a unit call counted as storage.
#[derive(Default)]
pub struct Ack {
    pub ring: Option<bool>,
    pub evicted: u64,
    pub db_attempted: u64,
    pub db_committed: u64,
    pub db_failed: u64,
    pub db_pending: u64,
    pub emit: Option<bool>,
}
struct State {
    snapshot: Snapshot,
    started: Option<Instant>,
    source_stopped: Option<Instant>,
    finished: Option<Instant>,
    last_publish: Instant,
    dequeued: u64,
    cancelled: bool,
}
pub struct Owner {
    state: Mutex<State>,
    updates: watch::Sender<Snapshot>,
    cancel: CancellationToken,
    heartbeat_stop: CancellationToken,
    producer_tasks: TaskTracker,
}
fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
impl Default for Owner {
    fn default() -> Self {
        Self {
            state: Mutex::new(State {
                snapshot: Snapshot::default(),
                started: None,
                source_stopped: None,
                finished: None,
                last_publish: Instant::now(),
                dequeued: 0,
                cancelled: false,
            }),
            updates: watch::channel(Snapshot::default()).0,
            cancel: CancellationToken::new(),
            heartbeat_stop: CancellationToken::new(),
            producer_tasks: TaskTracker::new(),
        }
    }
}
fn projected(state: &State) -> Snapshot {
    let mut out = state.snapshot.clone();
    let now = state.finished.unwrap_or_else(Instant::now);
    out.queue_pending = out
        .queue_accepted
        .saturating_sub(state.dequeued)
        .saturating_sub(out.queue_abandoned);
    if let Some(start) = state.started {
        let total = now.saturating_duration_since(start).as_secs_f64();
        let source = state
            .source_stopped
            .unwrap_or(now)
            .saturating_duration_since(start)
            .as_secs_f64();
        out.elapsed_ms = Some(total * 1000.0);
        out.source_elapsed_ms = Some(source * 1000.0);
        out.actual_attempted_per_second = (source > 0.0).then(|| out.attempted as f64 / source);
        out.actual_delivered_per_second = (total > 0.0).then(|| out.ring_accepted as f64 / total);
        out.actual_committed_per_second = (total > 0.0).then(|| out.db_committed as f64 / total);
    }
    out.cleanup_over_budget |= state
        .source_stopped
        .is_some_and(|at| now.saturating_duration_since(at) > CLEANUP_BUDGET);
    out
}
impl Owner {
    pub fn snapshot(&self) -> Snapshot {
        let mut state = lock(&self.state);
        if state.snapshot.phase.terminal() || state.snapshot.phase == Phase::NotStarted {
            return state.snapshot.clone();
        }
        state.snapshot.revision += 1;
        projected(&state)
    }
    pub fn observe(&self) -> watch::Receiver<Snapshot> {
        self.updates.subscribe()
    }
    fn update(&self, force: bool, change: impl FnOnce(&mut State)) {
        let mut state = lock(&self.state);
        if state.snapshot.phase.terminal() {
            return;
        }
        change(&mut state);
        if force || state.last_publish.elapsed() >= crate::runtime::PROGRESS_INTERVAL {
            state.last_publish = Instant::now();
            state.snapshot.revision += 1;
            let snapshot = projected(&state);
            if snapshot.phase.terminal() {
                state.snapshot = snapshot.clone();
            }
            self.updates.send_replace(snapshot);
        }
    }
    pub fn admit(self: &Arc<Self>, seed: &Seed) -> Result<Run, &'static str> {
        self.admit_once(seed, &USED)
    }
    fn admit_once(self: &Arc<Self>, seed: &Seed, used: &AtomicBool) -> Result<Run, &'static str> {
        if seed.scenario != "S2" || seed.id != "benchmark-s2" {
            return Err("benchmark_s2_unavailable");
        }
        if used
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err("benchmark_s2_process_used");
        }
        self.update(true, |state| {
            state.snapshot.phase = Phase::Queued;
            state.snapshot.run_id = Some(uuid::Uuid::new_v4().to_string());
            state.snapshot.fixture_id = Some(seed.id.clone());
            state.snapshot.logical_sha256 = Some(seed.logical_sha256.clone());
            state.snapshot.report_as_of_utc = Some(seed.as_of.to_rfc3339());
        });
        let (sender, receiver) = async_channel::bounded(CAPACITY);
        Ok(Run {
            owner: self.clone(),
            sender: Some(sender),
            receiver,
            as_of: seed.as_of,
            driven: false,
        })
    }
    pub fn request_cancel(&self, run_id: &str) -> Result<Snapshot, &'static str> {
        if self.snapshot().run_id.as_deref() != Some(run_id) {
            return Err("benchmark_s2_wrong_run");
        }
        if !self.snapshot().phase.terminal() {
            self.cancel.cancel();
            self.update(true, |state| state.cancelled = true);
        }
        Ok(self.snapshot())
    }
    pub fn request_shutdown(&self) {
        if self.snapshot().run_id.is_some() && !self.snapshot().phase.terminal() {
            self.cancel.cancel();
            self.update(true, |state| state.cancelled = true);
        }
    }
    pub async fn joined(&self) {
        let mut updates = self.observe();
        while !updates.borrow().phase.terminal() && updates.borrow().phase != Phase::NotStarted {
            if updates.changed().await.is_err() {
                break;
            }
        }
    }
    pub fn apply(&self, ack: Ack) {
        self.update(false, |state| {
            let out = &mut state.snapshot;
            if let Some(accepted) = ack.ring {
                if accepted {
                    out.ring_accepted += 1;
                } else {
                    out.ring_rejected += 1;
                }
            }
            out.ring_evicted += ack.evicted;
            out.db_attempted += ack.db_attempted;
            out.db_committed += ack.db_committed;
            out.db_failed += ack.db_failed;
            out.db_pending = ack.db_pending;
            if let Some(emitted) = ack.emit {
                out.emit_attempted += 1;
                if !emitted {
                    out.emit_failed += 1;
                }
            }
        });
    }
    fn stopped(&self) {
        self.update(true, |state| {
            state.source_stopped.get_or_insert_with(Instant::now);
            state.snapshot.phase = Phase::Cleaning;
        });
    }
    fn finish(&self, error: Option<&str>) {
        self.update(true, |state| {
            state.finished = Some(Instant::now());
            let over = state
                .source_stopped
                .is_some_and(|at| at.elapsed() > CLEANUP_BUDGET);
            state.snapshot.cleanup_over_budget |= over;
            state.snapshot.failure_code = error.map(str::to_string).or_else(|| {
                if over {
                    Some("benchmark_s2_cleanup_over_budget".into())
                } else if state.snapshot.db_failed > 0 || state.snapshot.emit_failed > 0 {
                    Some("benchmark_s2_delivery_failed".into())
                } else {
                    None
                }
            });
            state.snapshot.phase = if state.snapshot.failure_code.is_some() {
                Phase::Failed
            } else if state.cancelled {
                Phase::Cancelled
            } else {
                Phase::Completed
            };
        });
    }
    /// Called after a driver join failed. The producer is pure async and tracked;
    /// no terminal is published until it and the adapter's final IO are joined.
    pub async fn recover_worker(&self) {
        self.request_shutdown();
        self.heartbeat_stop.cancel();
        self.producer_tasks.close();
        self.producer_tasks.wait().await;
        self.stopped();
        self.update(true, |state| {
            state.snapshot.queue_abandoned =
                state.snapshot.queue_accepted.saturating_sub(state.dequeued);
            state.snapshot.unacknowledged = state
                .dequeued
                .saturating_sub(state.snapshot.ring_accepted + state.snapshot.ring_rejected);
        });
    }
    pub fn complete_joined(&self, error: Option<&str>) -> Result<Snapshot, &'static str> {
        let snapshot = self.snapshot();
        if snapshot.phase.terminal() {
            return Ok(snapshot);
        }
        if snapshot.phase != Phase::Cleaning
            || !self.producer_tasks.is_empty()
            || snapshot.db_pending != 0
        {
            return Err("benchmark_s2_cleanup_incomplete");
        }
        self.finish(error);
        Ok(self.snapshot())
    }
}
pub fn owns_event(id: &str) -> bool {
    id.strip_prefix(EVENT_PREFIX).is_some_and(|tail| {
        tail.len() == 5
            && tail.bytes().all(|b| b.is_ascii_digit())
            && tail.parse::<u64>().is_ok_and(|slot| slot < SLOTS)
    })
}
pub fn entry(as_of: chrono::DateTime<chrono::Utc>, slot: u64) -> MonitoringEntry {
    let mut entry = MonitoringEntry::new(
        MonitoringLevel::Info,
        "benchmark",
        "benchmark.s2.entry",
        "synthetic",
        "Synthetic S2 monitoring event",
    );
    entry.id = format!("{EVENT_PREFIX}{slot:05}");
    entry.timestamp = (as_of + chrono::Duration::milliseconds(slot as i64)).to_rfc3339();
    entry
}
pub struct Run {
    owner: Arc<Owner>,
    sender: Option<async_channel::Sender<MonitoringEntry>>,
    receiver: async_channel::Receiver<MonitoringEntry>,
    as_of: chrono::DateTime<chrono::Utc>,
    driven: bool,
}
impl Drop for Run {
    fn drop(&mut self) {
        if !self.driven {
            self.owner.stopped();
            self.owner.update(true, |state| {
                state.snapshot.failure_code = Some("benchmark_s2_not_started".into())
            });
        }
    }
}
impl Run {
    pub async fn drive<D, DF, F, FF>(
        mut self,
        external: CancellationToken,
        mut deliver: D,
        mut flush: F,
    ) -> Option<&'static str>
    where
        D: FnMut(MonitoringEntry) -> DF,
        DF: Future<Output = Ack>,
        F: FnMut() -> FF,
        FF: Future<Output = Ack>,
    {
        self.driven = true;
        let Some(sender) = self.sender.take() else {
            return Some("benchmark_s2_worker_failed");
        };
        let owner = self.owner.clone();
        let as_of = self.as_of;
        let source = owner.clone();
        let heartbeat_owner = owner.clone();
        let heartbeat=owner.producer_tasks.spawn(async move {
            loop {tokio::select! {
                _=heartbeat_owner.heartbeat_stop.cancelled()=>break,
                _=tokio::time::sleep(crate::runtime::PROGRESS_INTERVAL)=>heartbeat_owner.update(false,|_|{}),
            }}
        });
        let producer=owner.producer_tasks.spawn(async move {
            let start=Instant::now();source.update(true,|state|{state.started=Some(start);state.snapshot.phase=Phase::Running;});
            let mut slot=0;
            while slot<ACTIVE_SLOTS {
                let deadline=start+Duration::from_millis(slot);
                tokio::select! {biased;
                    _=source.cancel.cancelled()=>{source.update(true,|state|state.cancelled=true);break;}
                    _=external.cancelled()=>{source.update(true,|state|state.cancelled=true);source.cancel.cancel();break;}
                    _=tokio::time::sleep_until(tokio::time::Instant::from_std(deadline))=>{}
                }
                let elapsed=start.elapsed().as_millis() as u64;
                if elapsed>=ACTIVE_SLOTS {source.update(false,|state|{state.snapshot.scheduler_skipped+=ACTIVE_SLOTS-slot;state.snapshot.scheduled_slots=ACTIVE_SLOTS;});break;}
                let due=elapsed;
                if due>slot {source.update(false,|state|state.snapshot.scheduler_skipped+=due-slot);slot=due;}
                source.update(false,|state|{state.snapshot.scheduled_slots=slot+1;state.snapshot.attempted+=1;});
                match sender.try_send(entry(as_of,slot)) {
                    Ok(())=>source.update(false,|state|state.snapshot.queue_accepted+=1),
                    Err(async_channel::TrySendError::Full(_))=>source.update(false,|state|state.snapshot.queue_dropped+=1),
                    Err(async_channel::TrySendError::Closed(_))=>{source.update(false,|state|state.snapshot.queue_dropped+=1);break;}
                }
                slot+=1;
            }
            source.stopped();
        });
        while let Ok(entry) = self.receiver.recv().await {
            if lock(&owner.state)
                .source_stopped
                .is_some_and(|at| at.elapsed() >= CLEANUP_BUDGET)
            {
                let mut abandoned = 1;
                while self.receiver.try_recv().is_ok() {
                    abandoned += 1;
                }
                owner.update(true, |state| state.snapshot.queue_abandoned += abandoned);
                break;
            }
            owner.update(false, |state| state.dequeued += 1);
            owner.apply(deliver(entry).await);
        }
        let failed = producer.await.is_err();
        owner.stopped();
        owner.apply(flush().await);
        owner.heartbeat_stop.cancel();
        let _ = heartbeat.await;
        owner.producer_tasks.close();
        owner.producer_tasks.wait().await;
        failed.then_some("benchmark_s2_producer_failed")
    }
}

#[cfg(test)]
mod tests;
