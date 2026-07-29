use crate::managers::SettingsManager;
use crate::models::Platform;
use ccr_config::ConfigManager;
use ccr_config::PlatformConfigManager;
use ccr_core::core::error::{CcrError, Result};
use std::path::{Path, PathBuf};

pub struct ProfileOffResult {
    pub platform: Platform,
    pub previous_profile: Option<String>,
    pub changed: bool,
}

/// `profile_off` 写盘事务的 RAII 守卫。
///
/// 构造时把所有要写入的文件快照到 `~/.ccr/backups/profile-off/{label}-{timestamp}/`。
/// 三步全部成功后由调用方 `commit()`；否则在 `Drop` 时按相反顺序回滚——已存在的
/// 文件用快照覆盖回去，原本不存在但被新建的文件直接删除。备份目录不在成功路径上
/// 删除，保留作 undo 历史，便于事后排查。
struct ProfileOffBackup {
    backup_dir: PathBuf,
    snapshots: Vec<FileSnapshot>,
    committed: bool,
}

enum FileSnapshot {
    /// 写盘前文件已存在；备份保存在 `backup`，回滚时把它复制回 `original`。
    Existing { original: PathBuf, backup: PathBuf },
    /// 写盘前文件不存在；回滚时若被新建则删除。
    Missing { original: PathBuf },
}

impl ProfileOffBackup {
    fn new(label: &str) -> Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
        let backup_dir = home
            .join(".ccr")
            .join("backups")
            .join("profile-off")
            .join(format!("{label}-{timestamp}"));
        std::fs::create_dir_all(&backup_dir).map_err(|error| {
            CcrError::ConfigError(format!(
                "创建 profile-off 备份目录失败 {}: {error}",
                backup_dir.display()
            ))
        })?;
        Ok(Self {
            backup_dir,
            snapshots: Vec::new(),
            committed: false,
        })
    }

    fn snapshot(&mut self, original: &Path) -> Result<()> {
        if original.exists() {
            let filename = original
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "snapshot".to_string());
            let backup = self
                .backup_dir
                .join(format!("{:02}-{filename}", self.snapshots.len()));
            std::fs::copy(original, &backup).map_err(|error| {
                CcrError::ConfigError(format!(
                    "快照 {} 到 {} 失败: {error}",
                    original.display(),
                    backup.display()
                ))
            })?;
            self.snapshots.push(FileSnapshot::Existing {
                original: original.to_path_buf(),
                backup,
            });
        } else {
            self.snapshots.push(FileSnapshot::Missing {
                original: original.to_path_buf(),
            });
        }
        Ok(())
    }

    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for ProfileOffBackup {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        for snapshot in self.snapshots.iter().rev() {
            match snapshot {
                FileSnapshot::Existing { original, backup } => {
                    if let Err(error) = std::fs::copy(backup, original) {
                        tracing::warn!(
                            target: "ccr::profile_off",
                            "回滚 {} 失败（备份 {}）: {error}",
                            original.display(),
                            backup.display()
                        );
                    }
                }
                FileSnapshot::Missing { original } => {
                    if original.exists()
                        && let Err(error) = std::fs::remove_file(original)
                    {
                        tracing::warn!(
                            target: "ccr::profile_off",
                            "删除新建文件 {} 失败: {error}",
                            original.display()
                        );
                    }
                }
            }
        }
    }
}

pub fn profile_off_for_platform(platform: Platform) -> Result<ProfileOffResult> {
    match platform {
        Platform::Codex => codex_profile_off(),
        Platform::Claude => claude_profile_off(),
        _ => Err(CcrError::PlatformNotSupported(format!(
            "{} 暂不支持 profile off",
            platform
        ))),
    }
}

fn claude_profile_off() -> Result<ProfileOffResult> {
    let previous_profile = platform_previous_profile_hint("claude")?;
    let had_profiles_file_pointer = platform_profiles_file_has_current_config("claude")?;
    let had_profile_routing = previous_profile.is_some() || had_profiles_file_pointer;

    if !had_profile_routing {
        // 没有要清的状态，直接返回；不创建 backup 目录。
        return Ok(ProfileOffResult {
            platform: Platform::Claude,
            previous_profile,
            changed: false,
        });
    }

    // 写盘前快照三处文件，任何 ? 失败都会触发 Drop 回滚。
    let settings_manager = SettingsManager::with_default()?;
    let platform_manager = PlatformConfigManager::with_default()?;
    let claude_config_manager = ConfigManager::for_platform("claude")?;
    let mut backup = ProfileOffBackup::new("claude")?;
    backup.snapshot(settings_manager.settings_path())?;
    backup.snapshot(platform_manager.config_path())?;
    backup.snapshot(claude_config_manager.config_path())?;

    let settings_cleared = clear_claude_profile_settings_overrides()?;
    clear_platform_registry_pointer("claude")?;
    clear_profiles_file_pointer("claude")?;
    let changed = had_profile_routing || settings_cleared;

    backup.commit();

    Ok(ProfileOffResult {
        platform: Platform::Claude,
        previous_profile,
        changed,
    })
}

fn codex_profile_off() -> Result<ProfileOffResult> {
    let codex_platform = ccr_codex::CodexPlatform::new()?;
    let previous_profile = platform_previous_profile_hint("codex")?;
    let changed = previous_profile.is_some()
        || codex_profiles_file_has_current_config()?
        || codex_platform.has_profile_entry_auth_backup();

    if !changed {
        return Ok(ProfileOffResult {
            platform: Platform::Codex,
            previous_profile,
            changed: false,
        });
    }

    // 写盘前快照两处 codex 受影响文件（codex_platform 内部 runtime 状态由其自身的
    // backup/lock 机制守护，这里只兜 ccr 侧的 profiles 文件）。
    let codex_config_manager = ConfigManager::for_platform("codex")?;
    let mut backup = ProfileOffBackup::new("codex")?;
    backup.snapshot(codex_config_manager.config_path())?;

    // Re-apply official runtime defaults without mutating auth.json.
    codex_platform.clear_active_profile_runtime()?;
    clear_profiles_file_pointer("codex")?;

    backup.commit();

    Ok(ProfileOffResult {
        platform: Platform::Codex,
        previous_profile,
        changed,
    })
}

fn clear_claude_profile_settings_overrides() -> Result<bool> {
    let manager = SettingsManager::with_default()?;
    let mut settings = match manager.load() {
        Ok(settings) => settings,
        Err(CcrError::SettingsMissing(_)) => return Ok(false),
        Err(error) => return Err(error),
    };

    if !settings.has_managed_overrides() {
        return Ok(false);
    }

    settings.clear_ccr_managed_vars();
    manager.save_atomic(&settings)?;
    Ok(true)
}

fn clear_platform_registry_pointer(platform_name: &str) -> Result<()> {
    let manager = PlatformConfigManager::with_default()?;
    let mut unified = manager.load_or_create_default()?;
    if let Ok(entry) = unified.get_platform_mut(platform_name) {
        entry.current_profile = None;
        entry.last_used = Some(chrono::Utc::now().to_rfc3339());
    }
    manager.save(&unified)
}

fn clear_profiles_file_pointer(platform_name: &str) -> Result<()> {
    let manager = ConfigManager::for_platform(platform_name)?;
    let mut config = manager.load_with_autofix()?;
    config.current_config.clear();
    manager.save(&config)
}

fn platform_previous_profile_hint(platform_name: &str) -> Result<Option<String>> {
    let manager = PlatformConfigManager::with_default()?;
    let unified = manager.load_or_create_default()?;
    Ok(unified
        .get_platform(platform_name)
        .ok()
        .and_then(|entry| entry.current_profile.clone()))
}

fn codex_profiles_file_has_current_config() -> Result<bool> {
    platform_profiles_file_has_current_config("codex")
}

fn platform_profiles_file_has_current_config(platform_name: &str) -> Result<bool> {
    let manager = ConfigManager::for_platform(platform_name)?;
    let config = manager.load_with_autofix()?;
    Ok(!config.current_config.trim().is_empty())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestHome;
    use ccr_types::{ClaudeSettings, env_keys};

    #[test]
    fn clear_claude_profile_settings_removes_managed_and_keeps_user_env() {
        let _home = TestHome::new();
        let manager = SettingsManager::with_default().unwrap();
        let mut settings = ClaudeSettings::new();
        for key in env_keys::CCR_MANAGED_KEYS {
            settings.env.insert((*key).to_string(), "managed".into());
        }
        settings
            .env
            .insert("ANTHROPIC_API_KEY".into(), "user-api-key".into());
        settings
            .env
            .insert("ANTHROPIC_CUSTOM_HEADERS".into(), "X-User: keep".into());
        manager.save_atomic(&settings).unwrap();

        assert!(clear_claude_profile_settings_overrides().unwrap());

        let settings = manager.load().unwrap();
        assert!(!settings.has_managed_overrides());
        assert_eq!(
            settings.env.get("ANTHROPIC_API_KEY").map(String::as_str),
            Some("user-api-key")
        );
        assert_eq!(
            settings
                .env
                .get("ANTHROPIC_CUSTOM_HEADERS")
                .map(String::as_str),
            Some("X-User: keep")
        );
    }
}
