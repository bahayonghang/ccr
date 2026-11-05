//! Explicit, opt-in fixture preparation for real native acceptance.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use ccr_db::database::{
    create_pool,
    migrations::run_all_migrations,
    repositories::usage_repo::{self, UsageSessionArchiveEntry, UsageSourceState},
};
use ccr_usage::fixtures::{
    SeedBucket, SeedEvent, create_projection_db, seed_bucket, seed_event, seed_run_log,
};
use chrono::{TimeZone, Utc};
use std::{fs, path::PathBuf};

#[test]
#[ignore = "requires CCR_GPUI_INSIGHTS_FIXTURE_ROOT under outputs/desktop/insights"]
fn prepare_native_insights_fixture() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let allowed = workspace.join("outputs/desktop/insights");
    fs::create_dir_all(&allowed).unwrap();
    let allowed = allowed.canonicalize().unwrap();
    let requested = PathBuf::from(
        std::env::var_os("CCR_GPUI_INSIGHTS_FIXTURE_ROOT").expect("explicit fixture root required"),
    );
    assert!(requested.is_absolute(), "fixture root must be absolute");
    assert!(!requested.exists(), "refuse to overwrite existing fixture");
    let parent = requested.parent().unwrap().canonicalize().unwrap();
    assert!(
        parent.starts_with(&allowed),
        "fixture must stay in insights outputs"
    );
    let root = parent.join(requested.file_name().unwrap());
    fs::create_dir(&root).unwrap();
    fs::write(
        root.join(".ccr-gpui-insights-fixture"),
        "synthetic fixture; no real credentials\n",
    )
    .unwrap();
    let paths = create_projection_db(&root.join("llmusage"));
    let usage_pool = create_pool(&paths.db_path, None).unwrap();
    let conn = usage_pool.get().unwrap();
    for (source, provider, model, project, cost) in [
        ("codex", "openai", "gpt-5", "p1", 0.123456789),
        ("claude", "anthropic", "claude-fixture", "p2", 0.234567891),
    ] {
        seed_bucket(
            &conn,
            &SeedBucket {
                source: source.into(),
                provider_label: provider.into(),
                model: model.into(),
                project_hash: project.into(),
                hour_start: "2026-10-28T12:00:00Z".into(),
                cost_with_cache_usd: cost,
                event_count: if source == "codex" { 101 } else { 1 },
                ..Default::default()
            },
        );
    }
    for i in 0..102 {
        seed_event(
            &conn,
            &SeedEvent {
                event_key: format!("native-event-{i:03}"),
                source: if i == 101 { "claude" } else { "codex" }.into(),
                provider_label: if i == 101 { "anthropic" } else { "openai" }.into(),
                model: if i == 101 { "claude-fixture" } else { "gpt-5" }.into(),
                event_at: "2026-10-28T12:00:00Z".into(),
                raw_json: "{\"kind\":\"synthetic-native-fixture\"}".into(),
                ..Default::default()
            },
        );
    }
    seed_run_log(&conn, "codex", "recent", "2026-10-28T12:01:00Z");
    drop(conn);
    drop(usage_pool);

    let home = root.join("home");
    let sessions = home.join(".codex/sessions/2026/10/28");
    fs::create_dir_all(&sessions).unwrap();
    let archive = create_pool(&root.join("archive.db"), None).unwrap();
    run_all_migrations(&archive.get().unwrap(), &root).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 10, 28, 12, 0, 0).unwrap();
    for i in 0..205 {
        let path = sessions.join(format!("rollout-native-{i:03}.jsonl"));
        let count = if i == 204 { 250 } else { 3 };
        let mut content = format!(
            "{{\"type\":\"session_meta\",\"timestamp\":\"2026-10-28T12:00:00Z\",\"payload\":{{\"id\":\"native-session-{i:03}\",\"cwd\":\"/synthetic\"}}}}\n"
        );
        for ordinal in 0..count {
            content.push_str(&format!("{{\"type\":\"response_item\",\"payload\":{{\"type\":\"message\",\"role\":\"user\",\"content\":[{{\"type\":\"input_text\",\"text\":\"Synthetic native message {ordinal:03}\"}}]}}}}\n"));
        }
        fs::write(&path, content).unwrap();
    }
    let registry = ccr_store::sessions::AgentSessionProviderRegistry::new(home.clone())
        .without_environment_overrides();
    let mut discovered = registry
        .discover(ccr_store::sessions::AgentSessionAgentId::Codex)
        .unwrap();
    discovered.sort_by(|left, right| left.physical_path.cmp(&right.physical_path));
    assert_eq!(discovered.len(), 205);
    let conn = archive.get().unwrap();
    let tx = conn.unchecked_transaction().unwrap();
    for (i, source) in discovered.into_iter().enumerate() {
        let count = if i == 204 { 250 } else { 3 };
        let fingerprint = registry.quick_fingerprint(&source).unwrap();
        let file_path = source.physical_path.to_string_lossy().to_string();
        usage_repo::upsert_session_archive_entry(
            &tx,
            &UsageSessionArchiveEntry {
                archive_id: usage_repo::agent_session_archive_id("codex", &file_path, ""),
                session_id: format!("native-session-{i:03}"),
                platform: "codex".into(),
                title: Some(format!("Native fixture {i:03}")),
                cwd: "/synthetic".into(),
                file_path,
                file_hash: Some(fingerprint.clone()),
                source_variant: "codex-live".into(),
                source_kind: "file".into(),
                source_member_id: String::new(),
                source_size: None,
                source_mtime_ns: None,
                source_stat_hash: Some(fingerprint),
                message_count: count,
                user_message_count: count,
                assistant_message_count: 0,
                tool_use_count: 0,
                source_fidelity: "partial".into(),
                created_at: now,
                updated_at: now + chrono::Duration::seconds(i64::try_from(i).unwrap()),
                source_state: UsageSourceState::Live,
                last_seen_at: Some(now),
                raw_deleted_at: None,
                archived_at: now,
            },
        )
        .unwrap();
    }
    tx.commit().unwrap();
    drop(conn);
    let before_refresh = crate::sessions::list_sessions(
        &archive,
        crate::sessions::SessionListRequest {
            limit: Some(200),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(before_refresh.items.len(), 200);
    let refreshed =
        crate::sessions::refresh::refresh_archive_with_registry(&archive, &registry).unwrap();
    assert_eq!(refreshed.total.discovered, 205);
    assert_eq!(refreshed.total.unchanged, 205);
    assert_eq!(refreshed.total.upserted, 0);
    let after_refresh = crate::sessions::list_sessions(
        &archive,
        crate::sessions::SessionListRequest {
            limit: Some(200),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        before_refresh
            .items
            .iter()
            .map(|row| &row.archive_id)
            .collect::<Vec<_>>(),
        after_refresh
            .items
            .iter()
            .map(|row| &row.archive_id)
            .collect::<Vec<_>>()
    );
    let last_page = crate::sessions::list_sessions(
        &archive,
        crate::sessions::SessionListRequest {
            limit: Some(200),
            cursor: after_refresh.next_cursor,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(last_page.items.len(), 5);
    fs::write(
        root.join("fixture.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "synthetic": true, "usage_buckets": 2, "usage_events": 102, "archive_sessions": 205,
            "latest_session_messages": 250, "usage_root": paths.root_dir,
            "archive_db": root.join("archive.db"), "provider_home": home,
            "codex_cost_with_cache_usd": 0.123456789, "claude_cost_with_cache_usd": 0.234567891,
            "fixture_scope": "native read/filter/pagination/detail; not S1/S2 performance"
        }))
        .unwrap(),
    )
    .unwrap();
    println!("synthetic native insights fixture ready");
}
