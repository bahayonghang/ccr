use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub root_dir: PathBuf,
    pub db_path: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self, String> {
        if let Some(root_dir) = env_root("LLMUSAGE_HOME") {
            return Ok(Self::from_root(root_dir));
        }
        let home_dir = resolve_home_dir().ok_or_else(|| "无法解析用户主目录".to_string())?;
        Ok(Self::from_root(home_dir.join(".llmusage")))
    }

    pub fn from_root(root_dir: impl Into<PathBuf>) -> Self {
        let root_dir = root_dir.into();
        Self {
            db_path: root_dir.join("llmusage.db"),
            root_dir,
        }
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
}
