//! 💾 CCR 存储模块
//!
//! 提供 SQLite 持久化层，用于 Session 索引和缓存。
//!
//! ## 模块结构
//!
//! - [`database`] - 数据库连接管理和迁移
//! - [`session_store`] - Session 存储层
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use ccr::storage::{Database, SessionStore};
//! use ccr::sessions::models::SessionFilter;
//!
//! let db = Database::init_default()?;
//! let store = SessionStore::new(&db);
//! let sessions = store.list(SessionFilter::default().with_limit(20))?;
//! # Ok::<(), ccr::CcrError>(())
//! ```

pub mod database;
pub mod session_store;

pub use database::Database;
pub use session_store::SessionStore;
