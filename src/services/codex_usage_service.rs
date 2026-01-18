//! Codex 使用量服务
//!
//! 解析 Codex JSONL 日志文件，计算滚动窗口使用量统计

use crate::core::error::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Codex 使用量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexUsageRecord {
    /// 会话 ID
    pub session_id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 输入 tokens
    pub input_tokens: u64,
    /// 输出 tokens
    pub output_tokens: u64,
    /// 模型名称
    pub model: Option<String>,
}

/// Codex 使用量统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexUsageStats {
    /// 总输入 tokens
    pub total_input_tokens: u64,
    /// 总输出 tokens
    pub total_output_tokens: u64,
    /// 总请求数
    pub total_requests: u64,
    /// 窗口开始时间
    pub window_start: Option<DateTime<Utc>>,
    /// 窗口结束时间
    pub window_end: Option<DateTime<Utc>>,
}

impl CodexUsageStats {
    /// 总 tokens
    #[allow(dead_code)]
    pub fn total_tokens(&self) -> u64 {
        self.total_input_tokens + self.total_output_tokens
    }
}

/// 滚动窗口使用量
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexRollingUsage {
    /// 5小时窗口统计
    pub five_hour: CodexUsageStats,
    /// 7天窗口统计
    pub seven_day: CodexUsageStats,
    /// 全部时间统计
    pub all_time: CodexUsageStats,
    /// 按模型分组的统计
    pub by_model: HashMap<String, CodexUsageStats>,
}

/// JSONL 事件结构 (根据 Codex 日志格式)
#[derive(Debug, Deserialize)]
struct JsonlEvent {
    /// 事件类型 (保留用于过滤)
    #[serde(rename = "type")]
    #[allow(dead_code)]
    event_type: Option<String>,
    timestamp: Option<String>,
    session_id: Option<String>,
    #[serde(default)]
    usage: Option<JsonlUsage>,
    model: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct JsonlUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
}

/// Codex 使用量服务
pub struct CodexUsageService {
    /// Codex 配置目录
    codex_dir: PathBuf,
}

impl CodexUsageService {
    /// 创建新的服务实例
    pub fn new(codex_dir: PathBuf) -> Self {
        Self { codex_dir }
    }

    /// 获取日志目录
    fn logs_dir(&self) -> PathBuf {
        self.codex_dir.join("logs")
    }

    /// 解析所有 JSONL 日志文件
    pub fn parse_all_logs(&self) -> Result<Vec<CodexUsageRecord>> {
        let logs_dir = self.logs_dir();
        if !logs_dir.exists() {
            return Ok(Vec::new());
        }

        let mut records = Vec::new();

        // 读取所有 .jsonl 文件
        let entries = std::fs::read_dir(&logs_dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jsonl")
                && let Ok(file_records) = self.parse_jsonl_file(&path)
            {
                records.extend(file_records);
            }
        }

        // 按时间排序
        records.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(records)
    }

    /// 解析单个 JSONL 文件
    fn parse_jsonl_file(&self, path: &PathBuf) -> Result<Vec<CodexUsageRecord>> {
        let file = File::open(path)?;

        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };

            if line.trim().is_empty() {
                continue;
            }

            // 解析 JSON
            let event: JsonlEvent = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(_) => continue,
            };

            // 只处理有 usage 数据的事件
            if let Some(usage) = event.usage
                && (usage.input_tokens > 0 || usage.output_tokens > 0)
            {
                // 解析时间戳
                let timestamp = event
                    .timestamp
                    .as_ref()
                    .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);

                records.push(CodexUsageRecord {
                    session_id: event.session_id.unwrap_or_default(),
                    timestamp,
                    input_tokens: usage.input_tokens,
                    output_tokens: usage.output_tokens,
                    model: event.model,
                });
            }
        }

        Ok(records)
    }

    /// 计算滚动窗口使用量
    pub fn compute_rolling_usage(&self) -> Result<CodexRollingUsage> {
        let records = self.parse_all_logs()?;
        let now = Utc::now();

        let five_hours_ago = now - Duration::hours(5);
        let seven_days_ago = now - Duration::days(7);

        let mut rolling = CodexRollingUsage::default();

        for record in &records {
            // 全部时间
            Self::add_to_stats(&mut rolling.all_time, record);

            // 7天窗口
            if record.timestamp >= seven_days_ago {
                Self::add_to_stats(&mut rolling.seven_day, record);
            }

            // 5小时窗口
            if record.timestamp >= five_hours_ago {
                Self::add_to_stats(&mut rolling.five_hour, record);
            }

            // 按模型分组
            if let Some(model) = &record.model {
                let model_stats = rolling
                    .by_model
                    .entry(model.clone())
                    .or_insert_with(CodexUsageStats::default);
                Self::add_to_stats(model_stats, record);
            }
        }

        // 设置窗口时间
        rolling.five_hour.window_start = Some(five_hours_ago);
        rolling.five_hour.window_end = Some(now);
        rolling.seven_day.window_start = Some(seven_days_ago);
        rolling.seven_day.window_end = Some(now);

        if let Some(first) = records.first() {
            rolling.all_time.window_start = Some(first.timestamp);
        }
        rolling.all_time.window_end = Some(now);

        Ok(rolling)
    }

    /// 添加记录到统计
    fn add_to_stats(stats: &mut CodexUsageStats, record: &CodexUsageRecord) {
        stats.total_input_tokens += record.input_tokens;
        stats.total_output_tokens += record.output_tokens;
        stats.total_requests += 1;
    }

    /// 格式化 tokens 数量 (带 K/M 后缀)
    pub fn format_tokens(tokens: u64) -> String {
        if tokens >= 1_000_000 {
            format!("{:.1}M", tokens as f64 / 1_000_000.0)
        } else if tokens >= 1_000 {
            format!("{:.1}K", tokens as f64 / 1_000.0)
        } else {
            tokens.to_string()
        }
    }

    /// 获取使用量摘要文本
    #[allow(dead_code)]
    pub fn get_usage_summary(&self) -> Result<String> {
        let rolling = self.compute_rolling_usage()?;

        let mut summary = String::new();
        summary.push_str("📊 Codex 使用量统计\n\n");

        summary.push_str(&format!(
            "5小时窗口: {} tokens ({} 输入 / {} 输出) - {} 请求\n",
            Self::format_tokens(rolling.five_hour.total_tokens()),
            Self::format_tokens(rolling.five_hour.total_input_tokens),
            Self::format_tokens(rolling.five_hour.total_output_tokens),
            rolling.five_hour.total_requests
        ));

        summary.push_str(&format!(
            "7天窗口:   {} tokens ({} 输入 / {} 输出) - {} 请求\n",
            Self::format_tokens(rolling.seven_day.total_tokens()),
            Self::format_tokens(rolling.seven_day.total_input_tokens),
            Self::format_tokens(rolling.seven_day.total_output_tokens),
            rolling.seven_day.total_requests
        ));

        summary.push_str(&format!(
            "全部时间:  {} tokens ({} 输入 / {} 输出) - {} 请求\n",
            Self::format_tokens(rolling.all_time.total_tokens()),
            Self::format_tokens(rolling.all_time.total_input_tokens),
            Self::format_tokens(rolling.all_time.total_output_tokens),
            rolling.all_time.total_requests
        ));

        if !rolling.by_model.is_empty() {
            summary.push_str("\n按模型统计:\n");
            for (model, stats) in &rolling.by_model {
                summary.push_str(&format!(
                    "  {}: {} tokens - {} 请求\n",
                    model,
                    Self::format_tokens(stats.total_tokens()),
                    stats.total_requests
                ));
            }
        }

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use tempfile::TempDir;

    fn create_test_service() -> (CodexUsageService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let service = CodexUsageService::new(temp_dir.path().to_path_buf());
        (service, temp_dir)
    }

    #[test]
    fn test_empty_logs() {
        let (service, _temp) = create_test_service();
        let records = service.parse_all_logs().unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn test_format_tokens() {
        assert_eq!(CodexUsageService::format_tokens(500), "500");
        assert_eq!(CodexUsageService::format_tokens(1500), "1.5K");
        assert_eq!(CodexUsageService::format_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn test_parse_jsonl_with_usage() {
        let (service, temp_dir) = create_test_service();

        // 创建 logs 目录
        let logs_dir = temp_dir.path().join("logs");
        std::fs::create_dir_all(&logs_dir).unwrap();

        // 创建测试 JSONL 文件
        let jsonl_content = r#"{"type":"response","timestamp":"2026-01-15T10:00:00Z","session_id":"sess-1","usage":{"input_tokens":100,"output_tokens":50},"model":"gpt-4"}
{"type":"response","timestamp":"2026-01-15T11:00:00Z","session_id":"sess-1","usage":{"input_tokens":200,"output_tokens":100},"model":"gpt-4"}
{"type":"other","timestamp":"2026-01-15T12:00:00Z"}
"#;
        std::fs::write(logs_dir.join("test.jsonl"), jsonl_content).unwrap();

        let records = service.parse_all_logs().unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].input_tokens, 100);
        assert_eq!(records[0].output_tokens, 50);
        assert_eq!(records[1].input_tokens, 200);
        assert_eq!(records[1].output_tokens, 100);
    }

    #[test]
    fn test_compute_rolling_usage() {
        let (service, temp_dir) = create_test_service();

        // 创建 logs 目录
        let logs_dir = temp_dir.path().join("logs");
        std::fs::create_dir_all(&logs_dir).unwrap();

        // 创建测试数据 - 使用当前时间
        let now = Utc::now();
        let recent = now - Duration::hours(1);

        let jsonl_content = format!(
            r#"{{"type":"response","timestamp":"{}","session_id":"sess-1","usage":{{"input_tokens":100,"output_tokens":50}},"model":"gpt-4"}}
"#,
            recent.to_rfc3339()
        );
        std::fs::write(logs_dir.join("test.jsonl"), jsonl_content).unwrap();

        let rolling = service.compute_rolling_usage().unwrap();
        assert_eq!(rolling.five_hour.total_requests, 1);
        assert_eq!(rolling.five_hour.total_input_tokens, 100);
        assert_eq!(rolling.five_hour.total_output_tokens, 50);
        assert_eq!(rolling.seven_day.total_requests, 1);
        assert_eq!(rolling.all_time.total_requests, 1);
    }

    #[test]
    fn test_usage_stats_total_tokens() {
        let stats = CodexUsageStats {
            total_input_tokens: 100,
            total_output_tokens: 50,
            total_requests: 1,
            window_start: None,
            window_end: None,
        };
        assert_eq!(stats.total_tokens(), 150);
    }
}
