use ccr_core::{CcrError, Result};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Claude Code user-level runtime paths shared by CLI and desktop consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeRuntimePaths {
    pub config_dir: PathBuf,
    pub settings_file: PathBuf,
    pub credentials_file: PathBuf,
    pub state_file: PathBuf,
    pub backups_dir: PathBuf,
}

impl ClaudeRuntimePaths {
    /// Resolve paths from the current user home and process environment.
    pub fn from_env() -> Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        Ok(Self::resolve_with(home, |key| std::env::var_os(key)))
    }

    /// Pure resolver for callers and tests that provide their own environment.
    pub fn resolve_with<F>(home: impl AsRef<Path>, env_getter: F) -> Self
    where
        F: Fn(&str) -> Option<OsString>,
    {
        let home = home.as_ref();
        let configured_dir = env_path("CLAUDE_CONFIG_DIR", &env_getter);
        let config_dir = configured_dir
            .clone()
            .unwrap_or_else(|| home.join(".claude"));

        let settings_file = env_path("CCR_SETTINGS_PATH", &env_getter)
            .unwrap_or_else(|| config_dir.join("settings.json"));
        let credentials_file = config_dir.join(".credentials.json");
        let state_file = env_path("CLAUDE_JSON_PATH", &env_getter).unwrap_or_else(|| {
            if configured_dir.is_some() {
                config_dir.join(".claude.json")
            } else {
                home.join(".claude.json")
            }
        });
        let backups_dir =
            env_path("CCR_BACKUP_DIR", &env_getter).unwrap_or_else(|| config_dir.join("backups"));

        Self {
            config_dir,
            settings_file,
            credentials_file,
            state_file,
            backups_dir,
        }
    }
}

fn env_path<F>(key: &str, env_getter: &F) -> Option<PathBuf>
where
    F: Fn(&str) -> Option<OsString>,
{
    env_getter(key)
        .filter(|value| !value.is_empty())
        .map(|value| expand_env_path(value, env_getter))
}

#[cfg(windows)]
fn expand_env_path<F>(value: OsString, env_getter: &F) -> PathBuf
where
    F: Fn(&str) -> Option<OsString>,
{
    let Some(text) = value.to_str() else {
        return PathBuf::from(value);
    };

    let mut expanded = OsString::new();
    let mut cursor = 0;
    while let Some(open_offset) = text[cursor..].find('%') {
        let open = cursor + open_offset;
        expanded.push(&text[cursor..open]);

        let name_start = open + 1;
        let Some(close_offset) = text[name_start..].find('%') else {
            expanded.push(&text[open..]);
            return PathBuf::from(expanded);
        };
        let close = name_start + close_offset;
        let name = &text[name_start..close];
        if !name.is_empty()
            && let Some(replacement) = env_getter(name)
        {
            expanded.push(replacement);
        } else {
            expanded.push(&text[open..=close]);
        }
        cursor = close + 1;
    }
    expanded.push(&text[cursor..]);
    PathBuf::from(expanded)
}

#[cfg(not(windows))]
fn expand_env_path<F>(value: OsString, _env_getter: &F) -> PathBuf
where
    F: Fn(&str) -> Option<OsString>,
{
    PathBuf::from(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn resolve(home: &Path, vars: &[(&str, &str)]) -> ClaudeRuntimePaths {
        let vars = vars
            .iter()
            .map(|(key, value)| ((*key).to_string(), OsString::from(value)))
            .collect::<HashMap<_, _>>();
        ClaudeRuntimePaths::resolve_with(home, |key| vars.get(key).cloned())
    }

    #[test]
    fn claude_runtime_paths_default_to_the_legacy_home_layout() {
        let home = Path::new("test-home");
        let paths = resolve(home, &[]);

        assert_eq!(paths.config_dir, home.join(".claude"));
        assert_eq!(paths.settings_file, home.join(".claude/settings.json"));
        assert_eq!(
            paths.credentials_file,
            home.join(".claude/.credentials.json")
        );
        assert_eq!(paths.state_file, home.join(".claude.json"));
        assert_eq!(paths.backups_dir, home.join(".claude/backups"));
    }

    #[test]
    fn claude_config_dir_moves_all_config_scoped_runtime_paths() {
        let home = Path::new("test-home");
        let config_dir = Path::new("custom-claude");
        let paths = resolve(home, &[("CLAUDE_CONFIG_DIR", "custom-claude")]);

        assert_eq!(paths.config_dir, config_dir);
        assert_eq!(paths.settings_file, config_dir.join("settings.json"));
        assert_eq!(paths.credentials_file, config_dir.join(".credentials.json"));
        assert_eq!(paths.state_file, config_dir.join(".claude.json"));
        assert_eq!(paths.backups_dir, config_dir.join("backups"));
    }

    #[test]
    fn explicit_file_and_backup_overrides_take_priority() {
        let home = Path::new("test-home");
        let paths = resolve(
            home,
            &[
                ("CLAUDE_CONFIG_DIR", "custom-claude"),
                ("CCR_SETTINGS_PATH", "overrides/settings.json"),
                ("CLAUDE_JSON_PATH", "overrides/state.json"),
                ("CCR_BACKUP_DIR", "overrides/backups"),
            ],
        );

        assert_eq!(paths.config_dir, PathBuf::from("custom-claude"));
        assert_eq!(
            paths.settings_file,
            PathBuf::from("overrides/settings.json")
        );
        assert_eq!(
            paths.credentials_file,
            PathBuf::from("custom-claude/.credentials.json")
        );
        assert_eq!(paths.state_file, PathBuf::from("overrides/state.json"));
        assert_eq!(paths.backups_dir, PathBuf::from("overrides/backups"));
    }

    #[test]
    fn empty_overrides_fall_back_to_the_default_layout() {
        let home = Path::new("test-home");
        let paths = resolve(
            home,
            &[
                ("CLAUDE_CONFIG_DIR", ""),
                ("CCR_SETTINGS_PATH", ""),
                ("CLAUDE_JSON_PATH", ""),
                ("CCR_BACKUP_DIR", ""),
            ],
        );

        assert_eq!(paths.config_dir, home.join(".claude"));
        assert_eq!(paths.settings_file, home.join(".claude/settings.json"));
        assert_eq!(paths.state_file, home.join(".claude.json"));
        assert_eq!(paths.backups_dir, home.join(".claude/backups"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_paths_expand_percent_variables_and_preserve_path_syntax() {
        let paths = resolve(
            Path::new(r"C:\fallback"),
            &[
                ("USERPROFILE", r"D:\Users\Test User"),
                ("CLAUDE_CONFIG_DIR", r"%USERPROFILE%\Claude Data\runtime"),
            ],
        );

        let config_dir = PathBuf::from(r"D:\Users\Test User\Claude Data\runtime");
        assert_eq!(paths.config_dir, config_dir);
        assert_eq!(paths.settings_file, config_dir.join("settings.json"));
        assert_eq!(paths.state_file, config_dir.join(".claude.json"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_unknown_percent_variables_remain_literal() {
        let paths = resolve(
            Path::new(r"C:\fallback"),
            &[("CLAUDE_CONFIG_DIR", r"%CCR_UNKNOWN%\Claude")],
        );

        assert_eq!(paths.config_dir, PathBuf::from(r"%CCR_UNKNOWN%\Claude"));
    }

    #[cfg(not(windows))]
    #[test]
    fn non_windows_paths_do_not_expand_percent_variables() {
        let paths = resolve(
            Path::new("/home/test"),
            &[
                ("USERPROFILE", "/other/home"),
                ("CLAUDE_CONFIG_DIR", "%USERPROFILE%/.claude-alt"),
            ],
        );

        assert_eq!(paths.config_dir, PathBuf::from("%USERPROFILE%/.claude-alt"));
    }
}
