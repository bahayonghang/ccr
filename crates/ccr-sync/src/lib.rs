pub mod sync;

pub use sync::{
    FolderStats, SyncContentSelection, SyncContentSelector, SyncContentType, SyncFolder,
    SyncFolderManager, SyncFoldersConfig, SyncLimits, SyncService, WebDavConfig, expand_path,
    get_ccr_sync_path, insecure_loopback_http_enabled, validate_webdav_url,
};
pub use sync::{SyncConfig, SyncConfigManager};

#[cfg(test)]
pub(crate) mod test_support {
    use std::ffi::{OsStr, OsString};
    use std::path::{Path, PathBuf};
    use std::sync::{LazyLock, Mutex, MutexGuard};
    use tempfile::TempDir;

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    pub(crate) struct TestSyncEnv {
        temp_dir: TempDir,
        root: PathBuf,
        platforms_dir: PathBuf,
        sync_folders_path: PathBuf,
        sync_config_path: PathBuf,
        previous_vars: Vec<(&'static str, Option<OsString>)>,
        _env_guard: MutexGuard<'static, ()>,
    }

    impl TestSyncEnv {
        pub(crate) fn new() -> Self {
            let env_guard = ENV_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let temp_dir = tempfile::tempdir().expect("test sync env temp dir should be created");
            let home = temp_dir.path();
            let root = home.join(".ccr");
            let platforms_dir = root.join("platforms");
            let sync_folders_path = home.join("sync_folders.toml");
            let sync_config_path = home.join("sync.toml");

            for dir in [&root, &platforms_dir] {
                std::fs::create_dir_all(dir).expect("test sync env directory should be created");
            }

            let mut previous_vars = Vec::new();
            set_env_var(&mut previous_vars, "CCR_ROOT", root.as_os_str());
            set_env_var(
                &mut previous_vars,
                "CCR_SYNC_FOLDERS_CONFIG",
                sync_folders_path.as_os_str(),
            );
            set_env_var(
                &mut previous_vars,
                "CCR_SYNC_CONFIG_PATH",
                sync_config_path.as_os_str(),
            );

            Self {
                temp_dir,
                root,
                platforms_dir,
                sync_folders_path,
                sync_config_path,
                previous_vars,
                _env_guard: env_guard,
            }
        }

        pub(crate) fn home(&self) -> &Path {
            self.temp_dir.path()
        }

        pub(crate) fn root(&self) -> &Path {
            &self.root
        }

        pub(crate) fn platforms_dir(&self) -> &Path {
            &self.platforms_dir
        }

        pub(crate) fn sync_folders_path(&self) -> &Path {
            &self.sync_folders_path
        }

        pub(crate) fn sync_config_path(&self) -> &Path {
            &self.sync_config_path
        }
    }

    impl Drop for TestSyncEnv {
        fn drop(&mut self) {
            self.restore_env_vars();
        }
    }

    impl TestSyncEnv {
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
        // SAFETY: TestSyncEnv holds the process-wide ccr-sync test env lock until Drop.
        unsafe {
            std::env::set_var(key, value);
        }
    }

    fn restore_env_var(key: &str, previous: Option<OsString>) {
        // SAFETY: Drop runs while TestSyncEnv still holds the process-wide test env lock.
        unsafe {
            match previous {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::TestSyncEnv;
        use std::ffi::OsString;

        #[test]
        fn test_sync_env_sets_and_restores_sync_env_vars() {
            let mut env = TestSyncEnv::new();
            let previous_root = captured_previous(&env, "CCR_ROOT");
            let previous_folders_config = captured_previous(&env, "CCR_SYNC_FOLDERS_CONFIG");
            let previous_sync_config = captured_previous(&env, "CCR_SYNC_CONFIG_PATH");

            assert_eq!(
                std::env::var_os("CCR_ROOT").as_deref(),
                Some(env.root().as_os_str())
            );
            assert_eq!(
                std::env::var_os("CCR_SYNC_FOLDERS_CONFIG").as_deref(),
                Some(env.sync_folders_path().as_os_str())
            );
            assert_eq!(
                std::env::var_os("CCR_SYNC_CONFIG_PATH").as_deref(),
                Some(env.sync_config_path().as_os_str())
            );
            assert!(env.root().starts_with(env.home()));
            assert!(env.platforms_dir().starts_with(env.root()));
            assert!(env.sync_folders_path().starts_with(env.home()));
            assert!(env.sync_config_path().starts_with(env.home()));

            env.restore_env_vars();

            assert_eq!(std::env::var_os("CCR_ROOT"), previous_root);
            assert_eq!(
                std::env::var_os("CCR_SYNC_FOLDERS_CONFIG"),
                previous_folders_config
            );
            assert_eq!(
                std::env::var_os("CCR_SYNC_CONFIG_PATH"),
                previous_sync_config
            );
        }

        fn captured_previous(env: &TestSyncEnv, key: &'static str) -> Option<OsString> {
            env.previous_vars
                .iter()
                .find_map(|(captured_key, value)| (*captured_key == key).then(|| value.clone()))
                .flatten()
        }
    }
}
