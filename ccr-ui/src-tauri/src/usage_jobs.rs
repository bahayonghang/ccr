use crate::llmusage_adapter::error::LlmusageAdapterError;
use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

use crate::commands::usage::{UsageImportResultV2, UsageImportSummary};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub enum UsageImportJobStatus {
    Pending,
    Running,
    RecentReady,
    Finished,
    Failed,
    Cancelled,
    CancelRequested,
    TimedOut,
    CleanupFailed,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub enum UsageImportJobStage {
    Queued,
    ImportingRecent,
    ImportingHistory,
    Finished,
    Failed,
    Cancelled,
    CancelRequested,
    TimedOut,
    CleanupFailed,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct UsageImportJobSnapshot {
    pub job_id: String,
    pub status: UsageImportJobStatus,
    pub stage: UsageImportJobStage,
    pub platform_scope: String,
    pub recent_window_days: usize,
    pub files_total: usize,
    pub files_scanned: usize,
    pub files_imported: usize,
    pub records_imported: usize,
    pub records_skipped: usize,
    pub history_cursor_hit: bool,
    pub live_sources: usize,
    pub missing_sources: usize,
    pub deleted_sources: usize,
    pub started_at: String,
    pub updated_at: String,
    // skip_serializing_if 字段在 wire 上是"缺键"而非 null，ts(optional) 生成 `field?: T` 精确表达。
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub recent_ready_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub finished_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_file: Option<String>,
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub error: Option<String>,
    pub results: Vec<UsageImportResultV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub summary: Option<UsageImportSummary>,
}

impl UsageImportJobSnapshot {
    pub fn new(job_id: String, platform_scope: String, recent_window_days: usize) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            job_id,
            status: UsageImportJobStatus::Pending,
            stage: UsageImportJobStage::Queued,
            platform_scope,
            recent_window_days,
            files_total: 0,
            files_scanned: 0,
            files_imported: 0,
            records_imported: 0,
            records_skipped: 0,
            history_cursor_hit: false,
            live_sources: 0,
            missing_sources: 0,
            deleted_sources: 0,
            started_at: now.clone(),
            updated_at: now,
            recent_ready_at: None,
            finished_at: None,
            current_file: None,
            warnings: Vec::new(),
            error: None,
            results: Vec::new(),
            summary: None,
        }
    }

    pub fn mark_running(
        &mut self,
        stage: UsageImportJobStage,
        files_total: usize,
        current_file: Option<String>,
    ) {
        if self.status.is_terminal() || self.status == UsageImportJobStatus::CancelRequested {
            return;
        }
        self.status = UsageImportJobStatus::Running;
        self.stage = stage;
        self.files_total = files_total;
        self.current_file = current_file;
        self.touch();
    }

    pub fn push_warning(&mut self, warning: String) {
        if self.status.is_terminal() {
            return;
        }
        self.warnings.push(warning);
        if self.warnings.len() > 20 {
            let overflow = self.warnings.len() - 20;
            self.warnings.drain(0..overflow);
        }
        self.touch();
    }

    pub fn mark_recent_ready(&mut self, has_history_remaining: bool) {
        if self.status.is_terminal() || self.status == UsageImportJobStatus::CancelRequested {
            return;
        }
        self.status = UsageImportJobStatus::RecentReady;
        self.stage = if has_history_remaining {
            UsageImportJobStage::ImportingHistory
        } else {
            UsageImportJobStage::Finished
        };
        if self.recent_ready_at.is_none() {
            self.recent_ready_at = Some(Utc::now().to_rfc3339());
        }
        self.current_file = None;
        self.touch();
    }

    pub fn mark_finished(
        &mut self,
        results: Vec<UsageImportResultV2>,
        summary: UsageImportSummary,
    ) {
        if self.status.is_terminal() {
            return;
        }
        self.status = UsageImportJobStatus::Finished;
        self.stage = UsageImportJobStage::Finished;
        self.results = results;
        self.summary = Some(summary);
        self.current_file = None;
        let now = Utc::now().to_rfc3339();
        if self.recent_ready_at.is_none() {
            self.recent_ready_at = Some(now.clone());
        }
        self.finished_at = Some(now.clone());
        self.updated_at = now;
    }

    pub fn mark_failed(&mut self, message: impl Into<String>) {
        if self.status.is_terminal() {
            return;
        }
        self.status = UsageImportJobStatus::Failed;
        self.stage = UsageImportJobStage::Failed;
        self.error = Some(message.into());
        self.current_file = None;
        let now = Utc::now().to_rfc3339();
        self.finished_at = Some(now.clone());
        self.updated_at = now;
    }

    pub fn mark_cancelled(&mut self) {
        if self.status.is_terminal() {
            return;
        }
        self.status = UsageImportJobStatus::Cancelled;
        self.stage = UsageImportJobStage::Cancelled;
        self.current_file = None;
        let now = Utc::now().to_rfc3339();
        self.finished_at = Some(now.clone());
        self.updated_at = now;
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now().to_rfc3339();
    }
}

impl UsageImportJobStatus {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Finished | Self::Failed | Self::Cancelled | Self::TimedOut | Self::CleanupFailed
        )
    }
}

pub enum UsageImportCompletion {
    Finished {
        results: Vec<UsageImportResultV2>,
        summary: UsageImportSummary,
        source_count: usize,
    },
    Error(LlmusageAdapterError),
}

struct UsageImportJobRecord {
    snapshot: UsageImportJobSnapshot,
    cancellation: CancellationToken,
}

/// One lock covers admission, cancellation capability, progress, and completion.
#[derive(Default)]
pub struct UsageImportJobs {
    records: HashMap<String, UsageImportJobRecord>,
    active: Option<String>,
}

impl UsageImportJobs {
    pub fn admit(
        &mut self,
        snapshot: UsageImportJobSnapshot,
    ) -> (UsageImportJobSnapshot, Option<CancellationToken>) {
        if let Some(active) = self.active() {
            return (active, None);
        }
        let cancellation = CancellationToken::new();
        self.active = Some(snapshot.job_id.clone());
        self.records.insert(
            snapshot.job_id.clone(),
            UsageImportJobRecord {
                snapshot: snapshot.clone(),
                cancellation: cancellation.clone(),
            },
        );
        (snapshot, Some(cancellation))
    }

    pub fn get(&self, job_id: &str) -> Option<UsageImportJobSnapshot> {
        self.records
            .get(job_id)
            .map(|record| record.snapshot.clone())
    }

    pub fn active(&self) -> Option<UsageImportJobSnapshot> {
        self.active.as_deref().and_then(|job_id| self.get(job_id))
    }

    pub fn request_cancel(&mut self, job_id: &str) -> Option<UsageImportJobSnapshot> {
        let record = self.records.get_mut(job_id)?;
        if !record.snapshot.status.is_terminal() {
            record.cancellation.cancel();
            if record.snapshot.status != UsageImportJobStatus::CancelRequested {
                record.snapshot.status = UsageImportJobStatus::CancelRequested;
                record.snapshot.stage = UsageImportJobStage::CancelRequested;
                record.snapshot.touch();
            }
        }
        Some(record.snapshot.clone())
    }

    pub fn progress<F>(&mut self, job_id: &str, update: F) -> Option<UsageImportJobSnapshot>
    where
        F: FnOnce(&mut UsageImportJobSnapshot),
    {
        let record = self.records.get_mut(job_id)?;
        if record.snapshot.status.is_terminal() || record.cancellation.is_cancelled() {
            return None;
        }
        update(&mut record.snapshot);
        Some(record.snapshot.clone())
    }

    /// Called once by the runner after process exit and bounded cleanup return.
    pub fn complete(
        &mut self,
        job_id: &str,
        completion: UsageImportCompletion,
    ) -> Option<UsageImportJobSnapshot> {
        let record = self.records.get_mut(job_id)?;
        if record.snapshot.status.is_terminal() {
            return None;
        }
        match completion {
            UsageImportCompletion::Finished {
                results,
                summary,
                source_count,
            } => {
                record.snapshot.files_total = record.snapshot.files_total.max(source_count);
                record.snapshot.records_imported = record
                    .snapshot
                    .records_imported
                    .max(summary.imported_records);
                record.snapshot.mark_finished(results, summary);
            }
            UsageImportCompletion::Error(error) => match error {
                LlmusageAdapterError::Cancelled => record.snapshot.mark_cancelled(),
                LlmusageAdapterError::TimedOut => {
                    record.snapshot.mark_failed(error.to_string());
                    record.snapshot.status = UsageImportJobStatus::TimedOut;
                    record.snapshot.stage = UsageImportJobStage::TimedOut;
                }
                LlmusageAdapterError::CleanupFailed(_) => {
                    record.snapshot.mark_failed(error.to_string());
                    record.snapshot.status = UsageImportJobStatus::CleanupFailed;
                    record.snapshot.stage = UsageImportJobStage::CleanupFailed;
                }
                _ => record.snapshot.mark_failed(error.to_string()),
            },
        }
        if self.active.as_deref() == Some(job_id) {
            self.active = None;
        }
        Some(record.snapshot.clone())
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;

    #[test]
    fn cancelled_result_rejects_late_failure() {
        let mut snapshot =
            UsageImportJobSnapshot::new("test-job".to_string(), "all".to_string(), 30);
        snapshot.mark_cancelled();
        let before = serde_json::to_value(&snapshot).unwrap();
        snapshot.mark_failed("late process failure");
        assert_eq!(serde_json::to_value(&snapshot).unwrap(), before);
    }

    #[test]
    fn failed_result_rejects_cancel_and_late_progress() {
        let mut snapshot =
            UsageImportJobSnapshot::new("test-job".to_string(), "all".to_string(), 30);
        snapshot.mark_failed("process failed");
        let before = serde_json::to_value(&snapshot).unwrap();
        snapshot.mark_cancelled();
        snapshot.mark_running(UsageImportJobStage::ImportingRecent, 10, None);
        assert_eq!(serde_json::to_value(&snapshot).unwrap(), before);
    }
    fn snapshot(id: &str) -> UsageImportJobSnapshot {
        UsageImportJobSnapshot::new(id.to_string(), "all".to_string(), 30)
    }

    fn finished() -> UsageImportCompletion {
        UsageImportCompletion::Finished {
            results: Vec::new(),
            source_count: 1,
            summary: UsageImportSummary {
                success_count: 0,
                failure_count: 0,
                imported_records: 0,
                processed_files: 0,
                has_partial: false,
            },
        }
    }

    #[tokio::test]
    async fn cancel_before_runner_start_prevents_spawn_and_keeps_admission() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        let jobs = Arc::new(tokio::sync::Mutex::new(UsageImportJobs::default()));
        let (_, token) = jobs.lock().await.admit(snapshot("first"));
        let barrier = Arc::new(tokio::sync::Barrier::new(2));
        let spawns = Arc::new(AtomicUsize::new(0));
        let runner = tokio::spawn({
            let (jobs, barrier, spawns) = (jobs.clone(), barrier.clone(), spawns.clone());
            async move {
                barrier.wait().await;
                let result =
                    crate::llmusage_adapter::cli::spawn_sync_process(&token.unwrap(), || {
                        spawns.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    });
                jobs.lock()
                    .await
                    .complete("first", UsageImportCompletion::Error(result.unwrap_err()));
            }
        });
        let cancelled = jobs.lock().await.request_cancel("first").unwrap();
        assert_eq!(cancelled.status, UsageImportJobStatus::CancelRequested);
        assert!(cancelled.finished_at.is_none());
        let (existing, new_token) = jobs.lock().await.admit(snapshot("second"));
        assert_eq!(existing.job_id, "first");
        assert!(new_token.is_none());
        assert!(jobs.lock().await.active().is_some());
        barrier.wait().await;
        runner.await.unwrap();
        assert_eq!(spawns.load(Ordering::SeqCst), 0);
        assert_eq!(
            jobs.lock().await.get("first").unwrap().status,
            UsageImportJobStatus::Cancelled
        );
        assert!(jobs.lock().await.active().is_none());
        assert!(jobs.lock().await.admit(snapshot("second")).1.is_some());
    }

    #[test]
    fn all_terminal_results_reject_duplicate_cancel_progress_and_completion() {
        for result in [
            finished(),
            UsageImportCompletion::Error(LlmusageAdapterError::Cancelled),
            UsageImportCompletion::Error(LlmusageAdapterError::TimedOut),
            UsageImportCompletion::Error(LlmusageAdapterError::Cli("failed".into())),
            UsageImportCompletion::Error(LlmusageAdapterError::CleanupFailed("reap".into())),
        ] {
            let mut jobs = UsageImportJobs::default();
            jobs.admit(snapshot("test"));
            let terminal = jobs.complete("test", result).unwrap();
            let before = serde_json::to_value(terminal).unwrap();
            assert_eq!(
                serde_json::to_value(jobs.request_cancel("test")).unwrap(),
                before
            );
            assert!(jobs.progress("test", |job| job.files_total = 100).is_none());
            assert!(jobs.complete("test", finished()).is_none());
            assert_eq!(serde_json::to_value(jobs.get("test")).unwrap(), before);
        }
    }

    #[test]
    fn running_cancel_keeps_token_and_reports_cleanup_failure() {
        let mut jobs = UsageImportJobs::default();
        let (_, token) = jobs.admit(snapshot("test"));
        jobs.progress("test", |job| {
            job.mark_running(UsageImportJobStage::ImportingRecent, 1, None)
        });
        jobs.request_cancel("test");
        assert!(token.unwrap().is_cancelled());
        assert!(jobs.active().is_some());
        assert!(
            jobs.progress("test", |job| job.records_imported = 100)
                .is_none()
        );
        let terminal = jobs
            .complete(
                "test",
                UsageImportCompletion::Error(LlmusageAdapterError::CleanupFailed(
                    "reap failed".into(),
                )),
            )
            .unwrap();
        assert_eq!(terminal.status, UsageImportJobStatus::CleanupFailed);
        assert!(terminal.error.unwrap().contains("reap failed"));
        assert!(jobs.active().is_none());
    }
}
