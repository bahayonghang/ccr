// 📦 CCR 数据模型模块
// 定义跨模块共享的数据类型

pub mod budget;
pub mod codex_auth;
pub mod mcp_preset;
pub mod platform;
pub mod pricing;
pub mod prompt;
pub mod skill;
pub mod stats;
pub mod sync_folder;

// 重新导出常用类型
pub use codex_auth::{
    AuthIntent, AuthState, AuthStateStatus, CodexAccountQuota, CodexAuthAccount, CodexAuthExport,
    CodexAuthExportAccount, CodexAuthItem, CodexAuthJson, CodexAuthRegistry, CodexProfileAuthMode,
    CodexProfileSecret, CodexProfileSecretStore, CodexQuota, CredentialStoreKind, CurrentAuthInfo,
    ImportMode, ImportResult, LoginState, OpenAiAuthMethod, TokenFreshness,
    normalize_auth_map_for_intent,
};
// 测试时需要的类型
#[cfg(test)]
pub use codex_auth::CodexAuthTokens;
pub use platform::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
