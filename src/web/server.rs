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
        let config_manager = crate::managers::ConfigManager::with_default()?;
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
            .route("/api/reload", post(handlers::handle_reload_config))
            // 🆕 API 路由 - 平台管理 (Unified Mode)
            .route("/api/platforms", get(handlers::handle_get_platform_info))
            .route(
                "/api/platforms/switch",
                post(handlers::handle_switch_platform),
            )
            // ☁️ API 路由 - 同步相关
            .route("/api/sync/status", get(handlers::handle_sync_status))
            .route("/api/sync/config", post(handlers::handle_sync_config))
            .route("/api/sync/push", post(handlers::handle_sync_push))
            .route("/api/sync/pull", post(handlers::handle_sync_pull))
            // 🎯 添加 CORS 支持
            .layer(CorsLayer::permissive())
            // 🎯 注入共享状态
            .with_state(state);

        // 🚀 启动服务器
        axum::serve(listener, app)
            .await
            .map_err(|e| CcrError::ConfigError(format!("服务器运行错误: {}", e)))?;

        Ok(())
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
                    )))
                }
            }
        }
        
        unreachable!()
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
