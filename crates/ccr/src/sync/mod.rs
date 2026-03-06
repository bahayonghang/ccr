#[cfg(feature = "web")]
pub mod commands;
#[cfg(feature = "web")]
pub mod content_selector;

pub mod config;
pub mod folder;
pub mod folder_manager;
pub mod service;

#[cfg(feature = "web")]
#[allow(unused_imports)]
pub use commands::*;

#[cfg(feature = "web")]
#[allow(unused_imports)]
pub use content_selector::{SyncContentSelection, SyncContentSelector, SyncContentType};

#[allow(unused_imports)]
pub use config::{SyncConfig, SyncConfigManager};
#[allow(unused_imports)]
pub use folder::{FolderStats, SyncFolder, SyncFoldersConfig, WebDavConfig};
#[allow(unused_imports)]
pub use folder_manager::SyncFolderManager;
#[allow(unused_imports)]
pub use service::{SyncService, get_ccr_sync_path};
