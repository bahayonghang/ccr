//! WAF bypass 命令 — WebView 窗口 cookie 提取。
//!
//! 流程：
//! 1. 创建独立 WebView 窗口，导航到目标登录页面
//! 2. 通过 initialization_script 注入 JS，轮询 document.cookie
//!    并通过 Tauri IPC invoke 回调 `waf_deliver_cookie` 命令
//! 3. `waf_deliver_cookie` 将 cookie 写入全局 pending map 并触发 oneshot
//! 4. `open_waf_login` 等待 cookie 或 60 秒超时后关闭窗口
//! 5. cookie 保存到 WafCookieManager（SQLite，默认 24h 缓存）

use ccr_db::managers::checkin::WafCookieManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Manager, WebviewWindowBuilder};
use tokio::sync::oneshot;

// ── 全局 pending cookie 投递表 ──

/// 每个 provider_id 对应一个等待接收 cookie 的 oneshot sender。
/// open_waf_login 注册后等待 waf_deliver_cookie 触发。
static PENDING_COOKIES: Mutex<Option<HashMap<String, oneshot::Sender<String>>>> = Mutex::new(None);

fn with_pending<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<String, oneshot::Sender<String>>) -> R,
{
    let mut guard = PENDING_COOKIES.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    f(map)
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

    // 从 pending map 中取出 sender 并发送 cookie
    let sender = with_pending(|map| map.remove(&provider_id));
    if let Some(tx) = sender {
        let _ = tx.send(cookie);
    }

    Ok(())
}

/// 打开 WAF 登录窗口，等待用户完成登录后提取 cookie 并缓存。
///
/// 返回提取到的 cookie 字符串（`key=value; key=value; ...` 格式）。
/// 60 秒内未获取到 cookie 则超时返回错误。
#[tauri::command]
pub async fn open_waf_login(
    app: tauri::AppHandle,
    login_url: String,
    provider_id: String,
) -> Result<String, String> {
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

    // 注册 oneshot channel，供 waf_deliver_cookie 触发
    let (tx, rx) = oneshot::channel::<String>();
    with_pending(|map| {
        map.insert(provider_id.clone(), tx);
    });

    // 解析登录 URL
    let url = login_url
        .parse::<tauri::Url>()
        .map_err(|e| format!("无效的登录 URL: {}", e))?;

    // 构建注入脚本（在每次页面加载前运行）
    let init_script = build_cookie_script(&provider_id);

    // 创建 WebView 窗口
    // initialization_script 确保脚本在页面 JS 执行前已注入
    let webview_window =
        WebviewWindowBuilder::new(&app, &window_label, tauri::WebviewUrl::External(url))
            .title("WAF 登录")
            .inner_size(900.0, 700.0)
            .resizable(true)
            .visible(false) // 默认隐藏，实现无感绕过
            .initialization_script(&init_script)
            .build()
            .map_err(|e| {
                // 构建失败时清理 pending map
                with_pending(|map| {
                    map.remove(&provider_id);
                });
                format!("创建 WebView 窗口失败: {}", e)
            })?;

    // 如果 3 秒后仍未完成（可能需要真人点按或滑块），则显示窗口
    let window_clone = webview_window.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        // 如果窗口还没关，就显示出来给用户
        let _ = window_clone.show();
    });

    // 等待 cookie 或 60 秒超时
    let timeout_duration = std::time::Duration::from_secs(60);
    let result = tokio::time::timeout(timeout_duration, rx).await;

    // 关闭 WebView 窗口
    let _ = webview_window.close();

    // 清理 pending map（超时或错误时）
    with_pending(|map| {
        map.remove(&provider_id);
    });

    // 处理结果
    match result {
        Ok(Ok(cookie_str)) => {
            if cookie_str.trim().is_empty() {
                return Err("获取到的 cookie 为空".to_string());
            }

            // 将 cookie 字符串解析为 HashMap 并缓存到 WafCookieManager
            let mut cookies_map = HashMap::new();
            for pair in cookie_str.split(';') {
                let pair = pair.trim();
                if let Some((k, v)) = pair.split_once('=') {
                    cookies_map.insert(k.trim().to_string(), v.trim().to_string());
                }
            }

            if !cookies_map.is_empty() {
                let provider_id_for_save = provider_id.clone();
                tokio::task::spawn_blocking(move || {
                    let waf_manager = WafCookieManager::new();
                    waf_manager
                        .save(&provider_id_for_save, cookies_map)
                        .map_err(|e| format!("保存 WAF cookie 失败: {}", e))
                })
                .await
                .map_err(|e| format!("Task join error: {}", e))??;

                tracing::info!("[waf] provider {} 的 WAF cookie 已提取并缓存", provider_id);
            }

            Ok(cookie_str)
        }
        Ok(Err(_)) => {
            // sender 已丢弃（不应发生，防御性处理）
            Err("WAF cookie 接收通道意外关闭".to_string())
        }
        Err(_) => {
            // 60 秒超时
            Err("WAF 登录超时（60 秒内未检测到 cookie）".to_string())
        }
    }
}

/// 查询指定 provider 的 WAF cookie 缓存状态。
#[tauri::command]
pub async fn get_waf_cookie_status(provider_id: String) -> Result<WafCookieStatus, String> {
    let provider_id_for_query = provider_id.clone();

    tokio::task::spawn_blocking(move || {
        let waf_manager = WafCookieManager::new();

        match waf_manager.get_valid(&provider_id_for_query) {
            Ok(Some(_cookies)) => {
                // 有效缓存存在（WafCookieManager 内部已处理过期清理）
                Ok(WafCookieStatus {
                    provider_id: provider_id_for_query,
                    has_cookie: true,
                    expires_at: None,
                })
            }
            Ok(None) => {
                // 无缓存或已过期
                Ok(WafCookieStatus {
                    provider_id: provider_id_for_query,
                    has_cookie: false,
                    expires_at: None,
                })
            }
            Err(e) => Err(format!("查询 WAF cookie 状态失败: {}", e)),
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
