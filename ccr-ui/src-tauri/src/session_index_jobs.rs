use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionIndexJobStatus {
    Pending,
    Running,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionIndexJobStage {
    Queued,
    Indexing,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionIndexJobSnapshot {
    pub job_id: String,
    pub status: SessionIndexJobStatus,
    pub stage: SessionIndexJobStage,
    pub platforms_total: usize,
    pub platforms_completed: usize,
    pub files_total: u64,
    pub files_scanned: u64,
    pub sessions_added: u64,
    pub sessions_updated: u64,
    pub errors: u64,
    pub started_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_platform: Option<String>,
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl SessionIndexJobSnapshot {
    pub fn new(job_id: String, platforms_total: usize) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            job_id,
            status: SessionIndexJobStatus::Pending,
            stage: SessionIndexJobStage::Queued,
            platforms_total,
            platforms_completed: 0,
            files_total: 0,
            files_scanned: 0,
            sessions_added: 0,
            sessions_updated: 0,
            errors: 0,
            started_at: now.clone(),
            updated_at: now,
            finished_at: None,
            current_platform: None,
            warnings: Vec::new(),
            error: None,
        }
    }

    pub fn mark_running(
        &mut self,
        current_platform: Option<String>,
        files_total: u64,
        files_scanned: u64,
    ) {
        self.status = SessionIndexJobStatus::Running;
        self.stage = SessionIndexJobStage::Indexing;
        self.current_platform = current_platform;
        self.files_total = files_total;
        self.files_scanned = files_scanned;
        self.touch();
    }

    pub fn record_platform_result(
        &mut self,
        files_scanned: u64,
        sessions_added: u64,
        sessions_updated: u64,
        errors: u64,
    ) {
        self.platforms_completed += 1;
        self.files_scanned += files_scanned;
        self.sessions_added += sessions_added;
        self.sessions_updated += sessions_updated;
        self.errors += errors;
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

    pub fn mark_finished(&mut self) {
        self.status = SessionIndexJobStatus::Finished;
        self.stage = SessionIndexJobStage::Finished;
        self.current_platform = None;
        let now = Utc::now().to_rfc3339();
        self.finished_at = Some(now.clone());
        self.updated_at = now;
    }

    pub fn mark_failed(&mut self, message: impl Into<String>) {
        self.status = SessionIndexJobStatus::Failed;
        self.stage = SessionIndexJobStage::Failed;
        self.error = Some(message.into());
        self.current_platform = None;
        let now = Utc::now().to_rfc3339();
        self.finished_at = Some(now.clone());
        self.updated_at = now;
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now().to_rfc3339();
    }
}
