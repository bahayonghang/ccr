// 签到账号模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 签到账号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinAccount {
    /// 唯一标识符 (UUID)
    pub id: String,
    /// 关联的提供商 ID
    pub provider_id: String,
    /// 账号备注名称
    pub name: String,
    /// 加密的 API Key (AES-256-GCM 加密后 Base64 编码)
    pub api_key_encrypted: String,
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    /// 最后签到时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checkin_at: Option<DateTime<Utc>>,
    /// 最后余额检查时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_balance_check_at: Option<DateTime<Utc>>,
}

fn default_enabled() -> bool {
    true
}

impl CheckinAccount {
    /// 创建新账号
    pub fn new(provider_id: String, name: String, api_key_encrypted: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            provider_id,
            name,
            api_key_encrypted,
            enabled: true,
            created_at: Utc::now(),
            updated_at: None,
            last_checkin_at: None,
            last_balance_check_at: None,
        }
    }

    /// 更新最后签到时间
    pub fn update_checkin_time(&mut self) {
        self.last_checkin_at = Some(Utc::now());
        self.updated_at = Some(Utc::now());
    }

    /// 更新最后余额检查时间
    pub fn update_balance_check_time(&mut self) {
        self.last_balance_check_at = Some(Utc::now());
        self.updated_at = Some(Utc::now());
    }
}

/// 账号显示信息 (用于 API 响应，隐藏加密密钥)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    /// 唯一标识符
    pub id: String,
    /// 关联的提供商 ID
    pub provider_id: String,
    /// 提供商名称 (可选，用于显示)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_name: Option<String>,
    /// 账号备注名称
    pub name: String,
    /// 遮罩后的 API Key (如 "sk-****cdef")
    pub api_key_masked: String,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后签到时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checkin_at: Option<DateTime<Utc>>,
    /// 最后余额检查时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_balance_check_at: Option<DateTime<Utc>>,
    /// 最新余额 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_balance: Option<f64>,
    /// 余额货币单位 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_currency: Option<String>,
}

/// 创建账号请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    /// 关联的提供商 ID
    pub provider_id: String,
    /// 账号备注名称
    pub name: String,
    /// API Key (明文，将被加密存储)
    pub api_key: String,
}

/// 更新账号请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    /// 账号备注名称 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// API Key (可选，明文，将被加密存储)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// 是否启用 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// 账号列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsResponse {
    /// 账号列表 (已遮罩 API Key)
    pub accounts: Vec<AccountInfo>,
    /// 总数
    pub total: usize,
}

/// 测试连接响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TestConnectionResponse {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 用户信息 (如果成功)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_info: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_account() {
        let account = CheckinAccount::new(
            "provider-1".to_string(),
            "Main Account".to_string(),
            "encrypted-key".to_string(),
        );

        assert!(!account.id.is_empty());
        assert_eq!(account.provider_id, "provider-1");
        assert_eq!(account.name, "Main Account");
        assert!(account.enabled);
        assert!(account.last_checkin_at.is_none());
    }

    #[test]
    fn test_update_checkin_time() {
        let mut account = CheckinAccount::new(
            "provider-1".to_string(),
            "Test".to_string(),
            "key".to_string(),
        );

        assert!(account.last_checkin_at.is_none());
        account.update_checkin_time();
        assert!(account.last_checkin_at.is_some());
        assert!(account.updated_at.is_some());
    }
}
