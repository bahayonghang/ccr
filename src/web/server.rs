// 🌐 Web 服务器核心
// 管理 HTTP 服务器的生命周期

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::services::{BackupService, ConfigService, HistoryService, SettingsService};
use crate::web::handlers::Handlers;
use crate::web::system_info_cache::SystemInfoCache;
use std::sync::Arc;
use std::time::Duration;
use tiny_http::Server;

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

    /// 🚀 启动服务器
    pub fn start(&self) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let server = Server::http(&addr)
            .map_err(|e| CcrError::ConfigError(format!("无法启动 HTTP 服务器: {}", e)))?;

        ColorOutput::success(&format!("🌐 CCR Web 服务器已启动"));
        ColorOutput::info(&format!("📍 地址: http://localhost:{}", self.port));
        ColorOutput::info("⏹️ 按 Ctrl+C 停止服务器");
        println!();

        // 🌐 尝试自动打开浏览器
        if let Err(e) = open::that(format!("http://localhost:{}", self.port)) {
            ColorOutput::warning(&format!("⚠️ 无法自动打开浏览器: {}", e));
            ColorOutput::info(&format!("💡 请手动访问 http://localhost:{}", self.port));
        }

        // 创建处理器
        let handlers = Handlers::new(
            self.config_service.clone(),
            self.settings_service.clone(),
            self.history_service.clone(),
            self.backup_service.clone(),
            self.system_info_cache.clone(),
        );

        // 🔄 处理请求循环
        for request in server.incoming_requests() {
            if let Err(e) = handlers.handle_request(request) {
                log::error!("❌ 处理请求失败: {}", e);
            }
        }

        Ok(())
    }
}

/// Web 命令入口
pub fn web_command(port: Option<u16>) -> Result<()> {
    let port = port.unwrap_or(8080);
    let server = WebServer::new(port)?;
    server.start()
}
