#![allow(clippy::unwrap_used)]
use super::*;
use std::path::PathBuf;

#[tokio::test]
async fn output_styles_follow_config_dir_without_touching_default_home() {
    const TEST: &str = "commands::claude::fixture_tests::output_styles_follow_config_dir_without_touching_default_home";
    let Some(root) = std::env::var_os("CCR_TEST_OUTPUT_STYLES_ROOT") else {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env("CCR_TEST_OUTPUT_STYLES_ROOT", &root)
            .env("CLAUDE_CONFIG_DIR", root.join("custom-claude"))
            // settings 备份覆盖只作陷阱：样式备份必须仍落在 CCR 根下的 Claude 备份目录。
            .env("CCR_BACKUP_DIR", root.join("settings-backups"))
            .env("CCR_ROOT", root.join("ccr"))
            .env("CCR_LOCK_DIR", root.join("locks"))
            .env("HOME", root.join("home"))
            .env("USERPROFILE", root.join("home"))
            .env_remove("CCR_SETTINGS_PATH")
            .env_remove("CLAUDE_JSON_PATH")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
        return;
    };
    let root = PathBuf::from(root);
    let styles = root.join("custom-claude/output-styles");
    let default_styles = root.join("home/.claude/output-styles");
    std::fs::create_dir_all(&styles).unwrap();
    std::fs::create_dir_all(&default_styles).unwrap();
    let original = b"default home must remain byte-identical";
    std::fs::write(default_styles.join("ambient.md"), original).unwrap();
    std::fs::write(styles.join("custom.md"), b"custom style").unwrap();
    std::fs::write(styles.join("kept.md"), b"omitted style retained").unwrap();
    let value: Value = claude_get_output_styles().await.unwrap().into();
    assert!(
        value["styles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|style| style["name"] == "custom")
    );
    assert!(
        !value["styles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|style| style["name"] == "ambient")
    );
    claude_update_output_styles(
        json!([
            {"name":"custom","content":"changed style"},
            {"name":"fresh","content":"created style"}
        ])
        .try_into()
        .unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(
        std::fs::read(styles.join("custom.md")).unwrap(),
        b"changed style"
    );
    assert_eq!(
        std::fs::read(styles.join("fresh.md")).unwrap(),
        b"created style"
    );
    let reread: Value = claude_get_output_styles().await.unwrap().into();
    assert!(
        reread["styles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|style| style["name"] == "fresh" && style["content"] == "created style")
    );
    // 覆盖已有样式会备份；备份目录与 CLAUDE_CONFIG_DIR 无关，保持原有 CCR 位置。
    let backups = root.join("ccr/backups/claude/output-styles");
    assert!(std::fs::read_dir(&backups).unwrap().next().is_some());
    assert!(!root.join("custom-claude/backups").exists());
    assert!(!root.join("settings-backups").exists());
    assert!(
        std::fs::read_dir(root.join("locks"))
            .unwrap()
            .next()
            .is_some()
    );
    assert!(!default_styles.join("fresh.md").exists());
    assert_eq!(
        std::fs::read(styles.join("kept.md")).unwrap(),
        b"omitted style retained"
    );
    assert_eq!(
        std::fs::read(default_styles.join("ambient.md")).unwrap(),
        original
    );
    assert!(!default_styles.join("custom.md").exists());
}

fn isolated_layout(test: &str) -> Option<Arc<crate::fixture::FixtureLayout>> {
    if let Some(root) = std::env::var_os("CCR_TAURI_LOCK_TEST_ROOT") {
        let layout =
            Arc::new(crate::fixture::FixtureLayout::load(PathBuf::from(root).as_path()).unwrap());
        crate::fixture::install(layout.clone()).unwrap();
        layout.prepare().unwrap();
        return Some(layout);
    }
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    std::fs::write(
        root.join("ccr-tauri-fixture.json"),
        br#"{"version":1,"fixture_id":"captured-locks"}"#,
    )
    .unwrap();
    let poison = root.join("refused-ambient-lock-directory");
    std::fs::write(&poison, b"ordinary file, never a lock directory").unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture"])
        .env("CCR_TAURI_LOCK_TEST_ROOT", &root)
        .env("CCR_LOCK_DIR", &poison)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    assert_eq!(
        std::fs::read(poison).unwrap(),
        b"ordinary file, never a lock directory"
    );
    None
}

#[tokio::test]
async fn captured_output_styles_ignore_ambient_roots_and_locks() {
    let Some(layout) = isolated_layout(
        "commands::claude::fixture_tests::captured_output_styles_ignore_ambient_roots_and_locks",
    ) else {
        return;
    };
    let styles = layout.roots.claude.config_dir.join("output-styles");
    std::fs::create_dir_all(&styles).unwrap();
    std::fs::write(styles.join("fixture.md"), b"captured style").unwrap();
    assert!(
        claude_get_output_styles()
            .await
            .unwrap_err()
            .contains("fixture_capability_unavailable")
    );
    let store = output_styles_store().unwrap();
    let styles_read = ccr_app::resources::claude_files::list_output_styles(&store).unwrap();
    assert!(styles_read.iter().any(|style| style.name == "fixture"));
    ccr_app::resources::claude_files::save_output_styles(
        &store,
        &[ccr_app::resources::claude_files::OutputStyle {
            name: "fixture".into(),
            content: "changed captured style".into(),
        }],
    )
    .unwrap();
    assert_eq!(
        std::fs::read(styles.join("fixture.md")).unwrap(),
        b"changed captured style"
    );
    assert!(
        std::fs::read_dir(&layout.roots.locks)
            .unwrap()
            .next()
            .is_some()
    );
}

#[tokio::test]
async fn captured_profile_crud_ignores_ambient_lock_directory() {
    let Some(layout) = isolated_layout(
        "commands::claude::fixture_tests::captured_profile_crud_ignores_ambient_lock_directory",
    ) else {
        return;
    };
    let context = layout.profile_context(Platform::Claude).unwrap();
    context
        .run(|| {
            let mut config = ccr_config::CcsConfig {
                default_config: "kept".into(),
                current_config: String::new(),
                settings: Default::default(),
                sections: Default::default(),
            };
            config.sections.insert(
                "kept".into(),
                ccr_config::ConfigSection {
                    base_url: Some("https://synthetic.invalid".into()),
                    auth_token: Some(ccr_core::Secret::new("synthetic-key")),
                    ..Default::default()
                },
            );
            context.manager().save(&config)
        })
        .unwrap();
    claude_add_profile(json!({"name":"temporary","base_url":"https://synthetic.invalid","auth_token":"synthetic-key"}).try_into().unwrap()).await.unwrap();
    claude_delete_profile("temporary".into()).await.unwrap();
    let profiles = crate::fixture::claude_platform()
        .unwrap()
        .load_profiles()
        .unwrap();
    assert!(profiles.contains_key("kept"));
    assert!(!profiles.contains_key("temporary"));
    assert!(
        std::fs::read_dir(&layout.roots.locks)
            .unwrap()
            .next()
            .is_some()
    );
}

#[tokio::test]
async fn captured_settings_mutation_ignores_ambient_lock_directory() {
    let Some(layout) = isolated_layout(
        "commands::claude::fixture_tests::captured_settings_mutation_ignores_ambient_lock_directory",
    ) else {
        return;
    };
    update_local_settings(|settings| {
        settings
            .other
            .insert("fixture-marker".into(), json!("selected-root"));
        Ok(())
    })
    .await
    .unwrap();
    let settings: Value =
        serde_json::from_slice(&std::fs::read(&layout.roots.claude.settings_file).unwrap())
            .unwrap();
    assert_eq!(settings["fixture-marker"], "selected-root");
    assert!(
        std::fs::read_dir(&layout.roots.locks)
            .unwrap()
            .next()
            .is_some()
    );
}
