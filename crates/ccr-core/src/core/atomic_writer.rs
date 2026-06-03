// 📝 原子写入器
// 提供安全的原子文件写入功能,避免数据损坏

use crate::core::error::{CcrError, Result};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::thread;
#[cfg(windows)]
use std::time::Duration;
#[cfg(windows)]
use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
use tempfile::NamedTempFile;
use tokio::fs as async_fs;
use uuid::Uuid;

/// 📝 原子写入器
///
/// 使用临时文件 + 原子重命名模式确保文件写入的原子性
///
/// 工作原理:
/// 1. 创建临时文件在同一目录(确保在同一文件系统)
/// 2. 写入内容到临时文件
/// 3. 使用原子 rename 操作替换目标文件
///
/// 优点:
/// - ✅ 即使写入中断,原文件保持完整
/// - ✅ 读取者不会看到部分写入的数据
/// - ✅ 跨平台支持(Unix 和 Windows)
///
/// # Examples
///
/// ```rust,ignore
/// use ccr_core::core::AtomicWriter;
/// use std::path::Path;
///
/// let writer = AtomicWriter::new(Path::new("/path/to/file.json"))?;
/// let content = r#"{"key": "value"}"#;
/// writer.write(content.as_bytes())?;
/// ```
#[allow(dead_code)]
pub struct AtomicWriter {
    target_path: PathBuf,
}

/// 📝 异步原子写入器
pub struct AsyncAtomicWriter {
    target_path: PathBuf,
}

#[cfg(windows)]
const ATOMIC_WRITE_RETRY_LIMIT: usize = 6;

#[cfg(windows)]
fn is_windows_atomic_retry_error(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::PermissionDenied
            | std::io::ErrorKind::WouldBlock
            | std::io::ErrorKind::Interrupted
    ) || matches!(error.raw_os_error(), Some(5 | 32 | 33))
}

#[cfg(windows)]
const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
#[cfg(windows)]
const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn MoveFileExW(existing_file_name: *const u16, new_file_name: *const u16, flags: u32) -> i32;
}

#[cfg(windows)]
fn path_to_wide(path: &Path) -> Vec<u16> {
    OsStr::new(path)
        .encode_wide()
        .chain(once(0))
        .collect::<Vec<_>>()
}

#[cfg(windows)]
fn move_file_replace_existing(source: &Path, target: &Path) -> std::io::Result<()> {
    let source = path_to_wide(source);
    let target = path_to_wide(target);
    let flags = MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH;

    // SAFETY: Pointers come from nul-terminated UTF-16 buffers that live for the
    // duration of the call. MoveFileExW does not retain these pointers.
    let ok = unsafe { MoveFileExW(source.as_ptr(), target.as_ptr(), flags) };
    if ok == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(windows)]
fn retry_windows_atomic_replace<F>(target_path: &Path, mut replace: F) -> std::io::Result<()>
where
    F: FnMut() -> std::io::Result<()>,
{
    for attempt in 0..ATOMIC_WRITE_RETRY_LIMIT {
        match replace() {
            Ok(()) => return Ok(()),
            Err(error) => {
                if !is_windows_atomic_retry_error(&error) || attempt + 1 == ATOMIC_WRITE_RETRY_LIMIT
                {
                    return Err(error);
                }

                tracing::warn!(
                    target_path = %target_path.display(),
                    attempt = attempt + 1,
                    error = %error,
                    "⚠️ Windows 原子替换失败，准备重试"
                );

                thread::sleep(Duration::from_millis(((attempt + 1) * 20) as u64));
            }
        }
    }

    Err(std::io::Error::other(format!(
        "retry limit exhausted for {}",
        target_path.display()
    )))
}

#[cfg(windows)]
fn replace_path_windows(source: &Path, target: &Path) -> std::io::Result<()> {
    retry_windows_atomic_replace(target, || move_file_replace_existing(source, target))
}

#[allow(dead_code)]
impl AtomicWriter {
    /// 🏗️ 创建新的原子写入器
    ///
    /// # Arguments
    /// - `target_path` - 目标文件路径
    ///
    /// # Returns
    /// 新的 `AtomicWriter` 实例
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(target_path: P) -> Self {
        Self {
            target_path: target_path.as_ref().to_path_buf(),
        }
    }

    /// 💾 原子写入内容到文件
    ///
    /// # Arguments
    /// - `content` - 要写入的字节内容
    ///
    /// # Returns
    /// - `Ok(())` - 写入成功
    /// - `Err(CcrError)` - 写入失败
    ///
    /// # Process
    /// 1. 在目标目录创建临时文件
    /// 2. 写入内容到临时文件
    /// 3. 原子替换目标文件
    pub fn write(&self, content: &[u8]) -> Result<()> {
        // 📁 确保目标目录存在
        if let Some(parent) = self.target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CcrError::IoError(std::io::Error::other(format!("创建目录失败: {}", e)))
            })?;
        }

        // 📄 在同一目录创建临时文件(确保在同一文件系统)
        let temp_file = if let Some(parent) = self.target_path.parent() {
            NamedTempFile::new_in(parent)
        } else {
            NamedTempFile::new()
        }
        .map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!("创建临时文件失败: {}", e)))
        })?;

        // ✍️ 写入内容
        fs::write(temp_file.path(), content).map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!("写入临时文件失败: {}", e)))
        })?;

        self.persist_temp_file(temp_file)?;

        tracing::debug!("✅ 文件已原子写入: {:?}", self.target_path);
        Ok(())
    }

    /// 💾 原子写入字符串内容到文件
    ///
    /// # Arguments
    /// - `content` - 要写入的字符串内容
    ///
    /// # Returns
    /// - `Ok(())` - 写入成功
    /// - `Err(CcrError)` - 写入失败
    pub fn write_string(&self, content: &str) -> Result<()> {
        self.write(content.as_bytes())
    }

    fn persist_temp_file(&self, temp_file: NamedTempFile) -> Result<()> {
        #[cfg(windows)]
        {
            self.persist_temp_file_windows(temp_file)
        }

        #[cfg(not(windows))]
        {
            temp_file.persist(&self.target_path).map_err(|e| {
                CcrError::IoError(std::io::Error::other(format!(
                    "原子替换文件失败 (target: {}): {}",
                    self.target_path.display(),
                    e
                )))
            })?;
            Ok(())
        }
    }

    #[cfg(windows)]
    fn persist_temp_file_windows(&self, temp_file: NamedTempFile) -> Result<()> {
        let temp_path = temp_file.into_temp_path();

        replace_path_windows(temp_path.as_ref(), &self.target_path).map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!(
                "原子替换文件失败 (target: {}): {}",
                self.target_path.display(),
                e
            )))
        })
    }
}

impl AsyncAtomicWriter {
    /// 🏗️ 创建新的异步原子写入器
    pub fn new<P: AsRef<Path>>(target_path: P) -> Self {
        Self {
            target_path: target_path.as_ref().to_path_buf(),
        }
    }

    /// 💾 异步原子写入内容到文件
    pub async fn write_async(&self, content: &[u8]) -> Result<()> {
        if let Some(parent) = self.target_path.parent() {
            async_fs::create_dir_all(parent).await.map_err(|e| {
                CcrError::IoError(std::io::Error::other(format!("创建目录失败: {}", e)))
            })?;
        }

        let temp_path = self.temp_path();

        async_fs::write(&temp_path, content).await.map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!("写入临时文件失败: {}", e)))
        })?;

        if let Err(e) = persist_temp_path_async(temp_path.clone(), self.target_path.clone()).await {
            let _ = async_fs::remove_file(&temp_path).await;
            return Err(CcrError::IoError(std::io::Error::other(format!(
                "原子替换文件失败: {}",
                e
            ))));
        }

        tracing::debug!("✅ 文件已原子写入: {:?}", self.target_path);
        Ok(())
    }

    /// 💾 异步原子写入字符串内容到文件
    pub async fn write_string_async(&self, content: &str) -> Result<()> {
        self.write_async(content.as_bytes()).await
    }

    fn temp_path(&self) -> PathBuf {
        let parent = self.target_path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = self
            .target_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("tmp");
        let temp_name = format!(".{}.tmp-{}", file_name, Uuid::new_v4());
        parent.join(temp_name)
    }
}

async fn persist_temp_path_async(temp_path: PathBuf, target_path: PathBuf) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        tokio::task::spawn_blocking(move || replace_path_windows(&temp_path, &target_path))
            .await
            .map_err(|e| std::io::Error::other(format!("原子替换任务失败: {}", e)))?
    }

    #[cfg(not(windows))]
    {
        async_fs::rename(temp_path, target_path).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_atomic_write() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test.txt");

        let writer = AtomicWriter::new(&target_path);
        writer.write(b"Hello, World!").unwrap();

        // 验证文件存在且内容正确
        assert!(target_path.exists());
        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_atomic_write_string() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test.txt");

        let writer = AtomicWriter::new(&target_path);
        writer.write_string("Hello, Rust!").unwrap();

        // 验证文件内容
        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "Hello, Rust!");
    }

    #[test]
    fn test_atomic_overwrite() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test.txt");

        // 第一次写入
        let writer = AtomicWriter::new(&target_path);
        writer.write_string("First content").unwrap();
        assert_eq!(fs::read_to_string(&target_path).unwrap(), "First content");

        // 第二次覆盖写入
        writer.write_string("Second content").unwrap();
        assert_eq!(fs::read_to_string(&target_path).unwrap(), "Second content");
    }

    #[test]
    fn test_atomic_overwrite_repeatedly() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test-repeat.txt");
        let writer = AtomicWriter::new(&target_path);

        for idx in 0..5 {
            writer.write_string(&format!("content-{idx}")).unwrap();
        }

        assert_eq!(fs::read_to_string(&target_path).unwrap(), "content-4");
    }

    #[test]
    fn test_atomic_write_creates_directory() {
        let temp_dir = tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested").join("dir").join("test.txt");

        let writer = AtomicWriter::new(&nested_path);
        writer.write_string("Nested file").unwrap();

        // 验证目录和文件都被创建
        assert!(nested_path.exists());
        assert_eq!(fs::read_to_string(&nested_path).unwrap(), "Nested file");
    }

    #[tokio::test]
    async fn test_async_atomic_write() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test_async.txt");

        let writer = AsyncAtomicWriter::new(&target_path);
        writer.write_async(b"Hello, Async!").await.unwrap();

        assert!(target_path.exists());
        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "Hello, Async!");
    }

    #[tokio::test]
    async fn test_async_atomic_write_string() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test_async.txt");

        let writer = AsyncAtomicWriter::new(&target_path);
        writer
            .write_string_async("Hello, Async String!")
            .await
            .unwrap();

        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "Hello, Async String!");
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_atomic_replace_failure_keeps_original_file() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test.txt");
        fs::write(&target_path, "original").unwrap();

        let mut attempts = 0usize;
        let error = retry_windows_atomic_replace(&target_path, || {
            attempts += 1;
            Err(std::io::Error::from_raw_os_error(32))
        })
        .expect_err("retry exhaustion should fail");

        assert!(is_windows_atomic_retry_error(&error));
        assert_eq!(attempts, ATOMIC_WRITE_RETRY_LIMIT);
        assert_eq!(fs::read_to_string(&target_path).unwrap(), "original");
    }
}
