// 💰 Codex 配额查询服务
// 查询 Codex 账号的 API 配额余额（wham/usage API）
//
// 核心职责:
// - 🔍 查询单个/所有账号的配额信息
// - 🔑 OAuth token 过期检测与自动刷新
// - 📊 解析 5h 窗口和周限额数据

use crate::core::error::{CcrError, Result};
use crate::models::{CodexAccountQuota, CodexAuthJson, CodexAuthRegistry, CodexQuota};
use chrono::Utc;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, warn};

/// wham/usage API 端点
const USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

/// OAuth token 刷新端点
const TOKEN_REFRESH_URL: &str = "https://auth.openai.com/oauth/token";

/// OAuth client_id（Codex 官方）
const OAUTH_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";

/// 并发查询上限
const MAX_CONCURRENT: usize = 5;

// ── API 响应结构体（内部使用）──

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
struct TokenRefreshResponse {
    access_token: String,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    refresh_token: Option<String>,
}

/// Codex 配额查询服务
///
/// 独立的配额查询模块，通过 wham/usage API 获取账号配额信息。
/// 支持 token 自动刷新和并发查询。
pub struct CodexQuotaService {
    /// CCR Codex 数据目录 (~/.ccr/platforms/codex/)
    ccr_codex_dir: PathBuf,
}

impl CodexQuotaService {
    /// 创建新的配额查询服务
    pub fn new() -> Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;

        let ccr_codex_dir = if let Ok(custom) = std::env::var("CCR_DATA_DIR") {
            PathBuf::from(custom).join("platforms/codex")
        } else {
            home.join(".ccr/platforms/codex")
        };

        Ok(Self { ccr_codex_dir })
    }

    // ═══════════════════════════════════════════════════════════
    // 公开查询接口
    // ═══════════════════════════════════════════════════════════

    /// 查询指定账号的配额
    pub async fn fetch_account_quota(&self, account_name: &str) -> CodexAccountQuota {
        let now = Utc::now();

        // 读取 auth 文件
        let auth_path = self.account_auth_path(account_name);
        let auth_json = match std::fs::read_to_string(&auth_path) {
            Ok(content) => content,
            Err(e) => {
                return CodexAccountQuota {
                    account_name: account_name.to_string(),
                    email: None,
                    quota: None,
                    error: Some(format!("读取 auth 文件失败: {}", e)),
                    fetched_at: now,
                };
            }
        };

        let auth: CodexAuthJson = match serde_json::from_str(&auth_json) {
            Ok(auth) => auth,
            Err(e) => {
                return CodexAccountQuota {
                    account_name: account_name.to_string(),
                    email: None,
                    quota: None,
                    error: Some(format!("解析 auth JSON 失败: {}", e)),
                    fetched_at: now,
                };
            }
        };

        let tokens = match &auth.tokens {
            Some(tokens) => tokens,
            None => {
                return CodexAccountQuota {
                    account_name: account_name.to_string(),
                    email: None,
                    quota: None,
                    error: Some("账号缺少 OAuth tokens（可能是 API Key 模式）".to_string()),
                    fetched_at: now,
                };
            }
        };

        let mut access_token = tokens.access_token.clone().unwrap_or_default();
        let refresh_token = tokens.refresh_token.clone();
        let account_id = tokens.account_id.clone();

        if access_token.is_empty() {
            return CodexAccountQuota {
                account_name: account_name.to_string(),
                email: None,
                quota: None,
                error: Some("账号缺少 access_token".to_string()),
                fetched_at: now,
            };
        }

        // 检查 token 是否过期，如过期则刷新
        if Self::is_token_expired(&access_token) {
            match Self::try_refresh_token(&refresh_token, &auth_path, &auth_json).await {
                Ok(new_access) => access_token = new_access,
                Err(e) => {
                    return CodexAccountQuota {
                        account_name: account_name.to_string(),
                        email: Self::extract_email(&access_token),
                        quota: None,
                        error: Some(e),
                        fetched_at: now,
                    };
                }
            }
        }

        let email = Self::extract_email(&access_token);

        // 查询配额 API
        match Self::call_usage_api(&access_token, account_id.as_deref()).await {
            Ok(quota) => CodexAccountQuota {
                account_name: account_name.to_string(),
                email,
                quota: Some(quota),
                error: None,
                fetched_at: now,
            },
            Err(e) => {
                // 如果 token 失效，尝试强制刷新后重试
                if Self::should_force_refresh(&e)
                    && let Ok(new_access) =
                        Self::try_refresh_token(&refresh_token, &auth_path, &auth_json).await
                    && let Ok(quota) =
                        Self::call_usage_api(&new_access, account_id.as_deref()).await
                {
                    return CodexAccountQuota {
                        account_name: account_name.to_string(),
                        email: Self::extract_email(&new_access),
                        quota: Some(quota),
                        error: None,
                        fetched_at: now,
                    };
                }
                CodexAccountQuota {
                    account_name: account_name.to_string(),
                    email,
                    quota: None,
                    error: Some(e),
                    fetched_at: now,
                }
            }
        }
    }

    /// 并发查询所有账号的配额（semaphore 限制并发数）
    pub async fn fetch_all_quotas(&self) -> Vec<CodexAccountQuota> {
        let registry = match self.load_registry() {
            Ok(r) => r,
            Err(e) => {
                return vec![CodexAccountQuota {
                    account_name: "(registry)".to_string(),
                    email: None,
                    quota: None,
                    error: Some(format!("加载注册表失败: {}", e)),
                    fetched_at: Utc::now(),
                }];
            }
        };

        if registry.accounts.is_empty() {
            return vec![];
        }

        use futures::future::join_all;
        use std::sync::Arc;
        use tokio::sync::Semaphore;

        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
        let ccr_codex_dir = self.ccr_codex_dir.clone();

        let tasks: Vec<_> = registry
            .accounts
            .keys()
            .map(|name| {
                let semaphore = semaphore.clone();
                let dir = ccr_codex_dir.clone();
                let name = name.clone();
                async move {
                    let _permit = semaphore.acquire().await;
                    let service = CodexQuotaService { ccr_codex_dir: dir };
                    service.fetch_account_quota(&name).await
                }
            })
            .collect();

        join_all(tasks).await
    }

    // ═══════════════════════════════════════════════════════════
    // JWT 工具方法
    // ═══════════════════════════════════════════════════════════

    /// 检查 JWT access_token 是否过期
    pub fn is_token_expired(access_token: &str) -> bool {
        let parts: Vec<&str> = access_token.split('.').collect();
        if parts.len() != 3 {
            return true;
        }

        let payload = match Self::decode_base64_url(parts[1]) {
            Some(bytes) => bytes,
            None => return true,
        };

        let value: serde_json::Value = match serde_json::from_slice(&payload) {
            Ok(v) => v,
            Err(_) => return true,
        };

        let exp = match value.get("exp").and_then(|v| v.as_i64()) {
            Some(exp) => exp,
            None => return true,
        };

        // 提前 60 秒判定为过期
        exp < Utc::now().timestamp() + 60
    }

    /// 从 JWT access_token 中提取 chatgpt_account_id
    pub fn extract_account_id(access_token: &str) -> Option<String> {
        let parts: Vec<&str> = access_token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let payload = Self::decode_base64_url(parts[1])?;
        let value: serde_json::Value = serde_json::from_slice(&payload).ok()?;

        value
            .get("chatgpt_account_id")
            .or_else(|| value.get("account_id"))
            .or_else(|| {
                value
                    .get("https://api.openai.com/auth")
                    .and_then(|v| v.get("account_id"))
            })
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }

    // ═══════════════════════════════════════════════════════════
    // 格式化辅助方法
    // ═══════════════════════════════════════════════════════════

    /// 将秒数格式化为人类可读的相对时间
    pub fn format_reset_duration(reset_timestamp: i64) -> String {
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
            format!("{}d{}h", days, rem_hours)
        } else if hours > 0 {
            format!("{}h{}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }

    // ═══════════════════════════════════════════════════════════
    // 内部实现
    // ═══════════════════════════════════════════════════════════

    /// 注册表路径
    fn registry_path(&self) -> PathBuf {
        self.ccr_codex_dir.join("auth_registry.toml")
    }

    /// 账号 auth 文件路径
    fn account_auth_path(&self, name: &str) -> PathBuf {
        self.ccr_codex_dir
            .join("auth")
            .join(format!("{}.json", name))
    }

    /// 加载注册表
    fn load_registry(&self) -> Result<CodexAuthRegistry> {
        let path = self.registry_path();
        if !path.exists() {
            return Ok(CodexAuthRegistry::default());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| CcrError::ConfigError(format!("读取注册表失败: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析注册表失败: {}", e)))
    }

    /// 调用 wham/usage API
    async fn call_usage_api(
        access_token: &str,
        account_id: Option<&str>,
    ) -> std::result::Result<CodexQuota, String> {
        let client = reqwest::Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))
                .map_err(|e| format!("构建 Authorization 头失败: {}", e))?,
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        // 添加 ChatGPT-Account-Id 头（从参数或 JWT 提取）
        let effective_id = account_id
            .map(str::to_string)
            .or_else(|| Self::extract_account_id(access_token));

        if let Some(ref acc_id) = effective_id
            && !acc_id.is_empty()
            && let Ok(val) = HeaderValue::from_str(acc_id)
        {
            headers.insert("ChatGPT-Account-Id", val);
        }

        debug!(
            "Codex 配额请求: {} (account_id: {:?})",
            USAGE_URL, effective_id
        );

        let response = client
            .get(USAGE_URL)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("配额请求失败: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("读取配额响应失败: {}", e))?;

        if !status.is_success() {
            let body_preview = if body.len() > 200 {
                &body[..200]
            } else {
                &body
            };
            let error_code = Self::extract_error_code(&body);
            let mut msg = format!("API 返回错误 {}", status);
            if let Some(code) = error_code {
                msg.push_str(&format!(" [{}]", code));
            }
            msg.push_str(&format!(" - {}", body_preview));
            return Err(msg);
        }

        let usage: UsageResponse =
            serde_json::from_str(&body).map_err(|e| format!("解析配额 JSON 失败: {}", e))?;

        Self::parse_quota(&usage, &body)
    }

    /// 尝试使用 refresh_token 刷新 access_token
    async fn try_refresh_token(
        refresh_token: &Option<String>,
        auth_path: &std::path::Path,
        original_json: &str,
    ) -> std::result::Result<String, String> {
        let rt = match refresh_token {
            Some(rt) if !rt.is_empty() => rt,
            _ => return Err("Token 已过期且缺少 refresh_token".to_string()),
        };

        debug!("Codex token 已过期，正在刷新...");

        let new_tokens = Self::refresh_access_token(rt).await?;
        let new_access = new_tokens.access_token.clone();

        // 更新 auth 文件
        Self::update_auth_file(auth_path, original_json, &new_tokens);

        Ok(new_access)
    }

    /// 刷新 OAuth access_token
    async fn refresh_access_token(
        refresh_token: &str,
    ) -> std::result::Result<TokenRefreshResponse, String> {
        let client = reqwest::Client::new();

        let request = TokenRefreshRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
            client_id: OAUTH_CLIENT_ID.to_string(),
        };

        let response = client
            .post(TOKEN_REFRESH_URL)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Token 刷新请求失败: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("读取 Token 刷新响应失败: {}", e))?;

        if !status.is_success() {
            let body_preview = if body.len() > 300 {
                &body[..300]
            } else {
                &body
            };
            return Err(format!("Token 刷新失败 ({}): {}", status, body_preview));
        }

        serde_json::from_str(&body).map_err(|e| format!("解析 Token 刷新响应失败: {}", e))
    }

    /// 从 JWT 中提取邮箱
    fn extract_email(access_token: &str) -> Option<String> {
        let parts: Vec<&str> = access_token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let payload = Self::decode_base64_url(parts[1])?;
        let value: serde_json::Value = serde_json::from_slice(&payload).ok()?;

        value
            .get("email")
            .or_else(|| {
                value
                    .get("https://api.openai.com/profile")
                    .and_then(|v| v.get("email"))
            })
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }

    /// Base64 URL-safe 解码
    fn decode_base64_url(input: &str) -> Option<Vec<u8>> {
        use base64::Engine;
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;

        // 添加填充
        let padded = match input.len() % 4 {
            2 => format!("{}==", input),
            3 => format!("{}=", input),
            _ => input.to_string(),
        };

        URL_SAFE_NO_PAD.decode(&padded).ok().or_else(|| {
            use base64::engine::general_purpose::STANDARD;
            STANDARD.decode(&padded).ok()
        })
    }

    /// 解析配额响应为 CodexQuota
    fn parse_quota(
        usage: &UsageResponse,
        raw_body: &str,
    ) -> std::result::Result<CodexQuota, String> {
        let rate_limit = usage.rate_limit.as_ref();
        let primary = rate_limit.and_then(|r| r.primary_window.as_ref());
        let secondary = rate_limit.and_then(|r| r.secondary_window.as_ref());

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

        let raw_data: Option<serde_json::Value> = serde_json::from_str(raw_body).ok();

        Ok(CodexQuota {
            hourly_percentage,
            hourly_reset_time,
            hourly_window_minutes,
            hourly_window_present: Some(primary.is_some()),
            weekly_percentage,
            weekly_reset_time,
            weekly_window_minutes,
            weekly_window_present: Some(secondary.is_some()),
            plan_type: usage.plan_type.clone(),
            raw_data,
        })
    }

    /// 计算剩余百分比 (100 - used_percent)
    fn remaining_percentage(window: &WindowInfo) -> i32 {
        let used = window.used_percent.unwrap_or(0).clamp(0, 100);
        100 - used
    }

    /// 计算窗口时长（分钟）
    fn window_minutes(window: &WindowInfo) -> Option<i64> {
        let seconds = window.limit_window_seconds?;
        if seconds <= 0 {
            return None;
        }
        Some((seconds + 59) / 60)
    }

    /// 计算重置时间 (Unix timestamp)
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

    /// 从 API 错误响应中提取错误代码
    fn extract_error_code(body: &str) -> Option<String> {
        let value: serde_json::Value = serde_json::from_str(body).ok()?;
        value
            .get("detail")
            .and_then(|d| d.get("code"))
            .or_else(|| value.get("error").and_then(|e| e.get("code")))
            .or_else(|| value.get("code"))
            .and_then(|c| c.as_str())
            .map(str::to_string)
    }

    /// 判断错误是否表明需要强制刷新 token
    fn should_force_refresh(error_message: &str) -> bool {
        let lower = error_message.to_ascii_lowercase();
        lower.contains("token_invalidated")
            || lower.contains("authentication token has been invalidated")
            || lower.contains("401")
    }

    /// 更新 auth 文件中的 tokens（刷新后回写）
    fn update_auth_file(
        auth_path: &std::path::Path,
        original_json: &str,
        new_tokens: &TokenRefreshResponse,
    ) {
        let mut value: serde_json::Value = match serde_json::from_str(original_json) {
            Ok(v) => v,
            Err(e) => {
                warn!("无法解析 auth 文件以更新 tokens: {}", e);
                return;
            }
        };

        if let Some(tokens) = value.get_mut("tokens").and_then(|t| t.as_object_mut()) {
            tokens.insert(
                "access_token".to_string(),
                serde_json::Value::String(new_tokens.access_token.clone()),
            );
            if let Some(ref id_token) = new_tokens.id_token {
                tokens.insert(
                    "id_token".to_string(),
                    serde_json::Value::String(id_token.clone()),
                );
            }
            if let Some(ref refresh_token) = new_tokens.refresh_token {
                tokens.insert(
                    "refresh_token".to_string(),
                    serde_json::Value::String(refresh_token.clone()),
                );
            }
        }

        value["last_refresh"] = serde_json::Value::String(Utc::now().to_rfc3339());

        match serde_json::to_string_pretty(&value) {
            Ok(content) => {
                if let Err(e) = std::fs::write(auth_path, content) {
                    warn!("写回 auth 文件失败: {}", e);
                }
            }
            Err(e) => {
                warn!("序列化 auth 文件失败: {}", e);
            }
        }
    }
}
