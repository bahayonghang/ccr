// Cloudflare 绕过服务
// 通过启动本地 Chromium 浏览器访问目标页面，等待 Cloudflare 挑战完成，获取 cf_clearance cookie
// 复用 WafBypassService::find_browser() 进行浏览器发现

use chromiumoxide::browser::{Browser, BrowserConfig};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::task::JoinHandle;

use crate::services::waf_bypass::WafBypassService;

const CF_COOKIE_NAME: &str = "cf_clearance";
/// CF 挑战轮询间隔
const CF_POLL_INTERVAL: Duration = Duration::from_millis(500);
/// 最大轮询次数（60 × 500ms = 30s）
const CF_MAX_POLL_ATTEMPTS: u32 = 60;
const DEFAULT_BROWSER_LAUNCH_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_BROWSER_CLOSE_TIMEOUT: Duration = Duration::from_secs(10);

/// Stealth JS: 移除 headless 检测标记，伪装为真实浏览器
/// CF 比 WAF 更严格地检测 headless，此脚本至关重要
const CF_STEALTH_JS: &str = r#"
    // 移除 navigator.webdriver 标记
    Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
    // 伪装 Chrome runtime
    window.chrome = { runtime: {}, loadTimes: function(){}, csi: function(){} };
    // 伪装 permissions API
    const originalQuery = window.navigator.permissions.query;
    window.navigator.permissions.query = (parameters) => (
        parameters.name === 'notifications' ?
            Promise.resolve({ state: Notification.permission }) :
            originalQuery(parameters)
    );
    // 伪装 plugins（headless 通常为空）
    Object.defineProperty(navigator, 'plugins', {
        get: () => [1, 2, 3, 4, 5]
    });
    // 伪装 languages
    Object.defineProperty(navigator, 'languages', {
        get: () => ['zh-CN', 'zh', 'en-US', 'en']
    });
"#;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum CfBypassError {
    #[error("No Chromium-based browser found (Chrome/Chromium/Brave/Edge)")]
    BrowserNotFound,
    #[error("Failed to create temp profile directory: {0}")]
    TempDirError(String),
    #[error("Failed to build browser config: {0}")]
    BrowserConfigError(String),
    #[error("Failed to launch browser: {0}")]
    BrowserLaunchError(String),
    #[error("Browser launch timed out")]
    BrowserLaunchTimeout,
    #[error("Failed to create new page: {0}")]
    NewPageError(String),
    #[error("Failed to set user agent: {0}")]
    UserAgentError(String),
    #[error("Failed to navigate: {0}")]
    NavigationError(String),
    #[error("Failed to get cookies: {0}")]
    CookiesError(String),
    #[error("Cloudflare challenge not resolved within timeout")]
    ChallengeTimeout,
    #[error("No cf_clearance cookie obtained")]
    NoCfClearance,
}

pub type Result<T> = std::result::Result<T, CfBypassError>;

pub struct CfBypassService {
    headless: bool,
    proxy_url: Option<String>,
    user_agent: String,
}

impl CfBypassService {
    pub fn new(headless: bool, proxy_url: Option<String>, user_agent: String) -> Self {
        Self {
            headless,
            proxy_url,
            user_agent,
        }
    }

    /// 获取 cf_clearance cookie
    ///
    /// 通过浏览器访问目标站点，等待 Cloudflare 挑战自动完成后提取 cf_clearance。
    /// 注意：Cloudflare 可能检测 headless 模式，如果 headless 获取失败，
    /// 建议在带 GUI 的环境中以 `headless=false` 重试。
    pub async fn get_cf_cookies(
        &self,
        target_url: &str,
        account_name: &str,
    ) -> Result<HashMap<String, String>> {
        tracing::info!(
            "[{}] CF bypass: fetching cf_clearance via browser",
            account_name
        );

        let (browser, handler_task, temp_dir) = self.launch_browser(account_name).await?;

        let cookies_result = self
            .navigate_and_wait_for_clearance(&browser, target_url, account_name)
            .await;

        self.cleanup(browser, handler_task, &temp_dir, account_name)
            .await;

        cookies_result
    }

    async fn launch_browser(
        &self,
        account_name: &str,
    ) -> Result<(Browser, JoinHandle<()>, PathBuf)> {
        let temp_dir =
            std::env::temp_dir().join(format!("chromiumoxide-cf-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| CfBypassError::TempDirError(e.to_string()))?;

        // 复用 WafBypassService 的浏览器发现逻辑（DRY）
        let browser_path =
            WafBypassService::find_browser().ok_or(CfBypassError::BrowserNotFound)?;

        let mut builder = BrowserConfig::builder()
            .window_size(1920, 1080)
            .no_sandbox()
            .user_data_dir(&temp_dir)
            .chrome_executable(&browser_path)
            // 反 headless 检测参数（CF 检测比 WAF 更严格）
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--disable-features=IsolateOrigins,site-per-process")
            .arg("--disable-infobars")
            .arg("--disable-dev-shm-usage");

        if let Some(proxy_url) = self.proxy_url.as_deref() {
            builder = builder.arg(format!("--proxy-server={}", proxy_url));
        }

        if !self.headless {
            builder = builder.with_head();
        }

        let config = builder
            .build()
            .map_err(|e| CfBypassError::BrowserConfigError(e.to_string()))?;

        let launch_result =
            tokio::time::timeout(DEFAULT_BROWSER_LAUNCH_TIMEOUT, Browser::launch(config)).await;
        let (browser, mut handler) = match launch_result {
            Ok(Ok(browser_handler)) => browser_handler,
            Ok(Err(e)) => return Err(CfBypassError::BrowserLaunchError(e.to_string())),
            Err(_) => return Err(CfBypassError::BrowserLaunchTimeout),
        };

        let handler_task =
            tokio::spawn(async move { while let Some(_event) = handler.next().await {} });

        tracing::info!("[{}] CF bypass: browser launched", account_name);
        Ok((browser, handler_task, temp_dir))
    }

    async fn navigate_and_wait_for_clearance(
        &self,
        browser: &Browser,
        target_url: &str,
        account_name: &str,
    ) -> Result<HashMap<String, String>> {
        tracing::info!("[{}] CF bypass: navigating to {}", account_name, target_url);

        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| CfBypassError::NewPageError(e.to_string()))?;

        page.set_user_agent(&self.user_agent)
            .await
            .map_err(|e| CfBypassError::UserAgentError(e.to_string()))?;

        // 在导航前注入反检测 JS（about:blank 阶段执行，确保后续页面加载时生效）
        if let Err(e) = page.evaluate(CF_STEALTH_JS).await {
            tracing::warn!(
                "[{}] CF bypass: stealth JS injection failed: {}",
                account_name,
                e
            );
        }

        page.goto(target_url)
            .await
            .map_err(|e| CfBypassError::NavigationError(e.to_string()))?;

        // 轮询等待 Cloudflare 挑战解决
        // 检测页面标题从 "Just a moment..." / "Checking your browser" 变为实际站点标题
        let mut challenge_detected = false;
        let mut resolved = false;

        for attempt in 0..CF_MAX_POLL_ATTEMPTS {
            tokio::time::sleep(CF_POLL_INTERVAL).await;

            let title = page
                .evaluate("document.title")
                .await
                .ok()
                .and_then(|v| v.into_value::<String>().ok())
                .unwrap_or_default();

            let is_challenge = title.contains("Just a moment")
                || title.contains("Checking")
                || title.contains("Cloudflare");

            if is_challenge {
                challenge_detected = true;
                if attempt % 10 == 0 {
                    tracing::debug!(
                        "[{}] CF bypass: waiting for challenge (attempt {}/{}), title: \"{}\"",
                        account_name,
                        attempt + 1,
                        CF_MAX_POLL_ATTEMPTS,
                        title
                    );
                }
                continue;
            }

            // 挑战解决（之前检测到挑战页，现在标题已变化）
            if challenge_detected {
                tracing::info!(
                    "[{}] CF bypass: challenge resolved after {} attempts, title: \"{}\"",
                    account_name,
                    attempt + 1,
                    title
                );
                resolved = true;
                break;
            }

            // 从未检测到挑战页面 — 可能站点此刻不需要 CF 验证
            if attempt >= 5 {
                tracing::debug!(
                    "[{}] CF bypass: no challenge detected after {} polls, proceeding",
                    account_name,
                    attempt + 1
                );
                resolved = true;
                break;
            }
        }

        if challenge_detected && !resolved {
            tracing::error!(
                "[{}] CF bypass: challenge not resolved within {} attempts",
                account_name,
                CF_MAX_POLL_ATTEMPTS
            );
            return Err(CfBypassError::ChallengeTimeout);
        }

        // 等待额外时间确保 cookies 完全写入
        tokio::time::sleep(Duration::from_secs(2)).await;

        // 提取 Cloudflare 相关 cookies
        let cookies = page
            .get_cookies()
            .await
            .map_err(|e| CfBypassError::CookiesError(e.to_string()))?;

        let mut cf_cookies = HashMap::new();
        for cookie in cookies {
            if cookie.name == CF_COOKIE_NAME
                || cookie.name.starts_with("cf_")
                || cookie.name.starts_with("__cf")
            {
                cf_cookies.insert(cookie.name.clone(), cookie.value.clone());
            }
        }

        if !cf_cookies.contains_key(CF_COOKIE_NAME) {
            tracing::warn!(
                "[{}] CF bypass: cf_clearance not found. Captured cookies: {:?}",
                account_name,
                cf_cookies.keys().collect::<Vec<_>>()
            );
            return Err(CfBypassError::NoCfClearance);
        }

        tracing::info!(
            "[{}] CF bypass: captured {} CF cookies (cf_clearance ✓)",
            account_name,
            cf_cookies.len()
        );

        Ok(cf_cookies)
    }

    async fn cleanup(
        &self,
        mut browser: Browser,
        handler_task: JoinHandle<()>,
        temp_dir: &PathBuf,
        account_name: &str,
    ) {
        handler_task.abort();

        let _ = tokio::time::timeout(DEFAULT_BROWSER_CLOSE_TIMEOUT, browser.close()).await;

        match std::fs::remove_dir_all(temp_dir) {
            Ok(()) => tracing::debug!("[{}] CF bypass: cleaned temp dir", account_name),
            Err(e) => tracing::warn!(
                "[{}] CF bypass: failed to clean temp dir: {}",
                account_name,
                e
            ),
        }
    }
}
