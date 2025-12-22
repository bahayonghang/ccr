// 日志持久化服务
// 支持 JSON 文件存储监控日志
#![allow(dead_code)]

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::models::monitoring::LogMessage;

/// 日志存储配置
#[derive(Debug, Clone)]
pub struct LogStorageConfig {
    /// 存储目录
    pub storage_dir: PathBuf,
    /// 每个日志文件最大条目数
    pub max_entries_per_file: usize,
    /// 保留天数
    pub retention_days: u32,
    /// 是否启用压缩
    pub enable_compression: bool,
}

impl Default for LogStorageConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            storage_dir: home.join(".ccr").join("logs"),
            max_entries_per_file: 1000,
            retention_days: 7,
            enable_compression: false,
        }
    }
}

/// 持久化的日志条目
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
        // 将 HashMap 转换为 Option<Value>
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

/// 日志持久化服务
pub struct LogPersistenceService {
    config: LogStorageConfig,
    /// 当前批次的日志（内存缓冲）
    buffer: Arc<RwLock<Vec<PersistedLog>>>,
    /// 缓冲区刷新阈值
    flush_threshold: usize,
}

impl LogPersistenceService {
    pub fn new(config: LogStorageConfig) -> Self {
        // 确保存储目录存在
        if let Err(e) = fs::create_dir_all(&config.storage_dir) {
            error!("Failed to create log storage directory: {}", e);
        }

        Self {
            config,
            buffer: Arc::new(RwLock::new(Vec::new())),
            flush_threshold: 50, // 每 50 条日志刷新一次
        }
    }

    /// 获取当前日期的日志文件路径
    fn get_log_file_path(&self) -> PathBuf {
        let date = Local::now().format("%Y-%m-%d");
        self.config.storage_dir.join(format!("logs_{}.jsonl", date))
    }

    /// 添加日志到缓冲区
    pub async fn append_log(&self, log: &LogMessage) {
        let persisted = PersistedLog::from(log);
        let mut buffer = self.buffer.write().await;
        buffer.push(persisted);

        // 达到阈值时刷新
        if buffer.len() >= self.flush_threshold {
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

        if buffer.len() >= self.flush_threshold {
            let logs_to_flush: Vec<_> = buffer.drain(..).collect();
            drop(buffer);
            self.flush_logs(&logs_to_flush).await;
        }
    }

    /// 刷新缓冲区到文件
    async fn flush_logs(&self, logs: &[PersistedLog]) {
        if logs.is_empty() {
            return;
        }

        let file_path = self.get_log_file_path();

        // 使用追加模式写入
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
        {
            Ok(mut file) => {
                for log in logs {
                    if let Ok(json) = serde_json::to_string(log)
                        && let Err(e) = writeln!(file, "{}", json)
                    {
                        error!("Failed to write log: {}", e);
                    }
                }
                info!("Flushed {} logs to {:?}", logs.len(), file_path);
            }
            Err(e) => {
                error!("Failed to open log file: {}", e);
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
        let file_path = self.config.storage_dir.join(format!("logs_{}.jsonl", date));

        if !file_path.exists() {
            return Vec::new();
        }

        match File::open(&file_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                reader
                    .lines()
                    .map_while(Result::ok)
                    .filter_map(|line| serde_json::from_str::<PersistedLog>(&line).ok())
                    .collect()
            }
            Err(e) => {
                error!("Failed to read log file: {}", e);
                Vec::new()
            }
        }
    }

    /// 读取今天的日志
    pub async fn read_today_logs(&self) -> Vec<PersistedLog> {
        let date = Local::now().format("%Y-%m-%d").to_string();
        self.read_logs_by_date(&date).await
    }

    /// 获取最近 N 条日志
    pub async fn read_recent_logs(&self, count: usize) -> Vec<PersistedLog> {
        let mut all_logs = self.read_today_logs().await;

        // 如果今天的不够，读取昨天的
        if all_logs.len() < count {
            let yesterday = (Local::now() - chrono::Duration::days(1))
                .format("%Y-%m-%d")
                .to_string();
            let yesterday_logs = self.read_logs_by_date(&yesterday).await;
            all_logs = [yesterday_logs, all_logs].concat();
        }

        // 返回最近的 N 条
        let start = all_logs.len().saturating_sub(count);
        all_logs[start..].to_vec()
    }

    /// 清理过期日志
    pub async fn cleanup_old_logs(&self) {
        let retention_cutoff =
            Local::now() - chrono::Duration::days(self.config.retention_days as i64);

        if let Ok(entries) = fs::read_dir(&self.config.storage_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // 解析文件名中的日期
                    if name.starts_with("logs_") && name.ends_with(".jsonl") {
                        let date_str = &name[5..15]; // logs_YYYY-MM-DD.jsonl
                        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                            let file_date = date
                                .and_hms_opt(0, 0, 0)
                                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));

                            if let Some(fd) = file_date
                                && fd < retention_cutoff.with_timezone(&Utc)
                            {
                                if let Err(e) = fs::remove_file(&path) {
                                    error!("Failed to remove old log file: {}", e);
                                } else {
                                    info!("Removed old log file: {:?}", path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// 获取日志统计信息
    pub async fn get_stats(&self) -> LogStats {
        let buffer_count = self.buffer.read().await.len();
        let today_count = self.read_today_logs().await.len();

        let mut total_files = 0;
        let mut total_size = 0u64;

        if let Ok(entries) = fs::read_dir(&self.config.storage_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "jsonl") {
                    total_files += 1;
                    if let Ok(metadata) = fs::metadata(&path) {
                        total_size += metadata.len();
                    }
                }
            }
        }

        LogStats {
            buffer_count,
            today_count,
            total_files,
            total_size_bytes: total_size,
            storage_dir: self.config.storage_dir.to_string_lossy().to_string(),
        }
    }
}

/// 日志统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub buffer_count: usize,
    pub today_count: usize,
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub storage_dir: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::monitoring::LogSource;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_log_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config = LogStorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            max_entries_per_file: 100,
            retention_days: 7,
            enable_compression: false,
        };

        let service = LogPersistenceService::new(config);

        // 创建测试日志
        let log = LogMessage::info(LogSource::System, "Test message");
        service.append_log(&log).await;
        service.force_flush().await;

        // 读取日志
        let logs = service.read_today_logs().await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "Test message");
    }
}
