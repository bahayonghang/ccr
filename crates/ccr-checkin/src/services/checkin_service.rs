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
    CheckinDashboardTrendPoint, CheckinProvider, CheckinRecord, CheckinRecordInfo,
    CheckinRecordsResponse, CheckinStatus, CookieCredentials,
};
use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveDate};
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::Duration;

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
    /// 跳过原因（仅 status == Skipped 时有值：provider_unsupported / provider_disabled / account_disabled）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
    pub reward: Option<String>,
    pub balance: Option<f64>,
}

/// WAF Cookie 只读验证结果。
#[derive(Debug, Clone, Serialize)]
pub struct WafCookieValidationResult {
    pub account_id: String,
    pub provider_id: String,
    pub provider_name: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub challenge: String,
    pub message: String,
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

#[derive(Debug, Clone, Copy)]
enum ProxySource {
    Env,
    WindowsRegistry,
}

impl ProxySource {
    fn label(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::WindowsRegistry => "windows_registry",
        }
    }
}

fn response_body_chars(body: &str) -> usize {
    body.chars().count()
}

fn response_content_kind(body: &str) -> &'static str {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        "empty"
    } else if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
        "json"
    } else if trimmed.starts_with('<') || trimmed.contains("<html") {
        "html"
    } else {
        "text"
    }
}

fn response_challenge_classification(status: reqwest::StatusCode, body: &str) -> &'static str {
    if is_waf_challenge(body) {
        "waf"
    } else if is_cf_challenge(status, body) {
        "cf"
    } else {
        "none"
    }
}

fn json_object_keys(value: &serde_json::Value) -> Vec<String> {
    value
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default()
}

/// 默认 User-Agent
pub(crate) const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

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

fn get_proxy_settings_from_env() -> Option<(ProxySource, String)> {
    get_proxy_url_from_env().map(|url| (ProxySource::Env, url))
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
        let mut cmd = std::process::Command::new("reg");
        cmd.args(["query", key, "/v", name]);

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let output = cmd.output().ok()?;
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

#[cfg(target_os = "windows")]
fn get_proxy_settings_from_windows_registry() -> Option<(ProxySource, String)> {
    get_proxy_url_from_windows_registry().map(|url| (ProxySource::WindowsRegistry, url))
}

#[cfg(not(target_os = "windows"))]
fn get_proxy_settings_from_windows_registry() -> Option<(ProxySource, String)> {
    get_proxy_url_from_windows_registry().map(|url| (ProxySource::WindowsRegistry, url))
}

fn get_proxy_settings() -> Option<(ProxySource, String)> {
    get_proxy_settings_from_env().or_else(get_proxy_settings_from_windows_registry)
}

/// 解析签到/WAF 统一出口代理 URL（env → Windows 注册表，与签到 HTTP 客户端同源）。
///
/// 供 ccr-ui 同时配置共享 reqwest client 与 WAF WebView，保证两端出口一致：
/// 阿里云 WAF cookie 与来源 IP 绑定，出口不一致会导致 WebView 取到的 cookie 重放失败。
pub fn resolve_checkin_proxy_url() -> Option<String> {
    get_proxy_settings().map(|(_, url)| url)
}

fn is_waf_challenge(text: &str) -> bool {
    // 阿里云 WAF 特征检测
    text.contains("acw_sc__v2")
        || text.contains("<script>var arg1=")
        || text.contains("anti_spider")
        || text.contains("acw_tc")
}

/// 检测 Cloudflare 挑战页面。
///
/// 综合 Newapi-checkin 四签名 + 既有标记，对所有站点的每个响应运行时生效
/// （catalog 的 requires_cf_clearance 静态标记仅作 UI 预期提示，不参与行为判定）：
/// 1. 403 + "Just a moment"
/// 2. 403 + DOCTYPE + "cloudflare"
/// 3. 503 + "cloudflare" + ("challenge" | "checking your browser")
/// 4. 非 JSON 响应 + DOCTYPE + ("just a moment" | "challenge-platform" | "cf-challenge")，任意状态码
fn is_cf_challenge(status: reqwest::StatusCode, body: &str) -> bool {
    let lower = body.to_lowercase();
    let code = status.as_u16();

    if code == 403
        && (lower.contains("just a moment")
            || (lower.contains("<!doctype") && lower.contains("cloudflare")))
    {
        return true;
    }

    if code == 503
        && lower.contains("cloudflare")
        && (lower.contains("challenge") || lower.contains("checking your browser"))
    {
        return true;
    }

    let is_json = serde_json::from_str::<serde_json::Value>(body.trim()).is_ok();
    if !is_json
        && lower.contains("<!doctype")
        && (lower.contains("just a moment")
            || lower.contains("challenge-platform")
            || lower.contains("cf-challenge"))
    {
        return true;
    }

    // 既有标记（向后兼容）：非成功状态 + CF 特征字符串
    let has_cf_markers = body.contains("Just a moment")
        || body.contains("cf-browser-verification")
        || body.contains("_cf_chl")
        || body.contains("cf-challenge-running")
        || body.contains("cf_clearance");
    !status.is_success() && has_cf_markers
}

/// 已签到消息关键词（与 PRD 契约一致的中英文变体清单，归一为 AlreadyCheckedIn）
const ALREADY_CHECKED_IN_KEYWORDS: [&str; 7] = [
    "已签到",
    "已经签到",
    "重复签到",
    "签到过",
    "already checked",
    "already signed",
    "already",
];

/// 判断签到响应消息是否表示「今日已签到」
fn is_already_checked_in_message(message: &str) -> bool {
    let lower = message.to_lowercase();
    ALREADY_CHECKED_IN_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
}

/// 签到 JSON 响应的统一判定结果
#[derive(Debug, Clone, PartialEq, Eq)]
enum CheckinOutcome {
    /// 签到成功
    Success {
        message: String,
        reward: Option<String>,
    },
    /// 今日已签到（关键词归一，不计入失败）
    AlreadyCheckedIn { message: String },
    /// 业务失败（消息进入 CheckinServiceError::Api 的 error_code 分类）
    Failed { message: String },
}

/// `do_checkin` 的成功侧结果（status 仅为 Success / AlreadyCheckedIn；
/// 失败经由 `Err(CheckinServiceError)` 返回以保留 error_code 分类）
#[derive(Debug, Clone)]
struct CheckinSuccessOutcome {
    status: CheckinStatus,
    message: String,
    reward: Option<String>,
}

/// 宽容解析签到 JSON 响应（统一出口，所有 JSON 路径都经过这里）。
///
/// 成功 = `success == true || status == "success" || ret == 1 || code == 0 || code == 200`；
/// message 取 `message || msg || data`（字符串时）；
/// 已签到关键词无论 HTTP 状态/成功标志一律归一为 AlreadyCheckedIn，
/// 消灭「其实已签到却报失败」（Newapi-checkin 直连路径漏归一的教训）。
fn interpret_checkin_json(status: reqwest::StatusCode, data: &serde_json::Value) -> CheckinOutcome {
    let success = data["success"].as_bool() == Some(true)
        || data["status"].as_str() == Some("success")
        || data["ret"].as_i64() == Some(1)
        || data["code"].as_i64() == Some(0)
        || data["code"].as_i64() == Some(200);

    let message = data["message"]
        .as_str()
        .or(data["msg"].as_str())
        .or(data["data"].as_str())
        .or(data["error"].as_str())
        .map(|s| s.to_string());

    // 已签到关键词归一优先于一切失败分支（统一出口的关键约束）
    if let Some(msg) = message.as_deref()
        && is_already_checked_in_message(msg)
    {
        return CheckinOutcome::AlreadyCheckedIn {
            message: msg.to_string(),
        };
    }

    if !status.is_success() {
        return CheckinOutcome::Failed {
            message: format!(
                "HTTP {}: {}",
                status.as_u16(),
                message.as_deref().unwrap_or("签到失败")
            ),
        };
    }

    if success {
        let reward = data["data"].as_object().and_then(|d| {
            if let Some(reward_str) = d.get("reward").and_then(|v| v.as_str()) {
                Some(reward_str.to_string())
            } else {
                d.get("points")
                    .and_then(|v| v.as_i64())
                    .map(|points| format!("+{} 积分", points))
            }
        });
        CheckinOutcome::Success {
            message: message.unwrap_or_else(|| "签到成功".to_string()),
            reward,
        }
    } else {
        CheckinOutcome::Failed {
            message: message.unwrap_or_else(|| "签到失败".to_string()),
        }
    }
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

/// 额度换算汇率：500000 tokens = $1（与余额查询逻辑一致）
const QUOTA_TOKENS_PER_USD: f64 = 500_000.0;

/// `/api/user/self` 预查解析结果（签到状态 + 余额取样，签到前后各取一次供奖励兜底）
#[derive(Debug, Clone, Copy, Default)]
struct UserInfoProbe {
    /// 远程已签到标记（站点未提供相关字段时为 None）
    checked_in_today: Option<bool>,
    /// 剩余额度（token 数）
    quota: Option<f64>,
    /// 已用额度（token 数）
    used_quota: Option<f64>,
}

/// 解析用户信息响应为预查样本（兼容 check_in_today / is_checked_in / checkin_status 三种字段）
fn parse_user_info_probe(json: &serde_json::Value) -> UserInfoProbe {
    let data = json.get("data").unwrap_or(json);

    let checked_in_today = data
        .get("check_in_today")
        .and_then(|v| v.as_bool())
        .or_else(|| data.get("is_checked_in").and_then(|v| v.as_bool()))
        .or_else(|| {
            data.get("checkin_status")
                .and_then(|v| v.as_str())
                .map(|s| s.contains("checked") && !s.contains("not"))
        });

    UserInfoProbe {
        checked_in_today,
        quota: data.get("quota").and_then(|v| v.as_f64()),
        used_quota: data.get("used_quota").and_then(|v| v.as_f64()),
    }
}

/// 提取预查样本的剩余额度（USD，两位小数）
fn probe_remaining_usd(probe: &UserInfoProbe) -> Option<f64> {
    probe
        .quota
        .map(|q| (q / QUOTA_TOKENS_PER_USD * 100.0).round() / 100.0)
}

/// 余额差奖励推断：`(after_quota + after_used) - (before_quota + before_used)`。
///
/// 用总额差而非剩余差，排除签到期间消耗的干扰（anyrouter-check-in / metapi 模式）；
/// 返回的 reward 字串标注推断来源，便于记录/UI 区分「响应解析」与「余额差推断」。
fn infer_reward_from_probes(before: &UserInfoProbe, after: &UserInfoProbe) -> Option<String> {
    let before_total = before.quota? + before.used_quota?;
    let after_total = after.quota? + after.used_quota?;
    let diff_usd = ((after_total - before_total) / QUOTA_TOKENS_PER_USD * 100.0).round() / 100.0;
    if diff_usd <= 0.0 {
        return None;
    }
    Some(format!("+${:.2}（余额差推断）", diff_usd))
}

/// 评估账号/提供商是否应跳过签到，返回 `(skip_reason, 展示消息)`。
///
/// skip_reason 枚举值：`account_disabled` / `provider_disabled` / `provider_unsupported`。
fn evaluate_skip_reason(
    account: &crate::models::checkin::CheckinAccount,
    provider: &CheckinProvider,
) -> Option<(String, String)> {
    if !account.enabled {
        return Some((
            "account_disabled".to_string(),
            "账号已禁用，跳过签到".to_string(),
        ));
    }
    if !provider.enabled {
        return Some((
            "provider_disabled".to_string(),
            "提供商已禁用，跳过签到".to_string(),
        ));
    }

    // 内置站标记为不支持签到（balance_only 等），或自定义站清空了签到路径
    let builtin_unsupported =
        crate::managers::checkin::builtin_providers::resolve_builtin_for_provider(provider)
            .map(|bp| !bp.supports_checkin)
            .unwrap_or(false);
    if builtin_unsupported || provider.checkin_path.trim().is_empty() {
        return Some((
            "provider_unsupported".to_string(),
            "该站点不支持签到（仅余额查询），跳过".to_string(),
        ));
    }

    None
}

/// 补齐浏览器指纹头：现代 Chrome UA / Accept / Accept-Language / Referer / Origin / Sec-Fetch-*。
///
/// 参考 anyrouter-check-in 的请求构造，降低被 WAF/CF 按 bot 特征拦截的概率；
/// 对共享 AppState 客户端与自建客户端两条路径统一生效（UA 按请求设置而非依赖 client 默认值）。
fn apply_browser_headers(
    request: reqwest::RequestBuilder,
    origin: &str,
) -> reqwest::RequestBuilder {
    request
        .header("User-Agent", DEFAULT_USER_AGENT)
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .header("Referer", origin)
        .header("Origin", origin)
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-origin")
}

/// 构造余额/用户信息 GET 请求（含完整浏览器指纹头）
fn build_balance_request(
    client: &Client,
    url: &str,
    origin: &str,
    credentials: &CookieCredentials,
    cookie_string: &str,
) -> reqwest::RequestBuilder {
    let mut request =
        apply_browser_headers(client.get(url), origin).header("X-Requested-With", "XMLHttpRequest");

    if !cookie_string.is_empty() {
        request = request.header("Cookie", cookie_string);
    }
    if credentials.has_api_user() {
        request = request.header("new-api-user", &credentials.api_user);
    }
    request
}

/// 构造签到 POST 请求（含完整浏览器指纹头 + Content-Type + X-Requested-With）
fn build_checkin_request(
    client: &Client,
    url: &str,
    origin: &str,
    credentials: &CookieCredentials,
    cookie_string: &str,
) -> reqwest::RequestBuilder {
    let mut request = apply_browser_headers(client.post(url), origin)
        .header("Content-Type", "application/json")
        .header("X-Requested-With", "XMLHttpRequest");

    if !cookie_string.is_empty() {
        request = request.header("Cookie", cookie_string);
    }
    if credentials.has_api_user() {
        request = request.header("new-api-user", &credentials.api_user);
    }
    request
}

impl CheckinService {
    /// 创建新的签到服务（默认使用系统代理）
    #[allow(dead_code)]
    pub fn new(checkin_dir: PathBuf) -> Self {
        let proxy_settings = get_proxy_settings();
        let proxy_url = proxy_settings.as_ref().map(|(_, url)| url.clone());

        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .cookie_store(true)
            .user_agent(DEFAULT_USER_AGENT)
            .no_proxy();

        match proxy_settings.as_ref() {
            Some((source, url)) => match Proxy::all(url) {
                Ok(proxy) => {
                    tracing::info!(source = source.label(), "签到服务已启用代理");
                    client_builder = client_builder.proxy(proxy);
                }
                Err(e) => tracing::warn!(
                    source = source.label(),
                    error = %e,
                    "代理配置无效，将忽略"
                ),
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
        let proxy_url = get_proxy_settings().map(|(_, url)| url);
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

    async fn send_balance_request(
        &self,
        url: &str,
        domain: &str,
        credentials: &CookieCredentials,
        cookie_string: &str,
    ) -> Result<(reqwest::StatusCode, String)> {
        let request = build_balance_request(&self.client, url, domain, credentials, cookie_string);

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
        let request = build_checkin_request(&self.client, url, domain, credentials, cookie_string);

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

        // 跳过判定：账号/提供商禁用或站点不支持签到 → Skipped（4 态契约，不计入失败）
        if let Some((skip_reason, skip_message)) = evaluate_skip_reason(&account, &provider) {
            tracing::info!(
                "[跳过签到] 账号: {} | 提供商: {} | 原因: {}",
                account.name,
                provider.name,
                skip_reason
            );

            let record = CheckinRecord::skipped(
                account_id.to_string(),
                Some(skip_message.clone()),
                skip_reason.clone(),
            );
            record_manager
                .add(record)
                .map_err(|e| CheckinServiceError::Record(e.to_string()))?;

            return Ok(CheckinExecutionResult {
                account_id: account_id.to_string(),
                account_name: account.name.clone(),
                provider_name: provider.name.clone(),
                status: CheckinStatus::Skipped,
                message: Some(skip_message),
                error_code: None,
                skip_reason: Some(skip_reason),
                reward: None,
                balance: None,
            });
        }

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
                skip_reason: None,
                reward: None,
                balance: None,
            });
        }

        // 解密 Cookies JSON 并创建凭证
        let cookies_json = crypto
            .decrypt(&account.cookies_json_encrypted)
            .map_err(|e| CheckinServiceError::Crypto(e.to_string()))?;

        let credentials = CookieCredentials::from_json(cookies_json.expose(), account.api_user.clone())
            .map_err(|e| CheckinServiceError::Crypto(format!("Invalid cookies JSON: {}", e)))?;

        // 签到前远程预查：检查是否已签到，同时取一次余额样本（供奖励兜底）
        let probe_before = self
            .fetch_user_info_probe(&provider, &credentials, &account.name)
            .await;

        if probe_before.and_then(|p| p.checked_in_today) == Some(true) {
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
                skip_reason: None,
                reward: None,
                balance: probe_before.as_ref().and_then(probe_remaining_usd),
            };

            // 即使已签到，仍尝试 CDK 充值
            self.try_cdk_topup(&provider, &account, cookies_json.expose())
                .await;

            return Ok(result);
        }

        // 执行签到请求
        let checkin_result = self
            .do_checkin(&provider, &credentials, account_id, &account.name)
            .await;

        // 记录签到结果（统一出口：已签到归一已在 do_checkin/interpret_checkin_json 内完成）
        let (record, result) = match checkin_result {
            Ok(outcome) => {
                // 奖励兜底：响应未携带 reward 时，用签到前后两次用户信息的余额总额差推断
                let mut reward = outcome.reward.clone();
                let balance_before_usd = probe_before.as_ref().and_then(probe_remaining_usd);
                let mut balance_after_usd = None;

                if outcome.status == CheckinStatus::Success
                    && reward.is_none()
                    && let Some(probe_after) = self
                        .fetch_user_info_probe(&provider, &credentials, &account.name)
                        .await
                {
                    balance_after_usd = probe_remaining_usd(&probe_after);
                    if let Some(before) = probe_before.as_ref() {
                        reward = infer_reward_from_probes(before, &probe_after);
                    }
                }

                tracing::info!(
                    "[签到结果] 账号: {} | 提供商: {} | 状态: {} | 消息: {} | 奖励: {}",
                    account.name,
                    provider.name,
                    outcome.status,
                    outcome.message,
                    reward.as_deref().unwrap_or("-")
                );

                let record = match outcome.status {
                    CheckinStatus::AlreadyCheckedIn => CheckinRecord::already_checked_in(
                        account_id.to_string(),
                        Some(outcome.message.clone()),
                    ),
                    _ => CheckinRecord::success(
                        account_id.to_string(),
                        Some(outcome.message.clone()),
                        reward.clone(),
                    )
                    .with_balance(balance_before_usd, balance_after_usd),
                };

                let result = CheckinExecutionResult {
                    account_id: account_id.to_string(),
                    account_name: account.name.clone(),
                    provider_name: provider.name.clone(),
                    status: outcome.status,
                    message: Some(outcome.message),
                    error_code: None,
                    skip_reason: None,
                    reward,
                    balance: balance_after_usd,
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
                    skip_reason: None,
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
            self.try_cdk_topup(&provider, &account, cookies_json.expose())
                .await;
        }

        Ok(result)
    }

    /// 用户信息预查：拉取 `/api/user/self`，解析签到状态与余额样本。
    ///
    /// 请求失败/非 JSON 响应时返回 None，不阻塞签到主流程；
    /// 余额样本用于签到响应缺失 reward 时的余额差推断兜底。
    async fn fetch_user_info_probe(
        &self,
        provider: &CheckinProvider,
        credentials: &CookieCredentials,
        account_name: &str,
    ) -> Option<UserInfoProbe> {
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
                tracing::debug!("[{}] User info pre-check failed: {}", account_name, e);
                return None;
            }
        };

        let json: serde_json::Value = serde_json::from_str(&body).ok()?;
        let probe = parse_user_info_probe(&json);

        tracing::debug!(
            "[{}] User info probe: checked_in_today={:?}, quota={:?}, used_quota={:?}",
            account_name,
            probe.checked_in_today,
            probe.quota,
            probe.used_quota
        );

        Some(probe)
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
        account_id: &str,
        account_name: &str,
    ) -> Result<CheckinSuccessOutcome> {
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
        let cookie_string = cookie_header_string(&cookies);

        let (status, body) = self
            .send_checkin_request(&url, domain, credentials, &cookie_string)
            .await?;

        tracing::info!(
            provider_id = %provider.id,
            account_id,
            account_name,
            status = %status.as_u16(),
            content_kind = response_content_kind(&body),
            challenge = response_challenge_classification(status, &body),
            body_chars = response_body_chars(&body),
            "签到响应已接收"
        );

        // WAF/CF cookie 的获取由前端 WebView 补救（open_waf_login）独占负责；
        // 此处仅复用已合并的缓存 cookie 直连，挑战仍在则下方映射为 waf_blocked/cf_blocked
        // 错误，由前端触发 WebView 补救并重试。务必与签到出口同代理（见 state.rs）。

        // 优先尝试 JSON 解析：真正的 WAF 挑战页面是 HTML，不是 JSON。
        // 如果响应是合法 JSON，即使包含 WAF 特征字符串也应按 API 响应处理。
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
            tracing::debug!(
                provider_id = %provider.id,
                account_id,
                account_name,
                status = %status.as_u16(),
                json_keys = ?json_object_keys(&data),
                "签到响应解析为 JSON"
            );

            // 统一判定出口：宽容成功判定 + 已签到关键词归一（替代 [ALREADY_CHECKED_IN] 前缀 hack）
            return match interpret_checkin_json(status, &data) {
                CheckinOutcome::Success { message, reward } => Ok(CheckinSuccessOutcome {
                    status: CheckinStatus::Success,
                    message,
                    reward,
                }),
                CheckinOutcome::AlreadyCheckedIn { message } => Ok(CheckinSuccessOutcome {
                    status: CheckinStatus::AlreadyCheckedIn,
                    message,
                    reward: None,
                }),
                CheckinOutcome::Failed { message } => Err(CheckinServiceError::Api(message)),
            };
        }

        // JSON 解析失败：响应不是合法 JSON，检查是否为 WAF/CF 挑战页面
        tracing::warn!(
            provider_id = %provider.id,
            account_id,
            account_name,
            status = %status.as_u16(),
            content_kind = response_content_kind(&body),
            challenge = response_challenge_classification(status, &body),
            body_chars = response_body_chars(&body),
            "签到响应不是合法 JSON"
        );

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
                "HTTP {}: 返回非 JSON 响应",
                status.as_u16()
            )));
        }

        if is_waf_challenge(&body) {
            return Err(CheckinServiceError::Api(
                "检测到 WAF 挑战页面（响应为 HTML），已尝试自动获取 WAF cookies 但仍失败。请检查代理/出口是否一致，或稍后重试。"
                    .to_string(),
            ));
        }

        // 运行时 CF 检测对成功状态码同样生效（200 + HTML 挑战页，Newapi-checkin 签名 4）
        if is_cf_challenge(status, &body) {
            return Err(CheckinServiceError::Api(
                "检测到 Cloudflare 挑战页面（响应为 HTML），已尝试自动获取 cf_clearance 但仍失败。请检查网络环境，或在有 GUI 的环境中重试。"
                    .to_string(),
            ));
        }

        if body.to_lowercase().contains("success") || body.contains("成功") {
            Ok(CheckinSuccessOutcome {
                status: CheckinStatus::Success,
                message: "签到成功".to_string(),
                reward: None,
            })
        } else {
            Err(CheckinServiceError::Api(
                "无法解析响应: 返回非 JSON 响应".to_string(),
            ))
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

        let credentials = CookieCredentials::from_json(cookies_json.expose(), account.api_user.clone())
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
        let cookie_string = cookie_header_string(&cookies);

        let (status, body) = self
            .send_balance_request(&url, domain, credentials, &cookie_string)
            .await?;

        tracing::info!(
            provider_id = %provider.id,
            account_id,
            account_name,
            status = %status.as_u16(),
            content_kind = response_content_kind(&body),
            challenge = response_challenge_classification(status, &body),
            body_chars = response_body_chars(&body),
            "余额响应已接收"
        );

        // WAF/CF cookie 的获取由前端 WebView 补救（open_waf_login）独占负责；
        // 此处仅复用已合并的缓存 cookie 直连，挑战仍在则下方映射为错误。

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
                "HTTP {}: 返回非 JSON 响应",
                status.as_u16()
            )));
        }

        if is_waf_challenge(&body) {
            return Err(CheckinServiceError::Api(
                "检测到 WAF 挑战页面，已尝试自动获取 WAF cookies 但仍失败。请检查代理/出口是否一致，或稍后重试。"
                    .to_string(),
            ));
        }

        // 运行时 CF 检测对成功状态码同样生效（200 + HTML 挑战页）
        if is_cf_challenge(status, &body) {
            return Err(CheckinServiceError::Api(
                "检测到 Cloudflare 挑战页面（响应为 HTML），已尝试自动获取 cf_clearance 但仍失败。请检查网络环境，或在有 GUI 的环境中重试。"
                    .to_string(),
            ));
        }

        let data: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            tracing::warn!(
                provider_id = %provider.id,
                account_id,
                account_name,
                status = %status.as_u16(),
                content_kind = response_content_kind(&body),
                challenge = response_challenge_classification(status, &body),
                body_chars = response_body_chars(&body),
                error = %e,
                "余额响应不是合法 JSON"
            );
            CheckinServiceError::Api(format!("无法解析余额响应: {}", e))
        })?;

        tracing::debug!(
            provider_id = %provider.id,
            account_id,
            account_name,
            status = %status.as_u16(),
            json_keys = ?json_object_keys(&data),
            "余额响应解析为 JSON"
        );

        if data["data"].is_null() {
            let error_msg = data["message"]
                .as_str()
                .or_else(|| data["msg"].as_str())
                .unwrap_or("API 响应缺少 'data' 字段");
            return Err(CheckinServiceError::Api(format!(
                "{}: API 响应缺少余额数据",
                error_msg
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

            tracing::debug!(
                provider_id = %provider.id,
                account_id,
                account_name,
                total_quota,
                used_quota = total_consumed,
                remaining_quota = current_balance,
                currency = "USD",
                "余额响应已解析为 quota/used_quota 形式"
            );

            return Ok(BalanceSnapshot::new(
                account_id.to_string(),
                total_quota,
                total_consumed,
                current_balance,
                "USD".to_string(),
            ));
        }

        if let Some(balance) = data["data"]["balance"]
            .as_f64()
            .or(data["balance"].as_f64())
        {
            tracing::debug!(
                provider_id = %provider.id,
                account_id,
                account_name,
                total_quota = balance,
                used_quota = 0.0,
                remaining_quota = balance,
                currency = "USD",
                "余额响应已解析为 balance 形式"
            );

            return Ok(BalanceSnapshot::new(
                account_id.to_string(),
                balance,
                0.0,
                balance,
                "USD".to_string(),
            ));
        }

        let available_fields: Vec<&str> = data["data"]
            .as_object()
            .map(|obj| obj.keys().map(|k| k.as_str()).collect())
            .unwrap_or_default();

        Err(CheckinServiceError::Api(format!(
            "无法解析余额响应，缺少 quota/used_quota/balance 字段。可用字段: {:?}",
            available_fields
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
                            skip_reason: None,
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

        // 先取签到记录作为「是否签到 / 奖励金额」的主源；没有记录的账号则全部依赖余额差兜底。
        let record_manager = RecordManager::new();
        let record_resp = record_manager
            .get_by_account(account_id, None)
            .map_err(|e| CheckinServiceError::Record(e.to_string()))?;

        let daily_summaries = build_daily_summaries(&snapshots);
        let daily_checkins = build_daily_checkins(&record_resp.records);
        let streak = compute_streak(&daily_checkins);
        let calendar = build_calendar(account_id, year, month, &daily_checkins, &daily_summaries)?;
        let trend = build_trend(account_id, days, &daily_checkins, &daily_summaries)?;

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

        let credentials = CookieCredentials::from_json(cookies_json.expose(), account.api_user.clone())
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
        let cookie_string = cookie_header_string(&cookies);

        let (status, body) = self
            .send_balance_request(&url, domain, &credentials, &cookie_string)
            .await?;

        // WAF/CF cookie 由前端 WebView 补救获取；此处仅用缓存 cookie 测试连通性。
        Ok(status.is_success() && !is_waf_challenge(&body) && !is_cf_challenge(status, &body))
    }

    /// 使用已缓存的 WAF cookies 访问用户信息接口，验证 cookie 是否真实可用。
    ///
    /// 该方法不刷新 cookies，不写入余额历史，也不更新账号时间戳；用于 UI 自动补救重试前的轻量验证。
    pub async fn validate_cached_waf_access(
        &self,
        account_id: &str,
    ) -> Result<WafCookieValidationResult> {
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

        let cached_waf_cookies = self.get_cached_waf_cookies(&provider.id)?;
        if cached_waf_cookies
            .as_ref()
            .map(|cookies| cookies.is_empty())
            .unwrap_or(true)
        {
            return Ok(WafCookieValidationResult {
                account_id: account.id,
                provider_id: provider.id,
                provider_name: provider.name,
                success: false,
                status_code: None,
                challenge: "none".to_string(),
                message: "没有可用的 WAF Cookie 缓存".to_string(),
            });
        }

        let cookies_json = crypto
            .decrypt(&account.cookies_json_encrypted)
            .map_err(|e| CheckinServiceError::Crypto(e.to_string()))?;

        let credentials = CookieCredentials::from_json(cookies_json.expose(), account.api_user.clone())
            .map_err(|e| CheckinServiceError::Crypto(format!("Invalid cookies JSON: {}", e)))?;

        let validation_path = WafCookieManager::policy_for_provider(&provider)
            .and_then(|policy| policy.validation_path)
            .unwrap_or_else(|| provider.user_info_path.clone());
        let url = format!(
            "{}{}",
            provider.base_url.trim_end_matches('/'),
            validation_path
        );
        let domain = provider.base_url.trim_end_matches('/');

        let mut cookies = credentials.cookies.clone();
        if let Some(waf_cookies) = cached_waf_cookies {
            cookies = merge_cookies(&cookies, &waf_cookies);
        }
        if let Some(cf_cookies) = self.get_cached_cf_cookies(&provider.id)? {
            cookies = merge_cookies(&cookies, &cf_cookies);
        }
        let cookie_string = cookie_header_string(&cookies);

        let (status, body) = self
            .send_balance_request(&url, domain, &credentials, &cookie_string)
            .await?;

        let challenge = response_challenge_classification(status, &body);
        let success = status.is_success()
            && !is_waf_challenge(&body)
            && !is_cf_challenge(status, &body)
            && response_content_kind(&body) != "html";
        let message = if success {
            "WAF Cookie 验证通过".to_string()
        } else if challenge == "waf" {
            "仍返回 WAF 挑战页，请确认网页登录与签到请求使用同一代理/出口".to_string()
        } else if challenge == "cf" {
            "仍返回 Cloudflare 挑战页，请在有 GUI 的环境中获取 cf_clearance".to_string()
        } else if !status.is_success() {
            format!("验证接口返回 HTTP {}", status.as_u16())
        } else {
            "验证接口返回非预期响应".to_string()
        };

        tracing::info!(
            provider_id = %provider.id,
            account_id = %account.id,
            status = %status.as_u16(),
            challenge,
            success,
            body_chars = response_body_chars(&body),
            "WAF Cookie 验证完成"
        );

        Ok(WafCookieValidationResult {
            account_id: account.id,
            provider_id: provider.id,
            provider_name: provider.name,
            success,
            status_code: Some(status.as_u16()),
            challenge: challenge.to_string(),
            message,
        })
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

    daily.sort_by_key(|entry| entry.date);
    daily
}

#[derive(Debug, Clone)]
struct DailyCheckin {
    /// 该日至少存在一条 success / already_checked_in 记录
    checked_in: bool,
    /// 当日可识别的奖励金额（货币单位遵循 balance 字段）；无法解析时为 None
    reward_amount: Option<f64>,
}

/// 按本地日期聚合签到记录。
/// - `checked_in` = 任一条 `status == Success || AlreadyCheckedIn`
/// - `reward_amount` 优先 `balance_after - balance_before`，其次解析 `reward` 字串中的带符号数字
fn build_daily_checkins(records: &[CheckinRecordInfo]) -> BTreeMap<NaiveDate, DailyCheckin> {
    let mut result: BTreeMap<NaiveDate, DailyCheckin> = BTreeMap::new();

    for record in records {
        let is_checked_in = matches!(
            record.status,
            CheckinStatus::Success | CheckinStatus::AlreadyCheckedIn
        );
        if !is_checked_in {
            continue;
        }

        let date = record.checked_in_at.with_timezone(&Local).date_naive();

        let reward_from_balance = match (record.balance_before, record.balance_after) {
            (Some(before), Some(after)) => {
                let diff = after - before;
                if diff > 0.0 { Some(diff) } else { None }
            }
            _ => None,
        };
        let reward_from_string = record.reward.as_deref().and_then(parse_reward_amount);
        let candidate = reward_from_balance.or(reward_from_string);

        let entry = result.entry(date).or_insert(DailyCheckin {
            checked_in: true,
            reward_amount: None,
        });
        entry.checked_in = true;
        entry.reward_amount = match (entry.reward_amount, candidate) {
            (Some(a), Some(b)) if b > a => Some(b),
            (Some(a), _) => Some(a),
            (None, other) => other,
        };
    }

    result
}

/// 解析 reward 字串中的带符号数字前缀。
/// 例如 `"+10 积分" → 10.0`、`"+$2.50" → 2.50`、`"+2美元" → 2.0`、`"+1天" → 1.0`。
fn parse_reward_amount(reward: &str) -> Option<f64> {
    let trimmed = reward.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut iter = trimmed.chars().peekable();
    let mut buf = String::new();

    if matches!(iter.peek(), Some('+') | Some('-')) {
        buf.push(iter.next()?);
    }
    // 跳过货币符号 / 空格等非数字前缀
    while let Some(&c) = iter.peek() {
        if c.is_ascii_digit() || c == '.' {
            break;
        }
        iter.next();
    }

    let mut saw_digit = false;
    let mut saw_dot = false;
    while let Some(&c) = iter.peek() {
        if c.is_ascii_digit() {
            buf.push(c);
            saw_digit = true;
            iter.next();
        } else if c == '.' && !saw_dot {
            buf.push(c);
            saw_dot = true;
            iter.next();
        } else {
            break;
        }
    }

    if !saw_digit {
        return None;
    }
    buf.parse::<f64>().ok()
}

fn compute_streak(daily_checkins: &BTreeMap<NaiveDate, DailyCheckin>) -> CheckinDashboardStreak {
    let mut longest_streak = 0u32;
    let mut total_check_in_days = 0u32;
    let mut last_check_in_date: Option<NaiveDate> = None;
    let mut running_streak = 0u32;

    for (date, day) in daily_checkins {
        if !day.checked_in {
            continue;
        }
        total_check_in_days += 1;
        running_streak = match last_check_in_date {
            Some(prev) if date.signed_duration_since(prev).num_days() == 1 => running_streak + 1,
            _ => 1,
        };
        longest_streak = longest_streak.max(running_streak);
        last_check_in_date = Some(*date);
    }

    // 当前连续仅当最后签到发生在今天或昨天（本地日）时延续
    let current_streak = match last_check_in_date {
        Some(last) => {
            let today = Local::now().date_naive();
            let gap = today.signed_duration_since(last).num_days();
            if (0..=1).contains(&gap) {
                running_streak
            } else {
                0
            }
        }
        None => 0,
    };

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
    daily_checkins: &BTreeMap<NaiveDate, DailyCheckin>,
    daily_summaries: &[DailySummary],
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

    let summary_map: HashMap<NaiveDate, &DailySummary> =
        daily_summaries.iter().map(|s| (s.date, s)).collect();

    let today = Local::now().date_naive();

    let mut days = Vec::with_capacity(total_days as usize);
    let mut checked_in_days = 0u32;
    let mut total_quota_increment = 0.0;
    let mut days_up_to_today = 0u32;
    let mut prev_total: Option<f64> = None;

    for day in 1..=total_days {
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| CheckinServiceError::Api("Invalid date".to_string()))?;
        let date_str = date.format("%Y-%m-%d").to_string();

        let daily_checkin = daily_checkins.get(&date);
        let summary = summary_map.get(&date).copied();

        let is_checked_in = daily_checkin.map(|d| d.checked_in).unwrap_or(false);
        let reward_amount = daily_checkin.and_then(|d| d.reward_amount);

        // income_increment 优先来自签到记录的 reward_amount，否则回落到相邻余额差
        let income_increment =
            reward_amount.or_else(|| match (prev_total, summary.map(|s| s.total_quota)) {
                (Some(prev), Some(curr)) => {
                    let diff = curr - prev;
                    if diff > 0.0 { Some(diff) } else { None }
                }
                _ => None,
            });

        if date <= today {
            days_up_to_today += 1;
        }
        if is_checked_in {
            checked_in_days += 1;
            if let Some(inc) = income_increment {
                total_quota_increment += inc;
            }
        }

        let (current_balance, total_consumed, total_quota) = summary
            .map(|s| (s.remaining_quota, s.used_quota, s.total_quota))
            .unwrap_or((0.0, 0.0, 0.0));

        days.push(CheckinDashboardDay {
            date: date_str,
            is_checked_in,
            income_increment,
            reward_amount,
            current_balance,
            total_consumed,
            total_quota,
        });

        if let Some(s) = summary {
            prev_total = Some(s.total_quota);
        }
    }

    // 签到率：当前月以 "本月已到今天为止的天数" 作分母，避免月底前未来日拉低分母；
    // 历史/未来月份仍按 total_days。
    let is_current_month = today.year() == year && today.month() == month;
    let rate_denominator = if is_current_month {
        days_up_to_today.max(1)
    } else {
        total_days
    };
    let check_in_rate = if rate_denominator > 0 {
        (checked_in_days as f64 / rate_denominator as f64) * 100.0
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
    daily_checkins: &BTreeMap<NaiveDate, DailyCheckin>,
    daily_summaries: &[DailySummary],
) -> Result<CheckinDashboardTrend> {
    if days == 0 || days > 365 {
        return Err(CheckinServiceError::Api(
            "Days must be between 1 and 365".to_string(),
        ));
    }

    let end_date = Local::now().date_naive();
    let start_date = end_date - ChronoDuration::days(days as i64 - 1);

    let summary_map: HashMap<NaiveDate, &DailySummary> =
        daily_summaries.iter().map(|s| (s.date, s)).collect();

    // 窗口内每一天都输出一个点，方便前端画固定列数的热力柱
    let mut data_points = Vec::with_capacity(days as usize);
    let mut last_known_quota = daily_summaries
        .iter()
        .rev()
        .find(|s| s.date < start_date)
        .map(|s| s.total_quota)
        .unwrap_or(0.0);
    let mut last_known_balance = daily_summaries
        .iter()
        .rev()
        .find(|s| s.date < start_date)
        .map(|s| s.remaining_quota)
        .unwrap_or(0.0);

    let mut date = start_date;
    while date <= end_date {
        if let Some(s) = summary_map.get(&date) {
            last_known_quota = s.total_quota;
            last_known_balance = s.remaining_quota;
        }

        let daily_checkin = daily_checkins.get(&date);
        let is_checked_in = daily_checkin.map(|d| d.checked_in).unwrap_or(false);
        let reward_amount = daily_checkin.and_then(|d| d.reward_amount).unwrap_or(0.0);

        data_points.push(CheckinDashboardTrendPoint {
            date: date.format("%Y-%m-%d").to_string(),
            total_quota: last_known_quota,
            income_increment: reward_amount,
            reward_amount,
            current_balance: last_known_balance,
            is_checked_in,
        });

        match date.succ_opt() {
            Some(next) => date = next,
            None => break,
        }
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

    #[test]
    fn test_response_metadata_helpers() {
        assert_eq!(response_body_chars("余额"), 2);
        assert_eq!(response_content_kind(r#"{"ok":true}"#), "json");
        assert_eq!(response_content_kind("<html>challenge</html>"), "html");
        assert_eq!(response_content_kind("plain text"), "text");
        assert_eq!(
            response_challenge_classification(
                reqwest::StatusCode::OK,
                "<script>var arg1=abc</script>",
            ),
            "waf"
        );
        assert_eq!(
            response_challenge_classification(
                reqwest::StatusCode::FORBIDDEN,
                "<html>Just a moment _cf_chl</html>",
            ),
            "cf"
        );
        let mut keys = json_object_keys(&serde_json::json!({ "ret": 1, "message": "ok" }));
        keys.sort();
        assert_eq!(keys, vec!["message".to_string(), "ret".to_string()]);
    }

    use chrono::{TimeZone, Utc as ChronoUtc};

    fn record_info(
        account_id: &str,
        status: CheckinStatus,
        reward: Option<&str>,
        balance_before: Option<f64>,
        balance_after: Option<f64>,
        checked_in_at: chrono::DateTime<ChronoUtc>,
    ) -> CheckinRecordInfo {
        CheckinRecordInfo {
            id: "test-id".to_string(),
            account_id: account_id.to_string(),
            account_name: None,
            provider_name: None,
            status,
            message: None,
            error_code: None,
            reward: reward.map(|s| s.to_string()),
            balance_before,
            balance_after,
            balance_change: match (balance_before, balance_after) {
                (Some(b), Some(a)) => Some(a - b),
                _ => None,
            },
            checked_in_at,
        }
    }

    #[test]
    fn test_parse_reward_amount_variants() {
        assert_eq!(parse_reward_amount("+10 积分"), Some(10.0));
        assert_eq!(parse_reward_amount("+$2.50"), Some(2.50));
        assert_eq!(parse_reward_amount("+2美元"), Some(2.0));
        assert_eq!(parse_reward_amount("+1天"), Some(1.0));
        assert_eq!(parse_reward_amount("-3"), Some(-3.0));
        assert_eq!(parse_reward_amount(""), None);
        assert_eq!(parse_reward_amount("签到成功"), None);
    }

    #[test]
    fn test_build_daily_checkins_record_without_balance() {
        // 场景：有签到记录但没有余额快照字段 → is_checked_in=true, reward_amount=None
        let ts = ChronoUtc.with_ymd_and_hms(2026, 4, 19, 3, 0, 0).unwrap();
        let records = vec![record_info(
            "acc-1",
            CheckinStatus::Success,
            None,
            None,
            None,
            ts,
        )];

        let map = build_daily_checkins(&records);
        assert_eq!(map.len(), 1);
        let (_, day) = map.iter().next().unwrap();
        assert!(day.checked_in);
        assert_eq!(day.reward_amount, None);
    }

    #[test]
    fn test_build_daily_checkins_reward_from_balance_delta() {
        // 场景：balance_before=10, balance_after=12 → reward_amount=Some(2.0)
        let ts = ChronoUtc.with_ymd_and_hms(2026, 4, 19, 3, 0, 0).unwrap();
        let records = vec![record_info(
            "acc-1",
            CheckinStatus::Success,
            None,
            Some(10.0),
            Some(12.0),
            ts,
        )];

        let map = build_daily_checkins(&records);
        let (_, day) = map.iter().next().unwrap();
        assert!(day.checked_in);
        assert_eq!(day.reward_amount, Some(2.0));
    }

    #[test]
    fn test_build_daily_checkins_falls_back_to_reward_string() {
        // balance 缺失时应从 reward 字串解析
        let ts = ChronoUtc.with_ymd_and_hms(2026, 4, 19, 3, 0, 0).unwrap();
        let records = vec![record_info(
            "acc-1",
            CheckinStatus::Success,
            Some("+5 积分"),
            None,
            None,
            ts,
        )];

        let map = build_daily_checkins(&records);
        let (_, day) = map.iter().next().unwrap();
        assert_eq!(day.reward_amount, Some(5.0));
    }

    #[test]
    fn test_build_daily_checkins_excludes_failed() {
        // 失败的记录不应计入签到
        let ts = ChronoUtc.with_ymd_and_hms(2026, 4, 19, 3, 0, 0).unwrap();
        let records = vec![record_info(
            "acc-1",
            CheckinStatus::Failed,
            None,
            None,
            None,
            ts,
        )];

        let map = build_daily_checkins(&records);
        assert!(map.is_empty());
    }

    #[test]
    fn test_compute_streak_from_records() {
        // 构造三条连续本地日签到（03:00 UTC ≈ 东八区 11:00）
        let day1 = ChronoUtc.with_ymd_and_hms(2026, 4, 17, 3, 0, 0).unwrap();
        let day2 = ChronoUtc.with_ymd_and_hms(2026, 4, 18, 3, 0, 0).unwrap();
        let day3 = ChronoUtc.with_ymd_and_hms(2026, 4, 19, 3, 0, 0).unwrap();
        let records = vec![
            record_info("acc-1", CheckinStatus::Success, None, None, None, day1),
            record_info("acc-1", CheckinStatus::Success, None, None, None, day2),
            record_info("acc-1", CheckinStatus::Success, None, None, None, day3),
        ];

        let map = build_daily_checkins(&records);
        let streak = compute_streak(&map);
        assert_eq!(streak.longest_streak, 3);
        assert_eq!(streak.total_check_in_days, 3);
        assert!(streak.last_check_in_date.is_some());
    }

    #[test]
    fn test_build_calendar_marks_day_even_without_balance_snapshot() {
        // 关键回归：只有 CheckinRecord 没有 BalanceSnapshot 时，日历仍应打点、签到率>0
        let ts = ChronoUtc.with_ymd_and_hms(2026, 4, 19, 3, 0, 0).unwrap();
        let records = vec![record_info(
            "acc-1",
            CheckinStatus::Success,
            None,
            Some(10.0),
            Some(12.0),
            ts,
        )];

        let daily_checkins = build_daily_checkins(&records);
        let daily_summaries: Vec<DailySummary> = Vec::new();

        let calendar = build_calendar("acc-1", 2026, 4, &daily_checkins, &daily_summaries).unwrap();

        let target_date = daily_checkins.keys().next().copied().unwrap();
        let target_day = calendar
            .days
            .iter()
            .find(|d| d.date == target_date.format("%Y-%m-%d").to_string())
            .unwrap();

        assert!(target_day.is_checked_in);
        assert_eq!(target_day.reward_amount, Some(2.0));
        assert!(calendar.month_stats.checked_in_days >= 1);
        assert!(calendar.month_stats.check_in_rate > 0.0);
    }

    // ═══════════════════════════════════════════════════════════
    // 请求指纹（浏览器头 + HTTP/2）
    // ═══════════════════════════════════════════════════════════

    fn test_credentials(api_user: &str) -> CookieCredentials {
        CookieCredentials {
            cookies: HashMap::new(),
            api_user: api_user.to_string(),
        }
    }

    #[test]
    fn test_checkin_request_browser_fingerprint_headers() {
        let client = Client::new();
        let request = build_checkin_request(
            &client,
            "https://api.example.com/api/user/checkin",
            "https://api.example.com",
            &test_credentials("123"),
            "session=abc",
        )
        .build()
        .unwrap();

        assert_eq!(request.method(), &reqwest::Method::POST);
        let headers = request.headers();
        assert_eq!(headers.get("User-Agent").unwrap(), DEFAULT_USER_AGENT);
        assert_eq!(
            headers.get("Accept").unwrap(),
            "application/json, text/plain, */*"
        );
        assert_eq!(
            headers.get("Accept-Language").unwrap(),
            "zh-CN,zh;q=0.9,en;q=0.8"
        );
        assert_eq!(headers.get("Referer").unwrap(), "https://api.example.com");
        assert_eq!(headers.get("Origin").unwrap(), "https://api.example.com");
        assert_eq!(headers.get("Sec-Fetch-Dest").unwrap(), "empty");
        assert_eq!(headers.get("Sec-Fetch-Mode").unwrap(), "cors");
        assert_eq!(headers.get("Sec-Fetch-Site").unwrap(), "same-origin");
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(headers.get("X-Requested-With").unwrap(), "XMLHttpRequest");
        assert_eq!(headers.get("Cookie").unwrap(), "session=abc");
        assert_eq!(headers.get("new-api-user").unwrap(), "123");
    }

    #[test]
    fn test_balance_request_browser_fingerprint_headers() {
        let client = Client::new();
        let request = build_balance_request(
            &client,
            "https://api.example.com/api/user/self",
            "https://api.example.com",
            &test_credentials(""),
            "",
        )
        .build()
        .unwrap();

        assert_eq!(request.method(), &reqwest::Method::GET);
        let headers = request.headers();
        assert_eq!(headers.get("User-Agent").unwrap(), DEFAULT_USER_AGENT);
        assert_eq!(headers.get("Sec-Fetch-Dest").unwrap(), "empty");
        assert_eq!(headers.get("Sec-Fetch-Mode").unwrap(), "cors");
        assert_eq!(headers.get("Sec-Fetch-Site").unwrap(), "same-origin");
        assert_eq!(headers.get("Referer").unwrap(), "https://api.example.com");
        assert_eq!(headers.get("Origin").unwrap(), "https://api.example.com");
        // 空 cookie / 空 api_user 不应产生对应请求头
        assert!(!headers.contains_key("Cookie"));
        assert!(!headers.contains_key("new-api-user"));
    }

    #[test]
    fn test_http2_feature_enabled() {
        // 编译期守卫：http2_prior_knowledge 仅在 reqwest "http2" feature 开启时存在；
        // 若依赖声明回退（default-features=false 且缺少 http2），此测试将无法编译。
        let client = Client::builder().http2_prior_knowledge().build();
        assert!(client.is_ok());
    }

    // ═══════════════════════════════════════════════════════════
    // 宽容响应判定矩阵（统一出口）
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_interpret_checkin_json_success_styles() {
        let ok = reqwest::StatusCode::OK;
        let cases = [
            serde_json::json!({"success": true, "message": "签到成功"}),
            serde_json::json!({"status": "success", "msg": "done"}),
            serde_json::json!({"ret": 1, "msg": "签到成功"}),
            serde_json::json!({"code": 0, "message": "签到成功"}),
            serde_json::json!({"code": 200, "message": "签到成功"}),
        ];

        for data in &cases {
            match interpret_checkin_json(ok, data) {
                CheckinOutcome::Success { .. } => {}
                other => panic!("expected Success for {data}, got {other:?}"),
            }
        }
    }

    #[test]
    fn test_interpret_checkin_json_message_fallback_chain() {
        let ok = reqwest::StatusCode::OK;

        // message || msg || data（字符串）
        let data = serde_json::json!({"success": true, "data": "获得 10000 额度"});
        match interpret_checkin_json(ok, &data) {
            CheckinOutcome::Success { message, .. } => assert_eq!(message, "获得 10000 额度"),
            other => panic!("expected Success, got {other:?}"),
        }

        // 全部缺失时回落到默认文案
        let data = serde_json::json!({"ret": 1});
        match interpret_checkin_json(ok, &data) {
            CheckinOutcome::Success { message, .. } => assert_eq!(message, "签到成功"),
            other => panic!("expected Success, got {other:?}"),
        }
    }

    #[test]
    fn test_interpret_checkin_json_reward_extraction() {
        let ok = reqwest::StatusCode::OK;

        let data = serde_json::json!({"ret": 1, "msg": "ok", "data": {"reward": "+$2"}});
        match interpret_checkin_json(ok, &data) {
            CheckinOutcome::Success { reward, .. } => assert_eq!(reward.as_deref(), Some("+$2")),
            other => panic!("expected Success, got {other:?}"),
        }

        let data = serde_json::json!({"success": true, "data": {"points": 10}});
        match interpret_checkin_json(ok, &data) {
            CheckinOutcome::Success { reward, .. } => {
                assert_eq!(reward.as_deref(), Some("+10 积分"))
            }
            other => panic!("expected Success, got {other:?}"),
        }
    }

    #[test]
    fn test_interpret_checkin_json_already_checked_in_variants() {
        let ok = reqwest::StatusCode::OK;
        let variants = [
            "今日已签到",
            "已经签到了",
            "请勿重复签到",
            "今天签到过了",
            "Already checked in today",
            "already signed in",
            "You have already attended",
        ];

        for msg in variants {
            let data = serde_json::json!({"success": false, "message": msg});
            match interpret_checkin_json(ok, &data) {
                CheckinOutcome::AlreadyCheckedIn { message } => assert_eq!(message, msg),
                other => panic!("expected AlreadyCheckedIn for {msg}, got {other:?}"),
            }
        }

        // success=true 但消息表示已签到 → 同样归一
        let data = serde_json::json!({"success": true, "message": "今日已签到"});
        assert!(matches!(
            interpret_checkin_json(ok, &data),
            CheckinOutcome::AlreadyCheckedIn { .. }
        ));

        // HTTP 4xx + 已签到消息 → 仍归一（统一出口，不受状态码影响）
        let data = serde_json::json!({"success": false, "message": "已签到"});
        assert!(matches!(
            interpret_checkin_json(reqwest::StatusCode::BAD_REQUEST, &data),
            CheckinOutcome::AlreadyCheckedIn { .. }
        ));
    }

    #[test]
    fn test_interpret_checkin_json_failures() {
        // 业务失败：消息原样透传（供 error_code 关键词分类）
        let data = serde_json::json!({"success": false, "message": "cookie 无效"});
        match interpret_checkin_json(reqwest::StatusCode::OK, &data) {
            CheckinOutcome::Failed { message } => assert_eq!(message, "cookie 无效"),
            other => panic!("expected Failed, got {other:?}"),
        }

        // HTTP 错误 + JSON 错误体：HTTP 状态码进入消息
        let data = serde_json::json!({"message": "未登录"});
        match interpret_checkin_json(reqwest::StatusCode::UNAUTHORIZED, &data) {
            CheckinOutcome::Failed { message } => assert_eq!(message, "HTTP 401: 未登录"),
            other => panic!("expected Failed, got {other:?}"),
        }

        // 空对象：无成功标志 → 失败默认文案
        let data = serde_json::json!({});
        match interpret_checkin_json(reqwest::StatusCode::OK, &data) {
            CheckinOutcome::Failed { message } => assert_eq!(message, "签到失败"),
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn test_is_already_checked_in_message_keywords() {
        for msg in [
            "已签到",
            "已经签到",
            "重复签到",
            "签到过",
            "ALREADY CHECKED IN",
            "Already Signed",
            "already",
        ] {
            assert!(is_already_checked_in_message(msg), "message: {msg}");
        }

        for msg in ["签到成功", "余额不足", "checkin ok", "您已成功签到"] {
            assert!(!is_already_checked_in_message(msg), "message: {msg}");
        }
    }

    // ═══════════════════════════════════════════════════════════
    // 运行时 WAF/CF 检测（Newapi-checkin 四签名）
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_is_cf_challenge_four_signatures() {
        use reqwest::StatusCode;

        // 签名 1：403 + Just a moment
        assert!(is_cf_challenge(
            StatusCode::FORBIDDEN,
            "<html>Just a moment...</html>"
        ));
        // 签名 2：403 + DOCTYPE + cloudflare
        assert!(is_cf_challenge(
            StatusCode::FORBIDDEN,
            "<!DOCTYPE html><body>cloudflare protection</body>"
        ));
        // 签名 3：503 + cloudflare + challenge / checking your browser
        assert!(is_cf_challenge(
            StatusCode::SERVICE_UNAVAILABLE,
            "cloudflare challenge in progress"
        ));
        assert!(is_cf_challenge(
            StatusCode::SERVICE_UNAVAILABLE,
            "cloudflare is checking your browser"
        ));
        // 签名 4：非 JSON HTML 拦截页（200 也要命中——运行时检测对所有响应生效）
        assert!(is_cf_challenge(
            StatusCode::OK,
            "<!DOCTYPE html><script src=\"/cdn-cgi/challenge-platform/x.js\"></script>"
        ));
        assert!(is_cf_challenge(
            StatusCode::OK,
            "<!doctype html>Just a moment"
        ));
        // 既有标记（向后兼容）：非成功状态 + _cf_chl
        assert!(is_cf_challenge(
            StatusCode::FORBIDDEN,
            "<html>_cf_chl test</html>"
        ));
    }

    #[test]
    fn test_is_cf_challenge_not_triggered_by_normal_responses() {
        use reqwest::StatusCode;

        // 正常 JSON 响应
        assert!(!is_cf_challenge(StatusCode::OK, r#"{"success":true}"#));
        // 200 JSON 中包含 cf_clearance 字样 → 不是挑战
        assert!(!is_cf_challenge(
            StatusCode::OK,
            r#"{"message":"cf_clearance ok"}"#
        ));
        // 200 普通 HTML，无 CF 标记
        assert!(!is_cf_challenge(
            StatusCode::OK,
            "<!DOCTYPE html><body>hello</body>"
        ));
        // 403 普通 JSON 错误（无 CF 标记）
        assert!(!is_cf_challenge(
            StatusCode::FORBIDDEN,
            r#"{"message":"forbidden"}"#
        ));
    }

    // ═══════════════════════════════════════════════════════════
    // 用户信息预查与奖励兜底
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_parse_user_info_probe_variants() {
        // data 包裹 + check_in_today + quota/used_quota
        let probe = parse_user_info_probe(&serde_json::json!({
            "data": {"check_in_today": true, "quota": 5_000_000.0, "used_quota": 1_000_000.0}
        }));
        assert_eq!(probe.checked_in_today, Some(true));
        assert_eq!(probe.quota, Some(5_000_000.0));
        assert_eq!(probe.used_quota, Some(1_000_000.0));

        // 扁平结构 + is_checked_in
        let probe = parse_user_info_probe(&serde_json::json!({"is_checked_in": false}));
        assert_eq!(probe.checked_in_today, Some(false));

        // checkin_status 字符串
        let probe =
            parse_user_info_probe(&serde_json::json!({"data": {"checkin_status": "not_checked"}}));
        assert_eq!(probe.checked_in_today, Some(false));
        let probe =
            parse_user_info_probe(&serde_json::json!({"data": {"checkin_status": "checked"}}));
        assert_eq!(probe.checked_in_today, Some(true));

        // 无相关字段
        let probe = parse_user_info_probe(&serde_json::json!({"data": {"id": 1}}));
        assert_eq!(probe.checked_in_today, None);
        assert_eq!(probe.quota, None);
    }

    #[test]
    fn test_infer_reward_from_probes() {
        let before = UserInfoProbe {
            checked_in_today: None,
            quota: Some(2_500_000.0),
            used_quota: Some(500_000.0),
        };
        let after = UserInfoProbe {
            checked_in_today: Some(true),
            quota: Some(7_000_000.0),
            used_quota: Some(1_000_000.0),
        };

        // 总额差 = (7M+1M)-(2.5M+0.5M) = 5M tokens = $10，标注推断来源
        assert_eq!(
            infer_reward_from_probes(&before, &after).as_deref(),
            Some("+$10.00（余额差推断）")
        );

        // 无变化 → None
        assert_eq!(infer_reward_from_probes(&before, &before), None);

        // 字段缺失 → None
        assert_eq!(
            infer_reward_from_probes(&UserInfoProbe::default(), &after),
            None
        );

        // 期间消耗不影响推断（quota 减少但 used 等量增加 → 总额不变 → None）
        let consumed = UserInfoProbe {
            checked_in_today: None,
            quota: Some(2_000_000.0),
            used_quota: Some(1_000_000.0),
        };
        assert_eq!(infer_reward_from_probes(&before, &consumed), None);
    }

    #[test]
    fn test_probe_remaining_usd() {
        let probe = UserInfoProbe {
            checked_in_today: None,
            quota: Some(2_500_000.0),
            used_quota: None,
        };
        assert_eq!(probe_remaining_usd(&probe), Some(5.0));
        assert_eq!(probe_remaining_usd(&UserInfoProbe::default()), None);
    }

    // ═══════════════════════════════════════════════════════════
    // 4 态契约：skip_reason 路径
    // ═══════════════════════════════════════════════════════════

    fn skip_test_provider(builtin_id: Option<&str>) -> CheckinProvider {
        let mut provider = CheckinProvider::new(
            "SkipTest".to_string(),
            "https://api.example.com".to_string(),
        );
        provider.builtin_id = builtin_id.map(|s| s.to_string());
        provider
    }

    fn skip_test_account(enabled: bool) -> crate::models::checkin::CheckinAccount {
        let mut account = crate::models::checkin::CheckinAccount::new(
            "provider-1".to_string(),
            "acct".to_string(),
            "encrypted".to_string(),
            String::new(),
        );
        account.enabled = enabled;
        account
    }

    #[test]
    fn test_evaluate_skip_reason_matrix() {
        // 账号禁用优先
        let (reason, _) =
            evaluate_skip_reason(&skip_test_account(false), &skip_test_provider(None)).unwrap();
        assert_eq!(reason, "account_disabled");

        // 提供商禁用
        let mut provider = skip_test_provider(None);
        provider.enabled = false;
        let (reason, _) = evaluate_skip_reason(&skip_test_account(true), &provider).unwrap();
        assert_eq!(reason, "provider_disabled");

        // balance_only 内置站（builtin-coderouter 不支持签到）
        let provider = skip_test_provider(Some("builtin-coderouter"));
        let (reason, message) = evaluate_skip_reason(&skip_test_account(true), &provider).unwrap();
        assert_eq!(reason, "provider_unsupported");
        assert!(message.contains("不支持签到"));

        // 自定义站清空签到路径
        let mut provider = skip_test_provider(None);
        provider.checkin_path = "  ".to_string();
        let (reason, _) = evaluate_skip_reason(&skip_test_account(true), &provider).unwrap();
        assert_eq!(reason, "provider_unsupported");

        // 正常启用站点 → 不跳过
        assert!(
            evaluate_skip_reason(&skip_test_account(true), &skip_test_provider(None)).is_none()
        );

        // 支持签到的内置站 → 不跳过
        assert!(
            evaluate_skip_reason(
                &skip_test_account(true),
                &skip_test_provider(Some("builtin-anyrouter"))
            )
            .is_none()
        );
    }

    fn cleanup_checkin_tables() {
        database::with_connection(|conn| {
            conn.execute("DELETE FROM checkin_records", [])?;
            conn.execute("DELETE FROM checkin_accounts", [])?;
            conn.execute("DELETE FROM checkin_providers", [])?;
            Ok(())
        })
        .unwrap();
    }

    #[tokio::test]
    async fn test_checkin_skips_balance_only_builtin_provider() {
        use crate::managers::checkin::ProviderManager;
        use crate::models::checkin::{CreateAccountRequest, CreateProviderRequest};

        let (_temp_dir, service) = {
            database::initialize_for_test().unwrap();
            let temp_dir = TempDir::new().unwrap();
            let service = CheckinService::new(temp_dir.path().to_path_buf());
            (temp_dir, service)
        };

        // 共享全局测试库（--test-threads=1），先后清场避免污染其它用例
        cleanup_checkin_tables();

        let provider_manager = ProviderManager::new();
        let provider = provider_manager
            .create(CreateProviderRequest {
                name: "CodeRouter".to_string(),
                base_url: "https://api.codemirror.codes".to_string(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
                builtin_id: Some("builtin-coderouter".to_string()),
            })
            .unwrap();

        let account_manager = AccountManager::new(_temp_dir.path());
        let account = account_manager
            .create(CreateAccountRequest {
                provider_id: provider.id.clone(),
                name: "acct".to_string(),
                cookies_json: ccr_core::Secret::from(r#"{"session":"abc"}"#),
                api_user: String::new(),
                extra_config: "{}".to_string(),
            })
            .unwrap();

        // balance_only 站点：不发任何 HTTP 请求即返回 Skipped(provider_unsupported)
        let result = service.checkin(&account.id).await.unwrap();
        assert_eq!(result.status, CheckinStatus::Skipped);
        assert_eq!(result.skip_reason.as_deref(), Some("provider_unsupported"));
        assert!(result.error_code.is_none());

        // 跳过会落一条 skipped 记录（skip_reason 持久化在 error_code 字段）
        let records = RecordManager::new()
            .get_by_account(&account.id, None)
            .unwrap();
        assert_eq!(records.records.len(), 1);
        assert_eq!(records.records[0].status, CheckinStatus::Skipped);
        assert_eq!(
            records.records[0].error_code.as_deref(),
            Some("provider_unsupported")
        );

        cleanup_checkin_tables();
    }

    #[tokio::test]
    async fn test_checkin_skips_disabled_account() {
        use crate::managers::checkin::ProviderManager;
        use crate::models::checkin::{
            CreateAccountRequest, CreateProviderRequest, UpdateAccountRequest,
        };

        database::initialize_for_test().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let service = CheckinService::new(temp_dir.path().to_path_buf());

        cleanup_checkin_tables();

        let provider = ProviderManager::new()
            .create(CreateProviderRequest {
                name: "CustomSite".to_string(),
                base_url: "https://custom.example.com".to_string(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
                builtin_id: None,
            })
            .unwrap();

        let account_manager = AccountManager::new(temp_dir.path());
        let account = account_manager
            .create(CreateAccountRequest {
                provider_id: provider.id.clone(),
                name: "disabled-acct".to_string(),
                cookies_json: ccr_core::Secret::from(r#"{"session":"abc"}"#),
                api_user: String::new(),
                extra_config: "{}".to_string(),
            })
            .unwrap();
        account_manager
            .update(
                &account.id,
                UpdateAccountRequest {
                    name: None,
                    cookies_json: None,
                    api_user: None,
                    enabled: Some(false),
                    extra_config: None,
                },
            )
            .unwrap();

        let result = service.checkin(&account.id).await.unwrap();
        assert_eq!(result.status, CheckinStatus::Skipped);
        assert_eq!(result.skip_reason.as_deref(), Some("account_disabled"));

        cleanup_checkin_tables();
    }
}
