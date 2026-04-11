//! CCR CLI-side persistence and session indexing.

pub mod budget_manager;
pub mod cost_tracker;
pub mod history;
pub mod models;
pub mod pricing_manager;
pub mod services;
pub mod sessions;
pub mod storage;

pub use budget_manager::BudgetManager;
pub use cost_tracker::CostTracker;
pub use history::{
    EnvChange, HistoryEntry, HistoryManager, HistoryStats, OperationDetails, OperationResult,
    OperationType,
};
pub use models::{
    BudgetConfig, BudgetLimits, BudgetPeriod, BudgetStatus, BudgetWarning, Cost, CostRecord,
    CostStats, DailyCost, LimitAction, ModelPricing, PeriodCosts, PricingConfig, TimeRange,
    TokenStats, TokenUsage,
};
pub use pricing_manager::PricingManager;
pub use services::history_service::HistoryService;
pub use sessions::{Session, SessionEvent, SessionFilter, SessionIndexer, SessionSummary};
pub use storage::session_store::SessionStats;
pub use storage::{Database, SessionStore};
