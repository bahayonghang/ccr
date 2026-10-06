// 💰 Codex 配额查询服务
// 查询 Codex 账号或当前 runtime 登录的 API 配额余额（wham/usage API）。

use crate::models::{CodexAccountQuota, CodexAuthJson, CodexAuthRegistry};
use crate::utils::{CodexPaths, ensure_private_permissions};
use ccr_core::core::atomic_writer::AsyncAtomicWriter;
use ccr_core::core::error::Result;
use chrono::Utc;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, warn};

use super::codex_oauth_token_service::CodexOAuthTokenService;
use super::openai_quota_core::{
    OpenAiQuotaCore, OpenAiQuotaFetchOutcome, OpenAiQuotaSnapshot, TokenRefreshResponse,
};

/// 并发查询上限
const MAX_CONCURRENT: usize = 5;

fn observations_from_outcome(
    outcome: &OpenAiQuotaFetchOutcome,
) -> Vec<crate::managers::codex_quota_observation::CodexQuotaObservation> {
    use crate::managers::codex_quota_observation::CodexQuotaObservation;

    let Some(account_id) = outcome
        .account_id
        .as_ref()
        .filter(|id| !id.is_empty() && !outcome.cache_hit)
    else {
        return Vec::new();
    };
    let raw = outcome.quota.raw_data.as_ref();
    let limit = raw.and_then(|value| value.get("rate_limit"));
    let explicit_id = limit
        .and_then(|value| value.get("limit_id").or_else(|| value.get("id")))
        .or_else(|| raw.and_then(|value| value.get("limit_id")))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let has_unsupported_additional_bucket = raw
        .and_then(|value| value.get("additional_rate_limits"))
        .is_some_and(|value| value.as_array().is_none_or(|values| !values.is_empty()));

    ["primary_window", "secondary_window"]
        .into_iter()
        .filter_map(|role| {
            let window = limit.and_then(|value| value.get(role))?;
            let seconds = window
                .get("limit_window_seconds")
                .and_then(serde_json::Value::as_i64);
            let reset_at = window.get("reset_at").and_then(serde_json::Value::as_i64);
            let reset_after = window
                .get("reset_after_seconds")
                .and_then(serde_json::Value::as_i64)
                .filter(|value| *value >= 0);
            let resets_at = reset_at.or_else(|| {
                reset_after.map(|value| outcome.network_acquired_at.timestamp() + value)
            });
            // A countdown is not a stable reset identity. Keep its display time,
            // but do not calibrate across responses without a server reset ID.
            let reset_generation = reset_at.map(|value| format!("reset_at:{value}"));
            Some(CodexQuotaObservation {
                account_id: account_id.clone(),
                plan: outcome.quota.plan_type.clone(),
                source: "network".into(),
                bucket_source: if explicit_id.is_some() {
                    "wham_limit_id"
                } else {
                    "wham_main"
                }
                .into(),
                bucket_id: explicit_id.clone(),
                window_role: role.into(),
                duration_minutes: seconds
                    .filter(|value| *value > 0 && *value % 60 == 0)
                    .map(|value| value / 60),
                limit_window_seconds: seconds,
                used_percent: window
                    .get("used_percent")
                    .and_then(serde_json::Value::as_f64),
                resets_at,
                reset_generation,
                network_acquired_at: outcome.network_acquired_at,
                returned_at: outcome.returned_at,
                scope_supported: !has_unsupported_additional_bucket,
                ..Default::default()
            })
        })
        .collect()
}

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
        let snapshot = match Self::load_snapshot_from_path(&auth_path).await {
            Ok(snapshot) => snapshot,
            Err(error) => {
                return CodexAccountQuota {
                    account_name: account_name.to_string(),
                    email: None,
                    quota: None,
                    error: Some(error.to_string()),
                    fetched_at,
                    observation: None,
                };
            }
        };

        match self
            .fetch_saved_snapshot_with_repair(
                account_name,
                auth_path.clone(),
                snapshot.clone(),
                force_refresh,
            )
            .await
        {
            Ok(outcome) => self.build_success(account_name, outcome).await,
            Err(error) => CodexAccountQuota {
                account_name: account_name.to_string(),
                email: snapshot.email,
                quota: None,
                error: Some(error),
                fetched_at,
                observation: None,
            },
        }
    }

    async fn fetch_current_quota_inner(&self, force_refresh: bool) -> CodexAccountQuota {
        let fetched_at = Utc::now();
        let auth_path = self.current_auth_path();
        let snapshot = match Self::load_snapshot_from_path(&auth_path).await {
            Ok(snapshot) => snapshot,
            Err(error) => {
                return CodexAccountQuota {
                    account_name: "default".to_string(),
                    email: None,
                    quota: None,
                    error: Some(error.to_string()),
                    fetched_at,
                    observation: None,
                };
            }
        };

        match Self::fetch_snapshot_quota(snapshot.clone(), force_refresh, |tokens| {
            let auth_path = auth_path.clone();
            async move { Self::update_auth_file(&auth_path, &tokens).await }
        })
        .await
        {
            Ok(outcome) => self.build_success("default", outcome).await,
            Err(error) => CodexAccountQuota {
                account_name: "default".to_string(),
                email: snapshot.email,
                quota: None,
                error: Some(error),
                fetched_at,
                observation: None,
            },
        }
    }

    async fn fetch_saved_snapshot_with_repair(
        &self,
        account_name: &str,
        auth_path: PathBuf,
        snapshot: OpenAiQuotaSnapshot,
        force_refresh: bool,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, String> {
        match Self::fetch_snapshot_quota(snapshot, force_refresh, |tokens| {
            let auth_path = auth_path.clone();
            async move { Self::update_auth_file(&auth_path, &tokens).await }
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

                let repaired_snapshot = Self::load_snapshot_from_path(&auth_path)
                    .await
                    .map_err(|repair_error| repair_error.to_string())?;
                Self::fetch_snapshot_quota(repaired_snapshot, force_refresh, |tokens| {
                    let auth_path = auth_path.clone();
                    async move { Self::update_auth_file(&auth_path, &tokens).await }
                })
                .await
            }
            Err(error) => Err(error),
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

    async fn fetch_all_quotas_inner(&self, force_refresh: bool) -> Vec<CodexAccountQuota> {
        let registry = match self.load_registry().await {
            Ok(registry) => registry,
            Err(error) => {
                return vec![CodexAccountQuota {
                    account_name: "(registry)".to_string(),
                    email: None,
                    quota: None,
                    error: Some(format!("加载注册表失败: {error}")),
                    fetched_at: Utc::now(),
                    observation: None,
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
                                observation: None,
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

    async fn build_success(
        &self,
        account_name: &str,
        outcome: OpenAiQuotaFetchOutcome,
    ) -> CodexAccountQuota {
        let mut observations = observations_from_outcome(&outcome);
        let route_supported = match fs::read_to_string(self.codex_dir.join("config.toml")).await {
            Ok(text) => super::codex_usage_estimation::config_route_supported(&text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            Err(_) => false,
        };
        for observation in &mut observations {
            observation.scope_supported &= route_supported;
        }
        let history_warning = if !outcome.cache_hit {
            if observations.is_empty() {
                Some("quota_observation_identity_missing".to_string())
            } else {
                let path = self.ccr_codex_dir.join("quota_observations.json");
                match tokio::task::spawn_blocking(move || {
                    crate::managers::codex_quota_observation::CodexQuotaObservationStore::with_path(
                        path,
                    )
                    .record(observations, Utc::now())
                })
                .await
                {
                    Ok(Ok(())) => None,
                    _ => Some("quota_history_write_failed".to_string()),
                }
            }
        } else {
            None
        };
        let observation = crate::models::CodexQuotaProvenance {
            account_id: outcome.account_id,
            request_started_at: outcome.request_started_at,
            network_acquired_at: outcome.network_acquired_at,
            returned_at: outcome.returned_at,
            cache_hit: outcome.cache_hit,
            history_warning,
        };
        CodexAccountQuota {
            account_name: account_name.to_string(),
            email: outcome.email,
            quota: Some(outcome.quota),
            error: None,
            fetched_at: outcome.network_acquired_at,
            observation: Some(observation),
        }
    }

    fn current_auth_path(&self) -> PathBuf {
        self.codex_dir.join("auth.json")
    }

    fn account_auth_path(&self, name: &str) -> PathBuf {
        self.ccr_codex_dir.join("auth").join(format!("{name}.json"))
    }

    async fn load_registry(&self) -> Result<CodexAuthRegistry> {
        let ccr_codex_dir = self.ccr_codex_dir.clone();
        tokio::task::spawn_blocking(move || {
            super::codex_registry_store::CodexRegistryStore::new(&ccr_codex_dir).load()
        })
        .await
        .map_err(|error| {
            ccr_core::core::error::CcrError::ConfigError(format!("读取注册表任务失败: {error}"))
        })?
    }

    async fn load_snapshot_from_path(path: &Path) -> Result<OpenAiQuotaSnapshot> {
        let auth_json = fs::read_to_string(path).await.map_err(|error| {
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

    async fn update_auth_file(
        auth_path: &Path,
        new_tokens: &TokenRefreshResponse,
    ) -> std::result::Result<(), String> {
        let original_json = fs::read_to_string(auth_path)
            .await
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
        AsyncAtomicWriter::new(auth_path)
            .secret(true)
            .preserve_mode(true)
            .write_string_async(&content)
            .await
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

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let snapshot = runtime
            .block_on(CodexQuotaService::load_snapshot_from_path(&auth_path))
            .unwrap();
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

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime
            .block_on(CodexQuotaService::update_auth_file(
                &auth_path,
                &TokenRefreshResponse {
                    access_token: "rotated-access".to_string(),
                    id_token: Some("rotated-id".to_string()),
                    refresh_token: Some("rotated-refresh".to_string()),
                },
            ))
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

    fn outcome(acquired_at: chrono::DateTime<Utc>) -> OpenAiQuotaFetchOutcome {
        OpenAiQuotaFetchOutcome {
            email: Some("synthetic@example.invalid".into()),
            account_id: Some("stable-account".into()),
            request_started_at: acquired_at - Duration::seconds(1),
            network_acquired_at: acquired_at,
            returned_at: acquired_at + Duration::milliseconds(1),
            cache_hit: false,
            quota: crate::models::CodexQuota {
                hourly_percentage: 50,
                hourly_reset_time: Some(acquired_at.timestamp() + 18000),
                hourly_window_minutes: Some(300),
                hourly_window_present: Some(true),
                weekly_percentage: 80,
                weekly_reset_time: Some(acquired_at.timestamp() + 604800),
                weekly_window_minutes: Some(10080),
                weekly_window_present: Some(true),
                plan_type: Some("plus".into()),
                raw_data: Some(
                    json!({"rate_limit":{"primary_window":{"used_percent":20.25,"limit_window_seconds":604800,"reset_at":acquired_at.timestamp()+604800},"secondary_window":{"used_percent":50.125,"limit_window_seconds":18000,"reset_at":acquired_at.timestamp()+18000}},"code_review_rate_limit":{"primary_window":{"used_percent":99,"limit_window_seconds":18000}},"email":"raw-secret","access_token":"raw-secret"}),
                ),
            },
        }
    }

    #[test]
    fn observation_metadata_uses_actual_duration_raw_percent_and_reset_identity() {
        let acquired = Utc::now();
        let mut value = outcome(acquired);
        let observations = observations_from_outcome(&value);
        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].duration_minutes, Some(10080));
        assert_eq!(observations[1].duration_minutes, Some(300));
        assert_eq!(observations[1].used_percent, Some(50.125));
        assert_eq!(observations[0].bucket_source, "wham_main");
        assert!(
            observations
                .iter()
                .all(|sample| sample.reset_generation.is_some())
        );
        let raw = value.quota.raw_data.as_mut().unwrap();
        raw["rate_limit"]["secondary_window"]
            .as_object_mut()
            .unwrap()
            .remove("reset_at");
        raw["rate_limit"]["secondary_window"]["reset_after_seconds"] = json!(3000);
        raw["rate_limit"]["limit_id"] = json!("explicit-model-bucket");
        raw["additional_rate_limits"] = json!([{"limit_id":"other"}]);
        let observations = observations_from_outcome(&value);
        assert_eq!(observations[1].resets_at, Some(acquired.timestamp() + 3000));
        assert!(observations[1].reset_generation.is_none());
        assert_eq!(observations[1].bucket_source, "wham_limit_id");
        assert!(!observations[1].scope_supported);
        assert!(
            !serde_json::to_string(&observations)
                .unwrap()
                .contains("raw-secret")
        );
        value.cache_hit = true;
        assert!(observations_from_outcome(&value).is_empty());
        value.cache_hit = false;
        value.account_id = None;
        assert!(observations_from_outcome(&value).is_empty());
    }

    #[test]
    fn success_cache_and_rename_preserve_real_acquisition_and_single_history_sample() {
        let env = crate::test_support::TestCodexEnv::new();
        let service = CodexQuotaService {
            ccr_codex_dir: env.ccr_codex_dir().to_path_buf(),
            codex_dir: env.codex_dir().to_path_buf(),
        };
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let acquired = Utc::now();
        let value = outcome(acquired);
        let first = runtime.block_on(service.build_success("old-name", value.clone()));
        assert_eq!(first.fetched_at, acquired);
        assert!(!first.observation.as_ref().unwrap().cache_hit);
        assert!(
            first
                .observation
                .as_ref()
                .unwrap()
                .history_warning
                .is_none()
        );
        let path = env.ccr_codex_dir().join("quota_observations.json");
        let before = std::fs::read(&path).unwrap();
        let mut cached = value.clone();
        cached.cache_hit = true;
        cached.returned_at += Duration::seconds(10);
        let second = runtime.block_on(service.build_success("new-name", cached));
        assert_eq!(second.fetched_at, first.fetched_at);
        assert_eq!(second.account_name, "new-name");
        assert_eq!(
            second.observation.as_ref().unwrap().account_id,
            first.observation.as_ref().unwrap().account_id
        );
        assert!(second.observation.as_ref().unwrap().cache_hit);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let history =
            crate::managers::codex_quota_observation::CodexQuotaObservationStore::with_path(path)
                .load()
                .unwrap();
        assert_eq!(history.len(), 2);
        let text = String::from_utf8(before).unwrap();
        assert!(!text.contains("raw-secret"));
        assert!(!text.contains("synthetic@example.invalid"));
        assert!(!text.contains("old-name"));
    }

    #[test]
    fn successful_quota_survives_history_failure_and_custom_route_scope() {
        let env = crate::test_support::TestCodexEnv::new();
        let service = CodexQuotaService {
            ccr_codex_dir: env.ccr_codex_dir().to_path_buf(),
            codex_dir: env.codex_dir().to_path_buf(),
        };
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let path = env.ccr_codex_dir().join("quota_observations.json");
        std::fs::write(&path, b"corrupt synthetic history").unwrap();
        let result = runtime.block_on(service.build_success("main", outcome(Utc::now())));
        assert!(result.quota.is_some());
        assert!(result.error.is_none());
        assert_eq!(
            result.observation.unwrap().history_warning.as_deref(),
            Some("quota_history_write_failed")
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"corrupt synthetic history");
        // A separate target establishes route rejection without replacing corrupt history.
        let service = CodexQuotaService {
            ccr_codex_dir: env.ccr_codex_dir().join("custom-route"),
            codex_dir: env.codex_dir().to_path_buf(),
        };
        std::fs::write(
            env.codex_dir().join("config.toml"),
            "model_provider='custom'",
        )
        .unwrap();
        let result = runtime.block_on(service.build_success("main", outcome(Utc::now())));
        assert!(result.quota.is_some());
        let history =
            crate::managers::codex_quota_observation::CodexQuotaObservationStore::with_path(
                service.ccr_codex_dir.join("quota_observations.json"),
            )
            .load()
            .unwrap();
        assert!(history.iter().all(|sample| !sample.scope_supported));
    }
}
