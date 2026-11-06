#![allow(clippy::unwrap_used)]

use super::*;
pub(super) fn layout(root: &Path) -> FixtureLayout {
    std::fs::write(
        root.join(MARKER),
        br#"{"version":1,"fixture_id":"synthetic"}"#,
    )
    .unwrap();
    FixtureLayout::load(root).unwrap()
}
#[test]
fn fixture_boot_rejects_invalid_identity_before_any_directory_creation() {
    assert!(parse([OsString::from("application")]).unwrap().is_none());
    assert!(parse([OsString::from("--fixture-root")]).is_err());
    assert!(parse([OsString::from("--fixture-root"), OsString::from("relative")]).is_err());
    let root = tempfile::tempdir().unwrap();
    assert!(FixtureLayout::load(root.path()).is_err());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    let layout = layout(root.path());
    assert!(layout.path("../escape").is_err());
    assert!(layout.path("/absolute").is_err());
    layout.prepare().unwrap();
    assert!(layout.path("webview").unwrap().is_dir());
    assert!(
        layout
            .path("home/.ccr/data.db")
            .unwrap()
            .starts_with(&layout.root)
    );
}
#[test]
fn fixture_capabilities_are_explicit_and_external_graphs_stay_closed() {
    for id in [
        "list_configs",
        "agent_sessions_get_detail",
        "get_usage_dashboard_v2",
        "get_monitoring_feed",
    ] {
        assert!(captured_command(id));
    }
    for id in [
        "list_providers",
        "wsl_list_distros",
        "ssh_list_keys",
        "unknown",
    ] {
        assert!(!captured_command(id));
    }
    for &id in CAPTURED_COMMANDS {
        assert!(
            crate::commands::handler_registry::command_descriptor(id).is_some(),
            "unregistered fixture capability: {id}"
        );
    }
}

#[test]
fn fixture_core_graph_isolated_process_preserves_other_root() {
    let selected = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let other_layout = layout(other.path());
    other_layout.prepare().unwrap();
    let sentinel = other.path().join("preferences/desktop-shell.json");
    std::fs::write(&sentinel, b"other-root-sentinel").unwrap();
    let selected_layout = layout(selected.path());
    let ambient_trap = selected.path().join("ambient-root-is-a-file");
    std::fs::write(&ambient_trap, b"synthetic ambient root trap").unwrap();
    let legacy = selected_layout
        .platform_paths(Platform::Claude)
        .unwrap()
        .profiles_file;
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    std::fs::write(legacy,"default_config='legacy'\ncurrent_config=''\n[legacy]\nbase_url='https://synthetic.invalid'\nauth_token='synthetic-legacy-key'\n").unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "fixture::tests::fixture_core_graph_child",
            "--ignored",
            "--nocapture",
        ])
        .env("CCR_TAURI_FIXTURE_TEST_ROOT", &selected_layout.root)
        .env("CCR_TAURI_FIXTURE_OTHER_ROOT", &other_layout.root)
        .env("CCR_ROOT", &ambient_trap)
        .env("CCR_DATA_DIR", &ambient_trap)
        .env("CCR_CONFIG_PATH", &ambient_trap)
        .env("CLAUDE_CONFIG_DIR", &ambient_trap)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    assert_eq!(std::fs::read(sentinel).unwrap(), b"other-root-sentinel");
}

#[tokio::test]
#[ignore = "invoked only by the isolated fixture core-graph test"]
async fn fixture_core_graph_child() {
    use crate::{
        llmusage_adapter::{AppPaths, LlmusageRuntime},
        platform::ExecutionEnvironment,
    };
    use ccr_config::{CcsConfig, ConfigSection};
    let root = PathBuf::from(std::env::var_os("CCR_TAURI_FIXTURE_TEST_ROOT").unwrap());
    let layout = Arc::new(FixtureLayout::load(&root).unwrap());
    install(layout.clone()).unwrap();
    assert!(install(layout.clone()).is_err());
    layout.prepare().unwrap();
    let migrated = layout.platform_paths(Platform::Claude).unwrap();
    assert!(migrated.profiles_db().is_file());
    assert!(!migrated.profiles_file.exists());
    ccr_core::init_logger_at(&layout.path("logs").unwrap());
    let main = ccr_db::database::initialize_app_pool_at(
        &layout.path("ui/ccr-ui.db").unwrap(),
        &layout.roots.home,
    )
    .unwrap();
    let other = PathBuf::from(std::env::var_os("CCR_TAURI_FIXTURE_OTHER_ROOT").unwrap());
    assert!(ccr_db::database::initialize_app_pool_at(&other.join("duplicate.db"), &other).is_err());
    assert!(!other.join("duplicate.db").exists());
    main.get().unwrap().execute_batch("CREATE TABLE fixture_pool_identity(value TEXT); INSERT INTO fixture_pool_identity VALUES ('selected');").unwrap();
    assert_eq!(
        ccr_db::database::with_connection(|conn| conn.query_row(
            "SELECT value FROM fixture_pool_identity",
            [],
            |row| row.get::<_, String>(0)
        ))
        .unwrap(),
        "selected"
    );
    let archive = ccr_db::database::create_usage_archive_pool_at(
        &layout.path("ccr/analytics/usage.db").unwrap(),
        &layout.roots.home,
        &layout.path("ui/ccr-ui.db").unwrap(),
    )
    .unwrap();

    let paths = ccr_usage::fixtures::create_projection_db(&layout.path("llmusage").unwrap());
    let conn = rusqlite::Connection::open(&paths.db_path).unwrap();
    ccr_usage::fixtures::seed_bucket(
        &conn,
        &ccr_usage::fixtures::SeedBucket {
            source: "claude".into(),
            model: "synthetic".into(),
            hour_start: "2026-11-06T12:00:00Z".into(),
            event_count: 3,
            total_tokens: 42,
            ..Default::default()
        },
    );
    drop(conn);
    let runtime =
        LlmusageRuntime::from_paths(AppPaths::from_root(layout.path("llmusage").unwrap()));
    let usage = crate::services::usage::usage_summary(&runtime, None, None, None).unwrap();
    assert_eq!((usage.total_requests, usage.total_tokens), (3, 42));
    let state = crate::state::AppState::try_new_with_fixture(
        main.clone(),
        archive.clone(),
        runtime,
        Some(layout.clone()),
    )
    .unwrap();
    state
        .update_desktop_shell_preferences(|value| value.confirm_before_exit = false)
        .unwrap();
    let prefs = layout.path("preferences/desktop-shell.json").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if std::fs::read_to_string(&prefs)
            .is_ok_and(|value| value.contains("\"confirm_before_exit\": false"))
        {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "captured preference save timed out"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let context = layout.profile_context(Platform::Claude).unwrap();
    context
        .run(|| {
            let mut config = CcsConfig {
                default_config: "seed".into(),
                current_config: String::new(),
                settings: Default::default(),
                sections: Default::default(),
            };
            config.sections.insert(
                "seed".into(),
                ConfigSection {
                    base_url: Some("https://synthetic.invalid".into()),
                    auth_token: Some(ccr_core::Secret::new("synthetic-fixture-key")),
                    ..Default::default()
                },
            );
            context.manager().save(&config)
        })
        .unwrap();
    let list = crate::commands::config::list_configs(Some("claude".into()))
        .await
        .unwrap();
    assert_eq!(list[0].name, "seed");
    assert!(!list[0].auth_token.contains("synthetic-fixture-key"));
    let revision = list[0].version.clone();
    let patch = || {
        serde_json::from_value(serde_json::json!({"description":"updated-in-selected-root"}))
            .unwrap()
    };
    crate::commands::config::update_config(
        "seed".into(),
        Some("claude".into()),
        patch(),
        Some(revision.clone()),
    )
    .await
    .unwrap();
    assert!(
        crate::commands::config::update_config(
            "seed".into(),
            Some("claude".into()),
            patch(),
            Some(revision)
        )
        .await
        .is_err()
    );
    assert_eq!(
        crate::commands::config::list_configs(Some("claude".into()))
            .await
            .unwrap()[0]
            .description,
        "updated-in-selected-root"
    );

    let local = crate::platform::local::LocalEnvironment::from_fixture(layout.clone());
    local
        .write_config("claude", "settings.json", "{\"model\":\"first\"}")
        .await
        .unwrap();
    local
        .write_config("claude", "settings.json", "{\"model\":\"second\"}")
        .await
        .unwrap();
    assert!(
        local
            .read_config("claude", "settings.json")
            .await
            .unwrap()
            .contains("second")
    );
    assert!(
        std::fs::read_dir(&layout.roots.claude.backups_dir)
            .unwrap()
            .any(|entry| entry
                .unwrap()
                .path()
                .extension()
                .is_some_and(|value| value == "bak"))
    );
    assert!(local.detect_cli_status().await.is_err());
    crate::commands::claude::claude_add_mcp_server(
        "synthetic-server".into(),
        serde_json::json!({"command":"synthetic-no-execution","args":["--fixture"]})
            .try_into()
            .unwrap(),
        Some("user".into()),
    )
    .await
    .unwrap();
    let servers: serde_json::Value = crate::commands::claude::claude_list_mcp_servers()
        .await
        .unwrap()
        .into();
    assert!(servers.to_string().contains("synthetic-server"));
    assert!(layout.roots.claude.state_file.is_file());
    assert!(
        crate::commands::claude::claude_get_auth_current()
            .await
            .is_ok()
    );

    let session = layout
        .path("home/.codex/sessions/rollout-fixture.jsonl")
        .unwrap();
    std::fs::create_dir_all(session.parent().unwrap()).unwrap();
    std::fs::write(session,concat!("{\"type\":\"session_meta\",\"timestamp\":\"2026-11-06T12:00:00Z\",\"payload\":{\"id\":\"fixture-session\",\"cwd\":\"/synthetic\"}}\n",
            "{\"type\":\"response_item\",\"timestamp\":\"2026-11-06T12:00:01Z\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"selected-root-transcript\"}]}}\n")).unwrap();
    let refresh = crate::services::agent_sessions::refresh_archive(&archive).unwrap();
    assert_eq!(refresh.total.upserted, 1);
    let page = crate::services::agent_sessions::list_sessions(
        &archive,
        serde_json::from_value(serde_json::json!({})).unwrap(),
    )
    .unwrap();
    assert_eq!(page.items.len(), 1);
    let detail = crate::services::agent_sessions::get_detail(
        &archive,
        crate::services::agent_sessions::AgentSessionDetailRequestDto {
            archive_id: page.items[0].archive_id.clone(),
            before_cursor: None,
            limit: Some(10),
        },
    )
    .unwrap();
    assert!(
        detail
            .messages
            .iter()
            .any(|message| message.content.contains("selected-root-transcript"))
    );
    let entry = ccr_types::MonitoringEntry::new(
        ccr_types::MonitoringLevel::Info,
        "fixture",
        "saved",
        "tauri",
        "fixture saved",
    );
    state.monitoring_logs.append_entry(&entry).await;
    state.monitoring_logs.force_flush().await;
    assert_eq!(
        ccr_db::services::log_persistence::LogPersistenceService::query_logs_with_connection(
            &main.get().unwrap(),
            None,
            Some("fixture"),
            10
        )
        .unwrap()
        .len(),
        1
    );
    let forbidden = layout.path("forbidden-body-ran").unwrap();
    assert!(
        crate::commands::runtime_policy::execute("codex_oauth_login_start", async {
            std::fs::write(&forbidden, b"must not run").unwrap();
            Ok::<_, String>(())
        })
        .await
        .is_err()
    );
    assert!(!forbidden.exists());
    assert!(admit("unknown").is_err());
}

#[test]
fn benchmark_mode_is_explicit_and_only_it_injects_a_readonly_report_clock() {
    let root = tempfile::tempdir().unwrap();
    let mut layout = layout(root.path());
    assert!(layout.benchmark.is_none());
    assert!(report_clock_script(&layout).unwrap().is_none());
    assert!(
        parse([
            OsString::from("--fixture-root"),
            root.path().as_os_str().into(),
            OsString::from("--benchmark-fixture-root"),
            root.path().as_os_str().into()
        ])
        .is_err()
    );
    assert!(parse([OsString::from("--benchmark-fixture-root")]).is_err());
    let at = chrono::DateTime::parse_from_rfc3339("2000-01-01T00:30:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    layout.benchmark = Some(ccr_app::benchmark::Seed {
        root: layout.root.clone(),
        id: "benchmark-s0".into(),
        scenario: "S0".into(),
        logical_sha256: String::new(),
        as_of: at,
        timezone: ccr_usage::ReportTimezone::Utc,
        profile_count: 0,
        session_count: 0,
        usage_count: 0,
    });
    let script = report_clock_script(&layout).unwrap().unwrap();
    assert!(script.contains("Object.defineProperty(window,'__CCR_REPORT_CLOCK__'"));
    assert!(script.contains("Object.freeze("));
    assert!(script.contains("writable:false,configurable:false"));
    assert!(script.contains("2000-01-01T00:30:00Z"));
    assert!(script.contains("UTC"));
    assert!(!script.contains("window.Date"));
    assert!(!script.contains(layout.root.to_string_lossy().as_ref()));
}

#[test]
#[ignore = "explicit generated paired fixtures; headless service reads in isolated children"]
fn generated_benchmark_default_reports_use_manifest_clock() {
    let pair = PathBuf::from(std::env::var_os("CCR_NATIVE_BENCHMARK_PAIR_ROOT").unwrap());
    for scenario in ["S0", "S1", "S2"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "fixture::tests::generated_benchmark_clock_child",
                "--ignored",
                "--nocapture",
                "--skip",
                "export_bindings",
            ])
            .env(
                "CCR_TAURI_BENCHMARK_CLOCK_CHILD",
                pair.join(scenario).join("tauri"),
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        println!("{scenario}: {}", String::from_utf8_lossy(&output.stdout));
    }
}
#[test]
#[ignore = "closed child of generated_benchmark_default_reports_use_manifest_clock"]
fn generated_benchmark_clock_child() {
    let root = PathBuf::from(std::env::var_os("CCR_TAURI_BENCHMARK_CLOCK_CHILD").unwrap());
    let layout = parse([OsString::from("--benchmark-fixture-root"),root.as_os_str().into()]).unwrap().unwrap();
    let seed = layout.benchmark.as_ref().unwrap();
    install(layout.clone()).unwrap();
    let clock = report_clock();
    assert_eq!(clock.as_of, seed.as_of);
    assert_eq!(clock.timezone, ccr_usage::ReportTimezone::Utc);
    let llmusage = crate::llmusage_adapter::LlmusageRuntime::from_paths(
        crate::llmusage_adapter::AppPaths::from_root(layout.path("llmusage").unwrap()),
    );
    let archive =
        ccr_db::database::create_pool(&layout.path("ccr/analytics/usage.db").unwrap(), None)
            .unwrap();
    let response = crate::services::usage::compute_home_overview(
        &llmusage,
        &archive,
        7,
        crate::services::usage::HomeJobContext {
            active_usage_job_id: None,
            active_usage_imported_records: 0,
            active_session_job_id: None,
            active_session_indexed: 0,
            active_usage_import: false,
            active_session_index: false,
            raw_sessions_present: false,
        },
    )
    .unwrap();
    assert_eq!(response.last_updated, seed.as_of.to_rfc3339());
    assert_eq!(response.summary.total_sessions, seed.session_count as u64);
    let manifest:serde_json::Value=serde_json::from_slice(&std::fs::read(layout.root.parent().unwrap().join("logical-manifest.json")).unwrap()).unwrap();
    let start=clock.window(7).0.to_string();let end=clock.date().to_string();
    let expected_requests=manifest["ends"]["tauri"]["logical"]["usage"]["trends"].as_array().unwrap().iter()
        .filter(|row|row["date"].as_str().is_some_and(|date|date>=start.as_str() && date<=end.as_str()))
        .map(|row|row["request_count"].as_i64().unwrap()).sum::<i64>();
    assert_eq!(response.summary.total_requests,expected_requests as u64);

    assert_eq!(
        response.series.last().unwrap().date,
        clock.date().to_string()
    );
    let home = crate::services::home_insights::compute_home_insights(
        &llmusage,
        &archive,
        clock.date(),
        clock.as_of,
    )
    .unwrap();
    assert_eq!(home.as_of, clock.date().to_string());
    assert_eq!(home.generated_at, seed.as_of.to_rfc3339());
    assert_eq!(home.totals.sessions, seed.session_count as i64);
    assert_eq!(home.last_7_days.requests,expected_requests);
    let trend=crate::services::claude_observer::daily_trend(&llmusage,Some(7)).unwrap();
    assert!(trend.iter().all(|point|point.date<=clock.date().to_string()));
    if seed.usage_count>0 {assert_eq!(trend.last().unwrap().date,clock.date().to_string());}
    println!(
        "{}: home sessions {}, as_of {}, generated {}",
        seed.scenario, response.summary.total_sessions, home.as_of, home.generated_at
    );
}
