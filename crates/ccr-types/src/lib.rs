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
mod monitoring;

pub use claude_auth::{
    ClaudeAuthAccount, ClaudeAuthRegistry, ClaudeCurrentAuthInfo, ClaudeLoginState,
    ClaudeProfileAuthMode, ClaudeRuntimeMode, ClaudeRuntimeSummary,
};
pub use claude_settings::{
    Agent, ClaudeSettings, Hook, HookMatcherGroup, HooksConfig, McpServer, Plugin, SlashCommand,
    default_true, is_false,
};
pub use codex_auth::LoginState;
pub use monitoring::{FrontendLogInput, MonitoringEntry, MonitoringFeedQuery, MonitoringLevel};
