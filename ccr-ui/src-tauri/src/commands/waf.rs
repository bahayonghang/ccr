//! WAF bypass 命令 — WebView 窗口 cookie 提取。
//!
//! 流程：
//! 1. 创建独立 WebView 窗口，导航到目标登录页面
//! 2. 注入 JS 轮询 `document.cookie` 作为兼容信号
//! 3. `open_waf_login` 持续读取 WebView runtime cookie store
//! 4. 按 provider WAF 策略确认 required cookies 齐全后再缓存
//! 5. 返回不含 cookie 值的结构化恢复结果

use crate::state::AppState;
use ccr_checkin::managers::checkin::waf_cookie_manager::{WafCookiePolicy, WafCookieSelection};
use ccr_checkin::managers::checkin::{ProviderManager, WafCookieManager, get_checkin_dir};
use ccr_checkin::{CheckinService, WafCookieValidationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{Manager, State, WebviewWindowBuilder};

// ── 全局 pending cookie 投递表 ──

/// 每个 provider_id 对应注入 JS 最近一次看到的 document.cookie。
/// WebView cookie store 是主来源；该表只作为兼容补充信号。
static PENDING_COOKIES: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

fn with_pending<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<String, String>) -> R,
{
    let mut guard = PENDING_COOKIES.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    f(map)
}

fn latest_delivered_cookie(provider_id: &str) -> Option<String> {
    with_pending(|map| map.get(provider_id).cloned())
}

fn clear_delivered_cookie(provider_id: &str) {
    with_pending(|map| {
        map.remove(provider_id);
    });
}

// ── WAF Cookie 提取 JS（initialization_script）──

/// 在每次页面加载前注入，轮询 document.cookie 并通过 Tauri IPC 回传。
/// provider_id 在构建时通过字符串插值写入脚本。
fn build_cookie_script(provider_id: &str) -> String {
    let provider_id_literal =
        serde_json::to_string(provider_id).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        r#"
(function() {{
    // 避免重复启动轮询
    if (window.__wafCookiePolling) return;
    window.__wafCookiePolling = true;

    var _providerId = {provider_id_literal};

    function trySend(cookies) {{
        // Tauri v2：通过 __TAURI_INTERNALS__.invoke 调用后端命令
        var internals = window.__TAURI_INTERNALS__;
        if (internals && internals.invoke) {{
            internals.invoke('waf_deliver_cookie', {{
                providerId: _providerId,
                cookie: cookies
            }}).then(function() {{
                clearInterval(window.__wafTimer);
            }}).catch(function() {{}});
        }}
    }}

    function poll() {{
        try {{
            var c = document.cookie;
            if (c && c.trim().length > 0) {{
                trySend(c);
            }}
        }} catch(e) {{}}
    }}

    // 立即尝试一次，之后每 500ms 轮询
    poll();
    window.__wafTimer = setInterval(poll, 500);
}})();
"#,
        provider_id_literal = provider_id_literal
    )
}

// ── 数据结构 ──

/// WAF Cookie 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WafCookieStatus {
    pub provider_id: String,
    pub has_cookie: bool,
    pub expires_at: Option<String>,
    pub cookie_names: Vec<String>,
    pub required_cookie_names: Vec<String>,
    pub missing_cookie_names: Vec<String>,
}

/// WAF Cookie 自动恢复结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WafCookieRecoveryResult {
    pub provider_id: String,
    pub provider_name: String,
    pub found_cookie_names: Vec<String>,
    pub missing_cookie_names: Vec<String>,
    pub required_cookie_names: Vec<String>,
    pub persisted: bool,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Clone)]
struct WafRecoveryContext {
    provider_name: String,
    policy: Option<WafCookiePolicy>,
}

// ── 命令实现 ──

/// WebView 内 JS 调用此命令将 cookie 字符串投递给等待的 open_waf_login。
///
/// 此命令由注入的 JS 脚本通过 Tauri IPC invoke 调用，不应由前端直接调用。
#[tauri::command]
pub async fn waf_deliver_cookie(provider_id: String, cookie: String) -> Result<(), String> {
    if cookie.trim().is_empty() {
        return Ok(());
    }

    with_pending(|map| {
        map.insert(provider_id, cookie);
    });

    Ok(())
}

async fn load_recovery_context(provider_id: &str, login_url: &str) -> WafRecoveryContext {
    let provider_id_for_lookup = provider_id.to_string();
    let provider_result = tokio::task::spawn_blocking(move || {
        let provider_manager = ProviderManager::new();
        provider_manager.get(&provider_id_for_lookup)
    })
    .await
    .ok()
    .and_then(Result::ok);

    match provider_result {
        Some(provider) => WafRecoveryContext {
            provider_name: provider.name.clone(),
            policy: WafCookieManager::policy_for_provider(&provider),
        },
        None => WafRecoveryContext {
            provider_name: provider_id.to_string(),
            policy: WafCookieManager::policy_for_provider_parts(provider_id, None, login_url),
        },
    }
}

fn read_cookie_store<R: tauri::Runtime>(
    webview_window: &tauri::WebviewWindow<R>,
    url: tauri::Url,
) -> Result<HashMap<String, String>, String> {
    let mut cookies_map = HashMap::new();
    let cookies = webview_window
        .cookies_for_url(url)
        .map_err(|e| format!("读取 WebView Cookie Store 失败: {}", e))?;

    for cookie in cookies {
        let name = cookie.name().trim();
        let value = cookie.value().trim();
        if !name.is_empty() && !value.is_empty() {
            cookies_map.insert(name.to_string(), value.to_string());
        }
    }

    Ok(cookies_map)
}

fn source_label(has_store_cookies: bool, has_document_cookies: bool) -> String {
    match (has_store_cookies, has_document_cookies) {
        (true, true) => "webview_store+document_cookie",
        (true, false) => "webview_store",
        (false, true) => "document_cookie",
        (false, false) => "none",
    }
    .to_string()
}

fn select_cookies_for_policy(
    cookies: &HashMap<String, String>,
    policy: Option<&WafCookiePolicy>,
) -> WafCookieSelection {
    let required = policy
        .map(|policy| policy.required_cookie_names.as_slice())
        .unwrap_or(&[]);
    WafCookieManager::select_required_cookies(cookies, required)
}

fn build_recovery_result(
    provider_id: &str,
    provider_name: &str,
    policy: Option<&WafCookiePolicy>,
    selection: WafCookieSelection,
    persisted: bool,
    source: String,
) -> WafCookieRecoveryResult {
    let required_cookie_names = policy
        .map(|policy| policy.required_cookie_names.clone())
        .unwrap_or_default();
    let message = if persisted {
        if selection.found_cookie_names.is_empty() {
            "WAF Cookie 已获取并缓存".to_string()
        } else {
            format!(
                "WAF Cookie 已获取并缓存: {}",
                selection.found_cookie_names.join(", ")
            )
        }
    } else if !selection.missing_cookie_names.is_empty() {
        format!(
            "WAF Cookie 未获取完整，缺少: {}",
            selection.missing_cookie_names.join(", ")
        )
    } else {
        "等待超时，未检测到可保存的 WAF Cookie".to_string()
    };

    WafCookieRecoveryResult {
        provider_id: provider_id.to_string(),
        provider_name: provider_name.to_string(),
        found_cookie_names: selection.found_cookie_names,
        missing_cookie_names: selection.missing_cookie_names,
        required_cookie_names,
        persisted,
        source,
        message,
    }
}

/// 打开 WAF 登录窗口，等待用户完成登录后提取 cookie 并缓存。
///
/// 返回结构化恢复结果，不包含 cookie 值。
/// 只有 provider required cookies 齐全后才会写入 WafCookieManager。
#[tauri::command]
pub async fn open_waf_login(
    app: tauri::AppHandle,
    login_url: String,
    provider_id: String,
) -> Result<WafCookieRecoveryResult, String> {
    let safe_provider_id: String = provider_id
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect();
    // 构造唯一窗口标签
    let window_label = format!("waf-login-{}", safe_provider_id);

    // 若已有同名窗口则先关闭（避免重复打开）
    if let Some(existing) = app.get_webview_window(&window_label) {
        let _ = existing.close();
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    clear_delivered_cookie(&provider_id);
    let context = load_recovery_context(&provider_id, &login_url).await;

    // 解析登录 URL
    let url = login_url
        .parse::<tauri::Url>()
        .map_err(|e| format!("无效的登录 URL: {}", e))?;
    let cookie_url = url.clone();

    // 构建注入脚本（在每次页面加载前运行）
    let init_script = build_cookie_script(&provider_id);

    // 创建 WebView 窗口
    // initialization_script 确保脚本在页面 JS 执行前已注入
    //
    // 代理出口对齐：WAF cookie 与来源 IP 绑定，WebView 必须与签到 reqwest 走同一出口
    // （见 state.rs 共享 http_client 的代理配置），统一来源 env→Windows 注册表。
    let mut builder =
        WebviewWindowBuilder::new(&app, &window_label, tauri::WebviewUrl::External(url))
            .title("WAF 登录")
            .inner_size(900.0, 700.0)
            .resizable(true)
            .visible(false) // 默认隐藏，实现无感绕过
            .initialization_script(&init_script);
    if let Some(proxy_url) = ccr_checkin::resolve_checkin_proxy_url() {
        match proxy_url.parse::<tauri::Url>() {
            // Tauri WebView 仅支持 http/socks5 代理；其余 scheme 跳过以免 build 失败，回退系统出口
            Ok(parsed) if matches!(parsed.scheme(), "http" | "socks5") => {
                builder = builder.proxy_url(parsed)
            }
            Ok(parsed) => {
                tracing::warn!(
                    scheme = parsed.scheme(),
                    "WAF WebView 不支持该代理协议，回退系统默认出口"
                )
            }
            Err(e) => {
                tracing::warn!(error = %e, "WAF WebView 代理 URL 无效，回退系统默认出口")
            }
        }
    }
    let webview_window = builder.build().map_err(|e| {
        clear_delivered_cookie(&provider_id);
        format!("创建 WebView 窗口失败: {}", e)
    })?;

    // 如果 3 秒后仍未完成（可能需要真人点按或滑块），则显示窗口
    let window_clone = webview_window.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        // 如果窗口还没关，就显示出来给用户
        let _ = window_clone.show();
    });

    // 给 WebView2 冷启动 + WAF JS 挑战 + 可能的人工介入留足时间（旧值 60s 偏紧）
    let timeout_duration = Duration::from_secs(120);
    let started_at = Instant::now();

    loop {
        let mut combined_cookies = HashMap::new();
        let mut has_document_cookies = false;
        let mut has_store_cookies = false;

        if let Some(cookie_str) = latest_delivered_cookie(&provider_id) {
            let document_cookies = WafCookieManager::parse_cookie_pairs(&cookie_str);
            has_document_cookies = !document_cookies.is_empty();
            combined_cookies.extend(document_cookies);
        }

        match read_cookie_store(&webview_window, cookie_url.clone()) {
            Ok(store_cookies) => {
                has_store_cookies = !store_cookies.is_empty();
                combined_cookies.extend(store_cookies);
            }
            Err(error) => {
                tracing::debug!(
                    provider_id = %provider_id,
                    error = %error,
                    "读取 WebView Cookie Store 失败，继续等待"
                );
            }
        }

        let source = source_label(has_store_cookies, has_document_cookies);
        let selection = select_cookies_for_policy(&combined_cookies, context.policy.as_ref());
        let cookies_to_save = selection.cookies.clone();
        let can_persist = selection.is_complete();

        if can_persist {
            let provider_id_for_save = provider_id.clone();
            tokio::task::spawn_blocking(move || {
                let waf_manager = WafCookieManager::new();
                waf_manager
                    .save(&provider_id_for_save, cookies_to_save)
                    .map_err(|e| format!("保存 WAF cookie 失败: {}", e))
            })
            .await
            .map_err(|e| format!("Task join error: {}", e))??;

            tracing::info!(
                provider_id = %provider_id,
                found_cookie_names = ?selection.found_cookie_names,
                source = %source,
                "WAF cookie 已提取并缓存"
            );

            let _ = webview_window.close();
            clear_delivered_cookie(&provider_id);
            return Ok(build_recovery_result(
                &provider_id,
                &context.provider_name,
                context.policy.as_ref(),
                selection,
                true,
                source,
            ));
        }

        if started_at.elapsed() >= timeout_duration {
            let _ = webview_window.close();
            clear_delivered_cookie(&provider_id);
            return Ok(build_recovery_result(
                &provider_id,
                &context.provider_name,
                context.policy.as_ref(),
                selection,
                false,
                source,
            ));
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// 查询指定 provider 的 WAF cookie 缓存状态。
#[tauri::command]
pub async fn get_waf_cookie_status(provider_id: String) -> Result<WafCookieStatus, String> {
    let provider_id_for_query = provider_id.clone();

    tokio::task::spawn_blocking(move || {
        let provider = ProviderManager::new().get(&provider_id_for_query).ok();
        let policy = provider
            .as_ref()
            .and_then(WafCookieManager::policy_for_provider)
            .or_else(|| {
                WafCookieManager::policy_for_provider_parts(&provider_id_for_query, None, "")
            });
        let required_cookie_names = policy
            .as_ref()
            .map(|policy| policy.required_cookie_names.clone())
            .unwrap_or_default();
        let waf_manager = WafCookieManager::new();

        match waf_manager.get_valid(&provider_id_for_query) {
            Ok(Some(cookies)) => {
                let selection = select_cookies_for_policy(&cookies, policy.as_ref());
                let mut cookie_names: Vec<String> = cookies.keys().cloned().collect();
                cookie_names.sort();
                // 有效缓存存在（WafCookieManager 内部已处理过期清理）
                Ok(WafCookieStatus {
                    provider_id: provider_id_for_query,
                    has_cookie: selection.is_complete(),
                    expires_at: None,
                    cookie_names,
                    required_cookie_names,
                    missing_cookie_names: selection.missing_cookie_names,
                })
            }
            Ok(None) => {
                // 无缓存或已过期
                Ok(WafCookieStatus {
                    provider_id: provider_id_for_query,
                    has_cookie: false,
                    expires_at: None,
                    cookie_names: Vec::new(),
                    missing_cookie_names: required_cookie_names.clone(),
                    required_cookie_names,
                })
            }
            Err(e) => Err(format!("查询 WAF cookie 状态失败: {}", e)),
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// 验证缓存的 WAF Cookie 是否能通过用户信息接口。
#[tauri::command]
pub async fn validate_waf_cookie_for_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<WafCookieValidationResult, String> {
    let checkin_dir = get_checkin_dir().map_err(|e| format!("Failed to get checkin dir: {}", e))?;
    let service = CheckinService::with_client(checkin_dir, state.http_client.clone());
    service
        .validate_cached_waf_access(&account_id)
        .await
        .map_err(|e| format!("验证 WAF Cookie 失败: {}", e))
}
