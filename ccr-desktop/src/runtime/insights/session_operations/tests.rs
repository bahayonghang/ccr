#![allow(clippy::unwrap_used)]
use super::*;
use crate::runtime::ConfigSource;
use ccr_app::runtime::{Platform, RuntimeOwner};
use std::time::Duration;

#[test]
fn captured_non_codex_scope_rejects_trash_and_prepared_commands_without_side_effects() {
    let (root, owner, services, path, id) = fixture();
    let global = InsightsRequests::new(&services);
    let prepared = global.prepare_session(&id).unwrap();
    let claude = global.clone().scoped(Some(Platform::Claude));
    let before = std::fs::read(&path).unwrap();
    assert!(matches!(
        claude.read_session_trash(),
        Err(Failure::Unsupported)
    ));
    assert!(matches!(
        claude.prepare_session(&id),
        Err(Failure::Unsupported)
    ));
    assert!(matches!(
        claude.run_session_operation(&prepared, Action::Trash),
        Err(Failure::Unsupported)
    ));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(!root.path().join("home/.codex/.ccr-trash").exists());
    assert!(global.read_session_trash().unwrap().is_empty());
    owner.shutdown();
}

pub(crate) fn fixture() -> (tempfile::TempDir, RuntimeOwner, Services, PathBuf, String) {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let codex = home.join(".codex");
    let path = codex.join("sessions/2026/11/06/rollout-fixture.jsonl");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let content = "{\"timestamp\":\"2026-11-06T00:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"synthetic-session\",\"cwd\":\"/synthetic\"}}\n{\"timestamp\":\"2026-11-06T00:00:01Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"private synthetic transcript\"}]}}\n";
    std::fs::write(&path, content).unwrap();
    let archive = root.path().join("archive.db");
    let pool = database::create_pool(&archive, None).unwrap();
    database::migrations::run_all_migrations(&pool.get().unwrap(), root.path()).unwrap();
    let registry = AgentSessionProviderRegistry::new(home.clone()).without_environment_overrides();
    sessions::refresh::refresh_archive_with_registry(&pool, &registry).unwrap();
    let id = sessions::list_sessions(&pool, SessionListRequest::default())
        .unwrap()
        .items[0]
        .archive_id
        .clone();
    let owner = RuntimeOwner::new().unwrap();
    let services = Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        },
    )
    .with_insights_paths(InsightsPaths {
        usage_root: root.path().join("usage"),
        archive_db: archive,
        provider_home: home,
    });
    (root, owner, services, path, id)
}

#[test]
fn opaque_codex_actions_reject_conflicts_and_keep_trash_recoverable() {
    let (root, owner, services, path, id) = fixture();
    let codex = root.path().join("home/.codex");
    let content = std::fs::read_to_string(&path).unwrap();
    let requests = InsightsRequests::new(&services);
    let pool = requests
        .archive_pool(requests.paths.as_ref().unwrap())
        .unwrap();
    let prepared = requests.prepare_session(&id).unwrap();
    std::fs::write(&path, format!("{content}\n")).unwrap();
    assert!(matches!(
        requests.run_session_operation(&prepared, Action::Trash),
        Err(Failure::Conflict)
    ));
    assert!(path.exists());
    assert!(
        !CodexSessionTrashService::with_codex_home(&codex)
            .trash_root()
            .exists()
    );
    std::fs::write(&path, &content).unwrap();
    let cloned = requests
        .run_session_operation(&prepared, Action::Clone)
        .unwrap();
    assert!(cloned.session_id.starts_with("clone-"));
    assert!(cloned.archive_updated);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
    let exported = CodexSessionService::new(codex.clone())
        .export_session_markdown(&path, None)
        .unwrap();
    assert!(exported.content.contains("private synthetic transcript"));
    let result = requests
        .run_session_operation(&prepared, Action::Trash)
        .unwrap();
    assert!(result.archive_updated);
    assert!(!path.exists());
    let records = sessions::list_sessions(&pool, SessionListRequest::default()).unwrap();
    assert_eq!(
        records
            .items
            .iter()
            .find(|row| row.archive_id == id)
            .unwrap()
            .source_state
            .as_str(),
        "deleted_by_user"
    );
    let trash = CodexSessionTrashService::with_codex_home(&codex);
    assert_eq!(trash.list_trashed_sessions().unwrap().len(), 1);
    LockManager::with_scoped_directory(&root.path().join("locks"), || {
        trash.restore_sessions(["synthetic-session"])
    })
    .unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
    let key = RequestKey {
        environment: EnvironmentId::Ssh("fixture".into()),
        platform: Platform::Codex,
        generation: 1,
        request_id: 1,
    };
    assert_eq!(
        requests.start_session_operation(key, prepared, Action::Trash),
        Err(Failure::LocalOnly)
    );
    assert!(requests.prepare_session("arbitrary/path.jsonl").is_err());
    owner.shutdown();
}

#[test]
fn explicit_markdown_save_is_private_new_file_only_and_fixture_bounded() {
    let (root, owner, services, path, id) = fixture();
    let requests = InsightsRequests::new(&services);
    let export = Arc::new(
        CodexSessionService::new(root.path().join("home/.codex"))
            .export_session_markdown(&path, None)
            .unwrap(),
    );
    let target = root.path().join("export.md");
    requests.save_session_export(&id, &export, &target).unwrap();
    assert_eq!(std::fs::read_to_string(&target).unwrap(), export.content);
    #[cfg(windows)]
    {
        // Native dialogs return ordinary paths even when captured service paths
        // have the verbatim prefix emitted by filesystem canonicalization.
        let mut canonical_paths = requests.clone();
        canonical_paths.paths.as_mut().unwrap().archive_db =
            root.path().join("archive.db").canonicalize().unwrap();
        canonical_paths
            .save_session_export(&id, &export, &root.path().join("native-dialog-path.md"))
            .unwrap();
    }
    assert!(matches!(
        requests.save_session_export(&id, &export, &target),
        Err(Failure::Conflict)
    ));
    assert!(matches!(
        requests.save_session_export(&id, &export, &root.path().join("../escape.md")),
        Err(Failure::Storage)
    ));
    let outside = tempfile::tempdir().unwrap();
    assert!(matches!(
        requests.save_session_export(&id, &export, &outside.path().join("escape.md")),
        Err(Failure::Storage)
    ));
    let key = RequestKey {
        environment: EnvironmentId::Ssh("fixture".into()),
        platform: Platform::Codex,
        generation: 1,
        request_id: 1,
    };
    assert_eq!(
        requests.start_session_save(key, id, export, root.path().join("remote.md")),
        Err(Failure::LocalOnly)
    );
    assert!(!root.path().join("remote.md").exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    // Windows junctions need no developer-mode symlink privilege.
    let link = root.path().join("linked");
    #[cfg(windows)]
    {
        let status = std::process::Command::new("cmd")
            .args(["/c", "mklink", "/J"])
            .arg(&link)
            .arg(outside.path())
            .status()
            .unwrap();
        assert!(status.success());
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(outside.path(), &link).unwrap();
    assert!(matches!(
        requests.save_session_export(
            "id",
            &CodexSessionExport {
                session_id: "fixture".into(),
                file_name: "x.md".into(),
                content: "private".into(),
                truncated: false,
                max_messages: 1
            },
            &link.join("escape.md")
        ),
        Err(Failure::Storage)
    ));
    assert!(!outside.path().join("escape.md").exists());
    #[cfg(windows)]
    std::fs::remove_dir(&link).unwrap();
    owner.shutdown();
}

#[test]
fn reviewed_trash_restores_exact_payload_and_index_then_refreshes_captured_archive() {
    let (root, owner, services, path, id) = fixture();
    let requests = InsightsRequests::new(&services);
    let original = std::fs::read(&path).unwrap();
    let prepared = requests.prepare_session(&id).unwrap();
    requests
        .run_session_operation(&prepared, Action::Trash)
        .unwrap();
    let item = requests.read_session_trash().unwrap().remove(0);
    assert_eq!(item.session_id, "synthetic-session");
    let reviewed = requests
        .prepare_session_restore_item(item.clone(), || false)
        .unwrap();
    let key = RequestKey {
        environment: EnvironmentId::Ssh("fixture".into()),
        platform: Platform::Codex,
        generation: 1,
        request_id: 1,
    };
    assert_eq!(
        requests.start_session_restore(key, reviewed.clone()),
        Err(Failure::LocalOnly)
    );
    assert!(!path.exists());
    let trash = CodexSessionTrashService::with_codex_home(&item.codex_home);
    let manifest = trash.list_trashed_sessions().unwrap()[0]
        .trash_dir
        .join("manifest.json");
    let original_manifest = std::fs::read(&manifest).unwrap();
    let mut changed_manifest: serde_json::Value =
        serde_json::from_slice(&original_manifest).unwrap();
    changed_manifest["title"] = "changed title".into();
    std::fs::write(&manifest, serde_json::to_vec(&changed_manifest).unwrap()).unwrap();
    assert!(matches!(
        requests.restore_session(&reviewed),
        Err(Failure::Conflict)
    ));
    std::fs::write(&manifest, original_manifest).unwrap();
    let rollout = trash.list_trashed_sessions().unwrap()[0]
        .trash_dir
        .join("rollout.jsonl");
    std::fs::write(&rollout, b"changed since review").unwrap();
    assert!(matches!(
        requests.restore_session(&reviewed),
        Err(Failure::Conflict)
    ));
    assert!(!path.exists());
    std::fs::write(&rollout, &original).unwrap();
    std::fs::write(&path, b"new active rollout").unwrap();
    assert!(requests.restore_session(&reviewed).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"new active rollout");
    std::fs::remove_file(&path).unwrap();
    let index = item.codex_home.join("session_index.jsonl");
    std::fs::write(
        &index,
        "{\"id\":\"synthetic-session\",\"thread_name\":\"active\"}\n",
    )
    .unwrap();
    assert!(requests.restore_session(&reviewed).is_err());
    assert!(!path.exists());
    std::fs::write(&index, "").unwrap();
    let result = requests.restore_session(&reviewed).unwrap();
    assert!(result.archive_updated);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert!(
        std::fs::read_to_string(&index)
            .unwrap()
            .contains("synthetic-session")
    );
    assert!(requests.read_session_trash().unwrap().is_empty());
    let pool = requests
        .archive_pool(requests.paths.as_ref().unwrap())
        .unwrap();
    let rows = sessions::list_sessions(&pool, SessionListRequest::default()).unwrap();
    assert_ne!(
        rows.items
            .iter()
            .find(|row| row.archive_id == id)
            .unwrap()
            .source_state
            .as_str(),
        "deleted_by_user"
    );
    assert!(!root.path().join("outside").exists());
    owner.shutdown();
}

#[test]
fn restore_is_home_bound_and_reports_archive_failure_after_successful_restore() {
    let (root, owner, services, path, id) = fixture();
    let requests = InsightsRequests::new(&services);
    let prepared = requests.prepare_session(&id).unwrap();
    requests
        .run_session_operation(&prepared, Action::Trash)
        .unwrap();
    let item = requests.read_session_trash().unwrap().remove(0);
    let reviewed = requests
        .prepare_session_restore_item(item.clone(), || false)
        .unwrap();
    let mut other = requests.clone();
    other.paths.as_mut().unwrap().provider_home = root.path().join("other-home");
    let mut substituted = (*reviewed).clone();
    substituted.item.codex_home = other.paths.as_ref().unwrap().provider_home.join(".codex");
    assert!(matches!(
        other.restore_session(&substituted),
        Err(Failure::Conflict)
    ));
    assert!(!path.exists());
    let mut unavailable_archive = requests.clone();
    let broken = root.path().join("missing-archive.db");
    unavailable_archive.paths.as_mut().unwrap().archive_db = broken;
    unavailable_archive.archive = Default::default();
    let result = unavailable_archive.restore_session(&reviewed).unwrap();
    assert_eq!(result.action, Action::Restore);
    assert!(!result.archive_updated);
    assert!(path.exists());
    assert!(requests.read_session_trash().unwrap().is_empty());
    owner.shutdown();
}

#[test]
fn queued_restore_review_cancellation_is_observed_without_admitting_a_writer() {
    let (_root, owner, services, path, id) = fixture();
    let requests = InsightsRequests::new(&services);
    let prepared = requests.prepare_session(&id).unwrap();
    requests
        .run_session_operation(&prepared, Action::Trash)
        .unwrap();
    let item = requests.read_session_trash().unwrap().remove(0);
    let gate = Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new()));
    let (entered, receive) = std::sync::mpsc::channel();
    let key = |request_id| RequestKey {
        environment: EnvironmentId::Local("fixture".into()),
        platform: Platform::Codex,
        generation: 1,
        request_id,
    };
    let mut holders = Vec::new();
    for index in 0..4 {
        let gate = gate.clone();
        let entered = entered.clone();
        holders.push(
            services
                .executor
                .read(key(index), move |_, _| {
                    entered.send(()).unwrap();
                    let (lock, changed) = &*gate;
                    let mut released = lock.lock().unwrap();
                    while !*released {
                        released = changed.wait(released).unwrap();
                    }
                    Ok::<_, Failure>(())
                })
                .unwrap(),
        );
    }
    for _ in 0..4 {
        receive.recv_timeout(Duration::from_secs(10)).unwrap();
    }
    let mut review = requests.prepare_session_restore(key(5), item).unwrap();
    review.request_cancel();
    let (lock, changed) = &*gate;
    *lock.lock().unwrap() = true;
    changed.notify_all();
    let wait = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    wait.block_on(async {
        let event=tokio::time::timeout(Duration::from_secs(10),review.next()).await.unwrap().unwrap();
        assert!(matches!(event,ccr_app::runtime::TaskEvent::Terminal(value) if matches!(value.as_ref(),Completion::Cancelled)));
    });
    assert!(!path.exists());
    assert!(!requests.session_operation_snapshot().busy);
    assert_eq!(requests.read_session_trash().unwrap().len(), 1);
    drop(holders);
    drop(wait);
    owner.shutdown();
}
