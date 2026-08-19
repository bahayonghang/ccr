#![allow(dead_code)]

use ccr_types::{MonitoringEntry, MonitoringLevel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::database::{self, repositories::log_repo};

#[derive(Debug, Clone)]
pub struct LogStorageConfig {
    pub retention_days: i64,
    pub flush_threshold: usize,
}

impl Default for LogStorageConfig {
    fn default() -> Self {
        Self {
            retention_days: 14,
            flush_threshold: 20,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogPersistenceService {
    config: LogStorageConfig,
    buffer: Arc<RwLock<Vec<MonitoringEntry>>>,
}

impl LogPersistenceService {
    pub fn new(config: LogStorageConfig) -> Self {
        Self {
            config,
            buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn append_entry(&self, entry: &MonitoringEntry) {
        let mut buffer = self.buffer.write().await;
        buffer.push(entry.clone());

        if buffer.len() >= self.config.flush_threshold {
            let entries = buffer.drain(..).collect::<Vec<_>>();
            drop(buffer);
            self.flush_entries(&entries).await;
        }
    }

    pub async fn append_entries(&self, entries: &[MonitoringEntry]) {
        let mut buffer = self.buffer.write().await;
        buffer.extend(entries.iter().cloned());

        if buffer.len() >= self.config.flush_threshold {
            let entries = buffer.drain(..).collect::<Vec<_>>();
            drop(buffer);
            self.flush_entries(&entries).await;
        }
    }

    async fn flush_entries(&self, entries: &[MonitoringEntry]) {
        if entries.is_empty() {
            return;
        }

        let rows = entries.iter().map(to_repo_entry).collect::<Vec<_>>();
        match database::with_connection(|conn| log_repo::insert_logs_batch(conn, &rows)) {
            Ok(count) => info!(count, "flushed monitoring entries to sqlite"),
            Err(err) => error!(error = %err, "failed to flush monitoring entries"),
        }
    }

    pub async fn force_flush(&self) {
        let mut buffer = self.buffer.write().await;
        let entries = buffer.drain(..).collect::<Vec<_>>();
        drop(buffer);
        self.flush_entries(&entries).await;
    }

    pub async fn read_logs_by_date(&self, date: &str) -> Vec<MonitoringEntry> {
        match database::with_connection(|conn| log_repo::get_logs_by_date(conn, date, 10_000)) {
            Ok(rows) => rows.into_iter().map(from_repo_entry).collect(),
            Err(err) => {
                error!(error = %err, date, "failed to read monitoring entries by date");
                Vec::new()
            }
        }
    }

    pub async fn read_recent_logs(&self, count: usize) -> Vec<MonitoringEntry> {
        match database::with_connection(|conn| log_repo::get_recent_logs(conn, count)) {
            Ok(rows) => rows.into_iter().map(from_repo_entry).collect(),
            Err(err) => {
                error!(error = %err, count, "failed to read recent monitoring entries");
                Vec::new()
            }
        }
    }

    pub async fn query_logs(
        &self,
        level: Option<&str>,
        channel: Option<&str>,
        count: usize,
    ) -> Vec<MonitoringEntry> {
        match database::with_connection(|conn| log_repo::query_logs(conn, level, channel, count)) {
            Ok(rows) => rows.into_iter().map(from_repo_entry).collect(),
            Err(err) => {
                error!(error = %err, "failed to query monitoring entries");
                Vec::new()
            }
        }
    }

    pub async fn cleanup_old_logs(&self) {
        match database::with_connection(|conn| {
            log_repo::delete_old_logs(conn, self.config.retention_days)
        }) {
            Ok(deleted) if deleted > 0 => {
                info!(
                    deleted,
                    retention_days = self.config.retention_days,
                    "cleaned up old monitoring entries"
                );
            }
            Ok(_) => {}
            Err(err) => error!(error = %err, "failed to cleanup old monitoring entries"),
        }
    }

    pub async fn get_available_dates(&self) -> Vec<String> {
        match database::with_connection(log_repo::get_available_dates) {
            Ok(dates) => dates,
            Err(err) => {
                error!(error = %err, "failed to get monitoring entry dates");
                Vec::new()
            }
        }
    }

    pub async fn delete_logs_by_date(&self, date: &str) -> Result<usize, String> {
        database::with_connection(|conn| log_repo::delete_logs_by_date(conn, date))
            .map_err(|err| err.to_string())
    }

    pub async fn get_stats(&self) -> LogStats {
        let buffer_count = self.buffer.read().await.len();
        let (today_count, total_count) = match database::with_connection(log_repo::get_log_stats) {
            Ok(stats) => (stats.today_count as usize, stats.total_count as usize),
            Err(_) => (0, 0),
        };

        LogStats {
            buffer_count,
            today_count,
            total_count,
            storage_type: "SQLite".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub buffer_count: usize,
    pub today_count: usize,
    pub total_count: usize,
    pub storage_type: String,
}

fn to_repo_entry(entry: &MonitoringEntry) -> log_repo::LogEntry {
    let timestamp = DateTime::parse_from_rfc3339(&entry.timestamp)
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    log_repo::LogEntry {
        id: entry.id.clone(),
        timestamp,
        level: entry.level.as_str().to_string(),
        channel: Some(entry.channel.clone()).filter(|value| !value.is_empty()),
        event_type: Some(entry.event_type.clone()).filter(|value| !value.is_empty()),
        source: entry.source.clone(),
        message: entry.message.clone(),
        correlation_id: entry.correlation_id.clone(),
        metadata_json: entry.fields.as_ref().map(ToString::to_string),
    }
}

fn from_repo_entry(entry: log_repo::LogEntry) -> MonitoringEntry {
    MonitoringEntry {
        id: entry.id,
        timestamp: entry.timestamp.to_rfc3339(),
        level: MonitoringLevel::from(entry.level),
        channel: entry.channel.unwrap_or_else(|| "system".to_string()),
        event_type: entry.event_type.unwrap_or_else(|| "log.entry".to_string()),
        source: entry.source,
        message: entry.message,
        correlation_id: entry.correlation_id,
        fields: entry
            .metadata_json
            .and_then(|raw| serde_json::from_str(&raw).ok()),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_entry() -> MonitoringEntry {
        MonitoringEntry::new(
            MonitoringLevel::Info,
            "system",
            "runtime.test",
            "tests",
            "test message",
        )
    }

    #[test]
    fn default_flush_threshold_is_twenty() {
        assert_eq!(LogStorageConfig::default().flush_threshold, 20);
    }

    #[tokio::test]
    async fn test_log_persistence() {
        crate::database::initialize_for_test().unwrap();
        let service = LogPersistenceService::new(LogStorageConfig {
            retention_days: 14,
            flush_threshold: 1,
        });

        service.append_entry(&sample_entry()).await;
        service.force_flush().await;

        let logs = service.read_recent_logs(10).await;
        assert!(!logs.is_empty());
        assert_eq!(logs[0].channel, "system");
    }
}
