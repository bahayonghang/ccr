// 🌐 Web 服务器核心
// 管理 HTTP 服务器的生命周期
// 🎯 异步架构 - 使用 Axum 提供高性能并发处理

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::services::{BackupService, ConfigService, HistoryService, SettingsService};
use crate::web::handlers::{self, AppState};
use crate::web::system_info_cache::SystemInfoCache;
use axum::{
    Router,
    routing::{delete, get, post},
};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;

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
        let config_service = Arc::new(ConfigService::default()?);
        let settings_service = Arc::new(SettingsService::default()?);
        let history_service = Arc::new(HistoryService::default()?);
        let backup_service = Arc::new(BackupService::default()?);

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
    pub async fn start(&self) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);

        ColorOutput::success("🌐 CCR Web 服务器已启动（异步模式）");
        ColorOutput::info(&format!("📍 地址: http://localhost:{}", self.port));
        ColorOutput::info("⏹️ 按 Ctrl+C 停止服务器");
        println!();

        // 🌐 尝试自动打开浏览器
        if let Err(e) = open::that(format!("http://localhost:{}", self.port)) {
            ColorOutput::warning(&format!("⚠️ 无法自动打开浏览器: {}", e));
            ColorOutput::info(&format!("💡 请手动访问 http://localhost:{}", self.port));
        }

        // 创建共享状态
        let state = AppState::new(
            self.config_service.clone(),
            self.settings_service.clone(),
            self.history_service.clone(),
            self.backup_service.clone(),
            self.system_info_cache.clone(),
        );

        // 🎯 构建路由
        let app = Router::new()
            // 静态文件
            .route("/", get(handlers::serve_html))
            .route("/style.css", get(handlers::serve_css))
            .route("/script.js", get(handlers::serve_js))
            // API 路由 - 配置管理
            .route("/api/configs", get(handlers::handle_list_configs))
            .route("/api/switch", post(handlers::handle_switch_config))
            .route("/api/config", post(handlers::handle_add_config))
            .route("/api/config/{name}", post(handlers::handle_update_config))
            .route("/api/config/{name}", delete(handlers::handle_delete_config))
            .route("/api/history", get(handlers::handle_get_history))
            .route("/api/validate", post(handlers::handle_validate))
            .route("/api/clean", post(handlers::handle_clean))
            .route("/api/settings", get(handlers::handle_get_settings))
            .route(
                "/api/settings/backups",
                get(handlers::handle_get_settings_backups),
            )
            .route(
                "/api/settings/restore",
                post(handlers::handle_restore_settings),
            )
            .route("/api/export", post(handlers::handle_export))
            .route("/api/import", post(handlers::handle_import))
            .route("/api/system", get(handlers::handle_get_system_info))
            // 🆕 API 路由 - 平台管理 (Unified Mode)
            .route("/api/platforms", get(handlers::handle_get_platform_info))
            .route("/api/platforms/switch", post(handlers::handle_switch_platform))
            // 🎯 添加 CORS 支持
            .layer(CorsLayer::permissive())
            // 🎯 注入共享状态
            .with_state(state);

        // 🚀 启动服务器
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| CcrError::ConfigError(format!("无法绑定端口 {}: {}", self.port, e)))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| CcrError::ConfigError(format!("服务器运行错误: {}", e)))?;

        Ok(())
    }
}

/// Web 命令入口
pub fn web_command(port: Option<u16>) -> Result<()> {
    let port = port.unwrap_or(8080);
    let server = WebServer::new(port)?;

    // 🎯 创建 Tokio 运行时并执行异步服务器
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| CcrError::ConfigError(format!("创建 Tokio 运行时失败: {}", e)))?;

    runtime.block_on(server.start())
}
