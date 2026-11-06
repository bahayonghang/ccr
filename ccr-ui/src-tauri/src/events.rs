//! Tauri Event 系统，使用原生事件机制替代 WebSocket。
//!
//! 后端通过 `app_handle.emit(event, payload)` 广播事件，
//! 前端通过 `listen(event, handler)` 订阅事件。

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use ccr_types::MonitoringEntry;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// 应用事件联合枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AppEvent {
    /// 签到完成事件
    CheckinCompleted(CheckinEventPayload),
    /// 签到失败事件
    CheckinFailed(CheckinEventPayload),
    /// 同步状态变更事件
    SyncStatusChanged(SyncEventPayload),
    /// 任务进度更新事件
    TaskProgress(TaskProgressPayload),
    /// 通知事件
    Notification(NotificationPayload),
    /// 环境切换/状态变更事件（Local/WSL/SSH）
    EnvironmentChanged(EnvironmentEventPayload),
    /// 用量导入完成事件
    UsageImportCompleted(UsageImportPayload),
    /// 监控日志事件
    Monitoring(MonitoringEntry),
}

/// 签到事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinEventPayload {
    pub account_id: i64,
    pub provider_name: String,
    pub success: bool,
    pub message: String,
}

/// 同步事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEventPayload {
    pub operation: String,
    pub status: String,
    pub message: String,
}

/// 任务进度载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressPayload {
    pub task_id: String,
    pub progress: f64,
    pub message: String,
}

/// 通知载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub level: NotificationLevel,
    pub title: String,
    pub message: String,
}

/// 通知级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// 环境事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentEventPayload {
    pub env_id: String,
    pub env_type: String,
    pub status: String,
}

/// 用量导入事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageImportPayload {
    pub imported_count: usize,
    pub platform: String,
}

/// 事件日志统计信息
#[derive(Debug, Clone, Serialize)]
pub struct EventLogStats {
    pub entries: usize,
    pub total_size_bytes: usize,
    pub capacity: usize,
    pub max_event_size_bytes: usize,
    pub max_total_size_bytes: usize,
    pub dropped_events: u64,
}

/// Result of this push only; eviction IDs identify monitoring entries, not ring IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventLogPushReceipt {
    pub accepted: bool,
    pub oversized: bool,
    pub evicted: usize,
    pub evicted_monitoring_ids: Vec<String>,
    pub entry_id: Option<u64>,
}

// —— 内部事件日志缓冲结构 ——

#[derive(Debug, Clone)]
struct StoredEventLogEntry {
    entry: EventLogEntry,
    size_bytes: usize,
}

struct EventLogBuffer {
    entries: VecDeque<StoredEventLogEntry>,
    eviction_tracker: Option<Arc<EvictionCounter>>,
}

#[derive(Debug)]
struct EvictionCounter {
    id_prefix: String,
    evicted: AtomicU64,
    active: AtomicBool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLogTrackerError {
    EmptyPrefix,
    AlreadyRegistered,
    WrongLog,
}

/// A run owns this handle; dropping it disables tracking without retaining its owner.
#[derive(Debug)]
#[must_use]
pub struct EventLogEvictionTracker {
    counter: Arc<EvictionCounter>,
}

impl EventLogEvictionTracker {
    /// Live observation only. Use `finish_eviction_tracker` for a frozen terminal count.
    pub fn evicted(&self) -> u64 {
        self.counter.evicted.load(Ordering::Acquire)
    }
}

impl Drop for EventLogEvictionTracker {
    fn drop(&mut self) {
        // A single inactive tombstone may remain until the next queue operation.
        self.counter.active.store(false, Ordering::Release);
    }
}

/// 事件日志，按环形缓冲保存最近 N 条事件并限制总内存占用
pub struct EventLog {
    buffer: RwLock<EventLogBuffer>,
    capacity: usize,
    max_event_size_bytes: usize,
    max_total_size_bytes: usize,
    dropped_events: AtomicU64,
}

/// 事件日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub id: u64,
    pub timestamp: String,
    pub event: AppEvent,
}

impl EventLog {
    /// 创建仅限制条目数量的事件日志
    #[allow(dead_code)]
    pub fn new(capacity: usize) -> Self {
        Self::with_limits(capacity, 10 * 1024, 5 * 1024 * 1024)
    }

    /// 创建同时限制单条大小与总大小的事件日志
    pub fn with_limits(
        capacity: usize,
        max_event_size_bytes: usize,
        max_total_size_bytes: usize,
    ) -> Self {
        Self {
            buffer: RwLock::new(EventLogBuffer {
                entries: VecDeque::with_capacity(capacity),
                eviction_tracker: None,
            }),
            capacity,
            max_event_size_bytes,
            max_total_size_bytes,
            dropped_events: AtomicU64::new(0),
        }
    }

    pub async fn register_eviction_tracker(
        &self,
        id_prefix: &str,
    ) -> Result<EventLogEvictionTracker, EventLogTrackerError> {
        if id_prefix.is_empty() {
            return Err(EventLogTrackerError::EmptyPrefix);
        }
        let mut buffer = self.buffer.write().await;
        if buffer
            .eviction_tracker
            .as_ref()
            .is_some_and(|counter| counter.active.load(Ordering::Acquire))
        {
            return Err(EventLogTrackerError::AlreadyRegistered);
        }
        let counter = Arc::new(EvictionCounter {
            id_prefix: id_prefix.into(),
            evicted: AtomicU64::new(0),
            active: AtomicBool::new(true),
        });
        buffer.eviction_tracker = Some(counter.clone());
        Ok(EventLogEvictionTracker { counter })
    }

    pub async fn finish_eviction_tracker(
        &self,
        tracker: EventLogEvictionTracker,
    ) -> Result<u64, EventLogTrackerError> {
        let mut buffer = self.buffer.write().await;
        if !buffer
            .eviction_tracker
            .as_ref()
            .is_some_and(|counter| Arc::ptr_eq(counter, &tracker.counter))
        {
            return Err(EventLogTrackerError::WrongLog);
        }
        buffer.eviction_tracker = None;
        tracker.counter.active.store(false, Ordering::Release);
        Ok(tracker.evicted())
    }

    /// 推入事件
    pub async fn push(&self, event: AppEvent) {
        let _ = self.push_checked(event).await;
    }

    pub async fn push_checked(&self, event: AppEvent) -> EventLogPushReceipt {
        let mut receipt = EventLogPushReceipt {
            accepted: false,
            oversized: false,
            evicted: 0,
            evicted_monitoring_ids: Vec::new(),
            entry_id: None,
        };
        let event_size_bytes = serde_json::to_vec(&event).map_or(0, |bytes| bytes.len());
        if event_size_bytes > self.max_event_size_bytes {
            receipt.oversized = true;
            self.dropped_events.fetch_add(1, Ordering::Relaxed);
            tracing::warn!(
                "[event_log] dropped oversized event: size={} max={}",
                event_size_bytes,
                self.max_event_size_bytes
            );
            return receipt;
        }

        let mut buf = self.buffer.write().await;
        if buf
            .eviction_tracker
            .as_ref()
            .is_some_and(|counter| !counter.active.load(Ordering::Acquire))
        {
            buf.eviction_tracker = None;
        }
        let mut total_size_bytes = buf
            .entries
            .iter()
            .map(|entry| entry.size_bytes)
            .sum::<usize>();

        while !buf.entries.is_empty()
            && (buf.entries.len() >= self.capacity
                || total_size_bytes.saturating_add(event_size_bytes) > self.max_total_size_bytes)
        {
            if let Some(removed) = buf.entries.pop_front() {
                total_size_bytes = total_size_bytes.saturating_sub(removed.size_bytes);
                receipt.evicted += 1;
                if let AppEvent::Monitoring(entry) = removed.entry.event {
                    if let Some(counter) = &buf.eviction_tracker
                        && counter.active.load(Ordering::Acquire)
                        && entry.id.starts_with(&counter.id_prefix)
                    {
                        counter.evicted.fetch_add(1, Ordering::Release);
                    }
                    receipt.evicted_monitoring_ids.push(entry.id);
                }
            }
        }

        let id = buf.entries.back().map_or(1, |e| e.entry.id + 1);
        buf.entries.push_back(StoredEventLogEntry {
            entry: EventLogEntry {
                id,
                timestamp: chrono::Utc::now().to_rfc3339(),
                event,
            },
            size_bytes: event_size_bytes,
        });
        receipt.accepted = true;
        receipt.entry_id = Some(id);
        receipt
    }

    /// 获取最近 N 条事件
    pub async fn recent(&self, count: usize) -> Vec<EventLogEntry> {
        let buf = self.buffer.read().await;
        buf.entries
            .iter()
            .rev()
            .take(count)
            .map(|item| item.entry.clone())
            .collect()
    }

    /// 获取日志统计
    pub async fn stats(&self) -> EventLogStats {
        let buf = self.buffer.read().await;
        EventLogStats {
            entries: buf.entries.len(),
            total_size_bytes: buf.entries.iter().map(|item| item.size_bytes).sum(),
            capacity: self.capacity,
            max_event_size_bytes: self.max_event_size_bytes,
            max_total_size_bytes: self.max_total_size_bytes,
            dropped_events: self.dropped_events.load(Ordering::Relaxed),
        }
    }
}

/// 事件通道名称常量
///
/// 所有面向前端的 `app_handle.emit(channel, payload)` 应优先引用此处常量，
/// 便于前端对齐订阅名并在重构时一次性更新。
#[allow(dead_code)]
pub mod channels {
    // —— 签到 ——
    pub const CHECKIN_COMPLETED: &str = "checkin:completed";
    pub const CHECKIN_FAILED: &str = "checkin:failed";
    pub const CHECKIN_JOB_DELTA: &str = "checkin:job-delta";
    pub const CHECKIN_JOB_PROGRESS: &str = "checkin:job-progress";
    pub const CHECKIN_JOB_FINISHED: &str = "checkin:job-finished";
    pub const CHECKIN_JOB_TIMEOUT: &str = "checkin:job-timeout";

    // —— 同步 / 通用 ——
    pub const SYNC_STATUS: &str = "sync:status";
    pub const TASK_PROGRESS: &str = "task:progress";
    pub const NOTIFICATION: &str = "app:notification";
    pub const MONITORING_ENTRY: &str = "app:monitoring";
    pub const APP_LOG: &str = "app-log";
    pub const TOKEN_STATS: &str = "token-stats";
    /// fire-and-forget 后台任务 panic / join 失败时广播给前端。
    /// payload: `{ "name": <task-label>, "error": <display-string> }`
    pub const APP_TASK_PANICKED: &str = "app:task-panicked";

    // —— 环境 ——
    pub const ENVIRONMENT_CHANGED: &str = "env:changed";
    pub const ENV_REFRESH_REQUESTED: &str = "env:refresh-requested";

    // —— 用量 ——
    pub const USAGE_IMPORT: &str = "usage:import-completed";

    // —— Codex OAuth / Tray ——
    pub const CODEX_OAUTH_LOGIN_TIMEOUT: &str = "codex-oauth-login-timeout";
    pub const CODEX_OAUTH_LOGIN_COMPLETED: &str = "codex-oauth-login-completed";
    pub const CODEX_TRAY_REFRESH: &str = "codex-tray:refresh";
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn monitoring(id: &str) -> AppEvent {
        let mut entry = MonitoringEntry::new(
            ccr_types::MonitoringLevel::Info,
            "system",
            "ack.test",
            "tests",
            "controlled",
        );
        entry.id = id.into();
        entry.timestamp = "2026-11-06T00:00:00Z".into();
        AppEvent::Monitoring(entry)
    }

    #[tokio::test]
    async fn registered_eviction_tracker_counts_unit_victims_and_freezes_on_finish() {
        let log = EventLog::with_limits(2, 1024, 4096);
        let tracker = log.register_eviction_tracker("s2-run-").await.unwrap();
        log.push(monitoring("s2-run-1")).await;
        log.push(monitoring("ordinary")).await;
        log.push(monitoring("s2-run-2")).await;
        assert_eq!(tracker.evicted(), 1);
        log.push(AppEvent::UsageImportCompleted(UsageImportPayload {
            imported_count: 1,
            platform: "controlled".into(),
        }))
        .await;
        assert_eq!(tracker.evicted(), 1); // Ordinary victim is not this run's loss.
        log.push(monitoring("other-run-1")).await;
        assert_eq!(tracker.evicted(), 2);
        log.push_checked(monitoring("s2-run-3")).await;
        let frozen = tracker.counter.clone();
        assert_eq!(log.finish_eviction_tracker(tracker).await.unwrap(), 2);
        assert!(log.buffer.read().await.eviction_tracker.is_none());
        log.push(monitoring("ordinary-2")).await;
        log.push(monitoring("ordinary-3")).await; // Removes s2-run-3 after the run ended.
        assert_eq!(frozen.evicted.load(Ordering::Acquire), 2);
        assert!(!frozen.active.load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn eviction_tracker_is_lazy_single_registration_and_drop_cleans_tombstone() {
        let log = EventLog::with_limits(1, 1024, 4096);
        assert!(log.buffer.read().await.eviction_tracker.is_none());
        assert_eq!(
            log.register_eviction_tracker("").await.unwrap_err(),
            EventLogTrackerError::EmptyPrefix
        );
        let tracker = log.register_eviction_tracker("s2-one-").await.unwrap();
        assert_eq!(
            log.register_eviction_tracker("s2-two-").await.unwrap_err(),
            EventLogTrackerError::AlreadyRegistered
        );
        let weak = Arc::downgrade(&tracker.counter);
        drop(tracker);
        log.push(monitoring("ordinary")).await;
        assert!(log.buffer.read().await.eviction_tracker.is_none());
        assert!(weak.upgrade().is_none());
        let replacement = log.register_eviction_tracker("s2-two-").await.unwrap();
        assert_eq!(replacement.evicted(), 0);
        assert_eq!(log.finish_eviction_tracker(replacement).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn cancelled_tracker_finish_leaves_only_replaceable_inactive_tombstone() {
        let log = EventLog::with_limits(1, 1024, 4096);
        let tracker = log.register_eviction_tracker("s2-cancel-").await.unwrap();
        let weak = Arc::downgrade(&tracker.counter);
        let queue = log.buffer.write().await;
        let mut finish = Box::pin(log.finish_eviction_tracker(tracker));
        tokio::select! {
            biased;
            _ = &mut finish => panic!("finish must wait for the real queue lock"),
            _ = std::future::ready(()) => {}
        }
        drop(finish);
        assert!(
            !queue
                .eviction_tracker
                .as_ref()
                .unwrap()
                .active
                .load(Ordering::Acquire)
        );
        drop(queue);
        let replacement = log.register_eviction_tracker("s2-next-").await.unwrap();
        assert!(weak.upgrade().is_none());
        assert_eq!(log.finish_eviction_tracker(replacement).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn tracker_cannot_finalize_another_event_log() {
        let first = EventLog::with_limits(1, 1024, 4096);
        let second = EventLog::with_limits(1, 1024, 4096);
        let tracker = first.register_eviction_tracker("s2-shared-").await.unwrap();
        let other = second
            .register_eviction_tracker("s2-shared-")
            .await
            .unwrap();
        assert_eq!(
            second.finish_eviction_tracker(tracker).await,
            Err(EventLogTrackerError::WrongLog)
        );
        second.push(monitoring("s2-shared-1")).await;
        second.push(monitoring("ordinary")).await;
        assert_eq!(other.evicted(), 1);
        let _replacement = first
            .register_eviction_tracker("s2-replaced-")
            .await
            .unwrap();
        assert_eq!(second.finish_eviction_tracker(other).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn checked_ring_reports_oversize_without_accepting_or_evicting() {
        let log = EventLog::with_limits(2, 512, 1024);
        log.push(monitoring("ordinary")).await;
        let receipt = log
            .push_checked(AppEvent::Notification(NotificationPayload {
                level: NotificationLevel::Info,
                title: "controlled".into(),
                message: "x".repeat(1024),
            }))
            .await;
        assert!(!receipt.accepted);
        assert!(receipt.oversized);
        assert_eq!(receipt.entry_id, None);
        assert_eq!(receipt.evicted, 0);
        assert!(receipt.evicted_monitoring_ids.is_empty());
        assert_eq!(log.stats().await.dropped_events, 1);
        assert_eq!(log.recent(10).await.len(), 1);
    }

    #[tokio::test]
    async fn checked_ring_capacity_and_byte_evictions_preserve_monitoring_identity() {
        let log = EventLog::with_limits(2, 1024, 4096);
        log.push(monitoring("s2-first")).await;
        log.push(AppEvent::UsageImportCompleted(UsageImportPayload {
            imported_count: 1,
            platform: "controlled".into(),
        }))
        .await;
        let receipt = log.push_checked(monitoring("s2-last")).await;
        assert!(receipt.accepted);
        assert!(!receipt.oversized);
        assert_eq!(receipt.evicted, 1);
        assert_eq!(receipt.evicted_monitoring_ids, ["s2-first"]);
        assert_eq!(receipt.entry_id, Some(3));
        let receipt = log.push_checked(monitoring("s2-next")).await;
        assert_eq!(receipt.evicted, 1);
        assert!(receipt.evicted_monitoring_ids.is_empty());
        assert_eq!(log.stats().await.dropped_events, 0);

        let size = serde_json::to_vec(&monitoring("a")).unwrap().len();
        let bytes = EventLog::with_limits(10, 1024, size * 2);
        bytes.push(monitoring("a")).await;
        bytes.push(monitoring("b")).await;
        let receipt = bytes.push_checked(monitoring("c")).await;
        assert_eq!(receipt.evicted, 1);
        assert_eq!(receipt.evicted_monitoring_ids, ["a"]);
        assert_eq!(bytes.stats().await.entries, 2);
        assert_eq!(bytes.stats().await.total_size_bytes, size * 2);
    }

    #[tokio::test]
    async fn unit_push_preserves_existing_single_entry_limits_policy() {
        // Existing policy accepts an individually allowed event when the ring is empty,
        // even with zero capacity or a total byte limit below that event's size.
        let unit = EventLog::with_limits(0, 1024, 1);
        let checked = EventLog::with_limits(0, 1024, 1);
        unit.push(monitoring("a")).await;
        let first = checked.push_checked(monitoring("a")).await;
        assert!(first.accepted);
        assert_eq!(first.evicted, 0);
        unit.push(monitoring("b")).await;
        let second = checked.push_checked(monitoring("b")).await;
        assert!(second.accepted);
        assert_eq!(second.evicted_monitoring_ids, ["a"]);
        let unit_stats = unit.stats().await;
        let checked_stats = checked.stats().await;
        assert_eq!(unit_stats.entries, 1);
        assert_eq!(unit_stats.total_size_bytes, checked_stats.total_size_bytes);
        assert_eq!(unit_stats.dropped_events, checked_stats.dropped_events);
    }
}
