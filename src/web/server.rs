// 🌐 Web 服务器核心
// 管理 HTTP 服务器的生命周期
// 🎯 异步架构 - 使用 Axum 提供高性能并发处理

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::ConfigManager;
use crate::services::{BackupService, ConfigService, HistoryService, SettingsService};
use crate::web::handlers::AppState;
use crate::web::system_info_cache::SystemInfoCache;
use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::get,
};
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tower_http::cors::CorsLayer;

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
    system_info_cache: Arc<SystemInfoCache>,
    port: u16,
}

impl WebServer {
    /// 🏗️ 创建新的 Web 服务器
    pub fn new(port: u16) -> Result<Self> {
        let config_service = Arc::new(ConfigService::with_default()?);
        let settings_service = Arc::new(SettingsService::with_default()?);
        let history_service = Arc::new(HistoryService::with_default()?);
        let backup_service = Arc::new(BackupService::with_default()?);

        // 🎯 创建系统信息缓存，每 2 秒更新一次
        let system_info_cache = Arc::new(SystemInfoCache::new(Duration::from_secs(2)));

        Ok(Self {
            config_service,
            settings_service,
            history_service,
            backup_service,
            system_info_cache,
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
        let (listener, actual_port) = Self::bind_available_port(self.port).await?;

        ColorOutput::success("🌐 CCR Web 服务器已启动（异步模式）");
        ColorOutput::info(&format!("📍 地址: http://localhost:{}", actual_port));
        ColorOutput::info("⏹️ 按 Ctrl+C 停止服务器");
        println!();

        // 🌐 根据参数决定是否打开浏览器
        if !no_browser {
            if let Err(e) = open::that(format!("http://localhost:{}", actual_port)) {
                ColorOutput::warning(&format!("⚠️ 无法自动打开浏览器: {}", e));
                ColorOutput::info(&format!("💡 请手动访问 http://localhost:{}", actual_port));
            }
        } else {
            ColorOutput::info(&format!("💡 请手动访问 http://localhost:{}", actual_port));
        }

        // 🎯 加载初始配置到缓存
        let config_manager = ConfigManager::with_default()?;
        let initial_config = config_manager.load()?;

        // 创建共享状态
        let state = AppState::new(
            self.config_service.clone(),
            self.settings_service.clone(),
            self.history_service.clone(),
            self.backup_service.clone(),
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
                post "/api/config/{name}"               => crate::web::handlers::config_handlers::handle_update_config,
                delete "/api/config/{name}"             => crate::web::handlers::config_handlers::handle_delete_config,
                post "/api/export"                      => crate::web::handlers::config_handlers::handle_export,
                post "/api/import"                      => crate::web::handlers::config_handlers::handle_import,

                // 系统和设置
                get  "/api/history"                     => crate::web::handlers::system_handlers::handle_get_history,
                post "/api/validate"                    => crate::web::handlers::system_handlers::handle_validate,
                post "/api/clean"                       => crate::web::handlers::system_handlers::handle_clean,
                get  "/api/settings"                    => crate::web::handlers::system_handlers::handle_get_settings,
                get  "/api/settings/backups"            => crate::web::handlers::system_handlers::handle_get_settings_backups,
                post "/api/settings/restore"            => crate::web::handlers::system_handlers::handle_restore_settings,
                get  "/api/system"                      => crate::web::handlers::system_handlers::handle_get_system_info,
                post "/api/reload"                      => crate::web::handlers::system_handlers::handle_reload_config,

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
        let app = app.layer(CorsLayer::permissive()); // CORS 支持

        // 🚀 启动服务器
        axum::serve(listener, app)
            .await
            .map_err(|e| CcrError::ConfigError(format!("服务器运行错误: {}", e)))?;

        Ok(())
    }

    /// 📦 提供 HTML 页面（从原来的 handlers.rs 移过来）
    pub async fn serve_html() -> Html<&'static str> {
        Html(include_str!("../../web/index.html"))
    }

    /// 📦 提供 CSS 样式文件（从原来的 handlers.rs 移过来）
    pub async fn serve_css() -> impl IntoResponse {
        (
            [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
            include_str!("../../web/style.css"),
        )
    }

    /// 📦 提供 JavaScript 脚本文件（从原来的 handlers.rs 移过来）
    pub async fn serve_js() -> impl IntoResponse {
        (
            [(
                axum::http::header::CONTENT_TYPE,
                "application/javascript; charset=utf-8",
            )],
            include_str!("../../web/script.js"),
        )
    }

    /// 🎯 尝试绑定可用端口
    ///
    /// 从指定端口开始，如果被占用则尝试后续 10 个端口
    async fn bind_available_port(start_port: u16) -> Result<(tokio::net::TcpListener, u16)> {
        let max_attempts = 10;

        for offset in 0..max_attempts {
            let port = start_port + offset;
            let addr = format!("0.0.0.0:{}", port);

            match tokio::net::TcpListener::bind(&addr).await {
                Ok(listener) => {
                    if offset > 0 {
                        ColorOutput::warning(&format!(
                            "⚠️ 端口 {} 被占用，已切换到端口 {}",
                            start_port, port
                        ));
                    }
                    return Ok((listener, port));
                }
                Err(_) if offset < max_attempts - 1 => continue,
                Err(e) => {
                    return Err(CcrError::ConfigError(format!(
                        "无法绑定端口 {}-{}: {}",
                        start_port,
                        start_port + max_attempts - 1,
                        e
                    )));
                }
            }
        }

        unreachable!()
    }

    /// 🎯 获取平台模式（使用缓存）
    #[allow(dead_code)]
    pub fn get_platform_mode() -> (bool, Option<std::path::PathBuf>) {
        PLATFORM_MODE.read().unwrap().clone()
    }

    /// 🎯 刷新平台模式缓存
    #[allow(dead_code)]
    pub fn refresh_platform_mode() {
        let mut cache = PLATFORM_MODE.write().unwrap();
        *cache = ConfigManager::detect_unified_mode();
    }
}

/// Web 命令入口
pub fn web_command(port: Option<u16>, no_browser: bool) -> Result<()> {
    let port = port.unwrap_or(8080);
    let server = WebServer::new(port)?;

    // 🎯 创建 Tokio 运行时并执行异步服务器
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| CcrError::ConfigError(format!("创建 Tokio 运行时失败: {}", e)))?;

    runtime.block_on(server.start(no_browser))
}
