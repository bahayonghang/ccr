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

// Re-export shared types from ccr-types
pub use ccr_types::{LoginState, TokenFreshness};

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

    /// 到期时间 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
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

    /// 添加时间
    pub saved_at: Option<DateTime<Utc>>,

    /// 最后使用时间
    #[allow(dead_code)]
    pub last_used: Option<DateTime<Utc>>,

    /// 最后刷新时间
    pub last_refresh: Option<DateTime<Utc>>,

    /// Token 新鲜度
    pub freshness: TokenFreshness,

    /// 到期时间
    pub expires_at: Option<DateTime<Utc>>,
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

// ==================== 导入/导出数据结构 ====================

/// Codex Auth 导出格式
///
/// 用于导入/导出账号数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthExport {
    /// 导出格式版本
    #[serde(default = "default_export_version")]
    pub version: String,

    /// 导出时间
    pub exported_at: DateTime<Utc>,

    /// 账号数据
    pub accounts: IndexMap<String, CodexAuthExportAccount>,
}

fn default_export_version() -> String {
    "1.0".to_string()
}

/// 导出的账号数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthExportAccount {
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

    /// 到期时间 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

    /// 完整的 auth.json 数据（可选，根据 include_secrets 决定）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_data: Option<CodexAuthJson>,
}

/// 导入模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// 合并模式：保留现有账号，只添加新的
    Merge,
    /// 替换模式：覆盖同名账号
    Replace,
}

/// 导入结果
#[derive(Debug, Clone, Default)]
pub struct ImportResult {
    /// 新增账号数
    pub added: usize,
    /// 更新账号数
    pub updated: usize,
    /// 跳过账号数
    pub skipped: usize,
    /// 被覆盖的账号列表
    pub overwritten: Vec<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_token_freshness_icon() {
        // Icons are now emoji from ccr-types
        assert_eq!(TokenFreshness::Fresh.icon(), "🟢");
        assert_eq!(TokenFreshness::Stale.icon(), "🟡");
        assert_eq!(TokenFreshness::Old.icon(), "🔴");
        assert_eq!(TokenFreshness::unknown().icon(), "⚪");
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

    #[test]
    fn test_codex_auth_account_with_expiry() {
        let account = CodexAuthAccount {
            description: Some("Test account".to_string()),
            account_id: "acc-123".to_string(),
            email: Some("t***@example.com".to_string()),
            saved_at: Utc::now(),
            last_used: None,
            last_refresh: Some(Utc::now()),
            expires_at: Some(Utc::now() + chrono::Duration::days(30)),
        };

        // Test serialization
        let toml_str = toml::to_string(&account).unwrap();
        assert!(toml_str.contains("expires_at"));
        assert!(toml_str.contains("account_id"));

        // Test deserialization
        let parsed: CodexAuthAccount = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.account_id, "acc-123");
        assert!(parsed.expires_at.is_some());
    }

    #[test]
    fn test_codex_auth_account_without_expiry() {
        let account = CodexAuthAccount {
            description: None,
            account_id: "acc-456".to_string(),
            email: None,
            saved_at: Utc::now(),
            last_used: None,
            last_refresh: None,
            expires_at: None,
        };

        // Test serialization - expires_at should be omitted
        let toml_str = toml::to_string(&account).unwrap();
        assert!(!toml_str.contains("expires_at"));

        // Test deserialization
        let parsed: CodexAuthAccount = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.account_id, "acc-456");
        assert!(parsed.expires_at.is_none());
    }

    #[test]
    fn test_codex_auth_registry_serialization() {
        let mut registry = CodexAuthRegistry {
            current_auth: Some("main".to_string()),
            ..Default::default()
        };
        registry.accounts.insert(
            "main".to_string(),
            CodexAuthAccount {
                description: Some("Main account".to_string()),
                account_id: "acc-main".to_string(),
                email: Some("m***@example.com".to_string()),
                saved_at: Utc::now(),
                last_used: Some(Utc::now()),
                last_refresh: Some(Utc::now()),
                expires_at: Some(Utc::now() + chrono::Duration::days(7)),
            },
        );

        // Test serialization
        let toml_str = toml::to_string(&registry).unwrap();
        assert!(toml_str.contains("version"));
        assert!(toml_str.contains("current_auth"));
        assert!(toml_str.contains("[accounts.main]"));
        assert!(toml_str.contains("expires_at"));

        // Test deserialization
        let parsed: CodexAuthRegistry = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.version, "1.0");
        assert_eq!(parsed.current_auth, Some("main".to_string()));
        assert!(parsed.accounts.contains_key("main"));
        assert!(parsed.accounts["main"].expires_at.is_some());
    }

    #[test]
    fn test_codex_auth_registry_backward_compatibility() {
        // Old format without expires_at
        let old_toml = r#"
version = "1.0"
current_auth = "legacy"

[accounts.legacy]
account_id = "acc-legacy"
saved_at = "2026-01-01T00:00:00Z"
"#;

        let parsed: CodexAuthRegistry = toml::from_str(old_toml).unwrap();
        assert_eq!(parsed.version, "1.0");
        assert_eq!(parsed.current_auth, Some("legacy".to_string()));
        assert!(parsed.accounts.contains_key("legacy"));
        assert!(parsed.accounts["legacy"].expires_at.is_none());
    }
}
