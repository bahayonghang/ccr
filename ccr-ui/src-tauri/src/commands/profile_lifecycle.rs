use serde_json::{Value, json};
use std::path::Path;

use ccr_cli::application::profile_lifecycle::{
    ApplyProfileRequest, ProfileOutcome, ProfileStatus, apply_profile,
};
use ccr_config::PlatformPaths;

pub(crate) fn profile_outcome_payload(outcome: ProfileOutcome) -> Value {
    json!({
        "success": matches!(outcome.status, ProfileStatus::Applied | ProfileStatus::AppliedWithWarning),
        "applied_profile": outcome.profile,
        "message": outcome.message(),
        "outcome": outcome,
    })
}

/// Desktop service adapter; runs inside the handler's one blocking worker.
pub(crate) fn apply_profile_payload(request: ApplyProfileRequest) -> Result<Value, String> {
    apply_profile(request)
        .map(profile_outcome_payload)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod application_contract {
    use super::*;
    use ccr_cli::application::profile_contract;
    fn desktop(request: ApplyProfileRequest) -> ccr_core::Result<ProfileOutcome> {
        let payload = apply_profile_payload(request).map_err(ccr_core::CcrError::ConfigError)?;
        serde_json::from_value(payload["outcome"].clone())
            .map_err(|_| ccr_core::CcrError::ConfigError("Invalid outcome".into()))
    }
    #[test]
    fn profile_post_publish_failure_contract() {
        if ccr_cli::home_process::run_current_test() {
            return;
        }
        let _desktop = crate::test_support::lock_env();
        profile_contract::post_publish_failure(desktop);
    }
    #[test]
    fn profile_deleted_after_prepare_contract() {
        if ccr_cli::home_process::run_current_test() {
            return;
        }
        let _desktop = crate::test_support::lock_env();
        profile_contract::deleted_after_preparation(desktop);
    }
    #[test]
    fn profile_preflight_contract() {
        if ccr_cli::home_process::run_current_test() {
            return;
        }
        let _desktop = crate::test_support::lock_env();
        profile_contract::preflight(desktop);
    }
    #[test]
    fn profile_success_replay_contract() {
        if ccr_cli::home_process::run_current_test() {
            return;
        }
        let _desktop = crate::test_support::lock_env();
        profile_contract::success_and_replay(desktop);
    }
    #[test]
    fn profile_ancillary_contract() {
        if ccr_cli::home_process::run_current_test() {
            return;
        }
        let _desktop = crate::test_support::lock_env();
        profile_contract::ancillary_failures(desktop);
    }
    #[test]
    fn profile_each_write_contract() {
        if ccr_cli::home_process::run_current_test() {
            return;
        }
        let _desktop = crate::test_support::lock_env();
        profile_contract::failure_matrix(desktop);
    }
    #[test]
    fn profile_external_version_contract() {
        if ccr_cli::home_process::run_current_test() {
            return;
        }
        let _desktop = crate::test_support::lock_env();
        profile_contract::external_change(desktop);
    }
    #[test]
    fn profile_commit_failure_replays_operation() {
        if ccr_cli::home_process::run_current_test() {
            return;
        }
        let _desktop = crate::test_support::lock_env();
        profile_contract::commit_failure_replays(desktop);
    }
    #[test]
    fn profile_runtime_change_before_catalog_commit() {
        if ccr_cli::home_process::run_current_test() {
            return;
        }
        let _desktop = crate::test_support::lock_env();
        profile_contract::runtime_changed_before_catalog_commit(desktop);
    }
    #[test]
    fn profile_checkpoint_failure_keeps_activation() {
        if ccr_cli::home_process::run_current_test() {
            return;
        }
        let _desktop = crate::test_support::lock_env();
        profile_contract::checkpoint_failure_keeps_activation(desktop);
    }
}

use crate::commands::settings_raw::invalid_result;
use ccr_app::config::source::{self, SourceInvalidKind, SourceSaveOutcome};

pub(crate) fn resolve_profile_target_name(
    platform_label: &str,
    current_name: &str,
    request: &Value,
) -> Result<String, String> {
    let Some(raw) = request.get("name") else {
        return Ok(current_name.to_string());
    };

    let target = raw
        .as_str()
        .ok_or_else(|| format!("{platform_label} Profile 名称必须是字符串"))?
        .trim();

    if target.is_empty() {
        return Err(format!("{platform_label} Profile 名称不能为空"));
    }

    Ok(target.to_string())
}

pub(crate) fn profiles_export_payload_from_path(
    profiles_file: &Path,
    filename_prefix: &str,
    include_secrets: bool,
) -> Result<Value, String> {
    if !include_secrets {
        // Preserve the legacy wire error while the shared owner enforces the rule.
        return Err("Redacted profiles export is not supported".into());
    }
    let content =
        source::export(profiles_file, include_secrets).map_err(|error| error.to_string())?;
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("{filename_prefix}-{timestamp}.toml");

    Ok(json!({
        "content": content,
        "filename": filename,
    }))
}

pub(crate) fn profiles_raw_payload_from_paths(paths: &PlatformPaths) -> Result<Value, String> {
    let snapshot = source::read(paths).map_err(|error| error.to_string())?;
    Ok(json!({
        "status": "ok",
        "content": snapshot.content,
        "token": snapshot.token,
        "path": snapshot.path,
        "exists": snapshot.exists,
    }))
}

pub(crate) fn save_profiles_raw_to_paths(
    paths: &PlatformPaths,
    current_profile: Option<&str>,
    content: &str,
    token: &str,
    force: bool,
) -> Result<Value, String> {
    let outcome = source::save(paths, current_profile, content, token, force)
        .map_err(|error| error.to_string())?;
    Ok(match outcome {
        SourceSaveOutcome::Invalid {
            kind,
            message,
            position,
        } => invalid_result(
            match kind {
                SourceInvalidKind::Syntax => "syntax",
                SourceInvalidKind::Semantic => "semantic",
            },
            message,
            position,
        ),
        SourceSaveOutcome::ActivationConflict { current } => json!({
            "status": "activation_conflict", "current": current,
        }),
        SourceSaveOutcome::Conflict => json!({ "status": "conflict" }),
        SourceSaveOutcome::Saved {
            token,
            profiles_count,
        } => json!({
            "status": "saved", "token": token, "profiles_count": profiles_count,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;

    struct ScopedLockDir(Option<OsString>);

    impl ScopedLockDir {
        fn new(path: PathBuf) -> Self {
            let previous = std::env::var_os("CCR_LOCK_DIR");
            unsafe { std::env::set_var("CCR_LOCK_DIR", path) };
            Self(previous)
        }
    }

    impl Drop for ScopedLockDir {
        fn drop(&mut self) {
            unsafe {
                match self.0.take() {
                    Some(value) => std::env::set_var("CCR_LOCK_DIR", value),
                    None => std::env::remove_var("CCR_LOCK_DIR"),
                }
            }
        }
    }

    fn test_paths(root: &Path) -> PlatformPaths {
        PlatformPaths {
            root: root.to_path_buf(),
            registry_file: root.join("config.toml"),
            platform_dir: root.join("platforms").join("claude"),
            profiles_file: root.join("platforms").join("claude").join("profiles.toml"),
            settings_file: root.join("platforms").join("claude").join("settings.json"),
            history_file: root.join("history").join("claude.json"),
            backups_dir: root.join("backups").join("claude"),
        }
    }

    fn valid_profile(name: &str, model: &str) -> String {
        format!("[{name}]\nmodel = \"{model}\"\n")
    }

    #[test]
    fn target_name_defaults_to_current_name() {
        let target = resolve_profile_target_name("Codex", "current", &json!({})).unwrap();
        assert_eq!(target, "current");
    }

    #[test]
    fn target_name_rejects_blank_values() {
        let error =
            resolve_profile_target_name("Claude", "current", &json!({ "name": " " })).unwrap_err();
        assert!(error.contains("不能为空"));
    }

    #[test]
    fn profiles_raw_get_reports_missing_catalog_without_creating_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let paths = test_paths(temp_dir.path());

        let result = profiles_raw_payload_from_paths(&paths).unwrap();

        assert_eq!(result["status"], "ok");
        assert_eq!(result["content"], "");
        assert_eq!(result["token"], "");
        assert_eq!(result["exists"], false);
        assert_eq!(
            result["path"].as_str(),
            Some(paths.profiles_db().to_string_lossy().as_ref())
        );
        assert!(!paths.profiles_file.exists());
        assert!(!paths.profiles_db().exists());
    }

    #[test]
    fn profiles_raw_get_returns_canonical_toml_and_revision() {
        let _guard = crate::test_support::lock_env();
        let temp_dir = tempfile::tempdir().unwrap();
        let _lock_dir = ScopedLockDir::new(temp_dir.path().join("locks"));
        let paths = test_paths(temp_dir.path());
        fs::create_dir_all(&paths.platform_dir).unwrap();
        let saved = save_profiles_raw_to_paths(
            &paths,
            None,
            &valid_profile("active", "model-a"),
            "",
            false,
        )
        .unwrap();

        let result = profiles_raw_payload_from_paths(&paths).unwrap();

        assert_eq!(saved["status"], "saved");
        assert_eq!(result["status"], "ok");
        let content = result["content"].as_str().unwrap();
        assert!(content.contains("[active]"));
        assert!(content.contains("model = \"model-a\""));
        assert_eq!(result["token"], saved["token"]);
        assert_eq!(result["exists"], true);
        assert_eq!(
            result["path"].as_str(),
            Some(paths.profiles_db().to_string_lossy().as_ref())
        );
        assert!(!paths.profiles_file.exists());
    }

    #[test]
    fn profiles_raw_rejects_syntax_semantic_and_empty_content_without_leaks() {
        let temp_dir = tempfile::tempdir().unwrap();
        let paths = test_paths(temp_dir.path());
        let probe = "DO_NOT_LEAK_PROFILE_SECRET";

        let syntax = save_profiles_raw_to_paths(
            &paths,
            None,
            &format!("[broken\nauth_token = \"{probe}\""),
            "",
            false,
        )
        .unwrap();
        let semantic =
            save_profiles_raw_to_paths(&paths, None, &format!("profile = \"{probe}\""), "", false)
                .unwrap();
        let empty = save_profiles_raw_to_paths(&paths, None, "", "", false).unwrap();

        assert_eq!(syntax["status"], "invalid");
        assert_eq!(syntax["kind"], "syntax");
        assert!(syntax["line"].as_u64().is_some());
        assert_eq!(semantic["status"], "invalid");
        assert_eq!(semantic["kind"], "semantic");
        assert_eq!(empty["status"], "invalid");
        assert_eq!(empty["kind"], "semantic");
        for result in [&syntax, &semantic, &empty] {
            assert!(!result["message"].as_str().unwrap().contains(probe));
        }
        assert!(!paths.profiles_file.exists());
        assert!(!paths.profiles_db().exists());
        assert!(!paths.backups_dir.exists());
    }

    #[test]
    fn profiles_raw_requires_force_when_current_profile_disappears() {
        let _guard = crate::test_support::lock_env();
        let temp_dir = tempfile::tempdir().unwrap();
        let _lock_dir = ScopedLockDir::new(temp_dir.path().join("locks"));
        let paths = test_paths(temp_dir.path());
        fs::create_dir_all(&paths.platform_dir).unwrap();
        let original = valid_profile("active", "model-a");
        let replacement = valid_profile("replacement", "model-b");
        let seeded = save_profiles_raw_to_paths(&paths, None, &original, "", false).unwrap();
        assert_eq!(seeded["status"], "saved");
        let token = seeded["token"].as_str().unwrap();

        let blocked =
            save_profiles_raw_to_paths(&paths, Some("active"), &replacement, token, false).unwrap();
        assert_eq!(blocked["status"], "activation_conflict");
        assert_eq!(blocked["current"], "active");
        let kept = profiles_raw_payload_from_paths(&paths).unwrap();
        assert_eq!(kept["token"], token);
        assert!(kept["content"].as_str().unwrap().contains("model-a"));
        assert!(!paths.profiles_file.exists());
        assert!(!paths.backups_dir.exists());

        let saved =
            save_profiles_raw_to_paths(&paths, Some("active"), &replacement, token, true).unwrap();
        assert_eq!(saved["status"], "saved");
        assert_eq!(saved["profiles_count"], 1);
        let next = profiles_raw_payload_from_paths(&paths).unwrap();
        assert!(next["content"].as_str().unwrap().contains("model-b"));
        assert!(!next["content"].as_str().unwrap().contains("model-a"));
        assert!(!paths.profiles_file.exists());
        assert!(!paths.backups_dir.exists());
    }

    #[test]
    fn profiles_raw_rejects_stale_token_without_backup() {
        let _guard = crate::test_support::lock_env();
        let temp_dir = tempfile::tempdir().unwrap();
        let _lock_dir = ScopedLockDir::new(temp_dir.path().join("locks"));
        let paths = test_paths(temp_dir.path());
        fs::create_dir_all(&paths.platform_dir).unwrap();
        let original = valid_profile("active", "model-a");
        let external = valid_profile("active", "model-external");
        let seeded = save_profiles_raw_to_paths(&paths, None, &original, "", false).unwrap();
        let stale_token = seeded["token"].as_str().unwrap().to_string();
        let updated =
            save_profiles_raw_to_paths(&paths, Some("active"), &external, &stale_token, false)
                .unwrap();
        assert_eq!(updated["status"], "saved");

        let result = save_profiles_raw_to_paths(
            &paths,
            Some("active"),
            &valid_profile("active", "model-editor"),
            &stale_token,
            false,
        )
        .unwrap();

        assert_eq!(result["status"], "conflict");
        let kept = profiles_raw_payload_from_paths(&paths).unwrap();
        assert_eq!(kept["token"], updated["token"]);
        assert!(kept["content"].as_str().unwrap().contains("model-external"));
        assert!(!kept["content"].as_str().unwrap().contains("model-editor"));
        assert!(!paths.profiles_file.exists());
        assert!(!paths.backups_dir.exists());
    }

    #[cfg(unix)]
    #[test]
    fn profiles_raw_save_keeps_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = crate::test_support::lock_env();
        let temp_dir = tempfile::tempdir().unwrap();
        let _lock_dir = ScopedLockDir::new(temp_dir.path().join("locks"));
        let paths = test_paths(temp_dir.path());

        let result = save_profiles_raw_to_paths(
            &paths,
            None,
            &valid_profile("active", "model-a"),
            "",
            false,
        )
        .unwrap();

        assert_eq!(result["status"], "saved");
        assert!(!paths.profiles_file.exists());
        assert_eq!(
            fs::metadata(paths.profiles_db())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
