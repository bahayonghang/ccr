// 🗄️ CCR 配置缓存模块
// 📦 提供通用的配置缓存机制，减少重复的文件 I/O
//
// 核心职责:
// - 🔄 缓存配置数据并支持 TTL 过期
// - 🧹 支持手动失效缓存
// - 🔒 线程安全的读写操作
//
// 设计目标: 减少 80% 的重复文件读取

use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// 🗄️ 通用配置缓存
///
/// 线程安全的配置缓存，支持 TTL 过期和手动失效
///
/// ## 使用示例
/// ```rust,ignore
/// use ccr_core::core::cache::ConfigCache;
///
/// let cache: ConfigCache<MyConfig> = ConfigCache::new(Duration::from_secs(30));
///
/// // 获取或加载
/// let config = cache.get_or_load(|| load_from_file())?;
///
/// // 手动失效
/// cache.invalidate();
/// ```
#[allow(dead_code)]
pub struct ConfigCache<T> {
    /// 缓存数据和时间戳
    data: RwLock<Option<(Arc<T>, Instant)>>,
    /// 缓存有效期 (TTL)
    ttl: Duration,
    /// 单航班加载锁，避免并发 miss 时重复执行昂贵 loader
    load_lock: Mutex<()>,
}

#[allow(dead_code)]
impl<T> ConfigCache<T> {
    /// 🏗️ 创建新的缓存实例
    ///
    /// # 参数
    /// - `ttl`: 缓存有效期
    #[allow(dead_code)]
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: RwLock::new(None),
            ttl,
            load_lock: Mutex::new(()),
        }
    }

    /// 🏠 使用默认 TTL(30秒) 创建缓存
    pub fn with_default_ttl() -> Self {
        Self::new(Duration::from_secs(30))
    }

    /// 🔍 尝试获取共享缓存数据（不触发加载）
    ///
    /// 如果缓存有效，返回 `Arc<T>`，避免命中缓存时发生深拷贝。
    pub fn get_shared(&self) -> Option<Arc<T>> {
        let guard = self
            .data
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some((cached, cached_at)) = guard.as_ref()
            && cached_at.elapsed() < self.ttl
        {
            return Some(Arc::clone(cached));
        }
        None
    }

    /// 📖 获取共享缓存数据或通过加载器加载
    ///
    /// 在并发 miss 场景下，只允许一个线程执行 loader，其余线程复用同一批新鲜数据。
    pub fn get_or_load_shared<F, E>(&self, loader: F) -> Result<Arc<T>, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if let Some(cached) = self.get_shared() {
            return Ok(cached);
        }

        let _load_guard = self
            .load_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(cached) = self.get_shared() {
            return Ok(cached);
        }

        let new_data = Arc::new(loader()?);

        {
            let mut guard = self
                .data
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *guard = Some((Arc::clone(&new_data), Instant::now()));
        }

        Ok(new_data)
    }

    /// 🔄 更新共享缓存数据
    pub fn set_shared(&self, data: Arc<T>) {
        let mut guard = self
            .data
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = Some((data, Instant::now()));
    }
}

#[allow(dead_code)]
impl<T: Clone> ConfigCache<T> {
    /// 📖 获取缓存数据或通过加载器加载
    ///
    /// 如果缓存有效，直接返回缓存数据
    /// 如果缓存无效或过期，调用 loader 加载新数据并缓存
    ///
    /// # 参数
    /// - `loader`: 数据加载函数
    ///
    /// # 返回
    /// - Ok(T): 缓存或新加载的数据
    /// - Err: 加载失败
    pub fn get_or_load<F, E>(&self, loader: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        self.get_or_load_shared(loader)
            .map(|cached| cached.as_ref().clone())
    }

    /// 🔍 尝试获取缓存数据（不触发加载）
    ///
    /// 如果缓存有效，返回 Some(T)
    /// 如果缓存无效或过期，返回 None
    pub fn get(&self) -> Option<T> {
        self.get_shared().map(|cached| cached.as_ref().clone())
    }

    /// 🧹 手动失效缓存
    ///
    /// 清除当前缓存，下次 get_or_load 将重新加载数据
    pub fn invalidate(&self) {
        let mut guard = self
            .data
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = None;
    }

    /// 🔄 更新缓存数据
    ///
    /// 直接设置缓存数据，而不通过 loader
    pub fn set(&self, data: T) {
        self.set_shared(Arc::new(data));
    }

    /// ⏰ 检查缓存是否有效
    pub fn is_valid(&self) -> bool {
        let guard = self
            .data
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some((_, cached_at)) = *guard {
            cached_at.elapsed() < self.ttl
        } else {
            false
        }
    }

    /// 📊 获取缓存状态信息
    pub fn status(&self) -> CacheStatus {
        let guard = self
            .data
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match *guard {
            Some((_, cached_at)) => {
                let age = cached_at.elapsed();
                if age < self.ttl {
                    CacheStatus::Valid { age }
                } else {
                    CacheStatus::Expired { age }
                }
            }
            None => CacheStatus::Empty,
        }
    }
}

/// 📊 缓存状态枚举
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CacheStatus {
    /// 缓存有效
    Valid { age: Duration },
    /// 缓存已过期
    Expired { age: Duration },
    /// 缓存为空
    Empty,
}

// ═══════════════════════════════════════════════════════════
// 🧪 测试
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    #[test]
    fn test_cache_basic() {
        let cache: ConfigCache<String> = ConfigCache::new(Duration::from_secs(60));

        // 初始为空
        assert!(cache.get().is_none());
        assert!(!cache.is_valid());

        // 加载数据
        let result = cache.get_or_load(|| Ok::<_, ()>("test".to_string()));
        assert_eq!(result.unwrap(), "test");

        // 缓存应该有效
        assert!(cache.is_valid());
        assert_eq!(cache.get(), Some("test".to_string()));
    }

    #[test]
    fn test_cache_invalidate() {
        let cache: ConfigCache<i32> = ConfigCache::new(Duration::from_secs(60));

        cache.get_or_load(|| Ok::<_, ()>(42)).unwrap();
        assert!(cache.is_valid());

        cache.invalidate();
        assert!(!cache.is_valid());
        assert!(cache.get().is_none());
    }

    #[test]
    fn test_cache_loader_called_once() {
        let cache: ConfigCache<i32> = ConfigCache::new(Duration::from_secs(60));
        let call_count = Arc::new(AtomicUsize::new(0));

        // 第一次调用 loader
        let count1 = Arc::clone(&call_count);
        cache
            .get_or_load(|| {
                count1.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ()>(42)
            })
            .unwrap();

        // 第二次不应调用 loader
        let count2 = Arc::clone(&call_count);
        cache
            .get_or_load(|| {
                count2.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ()>(100)
            })
            .unwrap();

        // loader 应该只被调用一次
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_cache_thread_safety() {
        let cache = Arc::new(ConfigCache::new(Duration::from_secs(60)));
        let call_count = Arc::new(AtomicUsize::new(0));

        let mut handles = vec![];

        for _ in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let count_clone = Arc::clone(&call_count);

            let handle = thread::spawn(move || {
                cache_clone
                    .get_or_load(|| {
                        count_clone.fetch_add(1, Ordering::SeqCst);
                        // 模拟一点延迟
                        thread::sleep(Duration::from_millis(10));
                        Ok::<_, ()>("data".to_string())
                    })
                    .unwrap()
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // 并发 miss 只应触发一次真实加载
        assert!(cache.is_valid());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_cache_ttl_expiry() {
        let cache: ConfigCache<i32> = ConfigCache::new(Duration::from_millis(50));

        cache.get_or_load(|| Ok::<_, ()>(1)).unwrap();
        assert!(cache.is_valid());

        // 等待 TTL 过期
        thread::sleep(Duration::from_millis(100));

        assert!(!cache.is_valid());
        assert!(cache.get().is_none());

        // 再次加载应该成功
        let result = cache.get_or_load(|| Ok::<_, ()>(2)).unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn test_cache_status() {
        let cache: ConfigCache<i32> = ConfigCache::new(Duration::from_secs(60));

        // 初始为空
        matches!(cache.status(), CacheStatus::Empty);

        cache.get_or_load(|| Ok::<_, ()>(42)).unwrap();

        // 应该有效
        matches!(cache.status(), CacheStatus::Valid { .. });
    }

    #[test]
    fn test_cache_shared_hits_reuse_arc() {
        let cache: ConfigCache<String> = ConfigCache::new(Duration::from_secs(60));

        let first = cache
            .get_or_load_shared(|| Ok::<_, ()>("shared".to_string()))
            .unwrap();
        let second = cache.get_shared().unwrap();

        assert!(Arc::ptr_eq(&first, &second));
    }
}
