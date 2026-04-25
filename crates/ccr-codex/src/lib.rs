//! Dedicated crate for the CCR Codex domain.

pub mod managers;
pub mod models;
pub mod platforms;
pub mod services;
pub mod utils;

pub use ccr_config::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
pub use ccr_core::{CcrError, Result};
pub use managers::codex_config::{CachedCodexConfigManager, CodexConfigManager};
pub use models::codex_auth::*;
pub use models::codex_model_provider::*;
pub use models::opencode_auth::*;
pub use platforms::CodexPlatform;
pub use services::{
    AuthReadSnapshot, CodexAuthCacheAction, CodexAuthService, CodexHistoryBackupPruneResult,
    CodexHistoryBackupSummary, CodexHistoryProviderBuckets, CodexHistoryRestoreResult,
    CodexHistorySyncOptions, CodexHistorySyncResult, CodexHistorySyncService,
    CodexHistorySyncStatus, CodexModelProviderStoreService, CodexOAuthTokenService,
    CodexQuotaService, CodexRegistryStore, CodexRollingUsage, CodexRuntimeCommitPlan,
    CodexRuntimeService, CodexSessionDetail, CodexSessionExport, CodexSessionMessage,
    CodexSessionService, CodexSessionSummary, CodexUsageRecord, CodexUsageService, CodexUsageStats,
    OAuthRepairOutcome, OpenCodeAuthService, OpenCodeQuotaService, OpenCodeRollingUsage,
    OpenCodeUsageRecord, OpenCodeUsageService, OpenCodeUsageStats,
};
