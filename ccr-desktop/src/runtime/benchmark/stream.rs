//! One retained, explicitly started S2 stream. No startup work is scheduled here.
use super::Layout;
use crate::runtime::Services;
use ccr_app::{
    benchmark::{
        Seed,
        s2::{self, Ack, Owner, Snapshot},
    },
    runtime::{Completion, EnvironmentId, RequestKey},
};
use std::sync::{Arc, Mutex, MutexGuard};
fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
#[derive(Default)]
pub struct Adapter {
    pub owner: Arc<Owner>,
    cancellation: Mutex<Option<ccr_app::runtime::CancellationHandle>>,
}
fn seed(layout: &Layout) -> Seed {
    Seed {
        root: layout.root.clone(),
        id: layout.id.clone(),
        scenario: layout.scenario.clone(),
        logical_sha256: layout.logical_sha256.clone(),
        as_of: layout.as_of,
        timezone: layout.timezone,
        profile_count: layout.profile_count,
        session_count: layout.session_count,
        usage_count: layout.usage_count,
    }
}
impl Services {
    pub fn has_benchmark_s2(&self) -> bool {
        self.source
            .benchmark()
            .is_some_and(|layout| layout.scenario == "S2" && layout.id == "benchmark-s2")
    }
    fn s2_adapter(&self) -> Result<&Arc<Adapter>, &'static str> {
        if !self.has_benchmark_s2() {
            return Err("benchmark_s2_unavailable");
        }
        Ok(self
            .benchmark_s2
            .get_or_init(|| Arc::new(Adapter::default())))
    }
    pub fn benchmark_s2_status(&self) -> Result<Snapshot, &'static str> {
        Ok(self.s2_adapter()?.owner.snapshot())
    }
    pub fn benchmark_s2_observe(
        &self,
    ) -> Result<tokio::sync::watch::Receiver<Snapshot>, &'static str> {
        Ok(self.s2_adapter()?.owner.observe())
    }
    pub fn benchmark_s2_cancel(&self, id: &str) -> Result<Snapshot, &'static str> {
        let adapter = self.s2_adapter()?;
        let snapshot = adapter.owner.request_cancel(id)?;
        if !snapshot.phase.terminal()
            && let Some(handle) = lock(&adapter.cancellation).as_ref()
        {
            handle.request_cancel();
        }
        Ok(snapshot)
    }
    pub fn benchmark_s2_start(&self, key: RequestKey) -> Result<Snapshot, &'static str> {
        if !matches!(key.environment, EnvironmentId::Local(_)) {
            return Err("benchmark_s2_unavailable");
        }
        let adapter = self.s2_adapter()?;
        let layout = self.source.benchmark().ok_or("benchmark_s2_unavailable")?;
        let run = adapter.owner.admit(&seed(layout))?;
        if self.monitoring.track_evictions(s2::EVENT_PREFIX).is_err() {
            drop(run);
            return adapter
                .owner
                .complete_joined(Some("benchmark_s2_delivery_failed"));
        }
        let queued = Arc::new(Mutex::new(Some(run)));
        let work_run = queued.clone();
        let outcome = Arc::new(Mutex::new(None));
        let work_outcome = outcome.clone();
        let monitoring = self.monitoring.clone();
        let sink = monitoring.clone();
        let source = self.source.clone();
        let owner = adapter.owner.clone();
        let admitted = self.executor.job_with_finalizer(
            key,
            move |cancel, _| {
                let Some(run) = lock(&work_run).take() else {
                    return Completion::<(), ()>::Failed(());
                };
                let flush = sink.clone();
                let result = tokio::runtime::Handle::current().block_on(run.drive(
                    cancel,
                    move |entry| {
                        let receipt = sink.record_checked(&source, entry);
                        let ack = Ack {
                            ring: Some(true),
                            evicted: sink.eviction_delta(false),
                            db_attempted: u64::from(receipt.attempted),
                            db_committed: u64::from(receipt.persisted.is_ok()),
                            db_failed: u64::from(receipt.persisted.is_err()),
                            ..Default::default()
                        };
                        async move { ack }
                    },
                    move || {
                        let ack = Ack {
                            evicted: flush.eviction_delta(false),
                            ..Default::default()
                        };
                        async move { ack }
                    },
                ));
                *lock(&work_outcome) = Some(result);
                Completion::Success(())
            },
            move || {
                // The executor has joined work before this blocking finalizer runs.
                let unstarted = lock(&queued).take();
                let never_started = unstarted.is_some();
                drop(unstarted);
                let result = lock(&outcome).take();
                if result.is_none() {
                    tokio::runtime::Handle::current().block_on(owner.recover_worker());
                }
                owner.apply(Ack {
                    evicted: monitoring.eviction_delta(true),
                    ..Default::default()
                });
                let _ = owner.complete_joined(result.unwrap_or(if never_started {
                    None
                } else {
                    Some("benchmark_s2_worker_failed")
                }));
            },
        );
        match admitted {
            Ok(subscription) => {
                *lock(&adapter.cancellation) = Some(subscription.cancellation_handle())
            }
            Err(_) => {
                adapter.owner.apply(Ack {
                    evicted: self.monitoring.eviction_delta(true),
                    ..Default::default()
                });
                let _ = adapter
                    .owner
                    .complete_joined(Some("benchmark_s2_runtime_admission_failed"));
                return Err("benchmark_s2_runtime_admission_failed");
            }
        }
        Ok(adapter.owner.snapshot())
    }
}
#[cfg(test)]
pub(crate) mod tests;
