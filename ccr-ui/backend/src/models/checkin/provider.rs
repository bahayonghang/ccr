// 中转站提供商配置模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 中转站提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinProvider {
    /// 唯一标识符 (UUID)
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 基础 URL (如 https://api.example.com)
    pub base_url: String,
    /// 签到 API 路径 (默认 /api/user/checkin)
    #[serde(default = "default_checkin_path")]
    pub checkin_path: String,
    /// 余额查询 API 路径 (默认 /api/user/self)
    #[serde(default = "default_balance_path")]
    pub balance_path: String,
    /// 用户信息 API 路径 (默认 /api/user/self)
    #[serde(default = "default_user_info_path")]
    pub user_info_path: String,
    /// 认证头名称 (默认 Authorization)
    #[serde(default = "default_auth_header")]
    pub auth_header: String,
    /// 认证前缀 (默认 Bearer)
    #[serde(default = "default_auth_prefix")]
    pub auth_prefix: String,
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

fn default_checkin_path() -> String {
    "/api/user/checkin".to_string()
}

fn default_balance_path() -> String {
    "/api/user/self".to_string()
}

fn default_user_info_path() -> String {
    "/api/user/self".to_string()
}

fn default_auth_header() -> String {
    "Authorization".to_string()
}

fn default_auth_prefix() -> String {
    "Bearer".to_string()
}

fn default_enabled() -> bool {
    true
}

impl CheckinProvider {
    /// 创建新的提供商配置
    pub fn new(name: String, base_url: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            base_url,
            checkin_path: default_checkin_path(),
            balance_path: default_balance_path(),
            user_info_path: default_user_info_path(),
            auth_header: default_auth_header(),
            auth_prefix: default_auth_prefix(),
            enabled: true,
            created_at: Utc::now(),
            updated_at: None,
        }
    }

    /// 获取完整的签到 URL
    #[allow(dead_code)]
    pub fn checkin_url(&self) -> String {
        format!(
            "{}{}",
            self.base_url.trim_end_matches('/'),
            self.checkin_path
        )
    }

    /// 获取完整的余额查询 URL
    #[allow(dead_code)]
    pub fn balance_url(&self) -> String {
        format!(
            "{}{}",
            self.base_url.trim_end_matches('/'),
            self.balance_path
        )
    }

    /// 获取完整的用户信息 URL
    #[allow(dead_code)]
    pub fn user_info_url(&self) -> String {
        format!(
            "{}{}",
            self.base_url.trim_end_matches('/'),
            self.user_info_path
        )
    }

    /// 生成认证头值
    #[allow(dead_code)]
    pub fn auth_value(&self, api_key: &str) -> String {
        if self.auth_prefix.is_empty() {
            api_key.to_string()
        } else {
            format!("{} {}", self.auth_prefix, api_key)
        }
    }
}

/// 创建提供商请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProviderRequest {
    /// 显示名称
    pub name: String,
    /// 基础 URL
    pub base_url: String,
    /// 签到 API 路径 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkin_path: Option<String>,
    /// 余额查询 API 路径 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_path: Option<String>,
    /// 用户信息 API 路径 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_info_path: Option<String>,
    /// 认证头名称 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    /// 认证前缀 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_prefix: Option<String>,
}

impl CreateProviderRequest {
    /// 转换为 CheckinProvider
    pub fn into_provider(self) -> CheckinProvider {
        let mut provider = CheckinProvider::new(self.name, self.base_url);

        if let Some(path) = self.checkin_path {
            provider.checkin_path = path;
        }
        if let Some(path) = self.balance_path {
            provider.balance_path = path;
        }
        if let Some(path) = self.user_info_path {
            provider.user_info_path = path;
        }
        if let Some(header) = self.auth_header {
            provider.auth_header = header;
        }
        if let Some(prefix) = self.auth_prefix {
            provider.auth_prefix = prefix;
        }

        provider
    }
}

/// 更新提供商请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProviderRequest {
    /// 显示名称 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 基础 URL (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// 签到 API 路径 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkin_path: Option<String>,
    /// 余额查询 API 路径 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_path: Option<String>,
    /// 用户信息 API 路径 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_info_path: Option<String>,
    /// 认证头名称 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    /// 认证前缀 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_prefix: Option<String>,
    /// 是否启用 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// 提供商列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersResponse {
    /// 提供商列表
    pub providers: Vec<CheckinProvider>,
    /// 总数
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_urls() {
        let provider =
            CheckinProvider::new("Test".to_string(), "https://api.example.com".to_string());

        assert_eq!(
            provider.checkin_url(),
            "https://api.example.com/api/user/checkin"
        );
        assert_eq!(
            provider.balance_url(),
            "https://api.example.com/api/user/self"
        );
        assert_eq!(
            provider.user_info_url(),
            "https://api.example.com/api/user/self"
        );
    }

    #[test]
    fn test_provider_urls_with_trailing_slash() {
        let provider =
            CheckinProvider::new("Test".to_string(), "https://api.example.com/".to_string());

        assert_eq!(
            provider.checkin_url(),
            "https://api.example.com/api/user/checkin"
        );
    }

    #[test]
    fn test_auth_value() {
        let provider =
            CheckinProvider::new("Test".to_string(), "https://api.example.com".to_string());

        assert_eq!(provider.auth_value("sk-123"), "Bearer sk-123");
    }
}
