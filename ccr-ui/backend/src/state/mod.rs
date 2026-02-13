//! 统一应用状态模块
//!
//! 提供全局共享的应用状态，包括数据库连接池、HTTP 客户端、WebSocket 状态等。
//! 通过 Axum 的 State 提取器注入到 Handler 中，避免每次请求重复创建资源。
//!
//! ## 设计原则
//! - **单一实例**: 所有共享资源在应用启动时初始化一次
//! - **线程安全**: 使用 Arc 包装，支持多线程并发访问
//! - **依赖注入**: 通过 Axum State 注入，便于测试和模块化

use std::sync::Arc;
use std::time::Duration;

use crate::cache::{GLOBAL_SETTINGS_CACHE, SettingsCache};
use crate::database::{self, DbPool};
use crate::services::checkin_service::CheckinService;
use crate::services::websocket::WsState;

/// AppState 初始化错误
#[derive(Debug, thiserror::Error)]
pub enum AppStateError {
    #[error("Failed to create database pool: {0}")]
    Database(#[from] database::DatabaseError),

    #[error("Failed to create HTTP client: {0}")]
    HttpClient(#[from] reqwest::Error),
}

/// 统一的应用状态
///
/// 包含所有需要在 Handler 之间共享的资源。
/// 通过 `Clone` 实现零成本复制（内部都是 Arc）。
///
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接池（预留，后续 handler 迁移使用）
    #[allow(dead_code)]
    pub db: DbPool,
    /// 共享的 HTTP 客户端（用于签到、API 调用等）
    pub http_client: reqwest::Client,
    /// WebSocket 状态（广播通道、日志缓存）
    pub ws: Arc<WsState>,
    /// 设置缓存（30s TTL，预留）
    #[allow(dead_code)]
    pub settings_cache: Arc<SettingsCache>,
    /// 签到数据目录（启动时初始化一次）
    pub checkin_dir: std::path::PathBuf,
}

impl AppState {
    /// 创建新的应用状态
    ///
    /// # Arguments
    /// * `db` - 数据库连接池
    /// * `http_client` - 共享的 HTTP 客户端
    /// * `ws` - WebSocket 状态
    /// * `settings_cache` - 设置缓存
    pub fn new(
        db: DbPool,
        http_client: reqwest::Client,
        ws: Arc<WsState>,
        settings_cache: Arc<SettingsCache>,
        checkin_dir: std::path::PathBuf,
    ) -> Self {
        Self {
            db,
            http_client,
            ws,
            settings_cache,
            checkin_dir,
        }
    }

    /// 创建签到服务（复用共享 HTTP 客户端）
    pub fn checkin_service(&self) -> CheckinService {
        CheckinService::with_client(self.checkin_dir.clone(), self.http_client.clone())
    }

    /// 使用默认配置初始化应用状态
    ///
    /// 创建所有必要的共享资源：
    /// - 数据库连接池
    /// - HTTP 客户端（带超时和连接池配置）
    /// - WebSocket 状态
    /// - 设置缓存（使用全局单例）
    pub fn initialize() -> Result<Self, AppStateError> {
        // 创建数据库连接池
        let db = database::create_app_pool()?;

        // 创建共享的 HTTP 客户端
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .cookie_store(true)
            .user_agent("ccr-ui/1.0")
            .build()?;

        // 创建 WebSocket 状态
        let ws = Arc::new(WsState::new());

        // 使用全局设置缓存单例
        let settings_cache = GLOBAL_SETTINGS_CACHE.clone();

        // 计算签到数据目录
        let checkin_dir = CheckinService::default_checkin_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from(".ccr/checkin"));

        Ok(Self {
            db,
            http_client,
            ws,
            settings_cache,
            checkin_dir,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_clone() {
        // AppState 应该是 Clone 的（零成本复制）
        fn assert_clone<T: Clone>() {}
        assert_clone::<AppState>();
    }
}
