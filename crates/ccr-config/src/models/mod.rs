//! Shared platform and profile contracts.

pub mod claude_runtime_paths;
pub mod platform;

pub use claude_runtime_paths::ClaudeRuntimePaths;
pub use platform::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
