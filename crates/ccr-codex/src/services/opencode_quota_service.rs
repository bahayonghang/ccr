// 💰 OpenCode 配额查询服务
// 基于 OpenCode 当前 openai provider 或已保存账号快照查询 wham/usage。

use super::openai_quota_core::{
    OpenAiQuotaCore, OpenAiQuotaFetchOutcome, OpenAiQuotaSnapshot, TokenRefreshResponse,
};
use crate::models::{CodexAccountQuota, OpenCodeOpenAiAuth};
use crate::utils::{OpenCodePaths, ensure_private_permissions};
use ccr_core::core::atomic_writer::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use chrono::Utc;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};

/// OpenCode quota 查询服务。
pub struct OpenCodeQuotaService {
    /// CCR OpenCode 平台目录 (~/.ccr/platforms/opencode/)
    ccr_opencode_dir: PathBuf,
    /// OpenCode 数据目录（官方默认 ~/.local/share/opencode/）
    opencode_dir: PathBuf,
}

impl OpenCodeQuotaService {
    pub fn new() -> Result<Self> {
        let paths = OpenCodePaths::resolve()?;
        Ok(Self {
            ccr_opencode_dir: paths.ccr_opencode_dir,
            opencode_dir: paths.opencode_dir,
        })
    }

    pub fn from_dirs(ccr_opencode_dir: PathBuf, opencode_dir: PathBuf) -> Self {
        Self {
            ccr_opencode_dir,
            opencode_dir,
        }
    }

    pub async fn fetch_account_quota(&self, account_name: &str) -> CodexAccountQuota {
        self.fetch_account_quota_inner(account_name, false).await
    }

    pub async fn fetch_account_quota_force_refresh(&self, account_name: &str) -> CodexAccountQuota {
        self.fetch_account_quota_inner(account_name, true).await
    }

    pub async fn fetch_current_quota(&self) -> CodexAccountQuota {
        self.fetch_current_quota_inner(false).await
    }

    pub async fn fetch_current_quota_force_refresh(&self) -> CodexAccountQuota {
        self.fetch_current_quota_inner(true).await
    }

    pub fn format_reset_duration(reset_timestamp: i64) -> String {
        OpenAiQuotaCore::format_reset_duration(reset_timestamp)
    }

    fn auth_json_path(&self) -> PathBuf {
        self.opencode_dir.join("auth.json")
    }

    fn account_auth_path(&self, name: &str) -> PathBuf {
        self.ccr_opencode_dir
            .join("auth")
            .join(format!("{name}.json"))
    }

    async fn fetch_account_quota_inner(
        &self,
        account_name: &str,
        force_refresh: bool,
    ) -> CodexAccountQuota {
        let now = Utc::now();
        let auth_path = self.account_auth_path(account_name);
        let snapshot = match Self::load_snapshot_from_saved_file(&auth_path) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                return CodexAccountQuota {
                    account_name: account_name.to_string(),
                    email: None,
                    quota: None,
                    error: Some(error.to_string()),
                    fetched_at: now,
                };
            }
        };
        let fallback_email = snapshot.email.clone();

        match Self::fetch_snapshot_quota(snapshot, force_refresh, |tokens| {
            Self::persist_saved_tokens(&auth_path, tokens)
        })
        .await
        {
            Ok(outcome) => Self::build_success(account_name, outcome, now),
            Err(error) => CodexAccountQuota {
                account_name: account_name.to_string(),
                email: fallback_email,
                quota: None,
                error: Some(error),
                fetched_at: now,
            },
        }
    }

    async fn fetch_current_quota_inner(&self, force_refresh: bool) -> CodexAccountQuota {
        let now = Utc::now();
        let auth_path = self.auth_json_path();
        let snapshot = match Self::load_snapshot_from_current_auth(&auth_path) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                return CodexAccountQuota {
                    account_name: "current-login".to_string(),
                    email: None,
                    quota: None,
                    error: Some(error.to_string()),
                    fetched_at: now,
                };
            }
        };
        let fallback_email = snapshot.email.clone();

        match Self::fetch_snapshot_quota(snapshot, force_refresh, |tokens| {
            Self::persist_current_tokens(&auth_path, tokens)
        })
        .await
        {
            Ok(outcome) => Self::build_success("current-login", outcome, now),
            Err(error) => CodexAccountQuota {
                account_name: "current-login".to_string(),
                email: fallback_email,
                quota: None,
                error: Some(error),
                fetched_at: now,
            },
        }
    }

    async fn fetch_snapshot_quota<F>(
        snapshot: OpenAiQuotaSnapshot,
        force_refresh: bool,
        persist_tokens: F,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, String>
    where
        F: FnMut(&TokenRefreshResponse) -> std::result::Result<(), String>,
    {
        OpenAiQuotaCore::fetch_quota(snapshot, force_refresh, persist_tokens).await
    }

    fn build_success(
        account_name: &str,
        outcome: OpenAiQuotaFetchOutcome,
        fetched_at: chrono::DateTime<Utc>,
    ) -> CodexAccountQuota {
        CodexAccountQuota {
            account_name: account_name.to_string(),
            email: outcome.email,
            quota: Some(outcome.quota),
            error: None,
            fetched_at,
        }
    }

    fn load_snapshot_from_saved_file(path: &Path) -> Result<OpenAiQuotaSnapshot> {
        let content = fs::read_to_string(path).map_err(|error| {
            CcrError::ConfigError(format!("读取 OpenCode 账号快照失败: {error}"))
        })?;
        let auth: OpenCodeOpenAiAuth = serde_json::from_str(&content).map_err(|error| {
            CcrError::ConfigError(format!("解析 OpenCode 账号快照失败: {error}"))
        })?;
        Self::build_snapshot(auth)
    }

    fn load_snapshot_from_current_auth(path: &Path) -> Result<OpenAiQuotaSnapshot> {
        let content = fs::read_to_string(path).map_err(|error| {
            CcrError::ConfigError(format!("读取 OpenCode auth.json 失败: {error}"))
        })?;
        let root: JsonMap<String, JsonValue> = serde_json::from_str(&content).map_err(|error| {
            CcrError::ConfigError(format!("解析 OpenCode auth.json 失败: {error}"))
        })?;
        let openai = root
            .get("openai")
            .cloned()
            .filter(|value| !value.is_null())
            .ok_or_else(|| CcrError::ConfigError("当前 OpenCode 未检测到 openai 登录".into()))?;
        let auth: OpenCodeOpenAiAuth = serde_json::from_value(openai).map_err(|error| {
            CcrError::ConfigError(format!("解析 OpenCode openai provider 失败: {error}"))
        })?;
        Self::build_snapshot(auth)
    }

    fn build_snapshot(auth: OpenCodeOpenAiAuth) -> Result<OpenAiQuotaSnapshot> {
        let email = auth
            .extra
            .get("email")
            .and_then(JsonValue::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        Ok(OpenAiQuotaSnapshot {
            access_token: auth.access.unwrap_or_default(),
            refresh_token: auth.refresh,
            account_id: auth.account_id,
            email,
        })
    }

    fn persist_saved_tokens(
        path: &Path,
        new_tokens: &TokenRefreshResponse,
    ) -> std::result::Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|error| format!("读取 OpenCode 账号快照失败: {error}"))?;
        let mut auth: OpenCodeOpenAiAuth = serde_json::from_str(&content)
            .map_err(|error| format!("解析 OpenCode 账号快照失败: {error}"))?;

        auth.access = Some(new_tokens.access_token.clone());
        if let Some(refresh) = new_tokens.refresh_token.clone() {
            auth.refresh = Some(refresh);
        }
        if let Some(id_token) = new_tokens.id_token.clone() {
            auth.extra
                .insert("idToken".to_string(), JsonValue::String(id_token));
        }

        let serialized = serde_json::to_string_pretty(&auth)
            .map_err(|error| format!("序列化 OpenCode 账号快照失败: {error}"))?;
        AtomicWriter::new(path)
            .write_string(&serialized)
            .map_err(|error| format!("写回 OpenCode 账号快照失败: {error}"))?;
        ensure_private_permissions(path);
        Ok(())
    }

    fn persist_current_tokens(
        path: &Path,
        new_tokens: &TokenRefreshResponse,
    ) -> std::result::Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|error| format!("读取 OpenCode auth.json 失败: {error}"))?;
        let mut root: JsonMap<String, JsonValue> = serde_json::from_str(&content)
            .map_err(|error| format!("解析 OpenCode auth.json 失败: {error}"))?;
        let openai = root
            .get("openai")
            .cloned()
            .filter(|value| !value.is_null())
            .ok_or_else(|| "当前 OpenCode 未检测到 openai 登录".to_string())?;
        let mut auth: OpenCodeOpenAiAuth = serde_json::from_value(openai)
            .map_err(|error| format!("解析 OpenCode openai provider 失败: {error}"))?;

        auth.access = Some(new_tokens.access_token.clone());
        if let Some(refresh) = new_tokens.refresh_token.clone() {
            auth.refresh = Some(refresh);
        }
        if let Some(id_token) = new_tokens.id_token.clone() {
            auth.extra
                .insert("idToken".to_string(), JsonValue::String(id_token));
        }

        root.insert(
            "openai".to_string(),
            serde_json::to_value(auth)
                .map_err(|error| format!("序列化 OpenCode openai provider 失败: {error}"))?,
        );
        let serialized = serde_json::to_string_pretty(&root)
            .map_err(|error| format!("序列化 OpenCode auth.json 失败: {error}"))?;
        AtomicWriter::new(path)
            .write_string(&serialized)
            .map_err(|error| format!("写回 OpenCode auth.json 失败: {error}"))?;
        ensure_private_permissions(path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    fn sample_openai_auth() -> OpenCodeOpenAiAuth {
        OpenCodeOpenAiAuth {
            r#type: "oauth".to_string(),
            access: Some("access-token".to_string()),
            refresh: Some("refresh-token".to_string()),
            expires: Some(1_800_000_000_000),
            account_id: Some("acc-1".to_string()),
            extra: {
                let mut extra = indexmap::IndexMap::new();
                extra.insert(
                    "email".to_string(),
                    JsonValue::String("user@example.com".to_string()),
                );
                extra
            },
        }
    }

    #[test]
    fn load_snapshot_from_saved_file_reads_oauth_fields() {
        let temp = tempdir().unwrap();
        let snapshot_path = temp.path().join("saved.json");
        fs::write(
            &snapshot_path,
            serde_json::to_string_pretty(&sample_openai_auth()).unwrap(),
        )
        .unwrap();

        let snapshot = OpenCodeQuotaService::load_snapshot_from_saved_file(&snapshot_path).unwrap();
        assert_eq!(snapshot.account_id.as_deref(), Some("acc-1"));
        assert_eq!(snapshot.email.as_deref(), Some("user@example.com"));
        assert_eq!(snapshot.refresh_token.as_deref(), Some("refresh-token"));
    }

    #[test]
    fn persist_current_tokens_preserves_other_providers() {
        let temp = tempdir().unwrap();
        let auth_path = temp.path().join("auth.json");
        fs::write(
            &auth_path,
            serde_json::to_string_pretty(&json!({
                "openai": sample_openai_auth(),
                "github": { "token": "gh-token" }
            }))
            .unwrap(),
        )
        .unwrap();

        OpenCodeQuotaService::persist_current_tokens(
            &auth_path,
            &TokenRefreshResponse {
                access_token: "new-access".to_string(),
                id_token: Some("new-id".to_string()),
                refresh_token: Some("new-refresh".to_string()),
            },
        )
        .unwrap();

        let root: JsonValue =
            serde_json::from_str(&fs::read_to_string(&auth_path).unwrap()).unwrap();
        assert_eq!(
            root.get("github")
                .and_then(|value| value.get("token"))
                .and_then(JsonValue::as_str),
            Some("gh-token")
        );
        assert_eq!(
            root.get("openai")
                .and_then(|value| value.get("access"))
                .and_then(JsonValue::as_str),
            Some("new-access")
        );
        assert_eq!(
            root.get("openai")
                .and_then(|value| value.get("refresh"))
                .and_then(JsonValue::as_str),
            Some("new-refresh")
        );
    }

    #[test]
    fn persist_saved_tokens_updates_snapshot_file() {
        let temp = tempdir().unwrap();
        let snapshot_path = temp.path().join("saved.json");
        fs::write(
            &snapshot_path,
            serde_json::to_string_pretty(&sample_openai_auth()).unwrap(),
        )
        .unwrap();

        OpenCodeQuotaService::persist_saved_tokens(
            &snapshot_path,
            &TokenRefreshResponse {
                access_token: "rotated-access".to_string(),
                id_token: None,
                refresh_token: Some("rotated-refresh".to_string()),
            },
        )
        .unwrap();

        let saved: JsonValue =
            serde_json::from_str(&fs::read_to_string(&snapshot_path).unwrap()).unwrap();
        assert_eq!(
            saved.get("access").and_then(JsonValue::as_str),
            Some("rotated-access")
        );
        assert_eq!(
            saved.get("refresh").and_then(JsonValue::as_str),
            Some("rotated-refresh")
        );
    }
}
