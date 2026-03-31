//! 📚 CCR Session 管理模块
//!
//! 提供 AI CLI Session 的解析、索引和管理功能。
//!
//! ## 支持的平台
//!
//! - **Claude**: `~/.claude/projects/**/*.jsonl`
//! - **Codex**: `~/.codex/sessions/*.jsonl`
//! - **Gemini**: `~/.gemini/tmp/*`
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use ccr_store::SessionIndexer;
//!
//! // 索引所有 sessions
//! let indexer = SessionIndexer::new()?;
//! let stats = indexer.index_all()?;
//!
//! // 列出 sessions
//! let sessions = indexer.list(Default::default())?;
//! # Ok::<(), ccr_core::CcrError>(())
//! ```

pub mod indexer;
pub mod models;
pub mod parser;

pub use indexer::SessionIndexer;
#[allow(unused_imports)]
pub use models::{Session, SessionEvent, SessionFilter, SessionSummary};
