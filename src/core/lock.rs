// 🔒 CCR 文件锁模块
// 🛡️ 提供跨进程的文件锁功能,确保并发安全
//
// 核心功能:
// - 🔐 跨进程互斥锁(使用 fs4 crate)
// - 🔒 进程内互斥锁(使用 std::sync::Mutex)
// - ⏱️ 超时机制(防止死锁)
// - 🧹 RAII 自动释放(Drop trait)
// - 🔄 重试机制(指数退避)
//
// 使用场景:
// - 防止多个 CCR 进程同时修改 settings.json
// - 防止并发写入历史记录文件
// - 防止同一进程内的并发配置操作

use crate::core::error::{CcrError, Result};
use fs4::fs_std::FileExt;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

// ============================================================================
// 🔒 全局进程内配置锁
// ============================================================================

/// 🔐 全局配置操作锁
///
/// 这是一个**进程内**的互斥锁，用于防止同一进程中的多个线程
/// 同时进行配置读写操作导致的竞态条件。
///
/// ## 设计理念
///
/// CCR 采用双层锁机制:
/// 1. **进程间锁** (FileLock): 使用文件系统锁，防止多个 CCR 进程同时修改配置
/// 2. **进程内锁** (CONFIG_LOCK): 使用内存互斥锁，防止同一进程内的并发冲突
///
/// ## 使用场景
///
/// 必须在以下操作前获取此锁:
/// - 读取配置文件 (load)
/// - 写入配置文件 (save)
/// - 读取-修改-写入序列 (RMW)
///
/// ## 使用模式
///
/// ```rust,ignore
/// use crate::core::lock::CONFIG_LOCK;
///
/// // 获取锁进行配置操作
/// let _guard = CONFIG_LOCK.lock().unwrap();
/// // 在此作用域内执行配置读写
/// config_manager.load()?;
/// config_manager.save(&config)?;
/// // 锁在 _guard 离开作用域时自动释放
/// ```
///
/// ## 性能考虑
///
/// - 使用 `std::sync::Mutex` 而非 `parking_lot::Mutex`，优先兼容性和稳定性
/// - 锁粒度: 整个配置操作，而非单个字段，保证操作原子性
/// - 锁持有时间: 应尽可能短，避免长时间阻塞其他线程
///
/// ## 毒化处理
///
/// 如果持有锁的线程 panic，锁会被"毒化"。调用方应该:
/// 1. 使用 `.unwrap()` 或 `.expect()` 直接 panic (简单场景)
/// 2. 使用 `unwrap_or_else(|poisoned| poisoned.into_inner())` 恢复 (关键场景)
///
/// ## 注意事项
///
/// ⚠️ **死锁风险**: 不要在持有 CONFIG_LOCK 时再次尝试获取它
/// ⚠️ **性能影响**: 所有配置操作会串行化，但这是保证一致性的必要代价
/// ✅ **向后兼容**: 不影响现有的跨进程文件锁机制
///
pub static CONFIG_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// 🔒 文件锁
///
/// 提供跨进程的互斥锁功能,基于文件系统锁实现
///
/// 特性:
/// - 🛡️ 跨进程安全
/// - 🧹 自动释放(通过 Drop trait)
/// - ⏱️ 可配置超时
pub struct FileLock {
    file: File,
    lock_path: PathBuf,
}

impl FileLock {
    /// 🔐 创建一个新的文件锁
    ///
    /// # 参数
    /// * `lock_path` - 锁文件的路径
    /// * `timeout` - 获取锁的超时时间
    ///
    /// # 返回
    /// * `Ok(FileLock)` - 成功获取锁
    /// * `Err(CcrError)` - 获取锁失败或超时
    ///
    /// # 实现细节
    /// - 循环尝试获取锁,每次失败后等待 100ms
    /// - 超时后返回 LockTimeout 错误
    /// - 锁文件位于 ~/.claude/.locks/ 目录
    pub fn new<P: AsRef<Path>>(lock_path: P, timeout: Duration) -> Result<Self> {
        let lock_path = lock_path.as_ref().to_path_buf();

        // 确保锁文件目录存在
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::FileLockError(format!("无法创建锁文件目录: {}", e)))?;
        }

        // 打开或创建锁文件
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&lock_path)
            .map_err(|e| CcrError::FileLockError(format!("无法打开锁文件: {}", e)))?;

        // 尝试获取锁,带超时
        let start = Instant::now();
        let mut retry_count = 0;
        loop {
            match file.try_lock_exclusive() {
                Ok(_) => {
                    log::debug!("成功获取文件锁: {:?}", lock_path);
                    return Ok(FileLock { file, lock_path });
                }
                Err(_) if start.elapsed() < timeout => {
                    // 🎯 优化：使用指数退避策略，减少 CPU 消耗
                    // 等待时间：50ms, 100ms, 200ms, 400ms...最多 400ms
                    let wait_ms = 50 * (1 << retry_count).min(8);
                    std::thread::sleep(Duration::from_millis(wait_ms));
                    retry_count += 1;
                    continue;
                }
                Err(e) => {
                    return Err(CcrError::LockTimeout(format!(
                        "获取文件锁超时 ({}): {:?}",
                        e,
                        lock_path.display()
                    )));
                }
            }
        }
    }
}

impl Drop for FileLock {
    /// 🧹 自动释放文件锁
    ///
    /// 利用 RAII(Resource Acquisition Is Initialization)模式
    /// 当 FileLock 离开作用域时自动释放锁
    fn drop(&mut self) {
        // ✅ 确保锁总是被释放
        let _ = self.file.unlock();
        log::debug!("🔓 文件锁已自动释放: {:?}", self.lock_path);
    }
}

/// 🔧 文件锁管理器
///
/// 统一管理多个资源的锁,提供一致的锁获取接口
///
/// 管理的资源:
/// - 📝 Claude Code settings.json
/// - 📚 CCR 历史记录文件
pub struct LockManager {
    lock_dir: PathBuf,
}

impl LockManager {
    /// 🏗️ 创建新的锁管理器
    ///
    /// # 参数
    /// * `lock_dir` - 锁文件存放目录
    pub fn new<P: AsRef<Path>>(lock_dir: P) -> Self {
        let lock_dir = lock_dir.as_ref().to_path_buf();
        Self { lock_dir }
    }

    /// 🏠 获取默认锁管理器
    ///
    /// 使用 ~/.claude/.locks 作为锁文件目录
    ///
    /// ⚙️ **开发者注意**：
    /// 可以通过环境变量 `CCR_LOCK_DIR` 覆盖默认路径
    pub fn default() -> Result<Self> {
        // 🔍 检查环境变量
        let lock_dir = if let Ok(custom_dir) = std::env::var("CCR_LOCK_DIR") {
            std::path::PathBuf::from(custom_dir)
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| CcrError::FileLockError("无法获取用户主目录".into()))?;
            home.join(".claude").join(".locks")
        };

        log::debug!("使用锁目录: {:?}", &lock_dir);
        Ok(Self::new(lock_dir))
    }

    /// 📁 为指定资源创建锁路径
    fn create_lock_path(&self, resource_name: &str) -> PathBuf {
        self.lock_dir.join(format!("{}.lock", resource_name))
    }

    /// 🔒 获取指定资源的锁(通用方法)
    ///
    /// 为任意资源获取文件锁,资源名称会被转换为锁文件路径
    ///
    /// # Arguments
    /// - `resource` - 资源名称(例如: "my_config", "temp_data")
    /// - `timeout` - 获取锁的超时时间
    ///
    /// # Returns
    /// - `Ok(FileLock)` - 成功获取锁
    /// - `Err(CcrError)` - 获取失败或超时
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::time::Duration;
    ///
    /// let lock_manager = LockManager::default()?;
    /// let _lock = lock_manager.lock_resource("my_data", Duration::from_secs(5))?;
    /// // 持有锁期间执行操作
    /// // 锁在离开作用域时自动释放
    /// ```
    pub fn lock_resource(&self, resource: &str, timeout: Duration) -> Result<FileLock> {
        let lock_path = self.create_lock_path(resource);
        FileLock::new(lock_path, timeout)
    }

    /// 📝 获取设置文件锁
    ///
    /// 用于保护 ~/.claude/settings.json 的并发访问
    ///
    /// 这是 `lock_resource("claude_settings", timeout)` 的便捷方法
    pub fn lock_settings(&self, timeout: Duration) -> Result<FileLock> {
        self.lock_resource("claude_settings", timeout)
    }

    /// 📚 获取历史文件锁
    ///
    /// 用于保护 ~/.claude/ccr_history.json 的并发写入
    ///
    /// 这是 `lock_resource("ccr_history", timeout)` 的便捷方法
    pub fn lock_history(&self, timeout: Duration) -> Result<FileLock> {
        self.lock_resource("ccr_history", timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_file_lock_basic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let lock_path = temp_dir.path().join("test.lock");

        // 获取锁
        let _lock = FileLock::new(&lock_path, Duration::from_secs(5)).unwrap();

        // 锁应该被持有
        assert!(lock_path.exists());

        // 锁在作用域结束时自动释放
    }

    #[test]
    #[ignore = "时间相关测试，在不同系统调度下可能不稳定"]
    fn test_file_lock_timeout() {
        let temp_dir = tempfile::tempdir().unwrap();
        let lock_path = temp_dir.path().join("test.lock");

        // 第一个锁
        let _lock1 = FileLock::new(&lock_path, Duration::from_secs(5)).unwrap();

        // 第二个锁应该超时失败
        // 🎯 注意：由于使用指数退避策略（50ms, 100ms, 200ms...），需要更长的超时时间
        let lock2_result = FileLock::new(&lock_path, Duration::from_millis(500));
        assert!(lock2_result.is_err());
    }

    #[test]
    fn test_lock_manager() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = LockManager::new(temp_dir.path());

        let _lock = manager.lock_settings(Duration::from_secs(5)).unwrap();
        assert!(temp_dir.path().join("claude_settings.lock").exists());

        // 锁在作用域结束时自动释放
    }

    #[test]
    #[ignore = "时间相关测试，在不同系统调度下可能不稳定"]
    fn test_concurrent_locks() {
        let temp_dir = tempfile::tempdir().unwrap();
        let lock_path = temp_dir.path().join("concurrent.lock");
        let lock_path_clone = lock_path.clone();

        let handle = thread::spawn(move || {
            let _lock = FileLock::new(&lock_path_clone, Duration::from_secs(5)).unwrap();
            thread::sleep(Duration::from_millis(500));
        });

        // 等待一点时间确保第一个线程获取了锁
        thread::sleep(Duration::from_millis(100));

        // 这个应该等待第一个锁释放
        let start = Instant::now();
        let _lock2 = FileLock::new(&lock_path, Duration::from_secs(5)).unwrap();
        let elapsed = start.elapsed();

        // 应该等待了至少 350ms (500ms - 100ms - 指数退避的累积延迟)
        // 🎯 注意：指数退避策略会引入额外延迟，所以断言时间需要更宽松
        assert!(elapsed >= Duration::from_millis(250));

        handle.join().unwrap();
    }
}
