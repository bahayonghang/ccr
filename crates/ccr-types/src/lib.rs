//! CCR Shared Types
//!
//! This crate provides shared type definitions used across the CCR ecosystem:
//! - `ccr` core library
//! - `ccr-ui/backend` web API
//!
//! ## Design Principles
//!
//! - All types implement `Serialize` and `Deserialize`
//! - All nested types preserve unknown fields via `#[serde(flatten)] other`
//! - Use `#[serde(default)]` for backward compatibility
//! - Use `#[serde(skip_serializing_if = "...")]` for clean JSON output
//! - `ClaudeSettings.output_style` serializes as `outputStyle` and accepts legacy `output_style`
//! - `hooks` serializes as the official grouped object format and accepts legacy arrays on input

mod claude_auth;
mod claude_settings;
mod codex_auth;
mod model_rate_catalog;
mod monitoring;

pub use claude_auth::{
    ClaudeAuthAccount, ClaudeAuthRegistry, ClaudeCurrentAuthInfo, ClaudeLoginState,
    ClaudeProfileAuthMode, ClaudeRuntimeMode, ClaudeRuntimeSummary,
};
pub use claude_settings::{
    Agent, ClaudeSettings, Hook, HookMatcherGroup, HooksConfig, McpServer, Plugin, SlashCommand,
    default_true, env_keys, is_false,
};
pub use codex_auth::LoginState;
pub use model_rate_catalog::{
    ModelRate, ModelRateCatalog, ModelRateOverride, PricingComputation, normalize_model_id,
    official_model_rate_override_for, official_model_rate_overrides,
};
pub use monitoring::{FrontendLogInput, MonitoringEntry, MonitoringFeedQuery, MonitoringLevel};
