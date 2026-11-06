//! Explicit ignored fixture generator. This does not launch either application
//! or collect performance samples; all database writes use existing owners.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use ccr_config::{CcsConfig, ConfigManager, ConfigSection};
use ccr_core::core::LockManager;
use ccr_db::database::{create_app_pool_at, create_usage_archive_pool_at};
use ccr_store::sessions::AgentSessionProviderRegistry;
use ccr_usage::{Dashboard, LogsQuery, QueryFilter, ReportTimezone};
use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const AS_OF: &str = "2026-11-06T12:00:00Z";

#[derive(Clone, Copy)]
enum Scenario {
    S0,
    S1,
    S2,
}
impl Scenario {
    fn id(self) -> &'static str {
        match self {
            Self::S0 => "S0",
            Self::S1 => "S1",
            Self::S2 => "S2",
        }
    }
    fn counts(self) -> (usize, usize, usize) {
        match self {
            Self::S0 => (0, 0, 0),
            Self::S1 => (100, 10_000, 100_000),
            Self::S2 => (10_000, 10_000, 100_000),
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn hash(value: impl AsRef<[u8]>) -> String {
    hex(&Sha256::digest(value.as_ref()))
}
fn json_hash(value: &Value) -> String {
    hash(serde_json::to_vec(value).unwrap())
}
fn clock() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(AS_OF)
        .unwrap()
        .with_timezone(&Utc)
}
fn assert_unlinked(path: &Path) {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(meta) => {
                assert!(
                    !meta.file_type().is_symlink(),
                    "fixture path cannot follow links"
                );
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    assert_eq!(
                        meta.file_attributes() & 0x400,
                        0,
                        "fixture path cannot follow reparse points"
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => panic!("fixture ancestor metadata is unavailable"),
        }
    }
}

fn prepare_allowed_directory(path: &Path) -> PathBuf {
    assert_unlinked(path);
    fs::create_dir_all(path).unwrap();
    assert_unlinked(path);
    path.canonicalize().unwrap()
}

#[test]
#[cfg(any(unix, windows))]
fn allowed_directory_rejects_redirected_ancestor_before_creating_missing_suffix() {
    let temp = tempfile::tempdir().unwrap();
    let outside = temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    let redirected = temp.path().join("redirected");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, &redirected).unwrap();
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let output = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", "$ErrorActionPreference = 'Stop'; New-Item -ItemType Junction -Path $env:CCR_FIXTURE_LINK -Value $env:CCR_FIXTURE_TARGET | Out-Null"])
            .env("CCR_FIXTURE_LINK", &redirected)
            .env("CCR_FIXTURE_TARGET", &outside)
            .creation_flags(0x08000000)
            .output().unwrap();
        assert!(
            output.status.success(),
            "owned fixture junction creation failed"
        );
    }
    let result =
        std::panic::catch_unwind(|| prepare_allowed_directory(&redirected.join("missing-suffix")));
    assert!(result.is_err());
    assert!(
        !outside.join("missing-suffix").exists(),
        "validation must precede creation through the redirect"
    );
}

fn profiles(root: &Path, count: usize) -> Value {
    if count == 0 {
        return json!({"source":"missing","rows":[]});
    }
    let file = root.join("ccr/platforms/claude/profiles.toml");
    let mut config = CcsConfig {
        default_config: "bench-00000".into(),
        current_config: String::new(),
        settings: Default::default(),
        sections: Default::default(),
    };
    for index in 0..count {
        config.sections.insert(
            format!("bench-{index:05}"),
            ConfigSection {
                description: Some(format!("Synthetic benchmark profile {index}")),
                base_url: Some("https://synthetic.invalid".into()),
                auth_token: Some(ccr_core::Secret::new(
                    "synthetic-benchmark-no-real-credential",
                )),
                model: Some(format!("fixture-model-{}", index % 3)),
                enabled: Some(index % 10 != 9),
                ..Default::default()
            },
        );
    }
    let manager = ConfigManager::new(file);
    LockManager::with_scoped_directory(&root.join("locks"), || manager.save(&config)).unwrap();
    let read = manager.snapshot().unwrap();
    assert_eq!(read.config.sections.len(), count);
    let rows=read.config.sections.iter().map(|(name,section)| json!({
        "name":name,"description":section.description,"base_url":section.base_url,
        "model":section.model,"enabled":section.enabled,"has_synthetic_credential":section.auth_token.is_some()
    })).collect::<Vec<_>>();
    json!({"source":"catalog","default":read.config.default_config,"current":read.config.current_config,"rows":rows})
}

fn usage(root: &Path, count: usize) -> Value {
    use ccr_usage::fixtures::{
        SeedBucket, SeedEvent, create_projection_db, seed_bucket, seed_event, seed_run_log,
    };
    let paths = create_projection_db(&root.join("llmusage"));
    let mut conn = rusqlite::Connection::open(&paths.db_path).unwrap();
    let tx = conn.transaction().unwrap();
    let mut buckets = BTreeMap::<String, usize>::new();
    for index in 0..count {
        let timestamp = (clock() - Duration::hours((index % 240) as i64))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        *buckets.entry(timestamp.clone()).or_default() += 1;
        seed_event(
            &tx,
            &SeedEvent {
                event_key: format!("bench-event-{index:06}"),
                source: "claude".into(),
                provider_label: "synthetic-provider".into(),
                model: "fixture-model-0".into(),
                event_at: timestamp,
                project_hash: "synthetic-project".into(),
                project_label: "Synthetic project".into(),
                project_path: Some("/synthetic/benchmark".into()),
                cost_with_cache_usd: 0.000001,
                cost_without_cache_usd: 0.000002,
                raw_json: json!({"kind":"synthetic-benchmark","ordinal":index}).to_string(),
                ..Default::default()
            },
        );
    }
    for (timestamp, records) in buckets {
        let records = records as i64;
        seed_bucket(
            &tx,
            &SeedBucket {
                source: "claude".into(),
                provider_label: "synthetic-provider".into(),
                model: "fixture-model-0".into(),
                hour_start: timestamp,
                project_hash: "synthetic-project".into(),
                project_label: "Synthetic project".into(),
                input_tokens: records * 10,
                cache_read_tokens: records * 2,
                cache_creation_tokens: records,
                output_tokens: records * 5,
                reasoning_output_tokens: records,
                total_tokens: records * 19,
                event_count: records,
                cost_with_cache_usd: records as f64 * 0.000001,
                cost_without_cache_usd: records as f64 * 0.000002,
                ..Default::default()
            },
        );
    }
    if count > 0 {
        seed_run_log(&tx, "claude", "recent", AS_OF);
    }
    tx.commit().unwrap();
    drop(conn);
    let dashboard = Dashboard::open(paths).unwrap();
    let filter = QueryFilter {
        timezone: ReportTimezone::Utc,
        until: Some(clock().date_naive()),
        ..Default::default()
    };
    let snapshot = crate::usage::read::read_dashboard(&dashboard, &filter, None).unwrap();
    assert_eq!(snapshot.overview.total_events, count as i64);
    assert_eq!(snapshot.overview.total.total_tokens, count as i64 * 19);
    let mut cursor = None;
    let mut events = Sha256::new();
    let mut seen = 0;
    loop {
        let page = dashboard
            .logs(&LogsQuery {
                filter: filter.clone(),
                page_size: 500,
                cursor,
                include_total: false,
                include_raw_json: true,
            })
            .unwrap();
        for record in page.records {
            events.update(serde_json::to_vec(&record).unwrap());
            events.update([0]);
            seen += 1;
        }
        cursor = page.next_cursor;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(seen, count);
    json!({"records":seen,"events_sha256":hex(&events.finalize()),"tokens":snapshot.overview.total,
        "requests":snapshot.overview.total_events,"cost_usd":snapshot.overview.total_cost_usd,
        "trends":snapshot.trends,"models":snapshot.models,"providers":snapshot.providers,"projects":snapshot.projects,"sources":snapshot.sources})
}

fn sessions(root: &Path, count: usize) -> (Value, String) {
    let home = root.join("home");
    let directory = home.join(".codex/sessions");
    fs::create_dir_all(&directory).unwrap();
    let mut raw = Sha256::new();
    for index in 0..count {
        let name = format!("rollout-bench-{index:05}.jsonl");
        let content = format!(
            "{}\n{}\n",
            json!({"type":"session_meta","timestamp":AS_OF,"payload":{"id":format!("bench-session-{index:05}"),"cwd":"/synthetic/benchmark"}}),
            json!({"type":"response_item","timestamp":AS_OF,"payload":{"type":"message","role":"user","content":[{"type":"input_text","text":format!("Synthetic benchmark session {index}")}]}})
        );
        raw.update(name.as_bytes());
        raw.update([0]);
        raw.update(content.as_bytes());
        raw.update([0]);
        fs::write(directory.join(name), content).unwrap();
    }
    let pool = create_usage_archive_pool_at(
        &root.join("ccr/analytics/usage.db"),
        &home,
        &root.join("ui/ccr-ui.db"),
    )
    .unwrap();
    let registry = AgentSessionProviderRegistry::new(home).without_environment_overrides();
    let report = crate::sessions::refresh::refresh_archive_with_registry(&pool, &registry).unwrap();
    assert_eq!(report.total.upserted, count as u64);
    assert_eq!(report.total.errors, 0);
    let mut rows = Vec::new();
    let mut identities = Vec::new();
    let mut cursor = None;
    loop {
        let page = crate::sessions::list_sessions(
            &pool,
            crate::sessions::SessionListRequest {
                cursor,
                limit: Some(200),
                ..Default::default()
            },
        )
        .unwrap();
        for row in page.items {
            identities.push(row.archive_id);
            rows.push(json!({"id":row.session_id,"platform":row.platform,"variant":row.source_variant,"cwd":row.cwd,
                "title":row.title,"messages":row.message_count,"user_messages":row.user_message_count,"assistant_messages":row.assistant_message_count,
                "tool_uses":row.tool_use_count,"created_at":row.created_at,"updated_at":row.updated_at,"state":row.source_state.as_str(),"fidelity":row.source_fidelity}));
        }
        cursor = page.next_cursor;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(rows.len(), count);
    rows.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    identities.sort();
    if let Some(id) = identities.first() {
        let detail =
            crate::sessions::get_detail_with_registry(&pool, &registry, id, None, Some(10))
                .unwrap();
        assert!(
            detail
                .page
                .messages
                .iter()
                .any(|message| message.content.contains("Synthetic benchmark session"))
        );
    }
    drop(pool);
    (
        json!({"count":rows.len(),"raw_sources_sha256":hex(&raw.finalize()),"summaries_sha256":json_hash(&json!(rows))}),
        json_hash(&json!(identities)),
    )
}

fn seed_end(root: &Path, scenario: Scenario) -> Value {
    fs::create_dir(root).unwrap();
    for directory in [
        "home",
        "ccr/platforms/claude",
        "ui",
        "ccr/analytics",
        "llmusage",
        "logs",
        "preferences",
        "locks",
        "backups",
        "crypto",
        "project",
        "webview",
    ] {
        fs::create_dir_all(root.join(directory)).unwrap();
    }
    fs::write(root.join("ccr-tauri-fixture.json"),serde_json::to_vec_pretty(&json!({"version":1,"fixture_id":format!("benchmark-{}",scenario.id().to_ascii_lowercase())})).unwrap()).unwrap();
    let (profiles_count, sessions_count, usage_count) = scenario.counts();
    let main = create_app_pool_at(&root.join("ui/ccr-ui.db"), &root.join("home")).unwrap();
    drop(main);
    let profiles = profiles(root, profiles_count);
    let usage = usage(root, usage_count);
    let (sessions, identities) = sessions(root, sessions_count);
    let logical = json!({"profiles_sha256":json_hash(&profiles),"profiles":profiles_count,"usage":usage,"sessions":sessions,"initial_monitoring_events":0});
    let artifacts = [
        ("catalog", "ccr/platforms/claude/profiles.db"),
        ("usage", "llmusage/llmusage.db"),
        ("archive", "ccr/analytics/usage.db"),
        ("main", "ui/ccr-ui.db"),
    ]
    .into_iter()
    .map(|(role, path)| {
        (
            role.to_string(),
            if root.join(path).is_file() {
                json!(hash(fs::read(root.join(path)).unwrap()))
            } else {
                Value::Null
            },
        )
    })
    .collect::<BTreeMap<_, _>>();
    json!({"root":root,"logical":logical,"logical_sha256":json_hash(&logical),"source_identity_sha256":identities,"artifact_sha256":artifacts,
        "loading":"not_run","native_benchmark_adapter":"not_implemented","performance":"not_run"})
}

#[test]
#[ignore = "explicit new CCR_BENCHMARK_FIXTURE_ROOT under outputs/desktop/perf/fixtures required"]
fn generate_s0_s1_s2_pairs() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let allowed = workspace.join("outputs/desktop/perf/fixtures");
    let allowed = prepare_allowed_directory(&allowed);
    let requested = PathBuf::from(
        std::env::var_os("CCR_BENCHMARK_FIXTURE_ROOT").expect("explicit new root required"),
    );
    assert!(requested.is_absolute());
    assert!(!requested.exists(), "refuse to replace a fixture");
    assert_unlinked(&requested);
    let parent = requested.parent().unwrap().canonicalize().unwrap();
    assert!(
        parent.starts_with(&allowed),
        "fixture root must stay in benchmark outputs"
    );
    let root = parent.join(requested.file_name().unwrap());
    fs::create_dir(&root).unwrap();
    let mut manifests = Vec::new();
    for scenario in [Scenario::S0, Scenario::S1, Scenario::S2] {
        let pair = root.join(scenario.id());
        fs::create_dir(&pair).unwrap();
        let tauri = seed_end(&pair.join("tauri"), scenario);
        let gpui = seed_end(&pair.join("gpui"), scenario);
        assert_eq!(
            tauri["logical_sha256"], gpui["logical_sha256"],
            "two end projections differ"
        );
        if scenario.counts().1 > 0 {
            assert_ne!(
                tauri["source_identity_sha256"], gpui["source_identity_sha256"],
                "physical domain identities must remain root-bound"
            );
        }
        let manifest = json!({"version":1,"synthetic":true,"scenario":scenario.id(),"as_of_utc":AS_OF,"report_timezone":"UTC",
            "counts":{"claude_profiles":scenario.counts().0,"codex_sessions":scenario.counts().1,"usage_events":scenario.counts().2},
            "logical_sha256":tauri["logical_sha256"],"ends":{"tauri":tauri,"gpui":gpui},
            "stream_workload":if matches!(scenario,Scenario::S2) {json!({"events_per_second":1000,"duration_seconds":60,"state":"not_emitted"})} else {Value::Null},
            "hash_contract":"Profiles use non-secret display fields. Usage hashes all owner-paged records. Session summary hashes exclude root-bound opaque archive IDs; original IDs are retained and separately hashed. No source identity is rewritten.",
            "comparison":"not_comparable","reason":"native benchmark source/loading and real clock sampling are not implemented"});
        fs::write(
            pair.join("logical-manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        manifests.push(json!({"scenario":scenario.id(),"manifest":pair.join("logical-manifest.json"),"logical_sha256":manifest["logical_sha256"]}));
        println!(
            "{} paired data generated and owner projections match",
            scenario.id()
        );
    }
    fs::write(root.join("manifest.json"),serde_json::to_vec_pretty(&json!({"version":1,"as_of_utc":AS_OF,"scenarios":manifests,"performance":"not_run","comparison":"not_comparable"})).unwrap()).unwrap();
}
