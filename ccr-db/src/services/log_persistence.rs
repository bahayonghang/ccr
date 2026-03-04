// 日志持久化服务
// 使用 SQLite 统一存储监控日志
#![allow(dead_code)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::database::{self, repositories::log_repo};
use crate::models::monitoring::LogMessage;

/// 日志存储配置
#[derive(Debug, Clone)]
pub struct LogStorageConfig {
    /// 保留天数
    pub retention_days: i64,
    /// 缓冲区刷新阈值
    pub flush_threshold: usize,
}

impl Default for LogStorageConfig {
    fn default() -> Self {
        Self {
            retention_days: 7,
            flush_threshold: 50,
        }
    }
}

/// 持久化的日志条目（兼容旧格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedLog {
    pub id: String,
    pub timestamp: String,
    pub level: String,
    pub source: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

impl From<&LogMessage> for PersistedLog {
    fn from(log: &LogMessage) -> Self {
        let metadata_value = if log.metadata.is_empty() {
            None
        } else {
            serde_json::to_value(&log.metadata).ok()
        };

        Self {
            id: log.id.clone(),
            timestamp: log.timestamp.to_rfc3339(),
            level: format!("{:?}", log.level),
            source: format!("{:?}", log.source),
            message: log.message.clone(),
            metadata: metadata_value,
        }
    }
}

impl From<log_repo::LogEntry> for PersistedLog {
    fn from(entry: log_repo::LogEntry) -> Self {
        let metadata = entry
            .metadata_json
            .and_then(|s| serde_json::from_str(&s).ok());

        Self {
            id: entry.id,
            timestamp: entry.timestamp.to_rfc3339(),
            level: entry.level,
            source: entry.source,
            message: entry.message,
            metadata,
        }
    }
}

impl From<&PersistedLog> for log_repo::LogEntry {
    fn from(log: &PersistedLog) -> Self {
        let timestamp = chrono::DateTime::parse_from_rfc3339(&log.timestamp)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let metadata_json = log
            .metadata
            .as_ref()
            .and_then(|v| serde_json::to_string(v).ok());

        Self {
            id: log.id.clone(),
            timestamp,
            level: log.level.clone(),
            source: log.source.clone(),
            message: log.message.clone(),
            metadata_json,
        }
    }
}

/// 日志持久化服务
/// 使用 SQLite 统一存储
pub struct LogPersistenceService {
    config: LogStorageConfig,
    /// 当前批次的日志（内存缓冲）
    buffer: Arc<RwLock<Vec<PersistedLog>>>,
}

impl LogPersistenceService {
    pub fn new(config: LogStorageConfig) -> Self {
        Self {
            config,
            buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 添加日志到缓冲区
    pub async fn append_log(&self, log: &LogMessage) {
        let persisted = PersistedLog::from(log);
        let mut buffer = self.buffer.write().await;
        buffer.push(persisted);

        // 达到阈值时刷新
        if buffer.len() >= self.config.flush_threshold {
            let logs_to_flush: Vec<_> = buffer.drain(..).collect();
            drop(buffer); // 释放锁
            self.flush_logs(&logs_to_flush).await;
        }
    }

    /// 批量添加日志
    pub async fn append_logs(&self, logs: &[LogMessage]) {
        let persisted: Vec<PersistedLog> = logs.iter().map(PersistedLog::from).collect();
        let mut buffer = self.buffer.write().await;
        buffer.extend(persisted);

        if buffer.len() >= self.config.flush_threshold {
            let logs_to_flush: Vec<_> = buffer.drain(..).collect();
            drop(buffer);
            self.flush_logs(&logs_to_flush).await;
        }
    }

    /// 刷新缓冲区到数据库
    async fn flush_logs(&self, logs: &[PersistedLog]) {
        if logs.is_empty() {
            return;
        }

        let entries: Vec<log_repo::LogEntry> = logs.iter().map(|l| l.into()).collect();

        match database::with_connection(|conn| log_repo::insert_logs_batch(conn, &entries)) {
            Ok(count) => {
                info!("Flushed {} logs to SQLite", count);
            }
            Err(e) => {
                error!("Failed to flush logs to database: {}", e);
            }
        }
    }

    /// 强制刷新缓冲区
    pub async fn force_flush(&self) {
        let mut buffer = self.buffer.write().await;
        let logs_to_flush: Vec<_> = buffer.drain(..).collect();
        drop(buffer);
        self.flush_logs(&logs_to_flush).await;
    }

    /// 读取指定日期的日志
    pub async fn read_logs_by_date(&self, date: &str) -> Vec<PersistedLog> {
        match database::with_connection(|conn| log_repo::get_logs_by_date(conn, date, 10000)) {
            Ok(entries) => entries.into_iter().map(PersistedLog::from).collect(),
            Err(e) => {
                error!("Failed to read logs by date: {}", e);
                Vec::new()
            }
        }
    }

    /// 读取今天的日志
    pub async fn read_today_logs(&self) -> Vec<PersistedLog> {
        let date = Utc::now().format("%Y-%m-%d").to_string();
        self.read_logs_by_date(&date).await
    }

    /// 获取最近 N 条日志
    pub async fn read_recent_logs(&self, count: usize) -> Vec<PersistedLog> {
        match database::with_connection(|conn| log_repo::get_recent_logs(conn, count)) {
            Ok(entries) => entries.into_iter().map(PersistedLog::from).collect(),
            Err(e) => {
                error!("Failed to read recent logs: {}", e);
                Vec::new()
            }
        }
    }

    /// 清理过期日志
    pub async fn cleanup_old_logs(&self) {
        match database::with_connection(|conn| {
            log_repo::delete_old_logs(conn, self.config.retention_days)
        }) {
            Ok(deleted) => {
                if deleted > 0 {
                    info!(
                        "Cleaned up {} old log entries (retention: {} days)",
                        deleted, self.config.retention_days
                    );
                }
            }
            Err(e) => {
                error!("Failed to cleanup old logs: {}", e);
            }
        }
    }

    /// 获取可用的日志日期列表
    pub async fn get_available_dates(&self) -> Vec<String> {
        match database::with_connection(log_repo::get_available_dates) {
            Ok(dates) => dates,
            Err(e) => {
                error!("Failed to get available dates: {}", e);
                Vec::new()
            }
        }
    }

    /// 删除指定日期的日志
    pub async fn delete_logs_by_date(&self, date: &str) -> Result<usize, String> {
        database::with_connection(|conn| log_repo::delete_logs_by_date(conn, date))
            .map_err(|e| e.to_string())
    }

    /// 获取日志统计信息
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

/// 日志统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub buffer_count: usize,
    pub today_count: usize,
    pub total_count: usize,
    pub storage_type: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::database;
    use crate::models::monitoring::LogSource;

    #[tokio::test]
    async fn test_log_persistence() {
        // Initialize test database
        database::initialize_for_test().unwrap();

        let config = LogStorageConfig {
            retention_days: 7,
            flush_threshold: 1, // Flush immediately for testing
        };

        let service = LogPersistenceService::new(config);

        // 创建测试日志
        let log = LogMessage::info(LogSource::System, "Test message");
        service.append_log(&log).await;
        service.force_flush().await;

        // 读取日志
        let logs = service.read_today_logs().await;
        assert!(!logs.is_empty());
        assert_eq!(logs.last().unwrap().message, "Test message");
    }
}
