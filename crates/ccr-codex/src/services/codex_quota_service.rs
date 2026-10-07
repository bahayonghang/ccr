// 💰 Codex 配额查询服务
// 查询 Codex 账号或当前 runtime 登录的 API 配额余额（wham/usage API）。

use crate::models::{CodexAccountQuota, CodexAuthJson, CodexAuthRegistry, CodexAuthTokens};
use crate::utils::CodexPaths;
use ccr_core::core::error::Result;
use ccr_core::core::guarded_write::{
    VersionedWriteOutcome, WriteOptions, content_version_token,
    enforce_owner_only_permissions_versioned, write_guarded_versioned_async,
};
use chrono::Utc;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, warn};

use super::codex_auth_identity::{OAuthIdentity, identity_from_auth};
use super::codex_auth_refresh_lock::{CredentialLocks, CredentialResource};
use super::codex_oauth_token_service::CodexOAuthTokenService;
use super::openai_quota_core::{
    OpenAiQuotaCore, OpenAiQuotaFetchOutcome, OpenAiQuotaSnapshot, TokenRefreshResponse,
};

/// 并发查询上限
const MAX_CONCURRENT: usize = 5;

// Core callback control stays private and is accepted only with a captured upgrade.
const IDENTITY_UPGRADED: &str = "ccr-private-quota-identity-upgraded";

enum SourceQuotaError {
    Message(String),
    IdentityUpgrade(OpenAiQuotaSnapshot),
}

impl From<String> for SourceQuotaError {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}

/// refresh_token 永久失效（已吊销/已使用/已过期）且无可用修复来源时的错误前缀
pub const RELOGIN_REQUIRED_PREFIX: &str = "需重新登录：";

/// 若错误带有「需重新登录」前缀，返回去掉前缀后的原始错误
pub fn relogin_required_detail(error: &str) -> Option<&str> {
    error.strip_prefix(RELOGIN_REQUIRED_PREFIX)
}

fn mark_relogin_if_permanent(error: String) -> String {
    if OpenAiQuotaCore::should_repair_tokens(&error) && relogin_required_detail(&error).is_none() {
        format!("{RELOGIN_REQUIRED_PREFIX}{error}")
    } else {
        error
    }
}

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

    /// 查询指定账号的配额（绕过配额缓存，取得新配额）。
    pub async fn fetch_account_quota_force_refresh(&self, account_name: &str) -> CodexAccountQuota {
        self.fetch_account_quota_inner(account_name, true).await
    }

    /// 查询当前 runtime 登录的配额（未保存登录也可用）。
    pub async fn fetch_current_quota(&self) -> CodexAccountQuota {
        self.fetch_current_quota_inner(false).await
    }

    /// 查询当前 runtime 登录的配额（绕过配额缓存，取得新配额）。
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

    /// 按给定账号顺序批量查询配额（绕过配额缓存，取得新配额）。
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

    /// 并发查询所有已保存账号的配额（绕过配额缓存，取得新配额）。
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
        // The first read selects the resource; the locked read selects credentials.
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

        let result = self
            .fetch_account_quota_locked(account_name, &snapshot, force_refresh)
            .await;

        match result {
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

    async fn fetch_account_quota_locked(
        &self,
        account_name: &str,
        initial: &OpenAiQuotaSnapshot,
        force_refresh: bool,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, String> {
        let mut expected = initial.clone();
        for attempt in 0..=1 {
            match self
                .fetch_account_quota_once(account_name, &expected, force_refresh)
                .await
            {
                Ok(outcome) => return Ok(outcome),
                Err(SourceQuotaError::IdentityUpgrade(upgraded)) if attempt == 0 => {
                    expected = upgraded
                }
                Err(SourceQuotaError::IdentityUpgrade(_)) => {
                    return Err("auth 来源身份已变化，跳过查询".into());
                }
                Err(SourceQuotaError::Message(error)) => return Err(error),
            }
        }
        Err("auth 来源身份已变化，跳过查询".into())
    }

    async fn fetch_account_quota_once(
        &self,
        account_name: &str,
        initial: &OpenAiQuotaSnapshot,
        force_refresh: bool,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, SourceQuotaError> {
        let saved_path = self.account_auth_path(account_name);
        let mut sources = vec![(
            saved_path.clone(),
            Self::snapshot_resource(initial, &saved_path),
        )];
        let runtime_path = self.current_auth_path();
        let runtime_candidate = if self
            .load_registry()
            .await
            .ok()
            .is_some_and(|registry| registry.current_auth.as_deref() == Some(account_name))
        {
            Self::load_snapshot_from_path(&runtime_path)
                .await
                .ok()
                .filter(|runtime| {
                    Self::snapshot_identity(initial).is_some()
                        && Self::snapshot_identity(initial) == Self::snapshot_identity(runtime)
                })
        } else {
            None
        };
        if let Some(runtime) = &runtime_candidate {
            sources.push((
                runtime_path.clone(),
                Self::snapshot_resource(runtime, &runtime_path),
            ));
        }
        let locks = CredentialLocks::acquire_async_sources(sources.clone())
            .await
            .map_err(|error| error.to_string())?;
        for (path, _) in &sources {
            locks.verify_path(path).map_err(|error| error.to_string())?;
        }
        let current = Self::load_snapshot_from_path(&saved_path)
            .await
            .map_err(|error| error.to_string())?;
        Self::verify_locked_snapshot(initial, &current, &saved_path)?;
        CodexOAuthTokenService::from_dirs(self.ccr_codex_dir.clone(), self.codex_dir.clone())
            .repair_saved_account_locked(account_name)
            .map_err(|error| error.to_string())?;
        let runtime_route =
            runtime_candidate.is_some() && self.route_active_account_to_runtime(account_name).await;
        let auth_path = if runtime_route {
            self.current_auth_path()
        } else {
            saved_path
        };
        let snapshot = Self::load_snapshot_from_path(&auth_path)
            .await
            .map_err(|error| error.to_string())?;
        Self::verify_locked_snapshot(initial, &snapshot, &auth_path)?;
        let result = if runtime_route {
            let result = Self::fetch_source_quota(snapshot, force_refresh, &auth_path).await;
            if matches!(result, Err(SourceQuotaError::IdentityUpgrade(_))) {
                return result;
            }
            // runtime 刷新落盘后把新 tokens 同步到快照（内容相同则不写）
            if locks.verify_path(&auth_path).is_ok() {
                self.sync_runtime_snapshot("quota refresh").await;
            }
            result.map_err(|error| match error {
                SourceQuotaError::Message(message) => {
                    SourceQuotaError::Message(mark_relogin_if_permanent(message))
                }
                upgrade => upgrade,
            })
        } else {
            self.fetch_saved_snapshot_with_repair(
                account_name,
                auth_path.clone(),
                snapshot,
                force_refresh,
            )
            .await
        };

        if result.is_ok()
            && locks.holds_resource(&CredentialResource::from_path(&auth_path))
            && let Err(error) = CodexOAuthTokenService::from_dirs(
                self.ccr_codex_dir.clone(),
                self.codex_dir.clone(),
            )
            .sync_registered_aliases_locked(&auth_path)
        {
            warn!("Quota alias synchronization failed: {}", error);
        }
        result
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

        let result = self
            .fetch_current_quota_locked(&snapshot, force_refresh)
            .await;
        match result {
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

    async fn fetch_current_quota_locked(
        &self,
        initial: &OpenAiQuotaSnapshot,
        force_refresh: bool,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, String> {
        let mut expected = initial.clone();
        for attempt in 0..=1 {
            match self
                .fetch_current_quota_once(&expected, force_refresh)
                .await
            {
                Ok(outcome) => return Ok(outcome),
                Err(SourceQuotaError::IdentityUpgrade(upgraded)) if attempt == 0 => {
                    expected = upgraded
                }
                Err(SourceQuotaError::IdentityUpgrade(_)) => {
                    return Err("auth 来源身份已变化，跳过查询".into());
                }
                Err(SourceQuotaError::Message(error)) => return Err(error),
            }
        }
        Err("auth 来源身份已变化，跳过查询".into())
    }

    async fn fetch_current_quota_once(
        &self,
        initial: &OpenAiQuotaSnapshot,
        force_refresh: bool,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, SourceQuotaError> {
        let auth_path = self.current_auth_path();
        let locks = CredentialLocks::acquire_async_sources(vec![(
            auth_path.clone(),
            Self::snapshot_resource(initial, &auth_path),
        )])
        .await
        .map_err(|error| error.to_string())?;
        let current = Self::load_snapshot_from_path(&auth_path)
            .await
            .map_err(|error| error.to_string())?;
        locks
            .verify_path(&auth_path)
            .map_err(|error| error.to_string())?;
        Self::verify_locked_snapshot(initial, &current, &auth_path)?;
        if let Some(identity) = Self::snapshot_identity(initial) {
            self.prepare_current_quota(&identity).await?;
            self.sync_runtime_snapshot("current quota route").await;
        }
        let current = Self::load_snapshot_from_path(&auth_path)
            .await
            .map_err(|error| error.to_string())?;
        Self::verify_locked_snapshot(initial, &current, &auth_path)?;
        let result = Self::fetch_source_quota(current, force_refresh, &auth_path).await;
        if result.is_ok()
            && locks.holds_resource(&CredentialResource::from_path(&auth_path))
            && let Err(error) = CodexOAuthTokenService::from_dirs(
                self.ccr_codex_dir.clone(),
                self.codex_dir.clone(),
            )
            .sync_registered_aliases_locked(&auth_path)
        {
            warn!("Quota alias synchronization failed: {}", error);
        }
        result
    }

    async fn fetch_saved_snapshot_with_repair(
        &self,
        account_name: &str,
        auth_path: PathBuf,
        snapshot: OpenAiQuotaSnapshot,
        force_refresh: bool,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, SourceQuotaError> {
        let initial = snapshot.clone();
        match Self::fetch_source_quota(snapshot, force_refresh, &auth_path).await {
            Ok(outcome) => Ok(outcome),
            Err(SourceQuotaError::Message(error))
                if OpenAiQuotaCore::should_repair_tokens(&error) =>
            {
                let oauth = CodexOAuthTokenService::from_dirs(
                    self.ccr_codex_dir.clone(),
                    self.codex_dir.clone(),
                );
                let repaired = match oauth.repair_saved_account_locked(account_name) {
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
                        true
                    }
                    Ok(outcome) => {
                        debug!(
                            "OAuth repair skipped for '{}': {}",
                            account_name, outcome.message
                        );
                        false
                    }
                    Err(repair_error) => {
                        warn!(
                            "OAuth repair failed for '{}': {}",
                            account_name, repair_error
                        );
                        false
                    }
                };
                // 无更新的修复来源时重试只会再次提交同一失效 token
                if !repaired {
                    return Err(mark_relogin_if_permanent(error).into());
                }

                let repaired_snapshot = Self::load_snapshot_from_path(&auth_path)
                    .await
                    .map_err(|repair_error| repair_error.to_string())?;
                Self::verify_locked_snapshot(&initial, &repaired_snapshot, &auth_path)?;
                Self::fetch_source_quota(repaired_snapshot, force_refresh, &auth_path)
                    .await
                    .map_err(|error| match error {
                        SourceQuotaError::Message(message) => {
                            SourceQuotaError::Message(mark_relogin_if_permanent(message))
                        }
                        upgrade => upgrade,
                    })
            }
            Err(error) => Err(error),
        }
    }

    async fn fetch_source_quota(
        snapshot: OpenAiQuotaSnapshot,
        force_refresh: bool,
        auth_path: &Path,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, SourceQuotaError> {
        let mut upgraded = None;
        let result = Self::fetch_snapshot_quota(snapshot, force_refresh, |expected, tokens| {
            let mut refreshed = expected.clone();
            refreshed.access_token = tokens.access_token.clone();
            if let Some(id) = &tokens.id_token {
                refreshed.id_token = Some(id.clone());
            }
            if let Some(refresh) = &tokens.refresh_token {
                refreshed.refresh_token = Some(refresh.clone());
            }
            let upgrade = Self::snapshot_identity(&expected).is_none()
                && Self::snapshot_identity(&refreshed).is_some();
            if upgrade {
                upgraded = Some(refreshed);
            }
            async move {
                Self::update_auth_file(auth_path, &expected, &tokens).await?;
                if upgrade {
                    Err(IDENTITY_UPGRADED.into())
                } else {
                    Ok(())
                }
            }
        })
        .await;
        match (result, upgraded) {
            (Err(control), Some(expected)) if control == IDENTITY_UPGRADED => {
                Err(SourceQuotaError::IdentityUpgrade(expected))
            }
            (result, _) => result.map_err(SourceQuotaError::Message),
        }
    }

    async fn fetch_snapshot_quota<F, Fut>(
        snapshot: OpenAiQuotaSnapshot,
        force_refresh: bool,
        mut persist_tokens: F,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, String>
    where
        F: FnMut(OpenAiQuotaSnapshot, TokenRefreshResponse) -> Fut,
        Fut: std::future::Future<Output = std::result::Result<(), String>>,
    {
        let mut expected = snapshot.clone();
        OpenAiQuotaCore::fetch_quota(snapshot, force_refresh, move |tokens| {
            let before = expected.clone();
            expected.access_token = tokens.access_token.clone();
            if let Some(id_token) = &tokens.id_token {
                expected.id_token = Some(id_token.clone());
            }
            if let Some(refresh_token) = &tokens.refresh_token {
                expected.refresh_token = Some(refresh_token.clone());
            }
            persist_tokens(before, tokens)
        })
        .await
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

    fn snapshot_resource(snapshot: &OpenAiQuotaSnapshot, path: &Path) -> CredentialResource {
        CredentialResource::from_tokens(
            Some(&CodexAuthTokens {
                id_token: snapshot.id_token.clone(),
                access_token: Some(snapshot.access_token.clone()),
                refresh_token: snapshot.refresh_token.clone(),
                account_id: snapshot.account_id.clone(),
            }),
            path,
        )
    }

    fn snapshot_identity(snapshot: &OpenAiQuotaSnapshot) -> Option<OAuthIdentity> {
        OAuthIdentity::from_tokens(&CodexAuthTokens {
            id_token: snapshot.id_token.clone(),
            access_token: Some(snapshot.access_token.clone()),
            refresh_token: snapshot.refresh_token.clone(),
            account_id: snapshot.account_id.clone(),
        })
    }

    async fn prepare_current_quota(
        &self,
        identity: &OAuthIdentity,
    ) -> std::result::Result<(), String> {
        let identity = identity.clone();
        let ccr_codex_dir = self.ccr_codex_dir.clone();
        let codex_dir = self.codex_dir.clone();
        tokio::task::spawn_blocking(move || {
            super::codex_auth_service::CodexAuthService::from_dirs_with_env_lock(
                ccr_codex_dir,
                codex_dir,
            )
            .prepare_current_quota_locked(&identity)
        })
        .await
        .map_err(|_| "配额凭据准备任务失败".to_string())?
        .map_err(|error| error.to_string())
    }

    fn verify_locked_snapshot(
        initial: &OpenAiQuotaSnapshot,
        current: &OpenAiQuotaSnapshot,
        path: &Path,
    ) -> std::result::Result<(), String> {
        let same_resource =
            Self::snapshot_resource(initial, path) == Self::snapshot_resource(current, path);
        let known_identity = OAuthIdentity::from_tokens(&CodexAuthTokens {
            id_token: initial.id_token.clone(),
            access_token: Some(initial.access_token.clone()),
            refresh_token: initial.refresh_token.clone(),
            account_id: initial.account_id.clone(),
        })
        .is_some();
        if !same_resource
            || (!known_identity
                && (initial.id_token != current.id_token
                    || initial.access_token != current.access_token
                    || initial.refresh_token != current.refresh_token
                    || initial.account_id != current.account_id))
        {
            return Err("auth 来源身份已变化，跳过查询".into());
        }
        Ok(())
    }

    /// 账号为 current_auth 且 runtime 属于该账号时返回 true；判断前先做一次观测点同步
    async fn route_active_account_to_runtime(&self, account_name: &str) -> bool {
        let ccr_codex_dir = self.ccr_codex_dir.clone();
        let codex_dir = self.codex_dir.clone();
        let account_name = account_name.to_string();
        tokio::task::spawn_blocking(move || {
            let registry = super::codex_registry_store::CodexRegistryStore::new(&ccr_codex_dir)
                .load()
                .ok()?;
            if registry.current_auth.as_deref() != Some(account_name.as_str()) {
                return None;
            }
            let oauth = CodexOAuthTokenService::from_dirs(ccr_codex_dir.clone(), codex_dir.clone());
            let saved = oauth.saved_identity(&registry, &account_name)?;
            if oauth.runtime_identity().as_ref() != Some(&saved) {
                return None;
            }
            super::codex_auth_service::CodexAuthService::from_dirs_with_env_lock(
                ccr_codex_dir.clone(),
                codex_dir.clone(),
            )
            .sync_runtime_with_saved_account_locked()
            .ok()?;
            // Reload after synchronization and legacy identity backfill.
            let registry = super::codex_registry_store::CodexRegistryStore::new(&ccr_codex_dir)
                .load()
                .ok()?;
            if registry.current_auth.as_deref() != Some(account_name.as_str()) {
                return None;
            }
            let oauth = CodexOAuthTokenService::from_dirs(ccr_codex_dir, codex_dir);
            let saved = oauth.saved_identity(&registry, &account_name)?;
            (oauth.runtime_identity().as_ref() == Some(&saved)).then_some(())
        })
        .await
        .ok()
        .flatten()
        .is_some()
    }

    async fn sync_runtime_snapshot(&self, context: &'static str) {
        let ccr_codex_dir = self.ccr_codex_dir.clone();
        let codex_dir = self.codex_dir.clone();
        let _ = tokio::task::spawn_blocking(move || {
            super::codex_auth_service::CodexAuthService::from_dirs_with_env_lock(
                ccr_codex_dir,
                codex_dir,
            )
            .sync_runtime_with_saved_account_locked()
            .unwrap_or_else(|error| {
                warn!("Codex auth snapshot sync failed ({}): {}", context, error);
                super::codex_oauth_token_service::RuntimeSyncOutcome::NoOp
            })
        })
        .await;
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
            id_token: tokens.id_token,
            access_token: tokens.access_token.unwrap_or_default(),
            refresh_token: tokens.refresh_token,
            account_id: tokens.account_id,
            email: None,
        })
    }

    async fn update_auth_file(
        auth_path: &Path,
        expected: &OpenAiQuotaSnapshot,
        new_tokens: &TokenRefreshResponse,
    ) -> std::result::Result<(), String> {
        let original_json = fs::read_to_string(auth_path)
            .await
            .map_err(|error| format!("读取 auth 文件失败: {error}"))?;
        let mut value: serde_json::Value = serde_json::from_str(&original_json)
            .map_err(|error| format!("解析 auth JSON 失败: {error}"))?;
        let original: CodexAuthJson = serde_json::from_value(value.clone())
            .map_err(|_| "auth 身份无法解析，跳过写回".to_string())?;
        let expected_tokens = CodexAuthTokens {
            id_token: expected.id_token.clone(),
            access_token: Some(expected.access_token.clone()),
            refresh_token: expected.refresh_token.clone(),
            account_id: expected.account_id.clone(),
        };
        let matches_expected = original.tokens.as_ref().is_some_and(|tokens| {
            tokens.id_token == expected_tokens.id_token
                && tokens.access_token == expected_tokens.access_token
                && tokens.refresh_token == expected_tokens.refresh_token
                && tokens.account_id == expected_tokens.account_id
        });
        if !matches_expected {
            return Err("auth 凭据已变化，跳过写回".into());
        }
        let original_identity = identity_from_auth(&original);
        let expected_identity = OAuthIdentity::from_tokens(&expected_tokens);
        if expected_identity.is_some() && original_identity != expected_identity {
            return Err("auth OAuth 身份已变化，跳过写回".into());
        }

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

        let updated: CodexAuthJson = serde_json::from_value(value.clone())
            .map_err(|_| "刷新后的 auth 身份无法解析，跳过写回".to_string())?;
        if original_identity.is_some() && identity_from_auth(&updated) != original_identity {
            return Err("刷新后的 OAuth 身份不匹配，跳过写回".into());
        }

        value["last_refresh"] = serde_json::Value::String(Utc::now().to_rfc3339());

        let content = serde_json::to_string_pretty(&value)
            .map_err(|error| format!("序列化 auth 文件失败: {error}"))?;
        let expected_version = content_version_token(original_json.as_bytes());
        let path = auth_path.to_path_buf();
        let observed_version = expected_version.clone();
        let hardened = tokio::task::spawn_blocking(move || {
            enforce_owner_only_permissions_versioned(
                &path,
                &observed_version,
                std::time::Duration::from_secs(10),
            )
        })
        .await
        .map_err(|error| format!("auth 权限加固任务失败: {error}"))?
        .map_err(|error| format!("auth 权限加固失败: {error}"))?;
        if !hardened {
            return Err("auth 文件已变化，跳过写回".into());
        }
        let outcome = write_guarded_versioned_async(
            auth_path,
            content.into_bytes(),
            expected_version,
            WriteOptions {
                secret: true,
                ..Default::default()
            },
        )
        .await
        .map_err(|error| format!("写回 auth 文件失败: {error}"))?;
        if outcome == VersionedWriteOutcome::Conflict {
            return Err("auth 文件已变化，跳过写回".into());
        }
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
    fn update_auth_file_rejects_refreshed_tokens_for_another_user() {
        let _env = crate::test_support::TestCodexEnv::new();
        let (_service, _ccr, codex) = create_test_service();
        let auth_path = codex.path().join("auth.json");
        let token = |user: &str| {
            fake_jwt(json!({
                "chatgpt_user_id": user,
                "chatgpt_account_id": "write-workspace",
                "exp": (Utc::now() + Duration::hours(1)).timestamp()
            }))
        };
        write_auth_file(&auth_path, &token("write-user-b"), "refresh-b");
        let before = std::fs::read(&auth_path).unwrap();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let expected = runtime
            .block_on(CodexQuotaService::load_snapshot_from_path(&auth_path))
            .unwrap();
        let error = runtime.block_on(CodexQuotaService::update_auth_file(
            &auth_path,
            &expected,
            &TokenRefreshResponse {
                access_token: token("write-user-a"),
                id_token: Some(token("write-user-a")),
                refresh_token: Some("refresh-a-rotated".into()),
            },
        ));
        assert!(error.is_err());
        assert_eq!(std::fs::read(&auth_path).unwrap(), before);
    }

    #[test]
    fn quota_refresh_write_rejects_runtime_changed_during_request() {
        let _env = crate::test_support::TestCodexEnv::new();
        let (_service, _ccr, codex) = create_test_service();
        let auth_path = codex.path().join("auth.json");
        let token = |user: &str| {
            fake_jwt(json!({"chatgpt_user_id":user,"chatgpt_account_id":"write-workspace"}))
        };
        let runtime = tokio::runtime::Runtime::new().unwrap();
        for (user, refresh) in [
            ("write-user-b", "refresh-b"),
            ("write-user-a", "refresh-a-newer"),
        ] {
            write_auth_file(&auth_path, &token("write-user-a"), "refresh-a");
            let expected = runtime
                .block_on(CodexQuotaService::load_snapshot_from_path(&auth_path))
                .unwrap();
            write_auth_file(&auth_path, &token(user), refresh);
            let before = std::fs::read(&auth_path).unwrap();
            let error = runtime
                .block_on(CodexQuotaService::update_auth_file(
                    &auth_path,
                    &expected,
                    &TokenRefreshResponse {
                        access_token: token("write-user-a"),
                        id_token: Some(token("write-user-a")),
                        refresh_token: Some("refresh-a-rotated".into()),
                    },
                ))
                .unwrap_err();
            assert!(
                !error.contains("write-user")
                    && !error.contains("::")
                    && !error.contains("refresh-a")
            );
            assert_eq!(std::fs::read(&auth_path).unwrap(), before);
        }
    }

    #[test]
    fn update_auth_file_rewrites_access_and_refresh_tokens() {
        let _env = crate::test_support::TestCodexEnv::new();
        let (_service, _ccr, codex) = create_test_service();
        let auth_path = codex.path().join("auth.json");
        let token = fake_jwt(json!({
            "email": "user@example.com",
            "chatgpt_account_id": "acc-1",
            "exp": (Utc::now() + Duration::hours(1)).timestamp()
        }));
        write_auth_file(&auth_path, &token, "refresh-token");

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let expected = runtime
            .block_on(CodexQuotaService::load_snapshot_from_path(&auth_path))
            .unwrap();
        runtime
            .block_on(CodexQuotaService::update_auth_file(
                &auth_path,
                &expected,
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
    // ==================== 本地 OAuth/usage stub 端到端（10-06 切换可靠性） ====================

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StubTokenState {
        Valid,
        Consumed,
        Revoked,
    }

    #[derive(Default)]
    struct StubState {
        refresh_tokens: std::collections::HashMap<String, (String, StubTokenState)>,
        access_tokens: std::collections::HashSet<String>,
        issued: usize,
        replace_auth_on_refresh: Option<(PathBuf, String)>,
        refresh_calls: usize,
        refresh_gate: Option<(std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>)>,
        refresh_calls_by_token: std::collections::HashMap<String, usize>,
        usage_gate: Option<(std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>)>,
        reject_first_rotated_access: bool,
        usage_calls: usize,
    }

    impl StubState {
        fn add(&mut self, refresh: &str, account_id: &str) {
            self.refresh_tokens.insert(
                refresh.to_string(),
                (account_id.to_string(), StubTokenState::Valid),
            );
        }

        fn set(&mut self, refresh: &str, state: StubTokenState) {
            if let Some(entry) = self.refresh_tokens.get_mut(refresh) {
                entry.1 = state;
            }
        }

        /// 模拟 OAuth 轮换：旧 refresh_token 标记为已使用，签发新 token 对
        fn rotate(&mut self, refresh: &str) -> std::result::Result<(String, String), String> {
            let (account_id, state) = self
                .refresh_tokens
                .get(refresh)
                .cloned()
                .ok_or_else(|| "refresh_token_invalidated".to_string())?;
            match state {
                StubTokenState::Valid => {}
                StubTokenState::Consumed => return Err("refresh_token_reused".to_string()),
                StubTokenState::Revoked => return Err("refresh_token_invalidated".to_string()),
            }
            self.set(refresh, StubTokenState::Consumed);
            self.issued += 1;
            let new_refresh = format!("{refresh}-r{}", self.issued);
            self.add(&new_refresh, &account_id);
            let access = live_access_token(&account_id, self.issued);
            if !self.reject_first_rotated_access || self.issued != 1 {
                self.access_tokens.insert(access.clone());
            }
            if let Some((path, content)) = self.replace_auth_on_refresh.take() {
                std::fs::write(path, content).unwrap();
            }
            Ok((access, new_refresh))
        }
    }

    struct OAuthStub {
        state: std::sync::Arc<std::sync::Mutex<StubState>>,
        endpoints: crate::services::openai_quota_core::TestEndpoints,
    }

    fn live_access_token(account_id: &str, nonce: usize) -> String {
        fake_jwt(json!({
            "exp": Utc::now().timestamp() + 3600,
            "chatgpt_account_id": account_id,
            "chatgpt_user_id": format!("user-{account_id}"),
            "nonce": nonce
        }))
    }

    fn expired_access_token(account_id: &str) -> String {
        fake_jwt(json!({
            "exp": Utc::now().timestamp() - 3600,
            "chatgpt_account_id": account_id,
            "chatgpt_user_id": format!("user-{account_id}")
        }))
    }

    fn header_value(head: &str, name: &str) -> Option<String> {
        head.lines().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.trim()
                .eq_ignore_ascii_case(name)
                .then(|| value.trim().to_string())
        })
    }

    fn stub_response(
        state: &std::sync::Mutex<StubState>,
        head: &str,
        body: &str,
    ) -> (&'static str, serde_json::Value) {
        let path = head.split_whitespace().nth(1).unwrap_or("");
        if path == "/oauth/token" {
            let request: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let refresh = request["refresh_token"].as_str().unwrap_or("");
            let (result, gate) = {
                let mut state = state.lock().unwrap();
                state.refresh_calls += 1;
                *state
                    .refresh_calls_by_token
                    .entry(refresh.to_string())
                    .or_default() += 1;
                (state.rotate(refresh), state.refresh_gate.take())
            };
            if let Some((entered, release)) = gate {
                entered.send(()).unwrap();
                release
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .unwrap();
            }
            return match result {
                Ok((access, new_refresh)) => (
                    "200 OK",
                    json!({
                        "access_token": access,
                        "id_token": access,
                        "refresh_token": new_refresh
                    }),
                ),
                Err(code) => ("401 Unauthorized", json!({"error": {"code": code}})),
            };
        }
        if path == "/usage" {
            let gate = {
                let mut state = state.lock().unwrap();
                state.usage_calls += 1;
                state.usage_gate.take()
            };
            if let Some((entered, release)) = gate {
                entered.send(()).unwrap();
                release
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .unwrap();
            }
            let bearer = header_value(head, "authorization").unwrap_or_default();
            let bearer = bearer.trim_start_matches("Bearer ");
            if state.lock().unwrap().access_tokens.contains(bearer) {
                return (
                    "200 OK",
                    json!({
                        "plan_type": "plus",
                        "rate_limit": {
                            "primary_window": {
                                "used_percent": 10.0,
                                "limit_window_seconds": 18000,
                                "reset_after_seconds": 600
                            }
                        }
                    }),
                );
            }
            return (
                "401 Unauthorized",
                json!({"error": {"code": "token_expired"}}),
            );
        }
        ("404 Not Found", json!({}))
    }

    fn start_oauth_stub() -> OAuthStub {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let state = std::sync::Arc::new(std::sync::Mutex::new(StubState::default()));
        let server_state = state.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut buffer = Vec::new();
                let mut chunk = [0_u8; 4096];
                let header_end = loop {
                    let read = stream.read(&mut chunk).unwrap_or(0);
                    if read == 0 {
                        break None;
                    }
                    buffer.extend_from_slice(&chunk[..read]);
                    if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                        break Some(pos + 4);
                    }
                };
                let Some(header_end) = header_end else {
                    continue;
                };
                let head = String::from_utf8_lossy(&buffer[..header_end]).to_string();
                let content_length = header_value(&head, "content-length")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(0);
                while buffer.len() < header_end + content_length {
                    let read = stream.read(&mut chunk).unwrap_or(0);
                    if read == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&chunk[..read]);
                }
                let body = String::from_utf8_lossy(&buffer[header_end..]).to_string();
                let (status, payload) = stub_response(&server_state, &head, &body);
                let payload = payload.to_string();
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                    payload.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        OAuthStub {
            state,
            endpoints: crate::services::openai_quota_core::TestEndpoints {
                usage: format!("{base}/usage"),
                token: format!("{base}/oauth/token"),
            },
        }
    }

    struct E2eFixture {
        env: crate::test_support::TestCodexEnv,
        auth: crate::services::codex_auth_service::CodexAuthService,
        quota: CodexQuotaService,
        stub: OAuthStub,
    }

    impl E2eFixture {
        fn new() -> Self {
            let env = crate::test_support::TestCodexEnv::new();
            std::fs::write(
                env.codex_dir().join("config.toml"),
                "cli_auth_credentials_store = \"file\"\n",
            )
            .unwrap();
            let auth = crate::services::codex_auth_service::CodexAuthService::from_dirs(
                env.ccr_codex_dir().to_path_buf(),
                env.codex_dir().to_path_buf(),
            );
            let quota = CodexQuotaService {
                ccr_codex_dir: env.ccr_codex_dir().to_path_buf(),
                codex_dir: env.codex_dir().to_path_buf(),
            };
            Self {
                env,
                auth,
                quota,
                stub: start_oauth_stub(),
            }
        }

        fn runtime_path(&self) -> PathBuf {
            self.env.codex_dir().join("auth.json")
        }

        fn snapshot_path(&self, name: &str) -> PathBuf {
            self.quota.account_auth_path(name)
        }

        /// 以过期 access_token 写入 runtime，迫使配额查询走 refresh
        fn write_runtime(&self, account_id: &str, refresh: &str, last_refresh: &str) {
            let auth = json!({
                "auth_mode": "chatgpt",
                "OPENAI_API_KEY": null,
                "tokens": {
                    "id_token": expired_access_token(account_id),
                    "access_token": expired_access_token(account_id),
                    "refresh_token": refresh,
                    "account_id": account_id
                },
                "last_refresh": last_refresh
            });
            std::fs::write(self.runtime_path(), auth.to_string()).unwrap();
        }

        fn login_and_save(&self, name: &str, account_id: &str, refresh: &str) {
            self.stub.state.lock().unwrap().add(refresh, account_id);
            self.write_runtime(account_id, refresh, "2026-10-01T00:00:00Z");
            self.auth.save_current(name, None, false).unwrap();
        }

        /// 模拟 codex 自身轮换 refresh_token 并写回 runtime
        fn codex_rotates_runtime(&self, account_id: &str, refresh: &str) -> String {
            let (_, new_refresh) = self.stub.state.lock().unwrap().rotate(refresh).unwrap();
            self.write_runtime(account_id, &new_refresh, "2026-10-02T00:00:00Z");
            new_refresh
        }

        fn fetch_quota(&self, name: &str) -> CodexAccountQuota {
            tokio::runtime::Runtime::new().unwrap().block_on(
                crate::services::openai_quota_core::TEST_ENDPOINTS.scope(
                    self.stub.endpoints.clone(),
                    self.quota.fetch_account_quota(name),
                ),
            )
        }
    }

    fn unique_account_id(label: &str) -> String {
        format!("acc-{label}-{}", uuid::Uuid::new_v4())
    }

    fn file_refresh_token(path: &Path) -> String {
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        value["tokens"]["refresh_token"]
            .as_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn lifecycle_concurrent_queries_reuse_one_rotation_for_source_and_aliases() {
        for aliases in [false, true] {
            let fixture = E2eFixture::new();
            let account_id = unique_account_id("concurrent-refresh");
            fixture.login_and_save("a", &account_id, "rt-concurrent");
            if aliases {
                fixture.auth.save_current("alias", None, false).unwrap();
                fixture.write_runtime(
                    &unique_account_id("other"),
                    "rt-other",
                    "2026-10-01T00:00:00Z",
                );
            }
            let (entered_tx, entered_rx) = std::sync::mpsc::channel();
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            fixture.stub.state.lock().unwrap().refresh_gate = Some((entered_tx, release_rx));
            let controller = std::thread::spawn(move || {
                entered_rx
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .unwrap();
                std::thread::sleep(std::time::Duration::from_millis(100));
                release_tx.send(()).unwrap();
            });
            let results = tokio::runtime::Runtime::new().unwrap().block_on(
                crate::services::openai_quota_core::TEST_ENDPOINTS.scope(
                    fixture.stub.endpoints.clone(),
                    async {
                        if aliases {
                            tokio::join!(
                                fixture.quota.fetch_account_quota_force_refresh("a"),
                                fixture.quota.fetch_account_quota_force_refresh("alias")
                            )
                        } else {
                            tokio::join!(
                                fixture.quota.fetch_current_quota_force_refresh(),
                                fixture.quota.fetch_current_quota_force_refresh()
                            )
                        }
                    },
                ),
            );
            controller.join().unwrap();
            assert!(results.0.quota.is_some() && results.1.quota.is_some());
            assert_eq!(fixture.stub.state.lock().unwrap().refresh_calls, 1);
            let source = if aliases {
                fixture.snapshot_path("a")
            } else {
                fixture.runtime_path()
            };
            assert_eq!(file_refresh_token(&source), "rt-concurrent-r1");
            if aliases {
                assert_eq!(
                    file_refresh_token(&fixture.snapshot_path("alias")),
                    "rt-concurrent-r1"
                );
            }
        }
    }

    #[test]
    fn lifecycle_switch_waits_for_outgoing_refresh_and_preserves_new_tokens() {
        let fixture = E2eFixture::new();
        let account_a = unique_account_id("switch-refresh-a");
        fixture.login_and_save("a", &account_a, "rt-switch-a");
        fixture.login_and_save("b", &unique_account_id("switch-refresh-b"), "rt-switch-b");
        fixture.auth.switch_account("a").unwrap();
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        fixture.stub.state.lock().unwrap().refresh_gate = Some((entered_tx, release_rx));
        let ccr_dir = fixture.env.ccr_codex_dir().to_path_buf();
        let codex_dir = fixture.env.codex_dir().to_path_buf();
        let switch = std::thread::spawn(move || {
            entered_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            crate::services::CodexAuthService::from_dirs_with_env_lock(ccr_dir, codex_dir)
                .switch_account("b")
        });
        let release = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(150));
            release_tx.send(()).unwrap();
        });
        let result = fixture.fetch_quota("a");
        release.join().unwrap();
        switch.join().unwrap().unwrap();
        assert!(result.quota.is_some(), "refresh result: {:?}", result.error);
        assert_eq!(
            file_refresh_token(&fixture.snapshot_path("a")),
            "rt-switch-a-r1"
        );
        assert_eq!(file_refresh_token(&fixture.runtime_path()), "rt-switch-b");
    }

    #[test]
    fn lifecycle_waiting_query_rejects_replacement_identity_before_http() {
        let fixture = E2eFixture::new();
        let account_id = unique_account_id("waiting-query");
        fixture.login_and_save("a", &account_id, "rt-waiting");
        let path = fixture.snapshot_path("a");
        let initial = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(CodexQuotaService::load_snapshot_from_path(&path))
            .unwrap();
        let held = CredentialLocks::acquire(vec![CredentialResource::from_path(&path)]).unwrap();
        let quota = CodexQuotaService {
            ccr_codex_dir: fixture.quota.ccr_codex_dir.clone(),
            codex_dir: fixture.quota.codex_dir.clone(),
        };
        let endpoints = fixture.stub.endpoints.clone();
        let waiter = std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(
                crate::services::openai_quota_core::TEST_ENDPOINTS.scope(
                    endpoints,
                    quota.fetch_account_quota_locked("a", &initial, true),
                ),
            )
        });
        let replacement = fake_jwt(
            json!({"chatgpt_user_id":"replacement-user", "chatgpt_account_id":account_id, "exp":Utc::now().timestamp()-3600}),
        );
        std::fs::write(
            &path,
            json!({"tokens":{"id_token":replacement, "access_token":replacement,
            "refresh_token":"replacement-refresh", "account_id":account_id}})
            .to_string(),
        )
        .unwrap();
        let replacement_bytes = std::fs::read(&path).unwrap();
        drop(held);
        let error = waiter.join().unwrap().unwrap_err();
        assert!(error.contains("auth 来源身份已变化"));
        assert_eq!(fixture.stub.state.lock().unwrap().refresh_calls, 0);
        assert_eq!(std::fs::read(&path).unwrap(), replacement_bytes);
    }

    #[test]
    fn lifecycle_shared_workspace_users_have_independent_query_locks() {
        let fixture = E2eFixture::new();
        let account_id = unique_account_id("independent-users");
        for (name, user) in [("a", "independent-user-a"), ("b", "independent-user-b")] {
            let access = fake_jwt(
                json!({"chatgpt_user_id":user, "chatgpt_account_id":account_id,
                "exp":Utc::now().timestamp()+3600}),
            );
            std::fs::write(
                fixture.runtime_path(),
                json!({"tokens":{
                "id_token":access,"access_token":access,"refresh_token":format!("refresh-{name}"),
                "account_id":account_id},"last_refresh":"2026-10-01T00:00:00Z"})
                .to_string(),
            )
            .unwrap();
            fixture
                .stub
                .state
                .lock()
                .unwrap()
                .access_tokens
                .insert(access);
            fixture.auth.save_current(name, None, false).unwrap();
        }
        let before_a = std::fs::read(fixture.snapshot_path("a")).unwrap();
        let before_b = std::fs::read(fixture.snapshot_path("b")).unwrap();
        let held_b = CredentialLocks::acquire(vec![CredentialResource::from_path(
            &fixture.snapshot_path("b"),
        )])
        .unwrap();
        let result_a = tokio::runtime::Runtime::new().unwrap().block_on(
            crate::services::openai_quota_core::TEST_ENDPOINTS.scope(
                fixture.stub.endpoints.clone(),
                async {
                    tokio::time::timeout(
                        std::time::Duration::from_millis(500),
                        fixture.quota.fetch_account_quota_force_refresh("a"),
                    )
                    .await
                },
            ),
        );
        drop(held_b);
        assert!(result_a.unwrap().quota.is_some());
        assert!(fixture.fetch_quota("b").quota.is_some());
        assert_eq!(fixture.stub.state.lock().unwrap().refresh_calls, 0);
        assert_eq!(std::fs::read(fixture.snapshot_path("a")).unwrap(), before_a);
        assert_eq!(std::fs::read(fixture.snapshot_path("b")).unwrap(), before_b);
    }

    #[test]
    fn lifecycle_same_second_alias_rotation_never_restores_consumed_runtime_token() {
        use chrono::Timelike;
        let fixture = E2eFixture::new();
        let account_id = unique_account_id("same-second");
        fixture.login_and_save("a", &account_id, "rt-same-second");
        fixture.auth.save_current("alias", None, false).unwrap();
        let old_time = Utc::now().with_nanosecond(100_000_000).unwrap();
        let new_time = old_time.with_nanosecond(900_000_000).unwrap();
        fixture.write_runtime(&account_id, "rt-same-second", &old_time.to_rfc3339());
        std::fs::write(
            fixture.snapshot_path("a"),
            std::fs::read(fixture.runtime_path()).unwrap(),
        )
        .unwrap();
        let (new_access, new_refresh) = fixture
            .stub
            .state
            .lock()
            .unwrap()
            .rotate("rt-same-second")
            .unwrap();
        std::fs::write(
            fixture.snapshot_path("alias"),
            json!({"tokens":{
            "id_token":new_access,"access_token":new_access,"refresh_token":new_refresh,
            "account_id":account_id},"last_refresh":new_time.to_rfc3339()})
            .to_string(),
        )
        .unwrap();
        let store = super::super::codex_registry_store::CodexRegistryStore::new(
            fixture.env.ccr_codex_dir(),
        );
        let mut registry = store.load().unwrap();
        registry.current_auth = Some("a".into());
        store.save(&registry).unwrap();
        assert!(fixture.fetch_quota("alias").quota.is_some());
        let repaired: CodexAuthJson =
            serde_json::from_slice(&std::fs::read(fixture.snapshot_path("a")).unwrap()).unwrap();
        assert_eq!(
            chrono::DateTime::parse_from_rfc3339(repaired.last_refresh.as_deref().unwrap())
                .unwrap(),
            new_time
        );
        let result = fixture.fetch_quota("a");
        assert!(result.quota.is_some(), "quota: {:?}", result.error);
        assert_eq!(fixture.stub.state.lock().unwrap().refresh_calls, 0);
        assert_eq!(file_refresh_token(&fixture.runtime_path()), new_refresh);
        assert_eq!(file_refresh_token(&fixture.snapshot_path("a")), new_refresh);
        let snapshot: CodexAuthJson =
            serde_json::from_slice(&std::fs::read(fixture.snapshot_path("a")).unwrap()).unwrap();
        assert_eq!(
            chrono::DateTime::parse_from_rfc3339(snapshot.last_refresh.as_deref().unwrap())
                .unwrap(),
            new_time
        );
    }

    #[test]
    fn lifecycle_unknown_identity_upgrade_and_known_alias_never_repeat_new_refresh() {
        let fixture = E2eFixture::new();
        let account_id = unique_account_id("identity-upgrade");
        fixture.login_and_save("alias", &account_id, "rt-upgrade");
        let store = super::super::codex_registry_store::CodexRegistryStore::new(
            fixture.env.ccr_codex_dir(),
        );
        let mut registry = store.load().unwrap();
        registry.current_auth = None;
        store.save(&registry).unwrap();
        let unknown =
            fake_jwt(json!({"exp":Utc::now().timestamp()-3600, "chatgpt_account_id":account_id}));
        std::fs::write(
            fixture.runtime_path(),
            json!({"tokens":{
            "id_token":unknown,"access_token":unknown,"refresh_token":"rt-upgrade",
            "account_id":account_id},"last_refresh":"2026-10-01T00:00:00Z"})
            .to_string(),
        )
        .unwrap();
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        {
            let mut state = fixture.stub.state.lock().unwrap();
            state.reject_first_rotated_access = true;
            state.usage_gate = Some((entered_tx, release_rx));
        }
        let (begin_tx, begin_rx) = tokio::sync::oneshot::channel();
        let controller = std::thread::spawn(move || {
            entered_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            begin_tx.send(()).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(100));
            release_tx.send(()).unwrap();
        });
        let (current, alias) = tokio::runtime::Runtime::new().unwrap().block_on(
            crate::services::openai_quota_core::TEST_ENDPOINTS.scope(
                fixture.stub.endpoints.clone(),
                async {
                    tokio::join!(fixture.quota.fetch_current_quota_force_refresh(), async {
                        begin_rx.await.unwrap();
                        fixture
                            .quota
                            .fetch_account_quota_force_refresh("alias")
                            .await
                    })
                },
            ),
        );
        controller.join().unwrap();
        let state = fixture.stub.state.lock().unwrap();
        assert_eq!(state.refresh_calls_by_token.get("rt-upgrade"), Some(&1));
        assert_eq!(state.refresh_calls_by_token.get("rt-upgrade-r1"), Some(&1));
        assert!(current.quota.is_some(), "current: {:?}", current.error);
        assert!(alias.quota.is_some(), "alias: {:?}", alias.error);
        assert_eq!(
            file_refresh_token(&fixture.runtime_path()),
            "rt-upgrade-r1-r2"
        );
        assert_eq!(
            file_refresh_token(&fixture.snapshot_path("alias")),
            "rt-upgrade-r1-r2"
        );
    }

    #[test]
    fn lifecycle_identity_upgrade_retry_rejects_a_replacement_source() {
        let fixture = E2eFixture::new();
        let account_id = unique_account_id("upgrade-replacement");
        fixture
            .stub
            .state
            .lock()
            .unwrap()
            .add("rt-upgrade-replacement", &account_id);
        let unknown =
            fake_jwt(json!({"exp":Utc::now().timestamp()-3600, "chatgpt_account_id":account_id}));
        let path = fixture.runtime_path();
        std::fs::write(
            &path,
            json!({"tokens":{"id_token":unknown,"access_token":unknown,
            "refresh_token":"rt-upgrade-replacement","account_id":account_id}})
            .to_string(),
        )
        .unwrap();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let expected = runtime.block_on(crate::services::openai_quota_core::TEST_ENDPOINTS.scope(
            fixture.stub.endpoints.clone(),
            async {
                let initial = CodexQuotaService::load_snapshot_from_path(&path)
                    .await
                    .unwrap();
                let held = CredentialLocks::acquire_async_sources(vec![(
                    path.clone(),
                    CodexQuotaService::snapshot_resource(&initial, &path),
                )])
                .await
                .unwrap();
                let upgraded =
                    match CodexQuotaService::fetch_source_quota(initial, true, &path).await {
                        Err(SourceQuotaError::IdentityUpgrade(upgraded)) => upgraded,
                        _ => panic!("refresh must stop at the identity upgrade"),
                    };
                drop(held);
                upgraded
            },
        ));
        assert_eq!(fixture.stub.state.lock().unwrap().usage_calls, 0);
        let replacement = fake_jwt(
            json!({"chatgpt_user_id":"replacement-user", "chatgpt_account_id":account_id, "exp":Utc::now().timestamp()+3600}),
        );
        std::fs::write(
            &path,
            json!({"tokens":{"id_token":replacement,"access_token":replacement,
            "refresh_token":"replacement-refresh","account_id":account_id}})
            .to_string(),
        )
        .unwrap();
        let replacement_bytes = std::fs::read(&path).unwrap();
        let error = runtime
            .block_on(crate::services::openai_quota_core::TEST_ENDPOINTS.scope(
                fixture.stub.endpoints.clone(),
                fixture.quota.fetch_current_quota_locked(&expected, true),
            ))
            .unwrap_err();
        assert!(error.contains("auth 来源身份已变化"));
        let state = fixture.stub.state.lock().unwrap();
        assert_eq!(state.refresh_calls, 1);
        assert_eq!(state.usage_calls, 0);
        assert_eq!(std::fs::read(&path).unwrap(), replacement_bytes);
    }

    #[test]
    fn quota_refresh_request_cannot_overwrite_a_replacement_user() {
        for target in ["active", "saved", "default"] {
            let fixture = E2eFixture::new();
            let account_id = unique_account_id("request-race");
            fixture.login_and_save("a", &account_id, "refresh-a");
            let snapshot_before = std::fs::read(fixture.snapshot_path("a")).unwrap();
            if target == "saved" {
                let store = super::super::codex_registry_store::CodexRegistryStore::new(
                    fixture.env.ccr_codex_dir(),
                );
                let mut registry = store.load().unwrap();
                registry.current_auth = None;
                store.save(&registry).unwrap();
            }
            let path = if target == "saved" {
                fixture.snapshot_path("a")
            } else {
                fixture.runtime_path()
            };
            let token = fake_jwt(
                json!({"chatgpt_user_id":"replacement-user","chatgpt_account_id":account_id,"exp":Utc::now().timestamp()+3600}),
            );
            let replacement = json!({"tokens":{"id_token":token,"access_token":token,"refresh_token":"refresh-b","account_id":account_id},"last_refresh":"2026-10-03T00:00:00Z"}).to_string();
            fixture.stub.state.lock().unwrap().replace_auth_on_refresh =
                Some((path.clone(), replacement.clone()));
            let quota = if target == "default" {
                tokio::runtime::Runtime::new().unwrap().block_on(
                    crate::services::openai_quota_core::TEST_ENDPOINTS.scope(
                        fixture.stub.endpoints.clone(),
                        fixture.quota.fetch_current_quota(),
                    ),
                )
            } else {
                fixture.fetch_quota("a")
            };
            assert!(
                quota
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("已变化"))
            );
            assert!(quota.quota.is_none());
            assert_eq!(std::fs::read_to_string(path).unwrap(), replacement);
            if target != "saved" {
                assert_eq!(
                    std::fs::read(fixture.snapshot_path("a")).unwrap(),
                    snapshot_before
                );
            }
        }
    }

    #[test]
    fn runtime_quota_route_requires_the_current_complete_user_identity() {
        let fixture = E2eFixture::new();
        let document = |user: &str, refresh: &str, date: &str| {
            json!({
                "tokens":{"id_token":fake_jwt(json!({"chatgpt_user_id":user})),"access_token":fake_jwt(json!({"chatgpt_user_id":user,"chatgpt_account_id":"route-workspace","exp":Utc::now().timestamp()+3600})),"refresh_token":refresh,"account_id":"route-workspace"},
                "last_refresh":date
            })
        };
        for (name, user, refresh) in [
            ("a", "route-user-a", "rt-a1"),
            ("b", "route-user-b", "rt-b1"),
        ] {
            std::fs::write(
                fixture.runtime_path(),
                document(user, refresh, "2026-10-01T00:00:00Z").to_string(),
            )
            .unwrap();
            fixture.auth.save_current(name, None, false).unwrap();
        }
        let b_before = std::fs::read(fixture.snapshot_path("b")).unwrap();
        let a_before = std::fs::read(fixture.snapshot_path("a")).unwrap();
        std::fs::write(
            fixture.runtime_path(),
            document("route-user-a", "rt-a2", "2026-10-02T00:00:00Z").to_string(),
        )
        .unwrap();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        assert!(!runtime.block_on(fixture.quota.route_active_account_to_runtime("b")));
        assert_eq!(std::fs::read(fixture.snapshot_path("b")).unwrap(), b_before);
        assert_eq!(std::fs::read(fixture.snapshot_path("a")).unwrap(), a_before);
        assert_eq!(
            fixture.auth.sync_runtime_with_saved_account().unwrap(),
            crate::services::RuntimeSyncOutcome::SnapshotUpdated("a".into())
        );
        assert_eq!(file_refresh_token(&fixture.snapshot_path("a")), "rt-a2");
        assert_eq!(
            fixture
                .auth
                .sync_current_auth_registry()
                .unwrap()
                .as_deref(),
            Some("a")
        );
        assert!(runtime.block_on(fixture.quota.route_active_account_to_runtime("a")));
        let mut unknown = document("route-user-a", "rt-a2", "2026-10-02T00:00:00Z");
        unknown["tokens"]["id_token"] = serde_json::Value::Null;
        unknown["tokens"]["access_token"] = "opaque-access".into();
        std::fs::write(fixture.snapshot_path("a"), unknown.to_string()).unwrap();
        assert!(!runtime.block_on(fixture.quota.route_active_account_to_runtime("a")));
    }

    #[test]
    fn e2e_rotation_observed_then_external_login_then_switch_back_succeeds() {
        let fixture = E2eFixture::new();
        let account_a = unique_account_id("a");
        let account_b = unique_account_id("b");
        fixture.login_and_save("a", &account_a, "rt-a1");
        let rotated = fixture.codex_rotates_runtime(&account_a, "rt-a1");

        // CCR 观察点（TUI 加载 / 切换前同步）
        assert_eq!(
            fixture.auth.sync_runtime_with_saved_account().unwrap(),
            crate::services::RuntimeSyncOutcome::SnapshotUpdated("a".into())
        );
        assert_eq!(file_refresh_token(&fixture.snapshot_path("a")), rotated);

        // 外部 codex login B 覆盖 runtime，随后切回 A
        fixture.stub.state.lock().unwrap().add("rt-b1", &account_b);
        fixture.write_runtime(&account_b, "rt-b1", "2026-10-03T00:00:00Z");
        fixture.auth.switch_account("a").unwrap();
        assert_eq!(file_refresh_token(&fixture.runtime_path()), rotated);

        let result = fixture.fetch_quota("a");

        assert!(result.error.is_none(), "{:?}", result.error);
        assert!(result.quota.is_some());
        // 活动账号经 runtime 刷新：runtime 与快照持有同一新 token，旧值不再出现
        let runtime_refresh = file_refresh_token(&fixture.runtime_path());
        assert_ne!(runtime_refresh, rotated);
        assert_eq!(
            file_refresh_token(&fixture.snapshot_path("a")),
            runtime_refresh
        );
        for stale in ["rt-a1", rotated.as_str()] {
            for path in [fixture.runtime_path(), fixture.snapshot_path("a")] {
                let text = std::fs::read_to_string(&path).unwrap();
                assert!(!text.contains(&format!("\"{stale}\"")));
            }
        }
    }

    #[test]
    fn e2e_rotation_without_observation_reports_relogin_and_keeps_snapshot() {
        let fixture = E2eFixture::new();
        let account_a = unique_account_id("a");
        let account_b = unique_account_id("b");
        fixture.login_and_save("a", &account_a, "rt-a1");
        let rotated = fixture.codex_rotates_runtime(&account_a, "rt-a1");
        // codex login B 在写入新凭据前吊销旧 refresh_token
        {
            let mut state = fixture.stub.state.lock().unwrap();
            state.set(&rotated, StubTokenState::Revoked);
            state.add("rt-b1", &account_b);
        }
        fixture.write_runtime(&account_b, "rt-b1", "2026-10-03T00:00:00Z");
        let snapshot_before = std::fs::read(fixture.snapshot_path("a")).unwrap();

        let result = fixture.fetch_quota("a");

        let error = result.error.expect("consumed snapshot token must fail");
        assert!(relogin_required_detail(&error).is_some(), "{error}");
        assert!(result.quota.is_none());
        assert_eq!(
            std::fs::read(fixture.snapshot_path("a")).unwrap(),
            snapshot_before
        );
        let registry = fixture.auth.load_registry().unwrap();
        assert!(registry.accounts.contains_key("a"));
        // runtime 中的 B 未被改写
        assert_eq!(file_refresh_token(&fixture.runtime_path()), "rt-b1");
    }

    #[test]
    fn e2e_invalidated_snapshot_token_is_repaired_from_newer_backup() {
        let fixture = E2eFixture::new();
        let account_a = unique_account_id("a");
        let account_b = unique_account_id("b");
        fixture.login_and_save("a", &account_a, "rt-a1");
        fixture.login_and_save("b", &account_b, "rt-b1");
        {
            let mut state = fixture.stub.state.lock().unwrap();
            state.set("rt-a1", StubTokenState::Revoked);
            state.add("rt-a9", &account_a);
        }
        let backups = fixture.env.codex_dir().join("backups");
        std::fs::create_dir_all(&backups).unwrap();
        std::fs::write(
            backups.join("auth.runtime_switch.20261005_000000.json.bak"),
            json!({
                "tokens": {
                    "access_token": expired_access_token(&account_a),
                    "refresh_token": "rt-a9",
                    "account_id": account_a
                },
                "last_refresh": "2026-10-05T00:00:00Z"
            })
            .to_string(),
        )
        .unwrap();

        let result = fixture.fetch_quota("a");

        assert!(result.error.is_none(), "{:?}", result.error);
        let snapshot_refresh = file_refresh_token(&fixture.snapshot_path("a"));
        assert!(
            snapshot_refresh.starts_with("rt-a9-r"),
            "{snapshot_refresh}"
        );
        assert_eq!(file_refresh_token(&fixture.runtime_path()), "rt-b1");
    }

    #[test]
    fn relogin_marker_wraps_only_permanent_refresh_errors_once() {
        let marked = mark_relogin_if_permanent(
            "Token 刷新失败 (401) [refresh_token_invalidated]: x".to_string(),
        );
        assert_eq!(
            relogin_required_detail(&marked),
            Some("Token 刷新失败 (401) [refresh_token_invalidated]: x")
        );
        assert_eq!(mark_relogin_if_permanent(marked.clone()), marked);
        assert_eq!(
            mark_relogin_if_permanent("配额请求失败: timeout".to_string()),
            "配额请求失败: timeout"
        );
    }
}
