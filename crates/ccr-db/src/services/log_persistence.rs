#![allow(dead_code)]

use ccr_types::{MonitoringEntry, MonitoringLevel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
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

#[derive(Clone)]
pub struct LogPersistenceService {
    config: LogStorageConfig,
    db: database::DbAccess,
    buffer: Arc<RwLock<Vec<BufferedLogEntry>>>,
}

impl std::fmt::Debug for LogPersistenceService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LogPersistenceService")
            .field("config", &self.config)
            .field("buffer", &self.buffer)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogPersistenceState {
    Buffered,
    Attempting,
    Committed,
    Failed { attempted: bool },
}

/// Kept by the checked caller until terminal; the service retains no receipt journal.
#[derive(Debug, Clone)]
pub struct LogPersistenceTicket {
    entry_id: String,
    state: Arc<AtomicU8>,
}

impl LogPersistenceTicket {
    pub fn entry_id(&self) -> &str {
        &self.entry_id
    }

    pub fn state(&self) -> LogPersistenceState {
        match self.state.load(Ordering::Acquire) {
            0 => LogPersistenceState::Buffered,
            1 => LogPersistenceState::Attempting,
            2 => LogPersistenceState::Committed,
            3 => LogPersistenceState::Failed { attempted: false },
            _ => LogPersistenceState::Failed { attempted: true },
        }
    }
}

#[derive(Debug)]
struct BufferedLogEntry {
    entry: MonitoringEntry,
    ticket: Option<LogPersistenceTicket>,
}

impl Drop for BufferedLogEntry {
    fn drop(&mut self) {
        if let Some(ticket) = &self.ticket {
            let _ = ticket
                .state
                .compare_exchange(0, 3, Ordering::AcqRel, Ordering::Acquire);
            let _ = ticket
                .state
                .compare_exchange(1, 4, Ordering::AcqRel, Ordering::Acquire);
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct LogFlushReceipt {
    pub entry_ids: Vec<String>,
    pub attempted: usize,
    pub committed: usize,
    pub failed: usize,
    pub remaining_buffered: usize,
}

impl LogPersistenceService {
    /// Strict captured-connection access for adapters with an independently owned main DB.
    /// Unlike the legacy global API, storage errors remain visible to the caller.
    pub fn query_logs_with_connection(
        conn: &rusqlite::Connection,
        level: Option<&str>,
        channel: Option<&str>,
        count: usize,
    ) -> Result<Vec<MonitoringEntry>, crate::core::error::DbError> {
        Ok(
            log_repo::query_logs_strict(conn, level, channel, count.clamp(1, 500))?
                .into_iter()
                .map(from_repo_entry)
                .collect(),
        )
    }

    pub fn append_entry_with_connection(
        conn: &rusqlite::Connection,
        entry: &MonitoringEntry,
    ) -> Result<(), crate::core::error::DbError> {
        log_repo::insert_log(conn, &to_repo_entry(entry))?;
        Ok(())
    }

    pub fn new(config: LogStorageConfig) -> Self {
        Self::with_db(config, database::DbAccess::Global)
    }

    pub fn with_db(config: LogStorageConfig, db: database::DbAccess) -> Self {
        Self {
            config,
            db,
            buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn append_entry(&self, entry: &MonitoringEntry) {
        self.append_entries(std::slice::from_ref(entry)).await;
    }

    pub async fn append_entry_checked(&self, entry: &MonitoringEntry) -> LogPersistenceTicket {
        let ticket = LogPersistenceTicket {
            entry_id: entry.id.clone(),
            state: Arc::new(AtomicU8::new(0)),
        };
        self.append_buffered(vec![BufferedLogEntry {
            entry: entry.clone(),
            ticket: Some(ticket.clone()),
        }])
        .await;
        ticket
    }

    pub async fn append_entries(&self, entries: &[MonitoringEntry]) {
        self.append_buffered(
            entries
                .iter()
                .cloned()
                .map(|entry| BufferedLogEntry {
                    entry,
                    ticket: None,
                })
                .collect(),
        )
        .await;
    }

    async fn append_buffered(&self, entries: Vec<BufferedLogEntry>) {
        let mut buffer = self.buffer.write().await;
        buffer.extend(entries);

        if buffer.len() >= self.config.flush_threshold {
            let entries = buffer.drain(..).collect::<Vec<_>>();
            drop(buffer);
            self.flush_entries(&entries);
        }
    }

    fn flush_entries(&self, entries: &[BufferedLogEntry]) -> LogFlushReceipt {
        let mut receipt = LogFlushReceipt {
            entry_ids: entries.iter().map(|value| value.entry.id.clone()).collect(),
            ..Default::default()
        };
        if entries.is_empty() {
            return receipt;
        }

        let rows = entries
            .iter()
            .map(|value| to_repo_entry(&value.entry))
            .collect::<Vec<_>>();
        let result = self.db.with_connection(|conn| {
            receipt.attempted = entries.len();
            for value in entries {
                if let Some(ticket) = &value.ticket {
                    ticket.state.store(1, Ordering::Release);
                }
            }
            log_repo::insert_logs_batch(conn, &rows)
        });
        let state = match &result {
            Ok(count) => {
                receipt.committed = *count;
                2
            }
            Err(_) => {
                receipt.failed = entries.len();
                if receipt.attempted == 0 { 3 } else { 4 }
            }
        };
        for value in entries {
            if let Some(ticket) = &value.ticket {
                ticket.state.store(state, Ordering::Release);
            }
        }
        match result {
            Ok(count) => info!(count, "flushed monitoring entries to sqlite"),
            Err(err) => error!(error = %err, "failed to flush monitoring entries"),
        }
        receipt
    }

    pub async fn force_flush(&self) {
        let _ = self.force_flush_checked().await;
    }

    pub async fn force_flush_checked(&self) -> LogFlushReceipt {
        let mut buffer = self.buffer.write().await;
        let entries = buffer.drain(..).collect::<Vec<_>>();
        drop(buffer);
        let mut receipt = self.flush_entries(&entries);
        receipt.remaining_buffered = self.buffer.read().await.len();
        receipt
    }

    pub async fn read_logs_by_date(&self, date: &str) -> Vec<MonitoringEntry> {
        match self
            .db
            .with_connection(|conn| log_repo::get_logs_by_date(conn, date, 10_000))
        {
            Ok(rows) => rows.into_iter().map(from_repo_entry).collect(),
            Err(err) => {
                error!(error = %err, date, "failed to read monitoring entries by date");
                Vec::new()
            }
        }
    }

    pub async fn read_recent_logs(&self, count: usize) -> Vec<MonitoringEntry> {
        match self
            .db
            .with_connection(|conn| log_repo::get_recent_logs(conn, count))
        {
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
        match self
            .db
            .with_connection(|conn| log_repo::query_logs(conn, level, channel, count))
        {
            Ok(rows) => rows.into_iter().map(from_repo_entry).collect(),
            Err(err) => {
                error!(error = %err, "failed to query monitoring entries");
                Vec::new()
            }
        }
    }

    pub async fn cleanup_old_logs(&self) {
        match self
            .db
            .with_connection(|conn| log_repo::delete_old_logs(conn, self.config.retention_days))
        {
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
        match self.db.with_connection(log_repo::get_available_dates) {
            Ok(dates) => dates,
            Err(err) => {
                error!(error = %err, "failed to get monitoring entry dates");
                Vec::new()
            }
        }
    }

    pub async fn delete_logs_by_date(&self, date: &str) -> Result<usize, String> {
        self.db
            .with_connection(|conn| log_repo::delete_logs_by_date(conn, date))
            .map_err(|err| err.to_string())
    }

    pub async fn get_stats(&self) -> LogStats {
        let buffer_count = self.buffer.read().await.len();
        let (today_count, total_count) = match self.db.with_connection(log_repo::get_log_stats) {
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

    fn captured_service(threshold: usize) -> (LogPersistenceService, database::DbPool) {
        let pool = database::create_memory_pool().unwrap();
        pool.get()
            .unwrap()
            .execute_batch(database::schema::CREATE_TABLES_SQL)
            .unwrap();
        let service = LogPersistenceService::with_db(
            LogStorageConfig {
                flush_threshold: threshold,
                ..Default::default()
            },
            database::DbAccess::Pool(pool.clone()),
        );
        (service, pool)
    }

    #[tokio::test]
    async fn checked_tickets_follow_mixed_unit_threshold_and_final_flush() {
        let (service, _) = captured_service(20);
        let checked = sample_entry();
        let ticket = service.append_entry_checked(&checked).await;
        assert_eq!(ticket.entry_id(), checked.id);
        assert_eq!(ticket.state(), LogPersistenceState::Buffered);
        assert!(service.read_recent_logs(50).await.is_empty());
        let ordinary = (0..18).map(|_| sample_entry()).collect::<Vec<_>>();
        service.append_entries(&ordinary).await;
        assert_eq!(ticket.state(), LogPersistenceState::Buffered);
        // A unit caller owns the threshold flush; the checked outcome must survive.
        service.append_entry(&sample_entry()).await;
        assert_eq!(ticket.state(), LogPersistenceState::Committed);
        assert_eq!(service.read_recent_logs(50).await.len(), 20);
        let final_entry = sample_entry();
        let final_ticket = service.append_entry_checked(&final_entry).await;
        let receipt = service.force_flush_checked().await;
        assert_eq!(receipt.entry_ids, [final_entry.id]);
        assert_eq!(
            (
                receipt.attempted,
                receipt.committed,
                receipt.failed,
                receipt.remaining_buffered
            ),
            (1, 1, 0, 0)
        );
        assert_eq!(final_ticket.state(), LogPersistenceState::Committed);
        assert_eq!(service.get_stats().await.buffer_count, 0);
        assert_eq!(
            service.force_flush_checked().await,
            LogFlushReceipt::default()
        );
    }

    #[tokio::test]
    async fn checked_failed_transaction_drops_whole_batch_without_false_commits() {
        let (service, pool) = captured_service(20);
        pool.get().unwrap().execute_batch(
            "CREATE TRIGGER reject_monitoring BEFORE INSERT ON log_entries WHEN NEW.message = 'reject' BEGIN SELECT RAISE(ABORT, 'controlled failure'); END;"
        ).unwrap();
        let checked = sample_entry();
        let ticket = service.append_entry_checked(&checked).await;
        let mut rejected = sample_entry();
        rejected.message = "reject".into();
        service.append_entry(&rejected).await;
        let receipt = service.force_flush_checked().await;
        assert_eq!(receipt.entry_ids, [checked.id, rejected.id]);
        assert_eq!(
            (
                receipt.attempted,
                receipt.committed,
                receipt.failed,
                receipt.remaining_buffered
            ),
            (2, 0, 2, 0)
        );
        assert_eq!(
            ticket.state(),
            LogPersistenceState::Failed { attempted: true }
        );
        assert!(service.read_recent_logs(50).await.is_empty());
        // Existing loss-on-flush-error policy is preserved; later calls still work.
        pool.get()
            .unwrap()
            .execute_batch("DROP TRIGGER reject_monitoring")
            .unwrap();
        service.append_entry(&sample_entry()).await;
        service.force_flush().await;
        assert_eq!(service.read_recent_logs(50).await.len(), 1);
    }

    #[tokio::test]
    async fn unit_force_flush_updates_only_checked_ids_and_drop_finalizes_pending() {
        let (service, _) = captured_service(20);
        let tickets = [
            service.append_entry_checked(&sample_entry()).await,
            service.append_entry_checked(&sample_entry()).await,
        ];
        service
            .append_entries(&[sample_entry(), sample_entry()])
            .await;
        service.clone().force_flush().await;
        assert!(
            tickets
                .iter()
                .all(|ticket| ticket.state() == LogPersistenceState::Committed)
        );
        assert_eq!(service.read_recent_logs(50).await.len(), 4);
        let pending = service.append_entry_checked(&sample_entry()).await;
        drop(service);
        assert_eq!(
            pending.state(),
            LogPersistenceState::Failed { attempted: false }
        );
    }

    #[tokio::test]
    async fn checked_commit_failure_rolls_back_before_reusing_the_connection() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("commit-busy.db");
        let pool = database::create_pool(
            &file,
            Some(database::PoolConfig {
                max_size: 1,
                min_idle: Some(1),
                ..Default::default()
            }),
        )
        .unwrap();
        {
            let conn = pool.get().unwrap();
            conn.execute_batch(database::schema::CREATE_TABLES_SQL)
                .unwrap();
            conn.execute_batch("PRAGMA journal_mode=DELETE").unwrap();
            conn.busy_timeout(std::time::Duration::ZERO).unwrap();
        }
        let reader = rusqlite::Connection::open(&file).unwrap();
        reader.execute_batch("BEGIN DEFERRED").unwrap();
        assert_eq!(
            reader
                .query_row("SELECT COUNT(*) FROM log_entries", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        let service = LogPersistenceService::with_db(
            LogStorageConfig::default(),
            database::DbAccess::Pool(pool.clone()),
        );
        let failed = sample_entry();
        let ticket = service.append_entry_checked(&failed).await;
        let receipt = service.force_flush_checked().await;
        assert_eq!(
            (receipt.attempted, receipt.committed, receipt.failed),
            (1, 0, 1)
        );
        assert_eq!(
            ticket.state(),
            LogPersistenceState::Failed { attempted: true }
        );
        {
            let conn = pool.get().unwrap();
            assert!(
                conn.is_autocommit(),
                "COMMIT BUSY must roll back before returning the pooled connection"
            );
            assert!(log_repo::get_recent_logs(&conn, 10).unwrap().is_empty());
            let error = log_repo::insert_logs_batch(&conn, &[to_repo_entry(&failed)]).unwrap_err();
            assert_eq!(
                error.sqlite_error_code(),
                Some(rusqlite::ErrorCode::DatabaseBusy)
            );
            assert!(conn.is_autocommit());
        }
        reader.execute_batch("ROLLBACK").unwrap();
        let later = sample_entry();
        let later_ticket = service.append_entry_checked(&later).await;
        service.force_flush().await;
        assert_eq!(later_ticket.state(), LogPersistenceState::Committed);
        let rows = service.read_recent_logs(10).await;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, later.id);
        assert!(!rows.iter().any(|row| row.id == failed.id));
    }

    #[tokio::test]
    async fn unit_flush_ticket_stays_attempting_until_real_sqlite_io_finishes() {
        let root = tempfile::tempdir().unwrap();
        let pool = database::create_pool(&root.path().join("ack.db"), None).unwrap();
        let blocker = pool.get().unwrap();
        blocker
            .execute_batch(database::schema::CREATE_TABLES_SQL)
            .unwrap();
        let service = LogPersistenceService::with_db(
            LogStorageConfig::default(),
            database::DbAccess::Pool(pool),
        );
        let ticket = service.append_entry_checked(&sample_entry()).await;
        blocker.execute_batch("BEGIN IMMEDIATE").unwrap();
        let flush = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();
            runtime.block_on(service.force_flush());
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while ticket.state() == LogPersistenceState::Buffered
            && std::time::Instant::now() < deadline
        {
            std::thread::yield_now();
        }
        let during_io = ticket.state();
        let finished_early = flush.is_finished();
        blocker.execute_batch("COMMIT").unwrap();
        flush.join().unwrap();
        assert_eq!(during_io, LogPersistenceState::Attempting);
        assert!(!finished_early);
        assert_eq!(ticket.state(), LogPersistenceState::Committed);
        assert_eq!(log_repo::get_recent_logs(&blocker, 10).unwrap().len(), 1);
    }

    #[test]
    fn checked_uninitialized_global_reports_no_database_attempt_in_child() {
        const MARKER: &str = "CCR_LOG_ACK_UNINITIALIZED_CHILD";
        if std::env::var_os(MARKER).is_some() {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();
            runtime.block_on(async {
                let service = LogPersistenceService::new(LogStorageConfig::default());
                let ticket = service.append_entry_checked(&sample_entry()).await;
                let receipt = service.force_flush_checked().await;
                assert_eq!(
                    (receipt.attempted, receipt.committed, receipt.failed),
                    (0, 0, 1)
                );
                assert_eq!(
                    ticket.state(),
                    LogPersistenceState::Failed { attempted: false }
                );
            });
            return;
        }
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .env(MARKER, "1")
            .args(["--exact", "services::log_persistence::tests::checked_uninitialized_global_reports_no_database_attempt_in_child", "--skip", "export_bindings"])
            .output().unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
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
