// 签到服务
// 负责签到业务逻辑，包括执行签到、查询余额等

use crate::core::crypto::CryptoManager;
use crate::core::error::CheckinServiceError;
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
use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, Utc};
use once_cell::sync::Lazy;
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::Mutex;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
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
pub(crate) const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

/// WAF cookies 刷新锁（避免并发触发多次浏览器启动）
static WAF_REFRESH_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// CF cookies 刷新锁（避免并发触发多次浏览器启动）
static CF_REFRESH_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

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
    // 阿里云 WAF 特征检测
    text.contains("acw_sc__v2")
        || text.contains("<script>var arg1=")
        || text.contains("anti_spider")
        || text.contains("acw_tc")
}

/// 检测 Cloudflare 挑战页面
fn is_cf_challenge(status: reqwest::StatusCode, body: &str) -> bool {
    let is_cf_status =
        status == reqwest::StatusCode::FORBIDDEN || status.as_u16() == 503 || !status.is_success();
    let has_cf_markers = body.contains("Just a moment")
        || body.contains("cf-browser-verification")
        || body.contains("_cf_chl")
        || body.contains("cf-challenge-running")
        || body.contains("cf_clearance");
    is_cf_status && has_cf_markers
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
    #[allow(dead_code)]
    pub fn new(checkin_dir: PathBuf) -> Self {
        let proxy_url = get_proxy_url();

        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .cookie_store(true)
            .user_agent(DEFAULT_USER_AGENT)
            .no_proxy();

        match proxy_url.as_deref() {
            Some(url) => match Proxy::all(url) {
                Ok(proxy) => {
                    tracing::info!("签到服务使用代理: {}", url);
                    client_builder = client_builder.proxy(proxy);
                }
                Err(e) => tracing::warn!("代理格式无效，将忽略: {} ({})", url, e),
            },
            None => tracing::debug!("签到服务未检测到代理，直连模式"),
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

    /// 使用共享的 HTTP 客户端创建签到服务
    pub fn with_client(checkin_dir: PathBuf, client: Client) -> Self {
        let proxy_url = get_proxy_url();
        Self {
            checkin_dir,
            client,
            proxy_url,
        }
    }

    /// 获取默认签到目录
    pub fn default_checkin_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            CheckinServiceError::Provider("Cannot find home directory".to_string())
        })?;
        Ok(home.join(".ccr").join("checkin"))
    }

    fn get_cached_waf_cookies(&self, provider_id: &str) -> Result<Option<HashMap<String, String>>> {
        let manager = WafCookieManager::new();
        manager
            .get_valid(provider_id)
            .map_err(|e| CheckinServiceError::Balance(e.to_string()))
    }

    async fn refresh_waf_cookies(
        &self,
        provider: &CheckinProvider,
        _account_name: &str,
    ) -> Result<HashMap<String, String>> {
        let _guard = WAF_REFRESH_LOCK.lock().await;

        let manager = WafCookieManager::new();
        let _ = manager.delete(&provider.id);

        // WAF bypass 尚未在当前版本实现，返回错误让调用方优雅降级
        Err(CheckinServiceError::Api(
            "WAF 绕过功能尚未在当前版本实现，请检查是否有可用的缓存 WAF cookies".to_string(),
        ))
    }

    /// CF cookies 缓存 key：使用 `cf-` 前缀区分 WAF cookies
    fn cf_cache_key(provider_id: &str) -> String {
        format!("cf-{}", provider_id)
    }

    fn get_cached_cf_cookies(&self, provider_id: &str) -> Result<Option<HashMap<String, String>>> {
        let manager = WafCookieManager::new();
        manager
            .get_valid(&Self::cf_cache_key(provider_id))
            .map_err(|e| CheckinServiceError::Balance(e.to_string()))
    }

    async fn refresh_cf_cookies(
        &self,
        provider: &CheckinProvider,
        _account_name: &str,
    ) -> Result<HashMap<String, String>> {
        let _guard = CF_REFRESH_LOCK.lock().await;

        let manager = WafCookieManager::new();
        let cache_key = Self::cf_cache_key(&provider.id);
        let _ = manager.delete(&cache_key);

        // Cloudflare 绕过尚未在当前版本实现，返回错误让调用方优雅降级
        Err(CheckinServiceError::Api(
            "Cloudflare 绕过功能尚未在当前版本实现，请在有 GUI 的环境中手动获取 cf_clearance".to_string(),
        ))
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
            .map_err(|e| CheckinServiceError::Network(e.to_string()))?;

        let status = response.status();
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| CheckinServiceError::Network(e.to_string()))?;
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
            .map_err(|e| CheckinServiceError::Network(e.to_string()))?;

        let status = response.status();
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| CheckinServiceError::Network(e.to_string()))?;
        let body = String::from_utf8_lossy(&body_bytes).to_string();

        Ok((status, body))
    }

    /// 执行单个账号签到
    pub async fn checkin(&self, account_id: &str) -> Result<CheckinExecutionResult> {
        let provider_manager = ProviderManager::new();
        let account_manager = AccountManager::new(&self.checkin_dir);
        let record_manager = RecordManager::new();
        let crypto = CryptoManager::new(&self.checkin_dir)
            .map_err(|e| CheckinServiceError::Crypto(e.to_string()))?;

        // 获取账号信息
        let account = account_manager
            .get(account_id)
            .map_err(|e| CheckinServiceError::Account(e.to_string()))?;

        // 获取提供商信息
        let provider = provider_manager
            .get(&account.provider_id)
            .map_err(|e| CheckinServiceError::Provider(e.to_string()))?;

        tracing::info!(
            "[签到开始] 账号: {} | 提供商: {} | ID: {}",
            account.name,
            provider.name,
            account_id
        );

        // 检查今日是否已签到
        let already_checked = record_manager
            .has_checked_in_today(account_id)
            .map_err(|e| CheckinServiceError::Record(e.to_string()))?;

        if already_checked {
            tracing::info!(
                "[已签到] 账号: {} | 提供商: {} | 状态: 今日已签到，跳过",
                account.name,
                provider.name
            );

            let record = CheckinRecord::already_checked_in(
                account_id.to_string(),
                Some("今日已签到".to_string()),
            );
            record_manager
                .add(record)
                .map_err(|e| CheckinServiceError::Record(e.to_string()))?;

            return Ok(CheckinExecutionResult {
                account_id: account_id.to_string(),
                account_name: account.name.clone(),
                provider_name: provider.name.clone(),
                status: CheckinStatus::AlreadyCheckedIn,
                message: Some("今日已签到".to_string()),
                error_code: None,
                reward: None,
                balance: None,
            });
        }

        // 解密 Cookies JSON 并创建凭证
        let cookies_json = crypto
            .decrypt(&account.cookies_json_encrypted)
            .map_err(|e| CheckinServiceError::Crypto(e.to_string()))?;

        let credentials = CookieCredentials::from_json(&cookies_json, account.api_user.clone())
            .map_err(|e| CheckinServiceError::Crypto(format!("Invalid cookies JSON: {}", e)))?;

        // 签到前远程状态预查：通过 /api/user/self 检查是否已签到
        if let Some(true) = self
            .check_remote_checkin_status(&provider, &credentials, &account.name)
            .await
        {
            tracing::info!(
                "[远程预查] 账号: {} | 提供商: {} | 状态: 远程已签到，跳过",
                account.name,
                provider.name
            );

            let record = CheckinRecord::already_checked_in(
                account_id.to_string(),
                Some("今日已签到（远程预查）".to_string()),
            );
            record_manager
                .add(record)
                .map_err(|e| CheckinServiceError::Record(e.to_string()))?;

            // 更新签到时间
            let _ = account_manager.update_checkin_time(account_id);

            let result = CheckinExecutionResult {
                account_id: account_id.to_string(),
                account_name: account.name.clone(),
                provider_name: provider.name.clone(),
                status: CheckinStatus::AlreadyCheckedIn,
                message: Some("今日已签到（远程预查）".to_string()),
                error_code: None,
                reward: None,
                balance: None,
            };

            // 即使已签到，仍尝试 CDK 充值
            self.try_cdk_topup(&provider, &account, &cookies_json).await;

            return Ok(result);
        }

        // 执行签到请求
        let checkin_result = self
            .do_checkin(&provider, &credentials, &account.name)
            .await;

        // 记录签到结果
        let (record, result) = match checkin_result {
            Ok((message, reward)) => {
                // 检查 do_checkin 返回的"已签到"标记
                let (actual_status, actual_message) =
                    if let Some(stripped) = message.strip_prefix("[ALREADY_CHECKED_IN]") {
                        (CheckinStatus::AlreadyCheckedIn, stripped.to_string())
                    } else {
                        (CheckinStatus::Success, message)
                    };

                tracing::info!(
                    "[签到结果] 账号: {} | 提供商: {} | 状态: {} | 消息: {} | 奖励: {}",
                    account.name,
                    provider.name,
                    actual_status,
                    actual_message,
                    reward.as_deref().unwrap_or("-")
                );

                let record = match actual_status {
                    CheckinStatus::AlreadyCheckedIn => CheckinRecord::already_checked_in(
                        account_id.to_string(),
                        Some(actual_message.clone()),
                    ),
                    _ => CheckinRecord::success(
                        account_id.to_string(),
                        Some(actual_message.clone()),
                        reward.clone(),
                    ),
                };

                let result = CheckinExecutionResult {
                    account_id: account_id.to_string(),
                    account_name: account.name.clone(),
                    provider_name: provider.name.clone(),
                    status: actual_status,
                    message: Some(actual_message),
                    error_code: None,
                    reward,
                    balance: None,
                };

                (record, result)
            }
            Err(e) => {
                let error_code = e.error_code().to_string();
                let error_msg = e.to_string();
                tracing::error!(
                    "[签到失败] 账号: {} | 提供商: {} | 错误: {} | 分类: {}",
                    account.name,
                    provider.name,
                    error_msg,
                    error_code
                );

                let record = CheckinRecord::failed(
                    account_id.to_string(),
                    error_msg.clone(),
                    Some(error_code.clone()),
                );

                let result = CheckinExecutionResult {
                    account_id: account_id.to_string(),
                    account_name: account.name.clone(),
                    provider_name: provider.name.clone(),
                    status: CheckinStatus::Failed,
                    message: Some(error_msg),
                    error_code: Some(error_code),
                    reward: None,
                    balance: None,
                };

                (record, result)
            }
        };

        // 保存签到记录
        record_manager
            .add(record)
            .map_err(|e| CheckinServiceError::Record(e.to_string()))?;

        // 更新账号最后签到时间
        let _ = account_manager.update_checkin_time(account_id);

        // CDK 充值：签到完成后，检查是否有 CDK 需要处理
        if result.status == CheckinStatus::Success
            || result.status == CheckinStatus::AlreadyCheckedIn
        {
            self.try_cdk_topup(&provider, &account, &cookies_json).await;
        }

        Ok(result)
    }

    /// 远程签到状态预查：通过 /api/user/self 检查账号是否今天已签到
    async fn check_remote_checkin_status(
        &self,
        provider: &CheckinProvider,
        credentials: &CookieCredentials,
        account_name: &str,
    ) -> Option<bool> {
        let url = format!(
            "{}{}",
            provider.base_url.trim_end_matches('/'),
            provider.user_info_path
        );

        let domain = provider.base_url.trim_end_matches('/');

        let mut cookies = credentials.cookies.clone();
        if let Ok(Some(waf_cookies)) = self.get_cached_waf_cookies(&provider.id) {
            cookies = merge_cookies(&cookies, &waf_cookies);
        }
        if let Ok(Some(cf_cookies)) = self.get_cached_cf_cookies(&provider.id) {
            cookies = merge_cookies(&cookies, &cf_cookies);
        }
        let cookie_string = cookie_header_string(&cookies);

        let result = self
            .send_balance_request(&url, domain, credentials, &cookie_string)
            .await;

        let (_status, body) = match result {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!(
                    "[{}] Remote checkin status pre-check failed: {}",
                    account_name,
                    e
                );
                return None;
            }
        };

        let json: serde_json::Value = serde_json::from_str(&body).ok()?;

        let data = json.get("data").unwrap_or(&json);

        if let Some(checked) = data.get("check_in_today").and_then(|v| v.as_bool()) {
            tracing::debug!(
                "[{}] Remote pre-check: check_in_today = {}",
                account_name,
                checked
            );
            return Some(checked);
        }

        if let Some(checked) = data.get("is_checked_in").and_then(|v| v.as_bool()) {
            tracing::debug!(
                "[{}] Remote pre-check: is_checked_in = {}",
                account_name,
                checked
            );
            return Some(checked);
        }

        if let Some(status_str) = data.get("checkin_status").and_then(|v| v.as_str()) {
            let checked = status_str.contains("checked") && !status_str.contains("not");
            tracing::debug!(
                "[{}] Remote pre-check: checkin_status = {} -> {}",
                account_name,
                status_str,
                checked
            );
            return Some(checked);
        }

        tracing::debug!(
            "[{}] Remote pre-check: no checkin status field found in user info",
            account_name
        );
        None
    }

    /// 尝试执行 CDK 充值（签到后自动触发）
    ///
    /// CDK 充值功能尚未在 Tauri WebView 中实现，当前为 no-op。
    /// 不应阻塞签到主流程。
    async fn try_cdk_topup(
        &self,
        _provider: &CheckinProvider,
        _account: &crate::models::checkin::CheckinAccount,
        _cookies_json: &str,
    ) {
        // CDK 充值功能尚未在 Tauri WebView 中实现，暂时跳过
        tracing::debug!("CDK topup skipped: not yet implemented in Tauri WebView");
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
        if let Some(cf_cookies) = self.get_cached_cf_cookies(&provider.id)? {
            cookies = merge_cookies(&cookies, &cf_cookies);
        }
        let mut cookie_string = cookie_header_string(&cookies);

        let (mut status, mut body) = self
            .send_checkin_request(&url, domain, credentials, &cookie_string)
            .await?;

        tracing::info!("Checkin response status: {}", status);
        tracing::info!("Checkin response body: {}", truncate_string(&body, 500));

        // 检测 WAF 挑战页面：尝试刷新 WAF cookies 后重试（软失败模式）
        if is_waf_challenge(&body) {
            tracing::warn!(
                "[{}] Detected WAF challenge, attempting auto bypass...",
                account_name
            );

            match self.refresh_waf_cookies(provider, account_name).await {
                Ok(waf_cookies) => {
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
                Err(e) => {
                    tracing::warn!(
                        "[{}] WAF cookie refresh failed: {}, continuing with original response",
                        account_name, e
                    );
                }
            }
        }

        // 检测 Cloudflare 挑战页面：尝试获取 cf_clearance 后重试（软失败模式）
        if is_cf_challenge(status, &body) {
            tracing::warn!(
                "[{}] Detected CF challenge, attempting auto bypass...",
                account_name
            );

            match self.refresh_cf_cookies(provider, account_name).await {
                Ok(cf_cookies) => {
                    let mut merged = merge_cookies(&credentials.cookies, &cf_cookies);
                    // 同时保留 WAF cookies（如有）
                    if let Some(waf_cookies) = self.get_cached_waf_cookies(&provider.id)? {
                        merged = merge_cookies(&merged, &waf_cookies);
                    }
                    cookie_string = cookie_header_string(&merged);

                    let (retry_status, retry_body) = self
                        .send_checkin_request(&url, domain, credentials, &cookie_string)
                        .await?;

                    status = retry_status;
                    body = retry_body;

                    tracing::info!("Checkin CF retry status: {}", status);
                    tracing::info!("Checkin CF retry response: {}", truncate_string(&body, 500));
                }
                Err(e) => {
                    tracing::warn!(
                        "[{}] CF cookie refresh failed: {}, continuing with original response",
                        account_name, e
                    );
                }
            }
        }

        // 优先尝试 JSON 解析：真正的 WAF 挑战页面是 HTML，不是 JSON。
        // 如果响应是合法 JSON，即使包含 WAF 特征字符串也应按 API 响应处理。
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
            tracing::debug!(
                "Parsed JSON response: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );

            // HTTP 错误但返回了 JSON（API 级别的错误响应）
            if !status.is_success() {
                let message = data["msg"]
                    .as_str()
                    .or(data["message"].as_str())
                    .or(data["error"].as_str())
                    .unwrap_or("签到失败")
                    .to_string();
                return Err(CheckinServiceError::Api(format!(
                    "HTTP {}: {}",
                    status.as_u16(),
                    message
                )));
            }

            let ret_value = data["ret"].as_i64();
            let code_value = data["code"].as_i64();
            let success_value = data["success"].as_bool();

            tracing::debug!(
                "Success indicators - ret: {:?}, code: {:?}, success: {:?}",
                ret_value,
                code_value,
                success_value
            );

            let success = ret_value == Some(1)
                || code_value == Some(0)
                || code_value == Some(200)
                || success_value == Some(true);

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

            if !success && (message.contains("已") || message.contains("already")) {
                // 返回特殊标记，让 caller 识别为 AlreadyCheckedIn
                return Ok((format!("[ALREADY_CHECKED_IN]{}", message), None));
            }

            if !success {
                return Err(CheckinServiceError::Api(message));
            }

            let reward = data["data"].as_object().and_then(|d| {
                if let Some(reward_str) = d.get("reward").and_then(|v| v.as_str()) {
                    Some(reward_str.to_string())
                } else {
                    d.get("points")
                        .and_then(|v| v.as_i64())
                        .map(|points| format!("+{} 积分", points))
                }
            });

            return Ok((message, reward));
        }

        // JSON 解析失败：响应不是合法 JSON，检查是否为 WAF/CF 挑战页面
        tracing::warn!("Failed to parse as JSON, raw response: {}", truncate_string(&body, 500));

        if !status.is_success() {
            if is_waf_challenge(&body) {
                return Err(CheckinServiceError::Api(
                    "检测到 WAF 挑战页面，已尝试自动获取 WAF cookies 但仍失败。请检查代理/出口是否一致，或稍后重试。"
                        .to_string(),
                ));
            }

            if is_cf_challenge(status, &body) {
                return Err(CheckinServiceError::Api(
                    "检测到 Cloudflare 挑战页面，已尝试自动获取 cf_clearance 但仍失败。请检查网络环境，或在有 GUI 的环境中重试。"
                        .to_string(),
                ));
            }

            return Err(CheckinServiceError::Api(format!(
                "HTTP {}: {}",
                status.as_u16(),
                truncate_string(&body, 200)
            )));
        }

        if is_waf_challenge(&body) {
            return Err(CheckinServiceError::Api(
                "检测到 WAF 挑战页面（响应为 HTML），已尝试自动获取 WAF cookies 但仍失败。请检查代理/出口是否一致，或稍后重试。"
                    .to_string(),
            ));
        }

        if body.to_lowercase().contains("success") || body.contains("成功") {
            Ok(("签到成功".to_string(), None))
        } else {
            Err(CheckinServiceError::Api(format!(
                "无法解析响应: {}",
                truncate_string(&body, 100)
            )))
        }
    }

    /// 查询账号余额
    pub async fn query_balance(&self, account_id: &str) -> Result<BalanceSnapshot> {
        let provider_manager = ProviderManager::new();
        let account_manager = AccountManager::new(&self.checkin_dir);
        let balance_manager = BalanceManager::new();
        let crypto = CryptoManager::new(&self.checkin_dir)
            .map_err(|e| CheckinServiceError::Crypto(e.to_string()))?;

        let account = account_manager
            .get(account_id)
            .map_err(|e| CheckinServiceError::Account(e.to_string()))?;

        let provider = provider_manager
            .get(&account.provider_id)
            .map_err(|e| CheckinServiceError::Provider(e.to_string()))?;

        let cookies_json = crypto
            .decrypt(&account.cookies_json_encrypted)
            .map_err(|e| CheckinServiceError::Crypto(e.to_string()))?;

        let credentials = CookieCredentials::from_json(&cookies_json, account.api_user.clone())
            .map_err(|e| CheckinServiceError::Crypto(format!("Invalid cookies JSON: {}", e)))?;

        let snapshot = self
            .do_query_balance(&provider, &credentials, account_id, &account.name)
            .await?;

        balance_manager
            .add(snapshot.clone())
            .map_err(|e| CheckinServiceError::Balance(e.to_string()))?;

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
        if let Some(cf_cookies) = self.get_cached_cf_cookies(&provider.id)? {
            cookies = merge_cookies(&cookies, &cf_cookies);
        }
        let mut cookie_string = cookie_header_string(&cookies);

        let (mut status, mut body) = self
            .send_balance_request(&url, domain, credentials, &cookie_string)
            .await?;

        tracing::info!("Balance query response status: {}", status);
        tracing::info!("Balance query response: {}", truncate_string(&body, 500));

        // 检测 WAF 挑战页面：尝试刷新 WAF cookies 后重试（软失败模式）
        if is_waf_challenge(&body) {
            tracing::warn!(
                "[{}] Detected WAF challenge, attempting auto bypass...",
                account_name
            );

            match self.refresh_waf_cookies(provider, account_name).await {
                Ok(waf_cookies) => {
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
                Err(e) => {
                    tracing::warn!(
                        "[{}] WAF cookie refresh failed: {}, continuing with original response",
                        account_name, e
                    );
                }
            }
        }

        // 检测 Cloudflare 挑战页面：尝试获取 cf_clearance 后重试（软失败模式）
        if is_cf_challenge(status, &body) {
            tracing::warn!(
                "[{}] Detected CF challenge in balance query, attempting auto bypass...",
                account_name
            );

            match self.refresh_cf_cookies(provider, account_name).await {
                Ok(cf_cookies) => {
                    let mut merged = merge_cookies(&credentials.cookies, &cf_cookies);
                    if let Some(waf_cookies) = self.get_cached_waf_cookies(&provider.id)? {
                        merged = merge_cookies(&merged, &waf_cookies);
                    }
                    cookie_string = cookie_header_string(&merged);

                    let (retry_status, retry_body) = self
                        .send_balance_request(&url, domain, credentials, &cookie_string)
                        .await?;

                    status = retry_status;
                    body = retry_body;

                    tracing::info!("Balance query CF retry status: {}", status);
                    tracing::info!(
                        "Balance query CF retry response: {}",
                        truncate_string(&body, 500)
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "[{}] CF cookie refresh failed: {}, continuing with original response",
                        account_name, e
                    );
                }
            }
        }

        if !status.is_success() {
            if is_waf_challenge(&body) {
                return Err(CheckinServiceError::Api(
                    "检测到 WAF 挑战页面，已尝试自动获取 WAF cookies 但仍失败。请检查代理/出口是否一致，或稍后重试。"
                        .to_string(),
                ));
            }

            if is_cf_challenge(status, &body) {
                return Err(CheckinServiceError::Api(
                    "检测到 Cloudflare 挑战页面，已尝试自动获取 cf_clearance 但仍失败。请检查网络环境，或在有 GUI 的环境中重试。"
                        .to_string(),
                ));
            }

            return Err(CheckinServiceError::Api(format!(
                "HTTP {}: {}",
                status.as_u16(),
                truncate_string(&body, 200)
            )));
        }

        if is_waf_challenge(&body) {
            return Err(CheckinServiceError::Api(
                "检测到 WAF 挑战页面，已尝试自动获取 WAF cookies 但仍失败。请检查代理/出口是否一致，或稍后重试。"
                    .to_string(),
            ));
        }

        let data: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            CheckinServiceError::Api(format!(
                "无法解析余额响应: {} - {}",
                e,
                truncate_string(&body, 200)
            ))
        })?;

        tracing::debug!(
            "Parsed balance response: {}",
            serde_json::to_string_pretty(&data).unwrap_or_default()
        );

        if data["data"].is_null() {
            let error_msg = data["message"]
                .as_str()
                .or_else(|| data["msg"].as_str())
                .unwrap_or("API 响应缺少 'data' 字段");
            return Err(CheckinServiceError::Api(format!(
                "{}: {}",
                error_msg,
                truncate_string(&body, 200)
            )));
        }

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

        let available_fields: Vec<&str> = data["data"]
            .as_object()
            .map(|obj| obj.keys().map(|k| k.as_str()).collect())
            .unwrap_or_default();

        Err(CheckinServiceError::Api(format!(
            "无法解析余额响应，缺少 quota/used_quota 字段。可用字段: {:?}，响应: {}",
            available_fields,
            truncate_string(&body, 200)
        )))
    }

    /// 批量签到（并发执行，最多 5 个同时）
    pub async fn batch_checkin(&self, account_ids: &[String]) -> Vec<CheckinExecutionResult> {
        use std::sync::Arc;
        use tokio::sync::Semaphore;

        let semaphore = Arc::new(Semaphore::new(5));
        let futures: Vec<_> = account_ids
            .iter()
            .map(|account_id| {
                let sem = semaphore.clone();
                let id = account_id.clone();
                let client = self.client.clone();
                let checkin_dir = self.checkin_dir.clone();
                let proxy_url = self.proxy_url.clone();
                async move {
                    let _permit = sem.acquire().await.expect("semaphore closed");
                    let svc = CheckinService {
                        checkin_dir,
                        client,
                        proxy_url,
                    };
                    match svc.checkin(&id).await {
                        Ok(r) => r,
                        Err(e) => CheckinExecutionResult {
                            account_id: id,
                            account_name: "Unknown".to_string(),
                            provider_name: "Unknown".to_string(),
                            status: CheckinStatus::Failed,
                            message: Some(e.to_string()),
                            error_code: Some(e.error_code().to_string()),
                            reward: None,
                            balance: None,
                        },
                    }
                }
            })
            .collect();

        futures::future::join_all(futures).await
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
            .map_err(|e| CheckinServiceError::Record(e.to_string()))
    }

    /// 获取所有签到记录
    #[allow(dead_code)]
    pub fn get_all_records(&self, limit: Option<usize>) -> Result<CheckinRecordsResponse> {
        let record_manager = RecordManager::new();
        record_manager
            .get_all(limit)
            .map_err(|e| CheckinServiceError::Record(e.to_string()))
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
            .map_err(|e| CheckinServiceError::Balance(e.to_string()))
    }

    /// 获取账号最新余额
    #[allow(dead_code)]
    pub fn get_latest_balance(&self, account_id: &str) -> Result<Option<BalanceSnapshot>> {
        let balance_manager = BalanceManager::new();
        balance_manager
            .get_latest(account_id)
            .map_err(|e| CheckinServiceError::Balance(e.to_string()))
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
            .map_err(|e| CheckinServiceError::Account(e.to_string()))?;

        let provider = provider_manager
            .get(&account.provider_id)
            .map_err(|e| CheckinServiceError::Provider(e.to_string()))?;

        let latest_balance = balance_manager
            .get_latest(account_id)
            .map_err(|e| CheckinServiceError::Balance(e.to_string()))?;

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
            .map_err(|e| CheckinServiceError::Balance(e.to_string()))?;

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
            .map_err(|e| CheckinServiceError::Account(e.to_string()))?;

        let enabled_accounts: Vec<_> = all_accounts.iter().filter(|a| a.enabled).collect();
        let account_ids: Vec<String> = enabled_accounts.iter().map(|a| a.id.clone()).collect();

        let stats = record_manager
            .get_today_stats(&account_ids)
            .map_err(|e| CheckinServiceError::Record(e.to_string()))?;

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
            .map_err(|e| CheckinServiceError::Crypto(e.to_string()))?;

        let account = account_manager
            .get(account_id)
            .map_err(|e| CheckinServiceError::Account(e.to_string()))?;

        let provider = provider_manager
            .get(&account.provider_id)
            .map_err(|e| CheckinServiceError::Provider(e.to_string()))?;

        let cookies_json = crypto
            .decrypt(&account.cookies_json_encrypted)
            .map_err(|e| CheckinServiceError::Crypto(e.to_string()))?;

        let credentials = CookieCredentials::from_json(&cookies_json, account.api_user.clone())
            .map_err(|e| CheckinServiceError::Crypto(format!("Invalid cookies JSON: {}", e)))?;

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
        if let Some(cf_cookies) = self.get_cached_cf_cookies(&provider.id)? {
            cookies = merge_cookies(&cookies, &cf_cookies);
        }
        let mut cookie_string = cookie_header_string(&cookies);

        let (mut status, mut body) = self
            .send_balance_request(&url, domain, &credentials, &cookie_string)
            .await?;

        // WAF 挑战检测与自动绕过
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

        // CF 挑战检测与自动绕过
        if is_cf_challenge(status, &body) {
            let cf_cookies = self.refresh_cf_cookies(&provider, &account.name).await?;
            let mut merged = merge_cookies(&credentials.cookies, &cf_cookies);
            if let Some(waf_cookies) = self.get_cached_waf_cookies(&provider.id)? {
                merged = merge_cookies(&merged, &waf_cookies);
            }
            cookie_string = cookie_header_string(&merged);

            let (retry_status, retry_body) = self
                .send_balance_request(&url, domain, &credentials, &cookie_string)
                .await?;

            status = retry_status;
            body = retry_body;
        }

        Ok(status.is_success() && !is_waf_challenge(&body) && !is_cf_challenge(status, &body))
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
        .ok_or_else(|| CheckinServiceError::Api("Invalid month".to_string()))?;

    let first_day_next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };

    let last_day = first_day_next_month
        .and_then(|d| d.pred_opt())
        .ok_or_else(|| CheckinServiceError::Api("Invalid month".to_string()))?;

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
            .ok_or_else(|| CheckinServiceError::Api("Invalid date".to_string()))?;
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
        return Err(CheckinServiceError::Api(
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
