//! Shared CCR configuration contracts and platform profile helpers.

pub mod managers;
pub mod models;
pub mod platforms;
pub mod services;

pub use ccr_core::{AutoCompletable, CcrError, Result, Validatable};
pub use managers::{
    CcsConfig, ConfigManager, ConfigSection, ConfigValidator, GlobalSettings, PlatformConfigEntry,
    PlatformConfigManager, ProviderType, TuiConfig, TuiConfigManager, TuiLanguage, TuiTabId,
    TuiTheme, UnifiedConfig, ValidationReport,
};
pub use models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
pub use platforms::base::{
    get_current_profile_from_registry, load_profiles_from_toml, profile_to_section,
    reconcile_registry_current_profile_after_delete, save_profiles_to_toml, section_to_profile,
    update_current_config, update_registry_current_profile,
};
pub use services::config_service::{
    ConfigInfo, ConfigList, ConfigService, ImportMode, ImportResult,
};

#[cfg(test)]
pub(crate) mod test_support {
    use std::ffi::{OsStr, OsString};
    use std::path::{Path, PathBuf};
    use std::sync::{LazyLock, Mutex, MutexGuard};
    use tempfile::TempDir;

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    pub(crate) struct TestCcrEnv {
        temp_dir: TempDir,
        root: PathBuf,
        lock_dir: PathBuf,
        previous_vars: Vec<(&'static str, Option<OsString>)>,
        _env_guard: MutexGuard<'static, ()>,
    }

    impl TestCcrEnv {
        pub(crate) fn new() -> Self {
            let env_guard = ENV_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let temp_dir = tempfile::tempdir().expect("test ccr env temp dir should be created");
            let root = temp_dir.path().join(".ccr");
            let lock_dir = temp_dir.path().join(".locks");

            for dir in [&root, &lock_dir] {
                std::fs::create_dir_all(dir).expect("test ccr env directory should be created");
            }

            let mut previous_vars = Vec::new();
            set_env_var(&mut previous_vars, "CCR_ROOT", root.as_os_str());
            set_env_var(&mut previous_vars, "CCR_LOCK_DIR", lock_dir.as_os_str());

            Self {
                temp_dir,
                root,
                lock_dir,
                previous_vars,
                _env_guard: env_guard,
            }
        }

        pub(crate) fn root(&self) -> &Path {
            &self.root
        }

        pub(crate) fn lock_dir(&self) -> &Path {
            &self.lock_dir
        }

        #[allow(dead_code)]
        pub(crate) fn home(&self) -> &Path {
            self.temp_dir.path()
        }
    }

    impl Drop for TestCcrEnv {
        fn drop(&mut self) {
            self.restore_env_vars();
        }
    }

    impl TestCcrEnv {
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
        // SAFETY: TestCcrEnv holds the process-wide ccr-config test env lock until Drop.
        unsafe {
            std::env::set_var(key, value);
        }
    }

    fn restore_env_var(key: &str, previous: Option<OsString>) {
        // SAFETY: Drop runs while TestCcrEnv still holds the process-wide test env lock.
        unsafe {
            match previous {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::TestCcrEnv;
        use std::ffi::OsString;

        #[test]
        fn test_ccr_env_sets_and_restores_root_and_lock_dir() {
            let mut env = TestCcrEnv::new();
            let previous_root = captured_previous(&env, "CCR_ROOT");
            let previous_lock_dir = captured_previous(&env, "CCR_LOCK_DIR");

            assert_eq!(
                std::env::var_os("CCR_ROOT").as_deref(),
                Some(env.root().as_os_str())
            );
            assert_eq!(
                std::env::var_os("CCR_LOCK_DIR").as_deref(),
                Some(env.lock_dir().as_os_str())
            );
            assert!(env.root().starts_with(env.home()));
            assert!(env.lock_dir().starts_with(env.home()));

            env.restore_env_vars();

            assert_eq!(std::env::var_os("CCR_ROOT"), previous_root);
            assert_eq!(std::env::var_os("CCR_LOCK_DIR"), previous_lock_dir);
        }

        fn captured_previous(env: &TestCcrEnv, key: &'static str) -> Option<OsString> {
            env.previous_vars
                .iter()
                .find_map(|(captured_key, value)| (*captured_key == key).then(|| value.clone()))
                .flatten()
        }
    }
}
