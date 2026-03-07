//! Tauri Event 缁崵绮?閳?閺囧じ鍞?WebSocket 鐎圭偞妞傞幒銊┾偓浣碘偓?
//!
//! 闁俺绻?`app_handle.emit(event, payload)` 閸氭垵澧犵粩顖涘腹闁椒绨ㄦ禒璁圭礉
//! 閸撳秶顏担璺ㄦ暏 `listen(event, handler)` 閻╂垵鎯夐妴?

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

use ccr_types::MonitoringEntry;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// 娴滃娆㈢猾璇茬€烽弸姘
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AppEvent {
    /// 缁涙儳鍩岀€瑰本鍨?
    CheckinCompleted(CheckinEventPayload),
    /// 缁涙儳鍩屾径杈Е
    CheckinFailed(CheckinEventPayload),
    /// 閸氬本顒為悩鑸碘偓浣稿綁閺?
    SyncStatusChanged(SyncEventPayload),
    /// 閸氬骸褰存禒璇插鏉╂稑瀹?
    TaskProgress(TaskProgressPayload),
    /// 缁崵绮洪柅姘辩叀
    Notification(NotificationPayload),
    /// 閻滎垰顣ㄩ悩鑸碘偓浣稿綁閺囪揪绱橶SL/SSH 鏉╃偞甯?閺傤厼绱戦敍?
    EnvironmentChanged(EnvironmentEventPayload),
    /// 閻劑鍣洪弫鐗堝祦鐎电厧鍙嗙€瑰本鍨?
    UsageImportCompleted(UsageImportPayload),
    /// 缂佺喍绔撮惄鎴炲付閺夛紕娲?
    Monitoring(MonitoringEntry),
}

/// 缁涙儳鍩屾禍瀣╂鏉炲€熷祹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinEventPayload {
    pub account_id: i64,
    pub provider_name: String,
    pub success: bool,
    pub message: String,
}

/// 閸氬本顒炴禍瀣╂鏉炲€熷祹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEventPayload {
    pub operation: String,
    pub status: String,
    pub message: String,
}

/// 娴犺濮熸潻娑樺鏉炲€熷祹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressPayload {
    pub task_id: String,
    pub progress: f64,
    pub message: String,
}

/// 闁氨鐓℃潪鍊熷祹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub level: NotificationLevel,
    pub title: String,
    pub message: String,
}

/// 闁氨鐓＄痪褍鍩?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// 閻滎垰顣ㄩ崣妯绘纯娴滃娆㈡潪鍊熷祹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentEventPayload {
    pub env_id: String,
    pub env_type: String,
    pub status: String,
}

/// 閻劑鍣虹€电厧鍙嗙€瑰本鍨氭潪鍊熷祹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageImportPayload {
    pub imported_count: usize,
    pub platform: String,
}

/// 娴滃娆㈤弮銉ョ箶缂佺喕顓?
#[derive(Debug, Clone, Serialize)]
pub struct EventLogStats {
    pub entries: usize,
    pub total_size_bytes: usize,
    pub capacity: usize,
    pub max_event_size_bytes: usize,
    pub max_total_size_bytes: usize,
    pub dropped_events: u64,
}

// 閳光偓閳光偓 娴滃娆㈤弮銉ョ箶閻滎垰鑸扮紓鎾冲暱閸?閳光偓閳光偓

#[derive(Debug, Clone)]
struct StoredEventLogEntry {
    entry: EventLogEntry,
    size_bytes: usize,
}

/// 娴滃娆㈤弮銉ョ箶 閳?娣囨繄鏆€閺堚偓鏉?N 閺夆€茬皑娴犳湹绶甸崜宥囶伂閺屻儴顕?
pub struct EventLog {
    buffer: RwLock<VecDeque<StoredEventLogEntry>>,
    capacity: usize,
    max_event_size_bytes: usize,
    max_total_size_bytes: usize,
    dropped_events: AtomicU64,
}

/// 閺冦儱绻旈弶锛勬窗
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub id: u64,
    pub timestamp: String,
    pub event: AppEvent,
}

impl EventLog {
    /// 閸掓稑缂撻幐鍥х暰鐎瑰綊鍣洪惃鍕皑娴犺埖妫╄箛?
    #[allow(dead_code)]
    pub fn new(capacity: usize) -> Self {
        Self::with_limits(capacity, 10 * 1024, 5 * 1024 * 1024)
    }

    /// 閸掓稑缂撶敮锕€銇囩亸蹇涙閸掑墎娈戞禍瀣╂閺冦儱绻?
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

    /// 鏉╄棄濮炴禍瀣╂
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

    /// 閼惧嘲褰囬張鈧潻?N 閺夆€茬皑娴?
    pub async fn recent(&self, count: usize) -> Vec<EventLogEntry> {
        let buf = self.buffer.read().await;
        buf.iter()
            .rev()
            .take(count)
            .map(|item| item.entry.clone())
            .collect()
    }

    /// 缂佺喕顓告穱鈩冧紖
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

/// 娴滃娆㈤柅姘朵壕閸氬秶袨鐢悂鍣?
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

