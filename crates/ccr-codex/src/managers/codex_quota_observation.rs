//! Bounded, private quota metadata. No credentials, response body or email is persisted.

use ccr_core::core::guarded_write::{self, BackupPolicy, WriteOptions};
use ccr_core::core::lock::{FileLock, LockManager};
use ccr_core::{CcrError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration as LockDuration;

const MAX_OBSERVATIONS: usize = 4096;
const MAX_BYTES: u64 = 8 * 1024 * 1024;
const RETENTION_DAYS: i64 = 35;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CodexQuotaObservation {
    pub account_id: String,
    pub plan: Option<String>,
    pub source: String,
    pub bucket_source: String,
    pub bucket_id: Option<String>,
    pub window_role: String,
    pub duration_minutes: Option<i64>,
    pub limit_window_seconds: Option<i64>,
    pub used_percent: Option<f64>,
    pub resets_at: Option<i64>,
    /// Stable server reset identity. A reset-after countdown is insufficient.
    pub reset_generation: Option<String>,
    pub network_acquired_at: DateTime<Utc>,
    pub returned_at: DateTime<Utc>,
    pub scope_supported: bool,
    pub scan_generation: Option<String>,
    pub scan_watermark: Option<DateTime<Utc>>,
    pub scan_complete: bool,
}
impl CodexQuotaObservation {
    pub fn bucket(&self) -> String {
        format!(
            "{}:{}",
            self.bucket_source,
            self.bucket_id.as_deref().unwrap_or("wham_main")
        )
    }
    fn identity(&self) -> (&str, DateTime<Utc>, &str, &str, Option<&str>, Option<i64>) {
        (
            &self.account_id,
            self.network_acquired_at,
            &self.window_role,
            &self.bucket_source,
            self.bucket_id.as_deref(),
            self.duration_minutes,
        )
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct History {
    schema_version: u32,
    observations: Vec<CodexQuotaObservation>,
}

pub struct CodexQuotaObservationStore {
    path: PathBuf,
}
impl CodexQuotaObservationStore {
    pub fn new() -> Result<Self> {
        Ok(Self::with_path(
            crate::utils::CodexPaths::resolve()?
                .ccr_codex_dir
                .join("quota_observations.json"),
        ))
    }
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }
    fn lock(&self) -> Result<FileLock> {
        let absolute = guarded_write::normalized_resource_path(&self.path)?;
        let key = guarded_write::content_version_token(
            absolute.to_string_lossy().to_lowercase().as_bytes(),
        );
        LockManager::with_default_path()?.lock_resource(
            &format!("codex_quota_observations_{key}"),
            LockDuration::from_secs(10),
        )
    }
    pub fn load(&self) -> Result<Vec<CodexQuotaObservation>> {
        let _lock = self.lock()?;
        self.load_locked()
    }
    fn load_locked(&self) -> Result<Vec<CodexQuotaObservation>> {
        let file = match std::fs::File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };
        if file.metadata()?.len() > MAX_BYTES {
            return Err(history_error("quota_history_size_limit"));
        }
        let mut bytes = Vec::new();
        file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(history_error("quota_history_size_limit"));
        }
        let history: History =
            serde_json::from_slice(&bytes).map_err(|_| history_error("quota_history_invalid"))?;
        if history.schema_version != 1 || history.observations.len() > MAX_OBSERVATIONS {
            return Err(history_error("quota_history_schema_limit"));
        }
        Ok(history.observations)
    }
    /// A corrupt or oversized old history is never cleared or replaced.
    pub fn record(
        &self,
        observations: Vec<CodexQuotaObservation>,
        now: DateTime<Utc>,
    ) -> Result<()> {
        self.record_with(observations, now, guarded_write::write_guarded)
    }
    fn record_with(
        &self,
        observations: Vec<CodexQuotaObservation>,
        now: DateTime<Utc>,
        writer: impl FnOnce(&Path, &[u8], &WriteOptions) -> Result<()>,
    ) -> Result<()> {
        let _lock = self.lock()?;
        let mut old = self.load_locked()?;
        for observation in observations {
            if observation.source != "network"
                || observation.account_id.is_empty()
                || observation.network_acquired_at > now
            {
                continue;
            }
            if !old
                .iter()
                .any(|candidate| candidate.identity() == observation.identity())
            {
                old.push(observation);
            }
        }
        old.retain(|observation| {
            observation.network_acquired_at >= now - Duration::days(RETENTION_DAYS)
        });
        old.sort_by_key(|observation| observation.network_acquired_at);
        if old.len() > MAX_OBSERVATIONS {
            old.drain(..old.len() - MAX_OBSERVATIONS);
        }
        let mut bytes = serde_json::to_vec(&History {
            schema_version: 1,
            observations: old.clone(),
        })?;
        while bytes.len() as u64 > MAX_BYTES && !old.is_empty() {
            old.remove(0);
            bytes = serde_json::to_vec(&History {
                schema_version: 1,
                observations: old.clone(),
            })?;
        }
        if bytes.len() as u64 > MAX_BYTES {
            return Err(history_error("quota_history_size_limit"));
        }
        writer(
            &self.path,
            &bytes,
            &WriteOptions {
                secret: true,
                backup: BackupPolicy::None,
                ..Default::default()
            },
        )
    }
}
fn history_error(message: &str) -> CcrError {
    CcrError::ConfigError(message.into())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestCodexEnv;
    fn observation(time: DateTime<Utc>) -> CodexQuotaObservation {
        CodexQuotaObservation {
            account_id: "stable-account".into(),
            source: "network".into(),
            window_role: "primary".into(),
            network_acquired_at: time,
            ..Default::default()
        }
    }
    #[test]
    fn duplicate_acquisition_rename_and_failed_write_keep_history() {
        let env = TestCodexEnv::new();
        let store = CodexQuotaObservationStore::with_path(
            env.ccr_codex_dir().join("quota_observations.json"),
        );
        let now = Utc::now();
        store.record(vec![observation(now)], now).unwrap();
        store.record(vec![observation(now)], now).unwrap();
        assert_eq!(store.load().unwrap().len(), 1);
        let before = std::fs::read(&store.path).unwrap();
        assert!(
            store
                .record_with(
                    vec![observation(now + Duration::seconds(1))],
                    now + Duration::seconds(1),
                    |_, _, options| {
                        assert!(options.secret);
                        Err(history_error("synthetic_write_failure"))
                    }
                )
                .is_err()
        );
        assert_eq!(std::fs::read(&store.path).unwrap(), before);
        let text = String::from_utf8(before).unwrap();
        for forbidden in [
            "email",
            "access_token",
            "refresh_token",
            "raw_data",
            "prompt",
            "auth.json",
        ] {
            assert!(!text.contains(forbidden));
        }
    }
    #[test]
    fn corrupt_and_oversized_history_remain_unchanged() {
        let env = TestCodexEnv::new();
        let store = CodexQuotaObservationStore::with_path(
            env.ccr_codex_dir().join("quota_observations.json"),
        );
        std::fs::write(&store.path, b"bad-history").unwrap();
        assert!(
            store
                .record(vec![observation(Utc::now())], Utc::now())
                .is_err()
        );
        assert_eq!(std::fs::read(&store.path).unwrap(), b"bad-history");
        let file = std::fs::File::create(&store.path).unwrap();
        file.set_len(MAX_BYTES + 1).unwrap();
        assert!(store.load().is_err());
        assert_eq!(std::fs::metadata(&store.path).unwrap().len(), MAX_BYTES + 1);
    }
    #[test]
    fn retention_and_count_limits() {
        let env = TestCodexEnv::new();
        let store = CodexQuotaObservationStore::with_path(
            env.ccr_codex_dir().join("quota_observations.json"),
        );
        let now = Utc::now();
        let mut values = vec![observation(now - Duration::days(36))];
        values.extend((0..4100).map(|index| observation(now - Duration::seconds(index))));
        store.record(values, now).unwrap();
        let saved = store.load().unwrap();
        assert_eq!(saved.len(), 4096);
        assert!(
            saved
                .iter()
                .all(|value| value.network_acquired_at >= now - Duration::days(35))
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&store.path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn concurrent_records_keep_every_distinct_network_observation() {
        let env = TestCodexEnv::new();
        let path = env.ccr_codex_dir().join("quota_observations.json");
        let now = Utc::now();
        std::thread::scope(|scope| {
            let handles = (0..8)
                .map(|index| {
                    let path = path.clone();
                    scope.spawn(move || {
                        let time = now - Duration::seconds(index);
                        CodexQuotaObservationStore::with_path(path)
                            .record(vec![observation(time)], now)
                    })
                })
                .collect::<Vec<_>>();
            for handle in handles {
                handle.join().unwrap().unwrap();
            }
        });
        let saved = CodexQuotaObservationStore::with_path(path).load().unwrap();
        assert_eq!(saved.len(), 8);
    }

    #[test]
    fn cache_future_and_distinct_bucket_identities_follow_storage_contract() {
        let env = TestCodexEnv::new();
        let store = CodexQuotaObservationStore::with_path(
            env.ccr_codex_dir().join("quota_observations.json"),
        );
        let now = Utc::now();
        let mut cached = observation(now);
        cached.source = "cache".into();
        let future = observation(now + Duration::seconds(1));
        let mut first = observation(now);
        first.bucket_source = "wham_limit_id".into();
        first.bucket_id = Some("a".into());
        let mut second = first.clone();
        second.bucket_id = Some("b".into());
        store
            .record(vec![cached, future, first, second], now)
            .unwrap();
        let saved = store.load().unwrap();
        assert_eq!(saved.len(), 2);
        assert_ne!(saved[0].bucket(), saved[1].bucket());
    }

    #[test]
    fn byte_limit_trims_oldest_and_unknown_fields_do_not_enter_history() {
        let env = TestCodexEnv::new();
        let store = CodexQuotaObservationStore::with_path(
            env.ccr_codex_dir().join("quota_observations.json"),
        );
        let now = Utc::now();
        let mut old = observation(now - Duration::seconds(1));
        old.bucket_id = Some("x".repeat(MAX_BYTES as usize));
        store.record(vec![old, observation(now)], now).unwrap();
        assert!(std::fs::metadata(&store.path).unwrap().len() <= MAX_BYTES);
        assert_eq!(store.load().unwrap().len(), 1);
        let bytes = br#"{"schema_version":1,"observations":[],"access_token":"synthetic"}"#;
        std::fs::write(&store.path, bytes).unwrap();
        assert!(store.record(vec![observation(now)], now).is_err());
        assert_eq!(std::fs::read(&store.path).unwrap(), bytes);
    }

    #[cfg(windows)]
    #[test]
    fn windows_sharing_denial_keeps_old_complete_history() {
        use std::os::windows::fs::OpenOptionsExt;
        let env = TestCodexEnv::new();
        let store = CodexQuotaObservationStore::with_path(
            env.ccr_codex_dir().join("quota_observations.json"),
        );
        let now = Utc::now();
        store.record(vec![observation(now)], now).unwrap();
        let before = std::fs::read(&store.path).unwrap();
        // Allow reads and writes, but deny replacement while this handle is open.
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(3)
            .open(&store.path)
            .unwrap();
        assert!(
            store
                .record(
                    vec![observation(now + Duration::seconds(1))],
                    now + Duration::seconds(1)
                )
                .is_err()
        );
        assert_eq!(std::fs::read(&store.path).unwrap(), before);
        drop(held);
        store
            .record(
                vec![observation(now + Duration::seconds(1))],
                now + Duration::seconds(1),
            )
            .unwrap();
        assert_eq!(store.load().unwrap().len(), 2);
    }

    #[cfg(windows)]
    #[test]
    fn windows_create_replace_uses_private_acl_without_exposing_sid() {
        fn acl_receipt(path: &Path) -> String {
            let script = "$taskPath=[Environment]::GetEnvironmentVariable('CCR_TEST_QUOTA_PATH'); $acl=Get-Acl -LiteralPath $taskPath; $sid=[Security.Principal.WindowsIdentity]::GetCurrent().User; $private=$acl.AreAccessRulesProtected -and @($acl.Access).Count -eq 1 -and @($acl.Access)[0].IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Equals($sid); if (!$private) { exit 2 }; $bytes=[Text.Encoding]::UTF8.GetBytes($acl.GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::Access)); [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes))";
            let output = std::process::Command::new("pwsh")
                .args(["-NoProfile", "-NonInteractive", "-Command", script])
                .env("CCR_TEST_QUOTA_PATH", path)
                .output()
                .unwrap();
            assert!(output.status.success(), "native ACL predicate failed");
            String::from_utf8(output.stdout).unwrap().trim().to_owned()
        }
        let env = TestCodexEnv::new();
        let store = CodexQuotaObservationStore::with_path(
            env.ccr_codex_dir().join("quota_observations.json"),
        );
        let now = Utc::now();
        store.record(vec![observation(now)], now).unwrap();
        let before = acl_receipt(&store.path);
        store
            .record(
                vec![observation(now + Duration::seconds(1))],
                now + Duration::seconds(1),
            )
            .unwrap();
        let after = acl_receipt(&store.path);
        assert!(before == after, "replacement changed the private ACL");
    }
}
