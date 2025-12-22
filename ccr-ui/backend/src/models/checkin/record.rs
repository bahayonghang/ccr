// 签到记录模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 签到状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckinStatus {
    /// 签到成功
    Success,
    /// 今日已签到
    AlreadyCheckedIn,
    /// 签到失败
    Failed,
}

impl std::fmt::Display for CheckinStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckinStatus::Success => write!(f, "成功"),
            CheckinStatus::AlreadyCheckedIn => write!(f, "今日已签到"),
            CheckinStatus::Failed => write!(f, "失败"),
        }
    }
}

/// 签到记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinRecord {
    /// 唯一标识符 (UUID)
    pub id: String,
    /// 关联的账号 ID
    pub account_id: String,
    /// 签到状态
    pub status: CheckinStatus,
    /// 消息 (成功消息或错误信息)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// 签到奖励信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward: Option<String>,
    /// 签到前余额
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_before: Option<f64>,
    /// 签到后余额
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_after: Option<f64>,
    /// 签到时间
    pub checked_in_at: DateTime<Utc>,
}

impl CheckinRecord {
    /// 创建成功的签到记录
    pub fn success(account_id: String, message: Option<String>, reward: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            account_id,
            status: CheckinStatus::Success,
            message,
            reward,
            balance_before: None,
            balance_after: None,
            checked_in_at: Utc::now(),
        }
    }

    /// 创建已签到的记录
    pub fn already_checked_in(account_id: String, message: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            account_id,
            status: CheckinStatus::AlreadyCheckedIn,
            message,
            reward: None,
            balance_before: None,
            balance_after: None,
            checked_in_at: Utc::now(),
        }
    }

    /// 创建失败的签到记录
    pub fn failed(account_id: String, error: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            account_id,
            status: CheckinStatus::Failed,
            message: Some(error),
            reward: None,
            balance_before: None,
            balance_after: None,
            checked_in_at: Utc::now(),
        }
    }

    /// 设置余额变化
    #[allow(dead_code)]
    pub fn with_balance(mut self, before: Option<f64>, after: Option<f64>) -> Self {
        self.balance_before = before;
        self.balance_after = after;
        self
    }

    /// 计算余额变化
    pub fn balance_change(&self) -> Option<f64> {
        match (self.balance_before, self.balance_after) {
            (Some(before), Some(after)) => Some(after - before),
            _ => None,
        }
    }
}

/// 签到记录显示信息 (包含账号和提供商名称)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinRecordInfo {
    /// 唯一标识符
    pub id: String,
    /// 关联的账号 ID
    pub account_id: String,
    /// 账号名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_name: Option<String>,
    /// 提供商名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_name: Option<String>,
    /// 签到状态
    pub status: CheckinStatus,
    /// 消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// 奖励信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward: Option<String>,
    /// 签到前余额
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_before: Option<f64>,
    /// 签到后余额
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_after: Option<f64>,
    /// 余额变化
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_change: Option<f64>,
    /// 签到时间
    pub checked_in_at: DateTime<Utc>,
}

impl From<CheckinRecord> for CheckinRecordInfo {
    fn from(record: CheckinRecord) -> Self {
        let balance_change = record.balance_change();
        Self {
            id: record.id,
            account_id: record.account_id,
            account_name: None,
            provider_name: None,
            status: record.status,
            message: record.message,
            reward: record.reward,
            balance_before: record.balance_before,
            balance_after: record.balance_after,
            balance_change,
            checked_in_at: record.checked_in_at,
        }
    }
}

/// 签到记录列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinRecordsResponse {
    /// 记录列表 (按时间倒序)
    pub records: Vec<CheckinRecordInfo>,
    /// 总记录数
    pub total: usize,
}

/// 单次签到响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CheckinResponse {
    /// 签到状态
    pub status: CheckinStatus,
    /// 消息
    pub message: String,
    /// 奖励信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward: Option<String>,
    /// 当前余额
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<f64>,
    /// 余额变化
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_change: Option<f64>,
}

/// 批量签到结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BatchCheckinResult {
    /// 账号 ID
    pub account_id: String,
    /// 账号名称
    pub account_name: String,
    /// 签到状态
    pub status: CheckinStatus,
    /// 消息
    pub message: String,
    /// 奖励信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward: Option<String>,
}

/// 批量签到响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BatchCheckinResponse {
    /// 签到结果列表
    pub results: Vec<BatchCheckinResult>,
    /// 成功数量
    pub success_count: usize,
    /// 已签到数量
    pub already_checked_in_count: usize,
    /// 失败数量
    pub failed_count: usize,
    /// 总数量
    pub total: usize,
}

/// 今日签到状态概览
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CheckinStatusOverview {
    /// 总账号数
    pub total_accounts: usize,
    /// 今日已签到数
    pub checked_in_today: usize,
    /// 今日未签到数
    pub not_checked_in: usize,
    /// 今日签到失败数
    pub failed_today: usize,
    /// 启用的账号数
    pub enabled_accounts: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkin_record_success() {
        let record = CheckinRecord::success(
            "account-1".to_string(),
            Some("签到成功".to_string()),
            Some("+10 积分".to_string()),
        );

        assert!(!record.id.is_empty());
        assert_eq!(record.status, CheckinStatus::Success);
        assert_eq!(record.reward, Some("+10 积分".to_string()));
    }

    #[test]
    fn test_checkin_record_with_balance() {
        let record = CheckinRecord::success("account-1".to_string(), None, None)
            .with_balance(Some(100.0), Some(110.0));

        assert_eq!(record.balance_before, Some(100.0));
        assert_eq!(record.balance_after, Some(110.0));
        assert_eq!(record.balance_change(), Some(10.0));
    }

    #[test]
    fn test_checkin_status_display() {
        assert_eq!(format!("{}", CheckinStatus::Success), "成功");
        assert_eq!(format!("{}", CheckinStatus::AlreadyCheckedIn), "今日已签到");
        assert_eq!(format!("{}", CheckinStatus::Failed), "失败");
    }
}
