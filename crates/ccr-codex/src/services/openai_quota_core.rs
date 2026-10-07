// 💰 OpenAI OAuth 配额共享核心
// 复用 wham/usage API 查询、JWT 解析与 token 刷新逻辑。

use super::codex_auth_identity::OAuthIdentity;
use crate::models::{CodexAuthTokens, CodexQuota};
use chrono::{DateTime, Utc};
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tracing::debug;

/// 全局复用的 HTTP 客户端（内部为 Arc，clone 开销极低）
static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(build_http_client);
/// 账号级 quota 共享缓存；用于跨 Codex/OpenCode 页签复用最近一次查询结果。
static QUOTA_CACHE: LazyLock<Mutex<HashMap<String, CachedQuotaEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// wham/usage API 端点
const USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

/// OAuth token 刷新端点
const TOKEN_REFRESH_URL: &str = "https://auth.openai.com/oauth/token";

/// OAuth client_id（Codex / OpenCode 共用 OpenAI ChatGPT OAuth）
const OAUTH_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
/// 共享 quota 缓存有效期。
const QUOTA_CACHE_TTL: Duration = Duration::from_secs(30);

fn build_http_client() -> reqwest::Client {
    if cfg!(test) {
        // 测试只访问本地 stub，避免系统代理拦截 loopback 请求
        reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_default()
    } else {
        reqwest::Client::new()
    }
}

/// 测试注入的本地 stub 端点（仅测试构建，按 tokio 任务作用域生效）
#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) struct TestEndpoints {
    pub(crate) usage: String,
    pub(crate) token: String,
}

#[cfg(test)]
tokio::task_local! {
    pub(crate) static TEST_ENDPOINTS: TestEndpoints;
}

fn usage_url() -> String {
    #[cfg(test)]
    if let Ok(url) = TEST_ENDPOINTS.try_with(|endpoints| endpoints.usage.clone()) {
        return url;
    }
    USAGE_URL.to_string()
}

fn token_refresh_url() -> String {
    #[cfg(test)]
    if let Ok(url) = TEST_ENDPOINTS.try_with(|endpoints| endpoints.token.clone()) {
        return url;
    }
    TOKEN_REFRESH_URL.to_string()
}

/// 使用率窗口（5小时/周）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowInfo {
    used_percent: Option<f64>,
    limit_window_seconds: Option<i64>,
    reset_after_seconds: Option<i64>,
    reset_at: Option<i64>,
}

/// 速率限制信息
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RateLimitInfo {
    #[allow(dead_code)]
    allowed: Option<bool>,
    #[allow(dead_code)]
    limit_reached: Option<bool>,
    primary_window: Option<WindowInfo>,
    secondary_window: Option<WindowInfo>,
}

/// wham/usage 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UsageResponse {
    plan_type: Option<String>,
    rate_limit: Option<RateLimitInfo>,
    #[allow(dead_code)]
    code_review_rate_limit: Option<RateLimitInfo>,
}

/// OAuth token 刷新请求
#[derive(Serialize)]
struct TokenRefreshRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
}

impl std::fmt::Debug for TokenRefreshRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenRefreshRequest")
            .field("grant_type", &self.grant_type)
            .field("refresh_token", &"[REDACTED]")
            .field("client_id", &self.client_id)
            .finish()
    }
}

/// OAuth token 刷新响应
#[derive(Clone, Deserialize)]
pub(crate) struct TokenRefreshResponse {
    pub(crate) access_token: String,
    #[serde(default)]
    pub(crate) id_token: Option<String>,
    #[serde(default)]
    pub(crate) refresh_token: Option<String>,
}

impl std::fmt::Debug for TokenRefreshResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TokenRefreshResponse([REDACTED])")
    }
}

/// 共享 quota 查询所需的最小快照。
#[derive(Clone)]
pub(crate) struct OpenAiQuotaSnapshot {
    pub(crate) id_token: Option<String>,
    pub(crate) access_token: String,
    pub(crate) refresh_token: Option<String>,
    pub(crate) account_id: Option<String>,
    pub(crate) email: Option<String>,
}

impl std::fmt::Debug for OpenAiQuotaSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OpenAiQuotaSnapshot([REDACTED])")
    }
}

/// quota 查询成功结果。
#[derive(Debug, Clone)]
pub(crate) struct OpenAiQuotaFetchOutcome {
    pub(crate) email: Option<String>,
    pub(crate) quota: CodexQuota,
    pub(crate) account_id: Option<String>,
    pub(crate) request_started_at: DateTime<Utc>,
    pub(crate) network_acquired_at: DateTime<Utc>,
    pub(crate) returned_at: DateTime<Utc>,
    pub(crate) cache_hit: bool,
}

/// 统一格式化 OpenAI 账号类型标签。
///
/// 示例：
/// - `PLUS` / `plus` -> `plus`
/// - `TEAM` -> `team`
/// - `PRO_20X` / `pro-20x` -> `pro 20x`
pub(crate) fn normalize_openai_plan(plan: &str) -> String {
    plan.trim()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .map(|segment| segment.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone)]
struct CachedQuotaEntry {
    outcome: OpenAiQuotaFetchOutcome,
    cached_at: Instant,
}

/// OpenAI OAuth quota 共享核心。
pub(crate) struct OpenAiQuotaCore;

impl OpenAiQuotaCore {
    /// 查询 quota；必要时刷新 access token，并通过调用方回写新 token。
    /// `force_refresh` 仅绕过配额缓存；access token 过期或认证被拒绝时才刷新。
    pub(crate) async fn fetch_quota<F, Fut>(
        snapshot: OpenAiQuotaSnapshot,
        force_refresh: bool,
        mut persist_tokens: F,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, String>
    where
        F: FnMut(TokenRefreshResponse) -> Fut,
        Fut: Future<Output = std::result::Result<(), String>>,
    {
        let request_started_at = Utc::now();
        let mut access_token = snapshot.access_token.trim().to_string();
        if access_token.is_empty() {
            return Err("账号缺少 access_token".to_string());
        }

        let mut refresh_token = snapshot
            .refresh_token
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let mut id_token = snapshot.id_token;
        let mut account_id = snapshot.account_id;
        if account_id.is_none() {
            account_id = Self::extract_account_id(&access_token);
        }

        let mut email = snapshot.email;
        if email.is_none() {
            email = Self::extract_email(&access_token);
        }

        if !force_refresh
            && let Some(outcome) = Self::read_cached_quota(
                account_id.as_deref(),
                id_token.as_deref(),
                refresh_token.as_deref(),
                &access_token,
                Instant::now(),
            )
        {
            return Ok(OpenAiQuotaFetchOutcome {
                request_started_at,
                ..outcome
            });
        }

        if Self::is_token_expired(&access_token) {
            let rt = refresh_token
                .as_deref()
                .ok_or_else(|| "Token 已过期且缺少 refresh_token".to_string())?;
            let new_tokens = Self::refresh_access_token(rt).await?;
            persist_tokens(new_tokens.clone()).await?;
            if let Some(new_id) = new_tokens.id_token.clone() {
                id_token = Some(new_id);
            }
            access_token = new_tokens.access_token.clone();
            if let Some(new_refresh) = new_tokens.refresh_token.clone() {
                refresh_token = Some(new_refresh);
            }
            if account_id.is_none() {
                account_id = Self::extract_account_id(&access_token);
            }
            email = Self::extract_email(&access_token).or(email);
        }

        match Self::call_usage_api(&access_token, account_id.as_deref()).await {
            Ok((quota, network_acquired_at)) => {
                let outcome = OpenAiQuotaFetchOutcome {
                    email,
                    quota,
                    account_id: account_id.clone(),
                    request_started_at,
                    network_acquired_at,
                    returned_at: Utc::now(),
                    cache_hit: false,
                };
                Self::write_cached_quota(
                    account_id.as_deref(),
                    id_token.as_deref(),
                    refresh_token.as_deref(),
                    &access_token,
                    outcome.clone(),
                    Instant::now(),
                );
                Ok(outcome)
            }
            Err(error) => {
                if Self::should_force_refresh(&error)
                    && let Some(rt) = refresh_token.as_deref()
                {
                    let new_tokens = Self::refresh_access_token(rt).await?;
                    persist_tokens(new_tokens.clone()).await?;
                    if let Some(new_id) = new_tokens.id_token.clone() {
                        id_token = Some(new_id);
                    }
                    access_token = new_tokens.access_token.clone();
                    if let Some(new_refresh) = new_tokens.refresh_token.clone() {
                        refresh_token = Some(new_refresh);
                    }
                    if account_id.is_none() {
                        account_id = Self::extract_account_id(&access_token);
                    }
                    email = Self::extract_email(&access_token).or(email);
                    let (quota, network_acquired_at) =
                        Self::call_usage_api(&access_token, account_id.as_deref()).await?;
                    let outcome = OpenAiQuotaFetchOutcome {
                        email,
                        quota,
                        account_id: account_id.clone(),
                        request_started_at,
                        network_acquired_at,
                        returned_at: Utc::now(),
                        cache_hit: false,
                    };
                    Self::write_cached_quota(
                        account_id.as_deref(),
                        id_token.as_deref(),
                        refresh_token.as_deref(),
                        &access_token,
                        outcome.clone(),
                        Instant::now(),
                    );
                    return Ok(outcome);
                }

                Err(error)
            }
        }
    }

    /// 检查 JWT access_token 是否过期。
    pub(crate) fn is_token_expired(access_token: &str) -> bool {
        let parts: Vec<&str> = access_token.split('.').collect();
        if parts.len() != 3 {
            return true;
        }

        let payload = match crate::utils::decode_base64url(parts[1]) {
            Some(bytes) => bytes,
            None => return true,
        };

        let value: serde_json::Value = match serde_json::from_slice(&payload) {
            Ok(value) => value,
            Err(_) => return true,
        };

        let exp = match value.get("exp").and_then(|value| value.as_i64()) {
            Some(exp) => exp,
            None => return true,
        };

        // 提前 60 秒视为过期，避免临界点抖动。
        exp < Utc::now().timestamp() + 60
    }

    /// 从 JWT access_token 中提取 account_id。
    pub(crate) fn extract_account_id(access_token: &str) -> Option<String> {
        let parts: Vec<&str> = access_token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let payload = crate::utils::decode_base64url(parts[1])?;
        let value: serde_json::Value = serde_json::from_slice(&payload).ok()?;

        value
            .get("chatgpt_account_id")
            .or_else(|| value.get("account_id"))
            .or_else(|| {
                value
                    .get("https://api.openai.com/auth")
                    .and_then(|value| value.get("account_id"))
            })
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    /// 从 JWT access_token 中提取邮箱。
    pub(crate) fn extract_email(access_token: &str) -> Option<String> {
        let parts: Vec<&str> = access_token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let payload = crate::utils::decode_base64url(parts[1])?;
        let value: serde_json::Value = serde_json::from_slice(&payload).ok()?;

        value
            .get("email")
            .or_else(|| {
                value
                    .get("https://api.openai.com/profile")
                    .and_then(|value| value.get("email"))
            })
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    /// 将秒级 reset 时间戳格式化为人类可读字符串。
    pub(crate) fn format_reset_duration(reset_timestamp: i64) -> String {
        let now = Utc::now().timestamp();
        let remaining = reset_timestamp - now;

        if remaining <= 0 {
            return "即将重置".to_string();
        }

        let hours = remaining / 3600;
        let minutes = (remaining % 3600) / 60;

        if hours > 24 {
            let days = hours / 24;
            let rem_hours = hours % 24;
            format!("{days}d{rem_hours}h")
        } else if hours > 0 {
            format!("{hours}h{minutes}m")
        } else {
            format!("{minutes}m")
        }
    }

    /// 判断错误是否值得触发 refresh 后重试。
    pub(crate) fn should_force_refresh(error_message: &str) -> bool {
        let lower = error_message.to_ascii_lowercase();
        lower.contains("token_invalidated")
            || lower.contains("authentication token has been invalidated")
            || lower.contains("401")
    }

    /// 判断错误是否属于 refresh token 永久不可用（已轮换/已吊销/已过期），需要上层补做修复。
    pub(crate) fn should_repair_tokens(error_message: &str) -> bool {
        let lower = error_message.to_ascii_lowercase();
        lower.contains("refresh_token_reused")
            || lower.contains("refresh_token_invalidated")
            || lower.contains("refresh_token_expired")
            || lower.contains("invalid_grant")
    }

    async fn call_usage_api(
        access_token: &str,
        account_id: Option<&str>,
    ) -> std::result::Result<(CodexQuota, DateTime<Utc>), String> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {access_token}"))
                .map_err(|error| format!("构建 Authorization 头失败: {error}"))?,
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let effective_id = account_id
            .map(str::to_string)
            .or_else(|| Self::extract_account_id(access_token));

        if let Some(account_id) = effective_id.as_deref()
            && !account_id.is_empty()
            && let Ok(value) = HeaderValue::from_str(account_id)
        {
            headers.insert("ChatGPT-Account-Id", value);
        }

        let url = usage_url();
        debug!(
            "OpenAI quota request: {} (account_id: {:?})",
            url, effective_id
        );

        let response = HTTP_CLIENT
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| format!("配额请求失败: {error}"))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| format!("读取配额响应失败: {error}"))?;

        if !status.is_success() {
            let error_code = Self::extract_error_code(&body);
            let mut message = format!("API 返回错误 {status}");
            if let Some(code) = error_code {
                message.push_str(&format!(" [{code}]"));
            }
            return Err(message);
        }

        let usage: UsageResponse =
            serde_json::from_str(&body).map_err(|_| "解析配额 JSON 失败".to_string())?;

        let network_acquired_at = Utc::now();
        Self::parse_quota(&usage, &body).map(|quota| (quota, network_acquired_at))
    }

    async fn refresh_access_token(
        refresh_token: &str,
    ) -> std::result::Result<TokenRefreshResponse, String> {
        let request = TokenRefreshRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
            client_id: OAUTH_CLIENT_ID.to_string(),
        };

        let response = HTTP_CLIENT
            .post(token_refresh_url())
            .json(&request)
            .send()
            .await
            .map_err(|error| format!("Token 刷新请求失败: {error}"))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| format!("读取 Token 刷新响应失败: {error}"))?;

        if !status.is_success() {
            let error_code = Self::extract_error_code(&body);
            let mut message = format!("Token 刷新失败 ({status})");
            if let Some(code) = error_code {
                message.push_str(&format!(" [{code}]"));
            }
            return Err(message);
        }

        serde_json::from_str(&body).map_err(|_| "解析 Token 刷新响应失败".to_string())
    }

    fn parse_quota(
        usage: &UsageResponse,
        raw_body: &str,
    ) -> std::result::Result<CodexQuota, String> {
        let rate_limit = usage.rate_limit.as_ref();
        let windows = [
            rate_limit.and_then(|limit| limit.primary_window.as_ref()),
            rate_limit.and_then(|limit| limit.secondary_window.as_ref()),
        ];
        let select = |duration| {
            let matches: Vec<_> = windows
                .iter()
                .flatten()
                .filter(|window| Self::window_minutes(window) == Some(duration))
                .copied()
                .collect();
            let selected = if matches.len() == 1 {
                matches.first().copied().filter(|window| {
                    window.used_percent.is_some_and(|percent| {
                        percent.is_finite() && (0.0..=100.0).contains(&percent)
                    })
                })
            } else {
                None
            };
            let present = if selected.is_some() {
                Some(true)
            } else if !matches.is_empty()
                || windows
                    .iter()
                    .flatten()
                    .any(|window| Self::window_minutes(window).is_none())
            {
                None
            } else {
                Some(false)
            };
            (selected, present)
        };
        let (primary, hourly_present) = select(300);
        let (secondary, weekly_present) = select(10080);

        let (hourly_percentage, hourly_reset_time, hourly_window_minutes) =
            if let Some(window) = primary {
                (
                    Self::remaining_percentage(window),
                    Self::reset_time(window),
                    Self::window_minutes(window),
                )
            } else {
                (100, None, None)
            };

        let (weekly_percentage, weekly_reset_time, weekly_window_minutes) =
            if let Some(window) = secondary {
                (
                    Self::remaining_percentage(window),
                    Self::reset_time(window),
                    Self::window_minutes(window),
                )
            } else {
                (100, None, None)
            };

        let raw_data = serde_json::from_str(raw_body).ok();

        Ok(CodexQuota {
            hourly_percentage,
            hourly_reset_time,
            hourly_window_minutes,
            hourly_window_present: hourly_present,
            weekly_percentage,
            weekly_reset_time,
            weekly_window_minutes,
            weekly_window_present: weekly_present,
            plan_type: usage
                .plan_type
                .as_deref()
                .map(normalize_openai_plan)
                .filter(|value| !value.is_empty()),
            raw_data,
        })
    }

    fn remaining_percentage(window: &WindowInfo) -> i32 {
        let used = window.used_percent.unwrap_or(0.0).clamp(0.0, 100.0);
        (100.0 - used).round() as i32
    }

    fn window_minutes(window: &WindowInfo) -> Option<i64> {
        let seconds = window.limit_window_seconds?;
        if seconds <= 0 || seconds % 60 != 0 {
            return None;
        }
        Some(seconds / 60)
    }

    fn reset_time(window: &WindowInfo) -> Option<i64> {
        if let Some(reset_at) = window.reset_at {
            return Some(reset_at);
        }

        let reset_after = window.reset_after_seconds?;
        if reset_after < 0 {
            return None;
        }

        Some(Utc::now().timestamp() + reset_after)
    }

    fn extract_error_code(body: &str) -> Option<&'static str> {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
            for code in [
                value.get("detail").and_then(|detail| detail.get("code")),
                value.get("error").and_then(|error| error.get("code")),
                value.get("code"),
                value.get("error"),
            ]
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
            {
                match code {
                    "token_invalidated" => return Some("token_invalidated"),
                    "refresh_token_reused" => return Some("refresh_token_reused"),
                    "refresh_token_invalidated" => return Some("refresh_token_invalidated"),
                    "refresh_token_expired" => return Some("refresh_token_expired"),
                    "invalid_grant" => return Some("invalid_grant"),
                    _ => {}
                }
            }
        }
        body.to_ascii_lowercase()
            .contains("authentication token has been invalidated")
            .then_some("token_invalidated")
    }

    fn read_cached_quota(
        account_id: Option<&str>,
        id_token: Option<&str>,
        refresh_token: Option<&str>,
        access_token: &str,
        now: Instant,
    ) -> Option<OpenAiQuotaFetchOutcome> {
        let cache_key = Self::cache_key(account_id, id_token, refresh_token, access_token);
        let mut cache = QUOTA_CACHE.lock().ok()?;
        cache.retain(|_, entry| now.saturating_duration_since(entry.cached_at) <= QUOTA_CACHE_TTL);
        cache.get(&cache_key).map(|entry| {
            let mut outcome = entry.outcome.clone();
            outcome.cache_hit = true;
            outcome.returned_at = Utc::now();
            outcome
        })
    }

    fn write_cached_quota(
        account_id: Option<&str>,
        id_token: Option<&str>,
        refresh_token: Option<&str>,
        access_token: &str,
        outcome: OpenAiQuotaFetchOutcome,
        cached_at: Instant,
    ) {
        let cache_key = Self::cache_key(account_id, id_token, refresh_token, access_token);
        if let Ok(mut cache) = QUOTA_CACHE.lock() {
            cache.insert(cache_key, CachedQuotaEntry { outcome, cached_at });
        }
    }

    fn cache_key(
        account_id: Option<&str>,
        id_token: Option<&str>,
        refresh_token: Option<&str>,
        access_token: &str,
    ) -> String {
        let tokens = CodexAuthTokens {
            id_token: id_token.map(str::to_string),
            access_token: Some(access_token.to_string()),
            refresh_token: None,
            account_id: account_id.map(str::to_string),
        };
        if let Some(identity) = OAuthIdentity::from_tokens(&tokens) {
            return format!("identity:{}", identity.key());
        }

        if let Some(refresh_token) = Self::normalized_identity(refresh_token) {
            return format!("refresh:{}", Self::fingerprint(refresh_token));
        }

        format!("access:{}", Self::fingerprint(access_token))
    }

    fn normalized_identity(value: Option<&str>) -> Option<&str> {
        value.map(str::trim).filter(|value| !value.is_empty())
    }

    fn fingerprint(value: &str) -> String {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use chrono::Duration as ChronoDuration;
    use serde_json::json;

    #[test]
    fn p4_http_code_extraction_allows_only_fixed_diagnostics() {
        for body in [
            json!({"error":{"code":"p4-private-code"}}).to_string(),
            json!({"detail":{"code":"unknown_invalid_grant_marker"}}).to_string(),
            "中".repeat(5000),
        ] {
            assert!(OpenAiQuotaCore::extract_error_code(&body).is_none());
        }
        for code in [
            "token_invalidated",
            "refresh_token_reused",
            "refresh_token_invalidated",
            "refresh_token_expired",
            "invalid_grant",
        ] {
            for body in [
                json!({"detail":{"code":code}}),
                json!({"error":{"code":code}}),
                json!({"code":code}),
                json!({"error":code}),
            ] {
                assert_eq!(
                    OpenAiQuotaCore::extract_error_code(&body.to_string()),
                    Some(code)
                );
            }
        }
        assert_eq!(OpenAiQuotaCore::extract_error_code(&json!({"error":{"message":"authentication token has been invalidated; p4-private-body"}}).to_string()), Some("token_invalidated"));
    }

    #[test]
    fn p4_token_http_debug_hides_refresh_and_response_credentials() {
        let request = TokenRefreshRequest {
            grant_type: "refresh_token".into(),
            refresh_token: "p4-private-request".into(),
            client_id: "client-id".into(),
        };
        let response = TokenRefreshResponse {
            access_token: "p4-private-access".into(),
            refresh_token: Some("p4-private-refresh".into()),
            id_token: Some("p4-private-id".into()),
        };
        assert!(!format!("{request:?}").contains("p4-private-"));
        assert!(!format!("{response:?}").contains("p4-private-"));
    }

    #[tokio::test]
    async fn p4_http_errors_never_echo_unknown_codes_messages_or_unicode_bodies() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        for body in [
            json!({"error":{"code":"p4-private-code","message":"p4-private-message@example.invalid"}}).to_string(),
            format!("{}p4-private-body{}", "中".repeat(199), "文".repeat(5000)),
            json!({"error":"invalid_grant","error_description":"p4-private-description"}).to_string(),
        ] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let base = format!("http://{}", listener.local_addr().unwrap());
            let endpoints = TestEndpoints { usage:format!("{base}/usage"), token:format!("{base}/token") };
            let server_body = body.clone();
            let server = tokio::spawn(async move {
                for _ in 0..2 {
                    let (mut stream, _) = listener.accept().await.unwrap();
                    let mut buffer = [0u8; 4096];
                    let length = stream.read(&mut buffer).await.unwrap();
                    assert!(length > 0);
                    let response = format!("HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{server_body}", server_body.len());
                    stream.write_all(response.as_bytes()).await.unwrap();
                }
            });
            let (quota, refresh) = TEST_ENDPOINTS.scope(endpoints, async {
                (OpenAiQuotaCore::call_usage_api("synthetic-access", None).await.unwrap_err(), OpenAiQuotaCore::refresh_access_token("synthetic-refresh").await.unwrap_err())
            }).await;
            server.await.unwrap();
            for error in [quota, refresh] {
                assert!(error.contains("400"));
                assert!(!error.contains("p4-private-") && !error.contains('中') && !error.contains('文'));
                if body.contains("invalid_grant") { assert!(error.contains("[invalid_grant]")); }
            }
        }
    }

    #[tokio::test]
    async fn p4_invalidated_phrase_on_403_still_refreshes_and_retries() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let endpoints = TestEndpoints {
            usage: format!("{base}/usage"),
            token: format!("{base}/token"),
        };
        let server = tokio::spawn(async move {
            let mut methods = Vec::new();
            for (status, payload) in [
                (
                    "403 Forbidden",
                    json!({"error":{"message":"authentication token has been invalidated; p4-private-body"}}),
                ),
                (
                    "200 OK",
                    json!({"access_token":"synthetic-new-access", "refresh_token":"synthetic-new-refresh"}),
                ),
                (
                    "200 OK",
                    json!({"rate_limit":{"primary_window":{"used_percent":15, "limit_window_seconds":18000}}}),
                ),
            ] {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut buffer = [0u8; 4096];
                let length = stream.read(&mut buffer).await.unwrap();
                methods.push(
                    String::from_utf8_lossy(&buffer[..length])
                        .lines()
                        .next()
                        .unwrap()
                        .to_string(),
                );
                let body = payload.to_string();
                stream.write_all(format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
            }
            methods
        });
        let mut persisted = 0;
        let result = TEST_ENDPOINTS
            .scope(
                endpoints,
                OpenAiQuotaCore::fetch_quota(
                    OpenAiQuotaSnapshot {
                        id_token: None,
                        access_token: fake_jwt(json!({"exp":Utc::now().timestamp()+3600})),
                        refresh_token: Some("synthetic-old-refresh".into()),
                        account_id: None,
                        email: None,
                    },
                    true,
                    |_| {
                        persisted += 1;
                        std::future::ready(Ok(()))
                    },
                ),
            )
            .await
            .unwrap();
        assert_eq!(result.quota.hourly_percentage, 85);
        assert_eq!(persisted, 1);
        assert_eq!(
            server.await.unwrap(),
            [
                "GET /usage HTTP/1.1",
                "POST /token HTTP/1.1",
                "GET /usage HTTP/1.1"
            ]
        );
    }

    #[tokio::test]
    async fn p4_success_status_parse_errors_do_not_echo_invalid_response_values() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let endpoints = TestEndpoints {
            usage: format!("{base}/usage"),
            token: format!("{base}/token"),
        };
        let server = tokio::spawn(async move {
            for payload in [
                json!({"rate_limit":{"primary_window":{"used_percent":"p4-private-invalid-number"}}}),
                json!({"access_token":123456789}),
            ] {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut buffer = [0u8; 4096];
                let length = stream.read(&mut buffer).await.unwrap();
                assert!(length > 0);
                let body = payload.to_string();
                stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
            }
        });
        let errors = TEST_ENDPOINTS
            .scope(endpoints, async {
                [
                    OpenAiQuotaCore::call_usage_api("synthetic-access", None)
                        .await
                        .unwrap_err(),
                    OpenAiQuotaCore::refresh_access_token("synthetic-refresh")
                        .await
                        .unwrap_err(),
                ]
            })
            .await;
        server.await.unwrap();
        for error in errors {
            assert!(!error.contains("p4-private-") && !error.contains("123456789"));
        }
    }

    fn sample_outcome() -> OpenAiQuotaFetchOutcome {
        OpenAiQuotaFetchOutcome {
            account_id: Some("synthetic-account".into()),
            request_started_at: Utc::now(),
            network_acquired_at: Utc::now(),
            returned_at: Utc::now(),
            cache_hit: false,
            email: Some("user@example.com".to_string()),
            quota: CodexQuota {
                hourly_percentage: 75,
                hourly_reset_time: Some(1_800_000_000),
                hourly_window_minutes: Some(300),
                hourly_window_present: Some(true),
                weekly_percentage: 80,
                weekly_reset_time: Some(1_800_360_000),
                weekly_window_minutes: Some(7 * 24 * 60),
                weekly_window_present: Some(true),
                plan_type: Some("plus".to_string()),
                raw_data: None,
            },
        }
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

    #[derive(Default)]
    struct ManualQuotaRequests {
        usage_tokens: Vec<String>,
        refresh_calls: usize,
    }

    struct ManualQuotaStub {
        snapshot: OpenAiQuotaSnapshot,
        refreshed_access: String,
        endpoints: TestEndpoints,
        requests: std::sync::Arc<Mutex<ManualQuotaRequests>>,
        server: tokio::task::JoinHandle<()>,
    }

    impl Drop for ManualQuotaStub {
        fn drop(&mut self) {
            self.server.abort();
        }
    }

    impl ManualQuotaStub {
        async fn start(
            expired: bool,
            reject_access: bool,
            refresh_error: Option<&'static str>,
        ) -> Self {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};

            let account_id = format!("manual-quota-{}", uuid::Uuid::new_v4());
            let access = |expiry, nonce| {
                fake_jwt(json!({
                    "chatgpt_user_id": format!("user-{account_id}"),
                    "chatgpt_account_id": account_id,
                    "exp": expiry,
                    "nonce": nonce
                }))
            };
            let original_access = access(
                Utc::now().timestamp() + if expired { -3600 } else { 3600 },
                0,
            );
            let refreshed_access = access(Utc::now().timestamp() + 3600, 1);
            let snapshot = OpenAiQuotaSnapshot {
                id_token: Some(original_access.clone()),
                access_token: original_access.clone(),
                refresh_token: Some("synthetic-old-refresh".into()),
                account_id: Some(account_id),
                email: None,
            };
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let base = format!("http://{}", listener.local_addr().unwrap());
            let requests = std::sync::Arc::new(Mutex::new(ManualQuotaRequests::default()));
            let server_requests = requests.clone();
            let new_access = refreshed_access.clone();
            let server = tokio::spawn(async move {
                while let Ok((mut stream, _)) = listener.accept().await {
                    let mut buffer = Vec::new();
                    let mut chunk = [0_u8; 4096];
                    let header_end = loop {
                        let read = stream.read(&mut chunk).await.unwrap();
                        if read == 0 {
                            return;
                        }
                        buffer.extend_from_slice(&chunk[..read]);
                        if let Some(position) = buffer.windows(4).position(|v| v == b"\r\n\r\n") {
                            break position + 4;
                        }
                    };
                    let head = String::from_utf8_lossy(&buffer[..header_end]);
                    let header = |name: &str| {
                        head.lines().find_map(|line| {
                            let (key, value) = line.split_once(':')?;
                            key.eq_ignore_ascii_case(name).then(|| value.trim())
                        })
                    };
                    let body_length = header("content-length")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or(0);
                    let method_path = head.lines().next().unwrap().to_string();
                    let bearer = header("authorization")
                        .unwrap_or("")
                        .trim_start_matches("Bearer ")
                        .to_string();
                    while buffer.len() < header_end + body_length {
                        let read = stream.read(&mut chunk).await.unwrap();
                        if read == 0 {
                            return;
                        }
                        buffer.extend_from_slice(&chunk[..read]);
                    }
                    let (status, payload) = {
                        let mut requests = server_requests.lock().unwrap();
                        if method_path.starts_with("GET /usage ") {
                            requests.usage_tokens.push(bearer.clone());
                            if reject_access && bearer == original_access {
                                (
                                    "401 Unauthorized",
                                    json!({"error":{"code":"token_invalidated"}}),
                                )
                            } else {
                                (
                                    "200 OK",
                                    json!({"rate_limit":{"primary_window":{
                                        "used_percent": requests.usage_tokens.len() * 10,
                                        "limit_window_seconds": 18000
                                    }}}),
                                )
                            }
                        } else if method_path.starts_with("POST /oauth/token ") {
                            requests.refresh_calls += 1;
                            match refresh_error {
                                Some(code) => ("401 Unauthorized", json!({"error":{"code":code}})),
                                None => (
                                    "200 OK",
                                    json!({
                                        "access_token": new_access,
                                        "id_token": new_access,
                                        "refresh_token": "synthetic-new-refresh"
                                    }),
                                ),
                            }
                        } else {
                            ("404 Not Found", json!({}))
                        }
                    };
                    let body = payload.to_string();
                    let response = format!(
                        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    stream.write_all(response.as_bytes()).await.unwrap();
                }
            });
            Self {
                snapshot,
                refreshed_access,
                endpoints: TestEndpoints {
                    usage: format!("{base}/usage"),
                    token: format!("{base}/oauth/token"),
                },
                requests,
                server,
            }
        }
    }

    #[tokio::test]
    async fn manual_quota_uses_live_access_without_refresh_or_persistence() {
        let stub = ManualQuotaStub::start(false, false, Some("refresh_token_invalidated")).await;
        let mut persisted = 0;
        let result = TEST_ENDPOINTS
            .scope(
                stub.endpoints.clone(),
                OpenAiQuotaCore::fetch_quota(stub.snapshot.clone(), true, |_| {
                    persisted += 1;
                    std::future::ready(Ok(()))
                }),
            )
            .await;

        assert!(result.is_ok(), "live access should query quota: {result:?}");
        let outcome = result.unwrap();
        assert_eq!(outcome.quota.hourly_percentage, 90);
        assert!(!outcome.cache_hit);
        assert_eq!(persisted, 0);
        let requests = stub.requests.lock().unwrap();
        assert_eq!(requests.refresh_calls, 0);
        assert_eq!(requests.usage_tokens.len(), 1);
        assert!(requests.usage_tokens[0] == stub.snapshot.access_token);
    }

    #[tokio::test]
    async fn manual_quota_bypasses_cache_without_refreshing_live_access() {
        let stub = ManualQuotaStub::start(false, false, Some("refresh_token_reused")).await;
        let mut persisted = 0;
        TEST_ENDPOINTS
            .scope(stub.endpoints.clone(), async {
                let first = OpenAiQuotaCore::fetch_quota(stub.snapshot.clone(), false, |_| {
                    persisted += 1;
                    std::future::ready(Ok(()))
                })
                .await
                .unwrap();
                let cached = OpenAiQuotaCore::fetch_quota(stub.snapshot.clone(), false, |_| {
                    persisted += 1;
                    std::future::ready(Ok(()))
                })
                .await
                .unwrap();
                assert!(cached.cache_hit);
                assert_eq!(cached.network_acquired_at, first.network_acquired_at);
                let fresh = OpenAiQuotaCore::fetch_quota(stub.snapshot.clone(), true, |_| {
                    persisted += 1;
                    std::future::ready(Ok(()))
                })
                .await
                .unwrap();
                assert!(!fresh.cache_hit);
                assert_eq!(fresh.quota.hourly_percentage, 80);
            })
            .await;
        assert_eq!(persisted, 0);
        let requests = stub.requests.lock().unwrap();
        assert_eq!(requests.usage_tokens.len(), 2);
        assert_eq!(requests.refresh_calls, 0);
    }

    #[tokio::test]
    async fn manual_quota_refreshes_expired_access_before_query() {
        let stub = ManualQuotaStub::start(true, false, None).await;
        let mut persisted = Vec::new();
        let result = TEST_ENDPOINTS
            .scope(
                stub.endpoints.clone(),
                OpenAiQuotaCore::fetch_quota(stub.snapshot.clone(), true, |tokens| {
                    persisted.push(tokens.access_token);
                    std::future::ready(Ok(()))
                }),
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(persisted.len(), 1);
        assert!(persisted[0] == stub.refreshed_access);
        let requests = stub.requests.lock().unwrap();
        assert_eq!(requests.refresh_calls, 1);
        assert_eq!(requests.usage_tokens.len(), 1);
        assert!(requests.usage_tokens[0] == stub.refreshed_access);
    }

    #[tokio::test]
    async fn manual_quota_refreshes_rejected_access_and_retries_query() {
        let stub = ManualQuotaStub::start(false, true, None).await;
        let mut persisted = 0;
        let result = TEST_ENDPOINTS
            .scope(
                stub.endpoints.clone(),
                OpenAiQuotaCore::fetch_quota(stub.snapshot.clone(), true, |_| {
                    persisted += 1;
                    std::future::ready(Ok(()))
                }),
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(persisted, 1);
        let requests = stub.requests.lock().unwrap();
        assert_eq!(requests.refresh_calls, 1);
        assert_eq!(requests.usage_tokens.len(), 2);
        assert!(requests.usage_tokens[0] == stub.snapshot.access_token);
        assert!(requests.usage_tokens[1] == stub.refreshed_access);
    }

    #[tokio::test]
    async fn manual_quota_does_not_report_success_when_required_refresh_fails() {
        for expired in [false, true] {
            let stub =
                ManualQuotaStub::start(expired, true, Some("refresh_token_invalidated")).await;
            let mut persisted = 0;
            let result = TEST_ENDPOINTS
                .scope(
                    stub.endpoints.clone(),
                    OpenAiQuotaCore::fetch_quota(stub.snapshot.clone(), true, |_| {
                        persisted += 1;
                        std::future::ready(Ok(()))
                    }),
                )
                .await;
            let error = result.unwrap_err();
            assert!(OpenAiQuotaCore::should_repair_tokens(&error));
            assert_eq!(persisted, 0);
            assert!(
                OpenAiQuotaCore::read_cached_quota(
                    stub.snapshot.account_id.as_deref(),
                    stub.snapshot.id_token.as_deref(),
                    stub.snapshot.refresh_token.as_deref(),
                    &stub.snapshot.access_token,
                    Instant::now()
                )
                .is_none()
            );
            let requests = stub.requests.lock().unwrap();
            assert_eq!(requests.refresh_calls, 1);
            assert_eq!(requests.usage_tokens.len(), usize::from(!expired));
        }
    }

    #[test]
    fn extract_email_and_account_id_from_access_token() {
        let token = fake_jwt(json!({
            "email": "user@example.com",
            "chatgpt_account_id": "acc-123",
            "exp": (Utc::now() + ChronoDuration::hours(1)).timestamp()
        }));

        assert_eq!(
            OpenAiQuotaCore::extract_email(&token).as_deref(),
            Some("user@example.com")
        );
        assert_eq!(
            OpenAiQuotaCore::extract_account_id(&token).as_deref(),
            Some("acc-123")
        );
        assert!(!OpenAiQuotaCore::is_token_expired(&token));
    }

    #[test]
    fn format_reset_duration_covers_hours_and_days() {
        let now = Utc::now().timestamp();
        assert_eq!(
            OpenAiQuotaCore::format_reset_duration(now + 90 * 60),
            "1h30m"
        );
        assert_eq!(
            OpenAiQuotaCore::format_reset_duration(now + 49 * 3600),
            "2d1h"
        );
    }

    #[test]
    fn normalize_openai_plan_formats_common_variants() {
        assert_eq!(normalize_openai_plan("PLUS"), "plus");
        assert_eq!(normalize_openai_plan("TEAM"), "team");
        assert_eq!(normalize_openai_plan("PRO_20X"), "pro 20x");
        assert_eq!(normalize_openai_plan("pro-20x"), "pro 20x");
    }

    #[test]
    fn parse_quota_translates_used_percent_to_remaining_budget() {
        let usage = UsageResponse {
            plan_type: Some("plus".to_string()),
            rate_limit: Some(RateLimitInfo {
                allowed: Some(true),
                limit_reached: Some(false),
                primary_window: Some(WindowInfo {
                    used_percent: Some(48.0),
                    limit_window_seconds: Some(5 * 3600),
                    reset_after_seconds: Some(3600),
                    reset_at: None,
                }),
                secondary_window: Some(WindowInfo {
                    used_percent: Some(17.0),
                    limit_window_seconds: Some(7 * 24 * 3600),
                    reset_after_seconds: Some(7200),
                    reset_at: None,
                }),
            }),
            code_review_rate_limit: None,
        };

        let quota = OpenAiQuotaCore::parse_quota(&usage, r#"{"plan_type":"plus"}"#).unwrap();
        assert_eq!(quota.hourly_percentage, 52);
        assert_eq!(quota.weekly_percentage, 83);
        assert_eq!(quota.hourly_window_minutes, Some(300));
        assert_eq!(quota.plan_type.as_deref(), Some("plus"));
        assert!(quota.raw_data.is_some());
    }

    #[test]
    fn cache_key_prefers_complete_identity_for_cross_surface_reuse() {
        let saved_key = OpenAiQuotaCore::cache_key(
            Some("acc-shared"),
            Some(&super::super::codex_auth_identity::test_jwt(
                json!({"chatgpt_user_id":"cache-user"}),
            )),
            Some("refresh-a"),
            "access-a",
        );
        let runtime_key = OpenAiQuotaCore::cache_key(
            Some("acc-shared"),
            Some(&super::super::codex_auth_identity::test_jwt(
                json!({"chatgpt_user_id":"cache-user"}),
            )),
            Some("refresh-b"),
            "access-b",
        );

        assert_eq!(saved_key, runtime_key);
    }

    #[test]
    fn quota_cache_reuses_recent_entry_for_same_complete_identity() {
        let now = Instant::now();
        let outcome = sample_outcome();

        OpenAiQuotaCore::write_cached_quota(
            Some("acc-shared"),
            Some(&super::super::codex_auth_identity::test_jwt(
                json!({"chatgpt_user_id":"cache-user"}),
            )),
            Some("refresh-a"),
            "access-a",
            outcome.clone(),
            now,
        );

        let cached = OpenAiQuotaCore::read_cached_quota(
            Some("acc-shared"),
            Some(&super::super::codex_auth_identity::test_jwt(
                json!({"chatgpt_user_id":"cache-user"}),
            )),
            Some("refresh-b"),
            "access-b",
            now + ChronoDuration::seconds(5)
                .to_std()
                .expect("positive duration"),
        )
        .expect("cache should hit for same account id");

        assert_eq!(cached.email.as_deref(), Some("user@example.com"));
        assert_eq!(cached.quota.hourly_percentage, 75);
        assert!(cached.cache_hit);
        assert_eq!(cached.network_acquired_at, outcome.network_acquired_at);
        assert!(cached.returned_at >= outcome.returned_at);
    }

    #[test]
    fn quota_cache_expires_stale_entry() {
        let now = Instant::now();

        OpenAiQuotaCore::write_cached_quota(
            Some("acc-expired"),
            Some("user@example.com"),
            Some("refresh-token"),
            "access-token",
            sample_outcome(),
            now - QUOTA_CACHE_TTL - std::time::Duration::from_secs(1),
        );

        let cached = OpenAiQuotaCore::read_cached_quota(
            Some("acc-expired"),
            Some("user@example.com"),
            Some("refresh-token"),
            "access-token",
            now,
        );

        assert!(cached.is_none());
    }

    #[test]
    fn quota_cache_isolates_users_and_unknown_identities_in_one_workspace() {
        let now = Instant::now();
        let user_a = super::super::codex_auth_identity::test_jwt(
            json!({"chatgpt_user_id":"cache-isolation-a"}),
        );
        let user_b = super::super::codex_auth_identity::test_jwt(
            json!({"chatgpt_user_id":"cache-isolation-b"}),
        );
        OpenAiQuotaCore::write_cached_quota(
            Some("workspace-isolation"),
            Some(&user_a),
            Some("refresh-a-isolation"),
            "access-a-isolation",
            sample_outcome(),
            now,
        );
        assert!(
            OpenAiQuotaCore::read_cached_quota(
                Some("workspace-isolation"),
                Some(&user_a),
                Some("refresh-a2-isolation"),
                "access-a2-isolation",
                now
            )
            .is_some()
        );
        assert!(
            OpenAiQuotaCore::read_cached_quota(
                Some("workspace-isolation"),
                Some(&user_b),
                Some("refresh-b-isolation"),
                "access-b-isolation",
                now
            )
            .is_none()
        );
        assert!(
            OpenAiQuotaCore::read_cached_quota(
                Some("workspace-isolation"),
                None,
                Some("refresh-b-isolation"),
                "access-b-isolation",
                now
            )
            .is_none()
        );
        assert_ne!(
            OpenAiQuotaCore::cache_key(
                Some("workspace"),
                None,
                Some("refresh-unknown-a"),
                "access"
            ),
            OpenAiQuotaCore::cache_key(
                Some("workspace"),
                None,
                Some("refresh-unknown-b"),
                "access"
            )
        );
        assert_ne!(
            OpenAiQuotaCore::cache_key(Some("workspace"), None, None, "access-unknown-a"),
            OpenAiQuotaCore::cache_key(Some("workspace"), None, None, "access-unknown-b")
        );
        let snapshot = OpenAiQuotaSnapshot {
            id_token: Some(user_a.clone()),
            access_token: "access-secret".into(),
            refresh_token: Some("refresh-secret".into()),
            account_id: Some("workspace".into()),
            email: None,
        };
        let debug = format!("{snapshot:?}");
        assert!(
            !debug.contains(&user_a)
                && !debug.contains("access-secret")
                && !debug.contains("refresh-secret")
        );
    }

    #[test]
    fn quota_window_roles_follow_duration_and_missing_percent_stays_unknown() {
        let raw = json!({"plan_type":"plus","rate_limit":{"primary_window":{"used_percent":94.25,"limit_window_seconds":604800},"secondary_window":{"used_percent":41.125,"limit_window_seconds":18000}}});
        let usage: UsageResponse = serde_json::from_value(raw.clone()).unwrap();
        let quota = OpenAiQuotaCore::parse_quota(&usage, &raw.to_string()).unwrap();
        assert_eq!(quota.hourly_percentage, 59);
        assert_eq!(quota.weekly_percentage, 6);
        assert_eq!(quota.hourly_window_present, Some(true));
        assert_eq!(quota.weekly_window_present, Some(true));
        for percent in [serde_json::Value::Null, json!(-1), json!(101)] {
            let mut raw = raw.clone();
            raw["rate_limit"]["secondary_window"]["used_percent"] = percent;
            let usage = serde_json::from_value(raw.clone()).unwrap();
            let quota = OpenAiQuotaCore::parse_quota(&usage, &raw.to_string()).unwrap();
            assert_eq!(quota.hourly_window_present, None);
            assert_eq!(quota.weekly_window_present, Some(true));
        }
        let raw = json!({"rate_limit":{"primary_window":{"used_percent":20}}});
        let quota = OpenAiQuotaCore::parse_quota(
            &serde_json::from_value(raw.clone()).unwrap(),
            &raw.to_string(),
        )
        .unwrap();
        assert_eq!(quota.hourly_window_present, None);
        assert_eq!(quota.weekly_window_present, None);
        let raw = json!({"rate_limit":{"primary_window":{"used_percent":20,"limit_window_seconds":36000}}});
        let quota = OpenAiQuotaCore::parse_quota(
            &serde_json::from_value(raw.clone()).unwrap(),
            &raw.to_string(),
        )
        .unwrap();
        assert_eq!(quota.hourly_window_present, Some(false));
        assert_eq!(quota.weekly_window_present, Some(false));
    }

    #[test]
    fn should_repair_tokens_covers_permanent_refresh_failures() {
        for code in [
            "refresh_token_reused",
            "refresh_token_invalidated",
            "refresh_token_expired",
            "invalid_grant",
        ] {
            assert!(
                OpenAiQuotaCore::should_repair_tokens(&format!("Token 刷新失败 (401) [{code}]")),
                "{code}"
            );
        }
        assert!(!OpenAiQuotaCore::should_repair_tokens(
            "配额请求失败: timeout"
        ));
        assert!(!OpenAiQuotaCore::should_repair_tokens(
            "API 返回错误 401 [token_expired]"
        ));
    }
}
