#![allow(clippy::unwrap_used)]
use super::*;
use serde_json::json;
fn store(root: &Path, kind: Kind) -> Store {
    Store::new(
        root.join("resources"),
        root.join("backups"),
        root.join("locks"),
        kind,
    )
}
#[test]
fn opencode_patch_and_raw_cas_preserve_unknown_fields_and_backup_delete() {
    let temp = tempfile::tempdir().unwrap();
    let store = store(temp.path(), Kind::OpenCodeAgent);
    let new = store.open("review.md").unwrap();
    new.save("---\ndescription: Review\ncustom_flag: true\n---\n\nOld instructions\n")
        .unwrap();
    let first = store.open("review.md").unwrap();
    let stale = store.open("review.md").unwrap();
    let changed = opencode::patch(
        first.content(),
        json!({"description":"Updated","body":"New instructions"})
            .as_object()
            .unwrap(),
        true,
    )
    .unwrap();
    first.save(&changed).unwrap();
    assert_eq!(
        stale.save("---\ndescription: stale\n---\nold"),
        Err(Error::Conflict)
    );
    assert_eq!(stale.delete(), Err(Error::Conflict));
    let current = store.open("review.md").unwrap();
    let document = opencode::parse_markdown_document(current.content()).unwrap();
    assert_eq!(
        opencode::yaml_bool(&document.frontmatter, "custom_flag"),
        Some(true)
    );
    assert_eq!(document.body, "New instructions");
    current.delete().unwrap();
    assert!(!current.path().exists());
    assert!(
        fs::read_dir(temp.path().join("backups"))
            .unwrap()
            .any(|entry| fs::read_to_string(entry.unwrap().path())
                .is_ok_and(|text| text.contains("New instructions")))
    );
}
#[test]
fn antigravity_nested_commands_preserve_toml_and_reject_path_escape() {
    let temp = tempfile::tempdir().unwrap();
    let store = store(temp.path(), Kind::AntigravityCommand);
    let new = store.open("team/review.toml").unwrap();
    new.save("prompt = 'Inspect'\ndescription = 'Review'\n[future]\nflag = true\n")
        .unwrap();
    let current = store.open("team/review.toml").unwrap();
    let next = antigravity::patch(
        current.content(),
        &json!({"command":"New prompt","description":"New description"}),
    )
    .unwrap();
    current.save(&next).unwrap();
    let value = antigravity::validate(store.open("team/review.toml").unwrap().content()).unwrap();
    assert_eq!(value["future"]["flag"].as_bool(), Some(true));
    assert_eq!(store.list().unwrap()[0].key, "team/review.toml");
    for name in [
        "../outside.toml",
        "/outside.toml",
        "team/../../outside.toml",
        "C:/outside.toml",
        "team\\escape.toml",
    ] {
        assert!(matches!(store.open(name), Err(Error::InvalidName)));
    }
    assert!(!temp.path().join("outside.toml").exists());
}
#[test]
fn extension_inventory_prefers_existing_manifest_and_remains_read_only() {
    let temp = tempfile::tempdir().unwrap();
    let store = store(temp.path(), Kind::Extension);
    assert!(store.list().unwrap().is_empty());
    assert!(!temp.path().join("resources").exists());
    let directory = temp.path().join("resources/example");
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("package.json"),
        r#"{"description":"Package fallback","version":"1"}"#,
    )
    .unwrap();
    assert_eq!(
        store.list().unwrap()[0].description.as_deref(),
        Some("Package fallback")
    );
    fs::write(
        directory.join("gemini-extension.json"),
        r#"{"description":"Preferred","version":"2","unknown":true}"#,
    )
    .unwrap();
    let row = store.list().unwrap().remove(0);
    assert_eq!(row.version.as_deref(), Some("2"));
    let session = store.open(&row.key).unwrap();
    assert!(session.content().contains("unknown"));
    assert_eq!(session.save("{}"), Err(Error::ReadOnly));
    assert_eq!(session.delete(), Err(Error::ReadOnly));
    assert!(session.path().exists());
}
#[cfg(unix)]
#[test]
fn linked_ancestor_cannot_redirect_resource_reads_or_writes() {
    let temp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), temp.path().join("resources")).unwrap();
    let store = store(temp.path(), Kind::AntigravityCommand);
    assert!(matches!(store.open("escape.toml"), Err(Error::UnsafePath)));
    assert!(matches!(store.list(), Err(Error::UnsafePath)));
    assert!(!outside.path().join("escape.toml").exists());
}

#[test]
fn plugin_inventory_reads_metadata_only_and_disallows_source() {
    let temp = tempfile::tempdir().unwrap();
    let store = store(temp.path(), Kind::OpenCodePlugin);
    fs::create_dir_all(store.directory()).unwrap();
    for name in ["a.js", "b.cjs", "c.mjs", "d.TS"] {
        fs::write(store.directory().join(name), [0xff, 0xfe]).unwrap();
    }
    fs::write(store.directory().join("README.md"), "ignore").unwrap();
    let rows = store.list().unwrap();
    assert_eq!(rows.len(), 4);
    assert!(rows.iter().all(|row| row.bytes == 2));
    assert!(matches!(store.open("a.js"), Err(Error::ReadOnly)));
    assert!(!Kind::OpenCodePlugin.writable());
}

#[cfg(any(unix, windows))]
#[test]
fn redirected_backup_or_lock_directory_cannot_escape_captured_storage() {
    let temp = tempfile::tempdir().unwrap();
    let outside = temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    let redirected = temp.path().join("redirected");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, &redirected).unwrap();
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let result = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", "$ErrorActionPreference = 'Stop'; New-Item -ItemType Junction -Path $env:CCR_FIXTURE_LINK -Value $env:CCR_FIXTURE_TARGET | Out-Null"])
            .env("CCR_FIXTURE_LINK", &redirected)
            .env("CCR_FIXTURE_TARGET", &outside)
            .creation_flags(0x08000000)
            .output()
            .unwrap();
        assert!(result.status.success(), "fixture junction creation failed");
    }
    let safe = store(temp.path(), Kind::AntigravityCommand);
    safe.open("review.toml")
        .unwrap()
        .save("prompt = 'Original'\n")
        .unwrap();
    for (backups, locks) in [
        (redirected.clone(), temp.path().join("locks")),
        (temp.path().join("backups"), redirected.clone()),
    ] {
        let unsafe_store = Store::new(
            safe.directory().to_path_buf(),
            backups,
            locks,
            Kind::AntigravityCommand,
        );
        let session = unsafe_store.open("review.toml").unwrap();
        assert_eq!(session.save("prompt = 'Changed'\n"), Err(Error::UnsafePath));
        assert_eq!(session.delete(), Err(Error::UnsafePath));
        assert_eq!(
            fs::read_to_string(session.path()).unwrap(),
            "prompt = 'Original'\n"
        );
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    }
}
