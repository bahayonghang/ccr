//! Typed renderer-safe S2 views. Core state and persistence types never go on the wire.
use ccr_app::benchmark::s2;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/benchmark/")]
pub enum BenchmarkS2PhaseDto {
    NotStarted,
    Queued,
    Running,
    Cleaning,
    Completed,
    Cancelled,
    Failed,
}
impl From<s2::Phase> for BenchmarkS2PhaseDto {
    fn from(value: s2::Phase) -> Self {
        match value {
            s2::Phase::NotStarted => Self::NotStarted,
            s2::Phase::Queued => Self::Queued,
            s2::Phase::Running => Self::Running,
            s2::Phase::Cleaning => Self::Cleaning,
            s2::Phase::Completed => Self::Completed,
            s2::Phase::Cancelled => Self::Cancelled,
            s2::Phase::Failed => Self::Failed,
        }
    }
}
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/benchmark/")]
pub struct BenchmarkS2SnapshotDto {
    pub phase: BenchmarkS2PhaseDto,
    pub run_id: Option<String>,
    pub fixture_id: Option<String>,
    pub logical_sha256: Option<String>,
    pub report_as_of_utc: Option<String>,
    #[ts(as = "f64")]
    pub revision: u64,
    #[ts(as = "f64")]
    pub requested_rate: u64,
    #[ts(as = "f64")]
    pub requested_duration_ms: u64,
    #[ts(as = "f64")]
    pub requested_slots: u64,
    pub queue_capacity: usize,
    #[ts(as = "f64")]
    pub scheduled_slots: u64,
    #[ts(as = "f64")]
    pub attempted: u64,
    #[ts(as = "f64")]
    pub scheduler_skipped: u64,
    #[ts(as = "f64")]
    pub queue_accepted: u64,
    #[ts(as = "f64")]
    pub queue_dropped: u64,
    #[ts(as = "f64")]
    pub queue_abandoned: u64,
    #[ts(as = "f64")]
    pub queue_pending: u64,
    #[ts(as = "f64")]
    pub ring_accepted: u64,
    #[ts(as = "f64")]
    pub ring_rejected: u64,
    #[ts(as = "f64")]
    pub ring_evicted: u64,
    #[ts(as = "f64")]
    pub db_attempted: u64,
    #[ts(as = "f64")]
    pub db_committed: u64,
    #[ts(as = "f64")]
    pub db_failed: u64,
    #[ts(as = "f64")]
    pub db_pending: u64,
    #[ts(as = "f64")]
    pub emit_attempted: u64,
    #[ts(as = "f64")]
    pub emit_failed: u64,
    #[ts(as = "f64")]
    pub unacknowledged: u64,
    pub elapsed_ms: Option<f64>,
    pub source_elapsed_ms: Option<f64>,
    pub actual_attempted_per_second: Option<f64>,
    pub actual_delivered_per_second: Option<f64>,
    pub actual_committed_per_second: Option<f64>,
    pub cleanup_over_budget: bool,
    pub failure_code: Option<String>,
}
impl From<s2::Snapshot> for BenchmarkS2SnapshotDto {
    fn from(value: s2::Snapshot) -> Self {
        Self {
            phase: value.phase.into(),
            run_id: value.run_id,
            fixture_id: value.fixture_id,
            logical_sha256: value.logical_sha256,
            report_as_of_utc: value.report_as_of_utc,
            revision: value.revision,
            requested_rate: value.requested_rate,
            requested_duration_ms: value.requested_duration_ms,
            requested_slots: value.requested_slots,
            queue_capacity: value.queue_capacity,
            scheduled_slots: value.scheduled_slots,
            attempted: value.attempted,
            scheduler_skipped: value.scheduler_skipped,
            queue_accepted: value.queue_accepted,
            queue_dropped: value.queue_dropped,
            queue_abandoned: value.queue_abandoned,
            queue_pending: value.queue_pending,
            ring_accepted: value.ring_accepted,
            ring_rejected: value.ring_rejected,
            ring_evicted: value.ring_evicted,
            db_attempted: value.db_attempted,
            db_committed: value.db_committed,
            db_failed: value.db_failed,
            db_pending: value.db_pending,
            emit_attempted: value.emit_attempted,
            emit_failed: value.emit_failed,
            unacknowledged: value.unacknowledged,
            elapsed_ms: value.elapsed_ms,
            source_elapsed_ms: value.source_elapsed_ms,
            actual_attempted_per_second: value.actual_attempted_per_second,
            actual_delivered_per_second: value.actual_delivered_per_second,
            actual_committed_per_second: value.actual_committed_per_second,
            cleanup_over_budget: value.cleanup_over_budget,
            failure_code: value.failure_code,
        }
    }
}
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export, export_to = "../../src/types/generated/benchmark/")]
pub struct BenchmarkS2CancelRequest {
    pub run_id: String,
}
