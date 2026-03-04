//! Tauri Event 系统 — 替代 WebSocket 实时推送。
//!
//! 通过 `app_handle.emit(event, payload)` 向前端推送事件，
//! 前端使用 `listen(event, handler)` 监听。

use std::collections::VecDeque;

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

// ── 事件日志环形缓冲区 ──

/// 事件日志 — 保留最近 N 条事件供前端查询
pub struct EventLog {
    buffer: RwLock<VecDeque<EventLogEntry>>,
    capacity: usize,
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
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: RwLock::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    /// 追加事件
    pub async fn push(&self, event: AppEvent) {
        let mut buf = self.buffer.write().await;
        let id = buf.back().map_or(1, |e| e.id + 1);
        if buf.len() >= self.capacity {
            buf.pop_front();
        }
        buf.push_back(EventLogEntry {
            id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            event,
        });
    }

    /// 获取最近 N 条事件
    pub async fn recent(&self, count: usize) -> Vec<EventLogEntry> {
        let buf = self.buffer.read().await;
        buf.iter().rev().take(count).cloned().collect()
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
