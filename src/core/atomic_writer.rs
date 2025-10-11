// 📝 原子写入器
// 提供安全的原子文件写入功能,避免数据损坏

use crate::error::{CcrError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

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
/// use ccr::core::AtomicWriter;
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

#[allow(dead_code)]
impl AtomicWriter {
    /// 🏗️ 创建新的原子写入器
    ///
    /// # Arguments
    /// - `target_path` - 目标文件路径
    ///
    /// # Returns
    /// 新的 `AtomicWriter` 实例
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
                CcrError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("创建目录失败: {}", e),
                ))
            })?;
        }

        // 📄 在同一目录创建临时文件(确保在同一文件系统)
        let temp_file = if let Some(parent) = self.target_path.parent() {
            NamedTempFile::new_in(parent)
        } else {
            NamedTempFile::new()
        }
        .map_err(|e| {
            CcrError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("创建临时文件失败: {}", e),
            ))
        })?;

        // ✍️ 写入内容
        fs::write(temp_file.path(), content).map_err(|e| {
            CcrError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("写入临时文件失败: {}", e),
            ))
        })?;

        // 🔄 原子替换(这是关键操作)
        temp_file.persist(&self.target_path).map_err(|e| {
            CcrError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("原子替换文件失败: {}", e),
            ))
        })?;

        log::debug!("✅ 文件已原子写入: {:?}", self.target_path);
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

    /// 📁 获取目标文件路径
    pub fn target_path(&self) -> &Path {
        &self.target_path
    }
}

#[cfg(test)]
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
    fn test_atomic_write_creates_directory() {
        let temp_dir = tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested").join("dir").join("test.txt");

        let writer = AtomicWriter::new(&nested_path);
        writer.write_string("Nested file").unwrap();

        // 验证目录和文件都被创建
        assert!(nested_path.exists());
        assert_eq!(fs::read_to_string(&nested_path).unwrap(), "Nested file");
    }
}
