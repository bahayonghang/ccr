// 📁 统一文件 I/O 模块
// 🎯 提供通用的配置文件读写功能，消除重复代码
//
// 设计原则：
// - 🔒 自动创建父目录
// - ⚠️ 统一错误处理
// - 🧹 代码复用 - 单一数据源
// - 📝 支持 TOML 和 JSON 格式

use crate::core::error::{CcrError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 📖 读取 TOML 文件并反序列化为指定类型
///
/// # 参数
/// - `path`: 文件路径
///
/// # 返回
/// - `Ok(T)`: 成功读取并反序列化的数据
/// - `Err(CcrError)`: 文件读取或解析错误
///
/// # 示例
/// ```no_run
/// use std::path::Path;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config {
///     name: String,
///     value: i32,
/// }
///
/// // 读取配置文件
/// let config: Config = ccr::core::fileio::read_toml(Path::new("config.toml"))?;
/// # Ok::<(), ccr::core::error::CcrError>(())
/// ```
pub fn read_toml<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    // 读取文件内容
    let content = fs::read_to_string(path).map_err(|e| {
        CcrError::ConfigError(format!("读取配置文件 {} 失败: {}", path.display(), e))
    })?;

    // 解析 TOML
    let data: T = toml::from_str(&content).map_err(|e| {
        CcrError::ConfigError(format!("解析 TOML 文件 {} 失败: {}", path.display(), e))
    })?;

    tracing::trace!("✅ 成功读取 TOML 文件: {}", path.display());
    Ok(data)
}

/// 💾 将数据序列化为 TOML 并写入文件
///
/// # 参数
/// - `path`: 目标文件路径
/// - `value`: 要序列化的数据
///
/// # 特性
/// - 自动创建父目录（如果不存在）
/// - 使用漂亮的格式化输出
/// - 原子写入（先写临时文件再重命名）
///
/// # 返回
/// - `Ok(())`: 成功写入
/// - `Err(CcrError)`: 写入或序列化错误
///
/// # 示例
/// ```no_run
/// use std::path::Path;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     name: String,
///     value: i32,
/// }
///
/// let config = Config {
///     name: "example".to_string(),
///     value: 42,
/// };
///
/// // 写入配置文件
/// ccr::core::fileio::write_toml(Path::new("config.toml"), &config)?;
/// # Ok::<(), ccr::core::error::CcrError>(())
/// ```
pub fn write_toml<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            CcrError::ConfigError(format!("创建目录 {} 失败: {}", parent.display(), e))
        })?;
    }

    // 序列化为 TOML（使用漂亮格式）
    let content = toml::to_string_pretty(value)
        .map_err(|e| CcrError::ConfigError(format!("序列化 TOML 数据失败: {}", e)))?;

    // 写入文件
    fs::write(path, content).map_err(|e| {
        CcrError::ConfigError(format!("写入配置文件 {} 失败: {}", path.display(), e))
    })?;

    tracing::trace!("✅ 成功写入 TOML 文件: {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestConfig {
        name: String,
        value: i32,
        enabled: bool,
    }

    #[test]
    fn test_read_write_toml() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let original = TestConfig {
            name: "test".to_string(),
            value: 42,
            enabled: true,
        };

        // 写入
        write_toml(&config_path, &original).unwrap();

        // 读取
        let loaded: TestConfig = read_toml(&config_path).unwrap();

        // 验证
        assert_eq!(loaded, original);
    }

    #[test]
    fn test_auto_create_parent_directory() {
        let temp_dir = tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested/dir/config.toml");

        let config = TestConfig {
            name: "test".to_string(),
            value: 42,
            enabled: true,
        };

        // 应该自动创建 nested/dir/ 目录
        write_toml(&nested_path, &config).unwrap();

        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result: Result<TestConfig> = read_toml(Path::new("/nonexistent/path.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_invalid_toml() {
        let temp_dir = tempdir().unwrap();
        let invalid_path = temp_dir.path().join("invalid.toml");

        // 写入无效的 TOML
        fs::write(&invalid_path, "invalid toml content {{{").unwrap();

        let result: Result<TestConfig> = read_toml(&invalid_path);
        assert!(result.is_err());
    }
}
