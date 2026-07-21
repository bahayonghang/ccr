use serde_json::{Value, json};
use std::fs;
use std::path::Path;

use ccr_config::{PlatformPaths, parse_profiles_from_str};

use crate::commands::settings_raw::{
    invalid_result, read_raw_file, toml_error_position, write_raw_file_versioned,
};

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
        return Err("Redacted profiles export is not supported".to_string());
    }

    let content = fs::read_to_string(profiles_file)
        .map_err(|e| format!("Failed to read profiles.toml: {e}"))?;
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("{filename_prefix}-{timestamp}.toml");

    Ok(json!({
        "content": content,
        "filename": filename,
    }))
}

pub(crate) fn profiles_raw_payload_from_paths(paths: &PlatformPaths) -> Result<Value, String> {
    read_raw_file(&paths.profiles_file)
}

pub(crate) fn save_profiles_raw_to_paths(
    paths: &PlatformPaths,
    current_profile: Option<&str>,
    content: &str,
    token: &str,
    force: bool,
) -> Result<Value, String> {
    if let Err(error) = toml::from_str::<toml::Value>(content) {
        return Ok(invalid_result(
            "syntax",
            "Invalid profiles TOML syntax",
            toml_error_position(content, &error),
        ));
    }

    let profiles = match parse_profiles_from_str(content) {
        Ok(profiles) => profiles,
        Err(_) => {
            return Ok(invalid_result(
                "semantic",
                "Profiles TOML does not match the supported profile structure",
                None,
            ));
        }
    };
    if profiles.is_empty() {
        return Ok(invalid_result(
            "semantic",
            "Profiles TOML must contain at least one profile",
            None,
        ));
    }

    if let Some(current) = current_profile
        && !profiles.contains_key(current)
        && !force
    {
        return Ok(json!({
            "status": "activation_conflict",
            "current": current,
        }));
    }

    // CAS covers concurrent structured/raw writers without nesting the profile RMW lock.
    let mut result = write_raw_file_versioned(
        &paths.profiles_file,
        &paths.backups_dir,
        "profiles",
        content,
        token,
        true,
    )?;
    if result["status"] == "saved" {
        result["profiles_count"] = json!(profiles.len());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccr_core::core::content_version_token;
    use std::ffi::OsString;
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
    fn profiles_raw_get_reports_verbatim_content_and_token() {
        let temp_dir = tempfile::tempdir().unwrap();
        let paths = test_paths(temp_dir.path());
        fs::create_dir_all(&paths.platform_dir).unwrap();
        let content = valid_profile("active", "model-a");
        fs::write(&paths.profiles_file, &content).unwrap();

        let result = profiles_raw_payload_from_paths(&paths).unwrap();

        assert_eq!(result["status"], "ok");
        assert_eq!(result["content"], content);
        assert_eq!(result["token"], content_version_token(content.as_bytes()));
        assert_eq!(
            result["path"].as_str(),
            Some(paths.profiles_file.to_string_lossy().as_ref())
        );
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
        fs::write(&paths.profiles_file, &original).unwrap();
        let token = content_version_token(original.as_bytes());

        let blocked =
            save_profiles_raw_to_paths(&paths, Some("active"), &replacement, &token, false)
                .unwrap();
        assert_eq!(blocked["status"], "activation_conflict");
        assert_eq!(blocked["current"], "active");
        assert_eq!(fs::read_to_string(&paths.profiles_file).unwrap(), original);

        let saved =
            save_profiles_raw_to_paths(&paths, Some("active"), &replacement, &token, true).unwrap();
        assert_eq!(saved["status"], "saved");
        assert_eq!(saved["profiles_count"], 1);
        assert_eq!(
            fs::read_to_string(&paths.profiles_file).unwrap(),
            replacement
        );
        assert_eq!(fs::read_dir(&paths.backups_dir).unwrap().count(), 1);
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
        fs::write(&paths.profiles_file, &original).unwrap();
        let stale_token = content_version_token(original.as_bytes());
        fs::write(&paths.profiles_file, &external).unwrap();

        let result = save_profiles_raw_to_paths(
            &paths,
            Some("active"),
            &valid_profile("active", "model-editor"),
            &stale_token,
            false,
        )
        .unwrap();

        assert_eq!(result["status"], "conflict");
        assert_eq!(fs::read_to_string(&paths.profiles_file).unwrap(), external);
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
        assert_eq!(
            fs::metadata(&paths.profiles_file)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
