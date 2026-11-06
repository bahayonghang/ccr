//! Captured roots for native/embedding platform settings and authentication.
use ccr_config::{ClaudeRuntimePaths, Platform, PlatformPaths};
use ccr_core::{CcrError, Result};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct PlatformRoots {
    pub home: PathBuf,
    pub ccr: PathBuf,
    pub claude: ClaudeRuntimePaths,
    pub codex_store: PathBuf,
    pub codex_runtime: PathBuf,
    pub grok_runtime: PathBuf,
    pub locks: PathBuf,
    pub fixture: bool,
}
impl PlatformRoots {
    /// Resolves once on a worker; consumers retain this exact target through commit.
    pub fn capture() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("platform_home_unavailable".into()))?;
        let ccr = PlatformPaths::new(Platform::Claude)?.root;
        let codex = ccr_codex::utils::CodexPaths::resolve()?;
        let locks = ccr_core::core::LockManager::with_default_path()?
            .lock_dir()
            .to_path_buf();
        let grok_runtime = std::env::var_os("GROK_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".grok"));
        Ok(Self {
            home,
            ccr,
            claude: ClaudeRuntimePaths::from_env()?,
            codex_store: codex.ccr_codex_dir,
            codex_runtime: codex.codex_dir,
            grok_runtime,
            locks,
            fixture: false,
        })
    }
    /// Pure construction, no HOME/config/credential/OS preference access.
    pub fn fixture(root: &Path) -> Result<Self> {
        if !root.is_absolute() {
            return Err(CcrError::ValidationError(
                "platform_fixture_root_must_be_absolute".into(),
            ));
        }
        let home = root.join("home");
        let ccr = root.join("ccr");
        Ok(Self {
            claude: ClaudeRuntimePaths::resolve_with(&home, |_| None),
            codex_store: ccr.join("platforms/codex"),
            codex_runtime: home.join(".codex"),
            grok_runtime: home.join(".grok"),
            locks: root.join("locks"),
            home,
            ccr,
            fixture: true,
        })
    }
    pub fn claude_auth(&self) -> crate::services::ClaudeAuthService {
        let platform_dir = self.ccr.join("platforms/claude");
        let paths = PlatformPaths {
            root: self.ccr.clone(),
            registry_file: self.ccr.join("config.toml"),
            profiles_file: platform_dir.join("profiles.toml"),
            settings_file: platform_dir.join("settings.json"),
            history_file: self.ccr.join("history/claude.json"),
            backups_dir: self.ccr.join("backups/claude"),
            platform_dir,
        };
        crate::services::ClaudeAuthService::from_runtime_parts(
            self.ccr.join("platforms/claude"),
            self.claude.clone(),
            self.locks.clone(),
        )
        .with_process_environment(!self.fixture)
        .with_profile_paths(paths)
    }
    pub fn codex_auth(&self) -> ccr_codex::CodexAuthService {
        ccr_codex::CodexAuthService::from_dirs(self.codex_store.clone(), self.codex_runtime.clone())
    }
}
