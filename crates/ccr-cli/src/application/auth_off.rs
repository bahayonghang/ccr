//! Shared write core for `ccr {claude,codex,grok} auth off`.
//!
//! CLI / TUI / Tauri must call [`auth_off_for_platform`]. This module does not
//! call [`super::profile_off_for_platform`].

use crate::models::Platform;
use crate::services::install_detect::which_on_path;
use ccr_codex::{CodexPlatform, CredentialStoreKind};
use ccr_config::{ClaudeRuntimePaths, PlatformConfigManager};
use ccr_core::core::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

const NATIVE_LOGOUT_TIMEOUT: Duration = Duration::from_secs(15);

#[cfg(test)]
thread_local! {
    static FAIL_AFTER_DELETE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// How `auth off` cleared (or attempted to clear) the official session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthOffPath {
    File,
    NativeLogout,
}

impl AuthOffPath {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::NativeLogout => "native_logout",
        }
    }
}

/// Secret-free result of [`auth_off_for_platform`].
#[derive(Debug, Clone, Serialize)]
pub struct AuthOffResult {
    pub platform: Platform,
    pub changed: bool,
    pub path: AuthOffPath,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_pointer: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl AuthOffResult {
    fn file(
        platform: Platform,
        changed: bool,
        profile_pointer: Option<String>,
        warnings: Vec<String>,
    ) -> Self {
        Self {
            platform,
            changed,
            path: AuthOffPath::File,
            profile_pointer,
            warnings,
        }
    }

    fn native(platform: Platform, profile_pointer: Option<String>, warnings: Vec<String>) -> Self {
        Self {
            platform,
            changed: true,
            path: AuthOffPath::NativeLogout,
            profile_pointer,
            warnings,
        }
    }
}

/// `auth off` 写盘事务的 RAII 守卫。
///
/// 构造时把凭据文件快照到 `$CCR_ROOT/backups/auth-off/{label}-{timestamp}/`。
/// 删除成功后 `commit()` 会删除本次快照目录（D10）；未 commit 则 Drop 按相反
/// 顺序还原。`changed=false` 时不得创建该目录。
struct AuthOffBackup {
    backup_dir: PathBuf,
    snapshots: Vec<FileSnapshot>,
    committed: bool,
}

enum FileSnapshot {
    Existing { original: PathBuf, backup: PathBuf },
    Missing { original: PathBuf },
}

impl AuthOffBackup {
    fn new(label: &str) -> Result<Self> {
        let root = auth_off_backup_root()?;
        let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
        let backup_dir = root
            .join("backups")
            .join("auth-off")
            .join(format!("{label}-{timestamp}"));
        std::fs::create_dir_all(&backup_dir).map_err(|error| {
            CcrError::ConfigError(format!("创建 auth-off 备份目录失败: {error}"))
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
        if let Err(error) = std::fs::remove_dir_all(&self.backup_dir) {
            tracing::warn!(
                target: "ccr::auth_off",
                "删除 auth-off 快照目录失败: {error}"
            );
        }
    }
}

impl Drop for AuthOffBackup {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        for snapshot in self.snapshots.iter().rev() {
            match snapshot {
                FileSnapshot::Existing { original, backup } => {
                    if let Err(error) = write_secret_copy(backup, original) {
                        tracing::warn!(
                            target: "ccr::auth_off",
                            "回滚失败: {error}"
                        );
                    }
                }
                FileSnapshot::Missing { original } => {
                    if original.exists()
                        && let Err(error) = std::fs::remove_file(original)
                    {
                        tracing::warn!(
                            target: "ccr::auth_off",
                            "删除新建文件 {} 失败: {error}",
                            original.display()
                        );
                    }
                }
            }
        }
    }
}

fn auth_off_backup_root() -> Result<PathBuf> {
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
        .map_err(|error| CcrError::ConfigError(format!("读取 auth-off 快照失败: {error}")))?;
    AtomicWriter::new(to).secret(true).write(&content)
}

/// Whether the UI should offer official-session logout for this platform.
pub fn needs_auth_off(platform: Platform) -> Result<bool> {
    match platform {
        Platform::Claude => claude_needs_auth_off(),
        Platform::Codex => codex_needs_auth_off(),
        Platform::Grok => grok_auth_json_path().map(|path| path.exists()),
        _ => Ok(false),
    }
}

/// Log out the current official runtime session for `platform`.
///
/// File stores backup, delete, commit, then remove the snapshot directory.
/// Native stores spawn the official logout command. This function does not
/// call `profile_off_for_platform`.
pub fn auth_off_for_platform(platform: Platform) -> Result<AuthOffResult> {
    match platform {
        Platform::Claude => claude_auth_off(),
        Platform::Codex => codex_auth_off(),
        Platform::Grok => grok_auth_off(),
        _ => Err(CcrError::PlatformNotSupported(format!(
            "{} 暂不支持 auth off",
            platform
        ))),
    }
}

fn claude_needs_auth_off() -> Result<bool> {
    if cfg!(target_os = "macos") {
        return Ok(true);
    }
    let path = ClaudeRuntimePaths::from_env()?.credentials_file;
    claude_credentials_present(&path)
}

fn claude_credentials_present(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(true),
    };
    match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(serde_json::Value::Object(map)) => Ok(!map.is_empty()),
        Ok(_) => Ok(false),
        Err(_) => Ok(bytes.iter().any(|byte| !byte.is_ascii_whitespace())),
    }
}

fn claude_auth_off() -> Result<AuthOffResult> {
    if cfg!(target_os = "macos") {
        spawn_official_logout("claude", &["auth", "logout"])?;
        return Ok(AuthOffResult::native(Platform::Claude, None, Vec::new()));
    }

    let path = ClaudeRuntimePaths::from_env()?.credentials_file;
    delete_credential_files(Platform::Claude, "claude", std::slice::from_ref(&path))
}

fn detect_codex_credential_store() -> Result<CredentialStoreKind> {
    let dirs = CodexPlatform::login_prep_codex_dirs()?;
    let Some(dir) = dirs.first() else {
        return Ok(CredentialStoreKind::Auto);
    };
    let config_path = dir.join("config.toml");
    if !config_path.exists() {
        return Ok(CredentialStoreKind::Auto);
    }
    let content = match std::fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(_) => return Ok(CredentialStoreKind::Auto),
    };
    let parsed: toml::Value = match toml::from_str(&content) {
        Ok(value) => value,
        Err(_) => return Ok(CredentialStoreKind::Auto),
    };
    let store = parsed
        .as_table()
        .and_then(|table| table.get("cli_auth_credentials_store"))
        .and_then(|value| value.as_str());
    Ok(CredentialStoreKind::from_config_value(store))
}

fn codex_auth_json_paths() -> Result<Vec<PathBuf>> {
    Ok(CodexPlatform::login_prep_codex_dirs()?
        .into_iter()
        .map(|dir| dir.join("auth.json"))
        .collect())
}

fn codex_needs_auth_off() -> Result<bool> {
    match detect_codex_credential_store()? {
        CredentialStoreKind::File => Ok(codex_auth_json_paths()?.iter().any(|path| path.exists())),
        CredentialStoreKind::Keyring | CredentialStoreKind::Auto => Ok(true),
    }
}

fn codex_profile_pointer() -> Result<Option<String>> {
    platform_profile_pointer("codex")
}

fn codex_pointer_warning(pointer: Option<&str>) -> Vec<String> {
    match pointer {
        Some(name) if !name.trim().is_empty() => vec![format!(
            "仍存在 Codex profile 指针 '{name}'；运行时凭据已清除，需再次执行 profile switch 写回"
        )],
        _ => Vec::new(),
    }
}

fn codex_auth_off() -> Result<AuthOffResult> {
    let pointer = codex_profile_pointer()?;
    let warnings = codex_pointer_warning(pointer.as_deref());
    match detect_codex_credential_store()? {
        CredentialStoreKind::File => {
            let paths = codex_auth_json_paths()?;
            let mut result = delete_credential_files(Platform::Codex, "codex", &paths)?;
            result.profile_pointer = pointer;
            result.warnings = warnings;
            Ok(result)
        }
        CredentialStoreKind::Keyring | CredentialStoreKind::Auto => {
            spawn_official_logout("codex", &["logout"])?;
            Ok(AuthOffResult::native(Platform::Codex, pointer, warnings))
        }
    }
}

pub(crate) fn grok_auth_json_path() -> Result<PathBuf> {
    let grok_home = std::env::var_os("GROK_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".grok")))
        .ok_or_else(|| CcrError::ConfigError("无法获取 Grok 配置目录".into()))?;
    Ok(grok_home.join("auth.json"))
}

fn grok_auth_off() -> Result<AuthOffResult> {
    let path = grok_auth_json_path()?;
    delete_credential_files(Platform::Grok, "grok", std::slice::from_ref(&path))
}

fn delete_credential_files(
    platform: Platform,
    label: &str,
    paths: &[PathBuf],
) -> Result<AuthOffResult> {
    let existing: Vec<&PathBuf> = paths.iter().filter(|path| path.exists()).collect();
    if existing.is_empty() {
        return Ok(AuthOffResult::file(platform, false, None, Vec::new()));
    }

    let mut backup = AuthOffBackup::new(label)?;
    for path in &existing {
        backup.snapshot(path)?;
    }
    for path in &existing {
        std::fs::remove_file(path)
            .map_err(|error| CcrError::ConfigError(format!("删除官方凭据文件失败: {error}")))?;
    }

    #[cfg(test)]
    if take_fail_after_delete() {
        return Err(CcrError::ConfigError(
            "测试注入：凭据删除后提交前失败".into(),
        ));
    }

    backup.commit();
    Ok(AuthOffResult::file(platform, true, None, Vec::new()))
}

#[cfg(test)]
fn take_fail_after_delete() -> bool {
    FAIL_AFTER_DELETE.with(|flag| flag.replace(false))
}

fn platform_profile_pointer(platform_name: &str) -> Result<Option<String>> {
    let manager = PlatformConfigManager::with_default()?;
    if !manager.config_path().exists() {
        return Ok(None);
    }
    let unified = manager.load()?;
    Ok(unified
        .get_platform(platform_name)
        .ok()
        .and_then(|entry| entry.current_profile.clone())
        .filter(|name| !name.trim().is_empty()))
}

fn spawn_official_logout(program: &str, args: &[&str]) -> Result<()> {
    let bin = which_on_path(program).ok_or_else(|| {
        CcrError::ExternalCommandError(format!("未找到 {program}，无法执行官方 logout"))
    })?;

    let mut child = std::process::Command::new(&bin)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| {
            CcrError::ExternalCommandError(format!("无法启动 {program} logout: {error}"))
        })?;

    let deadline = Instant::now() + NATIVE_LOGOUT_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => {
                return Err(CcrError::ExternalCommandError(format!(
                    "{program} logout 退出码非 0 ({status})"
                )));
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CcrError::ExternalCommandError(format!(
                    "{program} logout 超时"
                )));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(error) => {
                return Err(CcrError::ExternalCommandError(format!(
                    "等待 {program} logout 失败: {error}"
                )));
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestHome;
    use std::ffi::OsString;
    use std::fs;

    #[cfg(test)]
    fn set_fail_after_delete() {
        FAIL_AFTER_DELETE.with(|flag| flag.set(true));
    }

    fn write_json(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    fn write_fake_logout_bin(dir: &Path, name: &str, marker: &Path, extra: &str) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        #[cfg(windows)]
        {
            let bin = dir.join(format!("{name}.cmd"));
            let script = format!(
                "@echo off\r\n>>{marker} echo spawned\r\n{extra}exit /b 0\r\n",
                marker = marker.display(),
            );
            fs::write(&bin, script).unwrap();
            bin
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let bin = dir.join(name);
            let script = format!(
                "#!/bin/sh\nprintf 'spawned\\n' >> '{marker}'\n{extra}exit 0\n",
                marker = marker.display(),
            );
            fs::write(&bin, script).unwrap();
            let mut permissions = fs::metadata(&bin).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&bin, permissions).unwrap();
            bin
        }
    }

    fn isolated_path(dir: &Path) -> OsString {
        dir.as_os_str().to_os_string()
    }

    fn auth_off_dirs(home: &TestHome) -> PathBuf {
        home.root().join("backups").join("auth-off")
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn claude_file_off_is_idempotent_and_skips_backup_when_unchanged() {
        let mut home = TestHome::new_with_home_env();
        let bin_dir = home.home().join("bin");
        let marker = home.home().join("spawned.txt");
        write_fake_logout_bin(&bin_dir, "claude", &marker, "");
        home.set_env("PATH", &isolated_path(&bin_dir));

        let credentials = home.claude_dir().join(".credentials.json");
        write_json(
            &credentials,
            r#"{"claudeAiOauth":{"accessToken":"secret"}}"#,
        );

        assert!(needs_auth_off(Platform::Claude).unwrap());
        let first = auth_off_for_platform(Platform::Claude).unwrap();
        assert!(first.changed);
        assert_eq!(first.path, AuthOffPath::File);
        assert!(!credentials.exists());
        assert!(!marker.exists());
        assert!(
            !auth_off_dirs(&home).exists()
                || auth_off_dirs(&home).read_dir().unwrap().next().is_none()
        );

        assert!(!needs_auth_off(Platform::Claude).unwrap());
        let second = auth_off_for_platform(Platform::Claude).unwrap();
        assert!(!second.changed);
        assert_eq!(second.path, AuthOffPath::File);
        assert!(!marker.exists());
        assert!(
            !auth_off_dirs(&home).exists()
                || auth_off_dirs(&home).read_dir().unwrap().next().is_none()
        );
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn claude_file_off_does_not_change_onboarding_state() {
        let home = TestHome::new_with_home_env();
        let credentials = home.claude_dir().join(".credentials.json");
        write_json(
            &credentials,
            r#"{"claudeAiOauth":{"accessToken":"secret"}}"#,
        );
        let state = home.claude_json_path().to_path_buf();
        write_json(
            &state,
            r#"{"hasCompletedOnboarding":true,"primaryApiKey":"keep-me"}"#,
        );
        let before = fs::read(&state).unwrap();

        auth_off_for_platform(Platform::Claude).unwrap();
        assert_eq!(fs::read(&state).unwrap(), before);
        assert!(!credentials.exists());
    }

    #[test]
    fn grok_off_deletes_auth_json_and_leaves_mcp_and_config() {
        let mut home = TestHome::new_with_home_env();
        let grok_home = home.home().join(".grok");
        fs::create_dir_all(&grok_home).unwrap();
        home.set_env("GROK_HOME", grok_home.as_os_str());

        let auth = grok_home.join("auth.json");
        let mcp = grok_home.join("mcp_credentials.json");
        let config = grok_home.join("config.toml");
        write_json(&auth, r#"{"token":"session-secret"}"#);
        write_json(&mcp, r#"{"mcp":"keep"}"#);
        fs::write(&config, "[model.custom]\napi_key = \"relay\"\n").unwrap();
        let mcp_before = fs::read(&mcp).unwrap();
        let config_before = fs::read(&config).unwrap();

        assert!(needs_auth_off(Platform::Grok).unwrap());
        let result = auth_off_for_platform(Platform::Grok).unwrap();
        assert!(result.changed);
        assert_eq!(result.path, AuthOffPath::File);
        assert!(!auth.exists());
        assert_eq!(fs::read(&mcp).unwrap(), mcp_before);
        assert_eq!(fs::read(&config).unwrap(), config_before);

        let second = auth_off_for_platform(Platform::Grok).unwrap();
        assert!(!second.changed);
    }

    #[test]
    fn file_off_rolls_back_when_commit_fails_after_delete() {
        let mut home = TestHome::new_with_home_env();
        let grok_home = home.home().join(".grok");
        fs::create_dir_all(&grok_home).unwrap();
        home.set_env("GROK_HOME", grok_home.as_os_str());
        let auth = grok_home.join("auth.json");
        write_json(&auth, r#"{"token":"restore-me"}"#);

        set_fail_after_delete();
        let error = auth_off_for_platform(Platform::Grok).unwrap_err();
        assert!(error.to_string().contains("测试注入"));
        assert_eq!(
            fs::read_to_string(&auth).unwrap(),
            r#"{"token":"restore-me"}"#
        );
    }

    #[test]
    fn auth_off_backup_uses_ccr_root_and_restores_secret_file() {
        let home = TestHome::new();
        let original = home.root().join("auth.json");
        fs::write(&original, r#"{"OPENAI_API_KEY":"sk-restore-me"}"#).unwrap();

        let backup_dir = {
            let mut backup = AuthOffBackup::new("codex").unwrap();
            assert!(backup.backup_dir.starts_with(home.root()));
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = fs::metadata(&backup.backup_dir)
                    .unwrap()
                    .permissions()
                    .mode();
                assert_eq!(mode & 0o777, 0o700);
            }
            backup.snapshot(&original).unwrap();
            fs::write(&original, r#"{"OPENAI_API_KEY":"sk-replaced"}"#).unwrap();
            backup.backup_dir.clone()
        };

        assert_eq!(
            fs::read_to_string(&original).unwrap(),
            r#"{"OPENAI_API_KEY":"sk-restore-me"}"#
        );
        assert!(backup_dir.exists());
    }

    #[test]
    fn codex_file_needs_auth_off_covers_login_prep_second_dir() {
        let mut home = TestHome::new_with_home_env();
        home.remove_env("CCR_CODEX_DIR");
        let sandbox = home.home().join("sandbox-codex");
        fs::create_dir_all(&sandbox).unwrap();
        home.set_env("CODEX_HOME", sandbox.as_os_str());
        fs::write(
            sandbox.join("config.toml"),
            "cli_auth_credentials_store = \"file\"\n",
        )
        .unwrap();

        let default_dir = home.home().join(".codex");
        fs::create_dir_all(&default_dir).unwrap();
        write_json(
            &default_dir.join("auth.json"),
            r#"{"OPENAI_API_KEY":"sk-stale-default-home"}"#,
        );

        assert!(needs_auth_off(Platform::Codex).unwrap());
        let result = auth_off_for_platform(Platform::Codex).unwrap();
        assert!(result.changed);
        assert_eq!(result.path, AuthOffPath::File);
        assert!(!default_dir.join("auth.json").exists());
        assert!(!sandbox.join("auth.json").exists());
    }

    #[test]
    fn codex_file_off_keeps_profile_pointer_and_reports_warning() {
        let home = TestHome::new_with_home_env();
        let auth = home.codex_dir().join("auth.json");
        write_json(&auth, r#"{"OPENAI_API_KEY":"sk-runtime"}"#);
        fs::write(
            home.codex_dir().join("config.toml"),
            "cli_auth_credentials_store = \"file\"\nmodel_provider = \"custom\"\n",
        )
        .unwrap();

        let manager = PlatformConfigManager::with_default().unwrap();
        let mut unified = manager.load_or_create_default().unwrap();
        unified.platforms.insert(
            "codex".into(),
            ccr_config::PlatformConfigEntry {
                enabled: true,
                current_profile: Some("relay".into()),
                description: None,
                last_used: None,
            },
        );
        manager.save(&unified).unwrap();

        let result = auth_off_for_platform(Platform::Codex).unwrap();
        assert!(result.changed);
        assert_eq!(result.profile_pointer.as_deref(), Some("relay"));
        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.contains("relay"))
        );

        let unified = manager.load().unwrap();
        assert_eq!(
            unified
                .get_platform("codex")
                .unwrap()
                .current_profile
                .as_deref(),
            Some("relay")
        );
        assert!(!auth.exists());
    }

    #[test]
    fn native_logout_missing_binary_is_error() {
        let mut home = TestHome::new_with_home_env();
        let empty = home.home().join("empty-bin");
        fs::create_dir_all(&empty).unwrap();
        home.set_env("PATH", &isolated_path(&empty));
        fs::write(
            home.codex_dir().join("config.toml"),
            "cli_auth_credentials_store = \"keyring\"\n",
        )
        .unwrap();

        let error = auth_off_for_platform(Platform::Codex).unwrap_err();
        assert!(error.to_string().contains("未找到 codex"));
    }

    #[test]
    fn native_logout_success_sets_changed_and_may_repeat() {
        let mut home = TestHome::new_with_home_env();
        let bin_dir = home.home().join("bin");
        let marker = home.home().join("codex-spawned.txt");
        write_fake_logout_bin(&bin_dir, "codex", &marker, "");
        home.set_env("PATH", &isolated_path(&bin_dir));
        fs::write(
            home.codex_dir().join("config.toml"),
            "cli_auth_credentials_store = \"keyring\"\n",
        )
        .unwrap();

        let first = auth_off_for_platform(Platform::Codex).unwrap();
        assert!(first.changed);
        assert_eq!(first.path, AuthOffPath::NativeLogout);
        assert!(marker.exists());

        let second = auth_off_for_platform(Platform::Codex).unwrap();
        assert!(second.changed);
        assert_eq!(second.path, AuthOffPath::NativeLogout);
    }

    #[test]
    fn file_off_does_not_spawn_when_unchanged() {
        let mut home = TestHome::new_with_home_env();
        let bin_dir = home.home().join("bin");
        let marker = home.home().join("codex-spawned.txt");
        write_fake_logout_bin(&bin_dir, "codex", &marker, "");
        home.set_env("PATH", &isolated_path(&bin_dir));
        fs::write(
            home.codex_dir().join("config.toml"),
            "cli_auth_credentials_store = \"file\"\n",
        )
        .unwrap();

        let result = auth_off_for_platform(Platform::Codex).unwrap();
        assert!(!result.changed);
        assert_eq!(result.path, AuthOffPath::File);
        assert!(!marker.exists());
    }
}
