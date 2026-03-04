//! CheckIn 数据模型。

pub mod account;
pub mod balance;
pub mod dashboard;
pub mod export;
pub mod provider;
pub mod record;

// ── account ──
pub use account::{
    AccountInfo, AccountsResponse, CheckinAccount, CookieCredentials, CreateAccountRequest,
    TestConnectionResponse, UpdateAccountRequest, mask_cookies_json,
};

// ── balance ──
pub use balance::{BalanceHistoryItem, BalanceHistoryResponse, BalanceResponse, BalanceSnapshot};

// ── dashboard ──
pub use dashboard::{
    CheckinAccountDashboardResponse, CheckinDashboardAccount, CheckinDashboardCalendar,
    CheckinDashboardDay, CheckinDashboardMonthStats, CheckinDashboardStreak, CheckinDashboardTrend,
    CheckinDashboardTrendPoint,
};

// ── export ──
pub use export::{
    EXPORT_VERSION, ExportAccount, ExportData, ExportOptions, ImportConflictStrategy,
    ImportOptions, ImportPreviewItem, ImportPreviewResponse, ImportResult,
};

// ── provider ──
pub use provider::{
    CheckinProvider, CreateProviderRequest, ProvidersResponse, UpdateProviderRequest,
};

// ── record ──
pub use record::{
    BatchCheckinResponse, BatchCheckinResult, CheckinRecord, CheckinRecordInfo,
    CheckinRecordsResponse, CheckinResponse, CheckinStatus, CheckinStatusOverview,
};
