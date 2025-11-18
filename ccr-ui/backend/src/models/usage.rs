// Usage Analytics Data Models
// 用于读取和解析 Claude Code 的 usage 日志

use serde::{Deserialize, Serialize};

/// Token 使用数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageData {
    /// 输入 token 数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,

    /// 输出 token 数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,

    /// 缓存读取的 token 数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
}

impl UsageData {
    /// 计算总 token 数量
    #[allow(dead_code)]
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens.unwrap_or(0)
            + self.output_tokens.unwrap_or(0)
            + self.cache_read_input_tokens.unwrap_or(0)
    }

    /// 检查是否有效（至少有一个 token > 0）
    pub fn is_valid(&self) -> bool {
        self.input_tokens.unwrap_or(0) > 0 || self.output_tokens.unwrap_or(0) > 0
    }
}

/// 单条 usage 记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    /// 唯一标识符
    pub uuid: String,

    /// 时间戳 (ISO 8601 格式)
    pub timestamp: String,

    /// 模型名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Token 使用数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageData>,
}

impl UsageRecord {
    /// 检查记录是否有效
    pub fn is_valid(&self) -> bool {
        !self.uuid.is_empty()
            && !self.timestamp.is_empty()
            && self.usage.as_ref().map_or(false, |u| u.is_valid())
    }
}

/// Usage 记录响应
#[derive(Debug, Serialize, Deserialize)]
pub struct UsageRecordsResponse {
    /// Usage 记录列表
    pub records: Vec<UsageRecord>,

    /// 总记录数（实际扫描到的）
    pub total_records: usize,

    /// 是否被截断（超过 limit）
    pub truncated: bool,
}

impl UsageRecordsResponse {
    pub fn new(records: Vec<UsageRecord>, total_records: usize, limit: usize) -> Self {
        let truncated = total_records > limit;
        Self {
            records,
            total_records,
            truncated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_data_total_tokens() {
        let usage = UsageData {
            input_tokens: Some(100),
            output_tokens: Some(50),
            cache_read_input_tokens: Some(25),
        };
        assert_eq!(usage.total_tokens(), 175);
    }

    #[test]
    fn test_usage_data_is_valid() {
        let valid = UsageData {
            input_tokens: Some(100),
            output_tokens: Some(0),
            cache_read_input_tokens: None,
        };
        assert!(valid.is_valid());

        let invalid = UsageData {
            input_tokens: Some(0),
            output_tokens: Some(0),
            cache_read_input_tokens: None,
        };
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_usage_record_is_valid() {
        let valid = UsageRecord {
            uuid: "abc123".to_string(),
            timestamp: "2025-11-18T10:30:00Z".to_string(),
            model: Some("claude-sonnet-4-5".to_string()),
            usage: Some(UsageData {
                input_tokens: Some(100),
                output_tokens: Some(50),
                cache_read_input_tokens: None,
            }),
        };
        assert!(valid.is_valid());

        let invalid = UsageRecord {
            uuid: "".to_string(),
            timestamp: "2025-11-18T10:30:00Z".to_string(),
            model: None,
            usage: None,
        };
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_usage_records_response_truncation() {
        let records = vec![];
        let response = UsageRecordsResponse::new(records, 15000, 10000);
        assert!(response.truncated);
        assert_eq!(response.total_records, 15000);
    }
}
