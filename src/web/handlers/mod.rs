// 🔌 Web 请求处理器模块
// 按功能拆分的处理器集合

pub mod codex_handlers;
pub mod config_handlers;
pub mod platform_handlers;
pub mod sync_handlers;
pub mod system_handlers;

// 重新导出常用类型
// pub use super::models::*;  // 暂时未使用

use crate::managers::config::CcsConfig;
/// 🔌 共享状态结构
/// 持有所有 Service 的引用和配置缓存
use crate::services::{BackupService, ConfigService, HistoryService, SettingsService};
use crate::web::system_info_cache::SystemInfoCache;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub config_service: Arc<ConfigService>,
    pub settings_service: Arc<SettingsService>,
    pub history_service: Arc<HistoryService>,
    pub backup_service: Arc<BackupService>,
    pub system_info_cache: Arc<SystemInfoCache>,
    pub config_cache: Arc<RwLock<CcsConfig>>,
}

impl AppState {
    pub fn new(
        config_service: Arc<ConfigService>,
        settings_service: Arc<SettingsService>,
        history_service: Arc<HistoryService>,
        backup_service: Arc<BackupService>,
        system_info_cache: Arc<SystemInfoCache>,
        initial_config: CcsConfig,
    ) -> Self {
        Self {
            config_service,
            settings_service,
            history_service,
            backup_service,
            system_info_cache,
            config_cache: Arc::new(RwLock::new(initial_config)),
        }
    }

    pub fn reload_config_cache(&self) -> Result<(), crate::core::error::CcrError> {
        let config_manager = crate::managers::ConfigManager::with_default()?;
        let new_config = config_manager.load()?;

        let mut cache = self.config_cache.write().map_err(|e| {
            crate::core::error::CcrError::ConfigError(format!("获取配置缓存写锁失败: {}", e))
        })?;
        *cache = new_config;

        Ok(())
    }
}
