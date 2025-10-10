// CCR 文件锁模块
// 提供跨进程的文件锁功能，确保并发安全

use crate::error::{CcrError, Result};
use fs4::fs_std::FileExt;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// 文件锁
///
/// 提供跨进程的互斥锁功能
pub struct FileLock {
    file: File,
    lock_path: PathBuf,
}

impl FileLock {
    /// 创建一个新的文件锁
    ///
    /// # 参数
    /// * `lock_path` - 锁文件的路径
    /// * `timeout` - 获取锁的超时时间
    ///
    /// # 返回
    /// * `Ok(FileLock)` - 成功获取锁
    /// * `Err(CcrError)` - 获取锁失败或超时
    pub fn new<P: AsRef<Path>>(lock_path: P, timeout: Duration) -> Result<Self> {
        let lock_path = lock_path.as_ref().to_path_buf();

        // 确保锁文件目录存在
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CcrError::FileLockError(format!("无法创建锁文件目录: {}", e))
            })?;
        }

        // 打开或创建锁文件
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&lock_path)
            .map_err(|e| CcrError::FileLockError(format!("无法打开锁文件: {}", e)))?;

        // 尝试获取锁，带超时
        let start = Instant::now();
        loop {
            match file.try_lock_exclusive() {
                Ok(_) => {
                    log::debug!("成功获取文件锁: {:?}", lock_path);
                    return Ok(FileLock { file, lock_path });
                }
                Err(_) if start.elapsed() < timeout => {
                    // 未超时，等待一小段时间后重试
                    std::thread::sleep(Duration::from_millis(100));
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

    /// 释放锁
    pub fn unlock(self) -> Result<()> {
        self.file
            .unlock()
            .map_err(|e| CcrError::FileLockError(format!("释放锁失败: {}", e)))?;
        log::debug!("释放文件锁: {:?}", self.lock_path);
        Ok(())
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // 确保锁总是被释放
        let _ = self.file.unlock();
        log::debug!("文件锁已自动释放: {:?}", self.lock_path);
    }
}

/// 文件锁管理器
///
/// 统一管理多个资源的锁
pub struct LockManager {
    lock_dir: PathBuf,
}

impl LockManager {
    /// 创建新的锁管理器
    ///
    /// # 参数
    /// * `lock_dir` - 锁文件存放目录
    pub fn new<P: AsRef<Path>>(lock_dir: P) -> Self {
        let lock_dir = lock_dir.as_ref().to_path_buf();
        Self { lock_dir }
    }

    /// 获取默认锁管理器
    ///
    /// 使用 ~/.claude/.locks 作为锁文件目录
    pub fn default() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::FileLockError("无法获取用户主目录".into()))?;
        let lock_dir = home.join(".claude").join(".locks");
        Ok(Self::new(lock_dir))
    }

    /// 为指定资源创建锁路径
    fn create_lock_path(&self, resource_name: &str) -> PathBuf {
        self.lock_dir.join(format!("{}.lock", resource_name))
    }

    /// 获取配置文件锁
    pub fn lock_config(&self, timeout: Duration) -> Result<FileLock> {
        let lock_path = self.create_lock_path("ccs_config");
        FileLock::new(lock_path, timeout)
    }

    /// 获取设置文件锁
    pub fn lock_settings(&self, timeout: Duration) -> Result<FileLock> {
        let lock_path = self.create_lock_path("claude_settings");
        FileLock::new(lock_path, timeout)
    }

    /// 获取历史文件锁
    pub fn lock_history(&self, timeout: Duration) -> Result<FileLock> {
        let lock_path = self.create_lock_path("ccr_history");
        FileLock::new(lock_path, timeout)
    }

    /// 获取自定义资源锁
    pub fn lock_resource(&self, resource_name: &str, timeout: Duration) -> Result<FileLock> {
        let lock_path = self.create_lock_path(resource_name);
        FileLock::new(lock_path, timeout)
    }

    /// 清理过期的锁文件
    ///
    /// 删除所有超过指定时间未被修改的锁文件
    pub fn cleanup_stale_locks(&self, max_age: Duration) -> Result<usize> {
        if !self.lock_dir.exists() {
            return Ok(0);
        }

        let mut cleaned = 0;
        let now = std::time::SystemTime::now();

        for entry in fs::read_dir(&self.lock_dir)
            .map_err(|e| CcrError::FileLockError(format!("读取锁目录失败: {}", e)))?
        {
            let entry =
                entry.map_err(|e| CcrError::FileLockError(format!("读取目录项失败: {}", e)))?;
            let path = entry.path();

            // 只处理 .lock 文件
            if !path.extension().map_or(false, |ext| ext == "lock") {
                continue;
            }

            // 检查文件修改时间
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(age) = now.duration_since(modified) {
                        if age > max_age {
                            // 尝试删除过期锁文件
                            if fs::remove_file(&path).is_ok() {
                                log::info!("清理过期锁文件: {:?}", path);
                                cleaned += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(cleaned)
    }
}

/// 创建标准锁路径
///
/// 统一生成锁文件路径的辅助函数
pub fn create_lock_path(resource_name: &str) -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| CcrError::FileLockError("无法获取用户主目录".into()))?;
    Ok(home
        .join(".claude")
        .join(".locks")
        .join(format!("{}.lock", resource_name)))
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
        let lock = FileLock::new(&lock_path, Duration::from_secs(5)).unwrap();

        // 锁应该被持有
        assert!(lock_path.exists());

        // 释放锁
        lock.unlock().unwrap();
    }

    #[test]
    fn test_file_lock_timeout() {
        let temp_dir = tempfile::tempdir().unwrap();
        let lock_path = temp_dir.path().join("test.lock");

        // 第一个锁
        let _lock1 = FileLock::new(&lock_path, Duration::from_secs(5)).unwrap();

        // 第二个锁应该超时失败
        let lock2_result = FileLock::new(&lock_path, Duration::from_millis(200));
        assert!(lock2_result.is_err());
    }

    #[test]
    fn test_lock_manager() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = LockManager::new(temp_dir.path());

        let lock = manager
            .lock_resource("test_resource", Duration::from_secs(5))
            .unwrap();
        assert!(temp_dir.path().join("test_resource.lock").exists());

        lock.unlock().unwrap();
    }

    #[test]
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

        // 应该等待了至少 400ms (500ms - 100ms)
        assert!(elapsed >= Duration::from_millis(300));

        handle.join().unwrap();
    }
}
