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
pub mod guarded_write;
pub mod http;
pub mod lock;
pub mod log_bridge;
pub mod log_redact;
pub mod log_writer;
pub mod logging;
pub mod process_gateway;
pub mod secret;
pub mod sqlite;

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
    write_json_opts, write_json_opts_async, write_toml, write_toml_async, write_toml_opts,
    write_toml_opts_async,
};
#[allow(unused_imports)]
pub use guarded_write::{
    BACKUP_KEEP, BackupPolicy, VersionedWriteOutcome, WriteOptions, backup_guarded,
    content_version_token, write_guarded, write_guarded_async, write_guarded_versioned,
    write_guarded_versioned_async,
};
#[allow(unused_imports)]
pub use http::HTTP_CLIENT;
#[allow(unused_imports)]
pub use lock::{CONFIG_LOCK, FileLock, LockManager};
#[allow(unused_imports)]
pub use log_bridge::{
    BridgedLogEvent, EnqueueResult, close_bridged_log_sender, current_log_correlation_id,
    dropped_bridged_log_count, enter_bridge_consumer, take_bridged_log_receiver,
    try_enqueue_bridged_log,
};
pub use log_redact::{is_sensitive_log_key, normalize_log_key, redact_log_text, redact_log_value};
pub use logging::{ColorOutput, init_file_only_logger, init_logger};
#[allow(unused_imports)]
pub use secret::{Secret, expose_plaintext, expose_plaintext_option};
#[allow(unused_imports)]
pub use sqlite::{DbConnection, DbPool, PoolConfig, create_sqlite_pool};
