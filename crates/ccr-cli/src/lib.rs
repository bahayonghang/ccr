//! CCR CLI/application domain crate.

pub mod application;
pub mod cli;
pub mod commands;
pub mod managers;
pub mod models;
pub mod platforms;
pub mod services;
pub mod sync;

#[cfg(test)]
pub(crate) mod test_support {
    use std::ffi::{OsStr, OsString};
    use std::path::{Path, PathBuf};
    use std::sync::{LazyLock, Mutex, MutexGuard};
    use tempfile::TempDir;

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    pub(crate) fn env_lock() -> MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) struct TestHome {
        temp_dir: TempDir,
        root: PathBuf,
        settings_path: PathBuf,
        backup_dir: PathBuf,
        lock_dir: PathBuf,
        codex_dir: PathBuf,
        previous_vars: Vec<(&'static str, Option<OsString>)>,
        _env_guard: MutexGuard<'static, ()>,
    }

    impl TestHome {
        pub(crate) fn new() -> Self {
            let env_guard = env_lock();
            let temp_dir = tempfile::tempdir().expect("test home temp dir should be created");
            let home = temp_dir.path();
            let root = home.join(".ccr");
            let claude_dir = home.join(".claude");
            let settings_path = claude_dir.join("settings.json");
            let backup_dir = claude_dir.join("backups");
            let lock_dir = home.join(".locks");
            let codex_dir = home.join(".codex");

            for dir in [&root, &claude_dir, &backup_dir, &lock_dir, &codex_dir] {
                std::fs::create_dir_all(dir).expect("test home directory should be created");
            }

            let mut previous_vars = Vec::new();
            set_env_var(&mut previous_vars, "CCR_ROOT", root.as_os_str());
            set_env_var(&mut previous_vars, "CCR_LOCK_DIR", lock_dir.as_os_str());
            set_env_var(
                &mut previous_vars,
                "CCR_SETTINGS_PATH",
                settings_path.as_os_str(),
            );
            set_env_var(&mut previous_vars, "CCR_BACKUP_DIR", backup_dir.as_os_str());
            set_env_var(&mut previous_vars, "CCR_CODEX_DIR", codex_dir.as_os_str());
            remove_env_var(&mut previous_vars, "CCR_CONFIG_PATH");

            Self {
                temp_dir,
                root,
                settings_path,
                backup_dir,
                lock_dir,
                codex_dir,
                previous_vars,
                _env_guard: env_guard,
            }
        }

        pub(crate) fn new_with_home_env() -> Self {
            let mut home = Self::new();
            let home_path = home.home().as_os_str().to_owned();
            home.set_env("HOME", home_path.as_os_str());
            home.set_env("USERPROFILE", home_path.as_os_str());
            home
        }

        pub(crate) fn home(&self) -> &Path {
            self.temp_dir.path()
        }

        pub(crate) fn root(&self) -> &Path {
            &self.root
        }

        pub(crate) fn settings_path(&self) -> &Path {
            &self.settings_path
        }

        pub(crate) fn backup_dir(&self) -> &Path {
            &self.backup_dir
        }

        pub(crate) fn lock_dir(&self) -> &Path {
            &self.lock_dir
        }

        pub(crate) fn codex_dir(&self) -> &Path {
            &self.codex_dir
        }

        pub(crate) fn set_env(&mut self, key: &'static str, value: &OsStr) {
            set_env_var(&mut self.previous_vars, key, value);
        }
    }

    impl Drop for TestHome {
        fn drop(&mut self) {
            self.restore_env_vars();
        }
    }

    impl TestHome {
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
        // SAFETY: TestHome holds the process-wide test env lock until Drop restores this key.
        unsafe {
            std::env::set_var(key, value);
        }
    }

    fn remove_env_var(
        previous_vars: &mut Vec<(&'static str, Option<OsString>)>,
        key: &'static str,
    ) {
        previous_vars.push((key, std::env::var_os(key)));
        // SAFETY: TestHome holds the process-wide test env lock until Drop restores this key.
        unsafe {
            std::env::remove_var(key);
        }
    }

    fn restore_env_var(key: &str, previous: Option<OsString>) {
        // SAFETY: Drop runs while TestHome still holds the process-wide test env lock.
        unsafe {
            match previous {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::TestHome;
        use std::ffi::OsString;

        #[test]
        fn test_home_sets_and_restores_common_ccr_env() {
            let mut home = TestHome::new();
            let previous_root = captured_previous(&home, "CCR_ROOT");
            let previous_config_path = captured_previous(&home, "CCR_CONFIG_PATH");

            assert_eq!(
                std::env::var_os("CCR_ROOT").as_deref(),
                Some(home.root().as_os_str())
            );
            assert!(std::env::var_os("CCR_CONFIG_PATH").is_none());
            assert!(home.settings_path().starts_with(home.home()));
            assert!(home.backup_dir().starts_with(home.home()));
            assert!(home.lock_dir().starts_with(home.home()));
            assert!(home.codex_dir().starts_with(home.home()));

            home.restore_env_vars();

            assert_eq!(std::env::var_os("CCR_ROOT"), previous_root);
            assert_eq!(std::env::var_os("CCR_CONFIG_PATH"), previous_config_path);
        }

        #[test]
        fn test_home_can_scope_host_home_env() {
            let mut home = TestHome::new_with_home_env();
            let previous_home = captured_previous(&home, "HOME");
            let previous_userprofile = captured_previous(&home, "USERPROFILE");

            assert_eq!(
                std::env::var_os("HOME").as_deref(),
                Some(home.home().as_os_str())
            );
            assert_eq!(
                std::env::var_os("USERPROFILE").as_deref(),
                Some(home.home().as_os_str())
            );

            home.restore_env_vars();

            assert_eq!(std::env::var_os("HOME"), previous_home);
            assert_eq!(std::env::var_os("USERPROFILE"), previous_userprofile);
        }

        fn captured_previous(home: &TestHome, key: &'static str) -> Option<OsString> {
            home.previous_vars
                .iter()
                .find_map(|(captured_key, value)| (*captured_key == key).then(|| value.clone()))
                .flatten()
        }
    }
}
