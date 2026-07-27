// 💰 OpenCode 配额查询服务
// 基于 OpenCode 当前 openai provider 或已保存账号快照查询 wham/usage。

use super::openai_quota_core::{
    OpenAiQuotaCore, OpenAiQuotaFetchOutcome, OpenAiQuotaSnapshot, TokenRefreshResponse,
};
use crate::models::{CodexAccountQuota, OpenCodeOpenAiAuth};
use crate::utils::{OpenCodePaths, ensure_private_permissions};
use ccr_core::core::atomic_writer::AsyncAtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use chrono::Utc;
use serde_json::{Map as JsonMap, Value as JsonValue};
#[cfg(test)]
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

/// 并发查询上限
const MAX_CONCURRENT: usize = 5;

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

    /// 按给定账号顺序批量查询配额。
    ///
    /// 特殊 key:
    /// - `current-login`: 当前 runtime 登录
    pub async fn fetch_quotas_for_accounts(
        &self,
        account_names: &[String],
    ) -> Vec<CodexAccountQuota> {
        self.fetch_quotas_for_accounts_inner(account_names, false)
            .await
    }

    /// 按给定账号顺序批量查询配额（强制 refresh token）。
    pub async fn fetch_quotas_for_accounts_force_refresh(
        &self,
        account_names: &[String],
    ) -> Vec<CodexAccountQuota> {
        self.fetch_quotas_for_accounts_inner(account_names, true)
            .await
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
        let snapshot = match Self::load_snapshot_from_saved_file(&auth_path).await {
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
            let auth_path = auth_path.clone();
            async move { Self::persist_saved_tokens(&auth_path, &tokens).await }
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
        let snapshot = match Self::load_snapshot_from_current_auth(&auth_path).await {
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
            let auth_path = auth_path.clone();
            async move { Self::persist_current_tokens(&auth_path, &tokens).await }
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

    async fn fetch_snapshot_quota<F, Fut>(
        snapshot: OpenAiQuotaSnapshot,
        force_refresh: bool,
        persist_tokens: F,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, String>
    where
        F: FnMut(TokenRefreshResponse) -> Fut,
        Fut: std::future::Future<Output = std::result::Result<(), String>>,
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

    async fn fetch_quotas_for_accounts_inner(
        &self,
        account_names: &[String],
        force_refresh: bool,
    ) -> Vec<CodexAccountQuota> {
        if account_names.is_empty() {
            return Vec::new();
        }

        use futures::future::join_all;
        use std::sync::Arc;
        use tokio::sync::Semaphore;

        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
        let ccr_opencode_dir = self.ccr_opencode_dir.clone();
        let opencode_dir = self.opencode_dir.clone();

        let tasks: Vec<_> = account_names
            .iter()
            .map(|name| {
                let semaphore = semaphore.clone();
                let ccr_opencode_dir = ccr_opencode_dir.clone();
                let opencode_dir = opencode_dir.clone();
                let name = name.clone();
                async move {
                    let permit = match semaphore.acquire_owned().await {
                        Ok(permit) => permit,
                        Err(error) => {
                            return CodexAccountQuota {
                                account_name: name,
                                email: None,
                                quota: None,
                                error: Some(format!("获取并发许可失败: {error}")),
                                fetched_at: Utc::now(),
                            };
                        }
                    };
                    let _permit = permit;
                    let service = OpenCodeQuotaService {
                        ccr_opencode_dir,
                        opencode_dir,
                    };
                    if name == "current-login" {
                        service.fetch_current_quota_inner(force_refresh).await
                    } else {
                        service
                            .fetch_account_quota_inner(&name, force_refresh)
                            .await
                    }
                }
            })
            .collect();

        join_all(tasks).await
    }

    async fn load_snapshot_from_saved_file(path: &Path) -> Result<OpenAiQuotaSnapshot> {
        let content = async_fs::read_to_string(path).await.map_err(|error| {
            CcrError::ConfigError(format!("读取 OpenCode 账号快照失败: {error}"))
        })?;
        let auth: OpenCodeOpenAiAuth = serde_json::from_str(&content).map_err(|error| {
            CcrError::ConfigError(format!("解析 OpenCode 账号快照失败: {error}"))
        })?;
        Self::build_snapshot(auth)
    }

    async fn load_snapshot_from_current_auth(path: &Path) -> Result<OpenAiQuotaSnapshot> {
        let content = async_fs::read_to_string(path).await.map_err(|error| {
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

    async fn persist_saved_tokens(
        path: &Path,
        new_tokens: &TokenRefreshResponse,
    ) -> std::result::Result<(), String> {
        let content = async_fs::read_to_string(path)
            .await
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
        AsyncAtomicWriter::new(path)
            .secret(true)
            .preserve_mode(true)
            .write_string_async(&serialized)
            .await
            .map_err(|error| format!("写回 OpenCode 账号快照失败: {error}"))?;
        ensure_private_permissions(path);
        Ok(())
    }

    async fn persist_current_tokens(
        path: &Path,
        new_tokens: &TokenRefreshResponse,
    ) -> std::result::Result<(), String> {
        let content = async_fs::read_to_string(path)
            .await
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
        AsyncAtomicWriter::new(path)
            .secret(true)
            .preserve_mode(true)
            .write_string_async(&serialized)
            .await
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

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let snapshot = runtime
            .block_on(OpenCodeQuotaService::load_snapshot_from_saved_file(
                &snapshot_path,
            ))
            .unwrap();
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

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime
            .block_on(OpenCodeQuotaService::persist_current_tokens(
                &auth_path,
                &TokenRefreshResponse {
                    access_token: "new-access".to_string(),
                    id_token: Some("new-id".to_string()),
                    refresh_token: Some("new-refresh".to_string()),
                },
            ))
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

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime
            .block_on(OpenCodeQuotaService::persist_saved_tokens(
                &snapshot_path,
                &TokenRefreshResponse {
                    access_token: "rotated-access".to_string(),
                    id_token: None,
                    refresh_token: Some("rotated-refresh".to_string()),
                },
            ))
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
