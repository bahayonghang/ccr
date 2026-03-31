//! Dedicated crate for the CCR Codex domain.

pub mod managers;
pub mod models;
pub mod platforms;
pub mod services;

pub use ccr_config::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
pub use ccr_core::{CcrError, Result};
pub use managers::codex_config::{CachedCodexConfigManager, CodexConfigManager};
pub use models::codex_auth::*;
pub use platforms::CodexPlatform;
pub use services::{
    AuthReadSnapshot, CodexAuthCacheAction, CodexAuthService, CodexOAuthTokenService,
    CodexQuotaService, CodexRollingUsage, CodexRuntimeCommitPlan, CodexRuntimeService,
    CodexSessionDetail, CodexSessionExport, CodexSessionMessage, CodexSessionService,
    CodexSessionSummary, CodexUsageService, OAuthRepairOutcome,
};
