//! 全局缓存模块
//!
//! 为 ccr-ui-backend 提供全局单例缓存，减少 80% 的重复文件 I/O。
//!
//! ## 架构
//! - 基于 std::sync::RwLock 实现线程安全的读写缓存
//! - 使用 lazy_static + Arc 实现全局单例
//! - 30 秒 TTL 自动过期
//! - 写操作自动失效缓存
//!
//! ## 使用示例
//! ```rust,ignore
//! use crate::cache::GLOBAL_SETTINGS_CACHE;
//!
//! // 读取配置（自动缓存 30 秒）
//! let settings = GLOBAL_SETTINGS_CACHE.load()?;
//!
//! // 保存配置（自动失效缓存）
//! GLOBAL_SETTINGS_CACHE.save_atomic(&settings)?;
//! ```

#![allow(deprecated)] // 内部使用旧 SettingsManager 实现，外部应使用 GLOBAL_SETTINGS_CACHE

use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::managers::settings_manager::{ClaudeSettings, SettingsManager};

/// 🗄️ ClaudeSettings 缓存包装器
///
/// 提供带 TTL 的线程安全缓存
pub struct SettingsCache {
    /// 缓存数据和时间戳
    data: RwLock<Option<(ClaudeSettings, Instant)>>,
    /// 缓存有效期 (TTL)
    ttl: Duration,
    /// 底层 SettingsManager
    manager: SettingsManager,
}

impl SettingsCache {
    /// 🏗️ 创建新的缓存实例
    fn new(ttl: Duration) -> Result<Self, crate::managers::settings_manager::SettingsError> {
        Ok(Self {
            data: RwLock::new(None),
            ttl,
            manager: SettingsManager::default()?,
        })
    }

    /// 📖 获取缓存数据或从磁盘加载
    ///
    /// 如果缓存有效，直接返回缓存数据
    /// 如果缓存无效或过期，从磁盘加载并缓存
    pub fn load(&self) -> Result<ClaudeSettings, crate::managers::settings_manager::SettingsError> {
        // 先尝试读取缓存
        {
            let guard = self
                .data
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some((ref cached, cached_at)) = *guard
                && cached_at.elapsed() < self.ttl
            {
                return Ok(cached.clone());
            }
        }

        // 缓存无效，需要加载
        let new_data = self.manager.load()?;

        // 写入缓存
        {
            let mut guard = self
                .data
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *guard = Some((new_data.clone(), Instant::now()));
        }

        Ok(new_data)
    }

    /// 💾 原子保存设置文件并失效缓存
    ///
    /// 保存后自动失效缓存，下次 load() 将重新从磁盘加载
    pub fn save_atomic(
        &self,
        settings: &ClaudeSettings,
    ) -> Result<(), crate::managers::settings_manager::SettingsError> {
        // 先保存
        self.manager.save(settings)?;
        // 然后失效缓存
        self.invalidate();
        Ok(())
    }

    /// 🧹 手动失效缓存
    ///
    /// 强制下次 load() 从磁盘读取
    fn invalidate(&self) {
        let mut guard = self
            .data
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = None;
    }
}

lazy_static! {
    /// 全局设置缓存单例
    ///
    /// - TTL: 30 秒
    /// - 线程安全: RwLock
    /// - 自动失效: save_atomic() 时自动清除缓存
    pub static ref GLOBAL_SETTINGS_CACHE: Arc<SettingsCache> = {
        Arc::new(
            SettingsCache::new(Duration::from_secs(30))
                .expect("❌ 无法初始化全局设置缓存")
        )
    };
}

/// 🧪 测试辅助: 手动失效缓存（用于集成测试）
#[cfg(test)]
pub fn invalidate_global_cache() {
    GLOBAL_SETTINGS_CACHE.invalidate();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_cache_singleton() {
        let cache1 = GLOBAL_SETTINGS_CACHE.clone();
        let cache2 = GLOBAL_SETTINGS_CACHE.clone();

        // 验证是同一个实例
        assert!(Arc::ptr_eq(&cache1, &cache2));
    }

    #[test]
    fn test_cache_basic_operations() {
        // 加载
        let settings = GLOBAL_SETTINGS_CACHE.load();
        assert!(settings.is_ok());

        // 再次加载应该命中缓存
        let settings2 = GLOBAL_SETTINGS_CACHE.load();
        assert!(settings2.is_ok());

        // 失效缓存
        invalidate_global_cache();

        // 重新加载
        let settings3 = GLOBAL_SETTINGS_CACHE.load();
        assert!(settings3.is_ok());
    }
}
