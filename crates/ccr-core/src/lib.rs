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

#[cfg(test)]
pub(crate) mod test_support {
    use std::ffi::{OsStr, OsString};
    use std::sync::{LazyLock, Mutex, MutexGuard};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    pub(crate) struct TestLogEnv {
        previous_vars: Vec<(&'static str, Option<OsString>)>,
        _env_guard: MutexGuard<'static, ()>,
    }

    impl TestLogEnv {
        pub(crate) fn new() -> Self {
            let env_guard = ENV_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            Self {
                previous_vars: Vec::new(),
                _env_guard: env_guard,
            }
        }

        pub(crate) fn set_env(&mut self, key: &'static str, value: &OsStr) {
            set_env_var(&mut self.previous_vars, key, value);
        }

        pub(crate) fn remove_env(&mut self, key: &'static str) {
            self.previous_vars.push((key, std::env::var_os(key)));
            // SAFETY: TestLogEnv holds the process-wide ccr-core test env lock until Drop.
            unsafe {
                std::env::remove_var(key);
            }
        }
    }

    impl Drop for TestLogEnv {
        fn drop(&mut self) {
            self.restore_env_vars();
        }
    }

    impl TestLogEnv {
        fn restore_env_vars(&mut self) {
            for (key, previous) in self.previous_vars.drain(..).rev() {
                restore_env_var(key, previous);
            }
        }
    }

    fn set_env_var(
        previous_vars: &mut Vec<(&'static str, Option<OsString>)>,
        key: &'static str,
        value: &OsStr,
    ) {
        previous_vars.push((key, std::env::var_os(key)));
        // SAFETY: TestLogEnv holds the process-wide ccr-core test env lock until Drop.
        unsafe {
            std::env::set_var(key, value);
        }
    }

    fn restore_env_var(key: &str, previous: Option<OsString>) {
        // SAFETY: Drop runs while TestLogEnv still holds the process-wide test env lock.
        unsafe {
            match previous {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::TestLogEnv;
        use std::ffi::OsStr;

        #[test]
        fn test_log_env_sets_removes_and_restores_log_vars() {
            let mut env = TestLogEnv::new();
            let previous_ccr_log_level = std::env::var_os("CCR_LOG_LEVEL");
            let previous_rust_log = std::env::var_os("RUST_LOG");

            env.set_env("CCR_LOG_LEVEL", OsStr::new("debug"));
            env.remove_env("RUST_LOG");

            assert_eq!(
                std::env::var_os("CCR_LOG_LEVEL").as_deref(),
                Some(OsStr::new("debug"))
            );
            assert_eq!(std::env::var_os("RUST_LOG"), None);

            env.restore_env_vars();

            assert_eq!(std::env::var_os("CCR_LOG_LEVEL"), previous_ccr_log_level);
            assert_eq!(std::env::var_os("RUST_LOG"), previous_rust_log);
        }
    }
}
