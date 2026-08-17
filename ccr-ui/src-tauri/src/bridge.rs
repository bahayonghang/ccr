use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use ccr_core::{BridgedLogEvent, enter_bridge_consumer, take_bridged_log_receiver};
use ccr_types::{MonitoringEntry, MonitoringLevel};
use tauri::{AppHandle, Manager};

use crate::monitoring::{record_monitoring_entry, should_persist};
use crate::state::AppState;

pub const MONITORING_FLUSH_INTERVAL: Duration = Duration::from_secs(2);
pub const MONITORING_EXIT_FLUSH_TIMEOUT: Duration = Duration::from_millis(500);

static BRIDGE_IO_FAILURES: AtomicU64 = AtomicU64::new(0);

pub fn bridge_io_failure_count() -> u64 {
    BRIDGE_IO_FAILURES.load(Ordering::Relaxed)
}

pub fn note_bridge_io_failure() {
    BRIDGE_IO_FAILURES.fetch_add(1, Ordering::Relaxed);
}

pub fn runtime_entry_from_bridged(event: BridgedLogEvent) -> MonitoringEntry {
    let level = MonitoringLevel::from(event.level.as_str());
    let event_type = match level {
        MonitoringLevel::Error => "runtime.error",
        _ => "runtime.warn",
    };

    MonitoringEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: event.timestamp,
        level,
        channel: "runtime".to_string(),
        event_type: event_type.to_string(),
        source: event.target,
        message: event.message,
        correlation_id: Some(event.correlation_id).filter(|value| !value.is_empty()),
        fields: Some(event.fields),
    }
}

pub fn start_monitoring_bridge(app: AppHandle) {
    let Some(receiver) = take_bridged_log_receiver() else {
        return;
    };

    let _ = std::thread::Builder::new()
        .name("ccr-log-bridge".to_string())
        .spawn(move || {
            while let Ok(event) = receiver.recv() {
                let app = app.clone();
                enter_bridge_consumer(|| {
                    tauri::async_runtime::block_on(record_bridged_event(&app, event));
                });
            }
        });
}

pub fn start_monitoring_flush_ticker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(MONITORING_FLUSH_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            let Some(state) = app.try_state::<AppState>() else {
                break;
            };
            state.monitoring_logs.force_flush().await;
        }
    });
}

pub fn flush_monitoring_on_exit(app: &tauri::AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let logs = state.monitoring_logs.clone();
    let _ = tauri::async_runtime::block_on(async {
        tokio::time::timeout(MONITORING_EXIT_FLUSH_TIMEOUT, logs.force_flush()).await
    });
}

async fn record_bridged_event(app: &AppHandle, event: BridgedLogEvent) {
    let entry = runtime_entry_from_bridged(event);
    let persist = should_persist(entry.level, &entry.event_type);
    record_monitoring_entry(app, entry, persist).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn runtime_entry_maps_error_and_warn() {
        let error = runtime_entry_from_bridged(BridgedLogEvent {
            level: "error".to_string(),
            target: "ccr_core::core::http".to_string(),
            message: "boom".to_string(),
            fields: json!({}),
            correlation_id: "proc-1".to_string(),
            timestamp: "2026-08-17T00:00:00Z".to_string(),
        });
        assert_eq!(error.channel, "runtime");
        assert_eq!(error.event_type, "runtime.error");
        assert_eq!(error.level, MonitoringLevel::Error);
        assert_eq!(error.correlation_id.as_deref(), Some("proc-1"));
        assert_eq!(error.source, "ccr_core::core::http");

        let warn = runtime_entry_from_bridged(BridgedLogEvent {
            level: "warn".to_string(),
            target: "ccr_cli::platforms::claude".to_string(),
            message: "slow".to_string(),
            fields: json!({}),
            correlation_id: "proc-1".to_string(),
            timestamp: "2026-08-17T00:00:00Z".to_string(),
        });
        assert_eq!(warn.event_type, "runtime.warn");
        assert!(!should_persist(warn.level, &warn.event_type));
        assert!(should_persist(error.level, &error.event_type));
    }

    #[test]
    fn note_bridge_io_failure_increments() {
        let before = bridge_io_failure_count();
        note_bridge_io_failure();
        assert!(bridge_io_failure_count() > before);
    }

    #[test]
    fn reentrant_consumer_does_not_enqueue() {
        use ccr_core::{EnqueueResult, try_enqueue_bridged_log};

        let before = ccr_core::dropped_bridged_log_count();
        enter_bridge_consumer(|| {
            let result = try_enqueue_bridged_log(BridgedLogEvent {
                level: "error".to_string(),
                target: "ccr_core::core::http".to_string(),
                message: "nested".to_string(),
                fields: json!({}),
                correlation_id: "proc-1".to_string(),
                timestamp: "2026-08-17T00:00:00Z".to_string(),
            });
            assert_eq!(result, EnqueueResult::Reentrant);
        });
        assert_eq!(ccr_core::dropped_bridged_log_count(), before);
    }
}
