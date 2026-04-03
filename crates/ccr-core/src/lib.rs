//! Shared CCR infrastructure primitives and utilities.

pub mod core;
pub mod utils;

pub use core::{
    AsyncAtomicWriter, AtomicWriter, CCR_GITHUB_REPO, CCR_UI_REPO, CONFIG_LOCK, CacheStatus,
    CcrError, ColorOutput, ConfigCache, FileLock, FileManager, HTTP_CLIENT, LockManager, Result,
    init_file_only_logger, init_logger, read_json, read_json_async, read_toml, read_toml_async,
    write_json, write_json_async, write_toml, write_toml_async,
};
pub use utils::{
    AutoCompletable, Validatable, is_qwen_chat_file, mask_if_sensitive, mask_sensitive,
    qwen_project_dir_name_from_chat_path, qwen_projects_dir, qwen_runtime_base_dir,
    resolve_qwen_runtime_base_dir,
};
