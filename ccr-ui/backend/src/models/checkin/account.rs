// 签到账号模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 签到账号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinAccount {
    /// 唯一标识符 (UUID)
    pub id: String,
    /// 关联的提供商 ID
    pub provider_id: String,
    /// 账号备注名称
    pub name: String,
    /// 加密的 Cookies JSON (AES-256-GCM 加密后 Base64 编码)
    /// 存储格式: {"session": "abc123", "token": "xyz789"}
    pub cookies_json_encrypted: String,
    /// API User 标识 (New-Api-User 请求头值，通常为 5 位数字)
    #[serde(default)]
    pub api_user: String,
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
    /// 扩展配置 (JSON 格式: CDK 凭证, OAuth tokens 等)
    #[serde(default = "default_extra_config")]
    pub extra_config: String,
}

fn default_enabled() -> bool {
    true
}

fn default_extra_config() -> String {
    "{}".to_string()
}

impl CheckinAccount {
    /// 创建新账号
    pub fn new(
        provider_id: String,
        name: String,
        cookies_json_encrypted: String,
        api_user: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            provider_id,
            name,
            cookies_json_encrypted,
            api_user,
            enabled: true,
            created_at: Utc::now(),
            updated_at: None,
            last_checkin_at: None,
            last_balance_check_at: None,
            extra_config: "{}".to_string(),
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

/// Cookie 凭证结构 (用于构造 HTTP Cookie 头)
#[derive(Debug, Clone)]
pub struct CookieCredentials {
    /// Cookie 键值对
    pub cookies: HashMap<String, String>,
    /// API User 标识
    pub api_user: String,
}

impl CookieCredentials {
    /// 从 JSON 字符串和 api_user 创建凭证
    pub fn from_json(json_str: &str, api_user: String) -> Result<Self, serde_json::Error> {
        let cookies: HashMap<String, String> = serde_json::from_str(json_str)?;
        Ok(Self { cookies, api_user })
    }

    /// 构造 Cookie 头字符串 (如 "session=abc123; token=xyz789")
    #[allow(dead_code)]
    pub fn cookie_string(&self) -> String {
        self.cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// 检查 api_user 是否非空
    pub fn has_api_user(&self) -> bool {
        !self.api_user.is_empty()
    }
}

/// 账号显示信息 (用于 API 响应，隐藏加密凭证)
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
    /// 遮罩后的 Cookies (如 "session=ab****89; token=****")
    pub cookies_masked: String,
    /// API User 标识
    pub api_user: String,
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
    /// 总额度 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_quota: Option<f64>,
    /// 历史消耗 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_consumed: Option<f64>,
    /// 扩展配置 (JSON 格式: CDK 凭证, OAuth tokens 等)
    #[serde(default = "default_extra_config")]
    pub extra_config: String,
}

/// 创建账号请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    /// 关联的提供商 ID
    pub provider_id: String,
    /// 账号备注名称
    pub name: String,
    /// Cookies JSON (明文，将被加密存储)
    /// 格式: {"session": "abc123", "token": "xyz789"}
    pub cookies_json: String,
    /// API User 标识 (可选，New-Api-User 请求头值)
    #[serde(default)]
    pub api_user: String,
    /// 扩展配置 JSON (可选，CDK 凭证等)
    #[serde(default = "default_extra_config")]
    pub extra_config: String,
}

/// 更新账号请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    /// 账号备注名称 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Cookies JSON (可选，明文，将被加密存储)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies_json: Option<String>,
    /// API User 标识 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_user: Option<String>,
    /// 是否启用 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// 扩展配置 JSON (可选，CDK 凭证等)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_config: Option<String>,
}

/// 账号列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsResponse {
    /// 账号列表 (已遮罩 Cookies)
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

/// 遮罩 Cookies JSON 字符串
/// 将 {"session": "abc123xyz", "token": "secret"} 转换为 "session=ab****yz; token=****"
pub fn mask_cookies_json(cookies_json: &str) -> String {
    match serde_json::from_str::<HashMap<String, String>>(cookies_json) {
        Ok(cookies) => cookies
            .iter()
            .map(|(k, v)| {
                let masked_value = mask_value(v);
                format!("{}={}", k, masked_value)
            })
            .collect::<Vec<_>>()
            .join("; "),
        Err(_) => "****".to_string(),
    }
}

/// 遮罩单个值
/// 保留前 2 位和后 2 位，中间用 **** 替代
fn mask_value(value: &str) -> String {
    let len = value.len();
    if len <= 4 {
        "****".to_string()
    } else if len <= 8 {
        format!("{}****", &value[..2])
    } else {
        format!("{}****{}", &value[..2], &value[len - 2..])
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_create_account() {
        let account = CheckinAccount::new(
            "provider-1".to_string(),
            "Main Account".to_string(),
            "encrypted-cookies".to_string(),
            "12345".to_string(),
        );

        assert!(!account.id.is_empty());
        assert_eq!(account.provider_id, "provider-1");
        assert_eq!(account.name, "Main Account");
        assert_eq!(account.api_user, "12345");
        assert!(account.enabled);
        assert!(account.last_checkin_at.is_none());
    }

    #[test]
    fn test_update_checkin_time() {
        let mut account = CheckinAccount::new(
            "provider-1".to_string(),
            "Test".to_string(),
            "cookies".to_string(),
            String::new(),
        );

        assert!(account.last_checkin_at.is_none());
        account.update_checkin_time();
        assert!(account.last_checkin_at.is_some());
        assert!(account.updated_at.is_some());
    }

    #[test]
    fn test_cookie_credentials_from_json() {
        let json = r#"{"session": "abc123", "token": "xyz789"}"#;
        let creds = CookieCredentials::from_json(json, "12345".to_string()).unwrap();

        assert_eq!(creds.cookies.len(), 2);
        assert_eq!(creds.cookies.get("session").unwrap(), "abc123");
        assert_eq!(creds.cookies.get("token").unwrap(), "xyz789");
        assert_eq!(creds.api_user, "12345");
        assert!(creds.has_api_user());
    }

    #[test]
    fn test_cookie_credentials_cookie_string() {
        let json = r#"{"session": "abc123", "token": "xyz789"}"#;
        let creds = CookieCredentials::from_json(json, String::new()).unwrap();

        let cookie_string = creds.cookie_string();
        // HashMap 迭代顺序不确定，所以检查包含关系
        assert!(cookie_string.contains("session=abc123"));
        assert!(cookie_string.contains("token=xyz789"));
        assert!(cookie_string.contains("; "));
        assert!(!creds.has_api_user());
    }

    #[test]
    fn test_mask_cookies_json() {
        let json = r#"{"session": "abc123xyz", "token": "secret"}"#;
        let masked = mask_cookies_json(json);

        // 检查遮罩格式
        assert!(masked.contains("session=ab****yz"));
        assert!(masked.contains("token=se****"));
    }

    #[test]
    fn test_mask_value() {
        // 短值 (<=4)
        assert_eq!(mask_value("abc"), "****");
        assert_eq!(mask_value("abcd"), "****");

        // 中等长度 (5-8)
        assert_eq!(mask_value("abcde"), "ab****");
        assert_eq!(mask_value("abcdefgh"), "ab****");

        // 长值 (>8)
        assert_eq!(mask_value("abc123xyz"), "ab****yz");
        assert_eq!(mask_value("verylongvalue"), "ve****ue");
    }

    #[test]
    fn test_mask_cookies_json_invalid() {
        let invalid_json = "not valid json";
        let masked = mask_cookies_json(invalid_json);
        assert_eq!(masked, "****");
    }
}
