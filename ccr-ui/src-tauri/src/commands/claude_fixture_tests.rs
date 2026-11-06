#![allow(clippy::unwrap_used)]
use super::*;

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
