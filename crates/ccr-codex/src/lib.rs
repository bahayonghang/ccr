//! Dedicated crate for the CCR Codex domain.

pub mod managers;
pub mod models;
pub mod platforms;
pub mod services;
pub mod utils;

pub use ccr_config::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
pub use ccr_core::{CcrError, Result};
pub use managers::codex_config::{CachedCodexConfigManager, CodexConfigManager};
pub use models::codex_auth::*;
pub use models::codex_model_provider::*;
pub use models::codex_runtime_diagnostic::*;
pub use models::opencode_auth::*;
pub use platforms::CodexPlatform;
pub use services::{
    AuthReadSnapshot, CodexAppServer, CodexAppServerCleanup, CodexAppServerCleanupReport,
    CodexAuthCacheAction, CodexAuthService, CodexHistoryBackupPruneResult,
    CodexHistoryBackupSummary, CodexHistoryProviderBuckets, CodexHistoryRestoreResult,
    CodexHistorySyncOptions, CodexHistorySyncResult, CodexHistorySyncService,
    CodexHistorySyncStatus, CodexHistoryVisibilityDiagnostics, CodexModelProviderStoreService,
    CodexOAuthTokenService, CodexProcessDiscoveryIssue, CodexProcessService, CodexQuotaService,
    CodexRegistryStore, CodexRollingUsage, CodexRuntimeCommitPlan, CodexRuntimeService,
    CodexSessionDetail, CodexSessionExport, CodexSessionMessage, CodexSessionRestoreSummary,
    CodexSessionService, CodexSessionSummary, CodexSessionTrashService, CodexSessionTrashSummary,
    CodexSignalFailure, CodexSignalStage, CodexTrashedSessionRecord, CodexUsageRecord,
    CodexUsageService, CodexUsageStats, OAuthRepairOutcome, OpenCodeAuthService,
    OpenCodeQuotaService, OpenCodeRollingUsage, OpenCodeUsageRecord, OpenCodeUsageService,
    OpenCodeUsageStats, TerminationKind,
};

#[cfg(test)]
pub(crate) mod test_support {
    use std::ffi::{OsStr, OsString};
    use std::path::{Path, PathBuf};
    use std::sync::{LazyLock, Mutex, MutexGuard};
    use tempfile::TempDir;

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    pub(crate) struct TestCodexEnv {
        temp_dir: TempDir,
        root: PathBuf,
        ccr_codex_dir: PathBuf,
        codex_dir: PathBuf,
        lock_dir: PathBuf,
        previous_vars: Vec<(&'static str, Option<OsString>)>,
        _env_guard: MutexGuard<'static, ()>,
    }

    impl TestCodexEnv {
        pub(crate) fn new() -> Self {
            let env_guard = ENV_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let temp_dir = tempfile::tempdir().expect("test codex env temp dir should be created");
            let home = temp_dir.path();
            let root = home.join(".ccr");
            let ccr_codex_dir = root.join("platforms").join("codex");
            let codex_dir = home.join(".codex");
            let lock_dir = home.join(".locks");

            for dir in [&root, &ccr_codex_dir, &codex_dir, &lock_dir] {
                std::fs::create_dir_all(dir).expect("test codex env directory should be created");
            }

            let mut previous_vars = Vec::new();
            set_env_var(&mut previous_vars, "CCR_ROOT", root.as_os_str());
            set_env_var(&mut previous_vars, "CCR_DATA_DIR", root.as_os_str());
            set_env_var(&mut previous_vars, "CCR_CODEX_DIR", codex_dir.as_os_str());
            set_env_var(&mut previous_vars, "CCR_LOCK_DIR", lock_dir.as_os_str());
            // inspect_runtime treats host CODEX_HOME / OpenAI env as overrides.
            remove_env_var(&mut previous_vars, "CODEX_HOME");
            remove_env_var(&mut previous_vars, "OPENAI_BASE_URL");
            remove_env_var(&mut previous_vars, "OPENAI_API_KEY");
            remove_env_var(&mut previous_vars, "CODEX_API_KEY");

            Self {
                temp_dir,
                root,
                ccr_codex_dir,
                codex_dir,
                lock_dir,
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

        pub(crate) fn ccr_codex_dir(&self) -> &Path {
            &self.ccr_codex_dir
        }

        pub(crate) fn codex_dir(&self) -> &Path {
            &self.codex_dir
        }

        pub(crate) fn lock_dir(&self) -> &Path {
            &self.lock_dir
        }

        pub(crate) fn set_env(&mut self, key: &'static str, value: &OsStr) {
            set_env_var(&mut self.previous_vars, key, value);
        }

        pub(crate) fn remove_env(&mut self, key: &'static str) {
            self.previous_vars.push((key, std::env::var_os(key)));
            // SAFETY: TestCodexEnv holds the process-wide ccr-codex test env lock until Drop.
            unsafe {
                std::env::remove_var(key);
            }
        }
    }

    impl Drop for TestCodexEnv {
        fn drop(&mut self) {
            self.restore_env_vars();
        }
    }

    impl TestCodexEnv {
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
        // SAFETY: TestCodexEnv holds the process-wide ccr-codex test env lock until Drop.
        unsafe {
            std::env::set_var(key, value);
        }
    }

    fn remove_env_var(
        previous_vars: &mut Vec<(&'static str, Option<OsString>)>,
        key: &'static str,
    ) {
        previous_vars.push((key, std::env::var_os(key)));
        // SAFETY: TestCodexEnv holds the process-wide ccr-codex test env lock until Drop.
        unsafe {
            std::env::remove_var(key);
        }
    }

    fn restore_env_var(key: &str, previous: Option<OsString>) {
        // SAFETY: Drop runs while TestCodexEnv still holds the process-wide test env lock.
        unsafe {
            match previous {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::TestCodexEnv;
        use std::ffi::OsString;

        #[test]
        fn test_codex_env_sets_and_restores_codex_env_vars() {
            let mut env = TestCodexEnv::new();
            let previous_root = captured_previous(&env, "CCR_ROOT");
            let previous_data_dir = captured_previous(&env, "CCR_DATA_DIR");
            let previous_codex_dir = captured_previous(&env, "CCR_CODEX_DIR");
            let previous_lock_dir = captured_previous(&env, "CCR_LOCK_DIR");

            assert_eq!(
                std::env::var_os("CCR_ROOT").as_deref(),
                Some(env.root().as_os_str())
            );
            assert_eq!(
                std::env::var_os("CCR_DATA_DIR").as_deref(),
                Some(env.root().as_os_str())
            );
            assert_eq!(
                std::env::var_os("CCR_CODEX_DIR").as_deref(),
                Some(env.codex_dir().as_os_str())
            );
            assert_eq!(
                std::env::var_os("CCR_LOCK_DIR").as_deref(),
                Some(env.lock_dir().as_os_str())
            );
            assert!(env.root().starts_with(env.home()));
            assert!(env.ccr_codex_dir().starts_with(env.root()));
            assert!(env.codex_dir().starts_with(env.home()));
            assert!(env.lock_dir().starts_with(env.home()));

            env.restore_env_vars();

            assert_eq!(std::env::var_os("CCR_ROOT"), previous_root);
            assert_eq!(std::env::var_os("CCR_DATA_DIR"), previous_data_dir);
            assert_eq!(std::env::var_os("CCR_CODEX_DIR"), previous_codex_dir);
            assert_eq!(std::env::var_os("CCR_LOCK_DIR"), previous_lock_dir);
        }

        fn captured_previous(env: &TestCodexEnv, key: &'static str) -> Option<OsString> {
            env.previous_vars
                .iter()
                .find_map(|(captured_key, value)| (*captured_key == key).then(|| value.clone()))
                .flatten()
        }
    }
}
