//! Tauri Event 系统 — 替代 WebSocket 实时推送。
//!
//! 通过 `app_handle.emit(event, payload)` 向前端推送事件，
//! 前端使用 `listen(event, handler)` 监听。

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// 事件类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AppEvent {
    /// 签到完成
    CheckinCompleted(CheckinEventPayload),
    /// 签到失败
    CheckinFailed(CheckinEventPayload),
    /// 同步状态变更
    SyncStatusChanged(SyncEventPayload),
    /// 后台任务进度
    TaskProgress(TaskProgressPayload),
    /// 系统通知
    Notification(NotificationPayload),
    /// 环境状态变更（WSL/SSH 连接/断开）
    EnvironmentChanged(EnvironmentEventPayload),
    /// 用量数据导入完成
    UsageImportCompleted(UsageImportPayload),
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

/// 环境变更事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentEventPayload {
    pub env_id: String,
    pub env_type: String,
    pub status: String,
}

/// 用量导入完成载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageImportPayload {
    pub imported_count: usize,
    pub platform: String,
}

/// 事件日志统计
#[derive(Debug, Clone, Serialize)]
pub struct EventLogStats {
    pub entries: usize,
    pub total_size_bytes: usize,
    pub capacity: usize,
    pub max_event_size_bytes: usize,
    pub max_total_size_bytes: usize,
    pub dropped_events: u64,
}

// ── 事件日志环形缓冲区 ──

#[derive(Debug, Clone)]
struct StoredEventLogEntry {
    entry: EventLogEntry,
    size_bytes: usize,
}

/// 事件日志 — 保留最近 N 条事件供前端查询
pub struct EventLog {
    buffer: RwLock<VecDeque<StoredEventLogEntry>>,
    capacity: usize,
    max_event_size_bytes: usize,
    max_total_size_bytes: usize,
    dropped_events: AtomicU64,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub id: u64,
    pub timestamp: String,
    pub event: AppEvent,
}

impl EventLog {
    /// 创建指定容量的事件日志
    #[allow(dead_code)]
    pub fn new(capacity: usize) -> Self {
        Self::with_limits(capacity, 10 * 1024, 5 * 1024 * 1024)
    }

    /// 创建带大小限制的事件日志
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

    /// 追加事件
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

    /// 统计信息
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
}
