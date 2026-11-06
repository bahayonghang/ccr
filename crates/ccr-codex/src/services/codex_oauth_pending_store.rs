//! Private, non-backed-up storage for an unfinished Codex OAuth login.

use crate::utils::CodexPaths;
use ccr_core::Secret;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::guarded_write::{self, BackupPolicy, WriteOptions};
use ccr_core::core::lock::{FileLock, LockManager};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Pending login data. Default serialization and Debug redact credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexOAuthPendingState {
    pub login_id: String,
    pub auth_url: Secret,
    pub redirect_uri: String,
    pub code_verifier: Secret,
    pub state: Secret,
    pub port: u16,
    pub expires_at: i64,
    pub callback_url: Option<Secret>,
}

// Only the store can serialize the plaintext disk representation.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PendingOnDisk<'a> {
    login_id: &'a str,
    #[serde(serialize_with = "ccr_core::expose_plaintext")]
    auth_url: &'a Secret,
    redirect_uri: &'a str,
    #[serde(serialize_with = "ccr_core::expose_plaintext")]
    code_verifier: &'a Secret,
    #[serde(serialize_with = "ccr_core::expose_plaintext")]
    state: &'a Secret,
    port: u16,
    expires_at: i64,
    #[serde(serialize_with = "ccr_core::expose_plaintext_option")]
    callback_url: &'a Option<Secret>,
}

/// Coordinates save, read/expiry cleanup, and cancellation of pending storage.
pub struct CodexOAuthPendingStore {
    path: PathBuf,
}

/// Opaque proof of one pending-file revision. Never serialize or log this proof.
#[derive(Clone)]
pub struct CodexOAuthPendingSnapshot {
    version: String,
    login_id: Option<String>,
    nonce: Option<String>,
}
impl CodexOAuthPendingSnapshot {
    pub fn login_id(&self) -> Option<&str> {
        self.login_id.as_deref()
    }
    pub fn same_login(&self, other: &Self) -> bool {
        self.login_id == other.login_id && self.nonce == other.nonce
    }
    fn absent() -> Self {
        Self {
            version: String::new(),
            login_id: None,
            nonce: None,
        }
    }
    fn of(bytes: &[u8], pending: &CodexOAuthPendingState) -> Self {
        Self {
            version: guarded_write::content_version_token(bytes),
            login_id: Some(pending.login_id.clone()),
            nonce: Some(guarded_write::content_version_token(
                pending.state.expose().as_bytes(),
            )),
        }
    }
}

impl CodexOAuthPendingStore {
    pub fn new() -> Result<Self> {
        Ok(Self::with_path(
            CodexPaths::resolve()?
                .ccr_codex_dir
                .join("oauth_pending.json"),
        ))
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    fn lock(&self) -> Result<FileLock> {
        let path =
            std::path::absolute(&self.path).map_err(|error| storage_error("解析路径", error))?;
        let identity =
            guarded_write::content_version_token(path.to_string_lossy().to_lowercase().as_bytes());
        LockManager::with_default_path()?.lock_resource(
            &format!("codex_oauth_pending_{identity}"),
            Duration::from_secs(10),
        )
    }

    /// Returns live data, or removes expired data without making a copy.
    pub fn load(&self, now: i64) -> Result<Option<CodexOAuthPendingState>> {
        self.load_owned(now).map(|(pending, _)| pending)
    }

    pub fn load_owned(
        &self,
        now: i64,
    ) -> Result<(Option<CodexOAuthPendingState>, CodexOAuthPendingSnapshot)> {
        let _lock = self.lock()?;
        let (pending, snapshot) = self.read_locked()?;
        if pending
            .as_ref()
            .is_some_and(|pending| pending.expires_at <= now)
        {
            self.expire_locked(&snapshot)?;
            return Ok((None, CodexOAuthPendingSnapshot::absent()));
        }
        Ok((pending, snapshot))
    }

    /// Confirmation preparation must preserve even expired pending data.
    pub fn snapshot(&self) -> Result<CodexOAuthPendingSnapshot> {
        let _lock = self.lock()?;
        self.read_locked().map(|(_, snapshot)| snapshot)
    }
    fn expire_locked(&self, snapshot: &CodexOAuthPendingSnapshot) -> Result<()> {
        if guarded_write::delete_guarded_versioned(&self.path, &snapshot.version)?
            != guarded_write::VersionedWriteOutcome::Written
        {
            return Err(CcrError::ValidationError(
                "OAuth pending changed during expiry cleanup".into(),
            ));
        }
        Ok(())
    }
    fn read_locked(&self) -> Result<(Option<CodexOAuthPendingState>, CodexOAuthPendingSnapshot)> {
        let bytes = match std::fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok((None, CodexOAuthPendingSnapshot::absent()));
            }
            Err(error) => return Err(storage_error("读取", error)),
        };
        let pending: CodexOAuthPendingState = serde_json::from_slice(&bytes).map_err(|error| {
            // Serde's diagnostic can quote an invalid value from the file.
            CcrError::ConfigError(format!(
                "OAuth pending 格式无效 (line {}, column {})",
                error.line(),
                error.column(),
            ))
        })?;
        let snapshot = CodexOAuthPendingSnapshot::of(&bytes, &pending);
        Ok((Some(pending), snapshot))
    }

    /// Saves credentials with private permissions and no backup or history.
    pub fn save(&self, pending: &CodexOAuthPendingState) -> Result<()> {
        self.save_with(pending, guarded_write::write_guarded)
    }

    fn save_with(
        &self,
        pending: &CodexOAuthPendingState,
        writer: impl FnOnce(&Path, &[u8], &WriteOptions) -> Result<()>,
    ) -> Result<()> {
        self.save_owned_with(pending, writer).map(|_| ())
    }
    pub fn save_owned(
        &self,
        pending: &CodexOAuthPendingState,
    ) -> Result<CodexOAuthPendingSnapshot> {
        self.save_owned_with(pending, guarded_write::write_guarded)
    }
    fn save_owned_with(
        &self,
        pending: &CodexOAuthPendingState,
        writer: impl FnOnce(&Path, &[u8], &WriteOptions) -> Result<()>,
    ) -> Result<CodexOAuthPendingSnapshot> {
        let _lock = self.lock()?;
        let disk = PendingOnDisk {
            login_id: &pending.login_id,
            auth_url: &pending.auth_url,
            redirect_uri: &pending.redirect_uri,
            code_verifier: &pending.code_verifier,
            state: &pending.state,
            port: pending.port,
            expires_at: pending.expires_at,
            callback_url: &pending.callback_url,
        };
        let bytes = serde_json::to_vec_pretty(&disk)
            .map_err(|_| CcrError::ConfigError("无法序列化 OAuth pending 状态".into()))?;
        writer(
            &self.path,
            &bytes,
            &WriteOptions {
                secret: true,
                backup: BackupPolicy::None,
                ..Default::default()
            },
        )?;
        Ok(CodexOAuthPendingSnapshot::of(&bytes, pending))
    }

    /// Cancels persisted state. Missing state is already cleared.
    pub fn clear(&self) -> Result<()> {
        let _lock = self.lock()?;
        self.clear_locked()
    }

    /// Same operation lock as save/load, plus the existing file CAS lock.
    /// Missing is already cleared; any later file revision remains intact.
    pub fn clear_if_snapshot(&self, snapshot: &CodexOAuthPendingSnapshot) -> Result<bool> {
        let _lock = self.lock()?;
        if !self.path.try_exists()? {
            return Ok(true);
        }
        Ok(
            guarded_write::delete_guarded_versioned(&self.path, &snapshot.version)?
                == guarded_write::VersionedWriteOutcome::Written,
        )
    }

    fn clear_locked(&self) -> Result<()> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(storage_error("清理", error)),
        }
    }
}

fn storage_error(operation: &str, error: std::io::Error) -> CcrError {
    CcrError::FileIoError(format!("OAuth pending {operation}失败: {error}"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestCodexEnv;
    use std::fs;

    const VERIFIER: &str = "synthetic-verifier-sentinel-0123456789";
    const STATE: &str = "synthetic-state-sentinel-9876543210";

    fn pending() -> CodexOAuthPendingState {
        CodexOAuthPendingState {
            login_id: "fixture-login".into(),
            auth_url: format!("https://example.invalid/authorize?state={STATE}").into(),
            redirect_uri: "http://localhost:1455/auth/callback".into(),
            code_verifier: VERIFIER.into(),
            state: STATE.into(),
            port: 1455,
            expires_at: 300,
            callback_url: Some(
                format!("http://localhost:1455/auth/callback?code={VERIFIER}&state={STATE}").into(),
            ),
        }
    }

    fn assert_no_copies(directory: &Path, current: Option<&Path>) {
        if !directory.exists() {
            return;
        }
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                assert_no_copies(&path, current);
            } else if Some(path.as_path()) != current {
                let bytes = fs::read(&path).unwrap();
                let content = String::from_utf8_lossy(&bytes);
                assert!(!content.contains(VERIFIER), "credential copy found");
                assert!(!content.contains(STATE), "state copy found");
            }
        }
    }

    #[test]
    fn conditional_cleanup_preserves_later_nonce_and_preparation_does_not_expire() {
        let root = tempfile::tempdir().unwrap();
        let store = CodexOAuthPendingStore::with_path(root.path().join("oauth_pending.json"));
        LockManager::with_scoped_directory(&root.path().join("locks"), || {
            store.save(&pending())?;
            let original = store.snapshot()?;
            assert!(
                store.path.exists(),
                "snapshot must not apply expiry cleanup"
            );
            let mut newer = pending();
            newer.state = "different-synthetic-nonce".into();
            newer.expires_at = 10_000;
            store.save(&newer)?;
            let bytes = fs::read(&store.path)?;
            assert!(!store.clear_if_snapshot(&original)?);
            assert_eq!(fs::read(&store.path)?, bytes);
            {
                let _lock = store.lock()?;
                assert!(store.expire_locked(&original).is_err());
            }
            assert_eq!(fs::read(&store.path)?, bytes);
            assert!(
                store.load_owned(1000)?.0.is_some(),
                "a later live pending must not be reported absent"
            );
            let current = store.snapshot()?;
            assert!(store.clear_if_snapshot(&current)?);
            assert!(store.clear_if_snapshot(&current)?);
            let (_, absent) = store.load_owned(1000)?;
            store.save(&newer)?;
            assert!(!store.clear_if_snapshot(&absent)?);
            assert_no_copies(root.path(), Some(&store.path));
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn oauth_pending_create_replace_cancel_and_expiry_never_make_copies() {
        let mut env = TestCodexEnv::new();
        let backups = env.root().join("configured-backups");
        fs::create_dir_all(&backups).unwrap();
        env.set_env("CCR_BACKUP_DIR", backups.as_os_str());
        let store = CodexOAuthPendingStore::new().unwrap();
        let mut value = pending();
        for login in ["first", "second"] {
            value.login_id = login.into();
            store.save(&value).unwrap();
            let loaded = store.load(100).unwrap().unwrap();
            assert_eq!(loaded.login_id, login);
            assert_eq!(loaded.code_verifier, VERIFIER);
            assert_eq!(loaded.state, STATE);
            assert_no_copies(env.root(), Some(&store.path));
            assert_eq!(fs::read_dir(&backups).unwrap().count(), 0);
            assert_eq!(fs::read_dir(env.ccr_codex_dir()).unwrap().count(), 1);
        }
        store.clear().unwrap();
        store.clear().unwrap();
        assert!(!store.path.exists());
        assert_no_copies(env.root(), None);
        store.save(&value).unwrap();
        assert!(store.load(value.expires_at).unwrap().is_none());
        assert!(!store.path.exists());
        assert_no_copies(env.root(), None);
        assert_eq!(fs::read_dir(&backups).unwrap().count(), 0);
    }

    #[test]
    fn oauth_pending_permission_failure_preserves_old_or_missing_target() {
        let mut env = TestCodexEnv::new();
        let backups = env.root().join("configured-backups");
        fs::create_dir_all(&backups).unwrap();
        env.set_env("CCR_BACKUP_DIR", backups.as_os_str());
        let store = CodexOAuthPendingStore::new().unwrap();
        let fail_permission = |_: &Path, _: &[u8], options: &WriteOptions| {
            assert!(options.secret);
            assert!(matches!(options.backup, BackupPolicy::None));
            Err(storage_error(
                "设置权限",
                std::io::ErrorKind::PermissionDenied.into(),
            ))
        };
        assert!(store.save_with(&pending(), fail_permission).is_err());
        assert!(!store.path.exists());
        assert_no_copies(env.root(), None);
        store.save(&pending()).unwrap();
        let before = fs::read(&store.path).unwrap();
        let error = store.save_with(&pending(), fail_permission).unwrap_err();
        assert_eq!(fs::read(&store.path).unwrap(), before);
        assert!(!error.to_string().contains(VERIFIER));
        assert!(!error.to_string().contains(STATE));
        assert_no_copies(env.root(), Some(&store.path));
        assert_eq!(fs::read_dir(&backups).unwrap().count(), 0);
    }

    #[cfg(windows)]
    #[test]
    fn oauth_pending_windows_create_and_replace_keep_private_dacl() {
        let env = TestCodexEnv::new();
        let store = CodexOAuthPendingStore::new().unwrap();
        let inspect = || {
            let output = std::process::Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command",
                    "$ErrorActionPreference = 'Stop'; $acl = [System.IO.File]::GetAccessControl($env:CCR_PENDING_ACL_TEST); $rules = @($acl.GetAccessRules($true, $true, [System.Security.Principal.SecurityIdentifier])); $sid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value; if (!$acl.AreAccessRulesProtected -or $rules.Count -ne 1 -or $rules[0].IdentityReference.Value -ne $sid -or $rules[0].AccessControlType -ne 'Allow') { exit 1 }; $acl.GetSecurityDescriptorSddlForm([System.Security.AccessControl.AccessControlSections]::Access)"])
                .env("CCR_PENDING_ACL_TEST", &store.path)
                .output().unwrap();
            assert!(
                output.status.success(),
                "private DACL assertion failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            output.stdout
        };
        store.save(&pending()).unwrap();
        let first = inspect();
        let mut replacement = pending();
        replacement.login_id = "replacement".into();
        store.save(&replacement).unwrap();
        assert_eq!(inspect(), first);
        assert_no_copies(env.root(), Some(&store.path));
    }

    #[cfg(windows)]
    #[test]
    fn oauth_pending_cancel_and_expiry_denial_return_errors_without_copies() {
        use std::os::windows::fs::OpenOptionsExt;
        let env = TestCodexEnv::new();
        let store = CodexOAuthPendingStore::new().unwrap();
        store.save(&pending()).unwrap();
        let before = fs::read(&store.path).unwrap();
        // Allow reads/writes, but deny delete-sharing for the real cleanup.
        let held = fs::OpenOptions::new()
            .read(true)
            .share_mode(3)
            .open(&store.path)
            .unwrap();
        for error in [store.clear().unwrap_err(), store.load(300).unwrap_err()] {
            assert!(!error.to_string().contains(VERIFIER));
            assert!(!error.to_string().contains(STATE));
        }
        assert_eq!(fs::read(&store.path).unwrap(), before);
        assert_no_copies(env.root(), Some(&store.path));
        drop(held);
        assert!(store.load(300).unwrap().is_none());
        assert_no_copies(env.root(), None);
    }

    #[test]
    fn oauth_pending_errors_and_default_output_redact_sentinels() {
        let env = TestCodexEnv::new();
        let store = CodexOAuthPendingStore::new().unwrap();
        let value = pending();
        for visible in [format!("{value:?}"), serde_json::to_string(&value).unwrap()] {
            assert!(!visible.contains(VERIFIER));
            assert!(!visible.contains(STATE));
        }
        fs::write(
            &store.path,
            format!("{{\"port\":\"{VERIFIER}\",\"state\":\"{STATE}\"}}"),
        )
        .unwrap();
        let error = store.load(100).unwrap_err();
        for visible in [error.to_string(), error.user_message()] {
            assert!(!visible.contains(VERIFIER));
            assert!(!visible.contains(STATE));
        }
        fs::remove_file(&store.path).unwrap();
        fs::create_dir(&store.path).unwrap();
        let cancel_error = store.clear().unwrap_err().to_string();
        assert!(!cancel_error.contains(VERIFIER));
        assert!(!cancel_error.contains(STATE));
        assert_no_copies(env.root(), None);
    }

    #[test]
    fn oauth_pending_loads_legacy_plaintext_shape_without_changing_values() {
        let env = TestCodexEnv::new();
        let store = CodexOAuthPendingStore::new().unwrap();
        let legacy = serde_json::json!({
            "loginId": "legacy-login",
            "authUrl": format!("https://example.invalid/authorize?state={STATE}"),
            "redirectUri": "http://localhost:1455/auth/callback",
            "codeVerifier": VERIFIER, "state": STATE, "port": 1455,
            "expiresAt": 300, "callbackUrl": null,
        });
        ccr_core::core::AtomicWriter::new(&store.path)
            .secret(true)
            .write(&serde_json::to_vec(&legacy).unwrap())
            .unwrap();
        let loaded = store.load(100).unwrap().unwrap();
        assert_eq!(loaded.code_verifier, VERIFIER);
        assert_eq!(loaded.state, STATE);
        store.save(&loaded).unwrap();
        let saved: serde_json::Value =
            serde_json::from_slice(&fs::read(&store.path).unwrap()).unwrap();
        assert_eq!(saved, legacy);
        assert_no_copies(env.root(), Some(&store.path));
    }

    #[cfg(unix)]
    #[test]
    fn oauth_pending_preserves_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let _env = TestCodexEnv::new();
        let store = CodexOAuthPendingStore::new().unwrap();
        store.save(&pending()).unwrap();
        assert_eq!(
            fs::metadata(&store.path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        fs::set_permissions(&store.path, fs::Permissions::from_mode(0o400)).unwrap();
        store.save(&pending()).unwrap();
        assert_eq!(
            fs::metadata(&store.path).unwrap().permissions().mode() & 0o777,
            0o400
        );
    }
}
