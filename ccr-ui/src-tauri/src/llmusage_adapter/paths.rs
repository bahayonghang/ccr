use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub root_dir: PathBuf,
    pub db_path: PathBuf,
    pub bin_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub exports_dir: PathBuf,
    pub hook_cmd_path: PathBuf,
    pub hook_sh_path: PathBuf,
    pub lock_path: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self, String> {
        if let Some(root_dir) = env_root("LLMUSAGE_HOME") {
            return Ok(Self::from_root(root_dir));
        }
        Self::with_cli_home(None)
    }

    pub fn with_cli_home(home: Option<PathBuf>) -> Result<Self, String> {
        if let Some(root_dir) = home {
            return Ok(Self::from_root(root_dir));
        }
        let home_dir = resolve_home_dir().ok_or_else(|| "无法解析用户主目录".to_string())?;
        Ok(Self::from_root(home_dir.join(".llmusage")))
    }

    fn from_root(root_dir: PathBuf) -> Self {
        let bin_dir = root_dir.join("bin");
        let backups_dir = root_dir.join("backups");
        let exports_dir = root_dir.join("exports");
        Self {
            db_path: root_dir.join("llmusage.db"),
            hook_cmd_path: bin_dir.join("llmusage-hook.cmd"),
            hook_sh_path: bin_dir.join("llmusage-hook.sh"),
            lock_path: root_dir.join("worker.lock"),
            root_dir,
            bin_dir,
            backups_dir,
            exports_dir,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_root_for_test(root_dir: impl Into<PathBuf>) -> Self {
        Self::from_root(root_dir.into())
    }
}

pub fn discover_llmusage_paths() -> Result<AppPaths, String> {
    AppPaths::discover()
}

fn env_root(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn resolve_home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        env_root("USERPROFILE")
            .or_else(|| env_root("HOME"))
            .or_else(dirs::home_dir)
    }
    #[cfg(not(windows))]
    {
        env_root("HOME")
            .or_else(|| env_root("USERPROFILE"))
            .or_else(dirs::home_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ffi::OsString, sync::Mutex};
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn restore_env(key: &str, value: Option<OsString>) {
        unsafe {
            if let Some(value) = value {
                std::env::set_var(key, value);
            } else {
                std::env::remove_var(key);
            }
        }
    }

    #[test]
    fn discovers_llmusage_home_override() {
        let _guard = ENV_LOCK.lock().expect("env lock should be available");
        let temp = TempDir::new().expect("temp dir should be created");
        let saved = std::env::var_os("LLMUSAGE_HOME");
        unsafe { std::env::set_var("LLMUSAGE_HOME", temp.path()) };

        let paths = discover_llmusage_paths().expect("LLMUSAGE_HOME should be discovered");
        assert_eq!(paths.root_dir, temp.path());
        assert_eq!(paths.db_path, temp.path().join("llmusage.db"));

        restore_env("LLMUSAGE_HOME", saved);
    }

    #[test]
    fn default_discovery_uses_llmusage_root_not_legacy_ccr_root() {
        let _guard = ENV_LOCK.lock().expect("env lock should be available");
        let temp_home = TempDir::new().expect("temp home should be created");
        let saved_llmusage = std::env::var_os("LLMUSAGE_HOME");
        let saved_home = std::env::var_os("HOME");
        let saved_userprofile = std::env::var_os("USERPROFILE");
        unsafe {
            std::env::remove_var("LLMUSAGE_HOME");
            std::env::set_var("HOME", temp_home.path());
            std::env::set_var("USERPROFILE", temp_home.path());
        }

        let paths = discover_llmusage_paths().expect("default llmusage root should be discovered");
        assert_eq!(paths.root_dir, temp_home.path().join(".llmusage"));
        assert_ne!(
            paths.root_dir,
            temp_home.path().join(".ccr").join("llmusage")
        );

        restore_env("LLMUSAGE_HOME", saved_llmusage);
        restore_env("HOME", saved_home);
        restore_env("USERPROFILE", saved_userprofile);
    }
}
