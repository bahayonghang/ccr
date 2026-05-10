use chrono::Utc;
use serde::Serialize;

use crate::commands::usage::{UsageImportResultV2, UsageImportSummary};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UsageImportJobStatus {
    Pending,
    Running,
    RecentReady,
    Finished,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UsageImportJobStage {
    Queued,
    ImportingRecent,
    ImportingHistory,
    Finished,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_ready_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_file: Option<String>,
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub results: Vec<UsageImportResultV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
        self.status = UsageImportJobStatus::Running;
        self.stage = stage;
        self.files_total = files_total;
        self.current_file = current_file;
        self.touch();
    }

    pub fn push_warning(&mut self, warning: String) {
        self.warnings.push(warning);
        if self.warnings.len() > 20 {
            let overflow = self.warnings.len() - 20;
            self.warnings.drain(0..overflow);
        }
        self.touch();
    }

    pub fn mark_recent_ready(&mut self, has_history_remaining: bool) {
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
        self.status = UsageImportJobStatus::Failed;
        self.stage = UsageImportJobStage::Failed;
        self.error = Some(message.into());
        self.current_file = None;
        let now = Utc::now().to_rfc3339();
        self.finished_at = Some(now.clone());
        self.updated_at = now;
    }

    pub fn mark_cancelled(&mut self) {
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
