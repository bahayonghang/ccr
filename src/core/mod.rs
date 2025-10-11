// 🏗️ CCR 核心模块
// 提供通用的核心抽象和基础设施
//
// Core 层职责:
// - ⚠️ 错误类型定义和处理
// - 🔒 文件锁机制
// - 🎨 日志和彩色输出
// - 📝 原子文件写入
// - 📁 文件管理抽象

pub mod atomic_writer;
pub mod error;
pub mod file_manager;
pub mod lock;
pub mod logging;

// 重新导出常用类型（供外部使用）
// 注意: 这些导出是为了库的公共 API，即使在模块内未使用也需要保留
#[allow(unused_imports)]
pub use atomic_writer::AtomicWriter;
#[allow(unused_imports)]
pub use error::{CcrError, Result};
#[allow(unused_imports)]
pub use file_manager::FileManager;
#[allow(unused_imports)]
pub use lock::{FileLock, LockManager};
pub use logging::{init_logger, ColorOutput};
