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

mod claude_settings;
mod codex_auth;

pub use claude_settings::{
    Agent, ClaudeSettings, Hook, McpServer, Plugin, SlashCommand, default_true, is_false,
};
pub use codex_auth::{LoginState, TokenFreshness};
