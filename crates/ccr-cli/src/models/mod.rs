// re-export 墙条目须有真实消费方；规则见 .trellis/spec/ccr/backend/public-api-boundary.md，逐符号盘点见任务归档 07-03-arch-ccr-facade/research/inventory.md

pub mod budget;
pub mod mcp_preset;
pub mod pricing;
pub mod prompt;
pub mod skill;
pub mod skills;
pub mod stats;
pub mod sync_folder;

#[allow(unused_imports)]
pub use ccr_codex::models::codex_auth::CodexAuthTokens;
#[allow(unused_imports)]
pub use ccr_codex::models::codex_auth::{
    AuthIntent, AuthState, AuthStateStatus, CodexAccountQuota, CodexAuthAccount, CodexAuthItem,
    CodexAuthJson, CodexAuthRegistry, CodexProfileAuthMode, CodexQuota, CodexRuntimeMode,
    CodexRuntimeSummary, CodexUsageActivation, CredentialStoreKind, CurrentAuthInfo, ImportMode,
    ImportResult, LoginState, OpenAiAuthMethod,
};
pub use ccr_config::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
#[allow(unused_imports)]
pub use ccr_types::{
    ClaudeAuthAccount, ClaudeAuthActionOutcome, ClaudeAuthConfidence, ClaudeAuthDiagnosis,
    ClaudeAuthEvidence, ClaudeAuthOwnership, ClaudeAuthRegistry, ClaudeAuthSourceKind,
    ClaudeAuthSourceLocation, ClaudeAuthSourceObservation, ClaudeCurrentAuthInfo, ClaudeLoginState,
    ClaudeProfileAuthMode, ClaudeRuntimeMode, ClaudeRuntimeSummary,
};
