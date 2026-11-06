#![allow(clippy::unwrap_used)]
use super::*;
use serde_json::json;

#[test]
fn paired_endpoints_share_validation_clock_and_refuse_changed_artifacts() {
    let pair = tempfile::tempdir().unwrap();
    let mut ends = serde_json::Map::new();
    let logical = json!({"test":"pure file admission"});
    let digest = logical_hash(&logical).unwrap();
    for endpoint in ["gpui", "tauri"] {
        let root = pair.path().join(endpoint);
        for name in [
            "home",
            "ccr",
            "ui",
            "llmusage",
            "logs",
            "preferences",
            "locks",
            "crypto",
            "ccr/analytics",
        ] {
            fs::create_dir_all(root.join(name)).unwrap();
        }
        let mut artifacts = serde_json::Map::new();
        artifacts.insert("catalog".into(), Value::Null);
        for (role, name) in [
            ("main", "ui/ccr-ui.db"),
            ("archive", "ccr/analytics/usage.db"),
            ("usage", "llmusage/llmusage.db"),
        ] {
            // Integrity-layer test bytes: no database is opened or claimed valid here.
            fs::write(root.join(name), b"synthetic integrity bytes").unwrap();
            artifacts.insert(role.into(), json!(file_hash(&root.join(name)).unwrap()));
        }
        fs::write(
            root.join("ccr-tauri-fixture.json"),
            br#"{"version":1,"fixture_id":"benchmark-s0"}"#,
        )
        .unwrap();
        ends.insert(endpoint.into(),json!({"root":root.canonicalize().unwrap(),"logical":logical,"logical_sha256":digest,"artifact_sha256":artifacts}));
    }
    let mut manifest = json!({"version":1,"synthetic":true,"scenario":"S0","counts":{"claude_profiles":0,"codex_sessions":0,"usage_events":0},"report_timezone":"UTC","as_of_utc":"2000-01-01T00:30:00Z","logical_sha256":digest,"ends":ends});
    let file = pair.path().join("logical-manifest.json");
    fs::write(&file, serde_json::to_vec(&manifest).unwrap()).unwrap();
    for invalid in ["2000-01-01T00:30:00+01:00", "2016-12-31T23:59:60Z"] {
        let mut invalid_clock = manifest.clone();
        invalid_clock["as_of_utc"] = json!(invalid);
        fs::write(&file, serde_json::to_vec(&invalid_clock).unwrap()).unwrap();
        assert!(
            Seed::load(&pair.path().join("gpui"), Endpoint::Gpui).is_err(),
            "unshared report clock: {invalid}"
        );
        assert!(
            Seed::load(&pair.path().join("tauri"), Endpoint::Tauri).is_err(),
            "unshared report clock: {invalid}"
        );
    }
    fs::write(&file, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let gpui = Seed::load(&pair.path().join("gpui"), Endpoint::Gpui).unwrap();
    let tauri = Seed::load(&pair.path().join("tauri"), Endpoint::Tauri).unwrap();
    assert_eq!(gpui.as_of, tauri.as_of);
    assert_eq!(gpui.logical_sha256, tauri.logical_sha256);
    assert_eq!(gpui.timezone, ReportTimezone::Utc);
    assert!(Seed::load(&gpui.root, Endpoint::Tauri).is_err());
    fs::write(tauri.root.join("ui/ccr-ui.db"), b"changed").unwrap();
    assert!(Seed::load(&tauri.root, Endpoint::Tauri).is_err());
    assert!(Seed::load(&gpui.root, Endpoint::Gpui).is_ok());

    for relative in [
        "ccr/platforms/claude/profiles.db",
        "ui/ccr-ui.db",
        "ccr/analytics/usage.db",
        "llmusage/llmusage.db",
    ] {
        let db = gpui.root.join(relative);
        fs::create_dir_all(db.parent().unwrap()).unwrap();
        for suffix in ["-wal", "-journal", "-shm"] {
            let mut sidecar = db.as_os_str().to_os_string();
            sidecar.push(suffix);
            let sidecar = PathBuf::from(sidecar);
            fs::write(&sidecar, b"").unwrap();
            assert!(
                Seed::load(&gpui.root, Endpoint::Gpui).is_ok(),
                "empty ordinary sidecar: {relative}{suffix}"
            );
            fs::write(&sidecar, b"unhashed state").unwrap();
            assert_eq!(
                Seed::load(&gpui.root, Endpoint::Gpui).is_ok(),
                suffix == "-shm",
                "nonempty sidecar: {relative}{suffix}"
            );
            fs::remove_file(&sidecar).unwrap();
            fs::create_dir(&sidecar).unwrap();
            assert!(
                Seed::load(&gpui.root, Endpoint::Gpui).is_err(),
                "sidecar must be regular: {relative}{suffix}"
            );
            fs::remove_dir(sidecar).unwrap();
        }
    }

    // A real captured domain write can change committed data only in WAL while
    // every manifest main-file digest still matches. Never checkpoint/repair it.
    let main = gpui.root.join("ui/ccr-ui.db");
    fs::remove_file(&main).unwrap();
    let pool = ccr_db::database::create_app_pool_at(&main, &gpui.root.join("home")).unwrap();
    let original_hash = file_hash(&main).unwrap();
    manifest["ends"]["gpui"]["artifact_sha256"]["main"] = json!(original_hash);
    fs::write(&file, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let manager = ccr_db::managers::ui_state::UiStateManager::with_db(
        ccr_db::database::DbAccess::Pool(pool.clone()),
    );
    manager
        .add_favorite(ccr_db::models::ui_state::AddFavoriteRequest {
            command: "synthetic-wal".into(),
            args: Vec::new(),
            display_name: None,
            module: "fixture".into(),
        })
        .unwrap();
    assert_eq!(manager.try_get_favorites().unwrap().len(), 1);
    assert_eq!(file_hash(&main).unwrap(), original_hash);
    let wal = main.with_file_name("ccr-ui.db-wal");
    assert!(fs::metadata(&wal).unwrap().len() > 0);
    let wal_bytes = fs::read(&wal).unwrap();
    assert!(
        Seed::load(&gpui.root, Endpoint::Gpui).is_err(),
        "unchecked committed WAL must not pass seed admission"
    );
    assert_eq!(file_hash(&main).unwrap(), original_hash);
    assert_eq!(
        fs::read(&wal).unwrap(),
        wal_bytes,
        "admission must not checkpoint/recover WAL"
    );
}

#[test]
#[ignore = "explicit existing generated pairs; filesystem validation only, no GUI or timings"]
fn generated_pairs_use_the_same_admission_owner_for_both_clients() {
    let root = PathBuf::from(std::env::var_os("CCR_NATIVE_BENCHMARK_PAIR_ROOT").unwrap());
    for scenario in ["S0", "S1", "S2"] {
        let gpui = Seed::load(&root.join(scenario).join("gpui"), Endpoint::Gpui).unwrap();
        let tauri = Seed::load(&root.join(scenario).join("tauri"), Endpoint::Tauri).unwrap();
        assert_eq!(gpui.as_of, tauri.as_of);
        assert_eq!(gpui.logical_sha256, tauri.logical_sha256);
        assert_eq!(gpui.profile_count, tauri.profile_count);
        assert_eq!(gpui.session_count, tauri.session_count);
        assert_eq!(gpui.usage_count, tauri.usage_count);
        println!(
            "{scenario}: both endpoints admitted, as_of {} UTC, logical {}",
            gpui.as_of, gpui.logical_sha256
        );
    }
}
