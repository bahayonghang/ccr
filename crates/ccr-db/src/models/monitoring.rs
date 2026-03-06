// 监控相关数据模型
// 用于实时日志、Token 统计和代理状态
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 日志级别
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

/// 日志来源
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogSource {
    Claude,
    Codex,
    Gemini,
    Qwen,
    IFlow,
    #[default]
    System,
    Proxy,
}

/// 实时日志消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    /// 唯一标识
    pub id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 日志级别
    pub level: LogLevel,
    /// 日志来源
    pub source: LogSource,
    /// 消息内容
    pub message: String,
    /// 额外元数据
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl LogMessage {
    pub fn new(level: LogLevel, source: LogSource, message: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            source,
            message: message.into(),
            metadata: HashMap::new(),
        }
    }

    pub fn info(source: LogSource, message: impl Into<String>) -> Self {
        Self::new(LogLevel::Info, source, message)
    }

    pub fn error(source: LogSource, message: impl Into<String>) -> Self {
        Self::new(LogLevel::Error, source, message)
    }
}

/// Token 使用统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenStats {
    /// 输入 token 数量
    pub input_tokens: u64,
    /// 输出 token 数量
    pub output_tokens: u64,
    /// 缓存 token 数量（如果支持）
    pub cache_tokens: u64,
    /// 总请求次数
    pub request_count: u64,
    /// 估算成本（美分）
    pub estimated_cost_cents: f64,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

impl TokenStats {
    pub fn new() -> Self {
        Self {
            last_updated: Utc::now(),
            ..Default::default()
        }
    }

    /// 添加 token 使用量
    pub fn add_usage(&mut self, input: u64, output: u64, cache: u64) {
        self.input_tokens += input;
        self.output_tokens += output;
        self.cache_tokens += cache;
        self.request_count += 1;
        self.last_updated = Utc::now();

        // 估算成本 (Claude 定价近似: $3/MTok input, $15/MTok output)
        self.estimated_cost_cents =
            (self.input_tokens as f64 * 0.0003) + (self.output_tokens as f64 * 0.0015);
    }

    /// 获取总 token 数量
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

/// 代理状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyState {
    /// 是否运行中
    pub running: bool,
    /// 监听端口
    pub port: u16,
    /// 当前连接数
    pub connections: u32,
    /// 启动时间
    pub started_at: Option<DateTime<Utc>>,
    /// 各平台的 token 统计
    pub platform_stats: HashMap<String, TokenStats>,
}

/// WebSocket 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// 日志消息
    Log(LogMessage),
    /// Token 统计更新
    TokenStats(TokenStats),
    /// 代理状态更新
    ProxyState(ProxyState),
    /// 心跳 ping
    Ping,
    /// 心跳 pong
    Pong,
    /// 错误消息
    Error { message: String },
    /// 历史日志批量发送
    LogBatch(Vec<LogMessage>),
}

impl WsMessage {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_message() {
        let log = LogMessage::info(LogSource::Claude, "Test message");
        assert_eq!(log.level, LogLevel::Info);
        assert_eq!(log.source, LogSource::Claude);
        assert_eq!(log.message, "Test message");
    }

    #[test]
    fn test_token_stats() {
        let mut stats = TokenStats::new();
        stats.add_usage(1000, 500, 0);
        assert_eq!(stats.input_tokens, 1000);
        assert_eq!(stats.output_tokens, 500);
        assert_eq!(stats.request_count, 1);
        assert_eq!(stats.total_tokens(), 1500);
    }
}
