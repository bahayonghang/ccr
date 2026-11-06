#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use ccr_db::database::{
    create_pool,
    migrations::run_all_migrations,
    repositories::usage_repo::{
        UsageSessionArchiveEntry, UsageSourceState, upsert_session_archive_entry,
    },
};
use std::path::Path;

fn pool(root: &Path) -> DbPool {
    let pool = create_pool(&root.join("archive.db"), None).unwrap();
    run_all_migrations(&pool.get().unwrap(), root).unwrap();
    pool
}

#[test]
fn refresh_skips_unchanged_sources_and_reconciles_missing_without_modifying_provider() {
    let temp = tempfile::tempdir().unwrap();
    let archive = pool(temp.path());
    let source = temp.path().join(".pi/agent/sessions/project/s1.jsonl");
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    let content = "{\"type\":\"message\",\"message\":{\"role\":\"user\",\"content\":\"synthetic refresh\"}}\n";
    std::fs::write(&source, content).unwrap();
    let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf())
        .without_environment_overrides();
    let first = refresh::refresh_archive_with_registry(&archive, &registry).unwrap();
    assert_eq!(first.total.parsed, 1);
    assert_eq!(first.total.upserted, 1);
    assert_eq!(first.providers.len(), 8);
    let second = refresh::refresh_archive_with_registry(&archive, &registry).unwrap();
    assert_eq!(second.total.discovered, 1);
    assert_eq!(second.total.unchanged, 1);
    assert_eq!(second.total.parsed, 0);
    assert_eq!(second.total.upserted, 0);
    assert_eq!(std::fs::read_to_string(&source).unwrap(), content);
    std::fs::remove_file(source).unwrap();
    refresh::refresh_archive_with_registry(&archive, &registry).unwrap();
    let page = list_sessions(&archive, SessionListRequest::default()).unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].source_state.as_str(), "missing");
}

#[test]
fn refresh_reactivates_restored_bytes_and_mtime_without_reviving_absent_deleted_sources() {
    let temp = tempfile::tempdir().unwrap();
    let archive = pool(temp.path());
    let root = temp.path().join(".pi/agent/sessions/project");
    std::fs::create_dir_all(&root).unwrap();
    let source = root.join("restored.jsonl");
    let absent = root.join("absent.jsonl");
    let content = "{\"type\":\"message\",\"message\":{\"role\":\"user\",\"content\":\"restorable fixture\"}}\n";
    for path in [&source, &absent] {
        std::fs::write(path, content).unwrap();
    }
    let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf())
        .without_environment_overrides();
    assert_eq!(
        refresh::refresh_archive_with_registry(&archive, &registry)
            .unwrap()
            .total
            .parsed,
        2
    );
    let original_mtime = std::fs::metadata(&source).unwrap().modified().unwrap();
    let held = temp.path().join("trash-restored");
    std::fs::rename(&source, &held).unwrap();
    std::fs::remove_file(&absent).unwrap();
    let conn = archive.get().unwrap();
    let rows = usage_repo::get_all_agent_session_archive_sources(&conn).unwrap();
    for row in &rows {
        usage_repo::mark_session_archive_deleted_by_path(&conn, &row.platform, &row.file_path)
            .unwrap();
    }
    drop(conn);
    refresh::refresh_archive_with_registry(&archive, &registry).unwrap();
    std::fs::rename(&held, &source).unwrap();
    assert_eq!(
        std::fs::metadata(&source).unwrap().modified().unwrap(),
        original_mtime
    );
    let restored = refresh::refresh_archive_with_registry(&archive, &registry).unwrap();
    assert_eq!(restored.total.parsed, 1);
    let page = list_sessions(&archive, SessionListRequest::default()).unwrap();
    assert_eq!(
        page.items
            .iter()
            .filter(|row| row.source_state == UsageSourceState::Live)
            .count(),
        1
    );
    assert_eq!(
        page.items
            .iter()
            .filter(|row| row.source_state == UsageSourceState::DeletedByUser)
            .count(),
        1
    );
    assert_eq!(
        refresh::refresh_archive_with_registry(&archive, &registry)
            .unwrap()
            .total
            .unchanged,
        1
    );
    // The same cache rule applies to a temporarily missing, non-deleted file.
    std::fs::rename(&source, &held).unwrap();
    refresh::refresh_archive_with_registry(&archive, &registry).unwrap();
    std::fs::rename(&held, &source).unwrap();
    assert_eq!(
        refresh::refresh_archive_with_registry(&archive, &registry)
            .unwrap()
            .total
            .parsed,
        1
    );
}

#[test]
fn refresh_reactivates_deleted_sqlite_members_before_skipping_unchanged_containers() {
    let temp = tempfile::tempdir().unwrap();
    let archive = pool(temp.path());
    let root = temp.path().join(".local/share/opencode");
    std::fs::create_dir_all(&root).unwrap();
    let source = root.join("opencode.db");
    let conn = rusqlite::Connection::open(&source).unwrap();
    conn.execute_batch("CREATE TABLE session(id TEXT PRIMARY KEY, title TEXT, directory TEXT, time_created INTEGER, time_updated INTEGER); CREATE TABLE message(id TEXT PRIMARY KEY, session_id TEXT, data TEXT, time_created INTEGER); CREATE TABLE part(id TEXT PRIMARY KEY, session_id TEXT, message_id TEXT, data TEXT, time_created INTEGER); INSERT INTO session VALUES('s1','Fixture one','/fixture',1000,2000); INSERT INTO session VALUES('s2','Fixture two','/fixture',1000,2000);").unwrap();
    drop(conn);
    let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf())
        .without_environment_overrides();
    assert_eq!(
        refresh::refresh_archive_with_registry(&archive, &registry)
            .unwrap()
            .total
            .parsed,
        2
    );
    assert_eq!(
        refresh::refresh_archive_with_registry(&archive, &registry)
            .unwrap()
            .total
            .unchanged,
        2
    );
    let before = std::fs::read(&source).unwrap();
    let conn = archive.get().unwrap();
    let rows = usage_repo::get_all_agent_session_archive_sources(&conn).unwrap();
    usage_repo::mark_session_archive_deleted_by_path(&conn, "opencode", &rows[0].file_path)
        .unwrap();
    let marked = usage_repo::get_agent_session_archive_source(&conn, &rows[0].archive_id)
        .unwrap()
        .unwrap();
    assert_eq!(marked.source_state, UsageSourceState::DeletedByUser);
    drop(conn);
    assert_eq!(
        refresh::refresh_archive_with_registry(&archive, &registry)
            .unwrap()
            .total
            .parsed,
        2
    );
    assert!(
        list_sessions(&archive, SessionListRequest::default())
            .unwrap()
            .items
            .iter()
            .all(|row| row.source_state == UsageSourceState::Live)
    );
    assert_eq!(
        refresh::refresh_archive_with_registry(&archive, &registry)
            .unwrap()
            .total
            .unchanged,
        2
    );
    assert_eq!(std::fs::read(&source).unwrap(), before);
}
fn seed(pool: &DbPool, source: &Path, agent: &str, fidelity: &str, index: usize) -> String {
    let now = DateTime::parse_from_rfc3339("2026-09-24T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let path = source.to_string_lossy().to_string();
    let id = usage_repo::agent_session_archive_id(agent, &path, "");
    let entry = UsageSessionArchiveEntry {
        archive_id: id.clone(),
        session_id: format!("session-{index}"),
        platform: agent.into(),
        title: Some(format!("Fixture {index}")),
        cwd: String::new(),
        file_path: path,
        file_hash: None,
        source_variant: format!("{agent}-live"),
        source_kind: "file".into(),
        source_member_id: String::new(),
        source_size: None,
        source_mtime_ns: None,
        source_stat_hash: None,
        message_count: 250,
        user_message_count: 250,
        assistant_message_count: 0,
        tool_use_count: 0,
        source_fidelity: fidelity.into(),
        created_at: now,
        updated_at: now,
        source_state: UsageSourceState::Live,
        last_seen_at: Some(now),
        raw_deleted_at: None,
        archived_at: now,
    };
    upsert_session_archive_entry(&pool.get().unwrap(), &entry).unwrap();
    id
}
#[test]
fn keyset_pages_are_stable_bounded_and_keep_pi_and_omp_separate() {
    let temp = tempfile::tempdir().unwrap();
    let pool = pool(temp.path());
    for i in 0..205 {
        seed(
            &pool,
            &temp.path().join(format!("{i}.jsonl")),
            if i % 2 == 0 { "pi" } else { "omp" },
            "partial",
            i,
        );
    }
    let first = list_sessions(
        &pool,
        SessionListRequest {
            limit: Some(999),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(first.items.len(), 200);
    let second = list_sessions(
        &pool,
        SessionListRequest {
            limit: Some(200),
            cursor: first.next_cursor.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(second.items.len(), 5);
    assert!(first.items.iter().all(|row| {
        second
            .items
            .iter()
            .all(|other| row.archive_id != other.archive_id)
    }));
    assert!(
        first
            .items
            .windows(2)
            .all(|pair| pair[0].archive_id > pair[1].archive_id)
    );
    let pi = list_sessions(
        &pool,
        SessionListRequest {
            agents: Some(vec![AgentSessionAgentId::Pi]),
            limit: Some(200),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(pi.items.len(), 103);
    assert!(pi.items.iter().all(|r| r.platform == "pi"));
    let omp = list_sessions(
        &pool,
        SessionListRequest {
            agents: Some(vec![AgentSessionAgentId::Omp]),
            limit: Some(200),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(omp.items.len(), 102);
    assert!(omp.items.iter().all(|r| r.platform == "omp"));
    for request in [
        SessionListRequest {
            cursor: Some("bad".into()),
            ..Default::default()
        },
        SessionListRequest {
            query: Some("x".repeat(201)),
            ..Default::default()
        },
        SessionListRequest {
            started_at: Some("2026-09-25T00:00:00Z".into()),
            ended_at: Some("2026-09-24T00:00:00Z".into()),
            ..Default::default()
        },
    ] {
        assert!(list_sessions(&pool, request).is_err());
    }
}
#[test]
fn explicit_provider_detail_is_paged_path_safe_and_does_not_upgrade_fidelity() {
    let temp = tempfile::tempdir().unwrap();
    let pool = pool(temp.path());
    let source = temp
        .path()
        .join(".codex/sessions/2026/09/24/rollout-fixture.jsonl");
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    let line = "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"private transcript fixture\"}]}}\n";
    std::fs::write(&source, line.repeat(250)).unwrap();
    let before = std::fs::read(&source).unwrap();
    let id = seed(&pool, &source, "codex", "partial", 1);
    assert!(!id.contains("rollout") && !id.contains(temp.path().to_str().unwrap()));
    let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf())
        .without_environment_overrides();
    let first = get_detail_with_registry(&pool, &registry, &id, None, Some(100)).unwrap();
    assert_eq!(first.page.messages.len(), 100);
    assert_eq!(first.page.fidelity, AgentSessionFidelity::Partial);
    assert!(first.page.has_older);
    let second =
        get_detail_with_registry(&pool, &registry, &id, first.page.next_before, Some(100)).unwrap();
    assert_eq!(second.page.messages.len(), 100);
    assert!(
        first.page.messages.iter().all(|r| second
            .page
            .messages
            .iter()
            .all(|other| r.key != other.key))
    );
    assert!(
        first
            .page
            .messages
            .iter()
            .all(|r| r.content == "private transcript fixture")
    );
    assert_eq!(before, std::fs::read(&source).unwrap());
    std::fs::remove_file(&source).unwrap();
    assert_eq!(
        get_detail_with_registry(&pool, &registry, &id, None, None).unwrap_err(),
        SessionError::SourceUnavailable
    );
    assert_eq!(
        get_detail_with_registry(&pool, &registry, "", None, None).unwrap_err(),
        SessionError::InvalidArchiveId
    );
    let escaped = temp.path().join("escape.jsonl");
    std::fs::write(&escaped, line).unwrap();
    let id = seed(&pool, &escaped, "codex", "full", 2);
    assert_eq!(
        get_detail_with_registry(&pool, &registry, &id, None, None).unwrap_err(),
        SessionError::SourceValidation
    );
}
