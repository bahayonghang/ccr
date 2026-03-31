// 📁 文件管理器 trait
// 为配置文件和设置文件提供统一的加载/保存接口

use crate::core::error::Result;
use std::path::Path;

/// 📁 文件管理器 trait
///
/// 为所有需要持久化的数据结构提供统一的文件操作接口
///
/// # Type Parameters
/// - `T`: 要持久化的数据类型
///
/// # Examples
///
/// ```rust,ignore
/// use ccr_core::core::FileManager;
/// use ccr_core::Result;
/// use std::path::{Path, PathBuf};
///
/// struct MyConfigManager {
///     path: PathBuf,
/// }
///
/// impl FileManager<MyConfig> for MyConfigManager {
///     fn load(&self) -> Result<MyConfig> {
///         // Load from file
///     }
///     
///     fn save(&self, data: &MyConfig) -> Result<()> {
///         // Save to file
///     }
///     
///     fn path(&self) -> &Path {
///         &self.path
///     }
/// }
/// ```
#[allow(dead_code)]
pub trait FileManager<T> {
    /// 📖 从文件加载数据
    ///
    /// # Returns
    /// - `Ok(T)` - 成功加载的数据
    /// - `Err(CcrError)` - 加载失败(文件不存在、格式错误等)
    fn load(&self) -> Result<T>;

    /// 💾 保存数据到文件
    ///
    /// # Arguments
    /// - `data` - 要保存的数据
    ///
    /// # Returns
    /// - `Ok(())` - 保存成功
    /// - `Err(CcrError)` - 保存失败(权限不足、磁盘满等)
    fn save(&self, data: &T) -> Result<()>;

    /// 📁 获取文件路径
    ///
    /// # Returns
    /// 文件的完整路径
    fn path(&self) -> &Path;
}
