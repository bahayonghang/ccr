// Handlers module - exports all API handlers

pub mod command;
pub mod config;
pub mod system;
pub mod version;

// Re-export handler functions
pub use command::*;
pub use config::*;
pub use system::*;
pub use version::*;

