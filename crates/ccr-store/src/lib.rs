//! CCR CLI-side persistence and session indexing.

pub mod history;
pub mod sessions;
pub mod storage;

pub use history::{
    EnvChange, HistoryEntry, HistoryManager, HistoryStats, OperationDetails, OperationResult,
    OperationType,
};
pub use sessions::{Session, SessionEvent, SessionFilter, SessionIndexer, SessionSummary};
pub use storage::session_store::SessionStats;
pub use storage::{Database, SessionStore};
