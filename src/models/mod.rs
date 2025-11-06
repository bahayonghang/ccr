// 📦 CCR 数据模型模块
// 定义跨模块共享的数据类型

pub mod platform;
pub mod stats;
pub mod sync_folder;

// 重新导出常用类型
pub use platform::{ConfigMode, Platform, PlatformConfig, PlatformPaths, ProfileConfig};
