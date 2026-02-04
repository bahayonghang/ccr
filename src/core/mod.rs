// 🏗️ CCR 核心模块
// 提供通用的核心抽象和基础设施
//
// Core 层职责:
// - ⚠️ 错误类型定义和处理
// - 🔒 文件锁机制
// - 🎨 日志和彩色输出
// - 📝 原子文件写入
// - 📁 文件管理抽象
// - 💾 统一文件 I/O
// - 🗄️ 配置缓存

pub mod atomic_writer;
pub mod cache;
pub mod error;
pub mod file_manager;
pub mod fileio;
pub mod http;
pub mod lock;
pub mod logging;

/// GitHub 仓库标识（owner/repo）
pub const CCR_GITHUB_REPO: &str = "bahayonghang/ccr";

/// UI 仓库标识（owner/repo），与主仓库一致时复用
pub const CCR_UI_REPO: &str = "bahayonghang/ccr";

// 重新导出常用类型（供外部使用）
// 注意: 这些导出是为了库的公共 API，即使在模块内未使用也需要保留
#[allow(unused_imports)]
pub use atomic_writer::{AsyncAtomicWriter, AtomicWriter};
#[allow(unused_imports)]
pub use cache::{CacheStatus, ConfigCache};
#[allow(unused_imports)]
pub use error::{CcrError, Result};
#[allow(unused_imports)]
pub use file_manager::FileManager;
#[allow(unused_imports)]
pub use fileio::{
    read_json, read_json_async, read_toml, read_toml_async, write_json, write_json_async,
    write_toml, write_toml_async,
};
#[allow(unused_imports)]
pub use http::HTTP_CLIENT;
#[allow(unused_imports)]
pub use lock::{CONFIG_LOCK, FileLock, LockManager};
#[allow(unused_imports)]
pub use logging::{ColorOutput, init_file_only_logger, init_logger};
