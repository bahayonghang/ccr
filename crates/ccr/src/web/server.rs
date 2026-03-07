// 🌐 Web 服务器核心
// 管理 HTTP 服务器的生命周期
// 🎯 异步架构 - 使用 Axum 提供高性能并发处理

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::ConfigManager;
use crate::services::{
    BackupService, ConfigService, HistoryService, SettingsService, ValidateService,
};
use crate::web::handlers::AppState;
use crate::web::system_info_cache::SystemInfoCache;
use axum::{
    Router,
    http::{HeaderValue, Method},
    response::{Html, IntoResponse},
    routing::get,
};
use once_cell::sync::Lazy;
use std::collections::BTreeSet;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::signal;
use tower_http::cors::{AllowHeaders, CorsLayer};

// 🎯 平台模式缓存 - 避免重复检测 （预留给将来的优化使用）
#[allow(dead_code)]
static PLATFORM_MODE: Lazy<RwLock<(bool, Option<std::path::PathBuf>)>> =
    Lazy::new(|| RwLock::new(ConfigManager::detect_unified_mode()));

// 🎯 路由注册宏 - 简化路由定义
macro_rules! routes {
    ($router:expr, $state:expr, {
        $( $method:ident $path:expr => $handler:path ),* $(,)?
    }) => {{
        $(
            $router = $router.route($path, axum::routing::$method($handler));
        )*
        $router.with_state($state)
    }};
}

/// 🌐 Web 服务器
///
/// 管理整个 Web 服务的核心结构
pub struct WebServer {
    config_service: Arc<ConfigService>,
    settings_service: Arc<SettingsService>,
    history_service: Arc<HistoryService>,
    backup_service: Arc<BackupService>,
    validate_service: Arc<ValidateService>,
    system_info_cache: Arc<SystemInfoCache>,
    host: std::net::IpAddr,
    port: u16,
}

#[allow(dead_code)]
impl WebServer {
    fn build_cors_layer() -> CorsLayer {
        let mut origins = BTreeSet::new();
        origins.insert("http://localhost:5173".to_string());
        origins.insert("http://127.0.0.1:5173".to_string());
        origins.insert("http://localhost:1420".to_string());
        origins.insert("http://127.0.0.1:1420".to_string());
        origins.insert("http://localhost:3000".to_string());
        origins.insert("http://127.0.0.1:3000".to_string());

        if let Ok(env_origins) = std::env::var("CCR_WEB_CORS_ORIGINS") {
            for origin in env_origins
                .split(',')
                .map(str::trim)
                .filter(|v| !v.is_empty())
            {
                origins.insert(origin.to_string());
            }
        }

        let allow_origins: Vec<HeaderValue> = origins
            .into_iter()
            .filter_map(|origin| match HeaderValue::from_str(&origin) {
                Ok(value) => Some(value),
                Err(error) => {
                    tracing::warn!("跳过非法 CORS Origin '{}': {}", origin, error);
                    None
                }
            })
            .collect();

        CorsLayer::new()
            .allow_origin(allow_origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(AllowHeaders::any())
    }

    /// 🏗️ 创建新的 Web 服务器
    pub fn new(host: std::net::IpAddr, port: u16) -> Result<Self> {
        let config_service = Arc::new(ConfigService::with_default()?);
        let settings_service = Arc::new(SettingsService::with_default()?);
        let history_service = Arc::new(HistoryService::with_default()?);
        let backup_service = Arc::new(BackupService::with_default()?);
        let validate_service = Arc::new(ValidateService::with_default()?);

        // 🎯 创建系统信息缓存，每 2 秒更新一次
        let system_info_cache = Arc::new(SystemInfoCache::new(Duration::from_secs(2)));

        Ok(Self {
            config_service,
            settings_service,
            history_service,
            backup_service,
            validate_service,
            system_info_cache,
            host,
            port,
        })
    }

    /// 🚀 启动服务器（异步版本）
    ///
    /// 使用 Axum 框架提供高性能异步 HTTP 服务
    ///
    /// 优势:
    /// - ⚡ 并发处理多个请求
    /// - 🎯 充分利用多核 CPU
    /// - 🔄 长时间操作不阻塞其他请求
    pub async fn start(&self, no_browser: bool) -> Result<()> {
        // 🎯 尝试绑定端口，如果失败则尝试其他端口
        // Windows 下可能遇到 WSAEACCES (os error 10013)，例如端口被系统/安全策略保留或禁止绑定到 0.0.0.0。
        let (listener, actual_port, bound_ip) =
            Self::bind_available_port(self.host, self.port).await?;

        ColorOutput::success("🌐 CCR Web 服务器已启动（异步模式）");

        // 🔍 检测 WSL 环境并获取 IP 地址
        let is_wsl = Self::detect_wsl_environment();
        let local_ip = Self::get_local_ip();
        let localhost_only = bound_ip.is_loopback();
        let bound_any = match bound_ip {
            std::net::IpAddr::V4(ip) => ip.is_unspecified(),
            std::net::IpAddr::V6(ip) => ip.is_unspecified(),
        };

        // 📍 输出访问地址
        if is_wsl {
            ColorOutput::info(&format!("📍 本地访问: http://localhost:{}", actual_port));
            if bound_any
                && !localhost_only
                && let Some(ip) = &local_ip
            {
                ColorOutput::info(&format!(
                    "📍 内网访问: http://{}:{} (推荐用于 Windows 主机)",
                    ip, actual_port
                ));
            } else if !bound_any && !localhost_only {
                ColorOutput::info(&format!("📍 绑定地址: http://{}:{}", bound_ip, actual_port));
            } else if localhost_only {
                ColorOutput::warning("⚠️ 当前仅绑定到 localhost，无法通过内网 IP 访问");
            } else {
                ColorOutput::warning("⚠️ 无法获取内网 IP 地址，请手动查看网络配置");
            }
        } else {
            if bound_any || localhost_only {
                ColorOutput::info(&format!("📍 地址: http://localhost:{}", actual_port));
            } else {
                ColorOutput::info(&format!("📍 地址: http://{}:{}", bound_ip, actual_port));
            }

            if bound_any
                && !localhost_only
                && let Some(ip) = &local_ip
            {
                ColorOutput::info(&format!("💡 内网访问: http://{}:{}", ip, actual_port));
            } else if localhost_only {
                ColorOutput::warning("⚠️ 当前仅绑定到 localhost，无法通过内网 IP 访问");
            }
        }

        ColorOutput::info("⏹️ 按 Ctrl+C 停止服务器");
        println!();

        // 🌐 根据参数决定是否打开浏览器
        // WSL 环境中不自动打开浏览器（避免打开 WSL 内部浏览器）
        if !no_browser && !is_wsl {
            let open_url = if bound_any || localhost_only {
                format!("http://localhost:{}", actual_port)
            } else {
                format!("http://{}:{}", bound_ip, actual_port)
            };
            if let Err(e) = open::that(open_url) {
                ColorOutput::warning(&format!("⚠️ 无法自动打开浏览器: {}", e));
                ColorOutput::info(&format!("💡 请手动访问 http://localhost:{}", actual_port));
            }
        } else if is_wsl {
            if let Some(ip) = local_ip {
                ColorOutput::info(&format!(
                    "💡 建议在 Windows 浏览器中访问: http://{}:{}",
                    ip, actual_port
                ));
            } else {
                ColorOutput::info(&format!("💡 请手动访问 http://localhost:{}", actual_port));
            }
        } else {
            ColorOutput::info(&format!("💡 请手动访问 http://localhost:{}", actual_port));
        }

        // 🎯 加载初始配置到缓存
        let initial_config = self.config_service.load_config()?;

        // 创建共享状态
        let state = AppState::new(
            self.config_service.clone(),
            self.settings_service.clone(),
            self.history_service.clone(),
            self.backup_service.clone(),
            self.validate_service.clone(),
            self.system_info_cache.clone(),
            initial_config,
        );

        // 🎯 构建路由 - 使用宏简化（从 20+ 行减少到 1 行！）
        let mut app = Router::new()
            // 静态文件
            .route("/", get(WebServer::serve_html))
            .route("/style.css", get(WebServer::serve_css))
            .route("/script.js", get(WebServer::serve_js));

        let app = routes!(
            app, state,
            {
                // 配置管理
                get  "/api/configs"                      => crate::web::handlers::config_handlers::handle_list_configs,
                post "/api/switch"                      => crate::web::handlers::config_handlers::handle_switch_config,
                post "/api/config"                      => crate::web::handlers::config_handlers::handle_add_config,
                get  "/api/config/{name}"               => crate::web::handlers::config_handlers::handle_get_config,
                put  "/api/config/{name}"               => crate::web::handlers::config_handlers::handle_update_config,
                delete "/api/config/{name}"             => crate::web::handlers::config_handlers::handle_delete_config,
                patch "/api/config/{name}/enable"       => crate::web::handlers::config_handlers::enable_config,
                patch "/api/config/{name}/disable"      => crate::web::handlers::config_handlers::disable_config,
                post "/api/export"                      => crate::web::handlers::config_handlers::handle_export,
                post "/api/import"                      => crate::web::handlers::config_handlers::handle_import,

                // Codex profiles
                get  "/api/codex/profiles"              => crate::web::handlers::codex_handlers::handle_list_codex_profiles,
                get  "/api/codex/profiles/{name}/env"   => crate::web::handlers::codex_handlers::handle_get_codex_profile_env,
                post "/api/codex/profiles"              => crate::web::handlers::codex_handlers::handle_add_codex_profile,
                put  "/api/codex/profiles/{name}"       => crate::web::handlers::codex_handlers::handle_update_codex_profile,
                delete "/api/codex/profiles/{name}"     => crate::web::handlers::codex_handlers::handle_delete_codex_profile,

                // 系统和设置
                get  "/api/history"                     => crate::web::handlers::system_handlers::handle_get_history,
                post "/api/validate"                    => crate::web::handlers::system_handlers::handle_validate,
                post "/api/clean"                       => crate::web::handlers::system_handlers::handle_clean,
                get  "/api/settings"                    => crate::web::handlers::system_handlers::handle_get_settings,
                get  "/api/settings/backups"            => crate::web::handlers::system_handlers::handle_get_settings_backups,
                post "/api/settings/restore"            => crate::web::handlers::system_handlers::handle_restore_settings,
                get  "/api/system"                      => crate::web::handlers::system_handlers::handle_get_system_info,
                post "/api/reload"                      => crate::web::handlers::system_handlers::handle_reload_config,

                // 统计
                get  "/api/stats/provider-usage"        => crate::web::handlers::stats_handlers::handle_provider_usage,

                // 成本追踪统计
                get  "/api/stats/cost/summary"          => crate::web::handlers::cost_handlers::handle_get_cost_summary,
                get  "/api/stats/cost/details"          => crate::web::handlers::cost_handlers::handle_get_cost_details,
                get  "/api/stats/cost/export"           => crate::web::handlers::cost_handlers::handle_export_costs,
                get  "/api/stats/cost/by-model"         => crate::web::handlers::cost_handlers::handle_get_model_usage,

                // 预算管理
                get  "/api/budget/status"               => crate::web::handlers::cost_handlers::handle_get_budget_status,
                post "/api/budget/set"                  => crate::web::handlers::cost_handlers::handle_set_budget,
                post "/api/budget/reset"                => crate::web::handlers::cost_handlers::handle_reset_budget,

                // 价格管理
                get  "/api/pricing/list"                => crate::web::handlers::cost_handlers::handle_list_pricing,
                post "/api/pricing/set"                 => crate::web::handlers::cost_handlers::handle_set_pricing,
                delete "/api/pricing/remove/{model}"    => crate::web::handlers::cost_handlers::handle_remove_pricing,
                post "/api/pricing/reset"               => crate::web::handlers::cost_handlers::handle_reset_pricing,

                // 平台管理 (Unified Mode)
                get  "/api/platforms"                   => crate::web::handlers::platform_handlers::handle_get_platform_info,
                post "/api/platforms/switch"            => crate::web::handlers::platform_handlers::handle_switch_platform,

                // 同步相关
                get  "/api/sync/status"                 => crate::web::handlers::sync_handlers::handle_sync_status,
                post "/api/sync/config"                 => crate::web::handlers::sync_handlers::handle_sync_config,
                post "/api/sync/push"                   => crate::web::handlers::sync_handlers::handle_sync_push,
                post "/api/sync/pull"                   => crate::web::handlers::sync_handlers::handle_sync_pull
            }
        );

        // 🎯 添加中间件
        let app = app.layer(Self::build_cors_layer()); // CORS 白名单

        // 🚀 启动服务器（支持 Ctrl+C 优雅退出）
        let shutdown_cache = Arc::clone(&self.system_info_cache);
        let shutdown_signal = async move {
            if let Err(e) = signal::ctrl_c().await {
                tracing::error!("监听 Ctrl+C 失败: {}", e);
                std::future::pending::<()>().await;
            }
            ColorOutput::warning("⚠️ 收到 Ctrl+C，正在停止服务器...");
            shutdown_cache.stop();

            // 🎯 给后台线程一点时间优雅退出（最多等待 500ms）
            tokio::time::sleep(Duration::from_millis(500)).await;
        };

        let serve_result = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await;

        // 确保后台线程停止（即使不是 Ctrl+C 触发的退出）
        self.system_info_cache.stop();

        serve_result.map_err(|e| CcrError::ConfigError(format!("服务器运行错误: {}", e)))?;

        Ok(())
    }

    /// 📦 提供 HTML 页面（从原来的 handlers.rs 移过来）
    pub async fn serve_html() -> Html<&'static str> {
        Html(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../ccr-ui/web/dist/index.html"
        )))
    }

    /// 📦 提供 CSS 样式文件（从原来的 handlers.rs 移过来）
    pub async fn serve_css() -> impl IntoResponse {
        (
            [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../ccr-ui/web/dist/style.css"
            )),
        )
    }

    /// 📦 提供 JavaScript 脚本文件（从原来的 handlers.rs 移过来）
    pub async fn serve_js() -> impl IntoResponse {
        (
            [(
                axum::http::header::CONTENT_TYPE,
                "application/javascript; charset=utf-8",
            )],
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../ccr-ui/web/dist/script.js"
            )),
        )
    }

    /// 🎯 尝试绑定可用端口
    ///
    /// 从指定端口开始，如果被占用则尝试后续 10 个端口
    async fn bind_available_port(
        host: std::net::IpAddr,
        start_port: u16,
    ) -> Result<(tokio::net::TcpListener, u16, std::net::IpAddr)> {
        let max_attempts = 10;
        let mut last_error_message: Option<String> = None;
        let host_is_any = match host {
            std::net::IpAddr::V4(ip) => ip.is_unspecified(),
            std::net::IpAddr::V6(ip) => ip.is_unspecified(),
        };

        for offset in 0..max_attempts {
            let port = start_port + offset;
            let addr = format!("{}:{}", host, port);
            match tokio::net::TcpListener::bind(&addr).await {
                Ok(listener) => {
                    if offset > 0 {
                        ColorOutput::warning(&format!(
                            "⚠️ 端口 {} 被占用，已切换到端口 {}",
                            start_port, port
                        ));
                    }
                    return Ok((listener, port, host));
                }
                Err(e) => {
                    last_error_message = Some(e.to_string());
                    if e.kind() == std::io::ErrorKind::PermissionDenied && host_is_any {
                        let addr_local = format!("127.0.0.1:{}", port);
                        match tokio::net::TcpListener::bind(&addr_local).await {
                            Ok(listener) => {
                                ColorOutput::warning(&format!(
                                    "⚠️ 绑定到 {} 被拒绝（权限/策略），已改为绑定到 localhost:{}",
                                    addr, port
                                ));
                                return Ok((
                                    listener,
                                    port,
                                    std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                                ));
                            }
                            Err(local_err) => {
                                last_error_message = Some(local_err.to_string());
                                continue;
                            }
                        }
                    } else {
                        continue;
                    }
                }
            };
        }

        let last_error_message = last_error_message.unwrap_or_else(|| "unknown".to_string());
        let listener = if host_is_any {
            match tokio::net::TcpListener::bind("127.0.0.1:0").await {
                Ok(listener) => listener,
                Err(e_local) => {
                    let local_message = e_local.to_string();
                    tokio::net::TcpListener::bind("0.0.0.0:0").await.map_err(|e_any| {
                        CcrError::ConfigError(format!(
                            "无法绑定任何端口（{}-{}），且自动分配端口也失败（localhost 最后错误: {}）: {}",
                            start_port,
                            start_port + max_attempts - 1,
                            local_message,
                            e_any
                        ))
                    })?
                }
            }
        } else {
            tokio::net::TcpListener::bind(format!("{}:0", host))
                .await
                .map_err(|e_host| {
                    CcrError::ConfigError(format!(
                        "无法绑定任何端口（{}-{}，host={}），且自动分配端口也失败: {}",
                        start_port,
                        start_port + max_attempts - 1,
                        host,
                        e_host
                    ))
                })?
        };

        let actual_port = listener.local_addr().map(|addr| addr.port()).unwrap_or(0);

        ColorOutput::warning(&format!(
            "⚠️ 端口 {}-{} 均不可用（最后错误: {}），已自动分配端口 {}",
            start_port,
            start_port + max_attempts - 1,
            last_error_message,
            actual_port
        ));

        let bound_ip = if host_is_any {
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
        } else {
            host
        };

        Ok((listener, actual_port, bound_ip))
    }

    /// 🎯 获取平台模式（使用缓存）
    #[allow(dead_code)]
    pub fn get_platform_mode() -> (bool, Option<std::path::PathBuf>) {
        PLATFORM_MODE
            .read()
            .unwrap_or_else(|poisoned| {
                eprintln!("⚠️  PLATFORM_MODE RwLock 被毒化，尝试恢复");
                poisoned.into_inner()
            })
            .clone()
    }

    /// 🎯 刷新平台模式缓存
    pub fn refresh_platform_mode() {
        let mut cache = PLATFORM_MODE.write().unwrap_or_else(|poisoned| {
            eprintln!("⚠️  PLATFORM_MODE RwLock 被毒化，尝试恢复");
            poisoned.into_inner()
        });
        *cache = ConfigManager::detect_unified_mode();
    }

    /// 🔍 检测是否在 WSL 环境中运行
    ///
    /// 通过读取 /proc/version 文件检测是否包含 "microsoft" 或 "wsl" 关键字
    fn detect_wsl_environment() -> bool {
        if let Ok(content) = std::fs::read_to_string("/proc/version") {
            let content_lower = content.to_lowercase();
            return content_lower.contains("microsoft") || content_lower.contains("wsl");
        }
        false
    }

    /// 🌐 获取本地网络 IP 地址
    ///
    /// 通过连接外部地址（不实际发送数据）获取本机的网络接口 IP
    /// 这样可以让系统自动选择合适的网络接口
    fn get_local_ip() -> Option<String> {
        use std::net::UdpSocket;

        // 尝试绑定并连接到外部地址（不会实际发送数据）
        // 这样可以让系统选择合适的网络接口
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            // 连接到 Google DNS (8.8.8.8) - 不会实际发送数据
            if socket.connect("8.8.8.8:80").is_ok()
                && let Ok(addr) = socket.local_addr()
            {
                return Some(addr.ip().to_string());
            }
        }
        None
    }
}

/// Web 命令入口
pub async fn web_command(
    host: Option<std::net::IpAddr>,
    port: Option<u16>,
    no_browser: bool,
) -> Result<()> {
    let host = host.unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    let port = port.unwrap_or(19527);
    let server = WebServer::new(host, port)?;
    server.start(no_browser).await
}
