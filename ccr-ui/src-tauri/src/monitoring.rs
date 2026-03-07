use ccr_types::{FrontendLogInput, MonitoringEntry, MonitoringLevel};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::checkin_jobs::CheckinJobSnapshot;
use crate::events::{self, AppEvent, EnvironmentEventPayload, UsageImportPayload};

pub fn should_persist(level: MonitoringLevel, event_type: &str) -> bool {
    matches!(level, MonitoringLevel::Warn | MonitoringLevel::Error)
        || matches!(
            event_type,
            "environment.changed"
                | "usage.import.completed"
                | "checkin.job.finished"
                | "checkin.job.timeout"
                | "frontend.warn"
                | "frontend.error"
        )
}

pub async fn record_monitoring_entry(app_handle: &AppHandle, entry: MonitoringEntry, persist: bool) {
    let state = app_handle.state::<crate::state::AppState>();
    state.event_log.push(AppEvent::Monitoring(entry.clone())).await;

    if persist {
        state.monitoring_logs.append_entry(&entry).await;
        state.monitoring_logs.force_flush().await;
    }

    if let Err(error) = app_handle.emit(events::channels::MONITORING_ENTRY, entry) {
        tracing::warn!(?error, "failed to emit monitoring entry");
    }
}

pub async fn emit_and_record_monitoring_event<T>(
    app_handle: &AppHandle,
    channel: &str,
    payload: &T,
    entry: MonitoringEntry,
    persist: bool,
) where
    T: Serialize + Clone,
{
    if let Err(error) = app_handle.emit(channel, payload.clone()) {
        tracing::warn!(channel, ?error, "failed to emit business event");
    }

    record_monitoring_entry(app_handle, entry, persist).await;
}

pub fn environment_changed_entry(payload: &EnvironmentEventPayload) -> MonitoringEntry {
    MonitoringEntry::new(
        MonitoringLevel::Info,
        "environment",
        "environment.changed",
        payload.env_type.clone(),
        format!("Environment switched to {} ({})", payload.env_id, payload.status),
    )
    .with_fields(serde_json::json!({
        "envId": payload.env_id,
        "envType": payload.env_type,
        "status": payload.status,
    }))
}

pub fn usage_import_entry(payload: &UsageImportPayload) -> MonitoringEntry {
    MonitoringEntry::new(
        MonitoringLevel::Info,
        "usage",
        "usage.import.completed",
        payload.platform.clone(),
        format!("Imported {} usage records for {}", payload.imported_count, payload.platform),
    )
    .with_fields(serde_json::json!({
        "platform": payload.platform,
        "importedCount": payload.imported_count,
    }))
}

pub fn frontend_log_entry(input: FrontendLogInput) -> MonitoringEntry {
    let source = if input.source.trim().is_empty() {
        "frontend".to_string()
    } else {
        input.source
    };
    let event_type = match input.level {
        MonitoringLevel::Warn => "frontend.warn",
        MonitoringLevel::Error => "frontend.error",
        MonitoringLevel::Debug => "frontend.debug",
        MonitoringLevel::Info => "frontend.info",
    };

    MonitoringEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: input
            .timestamp
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
        level: input.level,
        channel: "frontend".to_string(),
        event_type: event_type.to_string(),
        source,
        message: input.message,
        correlation_id: None,
        fields: input.fields,
    }
}

pub fn checkin_job_entry(event: &str, snapshot: &CheckinJobSnapshot) -> MonitoringEntry {
    let (level, event_type) = match event {
        "checkin:job-timeout" => (MonitoringLevel::Warn, "checkin.job.timeout"),
        "checkin:job-finished" => {
            if snapshot.summary.failed > 0 {
                (MonitoringLevel::Warn, "checkin.job.finished")
            } else {
                (MonitoringLevel::Info, "checkin.job.finished")
            }
        }
        _ => (MonitoringLevel::Info, "checkin.job.progress"),
    };

    let latest_message = snapshot
        .logs
        .iter()
        .rev()
        .find_map(|entry| entry.message.clone())
        .unwrap_or_else(|| format!("Processed {}/{} accounts", snapshot.completed, snapshot.total));

    MonitoringEntry::new(level, "checkin", event_type, "checkin", latest_message)
        .with_correlation_id(snapshot.job_id.clone())
        .with_fields(serde_json::json!({
            "jobId": snapshot.job_id,
            "status": snapshot.status,
            "total": snapshot.total,
            "completed": snapshot.completed,
            "currentAccountName": snapshot.current_account_name,
            "summary": snapshot.summary,
            "finishedAt": snapshot.finished_at,
        }))
}

pub fn event_to_monitoring_entry(event: &AppEvent) -> Option<MonitoringEntry> {
    match event {
        AppEvent::Monitoring(entry) => Some(entry.clone()),
        AppEvent::EnvironmentChanged(payload) => Some(environment_changed_entry(payload)),
        AppEvent::UsageImportCompleted(payload) => Some(usage_import_entry(payload)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkin_jobs::{CheckinJobLogEntry, CheckinJobStatus};
    use serde_json::json;

    #[test]
    fn should_persist_warn_and_whitelisted_events() {
        assert!(should_persist(MonitoringLevel::Warn, "frontend.info"));
        assert!(should_persist(MonitoringLevel::Info, "environment.changed"));
        assert!(should_persist(MonitoringLevel::Info, "frontend.error"));
        assert!(!should_persist(MonitoringLevel::Info, "frontend.info"));
    }

    #[test]
    fn frontend_log_entry_defaults_source_and_maps_event_type() {
        let entry = frontend_log_entry(FrontendLogInput {
            level: MonitoringLevel::Error,
            message: "frontend failed".to_string(),
            source: String::new(),
            timestamp: Some("2026-03-07T00:00:00Z".to_string()),
            fields: Some(json!({ "code": 500 })),
        });

        assert_eq!(entry.level, MonitoringLevel::Error);
        assert_eq!(entry.channel, "frontend");
        assert_eq!(entry.event_type, "frontend.error");
        assert_eq!(entry.source, "frontend");
        assert_eq!(entry.timestamp, "2026-03-07T00:00:00Z");
        assert_eq!(entry.fields, Some(json!({ "code": 500 })));
    }

    #[test]
    fn checkin_job_entry_marks_failed_finished_job_as_warn() {
        let mut log = CheckinJobLogEntry::pending(
            "account-1".to_string(),
            "Alpha".to_string(),
            "Provider".to_string(),
        );
        log.message = Some("request failed".to_string());

        let mut snapshot = CheckinJobSnapshot::new("job-1".to_string(), vec![log]);
        snapshot.status = CheckinJobStatus::Finished;
        snapshot.completed = 1;
        snapshot.summary.failed = 1;
        snapshot.finished_at = Some("2026-03-07T00:00:00Z".to_string());

        let entry = checkin_job_entry("checkin:job-finished", &snapshot);

        assert_eq!(entry.level, MonitoringLevel::Warn);
        assert_eq!(entry.channel, "checkin");
        assert_eq!(entry.event_type, "checkin.job.finished");
        assert_eq!(entry.correlation_id.as_deref(), Some("job-1"));
        assert!(entry.message.contains("request failed"));
    }

    #[test]
    fn event_to_monitoring_entry_maps_domain_events() {
        let environment = EnvironmentEventPayload {
            env_id: "local".to_string(),
            env_type: "local".to_string(),
            status: "active".to_string(),
        };
        let usage = UsageImportPayload {
            imported_count: 12,
            platform: "codex".to_string(),
        };

        let env_entry = event_to_monitoring_entry(&AppEvent::EnvironmentChanged(environment))
            .expect("environment event should map");
        let usage_entry = event_to_monitoring_entry(&AppEvent::UsageImportCompleted(usage))
            .expect("usage event should map");

        assert_eq!(env_entry.event_type, "environment.changed");
        assert_eq!(usage_entry.event_type, "usage.import.completed");
        assert_eq!(usage_entry.channel, "usage");
    }
}