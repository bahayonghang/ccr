// 余额快照模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 余额快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSnapshot {
    /// 唯一标识符 (UUID)
    pub id: String,
    /// 关联的账号 ID
    pub account_id: String,
    /// 总配额
    pub total_quota: f64,
    /// 已使用配额
    pub used_quota: f64,
    /// 剩余配额
    pub remaining_quota: f64,
    /// 货币/单位 (USD, CNY, tokens, credits)
    #[serde(default = "default_currency")]
    pub currency: String,
    /// 原始响应 (可选，用于调试)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_response: Option<String>,
    /// 记录时间
    pub recorded_at: DateTime<Utc>,
}

fn default_currency() -> String {
    "USD".to_string()
}

impl BalanceSnapshot {
    /// 创建新的余额快照
    pub fn new(
        account_id: String,
        total_quota: f64,
        used_quota: f64,
        remaining_quota: f64,
        currency: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            account_id,
            total_quota,
            used_quota,
            remaining_quota,
            currency,
            raw_response: None,
            recorded_at: Utc::now(),
        }
    }

    /// 带原始响应创建快照
    pub fn with_raw_response(mut self, raw: String) -> Self {
        self.raw_response = Some(raw);
        self
    }

    /// 计算使用百分比
    #[allow(dead_code)]
    pub fn usage_percentage(&self) -> f64 {
        if self.total_quota <= 0.0 {
            0.0
        } else {
            (self.used_quota / self.total_quota) * 100.0
        }
    }
}

/// 当前余额响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BalanceResponse {
    /// 账号 ID
    pub account_id: String,
    /// 总配额
    pub total_quota: f64,
    /// 已使用配额
    pub used_quota: f64,
    /// 剩余配额
    pub remaining_quota: f64,
    /// 货币/单位
    pub currency: String,
    /// 使用百分比
    pub usage_percentage: f64,
    /// 记录时间
    pub recorded_at: DateTime<Utc>,
}

impl From<BalanceSnapshot> for BalanceResponse {
    fn from(snapshot: BalanceSnapshot) -> Self {
        let usage_percentage = snapshot.usage_percentage();
        Self {
            account_id: snapshot.account_id,
            total_quota: snapshot.total_quota,
            used_quota: snapshot.used_quota,
            remaining_quota: snapshot.remaining_quota,
            currency: snapshot.currency,
            usage_percentage,
            recorded_at: snapshot.recorded_at,
        }
    }
}

/// 余额历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceHistoryItem {
    /// 总配额
    pub total_quota: f64,
    /// 已使用配额
    pub used_quota: f64,
    /// 剩余配额
    pub remaining_quota: f64,
    /// 货币/单位
    pub currency: String,
    /// 记录时间
    pub recorded_at: DateTime<Utc>,
    /// 与上一条记录的余额变化 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<f64>,
}

impl From<&BalanceSnapshot> for BalanceHistoryItem {
    fn from(snapshot: &BalanceSnapshot) -> Self {
        Self {
            total_quota: snapshot.total_quota,
            used_quota: snapshot.used_quota,
            remaining_quota: snapshot.remaining_quota,
            currency: snapshot.currency.clone(),
            recorded_at: snapshot.recorded_at,
            change: None,
        }
    }
}

/// 余额历史响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceHistoryResponse {
    /// 账号 ID
    pub account_id: String,
    /// 历史记录列表 (按时间倒序)
    pub history: Vec<BalanceHistoryItem>,
    /// 总记录数
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_snapshot() {
        let snapshot = BalanceSnapshot::new(
            "account-1".to_string(),
            100.0,
            30.0,
            70.0,
            "USD".to_string(),
        );

        assert!(!snapshot.id.is_empty());
        assert_eq!(snapshot.account_id, "account-1");
        assert_eq!(snapshot.usage_percentage(), 30.0);
    }

    #[test]
    fn test_usage_percentage_zero_total() {
        let snapshot =
            BalanceSnapshot::new("account-1".to_string(), 0.0, 0.0, 0.0, "USD".to_string());

        assert_eq!(snapshot.usage_percentage(), 0.0);
    }

    #[test]
    fn test_balance_response_from_snapshot() {
        let snapshot = BalanceSnapshot::new(
            "account-1".to_string(),
            100.0,
            25.0,
            75.0,
            "CNY".to_string(),
        );

        let response: BalanceResponse = snapshot.into();
        assert_eq!(response.account_id, "account-1");
        assert_eq!(response.usage_percentage, 25.0);
        assert_eq!(response.currency, "CNY");
    }
}
