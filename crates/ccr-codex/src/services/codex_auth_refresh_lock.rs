//! Credential operation locks precede registry and guarded-write leaf locks.
//! Acquire multiple resources in sorted order and never reacquire a held resource.

use super::codex_auth_identity::OAuthIdentity;
use crate::models::{CodexAuthJson, CodexAuthTokens};
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::guarded_write::content_version_token;
use ccr_core::core::lock::{FileLock, LockManager};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CredentialResource(String);

impl CredentialResource {
    pub(crate) fn path(path: &Path) -> Self {
        // Resolve the parent so creating a missing source does not change its key.
        let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
        let parent = absolute.parent().unwrap_or(&absolute);
        let normalized = parent
            .ancestors()
            .find_map(|ancestor| {
                ancestor.canonicalize().ok().and_then(|resolved| {
                    absolute
                        .strip_prefix(ancestor)
                        .ok()
                        .map(|suffix| resolved.join(suffix))
                })
            })
            .unwrap_or(absolute);
        let normalized = normalized.to_string_lossy();
        let normalized = if cfg!(windows) {
            normalized.to_ascii_lowercase()
        } else {
            normalized.into_owned()
        };
        Self::hashed(&format!("path:{normalized}"))
    }

    fn hashed(input: &str) -> Self {
        Self(format!(
            "codex_auth_credentials_{}",
            content_version_token(input.as_bytes())
        ))
    }

    pub(crate) fn from_tokens(tokens: Option<&CodexAuthTokens>, path: &Path) -> Self {
        match tokens.and_then(OAuthIdentity::from_tokens) {
            Some(identity) => Self::hashed(&format!("identity:{}", identity.key())),
            None => Self::path(path),
        }
    }

    pub(crate) fn from_path(path: &Path) -> Self {
        let auth = std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<CodexAuthJson>(&bytes).ok());
        Self::from_tokens(auth.as_ref().and_then(|auth| auth.tokens.as_ref()), path)
    }
}

pub(crate) struct CredentialLocks {
    resources: Vec<CredentialResource>,
    sources: Vec<(PathBuf, CredentialResource)>,
    _locks: Vec<FileLock>,
}

impl CredentialLocks {
    pub(crate) fn acquire(mut resources: Vec<CredentialResource>) -> Result<Self> {
        resources.sort();
        resources.dedup();
        let manager = LockManager::with_default_path()?;
        let mut locks = Vec::with_capacity(resources.len());
        for resource in &resources {
            locks.push(manager.lock_resource(&resource.0, Duration::from_secs(30))?);
        }
        Ok(Self {
            resources,
            sources: Vec::new(),
            _locks: locks,
        })
    }

    pub(crate) fn acquire_sources(sources: Vec<(PathBuf, CredentialResource)>) -> Result<Self> {
        Self::acquire_sources_with_resources(sources, Vec::new())
    }

    pub(crate) fn acquire_sources_with_resources(
        sources: Vec<(PathBuf, CredentialResource)>,
        extra_resources: Vec<CredentialResource>,
    ) -> Result<Self> {
        let resources = sources
            .iter()
            .flat_map(|(path, expected)| [CredentialResource::path(path), expected.clone()])
            .chain(extra_resources)
            .collect();
        let mut held = Self::acquire(resources)?;
        held.sources = sources;
        Ok(held)
    }

    pub(crate) async fn acquire_async_sources(
        sources: Vec<(PathBuf, CredentialResource)>,
    ) -> Result<Self> {
        tokio::task::spawn_blocking(move || Self::acquire_sources(sources))
            .await
            .map_err(|_| CcrError::FileLockError("凭据操作锁任务失败".into()))?
    }

    pub(crate) fn verify_path(&self, path: &Path) -> Result<()> {
        let key = CredentialResource::path(path);
        if self.sources.iter().any(|(source, expected)| {
            CredentialResource::path(source) == key
                && *expected == CredentialResource::from_path(path)
        }) {
            Ok(())
        } else {
            Err(CcrError::ConfigError(
                "auth 来源身份已变化，跳过操作".into(),
            ))
        }
    }

    pub(crate) fn holds_resource(&self, resource: &CredentialResource) -> bool {
        self.resources.contains(resource)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::process::{Command, Stdio};
    use std::time::Instant;

    fn auth(user: &str, refresh: &str) -> CodexAuthJson {
        let jwt = super::super::codex_auth_identity::test_jwt(
            serde_json::json!({"chatgpt_user_id":user}),
        );
        serde_json::from_value(serde_json::json!({"tokens":{
            "id_token":jwt, "access_token":jwt, "refresh_token":refresh,
            "account_id":"shared-lock-workspace"
        }}))
        .unwrap()
    }

    #[test]
    fn credential_resources_isolate_users_and_hide_identity_values() {
        let env = crate::test_support::TestCodexEnv::new();
        let path_a = env.codex_dir().join("auth.json");
        let path_b = env.ccr_codex_dir().join("auth/alias.json");
        let a = auth("lock-private-user-a", "synthetic-refresh-a");
        let b = auth("lock-private-user-b", "synthetic-refresh-b");
        let resource_a = CredentialResource::from_tokens(a.tokens.as_ref(), &path_a);
        let alias = CredentialResource::from_tokens(a.tokens.as_ref(), &path_b);
        let resource_b = CredentialResource::from_tokens(b.tokens.as_ref(), &path_a);
        assert_eq!(resource_a, alias);
        assert_ne!(resource_a, resource_b);
        let debug = format!("{resource_a:?}");
        assert!(!debug.contains("lock-private-user-a") && !debug.contains("shared-lock-workspace"));
        let locks =
            CredentialLocks::acquire(vec![resource_b.clone(), resource_a.clone(), alias]).unwrap();
        assert_eq!(locks.resources.len(), 2);
        assert!(
            locks
                .resources
                .windows(2)
                .all(|window| window[0] < window[1])
        );
        let contender =
            LockManager::new(env.lock_dir()).lock_resource(&resource_a.0, Duration::ZERO);
        assert!(matches!(contender, Err(CcrError::LockTimeout(_))));
        drop(locks);
        let _released = LockManager::new(env.lock_dir())
            .lock_resource(&resource_a.0, Duration::ZERO)
            .unwrap();
        assert_ne!(
            CredentialResource::from_tokens(None, &path_a),
            CredentialResource::from_tokens(None, &path_b)
        );
    }

    #[test]
    fn credential_lock_child() {
        let Some(path) = std::env::var_os("CCR_TEST_CREDENTIAL_LOCK_SOURCE") else {
            return;
        };
        let path = std::path::PathBuf::from(path);
        let ready =
            std::path::PathBuf::from(std::env::var_os("CCR_TEST_CREDENTIAL_LOCK_READY").unwrap());
        let result =
            std::path::PathBuf::from(std::env::var_os("CCR_TEST_CREDENTIAL_LOCK_RESULT").unwrap());
        let resource = CredentialResource::from_path(&path);
        assert!(matches!(
            LockManager::with_default_path()
                .unwrap()
                .lock_resource(&resource.0, Duration::ZERO),
            Err(CcrError::LockTimeout(_))
        ));
        std::fs::write(ready, b"waiting").unwrap();
        let held = CredentialLocks::acquire_sources(vec![(path.clone(), resource)]).unwrap();
        held.verify_path(&path).unwrap();
        let current: CodexAuthJson = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        std::fs::write(result, current.tokens.unwrap().refresh_token.unwrap()).unwrap();
    }

    #[test]
    fn source_paths_remain_stable_after_creation_and_verify_each_identity() {
        let env = crate::test_support::TestCodexEnv::new();
        let path_a = env.codex_dir().join("missing").join("a.json");
        let missing = CredentialResource::path(&path_a);
        std::fs::create_dir_all(path_a.parent().unwrap()).unwrap();
        std::fs::write(&path_a, b"{}").unwrap();
        assert_eq!(CredentialResource::path(&path_a), missing);
        std::fs::write(
            &path_a,
            serde_json::to_vec(&auth("user-a", "refresh-a")).unwrap(),
        )
        .unwrap();
        let path_b = path_a.with_file_name("b.json");
        std::fs::write(
            &path_b,
            serde_json::to_vec(&auth("user-b", "refresh-b")).unwrap(),
        )
        .unwrap();
        let held = CredentialLocks::acquire_sources(vec![
            (path_a.clone(), CredentialResource::from_path(&path_a)),
            (path_b.clone(), CredentialResource::from_path(&path_b)),
        ])
        .unwrap();
        std::fs::write(&path_b, std::fs::read(&path_a).unwrap()).unwrap();
        assert!(held.verify_path(&path_a).is_ok());
        assert!(held.verify_path(&path_b).is_err());
    }

    #[test]
    fn credential_file_lock_waits_across_processes_and_rereads_new_version() {
        let env = crate::test_support::TestCodexEnv::new();
        let path = env.codex_dir().join("auth.json");
        let ready = env.home().join("lock-child-ready");
        let result = env.home().join("lock-child-result");
        std::fs::write(
            &path,
            serde_json::to_vec(&auth("cross-process-user", "old-refresh")).unwrap(),
        )
        .unwrap();
        let held = CredentialLocks::acquire(vec![CredentialResource::from_path(&path)]).unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "services::codex_auth_refresh_lock::tests::credential_lock_child",
            ])
            .env("CCR_LOCK_DIR", env.lock_dir())
            .env("CCR_TEST_CREDENTIAL_LOCK_SOURCE", &path)
            .env("CCR_TEST_CREDENTIAL_LOCK_READY", &ready)
            .env("CCR_TEST_CREDENTIAL_LOCK_RESULT", &result)
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let start = Instant::now();
        while !ready.exists() {
            if let Some(status) = child.try_wait().unwrap() {
                panic!("child exited before ready: {status}")
            }
            if start.elapsed() > Duration::from_secs(5) {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("child did not reach credential lock");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(!result.exists());
        std::fs::write(
            &path,
            serde_json::to_vec(&auth("cross-process-user", "new-refresh")).unwrap(),
        )
        .unwrap();
        drop(held);
        assert!(child.wait().unwrap().success());
        assert_eq!(std::fs::read(result).unwrap(), b"new-refresh");
    }
}
