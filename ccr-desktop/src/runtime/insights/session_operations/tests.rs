#![allow(clippy::unwrap_used)]
use super::*;
use crate::runtime::ConfigSource;
use ccr_app::runtime::{Platform, RuntimeOwner};
use std::time::Duration;

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
    trash.restore_sessions(["synthetic-session"]).unwrap();
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
