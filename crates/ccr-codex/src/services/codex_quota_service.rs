// 💰 Codex 配额查询服务
// 查询 Codex 账号或当前 runtime 登录的 API 配额余额（wham/usage API）。

use crate::models::{CodexAccountQuota, CodexAuthJson, CodexAuthRegistry};
use crate::utils::{CodexPaths, ensure_private_permissions};
use ccr_core::core::atomic_writer::AtomicWriter;
use ccr_core::core::error::Result;
use chrono::Utc;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

use super::codex_oauth_token_service::CodexOAuthTokenService;
use super::openai_quota_core::{
    OpenAiQuotaCore, OpenAiQuotaFetchOutcome, OpenAiQuotaSnapshot, TokenRefreshResponse,
};

/// 并发查询上限
const MAX_CONCURRENT: usize = 5;

/// Codex 配额查询服务。
pub struct CodexQuotaService {
    /// CCR Codex 数据目录 (~/.ccr/platforms/codex/)
    ccr_codex_dir: PathBuf,
    /// Codex 运行时目录 (~/.codex/)
    codex_dir: PathBuf,
}

impl CodexQuotaService {
    pub fn new() -> Result<Self> {
        let paths = CodexPaths::resolve()?;
        Ok(Self {
            ccr_codex_dir: paths.ccr_codex_dir,
            codex_dir: paths.codex_dir,
        })
    }

    /// 查询指定账号的配额。
    pub async fn fetch_account_quota(&self, account_name: &str) -> CodexAccountQuota {
        self.fetch_account_quota_inner(account_name, false).await
    }

    /// 查询指定账号的配额（强制 refresh token）。
    pub async fn fetch_account_quota_force_refresh(&self, account_name: &str) -> CodexAccountQuota {
        self.fetch_account_quota_inner(account_name, true).await
    }

    /// 查询当前 runtime 登录的配额（未保存登录也可用）。
    pub async fn fetch_current_quota(&self) -> CodexAccountQuota {
        self.fetch_current_quota_inner(false).await
    }

    /// 查询当前 runtime 登录的配额（强制 refresh token）。
    pub async fn fetch_current_quota_force_refresh(&self) -> CodexAccountQuota {
        self.fetch_current_quota_inner(true).await
    }

    /// 按给定账号顺序批量查询配额。
    ///
    /// 特殊 key:
    /// - `default`: 当前 runtime 登录
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

    /// 并发查询所有已保存账号的配额。
    pub async fn fetch_all_quotas(&self) -> Vec<CodexAccountQuota> {
        self.fetch_all_quotas_inner(false).await
    }

    /// 并发查询所有已保存账号的配额（强制 refresh token）。
    pub async fn fetch_all_quotas_force_refresh(&self) -> Vec<CodexAccountQuota> {
        self.fetch_all_quotas_inner(true).await
    }

    pub fn is_token_expired(access_token: &str) -> bool {
        OpenAiQuotaCore::is_token_expired(access_token)
    }

    pub fn extract_account_id(access_token: &str) -> Option<String> {
        OpenAiQuotaCore::extract_account_id(access_token)
    }

    pub fn format_reset_duration(reset_timestamp: i64) -> String {
        OpenAiQuotaCore::format_reset_duration(reset_timestamp)
    }

    async fn fetch_account_quota_inner(
        &self,
        account_name: &str,
        force_refresh: bool,
    ) -> CodexAccountQuota {
        let fetched_at = Utc::now();
        let auth_path = self.account_auth_path(account_name);
        let snapshot = match Self::load_snapshot_from_path(&auth_path) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                return CodexAccountQuota {
                    account_name: account_name.to_string(),
                    email: None,
                    quota: None,
                    error: Some(error.to_string()),
                    fetched_at,
                };
            }
        };

        match self
            .fetch_saved_snapshot_with_repair(
                account_name,
                &auth_path,
                snapshot.clone(),
                force_refresh,
            )
            .await
        {
            Ok(outcome) => Self::build_success(account_name, outcome, fetched_at),
            Err(error) => CodexAccountQuota {
                account_name: account_name.to_string(),
                email: snapshot.email,
                quota: None,
                error: Some(error),
                fetched_at,
            },
        }
    }

    async fn fetch_current_quota_inner(&self, force_refresh: bool) -> CodexAccountQuota {
        let fetched_at = Utc::now();
        let auth_path = self.current_auth_path();
        let snapshot = match Self::load_snapshot_from_path(&auth_path) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                return CodexAccountQuota {
                    account_name: "default".to_string(),
                    email: None,
                    quota: None,
                    error: Some(error.to_string()),
                    fetched_at,
                };
            }
        };

        match Self::fetch_snapshot_quota(snapshot.clone(), force_refresh, |tokens| {
            Self::update_auth_file(&auth_path, tokens)
        })
        .await
        {
            Ok(outcome) => Self::build_success("default", outcome, fetched_at),
            Err(error) => CodexAccountQuota {
                account_name: "default".to_string(),
                email: snapshot.email,
                quota: None,
                error: Some(error),
                fetched_at,
            },
        }
    }

    async fn fetch_saved_snapshot_with_repair(
        &self,
        account_name: &str,
        auth_path: &Path,
        snapshot: OpenAiQuotaSnapshot,
        force_refresh: bool,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, String> {
        match Self::fetch_snapshot_quota(snapshot, force_refresh, |tokens| {
            Self::update_auth_file(auth_path, tokens)
        })
        .await
        {
            Ok(outcome) => Ok(outcome),
            Err(error) if OpenAiQuotaCore::should_repair_tokens(&error) => {
                if let Ok(oauth) = CodexOAuthTokenService::new() {
                    match oauth.repair_saved_account(account_name) {
                        Ok(outcome) if outcome.updated => {
                            debug!(
                                "Repaired OAuth tokens for '{}' from {}",
                                account_name,
                                outcome
                                    .source
                                    .as_ref()
                                    .map(|source| source.label())
                                    .unwrap_or_else(|| "-".to_string())
                            );
                        }
                        Ok(outcome) => {
                            debug!(
                                "OAuth repair skipped for '{}': {}",
                                account_name, outcome.message
                            );
                        }
                        Err(repair_error) => {
                            warn!(
                                "OAuth repair failed for '{}': {}",
                                account_name, repair_error
                            );
                        }
                    }
                }

                let repaired_snapshot = Self::load_snapshot_from_path(auth_path)
                    .map_err(|repair_error| repair_error.to_string())?;
                Self::fetch_snapshot_quota(repaired_snapshot, force_refresh, |tokens| {
                    Self::update_auth_file(auth_path, tokens)
                })
                .await
            }
            Err(error) => Err(error),
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

    async fn fetch_all_quotas_inner(&self, force_refresh: bool) -> Vec<CodexAccountQuota> {
        let registry = match self.load_registry() {
            Ok(registry) => registry,
            Err(error) => {
                return vec![CodexAccountQuota {
                    account_name: "(registry)".to_string(),
                    email: None,
                    quota: None,
                    error: Some(format!("加载注册表失败: {error}")),
                    fetched_at: Utc::now(),
                }];
            }
        };

        if registry.accounts.is_empty() {
            return vec![];
        }

        let account_names = registry.accounts.keys().cloned().collect::<Vec<_>>();
        self.fetch_quotas_for_accounts_inner(&account_names, force_refresh)
            .await
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
        let ccr_codex_dir = self.ccr_codex_dir.clone();
        let codex_dir = self.codex_dir.clone();

        let tasks: Vec<_> = account_names
            .iter()
            .map(|name| {
                let semaphore = semaphore.clone();
                let ccr_codex_dir = ccr_codex_dir.clone();
                let codex_dir = codex_dir.clone();
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
                    let service = CodexQuotaService {
                        ccr_codex_dir,
                        codex_dir,
                    };
                    if name == "default" {
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

    fn current_auth_path(&self) -> PathBuf {
        self.codex_dir.join("auth.json")
    }

    fn account_auth_path(&self, name: &str) -> PathBuf {
        self.ccr_codex_dir.join("auth").join(format!("{name}.json"))
    }

    fn load_registry(&self) -> Result<CodexAuthRegistry> {
        super::codex_registry_store::CodexRegistryStore::new(&self.ccr_codex_dir).load()
    }

    fn load_snapshot_from_path(path: &Path) -> Result<OpenAiQuotaSnapshot> {
        let auth_json = std::fs::read_to_string(path).map_err(|error| {
            ccr_core::core::error::CcrError::ConfigError(format!("读取 auth 文件失败: {error}"))
        })?;
        let auth: CodexAuthJson = serde_json::from_str(&auth_json).map_err(|error| {
            ccr_core::core::error::CcrError::ConfigError(format!("解析 auth JSON 失败: {error}"))
        })?;

        let tokens = auth.tokens.ok_or_else(|| {
            ccr_core::core::error::CcrError::ConfigError(
                "账号缺少 OAuth tokens（可能是 API Key 模式）".to_string(),
            )
        })?;

        Ok(OpenAiQuotaSnapshot {
            access_token: tokens.access_token.unwrap_or_default(),
            refresh_token: tokens.refresh_token,
            account_id: tokens.account_id,
            email: None,
        })
    }

    fn update_auth_file(
        auth_path: &Path,
        new_tokens: &TokenRefreshResponse,
    ) -> std::result::Result<(), String> {
        let original_json = std::fs::read_to_string(auth_path)
            .map_err(|error| format!("读取 auth 文件失败: {error}"))?;
        let mut value: serde_json::Value = serde_json::from_str(&original_json)
            .map_err(|error| format!("解析 auth JSON 失败: {error}"))?;

        if let Some(tokens) = value
            .get_mut("tokens")
            .and_then(|tokens| tokens.as_object_mut())
        {
            tokens.insert(
                "access_token".to_string(),
                serde_json::Value::String(new_tokens.access_token.clone()),
            );
            if let Some(id_token) = new_tokens.id_token.clone() {
                tokens.insert("id_token".to_string(), serde_json::Value::String(id_token));
            }
            if let Some(refresh_token) = new_tokens.refresh_token.clone() {
                tokens.insert(
                    "refresh_token".to_string(),
                    serde_json::Value::String(refresh_token),
                );
            }
        } else {
            return Err("auth 文件缺少 tokens 字段".to_string());
        }

        value["last_refresh"] = serde_json::Value::String(Utc::now().to_rfc3339());

        let content = serde_json::to_string_pretty(&value)
            .map_err(|error| format!("序列化 auth 文件失败: {error}"))?;
        AtomicWriter::new(auth_path)
            .write_string(&content)
            .map_err(|error| format!("写回 auth 文件失败: {error}"))?;
        ensure_private_permissions(auth_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::models::CodexAuthTokens;
    use chrono::Duration;
    use serde_json::json;
    use tempfile::TempDir;

    fn create_test_service() -> (CodexQuotaService, TempDir, TempDir) {
        let ccr = TempDir::new().unwrap();
        let codex = TempDir::new().unwrap();
        (
            CodexQuotaService {
                ccr_codex_dir: ccr.path().join("platforms").join("codex"),
                codex_dir: codex.path().to_path_buf(),
            },
            ccr,
            codex,
        )
    }

    fn fake_jwt(payload: serde_json::Value) -> String {
        let header = r#"{"alg":"none","typ":"JWT"}"#;
        format!(
            "{}.{}.signature",
            base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, header),
            base64::Engine::encode(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                payload.to_string()
            )
        )
    }

    fn write_auth_file(path: &Path, access_token: &str, refresh_token: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let auth = CodexAuthJson {
            openai_api_key: None,
            tokens: Some(CodexAuthTokens {
                id_token: Some("id-token".to_string()),
                access_token: Some(access_token.to_string()),
                refresh_token: Some(refresh_token.to_string()),
                account_id: Some("acc-1".to_string()),
            }),
            last_refresh: Some(Utc::now().to_rfc3339()),
        };
        std::fs::write(path, serde_json::to_string_pretty(&auth).unwrap()).unwrap();
    }

    #[test]
    fn load_snapshot_from_path_reads_codex_tokens() {
        let (_service, _ccr, codex) = create_test_service();
        let auth_path = codex.path().join("auth.json");
        let token = fake_jwt(json!({
            "email": "user@example.com",
            "chatgpt_account_id": "acc-1",
            "exp": (Utc::now() + Duration::hours(1)).timestamp()
        }));
        write_auth_file(&auth_path, &token, "refresh-token");

        let snapshot = CodexQuotaService::load_snapshot_from_path(&auth_path).unwrap();
        assert_eq!(snapshot.account_id.as_deref(), Some("acc-1"));
        assert_eq!(snapshot.refresh_token.as_deref(), Some("refresh-token"));
        assert!(snapshot.email.is_none());
    }

    #[test]
    fn update_auth_file_rewrites_access_and_refresh_tokens() {
        let (_service, _ccr, codex) = create_test_service();
        let auth_path = codex.path().join("auth.json");
        let token = fake_jwt(json!({
            "email": "user@example.com",
            "chatgpt_account_id": "acc-1",
            "exp": (Utc::now() + Duration::hours(1)).timestamp()
        }));
        write_auth_file(&auth_path, &token, "refresh-token");

        CodexQuotaService::update_auth_file(
            &auth_path,
            &TokenRefreshResponse {
                access_token: "rotated-access".to_string(),
                id_token: Some("rotated-id".to_string()),
                refresh_token: Some("rotated-refresh".to_string()),
            },
        )
        .unwrap();

        let saved: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&auth_path).unwrap()).unwrap();
        assert_eq!(
            saved
                .get("tokens")
                .and_then(|value| value.get("access_token"))
                .and_then(serde_json::Value::as_str),
            Some("rotated-access")
        );
        assert_eq!(
            saved
                .get("tokens")
                .and_then(|value| value.get("refresh_token"))
                .and_then(serde_json::Value::as_str),
            Some("rotated-refresh")
        );
    }
}
