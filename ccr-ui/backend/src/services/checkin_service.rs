// 签到服务
// 负责签到业务逻辑，包括执行签到、查询余额等

use crate::core::crypto::CryptoManager;
use crate::managers::checkin::{
    AccountManager, BalanceManager, ProviderManager, RecordManager, WafCookieManager,
};
use crate::models::checkin::{
    BalanceHistoryResponse, BalanceSnapshot, CheckinAccountDashboardResponse,
    CheckinDashboardAccount, CheckinDashboardCalendar, CheckinDashboardDay,
    CheckinDashboardMonthStats, CheckinDashboardStreak, CheckinDashboardTrend,
    CheckinDashboardTrendPoint, CheckinProvider, CheckinRecord, CheckinRecordsResponse,
    CheckinStatus, CookieCredentials,
};
use crate::services::waf_bypass::WafBypassService;
use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, Utc};
use once_cell::sync::Lazy;
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum CheckinServiceError {
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Account error: {0}")]
    AccountError(String),
    #[error("Crypto error: {0}")]
    CryptoError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Record error: {0}")]
    RecordError(String),
    #[error("Balance error: {0}")]
    BalanceError(String),
}

pub type Result<T> = std::result::Result<T, CheckinServiceError>;

/// new-api 标准签到响应（保留用于参考，实际使用 serde_json::Value 解析）
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NewApiCheckinResponse {
    success: Option<bool>,
    message: Option<String>,
    data: Option<serde_json::Value>,
}

/// 签到执行结果
#[derive(Debug, Clone, Serialize)]
pub struct CheckinExecutionResult {
    pub account_id: String,
    pub account_name: String,
    pub provider_name: String,
    pub status: CheckinStatus,
    pub message: Option<String>,
    pub reward: Option<String>,
    pub balance: Option<f64>,
}

/// 签到服务
pub struct CheckinService {
    /// 签到数据目录
    checkin_dir: PathBuf,
    /// HTTP 客户端
    client: Client,
    /// 统一的代理配置（保证 HTTP 请求与浏览器出口一致）
    proxy_url: Option<String>,
}

#[derive(Debug, Clone)]
struct DailySummary {
    date: NaiveDate,
    total_quota: f64,
    used_quota: f64,
    remaining_quota: f64,
}

/// 安全截断 UTF-8 字符串（避免在多字节字符中间截断导致 panic）
fn truncate_string(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

/// 默认 User-Agent
const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

/// WAF cookies 刷新锁（避免并发触发多次浏览器启动）
static WAF_REFRESH_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn get_proxy_url_from_env() -> Option<String> {
    for key in [
        "HTTPS_PROXY",
        "HTTP_PROXY",
        "ALL_PROXY",
        "https_proxy",
        "http_proxy",
        "all_proxy",
    ] {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn parse_windows_proxy_server(proxy_server: &str) -> Option<String> {
    fn normalize_http_proxy(value: &str) -> Option<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return None;
        }

        if trimmed.contains("://") {
            return Some(trimmed.to_string());
        }

        Some(format!("http://{}", trimmed))
    }

    let raw = proxy_server.trim();
    if raw.is_empty() {
        return None;
    }

    if !raw.contains(';') && !raw.contains('=') {
        return normalize_http_proxy(raw);
    }

    let mut https: Option<String> = None;
    let mut http: Option<String> = None;

    for segment in raw.split(';') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }

        let (key, value) = match segment.split_once('=') {
            Some((k, v)) => (k.trim().to_lowercase(), v.trim()),
            None => ("".to_string(), segment),
        };

        match key.as_str() {
            "https" => https = Some(value.to_string()),
            "http" | "" => http = Some(value.to_string()),
            // socks/ftp 等暂不处理（ccr-ui backend 目前未启用 reqwest socks feature）
            _ => {}
        }
    }

    https
        .as_deref()
        .and_then(normalize_http_proxy)
        .or_else(|| http.as_deref().and_then(normalize_http_proxy))
}

#[cfg(target_os = "windows")]
fn get_proxy_url_from_windows_registry() -> Option<String> {
    const KEY: &str =
        r"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Internet Settings";

    fn query_reg_value(key: &str, name: &str) -> Option<String> {
        let output = std::process::Command::new("reg")
            .args(["query", key, "/v", name])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if !line.starts_with(name) {
                continue;
            }

            if let Some(rest) = line.split("REG_DWORD").nth(1) {
                return Some(rest.trim().to_string());
            }
            if let Some(rest) = line.split("REG_SZ").nth(1) {
                return Some(rest.trim().to_string());
            }
        }

        None
    }

    let enabled = query_reg_value(KEY, "ProxyEnable")?;
    let enabled = enabled.trim().to_lowercase();
    if enabled != "0x1" && enabled != "1" {
        return None;
    }

    let proxy_server = query_reg_value(KEY, "ProxyServer")?;
    parse_windows_proxy_server(&proxy_server)
}

#[cfg(not(target_os = "windows"))]
fn get_proxy_url_from_windows_registry() -> Option<String> {
    None
}

fn get_proxy_url() -> Option<String> {
    get_proxy_url_from_env().or_else(get_proxy_url_from_windows_registry)
}

fn is_waf_challenge(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with('<')
        || trimmed.contains("acw_sc__v2")
        || trimmed.contains("<script>var arg1=")
}

fn merge_cookies(
    base: &HashMap<String, String>,
    extra: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut merged = base.clone();
    for (k, v) in extra {
        merged.insert(k.clone(), v.clone());
    }
    merged
}

fn cookie_header_string(cookies: &HashMap<String, String>) -> String {
    cookies
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ")
}

impl CheckinService {
    /// 创建新的签到服务（默认使用系统代理）
    pub fn new(checkin_dir: PathBuf) -> Self {
        let proxy_url = get_proxy_url();

        // 为保证浏览器获取的 WAF cookies 与 HTTP 请求出口一致：统一由这里决定代理，并显式注入 reqwest。
        // （Windows 上很多代理软件只写入“系统代理”，不会写入 HTTP(S)_PROXY 环境变量）
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .cookie_store(true)
            .user_agent(DEFAULT_USER_AGENT)
            // 仅使用本服务显式配置的代理，避免环境/系统代理与浏览器不一致
            .no_proxy();

        match proxy_url.as_deref() {
            Some(url) => match Proxy::all(url) {
                Ok(proxy) => {
                    tracing::info!("📡 签到服务使用代理: {}", url);
                    client_builder = client_builder.proxy(proxy);
                }
                Err(e) => tracing::warn!("📡 代理格式无效，将忽略: {} ({})", url, e),
            },
            None => tracing::debug!("📡 签到服务未检测到代理，直连模式"),
        }

        let client = client_builder
            .build()
            .expect("Failed to create HTTP client");

        Self {
            checkin_dir,
            client,
            proxy_url,
        }
    }

    /// 获取默认签到目录
    pub fn default_checkin_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            CheckinServiceError::ProviderError("Cannot find home directory".to_string())
        })?;
        Ok(home.join(".ccr").join("checkin"))
    }

    fn get_cached_waf_cookies(&self, provider_id: &str) -> Result<Option<HashMap<String, String>>> {
        let manager = WafCookieManager::new();
        manager
            .get_valid(provider_id)
            .map_err(|e| CheckinServiceError::BalanceError(e.to_string()))
    }

    async fn refresh_waf_cookies(
        &self,
        provider: &CheckinProvider,
        account_name: &str,
    ) -> Result<HashMap<String, String>> {
        let _guard = WAF_REFRESH_LOCK.lock().await;

        // 这里是“检测到 WAF 挑战页后的刷新逻辑”，必须强制刷新。
        // 否则如果缓存里的 WAF cookies 已因出口变化/失效而触发挑战页，会一直复用旧缓存导致永远绕不过去。
        let manager = WafCookieManager::new();
        let _ = manager.delete(&provider.id);

        let login_url = format!("{}/login", provider.base_url.trim_end_matches('/'));
        let waf_service =
            WafBypassService::new(true, self.proxy_url.clone(), DEFAULT_USER_AGENT.to_string());

        let waf_cookies = waf_service
            .get_waf_cookies(&login_url, account_name)
            .await
            .map_err(|e| CheckinServiceError::ApiError(format!("WAF 绕过失败: {}", e)))?;

        manager
            .save(&provider.id, waf_cookies.clone())
            .map_err(|e| CheckinServiceError::BalanceError(e.to_string()))?;

        Ok(waf_cookies)
    }

    async fn send_balance_request(
        &self,
        url: &str,
        domain: &str,
        credentials: &CookieCredentials,
        cookie_string: &str,
    ) -> Result<(reqwest::StatusCode, String)> {
        let mut request = self
            .client
            .get(url)
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", domain)
            .header("Origin", domain);

        if !cookie_string.is_empty() {
            request = request.header("Cookie", cookie_string);
        }

        if credentials.has_api_user() {
            request = request.header("new-api-user", &credentials.api_user);
        }

        let response = request
            .send()
            .await
            .map_err(|e| CheckinServiceError::NetworkError(e.to_string()))?;

        let status = response.status();
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| CheckinServiceError::NetworkError(e.to_string()))?;
        let body = String::from_utf8_lossy(&body_bytes).to_string();

        Ok((status, body))
    }

    async fn send_checkin_request(
        &self,
        url: &str,
        domain: &str,
        credentials: &CookieCredentials,
        cookie_string: &str,
    ) -> Result<(reqwest::StatusCode, String)> {
        let mut request = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/plain, */*")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", domain)
            .header("Origin", domain);

        if !cookie_string.is_empty() {
            request = request.header("Cookie", cookie_string);
        }

        if credentials.has_api_user() {
            request = request.header("new-api-user", &credentials.api_user);
        }

        let response = request
            .send()
            .await
            .map_err(|e| CheckinServiceError::NetworkError(e.to_string()))?;

        let status = response.status();
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| CheckinServiceError::NetworkError(e.to_string()))?;
        let body = String::from_utf8_lossy(&body_bytes).to_string();

        Ok((status, body))
    }

    /// 执行单个账号签到
    pub async fn checkin(&self, account_id: &str) -> Result<CheckinExecutionResult> {
        let provider_manager = ProviderManager::new();
        let account_manager = AccountManager::new(&self.checkin_dir);
        let record_manager = RecordManager::new();
        let crypto = CryptoManager::new(&self.checkin_dir)
            .map_err(|e| CheckinServiceError::CryptoError(e.to_string()))?;

        // 获取账号信息
        let account = account_manager
            .get(account_id)
            .map_err(|e| CheckinServiceError::AccountError(e.to_string()))?;

        // 获取提供商信息
        let provider = provider_manager
            .get(&account.provider_id)
            .map_err(|e| CheckinServiceError::ProviderError(e.to_string()))?;

        // 检查今日是否已签到
        let already_checked = record_manager
            .has_checked_in_today(account_id)
            .map_err(|e| CheckinServiceError::RecordError(e.to_string()))?;

        if already_checked {
            let record = CheckinRecord::already_checked_in(
                account_id.to_string(),
                Some("今日已签到".to_string()),
            );
            record_manager
                .add(record)
                .map_err(|e| CheckinServiceError::RecordError(e.to_string()))?;

            return Ok(CheckinExecutionResult {
                account_id: account_id.to_string(),
                account_name: account.name.clone(),
                provider_name: provider.name.clone(),
                status: CheckinStatus::AlreadyCheckedIn,
                message: Some("今日已签到".to_string()),
                reward: None,
                balance: None,
            });
        }

        // 解密 Cookies JSON 并创建凭证
        let cookies_json = crypto
            .decrypt(&account.cookies_json_encrypted)
            .map_err(|e| CheckinServiceError::CryptoError(e.to_string()))?;

        let credentials = CookieCredentials::from_json(&cookies_json, account.api_user.clone())
            .map_err(|e| {
                CheckinServiceError::CryptoError(format!("Invalid cookies JSON: {}", e))
            })?;

        // 执行签到请求
        let checkin_result = self
            .do_checkin(&provider, &credentials, &account.name)
            .await;

        // 记录签到结果
        let (record, result) = match checkin_result {
            Ok((message, reward)) => {
                let record = CheckinRecord::success(
                    account_id.to_string(),
                    Some(message.clone()),
                    reward.clone(),
                );

                let result = CheckinExecutionResult {
                    account_id: account_id.to_string(),
                    account_name: account.name.clone(),
                    provider_name: provider.name.clone(),
                    status: CheckinStatus::Success,
                    message: Some(message),
                    reward,
                    balance: None,
                };

                (record, result)
            }
            Err(e) => {
                let error_msg = e.to_string();
                let record = CheckinRecord::failed(account_id.to_string(), error_msg.clone());

                let result = CheckinExecutionResult {
                    account_id: account_id.to_string(),
                    account_name: account.name.clone(),
                    provider_name: provider.name.clone(),
                    status: CheckinStatus::Failed,
                    message: Some(error_msg),
                    reward: None,
                    balance: None,
                };

                (record, result)
            }
        };

        // 保存签到记录
        record_manager
            .add(record)
            .map_err(|e| CheckinServiceError::RecordError(e.to_string()))?;

        // 更新账号最后签到时间
        let _ = account_manager.update_checkin_time(account_id);

        Ok(result)
    }

    /// 执行签到 HTTP 请求（使用 Cookie 认证）
    async fn do_checkin(
        &self,
        provider: &CheckinProvider,
        credentials: &CookieCredentials,
        account_name: &str,
    ) -> Result<(String, Option<String>)> {
        let url = format!(
            "{}{}",
            provider.base_url.trim_end_matches('/'),
            provider.checkin_path
        );

        let domain = provider.base_url.trim_end_matches('/');

        let mut cookies = credentials.cookies.clone();
        if let Some(waf_cookies) = self.get_cached_waf_cookies(&provider.id)? {
            cookies = merge_cookies(&cookies, &waf_cookies);
        }
        let mut cookie_string = cookie_header_string(&cookies);

        let (mut status, mut body) = self
            .send_checkin_request(&url, domain, credentials, &cookie_string)
            .await?;

        tracing::info!("Checkin response status: {}", status);
        tracing::info!("Checkin response body: {}", truncate_string(&body, 500));

        // 检测 WAF 挑战页面：自动刷新 WAF cookies 后重试一次
        if is_waf_challenge(&body) {
            tracing::warn!(
                "[{}] Detected WAF challenge, attempting auto bypass...",
                account_name
            );

            let waf_cookies = self.refresh_waf_cookies(provider, account_name).await?;
            let merged = merge_cookies(&credentials.cookies, &waf_cookies);
            cookie_string = cookie_header_string(&merged);

            let (retry_status, retry_body) = self
                .send_checkin_request(&url, domain, credentials, &cookie_string)
                .await?;

            status = retry_status;
            body = retry_body;

            tracing::info!("Checkin retry status: {}", status);
            tracing::info!("Checkin retry response: {}", truncate_string(&body, 500));
        }

        if !status.is_success() {
            if is_waf_challenge(&body) {
                return Err(CheckinServiceError::ApiError(
                    "检测到 WAF 挑战页面，已尝试自动获取 WAF cookies 但仍失败。请检查代理/出口是否一致，或稍后重试。"
                        .to_string(),
                ));
            }

            return Err(CheckinServiceError::ApiError(format!(
                "HTTP {}: {}",
                status.as_u16(),
                truncate_string(&body, 200)
            )));
        }

        if is_waf_challenge(&body) {
            return Err(CheckinServiceError::ApiError(
                "检测到 WAF 挑战页面，已尝试自动获取 WAF cookies 但仍失败。请检查代理/出口是否一致，或稍后重试。"
                    .to_string(),
            ));
        }

        // 尝试解析 JSON 响应（支持多种 API 格式）
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
            tracing::debug!(
                "Parsed JSON response: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );

            // 检查成功标识（支持多种格式，参考 NeuraDock）
            let ret_value = data["ret"].as_i64();
            let code_value = data["code"].as_i64();
            let success_value = data["success"].as_bool();

            tracing::debug!(
                "Success indicators - ret: {:?}, code: {:?}, success: {:?}",
                ret_value,
                code_value,
                success_value
            );

            // 判断是否成功
            let success = ret_value == Some(1)
                || code_value == Some(0)
                || code_value == Some(200)
                || success_value == Some(true);

            // 提取消息（支持多种字段名）
            let message = if success {
                data["msg"]
                    .as_str()
                    .or(data["message"].as_str())
                    .or(data["data"].as_str())
                    .unwrap_or("签到成功")
                    .to_string()
            } else {
                data["msg"]
                    .as_str()
                    .or(data["message"].as_str())
                    .or(data["error"].as_str())
                    .unwrap_or("签到失败")
                    .to_string()
            };

            // 检查是否是"已签到"的情况
            if !success && (message.contains("已") || message.contains("already")) {
                return Ok((message, None));
            }

            if !success {
                return Err(CheckinServiceError::ApiError(message));
            }

            // 尝试从 data 中提取奖励信息
            let reward = data["data"].as_object().and_then(|d| {
                if let Some(reward_str) = d.get("reward").and_then(|v| v.as_str()) {
                    Some(reward_str.to_string())
                } else {
                    d.get("points")
                        .and_then(|v| v.as_i64())
                        .map(|points| format!("+{} 积分", points))
                }
            });

            Ok((message, reward))
        } else {
            tracing::warn!("Failed to parse as JSON, raw response: {}", body);

            // 如果不是 JSON，检查响应是否包含成功标识
            if body.to_lowercase().contains("success") || body.contains("成功") {
                Ok(("签到成功".to_string(), None))
            } else {
                // 返回原始响应作为错误信息
                Err(CheckinServiceError::ApiError(format!(
                    "无法解析响应: {}",
                    truncate_string(&body, 100)
                )))
            }
        }
    }

    /// 查询账号余额
    pub async fn query_balance(&self, account_id: &str) -> Result<BalanceSnapshot> {
        let provider_manager = ProviderManager::new();
        let account_manager = AccountManager::new(&self.checkin_dir);
        let balance_manager = BalanceManager::new();
        let crypto = CryptoManager::new(&self.checkin_dir)
            .map_err(|e| CheckinServiceError::CryptoError(e.to_string()))?;

        // 获取账号信息
        let account = account_manager
            .get(account_id)
            .map_err(|e| CheckinServiceError::AccountError(e.to_string()))?;

        // 获取提供商信息
        let provider = provider_manager
            .get(&account.provider_id)
            .map_err(|e| CheckinServiceError::ProviderError(e.to_string()))?;

        // 解密 Cookies JSON 并创建凭证
        let cookies_json = crypto
            .decrypt(&account.cookies_json_encrypted)
            .map_err(|e| CheckinServiceError::CryptoError(e.to_string()))?;

        let credentials = CookieCredentials::from_json(&cookies_json, account.api_user.clone())
            .map_err(|e| {
                CheckinServiceError::CryptoError(format!("Invalid cookies JSON: {}", e))
            })?;

        // 查询余额
        let snapshot = self
            .do_query_balance(&provider, &credentials, account_id, &account.name)
            .await?;

        // 保存余额快照
        balance_manager
            .add(snapshot.clone())
            .map_err(|e| CheckinServiceError::BalanceError(e.to_string()))?;

        // 更新账号最后余额查询时间
        let _ = account_manager.update_balance_time(account_id);

        Ok(snapshot)
    }

    /// 执行余额查询 HTTP 请求（使用 Cookie 认证）
    async fn do_query_balance(
        &self,
        provider: &CheckinProvider,
        credentials: &CookieCredentials,
        account_id: &str,
        account_name: &str,
    ) -> Result<BalanceSnapshot> {
        let url = format!(
            "{}{}",
            provider.base_url.trim_end_matches('/'),
            provider.balance_path
        );

        let domain = provider.base_url.trim_end_matches('/');

        tracing::debug!("Querying balance for account {}: {}", account_id, url);

        let mut cookies = credentials.cookies.clone();
        if let Some(waf_cookies) = self.get_cached_waf_cookies(&provider.id)? {
            cookies = merge_cookies(&cookies, &waf_cookies);
        }
        let mut cookie_string = cookie_header_string(&cookies);

        let (mut status, mut body) = self
            .send_balance_request(&url, domain, credentials, &cookie_string)
            .await?;

        tracing::info!("Balance query response status: {}", status);
        tracing::info!("Balance query response: {}", truncate_string(&body, 500));

        // 检测 WAF 挑战页面：自动刷新 WAF cookies 后重试一次
        if is_waf_challenge(&body) {
            tracing::warn!(
                "[{}] Detected WAF challenge, attempting auto bypass...",
                account_name
            );

            let waf_cookies = self.refresh_waf_cookies(provider, account_name).await?;
            let merged = merge_cookies(&credentials.cookies, &waf_cookies);
            cookie_string = cookie_header_string(&merged);

            let (retry_status, retry_body) = self
                .send_balance_request(&url, domain, credentials, &cookie_string)
                .await?;

            status = retry_status;
            body = retry_body;

            tracing::info!("Balance query retry status: {}", status);
            tracing::info!(
                "Balance query retry response: {}",
                truncate_string(&body, 500)
            );
        }

        if !status.is_success() {
            if is_waf_challenge(&body) {
                return Err(CheckinServiceError::ApiError(
                    "检测到 WAF 挑战页面，已尝试自动获取 WAF cookies 但仍失败。请检查代理/出口是否一致，或稍后重试。"
                        .to_string(),
                ));
            }

            return Err(CheckinServiceError::ApiError(format!(
                "HTTP {}: {}",
                status.as_u16(),
                truncate_string(&body, 200)
            )));
        }

        if is_waf_challenge(&body) {
            return Err(CheckinServiceError::ApiError(
                "检测到 WAF 挑战页面，已尝试自动获取 WAF cookies 但仍失败。请检查代理/出口是否一致，或稍后重试。"
                    .to_string(),
            ));
        }

        // 使用 serde_json::Value 灵活解析（参考 NeuraDock）
        let data: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            CheckinServiceError::ApiError(format!(
                "无法解析余额响应: {} - {}",
                e,
                truncate_string(&body, 200)
            ))
        })?;

        tracing::debug!(
            "Parsed balance response: {}",
            serde_json::to_string_pretty(&data).unwrap_or_default()
        );

        // 参考 NeuraDock: 先检查 data 字段是否存在
        if data["data"].is_null() {
            // 检查是否有错误信息
            let error_msg = data["message"]
                .as_str()
                .or_else(|| data["msg"].as_str())
                .unwrap_or("API 响应缺少 'data' 字段");
            return Err(CheckinServiceError::ApiError(format!(
                "{}: {}",
                error_msg,
                truncate_string(&body, 200)
            )));
        }

        // 参考 NeuraDock: 使用 ok_or_else 返回明确的错误信息
        // 提取 quota 和 used_quota（支持 data.quota 或直接 quota 两种格式）
        let quota_bytes = data["data"]["quota"]
            .as_f64()
            .or_else(|| data["quota"].as_f64());

        let used_quota_bytes = data["data"]["used_quota"]
            .as_f64()
            .or_else(|| data["used_quota"].as_f64());

        if let (Some(quota), Some(used_quota)) = (quota_bytes, used_quota_bytes) {
            // quota 和 used_quota 是 token 数量，转换为金额 (500000 tokens = $1)
            let quota_rate = 500000.0;
            let current_balance = (quota / quota_rate * 100.0).round() / 100.0;
            let total_consumed = (used_quota / quota_rate * 100.0).round() / 100.0;
            let total_quota = current_balance + total_consumed;

            return Ok(BalanceSnapshot::new(
                account_id.to_string(),
                total_quota,
                total_consumed,
                current_balance,
                "USD".to_string(),
            )
            .with_raw_response(body));
        }

        // 尝试从其他字段获取余额信息
        if let Some(balance) = data["data"]["balance"]
            .as_f64()
            .or(data["balance"].as_f64())
        {
            return Ok(BalanceSnapshot::new(
                account_id.to_string(),
                balance,
                0.0,
                balance,
                "USD".to_string(),
            )
            .with_raw_response(body));
        }

        // 无法解析 - 提供更详细的错误信息
        let available_fields: Vec<&str> = data["data"]
            .as_object()
            .map(|obj| obj.keys().map(|k| k.as_str()).collect())
            .unwrap_or_default();

        Err(CheckinServiceError::ApiError(format!(
            "无法解析余额响应，缺少 quota/used_quota 字段。可用字段: {:?}，响应: {}",
            available_fields,
            truncate_string(&body, 200)
        )))
    }

    /// 批量签到
    pub async fn batch_checkin(&self, account_ids: &[String]) -> Vec<CheckinExecutionResult> {
        let mut results = Vec::with_capacity(account_ids.len());

        for account_id in account_ids {
            let result = self.checkin(account_id).await;
            match result {
                Ok(r) => results.push(r),
                Err(e) => {
                    results.push(CheckinExecutionResult {
                        account_id: account_id.clone(),
                        account_name: "Unknown".to_string(),
                        provider_name: "Unknown".to_string(),
                        status: CheckinStatus::Failed,
                        message: Some(e.to_string()),
                        reward: None,
                        balance: None,
                    });
                }
            }
        }

        results
    }

    /// 签到所有启用的账号
    pub async fn checkin_all(&self) -> Vec<CheckinExecutionResult> {
        let account_manager = AccountManager::new(&self.checkin_dir);

        let enabled_accounts = match account_manager.get_enabled_accounts() {
            Ok(accounts) => accounts,
            Err(e) => {
                tracing::error!("Failed to get enabled accounts: {}", e);
                return vec![];
            }
        };

        let account_ids: Vec<String> = enabled_accounts.iter().map(|a| a.id.clone()).collect();
        self.batch_checkin(&account_ids).await
    }

    /// 获取账号签到记录
    #[allow(dead_code)]
    pub fn get_checkin_records(
        &self,
        account_id: &str,
        limit: Option<usize>,
    ) -> Result<CheckinRecordsResponse> {
        let record_manager = RecordManager::new();
        record_manager
            .get_by_account(account_id, limit)
            .map_err(|e| CheckinServiceError::RecordError(e.to_string()))
    }

    /// 获取所有签到记录
    #[allow(dead_code)]
    pub fn get_all_records(&self, limit: Option<usize>) -> Result<CheckinRecordsResponse> {
        let record_manager = RecordManager::new();
        record_manager
            .get_all(limit)
            .map_err(|e| CheckinServiceError::RecordError(e.to_string()))
    }

    /// 获取账号余额历史
    pub fn get_balance_history(
        &self,
        account_id: &str,
        limit: Option<usize>,
    ) -> Result<BalanceHistoryResponse> {
        let balance_manager = BalanceManager::new();
        balance_manager
            .get_history(account_id, limit)
            .map_err(|e| CheckinServiceError::BalanceError(e.to_string()))
    }

    /// 获取账号最新余额
    #[allow(dead_code)]
    pub fn get_latest_balance(&self, account_id: &str) -> Result<Option<BalanceSnapshot>> {
        let balance_manager = BalanceManager::new();
        balance_manager
            .get_latest(account_id)
            .map_err(|e| CheckinServiceError::BalanceError(e.to_string()))
    }

    /// 获取账号 Dashboard 聚合数据
    pub fn get_account_dashboard(
        &self,
        account_id: &str,
        year: i32,
        month: u32,
        days: u32,
    ) -> Result<CheckinAccountDashboardResponse> {
        let account_manager = AccountManager::new(&self.checkin_dir);
        let provider_manager = ProviderManager::new();
        let balance_manager = BalanceManager::new();

        let account = account_manager
            .get(account_id)
            .map_err(|e| CheckinServiceError::AccountError(e.to_string()))?;

        let provider = provider_manager
            .get(&account.provider_id)
            .map_err(|e| CheckinServiceError::ProviderError(e.to_string()))?;

        let latest_balance = balance_manager
            .get_latest(account_id)
            .map_err(|e| CheckinServiceError::BalanceError(e.to_string()))?;

        let dashboard_account = CheckinDashboardAccount {
            id: account.id.clone(),
            name: account.name.clone(),
            provider_id: account.provider_id.clone(),
            provider_name: provider.name.clone(),
            enabled: account.enabled,
            last_checkin_at: account.last_checkin_at,
            last_balance_check_at: account.last_balance_check_at,
            latest_balance: latest_balance.as_ref().map(|s| s.remaining_quota),
            balance_currency: latest_balance.as_ref().map(|s| s.currency.clone()),
            total_quota: latest_balance.as_ref().map(|s| s.total_quota),
            used_quota: latest_balance.as_ref().map(|s| s.used_quota),
            remaining_quota: latest_balance.as_ref().map(|s| s.remaining_quota),
        };

        let snapshots = balance_manager
            .list_by_account(account_id)
            .map_err(|e| CheckinServiceError::BalanceError(e.to_string()))?;

        let daily_summaries = build_daily_summaries(&snapshots);
        let streak = compute_streak(&daily_summaries);
        let calendar = build_calendar(account_id, year, month, &daily_summaries)?;
        let trend = build_trend(account_id, days, &daily_summaries)?;

        Ok(CheckinAccountDashboardResponse {
            account: dashboard_account,
            streak,
            calendar,
            trend,
        })
    }

    /// 获取今日签到统计
    pub fn get_today_stats(&self) -> Result<TodayCheckinStats> {
        let account_manager = AccountManager::new(&self.checkin_dir);
        let record_manager = RecordManager::new();

        let all_accounts = account_manager
            .load_all()
            .map_err(|e| CheckinServiceError::AccountError(e.to_string()))?;

        let enabled_accounts: Vec<_> = all_accounts.iter().filter(|a| a.enabled).collect();
        let account_ids: Vec<String> = enabled_accounts.iter().map(|a| a.id.clone()).collect();

        let stats = record_manager
            .get_today_stats(&account_ids)
            .map_err(|e| CheckinServiceError::RecordError(e.to_string()))?;

        Ok(TodayCheckinStats {
            total_accounts: stats.total,
            checked_in: stats.checked_in,
            not_checked_in: stats.not_checked_in,
            failed: stats.failed,
        })
    }

    /// 测试账号连接
    pub async fn test_connection(&self, account_id: &str) -> Result<bool> {
        let provider_manager = ProviderManager::new();
        let account_manager = AccountManager::new(&self.checkin_dir);
        let crypto = CryptoManager::new(&self.checkin_dir)
            .map_err(|e| CheckinServiceError::CryptoError(e.to_string()))?;

        // 获取账号信息
        let account = account_manager
            .get(account_id)
            .map_err(|e| CheckinServiceError::AccountError(e.to_string()))?;

        // 获取提供商信息
        let provider = provider_manager
            .get(&account.provider_id)
            .map_err(|e| CheckinServiceError::ProviderError(e.to_string()))?;

        // 解密 Cookies JSON 并创建凭证
        let cookies_json = crypto
            .decrypt(&account.cookies_json_encrypted)
            .map_err(|e| CheckinServiceError::CryptoError(e.to_string()))?;

        let credentials = CookieCredentials::from_json(&cookies_json, account.api_user.clone())
            .map_err(|e| {
                CheckinServiceError::CryptoError(format!("Invalid cookies JSON: {}", e))
            })?;

        // 使用 user_info_path 测试连接
        let url = format!(
            "{}{}",
            provider.base_url.trim_end_matches('/'),
            provider.user_info_path
        );

        let domain = provider.base_url.trim_end_matches('/');

        let mut cookies = credentials.cookies.clone();
        if let Some(waf_cookies) = self.get_cached_waf_cookies(&provider.id)? {
            cookies = merge_cookies(&cookies, &waf_cookies);
        }
        let mut cookie_string = cookie_header_string(&cookies);

        let (mut status, mut body) = self
            .send_balance_request(&url, domain, &credentials, &cookie_string)
            .await?;

        if is_waf_challenge(&body) {
            let waf_cookies = self.refresh_waf_cookies(&provider, &account.name).await?;
            let merged = merge_cookies(&credentials.cookies, &waf_cookies);
            cookie_string = cookie_header_string(&merged);

            let (retry_status, retry_body) = self
                .send_balance_request(&url, domain, &credentials, &cookie_string)
                .await?;

            status = retry_status;
            body = retry_body;
        }

        Ok(status.is_success() && !is_waf_challenge(&body))
    }
}

fn build_daily_summaries(snapshots: &[BalanceSnapshot]) -> Vec<DailySummary> {
    let mut latest_by_day: HashMap<NaiveDate, BalanceSnapshot> = HashMap::new();

    for snapshot in snapshots {
        let date = snapshot.recorded_at.date_naive();
        let replace = match latest_by_day.get(&date) {
            Some(existing) => snapshot.recorded_at > existing.recorded_at,
            None => true,
        };

        if replace {
            latest_by_day.insert(date, snapshot.clone());
        }
    }

    let mut daily: Vec<DailySummary> = latest_by_day
        .into_iter()
        .map(|(date, snapshot)| DailySummary {
            date,
            total_quota: snapshot.total_quota,
            used_quota: snapshot.used_quota,
            remaining_quota: snapshot.remaining_quota,
        })
        .collect();

    daily.sort_by(|a, b| a.date.cmp(&b.date));
    daily
}

fn compute_streak(daily: &[DailySummary]) -> CheckinDashboardStreak {
    let mut prev_total: Option<f64> = None;
    let mut current_streak = 0u32;
    let mut longest_streak = 0u32;
    let mut total_check_in_days = 0u32;
    let mut last_check_in_date: Option<NaiveDate> = None;

    for day in daily {
        let is_checked_in = prev_total.is_none_or(|prev| day.total_quota > prev);

        if is_checked_in {
            current_streak = match last_check_in_date {
                Some(prev_date) if day.date.signed_duration_since(prev_date).num_days() == 1 => {
                    if current_streak == 0 {
                        1
                    } else {
                        current_streak + 1
                    }
                }
                _ => 1,
            };

            longest_streak = longest_streak.max(current_streak);
            total_check_in_days += 1;
            last_check_in_date = Some(day.date);
        } else if let Some(prev_date) = last_check_in_date
            && day.date.signed_duration_since(prev_date).num_days() > 1
        {
            current_streak = 0;
        }

        prev_total = Some(day.total_quota);
    }

    CheckinDashboardStreak {
        current_streak,
        longest_streak,
        total_check_in_days,
        last_check_in_date: last_check_in_date.map(|d| d.format("%Y-%m-%d").to_string()),
    }
}

fn build_calendar(
    account_id: &str,
    year: i32,
    month: u32,
    daily: &[DailySummary],
) -> Result<CheckinDashboardCalendar> {
    let _first_day = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| CheckinServiceError::ApiError("Invalid month".to_string()))?;

    let first_day_next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };

    let last_day = first_day_next_month
        .and_then(|d| d.pred_opt())
        .ok_or_else(|| CheckinServiceError::ApiError("Invalid month".to_string()))?;

    let total_days = last_day.day();
    let mut daily_map: HashMap<NaiveDate, &DailySummary> = HashMap::new();
    for item in daily {
        daily_map.insert(item.date, item);
    }

    let mut days = Vec::new();
    let mut prev_total: Option<f64> = None;
    let mut checked_in_days = 0u32;
    let mut total_quota_increment = 0.0;

    for day in 1..=total_days {
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| CheckinServiceError::ApiError("Invalid date".to_string()))?;
        let date_str = date.format("%Y-%m-%d").to_string();

        if let Some(summary) = daily_map.get(&date) {
            let income_increment = prev_total.and_then(|prev| {
                let diff = summary.total_quota - prev;
                if diff > 0.0 { Some(diff) } else { None }
            });

            let is_checked_in = income_increment.is_some() || prev_total.is_none();

            if is_checked_in {
                checked_in_days += 1;
                if let Some(inc) = income_increment {
                    total_quota_increment += inc;
                } else if prev_total.is_none() && summary.total_quota > 0.0 {
                    total_quota_increment += summary.total_quota;
                }
            }

            days.push(CheckinDashboardDay {
                date: date_str,
                is_checked_in,
                income_increment,
                current_balance: summary.remaining_quota,
                total_consumed: summary.used_quota,
                total_quota: summary.total_quota,
            });

            prev_total = Some(summary.total_quota);
        } else {
            days.push(CheckinDashboardDay {
                date: date_str,
                is_checked_in: false,
                income_increment: None,
                current_balance: 0.0,
                total_consumed: 0.0,
                total_quota: 0.0,
            });
        }
    }

    let check_in_rate = if total_days > 0 {
        (checked_in_days as f64 / total_days as f64) * 100.0
    } else {
        0.0
    };

    Ok(CheckinDashboardCalendar {
        account_id: account_id.to_string(),
        year,
        month,
        days,
        month_stats: CheckinDashboardMonthStats {
            total_days,
            checked_in_days,
            check_in_rate,
            total_quota_increment,
        },
    })
}

fn build_trend(
    account_id: &str,
    days: u32,
    daily: &[DailySummary],
) -> Result<CheckinDashboardTrend> {
    if days == 0 || days > 365 {
        return Err(CheckinServiceError::ApiError(
            "Days must be between 1 and 365".to_string(),
        ));
    }

    let end_date = Utc::now().date_naive();
    let start_date = end_date - ChronoDuration::days(days as i64 - 1);

    let mut data_points = Vec::new();
    let mut prev_total: Option<f64> = None;

    for item in daily
        .iter()
        .filter(|d| d.date >= start_date && d.date <= end_date)
    {
        let income_increment = prev_total.map_or(0.0, |prev| {
            let diff = item.total_quota - prev;
            if diff > 0.0 { diff } else { 0.0 }
        });

        let is_checked_in = income_increment > 0.0 || prev_total.is_none();

        data_points.push(CheckinDashboardTrendPoint {
            date: item.date.format("%Y-%m-%d").to_string(),
            total_quota: item.total_quota,
            income_increment,
            current_balance: item.remaining_quota,
            is_checked_in,
        });

        prev_total = Some(item.total_quota);
    }

    Ok(CheckinDashboardTrend {
        account_id: account_id.to_string(),
        start_date: start_date.format("%Y-%m-%d").to_string(),
        end_date: end_date.format("%Y-%m-%d").to_string(),
        data_points,
    })
}

/// 今日签到统计
#[derive(Debug, Clone, Serialize)]
pub struct TodayCheckinStats {
    /// 总账号数
    pub total_accounts: usize,
    /// 今日已签到数
    pub checked_in: usize,
    /// 今日未签到数
    pub not_checked_in: usize,
    /// 今日签到失败数
    pub failed: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::database;
    use tempfile::TempDir;

    fn setup() -> (TempDir, CheckinService) {
        // Initialize in-memory database for tests
        database::initialize_for_test().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let service = CheckinService::new(temp_dir.path().to_path_buf());
        (temp_dir, service)
    }

    #[test]
    fn test_default_checkin_dir() {
        let dir = CheckinService::default_checkin_dir();
        assert!(dir.is_ok());
        let path = dir.unwrap();
        assert!(path.ends_with("checkin"));
    }

    #[test]
    fn test_get_today_stats_empty() {
        let (_temp_dir, service) = setup();
        let stats = service.get_today_stats().unwrap();
        assert_eq!(stats.total_accounts, 0);
        assert_eq!(stats.checked_in, 0);
    }
}
