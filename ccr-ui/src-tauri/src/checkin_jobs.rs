use chrono::Utc;
use serde::Serialize;

use ccr_checkin::services::checkin_service::CheckinExecutionResult;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckinJobStatus {
    Pending,
    Running,
    Finished,
    TimedOut,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckinJobLogStatus {
    Pending,
    Processing,
    Success,
    AlreadyCheckedIn,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckinJobLogEntry {
    pub account_id: String,
    pub account_name: String,
    pub provider_name: String,
    pub status: CheckinJobLogStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<f64>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CheckinJobSummary {
    pub total: usize,
    pub success: usize,
    pub already_checked_in: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckinJobSnapshot {
    pub job_id: String,
    pub status: CheckinJobStatus,
    pub total: usize,
    pub completed: usize,
    pub current_account_name: String,
    pub logs: Vec<CheckinJobLogEntry>,
    pub results: Vec<CheckinExecutionResult>,
    pub summary: CheckinJobSummary,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
}

/// 签到任务增量事件载荷。
///
/// 用于 progress 事件 emit，仅携带相对上一次快照的变化，避免 IPC 流量随批量账号规模 O(N²) 增长。
/// 前端通过 job_id 将 delta 合并到本地完整快照。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckinJobDelta {
    pub job_id: String,
    pub status: CheckinJobStatus,
    pub completed: usize,
    pub total: usize,
    pub current_account_name: String,
    pub summary: CheckinJobSummary,
    /// 本次 tick 相对上一次变化的 log 条目（按 account_id 定位）
    pub changed_logs: Vec<CheckinJobLogEntry>,
    /// 本次新增的 result（首次出现）
    pub new_results: Vec<CheckinExecutionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
}

impl CheckinJobLogEntry {
    pub fn pending(account_id: String, account_name: String, provider_name: String) -> Self {
        Self {
            account_id,
            account_name,
            provider_name,
            status: CheckinJobLogStatus::Pending,
            message: None,
            error_code: None,
            reward: None,
            balance: None,
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

impl CheckinJobSnapshot {
    pub fn new(job_id: String, logs: Vec<CheckinJobLogEntry>) -> Self {
        let total = logs.len();
        Self {
            job_id,
            status: CheckinJobStatus::Pending,
            total,
            completed: 0,
            current_account_name: String::new(),
            results: Vec::with_capacity(total),
            summary: CheckinJobSummary {
                total,
                ..CheckinJobSummary::default()
            },
            logs,
            started_at: Utc::now().to_rfc3339(),
            finished_at: None,
        }
    }

    pub fn mark_processing(&mut self, account_id: &str) {
        if matches!(
            self.status,
            CheckinJobStatus::Finished | CheckinJobStatus::TimedOut
        ) {
            return;
        }

        self.status = CheckinJobStatus::Running;
        if let Some(log) = self
            .logs
            .iter_mut()
            .find(|entry| entry.account_id == account_id)
        {
            log.status = CheckinJobLogStatus::Processing;
            log.timestamp = Utc::now().to_rfc3339();
            self.current_account_name = log.account_name.clone();
        }
    }

    pub fn apply_result(&mut self, result: CheckinExecutionResult) {
        if matches!(self.status, CheckinJobStatus::TimedOut) {
            return;
        }

        if self
            .results
            .iter()
            .any(|existing| existing.account_id == result.account_id)
        {
            return;
        }

        if let Some(log) = self
            .logs
            .iter_mut()
            .find(|entry| entry.account_id == result.account_id)
        {
            log.status = match result.status {
                ccr_checkin::models::checkin::CheckinStatus::Success => CheckinJobLogStatus::Success,
                ccr_checkin::models::checkin::CheckinStatus::AlreadyCheckedIn => {
                    CheckinJobLogStatus::AlreadyCheckedIn
                }
                ccr_checkin::models::checkin::CheckinStatus::Failed => CheckinJobLogStatus::Failed,
            };
            log.message = result.message.clone();
            log.error_code = result.error_code.clone();
            log.reward = result.reward.clone();
            log.balance = result.balance;
            log.timestamp = Utc::now().to_rfc3339();
        }

        self.completed += 1;
        match result.status {
            ccr_checkin::models::checkin::CheckinStatus::Success => self.summary.success += 1,
            ccr_checkin::models::checkin::CheckinStatus::AlreadyCheckedIn => {
                self.summary.already_checked_in += 1
            }
            ccr_checkin::models::checkin::CheckinStatus::Failed => self.summary.failed += 1,
        }
        self.results.push(result);

        if self.completed >= self.total {
            self.mark_finished(CheckinJobStatus::Finished);
        }
    }

    pub fn mark_finished(&mut self, status: CheckinJobStatus) {
        self.status = status;
        self.current_account_name.clear();
        if self.finished_at.is_none() {
            self.finished_at = Some(Utc::now().to_rfc3339());
        }
    }

    pub fn mark_pending_failed(&mut self, message: &str) {
        if matches!(self.status, CheckinJobStatus::TimedOut) {
            return;
        }

        let now = Utc::now().to_rfc3339();
        let mut failure_results = Vec::new();

        for log in &mut self.logs {
            if matches!(
                log.status,
                CheckinJobLogStatus::Pending | CheckinJobLogStatus::Processing
            ) {
                log.status = CheckinJobLogStatus::Failed;
                log.message = Some(message.to_string());
                log.error_code = Some("task_error".to_string());
                log.timestamp = now.clone();
                failure_results.push(CheckinExecutionResult {
                    account_id: log.account_id.clone(),
                    account_name: log.account_name.clone(),
                    provider_name: log.provider_name.clone(),
                    status: ccr_checkin::models::checkin::CheckinStatus::Failed,
                    message: Some(message.to_string()),
                    error_code: Some("task_error".to_string()),
                    reward: None,
                    balance: None,
                });
            }
        }

        self.summary.failed += failure_results.len();
        self.results.extend(failure_results);
        self.completed = self.results.len().min(self.total);
        if self.completed >= self.total {
            self.mark_finished(CheckinJobStatus::Finished);
        }
    }

    pub fn mark_timed_out(&mut self) {
        if matches!(self.status, CheckinJobStatus::TimedOut) {
            return;
        }

        let now = Utc::now().to_rfc3339();
        let mut timeout_results = Vec::new();

        for log in &mut self.logs {
            if matches!(
                log.status,
                CheckinJobLogStatus::Pending | CheckinJobLogStatus::Processing
            ) {
                log.status = CheckinJobLogStatus::Failed;
                log.message = Some("签到超时".to_string());
                log.error_code = Some("timeout".to_string());
                log.timestamp = now.clone();
                timeout_results.push(CheckinExecutionResult {
                    account_id: log.account_id.clone(),
                    account_name: log.account_name.clone(),
                    provider_name: log.provider_name.clone(),
                    status: ccr_checkin::models::checkin::CheckinStatus::Failed,
                    message: Some("签到超时".to_string()),
                    error_code: Some("timeout".to_string()),
                    reward: None,
                    balance: None,
                });
            }
        }

        self.summary.failed += timeout_results.len();
        self.results.extend(timeout_results);
        self.completed = self.total;
        self.mark_finished(CheckinJobStatus::TimedOut);
    }
}
