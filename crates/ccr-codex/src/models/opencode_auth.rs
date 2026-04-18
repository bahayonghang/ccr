// 🔐 OpenCode Auth 数据模型
// 用于管理 OpenCode 的多账号登录状态（仅 openai provider）

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// OpenCode auth.json 中 openai provider 的已知字段
///
/// 实际文件可能包含更多未知字段，因此通过 `extra` 保留原始 payload。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenCodeOpenAiAuth {
    /// 认证类型 ("oauth")
    #[serde(default)]
    pub r#type: String,

    /// Access Token (JWT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<String>,

    /// Refresh Token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<String>,

    /// 到期时间 (Unix 毫秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<i64>,

    /// 账号 ID
    #[serde(rename = "accountId", skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,

    /// 兼容未知字段，避免写回时丢失信息
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

/// CCR 管理的 OpenCode 账号注册表
///
/// 存储在 `~/.ccr/platforms/opencode/auth_registry.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeAuthRegistry {
    /// 版本号
    #[serde(default = "default_registry_version")]
    pub version: String,

    /// 当前激活的账号名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_auth: Option<String>,

    /// 所有保存的账号
    #[serde(default)]
    pub accounts: IndexMap<String, OpenCodeAuthAccount>,
}

fn default_registry_version() -> String {
    "1.0".to_string()
}

impl Default for OpenCodeAuthRegistry {
    fn default() -> Self {
        Self {
            version: default_registry_version(),
            current_auth: None,
            accounts: IndexMap::new(),
        }
    }
}

/// OpenCode 账号元数据
///
/// 敏感 token 不存入 TOML，真正的 openai provider 快照单独写入 `auth/{name}.json`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeAuthAccount {
    /// OpenAI 账号 ID（从 auth.json 或 access token 提取）
    pub account_id: String,

    /// 邮箱（脱敏后存储）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// 订阅类型 (FREE/PLUS/PRO/TEAM)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,

    /// 保存时间
    pub saved_at: DateTime<Utc>,

    /// 最后使用时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,

    /// 到期时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// OpenCode 账号列表项（用于 TUI 展示）
#[derive(Debug, Clone)]
pub struct OpenCodeAuthItem {
    /// 账号名称
    pub name: String,

    /// OpenAI 账号 ID
    pub account_id: Option<String>,

    /// 脱敏后的邮箱
    pub email: Option<String>,

    /// 订阅类型
    pub plan_type: Option<String>,

    /// 是否为当前激活账号
    pub is_current: bool,

    /// 是否为虚拟项（未保存的当前登录）
    pub is_virtual: bool,

    /// 保存时间
    pub saved_at: Option<DateTime<Utc>>,

    /// 最后使用时间
    pub last_used: Option<DateTime<Utc>>,

    /// 到期时间
    pub expires_at: Option<DateTime<Utc>>,
}

/// OpenCode 登录状态（简化版）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenCodeLoginState {
    /// 未登录（auth.json 不存在或无 openai 字段）
    NotLoggedIn,
    /// 已登录但未保存到 CCR 注册表
    LoggedInUnsaved,
    /// 已登录且已保存（账号名称）
    LoggedInSaved(String),
}

/// 当前 OpenCode openai 认证信息
#[derive(Debug, Clone)]
pub struct OpenCodeCurrentAuthInfo {
    /// 账号 ID
    pub account_id: String,
    /// 邮箱（原始）
    pub email: Option<String>,
    /// 订阅类型
    pub plan_type: Option<String>,
    /// 到期时间
    pub expires_at: Option<DateTime<Utc>>,
}

/// 当前 OpenCode 读取快照
#[derive(Debug, Clone)]
pub struct OpenCodeReadSnapshot {
    pub login_state: OpenCodeLoginState,
    pub current_info: Option<OpenCodeCurrentAuthInfo>,
    pub registry: OpenCodeAuthRegistry,
    pub current_account_name: Option<String>,
}

/// Codex -> OpenCode 迁移单账号结果类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexToOpenCodeMigrationStatus {
    Imported,
    SkippedExistingName,
    SkippedExistingAccountId,
    SkippedIncompatibleAuth,
    SkippedMissingSnapshot,
    SkippedInvalidSnapshot,
}

/// Codex -> OpenCode 单账号迁移结果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexToOpenCodeMigrationItem {
    /// 源 Codex 账号名称
    pub name: String,

    /// 迁移结果
    pub status: CodexToOpenCodeMigrationStatus,

    /// 账号 ID（当可提取时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,

    /// 人类可读原因
    pub message: String,
}

/// Codex -> OpenCode 迁移汇总报告
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CodexToOpenCodeMigrationReport {
    /// 是否为 dry-run
    #[serde(default)]
    pub dry_run: bool,

    /// 可导入 / 已导入数量
    #[serde(default)]
    pub imported: usize,

    /// 因目标同名而跳过
    #[serde(default)]
    pub skipped_existing_name: usize,

    /// 因目标存在相同 account_id 而跳过
    #[serde(default)]
    pub skipped_existing_account_id: usize,

    /// 因源认证类型不兼容而跳过
    #[serde(default)]
    pub skipped_incompatible_auth: usize,

    /// 因缺失源快照而跳过
    #[serde(default)]
    pub skipped_missing_snapshot: usize,

    /// 因源快照无效而跳过
    #[serde(default)]
    pub skipped_invalid_snapshot: usize,

    /// 逐账号结果
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcomes: Vec<CodexToOpenCodeMigrationItem>,
}

impl CodexToOpenCodeMigrationReport {
    pub fn skipped_total(&self) -> usize {
        self.skipped_existing_name
            + self.skipped_existing_account_id
            + self.skipped_incompatible_auth
            + self.skipped_missing_snapshot
            + self.skipped_invalid_snapshot
    }

    pub fn total(&self) -> usize {
        self.imported + self.skipped_total()
    }

    pub fn has_importable_accounts(&self) -> bool {
        self.imported > 0
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn opencode_openai_auth_deserializes_from_actual_format() {
        let json = r#"{
            "type": "oauth",
            "access": "eyJ...",
            "refresh": "rt_...",
            "expires": 1776919788911,
            "accountId": "f57414ba-b783-4423-81d6-da45aa18787d"
        }"#;

        let auth: OpenCodeOpenAiAuth = serde_json::from_str(json).unwrap();
        assert_eq!(auth.r#type, "oauth");
        assert_eq!(
            auth.account_id,
            Some("f57414ba-b783-4423-81d6-da45aa18787d".to_string())
        );
        assert_eq!(auth.expires, Some(1776919788911));
    }

    #[test]
    fn opencode_openai_auth_preserves_unknown_fields() {
        let json = r#"{
            "type": "oauth",
            "access": "eyJ...",
            "refresh": "rt_...",
            "expires": 1776919788911,
            "accountId": "f57414ba-b783-4423-81d6-da45aa18787d",
            "plan": "plus",
            "customMeta": { "region": "us" }
        }"#;

        let auth: OpenCodeOpenAiAuth = serde_json::from_str(json).unwrap();
        assert_eq!(auth.extra.get("plan").and_then(Value::as_str), Some("plus"));
        assert_eq!(
            auth.extra
                .get("customMeta")
                .and_then(|value| value.get("region"))
                .and_then(Value::as_str),
            Some("us")
        );
    }

    #[test]
    fn opencode_openai_auth_serializes_with_camel_case_account_id() {
        let auth = OpenCodeOpenAiAuth {
            r#type: "oauth".to_string(),
            access: Some("token".to_string()),
            refresh: Some("rt".to_string()),
            expires: Some(1000),
            account_id: Some("abc-123".to_string()),
            extra: IndexMap::new(),
        };

        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("\"accountId\""));
        assert!(!json.contains("\"account_id\""));
    }

    #[test]
    fn opencode_auth_registry_default() {
        let registry = OpenCodeAuthRegistry::default();
        assert_eq!(registry.version, "1.0");
        assert!(registry.current_auth.is_none());
        assert!(registry.accounts.is_empty());
    }

    #[test]
    fn opencode_auth_registry_serialization_roundtrip() {
        let mut registry = OpenCodeAuthRegistry {
            current_auth: Some("main".to_string()),
            ..Default::default()
        };
        registry.accounts.insert(
            "main".to_string(),
            OpenCodeAuthAccount {
                account_id: "acc-main".to_string(),
                email: Some("b***@gmail.com".to_string()),
                plan_type: Some("plus".to_string()),
                saved_at: Utc::now(),
                last_used: None,
                expires_at: None,
            },
        );

        let toml_str = toml::to_string(&registry).unwrap();
        assert!(toml_str.contains("current_auth"));
        assert!(toml_str.contains("[accounts.main]"));

        let parsed: OpenCodeAuthRegistry = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.current_auth, Some("main".to_string()));
        assert!(parsed.accounts.contains_key("main"));
        assert_eq!(parsed.accounts["main"].plan_type, Some("plus".to_string()));
    }

    #[test]
    fn opencode_auth_registry_backward_compat_without_optional_fields() {
        let toml = r#"
version = "1.0"
current_auth = "legacy"

[accounts.legacy]
account_id = "acc-legacy"
saved_at = "2026-01-01T00:00:00Z"
"#;

        let parsed: OpenCodeAuthRegistry = toml::from_str(toml).unwrap();
        assert_eq!(parsed.accounts["legacy"].email, None);
        assert_eq!(parsed.accounts["legacy"].plan_type, None);
        assert_eq!(parsed.accounts["legacy"].expires_at, None);
    }

    #[test]
    fn opencode_login_state_equality() {
        assert_eq!(
            OpenCodeLoginState::NotLoggedIn,
            OpenCodeLoginState::NotLoggedIn
        );
        assert_eq!(
            OpenCodeLoginState::LoggedInSaved("a".to_string()),
            OpenCodeLoginState::LoggedInSaved("a".to_string())
        );
        assert_ne!(
            OpenCodeLoginState::LoggedInUnsaved,
            OpenCodeLoginState::NotLoggedIn
        );
    }

    #[test]
    fn codex_to_opencode_migration_report_totals() {
        let report = CodexToOpenCodeMigrationReport {
            dry_run: true,
            imported: 2,
            skipped_existing_name: 1,
            skipped_existing_account_id: 1,
            skipped_incompatible_auth: 1,
            skipped_missing_snapshot: 1,
            skipped_invalid_snapshot: 1,
            outcomes: vec![],
        };

        assert_eq!(report.skipped_total(), 5);
        assert_eq!(report.total(), 7);
        assert!(report.has_importable_accounts());
    }
}
