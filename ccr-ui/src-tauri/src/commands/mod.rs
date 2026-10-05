//! Tauri 命令聚合模块 —— 按功能分组组织各子模块。
//!
//! 每个子模块对应一个平台或功能域，并提供 `#[ccr_tauri_command_macros::command]` 命令实现。

pub mod agent_sessions;
pub mod builtin_prompts;
pub mod checkin;
pub mod claude;
pub mod claude_mcp_config;
pub mod claude_observer;
pub mod codex;
pub mod command_exec;
pub mod config;
pub mod converter;
pub mod environment;
pub mod gemini;
pub mod grok;
pub mod install;
pub mod mcp_presets;
pub mod opencode;
pub mod pricing;
pub mod profile_lifecycle;
pub mod settings_raw;
pub mod shell;
pub mod ssh;
pub mod sync;
pub mod system;
pub mod system_prompts;
pub mod ui_state;
pub mod unified_mcp;
pub mod usage;
pub mod waf;
#[cfg(target_os = "windows")]
pub mod wsl;

mod wire;

mod handler_registry;
pub(crate) mod runtime_policy;

pub use handler_registry::generate_handler;
