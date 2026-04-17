// 💰 OpenAI OAuth 配额共享核心
// 复用 wham/usage API 查询、JWT 解析与 token 刷新逻辑。

use crate::models::CodexQuota;
use chrono::Utc;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tracing::debug;

/// 全局复用的 HTTP 客户端（内部为 Arc，clone 开销极低）
static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);
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

/// 使用率窗口（5小时/周）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowInfo {
    used_percent: Option<i32>,
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
#[derive(Debug, Serialize)]
struct TokenRefreshRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
}

/// OAuth token 刷新响应
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TokenRefreshResponse {
    pub(crate) access_token: String,
    #[serde(default)]
    pub(crate) id_token: Option<String>,
    #[serde(default)]
    pub(crate) refresh_token: Option<String>,
}

/// 共享 quota 查询所需的最小快照。
#[derive(Debug, Clone)]
pub(crate) struct OpenAiQuotaSnapshot {
    pub(crate) access_token: String,
    pub(crate) refresh_token: Option<String>,
    pub(crate) account_id: Option<String>,
    pub(crate) email: Option<String>,
}

/// quota 查询成功结果。
#[derive(Debug, Clone)]
pub(crate) struct OpenAiQuotaFetchOutcome {
    pub(crate) email: Option<String>,
    pub(crate) quota: CodexQuota,
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
    pub(crate) async fn fetch_quota<F>(
        snapshot: OpenAiQuotaSnapshot,
        force_refresh: bool,
        mut persist_tokens: F,
    ) -> std::result::Result<OpenAiQuotaFetchOutcome, String>
    where
        F: FnMut(&TokenRefreshResponse) -> std::result::Result<(), String>,
    {
        let mut access_token = snapshot.access_token.trim().to_string();
        if access_token.is_empty() {
            return Err("账号缺少 access_token".to_string());
        }

        let mut refresh_token = snapshot
            .refresh_token
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
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
                email.as_deref(),
                refresh_token.as_deref(),
                &access_token,
                Instant::now(),
            )
        {
            return Ok(outcome);
        }

        if force_refresh || Self::is_token_expired(&access_token) {
            let rt = refresh_token
                .as_deref()
                .ok_or_else(|| "Token 已过期且缺少 refresh_token".to_string())?;
            let new_tokens = Self::refresh_access_token(rt).await?;
            persist_tokens(&new_tokens)?;
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
            Ok(quota) => {
                let outcome = OpenAiQuotaFetchOutcome { email, quota };
                Self::write_cached_quota(
                    account_id.as_deref(),
                    outcome.email.as_deref(),
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
                    persist_tokens(&new_tokens)?;
                    access_token = new_tokens.access_token.clone();
                    if let Some(new_refresh) = new_tokens.refresh_token.clone() {
                        refresh_token = Some(new_refresh);
                    }
                    if account_id.is_none() {
                        account_id = Self::extract_account_id(&access_token);
                    }
                    email = Self::extract_email(&access_token).or(email);
                    let quota = Self::call_usage_api(&access_token, account_id.as_deref()).await?;
                    let outcome = OpenAiQuotaFetchOutcome { email, quota };
                    Self::write_cached_quota(
                        account_id.as_deref(),
                        outcome.email.as_deref(),
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

    /// 判断错误是否属于 refresh token 已轮换，需要上层补做修复。
    pub(crate) fn should_repair_tokens(error_message: &str) -> bool {
        let lower = error_message.to_ascii_lowercase();
        lower.contains("refresh_token_reused") || lower.contains("invalid_grant")
    }

    async fn call_usage_api(
        access_token: &str,
        account_id: Option<&str>,
    ) -> std::result::Result<CodexQuota, String> {
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

        debug!(
            "OpenAI quota request: {} (account_id: {:?})",
            USAGE_URL, effective_id
        );

        let response = HTTP_CLIENT
            .get(USAGE_URL)
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
            let body_preview = if body.len() > 200 {
                &body[..200]
            } else {
                &body
            };
            let error_code = Self::extract_error_code(&body);
            let mut message = format!("API 返回错误 {status}");
            if let Some(code) = error_code {
                message.push_str(&format!(" [{code}]"));
            }
            message.push_str(&format!(" - {body_preview}"));
            return Err(message);
        }

        let usage: UsageResponse =
            serde_json::from_str(&body).map_err(|error| format!("解析配额 JSON 失败: {error}"))?;

        Self::parse_quota(&usage, &body)
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
            .post(TOKEN_REFRESH_URL)
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
            let body_preview = if body.len() > 300 {
                &body[..300]
            } else {
                &body
            };
            let error_code = Self::extract_error_code(&body);
            let mut message = format!("Token 刷新失败 ({status})");
            if let Some(code) = error_code {
                message.push_str(&format!(" [{code}]"));
            }
            message.push_str(&format!(": {body_preview}"));
            return Err(message);
        }

        serde_json::from_str(&body).map_err(|error| format!("解析 Token 刷新响应失败: {error}"))
    }

    fn parse_quota(
        usage: &UsageResponse,
        raw_body: &str,
    ) -> std::result::Result<CodexQuota, String> {
        let rate_limit = usage.rate_limit.as_ref();
        let primary = rate_limit.and_then(|limit| limit.primary_window.as_ref());
        let secondary = rate_limit.and_then(|limit| limit.secondary_window.as_ref());

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
            hourly_window_present: Some(primary.is_some()),
            weekly_percentage,
            weekly_reset_time,
            weekly_window_minutes,
            weekly_window_present: Some(secondary.is_some()),
            plan_type: usage
                .plan_type
                .as_deref()
                .map(normalize_openai_plan)
                .filter(|value| !value.is_empty()),
            raw_data,
        })
    }

    fn remaining_percentage(window: &WindowInfo) -> i32 {
        let used = window.used_percent.unwrap_or(0).clamp(0, 100);
        100 - used
    }

    fn window_minutes(window: &WindowInfo) -> Option<i64> {
        let seconds = window.limit_window_seconds?;
        if seconds <= 0 {
            return None;
        }
        Some((seconds + 59) / 60)
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

    fn extract_error_code(body: &str) -> Option<String> {
        let value: serde_json::Value = serde_json::from_str(body).ok()?;
        value
            .get("detail")
            .and_then(|detail| detail.get("code"))
            .or_else(|| value.get("error").and_then(|error| error.get("code")))
            .or_else(|| value.get("code"))
            .and_then(|code| code.as_str())
            .map(str::to_string)
    }

    fn read_cached_quota(
        account_id: Option<&str>,
        email: Option<&str>,
        refresh_token: Option<&str>,
        access_token: &str,
        now: Instant,
    ) -> Option<OpenAiQuotaFetchOutcome> {
        let cache_key = Self::cache_key(account_id, email, refresh_token, access_token);
        let mut cache = QUOTA_CACHE.lock().ok()?;
        cache.retain(|_, entry| now.saturating_duration_since(entry.cached_at) <= QUOTA_CACHE_TTL);
        cache.get(&cache_key).map(|entry| entry.outcome.clone())
    }

    fn write_cached_quota(
        account_id: Option<&str>,
        email: Option<&str>,
        refresh_token: Option<&str>,
        access_token: &str,
        outcome: OpenAiQuotaFetchOutcome,
        cached_at: Instant,
    ) {
        let cache_key = Self::cache_key(account_id, email, refresh_token, access_token);
        if let Ok(mut cache) = QUOTA_CACHE.lock() {
            cache.insert(cache_key, CachedQuotaEntry { outcome, cached_at });
        }
    }

    fn cache_key(
        account_id: Option<&str>,
        email: Option<&str>,
        refresh_token: Option<&str>,
        access_token: &str,
    ) -> String {
        if let Some(account_id) = Self::normalized_identity(account_id) {
            return format!("account:{account_id}");
        }

        if let Some(email) = Self::normalized_identity(email) {
            return format!("email:{}", email.to_ascii_lowercase());
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

    fn sample_outcome() -> OpenAiQuotaFetchOutcome {
        OpenAiQuotaFetchOutcome {
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

    fn clear_quota_cache() {
        QUOTA_CACHE
            .lock()
            .expect("quota cache mutex should not be poisoned in tests")
            .clear();
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
                    used_percent: Some(48),
                    limit_window_seconds: Some(5 * 3600),
                    reset_after_seconds: Some(3600),
                    reset_at: None,
                }),
                secondary_window: Some(WindowInfo {
                    used_percent: Some(17),
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
    fn cache_key_prefers_account_id_for_cross_surface_reuse() {
        let saved_key = OpenAiQuotaCore::cache_key(
            Some("acc-shared"),
            Some("saved@example.com"),
            Some("refresh-a"),
            "access-a",
        );
        let runtime_key = OpenAiQuotaCore::cache_key(
            Some("acc-shared"),
            Some("runtime@example.com"),
            Some("refresh-b"),
            "access-b",
        );

        assert_eq!(saved_key, runtime_key);
    }

    #[test]
    fn quota_cache_reuses_recent_entry_for_same_account_id() {
        clear_quota_cache();
        let now = Instant::now();
        let outcome = sample_outcome();

        OpenAiQuotaCore::write_cached_quota(
            Some("acc-shared"),
            Some("saved@example.com"),
            Some("refresh-a"),
            "access-a",
            outcome.clone(),
            now,
        );

        let cached = OpenAiQuotaCore::read_cached_quota(
            Some("acc-shared"),
            Some("runtime@example.com"),
            Some("refresh-b"),
            "access-b",
            now + ChronoDuration::seconds(5)
                .to_std()
                .expect("positive duration"),
        )
        .expect("cache should hit for same account id");

        assert_eq!(cached.email.as_deref(), Some("user@example.com"));
        assert_eq!(cached.quota.hourly_percentage, 75);
    }

    #[test]
    fn quota_cache_expires_stale_entry() {
        clear_quota_cache();
        let now = Instant::now();

        OpenAiQuotaCore::write_cached_quota(
            Some("acc-expired"),
            Some("user@example.com"),
            Some("refresh-token"),
            "access-token",
            sample_outcome(),
            now,
        );

        let cached = OpenAiQuotaCore::read_cached_quota(
            Some("acc-expired"),
            Some("user@example.com"),
            Some("refresh-token"),
            "access-token",
            now + QUOTA_CACHE_TTL + std::time::Duration::from_secs(1),
        );

        assert!(cached.is_none());
    }
}
