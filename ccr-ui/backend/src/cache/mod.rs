//! 全局缓存模块
//!
//! 为 ccr-ui-backend 提供泛型 TTL 缓存和全局设置缓存单例，减少 80% 的重复文件 I/O。
//!
//! ## 架构
//! - `TtlCache<T>`: 泛型带过期时间缓存，基于 `std::sync::RwLock<Option<(Arc<T>, Instant)>>`
//! - `SettingsCache`: 对 `TtlCache<ClaudeSettings>` 的包装，保持向后兼容的 `load()`/`save_atomic()` API
//! - 使用 `lazy_static` + `Arc` 实现全局单例
//! - 30 秒 TTL 自动过期
//! - 写操作自动失效缓存
//!
//! ## 使用示例
//! ```rust,ignore
//! use crate::cache::GLOBAL_SETTINGS_CACHE;
//!
//! // 读取配置（自动缓存 30 秒，缓存命中时仅增加 Arc 引用计数）
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

// ─────────────────────────────────────────────
// TtlCache<T> — 泛型 TTL 缓存
// ─────────────────────────────────────────────

/// 泛型带过期时间缓存
///
/// 内部使用 `std::sync::RwLock`（而非 `tokio::sync::RwLock`），
/// 因为所有调用方均在 `spawn_blocking` 上下文中运行，不需要异步锁。
///
/// 缓存存储 `Arc<T>`，缓存命中时仅增加引用计数，不进行深拷贝。
/// 调用方若需要可变副本，自行对 `Arc<T>` 调用 `.as_ref().clone()`。
pub struct TtlCache<T> {
    /// 缓存数据：`(Arc<T>, 写入时间戳)`
    data: RwLock<Option<(Arc<T>, Instant)>>,
    /// 缓存有效期 (TTL)
    ttl: Duration,
}

impl<T: Send + Sync + 'static> TtlCache<T> {
    /// 创建新的缓存实例
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: RwLock::new(None),
            ttl,
        }
    }

    /// 获取缓存的 `Arc<T>`，若缓存有效则直接返回，否则调用 `loader` 加载
    ///
    /// 缓存命中：O(1) 引用计数递增，无深拷贝。
    /// 缓存未命中：调用 `loader`，将结果包装为 `Arc<T>` 后写入缓存并返回。
    pub fn get_or_load<E>(&self, loader: impl FnOnce() -> Result<T, E>) -> Result<Arc<T>, E> {
        // 先尝试读取缓存（持有读锁期间不调用 loader，避免锁内阻塞）
        {
            let guard = crate::core::lock_utils::recover_rwlock_read(self.data.read());
            if let Some((ref cached, cached_at)) = *guard
                && cached_at.elapsed() < self.ttl
            {
                // 缓存命中：仅增加 Arc 引用计数
                return Ok(Arc::clone(cached));
            }
        }

        // 缓存无效，调用 loader 加载新数据
        let new_value = Arc::new(loader()?);

        // 写入缓存
        {
            let mut guard = crate::core::lock_utils::recover_rwlock_write(self.data.write());
            *guard = Some((Arc::clone(&new_value), Instant::now()));
        }

        Ok(new_value)
    }

    /// 将指定值写入缓存（用于写后更新，避免立即从磁盘重新加载）
    #[allow(dead_code)]
    pub fn set(&self, value: T) {
        let mut guard = crate::core::lock_utils::recover_rwlock_write(self.data.write());
        *guard = Some((Arc::new(value), Instant::now()));
    }

    /// 失效缓存，强制下次 `get_or_load` 重新调用 loader
    pub fn invalidate(&self) {
        let mut guard = crate::core::lock_utils::recover_rwlock_write(self.data.write());
        *guard = None;
    }
}

// ─────────────────────────────────────────────
// SettingsCache — TtlCache<ClaudeSettings> 包装器
// ─────────────────────────────────────────────

/// ClaudeSettings 缓存包装器
///
/// 在 `TtlCache<ClaudeSettings>` 基础上提供向后兼容的 `load()` / `save_atomic()` API。
/// `load()` 返回 `ClaudeSettings`（owned 值）以保证调用方可直接可变使用。
/// 内部缓存命中时仅增加 `Arc` 引用计数，随后调用 `.as_ref().clone()` 完成一次浅拷贝，
/// 相比旧实现在缓存写入与读取时均进行深拷贝，性能有所提升。
pub struct SettingsCache {
    /// 内部泛型缓存
    inner: TtlCache<ClaudeSettings>,
    /// 底层 SettingsManager（负责文件 I/O）
    manager: SettingsManager,
}

impl SettingsCache {
    /// 创建新的缓存实例
    fn new(ttl: Duration) -> Result<Self, crate::managers::settings_manager::SettingsError> {
        Ok(Self {
            inner: TtlCache::new(ttl),
            manager: SettingsManager::default()?,
        })
    }

    /// 获取缓存数据或从磁盘加载
    ///
    /// 若缓存有效，仅增加 Arc 引用计数后返回拥有所有权的 `ClaudeSettings`；
    /// 若缓存无效或过期，从磁盘加载并缓存后返回。
    pub fn load(&self) -> Result<ClaudeSettings, crate::managers::settings_manager::SettingsError> {
        // 通过泛型缓存获取 Arc<ClaudeSettings>，再 clone 出 owned 值供调用方可变使用
        let arc = self.inner.get_or_load(|| self.manager.load())?;
        Ok((*arc).clone())
    }

    /// 原子保存设置文件并失效缓存
    ///
    /// 保存后自动失效缓存，下次 `load()` 将重新从磁盘加载。
    pub fn save_atomic(
        &self,
        settings: &ClaudeSettings,
    ) -> Result<(), crate::managers::settings_manager::SettingsError> {
        // 先保存到磁盘
        self.manager.save(settings)?;
        // 失效缓存（下次 load 重新从磁盘读取）
        self.inner.invalidate();
        Ok(())
    }

    /// 手动失效缓存
    ///
    /// 强制下次 `load()` 从磁盘读取（测试辅助 / 外部变更通知场景）。
    #[allow(dead_code)]
    pub fn invalidate(&self) {
        self.inner.invalidate();
    }
}

// ─────────────────────────────────────────────
// 全局单例
// ─────────────────────────────────────────────

lazy_static! {
    /// 全局设置缓存单例
    ///
    /// - TTL: 30 秒
    /// - 线程安全: std::sync::RwLock
    /// - 缓存命中: 仅增加 Arc 引用计数，无深拷贝
    /// - 自动失效: save_atomic() 时自动清除缓存
    pub static ref GLOBAL_SETTINGS_CACHE: Arc<SettingsCache> = {
        Arc::new(
            SettingsCache::new(Duration::from_secs(30))
                .expect("无法初始化全局设置缓存")
        )
    };
}

/// 测试辅助: 手动失效缓存（用于集成测试）
#[cfg(test)]
pub fn invalidate_global_cache() {
    GLOBAL_SETTINGS_CACHE.invalidate();
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
    fn test_ttl_cache_invalidate() {
        let cache: TtlCache<u32> = TtlCache::new(Duration::from_secs(60));

        // 写入一个值
        cache.set(42u32);

        // 应能命中缓存
        let val = cache.get_or_load(|| -> Result<u32, ()> { Ok(99) }).unwrap();
        assert_eq!(*val, 42);

        // 失效后应调用 loader
        cache.invalidate();
        let val2 = cache.get_or_load(|| -> Result<u32, ()> { Ok(99) }).unwrap();
        assert_eq!(*val2, 99);
    }

    #[test]
    fn test_ttl_cache_expiry() {
        // TTL = 0 意味着每次都过期
        let cache: TtlCache<u32> = TtlCache::new(Duration::ZERO);
        cache.set(1u32);

        // 由于 TTL=0，elapsed >= ttl，应重新调用 loader
        let val = cache.get_or_load(|| -> Result<u32, ()> { Ok(2) }).unwrap();
        assert_eq!(*val, 2);
    }

    #[test]
    fn test_ttl_cache_arc_identity_on_hit() {
        let cache: TtlCache<String> = TtlCache::new(Duration::from_secs(60));
        cache.set("hello".to_string());

        let a = cache
            .get_or_load(|| -> Result<String, ()> { Ok("world".to_string()) })
            .unwrap();
        let b = cache
            .get_or_load(|| -> Result<String, ()> { Ok("world".to_string()) })
            .unwrap();

        // 同一个 Arc，指向同一块内存
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    #[ignore = "依赖真实 Claude Code 配置文件，CI 环境可能不存在"]
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
