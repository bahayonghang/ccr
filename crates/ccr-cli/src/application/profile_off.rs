use crate::managers::SettingsManager;
use crate::models::ClaudeAuthActionOutcome;
use crate::models::Platform;
use crate::platforms::{GrokActivationState, GrokPlatform};
use crate::services::ClaudeAuthService;
use ccr_config::ConfigManager;
use ccr_config::PlatformConfigManager;
use ccr_core::core::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use std::path::{Path, PathBuf};

pub struct ProfileOffResult {
    pub platform: Platform,
    pub previous_profile: Option<String>,
    pub changed: bool,
    pub runtime_mode: &'static str,
    pub auth_outcome: Option<ClaudeAuthActionOutcome>,
    pub warnings: Vec<String>,
}

impl ProfileOffResult {
    fn official(
        platform: Platform,
        previous_profile: Option<String>,
        changed: bool,
        auth_outcome: Option<ClaudeAuthActionOutcome>,
    ) -> Self {
        let warnings = auth_outcome
            .as_ref()
            .map(|outcome| outcome.warnings.clone())
            .unwrap_or_default();
        Self {
            platform,
            previous_profile,
            changed,
            runtime_mode: "official_auth",
            auth_outcome,
            warnings,
        }
    }

    fn grok_native(previous_profile: Option<String>, changed: bool) -> Self {
        Self {
            platform: Platform::Grok,
            previous_profile,
            changed,
            runtime_mode: "grok_native",
            auth_outcome: None,
            warnings: Vec::new(),
        }
    }
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
        let root = profile_off_backup_root()?;
        let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
        let backup_dir = root
            .join("backups")
            .join("profile-off")
            .join(format!("{label}-{timestamp}"));
        std::fs::create_dir_all(&backup_dir).map_err(|error| {
            CcrError::ConfigError(format!("创建 profile-off 备份目录失败: {error}"))
        })?;
        restrict_directory_to_owner(&backup_dir)?;
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
            write_secret_copy(original, &backup)?;
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
                    if let Err(error) = write_secret_copy(backup, original) {
                        tracing::warn!(
                            target: "ccr::profile_off",
                            "回滚失败: {error}"
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

fn profile_off_backup_root() -> Result<PathBuf> {
    if let Ok(custom_root) = std::env::var("CCR_ROOT") {
        let trimmed = custom_root.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }
    let home =
        dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
    Ok(home.join(".ccr"))
}

fn restrict_directory_to_owner(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path)
            .map_err(|error| CcrError::ConfigError(format!("读取备份目录权限失败: {error}")))?
            .permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(path, permissions)
            .map_err(|error| CcrError::ConfigError(format!("收紧备份目录权限失败: {error}")))?;
    }
    let _ = path;
    Ok(())
}

fn write_secret_copy(from: &Path, to: &Path) -> Result<()> {
    let content = std::fs::read(from)
        .map_err(|error| CcrError::ConfigError(format!("读取 profile-off 快照失败: {error}")))?;
    AtomicWriter::new(to).secret(true).write(&content)
}

/// Whether this platform has CCR profile-mode leftovers that would suppress official login.
pub fn needs_login_prep(platform: Platform) -> Result<bool> {
    match platform {
        Platform::Claude => claude_needs_login_prep(),
        Platform::Codex => codex_needs_login_prep(),
        Platform::Grok => grok_needs_login_prep(),
        _ => Ok(false),
    }
}

pub fn profile_off_for_platform(platform: Platform) -> Result<ProfileOffResult> {
    match platform {
        Platform::Codex => codex_profile_off(),
        Platform::Claude => claude_profile_off(),
        Platform::Grok => grok_profile_off(),
        _ => Err(CcrError::PlatformNotSupported(format!(
            "{} 暂不支持 profile off",
            platform
        ))),
    }
}

fn claude_needs_login_prep() -> Result<bool> {
    if platform_previous_profile_hint("claude")?.is_some() {
        return Ok(true);
    }
    if platform_profiles_file_has_current_config("claude")? {
        return Ok(true);
    }
    claude_has_managed_env()
}

fn claude_has_managed_env() -> Result<bool> {
    let manager = SettingsManager::with_default()?;
    match manager.load() {
        Ok(settings) => Ok(settings.has_managed_overrides()),
        Err(CcrError::SettingsMissing(_)) => Ok(false),
        Err(error) => Err(error),
    }
}

fn claude_profile_off() -> Result<ProfileOffResult> {
    let auth_service = ClaudeAuthService::new()?;
    let previous_profile = platform_previous_profile_hint("claude")?;
    let had_profiles_file_pointer = platform_profiles_file_has_current_config("claude")?;
    let had_profile_routing = previous_profile.is_some() || had_profiles_file_pointer;
    let had_managed_env = claude_has_managed_env()?;

    if !had_profile_routing && !had_managed_env {
        // 没有要清的状态，直接返回；不创建 backup 目录。
        return Ok(ProfileOffResult::official(
            Platform::Claude,
            previous_profile,
            false,
            Some(auth_service.action_outcome(Vec::new())),
        ));
    }

    // 写盘前快照三处文件，任何 ? 失败都会触发 Drop 回滚。
    let settings_manager = SettingsManager::with_default()?;
    let platform_manager = PlatformConfigManager::with_default()?;
    let claude_config_manager = ConfigManager::for_platform("claude")?;
    let mut backup = ProfileOffBackup::new("claude")?;
    backup.snapshot(settings_manager.settings_path())?;
    backup.snapshot(platform_manager.config_path())?;
    backup.snapshot(claude_config_manager.config_path())?;

    let cleared_managed_sources = clear_claude_profile_settings_overrides()?;
    clear_platform_registry_pointer("claude")?;
    clear_profiles_file_pointer("claude")?;
    let changed = had_profile_routing || !cleared_managed_sources.is_empty();

    backup.commit();

    Ok(ProfileOffResult::official(
        Platform::Claude,
        previous_profile,
        changed,
        Some(auth_service.action_outcome(cleared_managed_sources)),
    ))
}

fn codex_needs_login_prep() -> Result<bool> {
    let codex_platform = ccr_codex::CodexPlatform::new()?;
    Ok(codex_platform.has_raw_profile_pointer()?
        || codex_platform.has_profile_entry_auth_backup()
        || codex_platform.has_third_party_ccr_runtime()?)
}

fn codex_profile_off() -> Result<ProfileOffResult> {
    let codex_platform = ccr_codex::CodexPlatform::new()?;
    let previous_profile = platform_previous_profile_hint("codex")?;
    let changed = codex_needs_login_prep()?;

    if !changed {
        return Ok(ProfileOffResult::official(
            Platform::Codex,
            previous_profile,
            false,
            None,
        ));
    }

    // 写盘前快照 CCR 指针与 Codex runtime（含 auth.json），失败时 Drop 回滚。
    let codex_config_manager = ConfigManager::for_platform("codex")?;
    let runtime_manager = ccr_codex::CodexConfigManager::with_default()
        .map_err(|error| CcrError::ConfigError(format!("初始化 Codex 运行时配置失败: {error}")))?;
    let platform_manager = PlatformConfigManager::with_default()?;
    let mut backup = ProfileOffBackup::new("codex")?;
    backup.snapshot(codex_config_manager.config_path())?;
    backup.snapshot(platform_manager.config_path())?;
    backup.snapshot(runtime_manager.config_path())?;
    backup.snapshot(runtime_manager.auth_path())?;

    codex_platform.clear_active_profile_runtime()?;
    clear_profiles_file_pointer("codex")?;

    backup.commit();

    Ok(ProfileOffResult::official(
        Platform::Codex,
        previous_profile,
        true,
        None,
    ))
}

fn grok_needs_login_prep() -> Result<bool> {
    let platform = GrokPlatform::new()?;
    Ok(!matches!(
        platform.inspect_activation_state()?,
        GrokActivationState::Inactive
    ))
}

fn grok_profile_off() -> Result<ProfileOffResult> {
    let platform = GrokPlatform::new()?;
    let state = platform.inspect_activation_state()?;
    let previous_profile = match &state {
        GrokActivationState::Inactive => None,
        GrokActivationState::Active { name } | GrokActivationState::Drifted { name } => {
            Some(name.clone())
        }
        GrokActivationState::UnsafeMissingEntryState { name } => name.clone(),
    };

    if matches!(state, GrokActivationState::Inactive) {
        return Ok(ProfileOffResult::grok_native(previous_profile, false));
    }

    // Unsafe 会在 clear 内失败关闭；此处不猜测删除。
    platform.clear_active_profile_runtime()?;
    Ok(ProfileOffResult::grok_native(previous_profile, true))
}

fn clear_claude_profile_settings_overrides() -> Result<Vec<String>> {
    let manager = SettingsManager::with_default()?;
    let settings = match manager.load() {
        Ok(settings) => settings,
        Err(CcrError::SettingsMissing(_)) => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    if !settings.has_managed_overrides() {
        return Ok(Vec::new());
    }

    manager.update_atomic(|settings| {
        let cleared = settings
            .managed_env_entries()
            .into_iter()
            .map(|(key, _)| key)
            .collect();
        settings.clear_ccr_managed_vars();
        Ok(cleared)
    })
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
    if !manager.config_path().exists() {
        return Ok(None);
    }
    let unified = manager.load()?;
    Ok(unified
        .get_platform(platform_name)
        .ok()
        .and_then(|entry| entry.current_profile.clone()))
}

fn platform_profiles_path(platform_name: &str) -> Result<PathBuf> {
    let manager = PlatformConfigManager::with_default()?;
    let root = manager
        .config_path()
        .parent()
        .ok_or_else(|| CcrError::ConfigError("无法解析 CCR 根目录".into()))?;
    Ok(root
        .join("platforms")
        .join(platform_name)
        .join("profiles.toml"))
}

fn platform_profiles_file_has_current_config(platform_name: &str) -> Result<bool> {
    let path = platform_profiles_path(platform_name)?;
    if !path.exists() {
        return Ok(false);
    }
    let manager = ConfigManager::new(path);
    let config = manager.load()?;
    Ok(!config.current_config.trim().is_empty())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestHome;
    use ccr_config::PlatformConfig;
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

        assert!(
            !clear_claude_profile_settings_overrides()
                .unwrap()
                .is_empty()
        );

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

    #[test]
    fn clear_claude_profile_settings_is_noop_when_settings_are_missing() {
        let _home = TestHome::new();
        let manager = SettingsManager::with_default().unwrap();

        assert!(
            clear_claude_profile_settings_overrides()
                .unwrap()
                .is_empty()
        );
        assert!(!manager.settings_path().exists());
    }

    #[test]
    fn claude_profile_off_clears_managed_env_without_pointer() {
        let _home = TestHome::new();
        let manager = SettingsManager::with_default().unwrap();
        let mut settings = ClaudeSettings::new();
        settings.env.insert(
            env_keys::ANTHROPIC_AUTH_TOKEN.to_string(),
            "managed-token".into(),
        );
        settings
            .env
            .insert("ANTHROPIC_API_KEY".into(), "user-api-key".into());
        manager.save_atomic(&settings).unwrap();

        assert!(needs_login_prep(Platform::Claude).unwrap());
        let result = profile_off_for_platform(Platform::Claude).unwrap();
        assert!(result.changed);
        assert_eq!(result.runtime_mode, "official_auth");

        let settings = manager.load().unwrap();
        assert!(!settings.has_managed_overrides());
        assert_eq!(
            settings.env.get("ANTHROPIC_API_KEY").map(String::as_str),
            Some("user-api-key")
        );
        assert!(result.auth_outcome.as_ref().is_some_and(|outcome| {
            outcome
                .remaining_suppressors
                .iter()
                .any(|source| source.kind.as_str() == "anthropic_api_key")
        }));
    }

    #[test]
    fn claude_profile_off_is_unchanged_without_pointer_or_managed_env() {
        let _home = TestHome::new();
        assert!(!needs_login_prep(Platform::Claude).unwrap());
        let result = profile_off_for_platform(Platform::Claude).unwrap();
        assert!(!result.changed);
        assert_eq!(result.runtime_mode, "official_auth");
    }

    fn grok_platform_home() -> (TestHome, std::path::PathBuf) {
        let mut home = TestHome::new();
        let grok_home = home.home().join(".grok");
        std::fs::create_dir_all(&grok_home).unwrap();
        home.set_env("GROK_HOME", grok_home.as_os_str());
        (home, grok_home)
    }

    fn grok_relay_profile() -> crate::models::ProfileConfig {
        let mut profile = crate::models::ProfileConfig::new()
            .with_base_url("https://api.example.com/v1".into())
            .with_model("grok-4.5".into());
        profile
            .platform_data
            .insert("env_key".into(), serde_json::json!("EXAMPLE_GROK_API_KEY"));
        profile
    }

    #[test]
    fn grok_profile_off_clears_managed_route_when_entry_state_is_missing() {
        let (home, grok_home) = grok_platform_home();
        let platform = GrokPlatform::new().unwrap();
        platform
            .save_profile("relay", &grok_relay_profile())
            .unwrap();
        platform.apply_profile("relay").unwrap();

        let entry_state = home
            .root()
            .join("platforms")
            .join("grok")
            .join("profile_entry_config_state.json");
        std::fs::remove_file(entry_state).unwrap();

        let runtime = grok_home.join("config.toml");
        assert!(needs_login_prep(Platform::Grok).unwrap());

        let result = profile_off_for_platform(Platform::Grok).unwrap();
        assert!(result.changed);
        assert_eq!(result.previous_profile.as_deref(), Some("relay"));
        assert_eq!(result.runtime_mode, "grok_native");

        let raw = std::fs::read_to_string(runtime).unwrap();
        assert!(!raw.contains("EXAMPLE_GROK_API_KEY"));
        let config: toml::Value = toml::from_str(&raw).unwrap();
        assert!(config.get("model").is_none());
        assert!(config.get("models").is_none());
    }

    #[test]
    fn grok_profile_off_is_unchanged_when_inactive() {
        let (_home, _grok_home) = grok_platform_home();
        assert!(!needs_login_prep(Platform::Grok).unwrap());
        let result = profile_off_for_platform(Platform::Grok).unwrap();
        assert!(!result.changed);
        assert_eq!(result.runtime_mode, "grok_native");
        assert!(result.previous_profile.is_none());
    }

    #[test]
    fn profile_off_backup_uses_ccr_root_and_restores_secret_file() {
        let home = TestHome::new();
        let original = home.root().join("auth.json");
        std::fs::write(&original, r#"{"OPENAI_API_KEY":"sk-restore-me"}"#).unwrap();

        let backup_dir = {
            let mut backup = ProfileOffBackup::new("codex").unwrap();
            assert!(backup.backup_dir.starts_with(home.root()));
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = std::fs::metadata(&backup.backup_dir)
                    .unwrap()
                    .permissions()
                    .mode();
                assert_eq!(mode & 0o777, 0o700);
            }
            backup.snapshot(&original).unwrap();
            std::fs::write(&original, r#"{"OPENAI_API_KEY":"sk-replaced"}"#).unwrap();
            backup.backup_dir.clone()
        };

        let restored = std::fs::read_to_string(&original).unwrap();
        assert_eq!(restored, r#"{"OPENAI_API_KEY":"sk-restore-me"}"#);
        assert!(backup_dir.exists());
    }
}
