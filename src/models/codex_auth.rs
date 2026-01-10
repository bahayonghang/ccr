// 🔐 Codex Auth 数据模型
// 用于管理 Codex CLI 的多账号登录状态
//
// 核心职责:
// - 📋 定义账号元数据结构
// - 📦 定义注册表结构
// - 🎨 定义 TUI 状态枚举

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Token 新鲜度状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenFreshness {
    /// 新鲜 (< 1 天)
    Fresh,
    /// 陈旧 (1-7 天)
    Stale,
    /// 过期 (> 7 天)
    Old,
    /// 未知 (无法解析时间)
    Unknown,
}

impl TokenFreshness {
    /// 获取显示图标
    #[allow(dead_code)]
    pub fn icon(&self) -> &'static str {
        match self {
            TokenFreshness::Fresh => "✓",
            TokenFreshness::Stale => "⚠",
            TokenFreshness::Old => "✗",
            TokenFreshness::Unknown => "?",
        }
    }

    /// 获取描述文本
    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            TokenFreshness::Fresh => "Token 状态良好",
            TokenFreshness::Stale => "Token 可能需要刷新",
            TokenFreshness::Old => "Token 可能已过期，建议重新登录",
            TokenFreshness::Unknown => "无法确定 Token 状态",
        }
    }
}

/// Codex 账号元数据
///
/// 存储在 auth_registry.toml 中的账号信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthAccount {
    /// 账号描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 账号 ID (从 auth.json 提取)
    pub account_id: String,

    /// 邮箱 (脱敏后存储)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// 保存时间
    pub saved_at: DateTime<Utc>,

    /// 最后使用时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,

    /// 最后刷新时间 (从 auth.json 提取)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_refresh: Option<DateTime<Utc>>,
}

/// Codex 账号注册表
///
/// 存储在 ~/.ccr/platforms/codex/auth_registry.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthRegistry {
    /// 版本号
    #[serde(default = "default_version")]
    pub version: String,

    /// 当前激活的账号名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_auth: Option<String>,

    /// 所有账号
    #[serde(default)]
    pub accounts: IndexMap<String, CodexAuthAccount>,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl Default for CodexAuthRegistry {
    fn default() -> Self {
        Self {
            version: default_version(),
            current_auth: None,
            accounts: IndexMap::new(),
        }
    }
}

/// 账号列表项 (用于显示)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CodexAuthItem {
    /// 账号名称
    pub name: String,

    /// 账号描述
    pub description: Option<String>,

    /// 脱敏后的邮箱
    pub email: Option<String>,

    /// 是否为当前激活账号
    pub is_current: bool,

    /// 是否为虚拟项 (未保存的 default)
    pub is_virtual: bool,

    /// 最后使用时间
    pub last_used: Option<DateTime<Utc>>,

    /// 最后刷新时间
    pub last_refresh: Option<DateTime<Utc>>,

    /// Token 新鲜度
    pub freshness: TokenFreshness,
}

/// 当前 auth.json 解析信息
#[derive(Debug, Clone)]
pub struct CurrentAuthInfo {
    /// 账号 ID
    pub account_id: String,

    /// 邮箱 (原始)
    pub email: Option<String>,

    /// 最后刷新时间
    pub last_refresh: Option<DateTime<Utc>>,

    /// Token 新鲜度
    pub freshness: TokenFreshness,
}

/// TUI 登录状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginState {
    /// 未登录 (auth.json 不存在)
    NotLoggedIn,

    /// 已登录但未保存
    LoggedInUnsaved,

    /// 已登录且已保存 (账号名)
    LoggedInSaved(String),
}

/// TUI 输入模式
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum InputMode {
    /// 正常模式
    #[default]
    Normal,

    /// 保存账号输入模式
    SaveInput,

    /// 删除确认模式
    DeleteConfirm,
}

/// Codex auth.json 文件结构
///
/// 用于解析 ~/.codex/auth.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthJson {
    /// OpenAI API Key (可选)
    #[serde(rename = "OPENAI_API_KEY")]
    pub openai_api_key: Option<String>,

    /// OAuth tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<CodexAuthTokens>,

    /// 最后刷新时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_refresh: Option<String>,
}

/// Codex OAuth tokens 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthTokens {
    /// ID Token (JWT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,

    /// Access Token (JWT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,

    /// Refresh Token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// 账号 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_token_freshness_icon() {
        assert_eq!(TokenFreshness::Fresh.icon(), "✓");
        assert_eq!(TokenFreshness::Stale.icon(), "⚠");
        assert_eq!(TokenFreshness::Old.icon(), "✗");
        assert_eq!(TokenFreshness::Unknown.icon(), "?");
    }

    #[test]
    fn test_registry_default() {
        let registry = CodexAuthRegistry::default();
        assert_eq!(registry.version, "1.0");
        assert!(registry.current_auth.is_none());
        assert!(registry.accounts.is_empty());
    }

    #[test]
    fn test_input_mode_default() {
        let mode = InputMode::default();
        assert_eq!(mode, InputMode::Normal);
    }

    #[test]
    fn test_codex_auth_json_deserialize() {
        let json = r#"{
            "OPENAI_API_KEY": null,
            "tokens": {
                "id_token": "eyJ...",
                "access_token": "eyJ...",
                "refresh_token": "rt_...",
                "account_id": "test-id"
            },
            "last_refresh": "2026-01-08T03:09:53.894843900Z"
        }"#;

        let auth: CodexAuthJson = serde_json::from_str(json).unwrap();
        assert!(auth.openai_api_key.is_none());
        assert!(auth.tokens.is_some());
        let tokens = auth.tokens.unwrap();
        assert_eq!(tokens.account_id, Some("test-id".to_string()));
    }
}
