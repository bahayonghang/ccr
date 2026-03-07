//! Tauri Event 系统，使用原生事件机制替代 WebSocket。
//!
//! 后端通过 `app_handle.emit(event, payload)` 广播事件，
//! 前端通过 `listen(event, handler)` 订阅事件。

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

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

// —— 内部事件日志缓冲结构 ——

#[derive(Debug, Clone)]
struct StoredEventLogEntry {
    entry: EventLogEntry,
    size_bytes: usize,
}

/// 事件日志，按环形缓冲保存最近 N 条事件并限制总内存占用
pub struct EventLog {
    buffer: RwLock<VecDeque<StoredEventLogEntry>>,
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
            buffer: RwLock::new(VecDeque::with_capacity(capacity)),
            capacity,
            max_event_size_bytes,
            max_total_size_bytes,
            dropped_events: AtomicU64::new(0),
        }
    }

    /// 推入事件
    pub async fn push(&self, event: AppEvent) {
        let event_size_bytes = serde_json::to_vec(&event).map_or(0, |bytes| bytes.len());
        if event_size_bytes > self.max_event_size_bytes {
            self.dropped_events.fetch_add(1, Ordering::Relaxed);
            tracing::warn!(
                "[event_log] dropped oversized event: size={} max={}",
                event_size_bytes,
                self.max_event_size_bytes
            );
            return;
        }

        let mut buf = self.buffer.write().await;
        let mut total_size_bytes = buf.iter().map(|entry| entry.size_bytes).sum::<usize>();

        while !buf.is_empty()
            && (buf.len() >= self.capacity
                || total_size_bytes.saturating_add(event_size_bytes) > self.max_total_size_bytes)
        {
            if let Some(removed) = buf.pop_front() {
                total_size_bytes = total_size_bytes.saturating_sub(removed.size_bytes);
            }
        }

        let id = buf.back().map_or(1, |e| e.entry.id + 1);
        buf.push_back(StoredEventLogEntry {
            entry: EventLogEntry {
                id,
                timestamp: chrono::Utc::now().to_rfc3339(),
                event,
            },
            size_bytes: event_size_bytes,
        });
    }

    /// 获取最近 N 条事件
    pub async fn recent(&self, count: usize) -> Vec<EventLogEntry> {
        let buf = self.buffer.read().await;
        buf.iter()
            .rev()
            .take(count)
            .map(|item| item.entry.clone())
            .collect()
    }

    /// 获取日志统计
    pub async fn stats(&self) -> EventLogStats {
        let buf = self.buffer.read().await;
        EventLogStats {
            entries: buf.len(),
            total_size_bytes: buf.iter().map(|item| item.size_bytes).sum(),
            capacity: self.capacity,
            max_event_size_bytes: self.max_event_size_bytes,
            max_total_size_bytes: self.max_total_size_bytes,
            dropped_events: self.dropped_events.load(Ordering::Relaxed),
        }
    }
}

/// 事件通道名称常量
#[allow(dead_code)]
pub mod channels {
    pub const CHECKIN_COMPLETED: &str = "checkin:completed";
    pub const CHECKIN_FAILED: &str = "checkin:failed";
    pub const SYNC_STATUS: &str = "sync:status";
    pub const TASK_PROGRESS: &str = "task:progress";
    pub const NOTIFICATION: &str = "app:notification";
    pub const ENVIRONMENT_CHANGED: &str = "env:changed";
    pub const USAGE_IMPORT: &str = "usage:import-completed";
    pub const MONITORING_ENTRY: &str = "app:monitoring";
}
