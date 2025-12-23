// WAF 绕过服务
// 通过启动本地 Chromium 浏览器访问登录页，获取 WAF 相关 cookies（acw_tc / cdn_sec_tc / acw_sc__v2）

use chromiumoxide::browser::{Browser, BrowserConfig};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::task::JoinHandle;

const REQUIRED_WAF_COOKIES: &[&str] = &["acw_tc", "cdn_sec_tc", "acw_sc__v2"];
const DEFAULT_WAF_WAIT: Duration = Duration::from_secs(8);
const DEFAULT_BROWSER_LAUNCH_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_BROWSER_CLOSE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum WafBypassError {
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
    #[error("Failed to navigate to login page: {0}")]
    NavigationError(String),
    #[error("Failed to get cookies: {0}")]
    CookiesError(String),
    #[error("No WAF cookies obtained (expected one of: {0:?})")]
    EmptyCookies(Vec<&'static str>),
}

pub type Result<T> = std::result::Result<T, WafBypassError>;

pub struct WafBypassService {
    headless: bool,
    proxy_url: Option<String>,
    user_agent: String,
    waf_wait: Duration,
}

impl WafBypassService {
    pub fn new(headless: bool, proxy_url: Option<String>, user_agent: String) -> Self {
        Self {
            headless,
            proxy_url,
            user_agent,
            waf_wait: DEFAULT_WAF_WAIT,
        }
    }

    pub async fn get_waf_cookies(&self, login_url: &str, account_name: &str) -> Result<HashMap<String, String>> {
        tracing::info!("[{}] WAF bypass: fetching cookies via browser", account_name);

        let (browser, handler_task, temp_dir) = self.launch_browser(account_name).await?;

        let cookies_result = self
            .navigate_and_extract_cookies(&browser, login_url, account_name)
            .await;

        self.cleanup(browser, handler_task, &temp_dir, account_name).await;

        cookies_result
    }

    async fn launch_browser(&self, account_name: &str) -> Result<(Browser, JoinHandle<()>, PathBuf)> {
        let temp_dir = std::env::temp_dir().join(format!("chromiumoxide-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| WafBypassError::TempDirError(e.to_string()))?;

        let browser_path = Self::find_browser().ok_or(WafBypassError::BrowserNotFound)?;

        let mut builder = BrowserConfig::builder()
            .window_size(1920, 1080)
            .no_sandbox()
            .user_data_dir(&temp_dir)
            .chrome_executable(&browser_path);

        if let Some(proxy_url) = self.proxy_url.as_deref() {
            builder = builder.arg(format!("--proxy-server={}", proxy_url));
        }

        if !self.headless {
            builder = builder.with_head();
        }

        let config = builder
            .build()
            .map_err(|e| WafBypassError::BrowserConfigError(e.to_string()))?;

        let launch_result = tokio::time::timeout(DEFAULT_BROWSER_LAUNCH_TIMEOUT, Browser::launch(config)).await;
        let (browser, mut handler) = match launch_result {
            Ok(Ok(browser_handler)) => browser_handler,
            Ok(Err(e)) => return Err(WafBypassError::BrowserLaunchError(e.to_string())),
            Err(_) => return Err(WafBypassError::BrowserLaunchTimeout),
        };

        let handler_task = tokio::spawn(async move {
            while let Some(_event) = handler.next().await {}
        });

        tracing::info!("[{}] WAF bypass: browser launched", account_name);
        Ok((browser, handler_task, temp_dir))
    }

    async fn navigate_and_extract_cookies(
        &self,
        browser: &Browser,
        login_url: &str,
        account_name: &str,
    ) -> Result<HashMap<String, String>> {
        tracing::info!("[{}] WAF bypass: navigating to {}", account_name, login_url);

        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| WafBypassError::NewPageError(e.to_string()))?;

        page.set_user_agent(&self.user_agent)
            .await
            .map_err(|e| WafBypassError::UserAgentError(e.to_string()))?;

        page.goto(login_url)
            .await
            .map_err(|e| WafBypassError::NavigationError(e.to_string()))?;

        tokio::time::sleep(self.waf_wait).await;

        let cookies = page
            .get_cookies()
            .await
            .map_err(|e| WafBypassError::CookiesError(e.to_string()))?;

        let mut waf_cookies = HashMap::new();
        for cookie in cookies {
            if REQUIRED_WAF_COOKIES.contains(&cookie.name.as_str()) {
                waf_cookies.insert(cookie.name.clone(), cookie.value.clone());
            }
        }

        if waf_cookies.is_empty() {
            return Err(WafBypassError::EmptyCookies(REQUIRED_WAF_COOKIES.to_vec()));
        }

        tracing::info!(
            "[{}] WAF bypass: captured {} cookies",
            account_name,
            waf_cookies.len()
        );
        Ok(waf_cookies)
    }

    async fn cleanup(&self, mut browser: Browser, handler_task: JoinHandle<()>, temp_dir: &PathBuf, account_name: &str) {
        handler_task.abort();

        let _ = tokio::time::timeout(DEFAULT_BROWSER_CLOSE_TIMEOUT, browser.close()).await;

        match std::fs::remove_dir_all(temp_dir) {
            Ok(()) => tracing::debug!("[{}] WAF bypass: cleaned temp dir", account_name),
            Err(e) => tracing::warn!(
                "[{}] WAF bypass: failed to clean temp dir: {}",
                account_name,
                e
            ),
        }
    }

    fn find_browser() -> Option<PathBuf> {
        let browser_paths = vec![
            // macOS
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            // Linux
            "/usr/bin/google-chrome",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/usr/bin/brave-browser",
            "/usr/bin/microsoft-edge",
            "/snap/bin/chromium",
            "/opt/google/chrome/chrome",
            "/opt/chromium/chromium",
        ];

        for path in browser_paths {
            let browser_path = PathBuf::from(path);
            if browser_path.exists() {
                return Some(browser_path);
            }
        }

        #[cfg(target_os = "windows")]
        {
            let windows_paths: Vec<PathBuf> = vec![
                PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
                PathBuf::from(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"),
                PathBuf::from(r"C:\Program Files\Chromium\Application\chrome.exe"),
                PathBuf::from(r"C:\Program Files (x86)\Chromium\Application\chrome.exe"),
                PathBuf::from(r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe"),
                PathBuf::from(r"C:\Program Files (x86)\BraveSoftware\Brave-Browser\Application\brave.exe"),
                PathBuf::from(r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"),
                PathBuf::from(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
            ];

            for path in windows_paths {
                if path.exists() {
                    return Some(path);
                }
            }

            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let candidates = vec![
                    PathBuf::from(&local_app_data).join("Google/Chrome/Application/chrome.exe"),
                    PathBuf::from(&local_app_data).join("BraveSoftware/Brave-Browser/Application/brave.exe"),
                ];
                for path in candidates {
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }

        None
    }
}
