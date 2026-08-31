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
use fs4::{FileExt, TryLockError};
use std::fs::{self, File, OpenOptions};
use std::io;
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

/// Normalize the fs4 1.x try-lock result to CCR's established 0.13-era shape.
///
/// Keeping this adapter local lets the acquisition loop retain its existing
/// timeout, retry, and final error mapping while still distinguishing a held
/// lock from a real I/O error.
fn map_try_lock_result(result: std::result::Result<(), TryLockError>) -> io::Result<bool> {
    match result {
        Ok(()) => Ok(true),
        Err(TryLockError::WouldBlock) => Ok(false),
        Err(TryLockError::Error(error)) => Err(error),
    }
}

fn try_lock_exclusive(file: &File) -> io::Result<bool> {
    map_try_lock_result(FileExt::try_lock(file))
}

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
            .truncate(true)
            .open(&lock_path)
            .map_err(|e| CcrError::FileLockError(format!("无法打开锁文件: {}", e)))?;

        // 尝试获取锁,带超时
        let start = Instant::now();
        let mut retry_count = 0;
        loop {
            // 兼容适配层保持旧合同：Ok(true) = 获取；Ok(false) = 被占用；Err = I/O 错误。
            match try_lock_exclusive(&file) {
                Ok(true) => {
                    tracing::debug!("成功获取文件锁: {:?}", lock_path);
                    return Ok(FileLock { file, lock_path });
                }
                Ok(false) | Err(_) if start.elapsed() < timeout => {
                    // 🎯 优化：使用指数退避策略，减少 CPU 消耗
                    // 等待时间：50ms, 100ms, 200ms, 400ms...最多 400ms
                    let wait_ms = 50 * (1 << retry_count).min(8);
                    std::thread::sleep(Duration::from_millis(wait_ms));
                    retry_count += 1;
                    continue;
                }
                Ok(false) => {
                    return Err(CcrError::LockTimeout(format!(
                        "获取文件锁超时 (锁被占用): {:?}",
                        lock_path.display()
                    )));
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
        // 显式调用 fs4::FileExt，避免在较新 Rust 版本中命中 std::fs::File::unlock 的 MSRV 变更
        let _ = FileExt::unlock(&self.file);
        tracing::debug!("🔓 文件锁已自动释放: {:?}", self.lock_path);
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
    pub fn with_default_path() -> Result<Self> {
        // 🔍 检查环境变量
        let lock_dir = if let Ok(custom_dir) = std::env::var("CCR_LOCK_DIR") {
            std::path::PathBuf::from(custom_dir)
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| CcrError::FileLockError("无法获取用户主目录".into()))?;
            home.join(".claude").join(".locks")
        };

        tracing::debug!("使用锁目录: {:?}", &lock_dir);
        Ok(Self::new(lock_dir))
    }

    /// 📂 返回锁目录路径
    pub fn lock_dir(&self) -> &Path {
        &self.lock_dir
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
    /// let lock_manager = LockManager::with_default_path()?;
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
    #[allow(dead_code)]
    pub fn lock_history(&self, timeout: Duration) -> Result<FileLock> {
        self.lock_resource("ccr_history", timeout)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    const CHILD_LOCK_PATH_ENV: &str = "CCR_TEST_CHILD_LOCK_PATH";
    const CHILD_LOCK_READY_ENV: &str = "CCR_TEST_CHILD_LOCK_READY";
    const CHILD_LOCK_RELEASE_ENV: &str = "CCR_TEST_CHILD_LOCK_RELEASE";

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
    fn test_try_lock_result_adapter_preserves_all_outcomes() {
        assert!(map_try_lock_result(Ok(())).unwrap());
        assert!(!map_try_lock_result(Err(TryLockError::WouldBlock)).unwrap());

        let source = io::Error::other("adapter source");
        let error = map_try_lock_result(Err(TryLockError::Error(io::Error::new(
            io::ErrorKind::PermissionDenied,
            source,
        ))))
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        let source = error
            .get_ref()
            .and_then(|source| source.downcast_ref::<io::Error>())
            .unwrap();
        assert_eq!(source.kind(), io::ErrorKind::Other);
        assert_eq!(source.to_string(), "adapter source");
    }

    #[test]
    fn test_file_lock_contention_then_release_allows_retry() {
        let temp_dir = tempfile::tempdir().unwrap();
        let lock_path = temp_dir.path().join("test.lock");

        let lock1 = FileLock::new(&lock_path, Duration::from_secs(5)).unwrap();

        // Zero timeout exercises contention without scheduler-dependent sleeps.
        let lock2_result = FileLock::new(&lock_path, Duration::ZERO);
        assert!(matches!(lock2_result, Err(CcrError::LockTimeout(_))));

        drop(lock1);
        let _lock2 = FileLock::new(&lock_path, Duration::ZERO).unwrap();
    }

    #[test]
    fn cross_process_lock_holder_child() {
        let Some(lock_path) = std::env::var_os(CHILD_LOCK_PATH_ENV) else {
            return;
        };
        let ready_path = PathBuf::from(std::env::var_os(CHILD_LOCK_READY_ENV).unwrap());
        let release_path = PathBuf::from(std::env::var_os(CHILD_LOCK_RELEASE_ENV).unwrap());

        let _lock = FileLock::new(PathBuf::from(lock_path), Duration::from_secs(5)).unwrap();
        fs::write(&ready_path, b"ready").unwrap();

        let start = Instant::now();
        while !release_path.exists() {
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "parent did not release the child lock in time"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn test_file_lock_cross_process_contention_and_release() {
        let temp_dir = tempfile::tempdir().unwrap();
        let lock_path = temp_dir.path().join("cross-process.lock");
        let ready_path = temp_dir.path().join("child.ready");
        let release_path = temp_dir.path().join("child.release");

        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "core::lock::tests::cross_process_lock_holder_child",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD_LOCK_PATH_ENV, &lock_path)
            .env(CHILD_LOCK_READY_ENV, &ready_path)
            .env(CHILD_LOCK_RELEASE_ENV, &release_path)
            .stdout(Stdio::null())
            .spawn()
            .unwrap();

        let start = Instant::now();
        while !ready_path.exists() {
            if let Some(status) = child.try_wait().unwrap() {
                panic!("child lock holder exited before ready: {status}");
            }
            if start.elapsed() >= Duration::from_secs(5) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("child lock holder did not become ready in time");
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        let contender = FileLock::new(&lock_path, Duration::ZERO);
        let contended = matches!(contender, Err(CcrError::LockTimeout(_)));

        fs::write(&release_path, b"release").unwrap();
        let child_status = child.wait().unwrap();

        assert!(contended, "a second process unexpectedly acquired the lock");
        assert!(child_status.success(), "child lock holder failed");
        let _lock = FileLock::new(&lock_path, Duration::ZERO).unwrap();
    }

    #[test]
    fn test_file_lock_open_error_preserves_file_lock_error() {
        let temp_dir = tempfile::tempdir().unwrap();
        let parent_file = temp_dir.path().join("not-a-directory");
        fs::write(&parent_file, b"occupied").unwrap();

        let result = FileLock::new(parent_file.join("test.lock"), Duration::ZERO);
        assert!(matches!(result, Err(CcrError::FileLockError(_))));
    }

    #[test]
    fn test_lock_manager() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = LockManager::new(temp_dir.path());

        let _lock = manager.lock_settings(Duration::from_secs(5)).unwrap();
        assert!(temp_dir.path().join("claude_settings.lock").exists());

        // 锁在作用域结束时自动释放
    }
}
