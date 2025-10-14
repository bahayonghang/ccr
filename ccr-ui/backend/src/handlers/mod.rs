// Handlers module - exports all API handlers

pub mod command;
pub mod config;
pub mod system;
pub mod version;
pub mod mcp;
pub mod slash_commands;
pub mod agents;
pub mod plugins;

// Re-export handler functions
pub use command::*;
pub use config::*;
pub use system::*;
pub use version::*;
pub use mcp::*;
pub use slash_commands::*;
pub use agents::*;
pub use plugins::*;

