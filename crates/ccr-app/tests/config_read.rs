#![allow(clippy::unwrap_used, clippy::expect_used)]

use ccr_app::config::ConfigReadService;
use ccr_config::{ConfigManager, platforms::base};
use ccr_core::CcrError;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

const SECRET: &str = "synthetic-services-credential-never-in-row";
const PROFILES: &str = r#"
default_config = "z-last"
current_config = "ghost"
[z-last]
base_url = "https://fixture.invalid"
auth_token = "synthetic-services-credential-never-in-row"
model = "model-z"
small_fast_model = "fast-z"
provider = "fixture"
provider_type = "third_party_model"
account = "account"
tags = ["one", "two"]
usage_count = 7
enabled = false
[a-first]
description = "Second row"
base_url = "https://second.invalid"
auth_token = "synthetic-second-long-token"
enabled = true
"#;

fn isolated(name: &str) -> bool {
    if std::env::var("CCR_APP_CONFIG_CASE").as_deref() == Ok(name) {
        return false;
    }
    let temp = tempfile::tempdir().unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env("CCR_APP_CONFIG_CASE", name)
        .env("CCR_ROOT", temp.path().join("root"))
        .env("CCR_LOCK_DIR", temp.path().join("locks"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    true
}

fn paths(root: &Path) -> (PathBuf, PathBuf) {
    (root.join("profiles.toml"), root.join("config.toml"))
}

fn fingerprint(root: &Path) -> BTreeMap<PathBuf, (Vec<u8>, SystemTime)> {
    fn collect(root: &Path, out: &mut BTreeMap<PathBuf, (Vec<u8>, SystemTime)>) {
        for entry in fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(&path, out);
            } else {
                out.insert(
                    path.clone(),
                    (
                        fs::read(&path).unwrap(),
                        fs::metadata(&path).unwrap().modified().unwrap(),
                    ),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    collect(root, &mut out);
    out
}

fn assert_unchanged(root: &Path, before: &BTreeMap<PathBuf, (Vec<u8>, SystemTime)>) {
    let after = fingerprint(root);
    let changed: Vec<_> = before
        .keys()
        .chain(after.keys())
        .filter(|path| {
            if before.get(*path) == after.get(*path) {
                return false;
            }
            // Existing read-only domain connections retain live-WAL visibility.
            // SQLite may create these two sidecars; no business file may change.
            let sidecar = path.file_name().unwrap().to_string_lossy();
            let expected_size = match sidecar.as_ref() {
                "profiles.db-wal" => Some(0),
                "profiles.db-shm" => Some(32768),
                _ => None,
            };
            let new_sidecar = before.get(*path).is_none()
                && before.contains_key(&root.join("profiles.db"))
                && expected_size == after.get(*path).map(|(bytes, _)| bytes.len());
            // A read-only SQLite mapping may update SHM mtime when closing, even
            // though its bytes stay identical. All business file mtimes stay fixed.
            let unchanged_shm = sidecar == "profiles.db-shm"
                && before
                    .get(*path)
                    .zip(after.get(*path))
                    .is_some_and(|((left, _), (right, _))| left == right && right.len() == 32768);
            !(new_sidecar || unchanged_shm)
        })
        .map(|path| {
            let left = before.get(path);
            let right = after.get(path);
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                left.zip(right).map(|((a, at), (b, bt))| {
                    (
                        a == b,
                        at == bt,
                        a.iter().zip(b).filter(|(a, b)| a != b).count(),
                    )
                }),
            )
        })
        .collect();
    assert!(
        changed.is_empty(),
        "read changed business fixture files: {changed:?}"
    );
}

#[test]
fn config_list_keeps_revision_order_mask_defaults_and_registry_fallback_without_writes() {
    if isolated(
        "config_list_keeps_revision_order_mask_defaults_and_registry_fallback_without_writes",
    ) {
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    let (profiles, registry) = paths(temp.path());
    let manager = ConfigManager::new(&profiles);
    let revision = manager.replace_toml(PROFILES, "").unwrap();
    fs::write(&registry, "[claude]\ncurrent_profile = 'a-first'\n").unwrap();
    let before = fingerprint(temp.path());
    let service = ConfigReadService::from_paths(profiles.clone(), registry.clone());
    let rows = service.list().unwrap();
    assert_eq!(
        rows.iter().map(|row| row.name.as_str()).collect::<Vec<_>>(),
        ["z-last", "a-first"]
    );
    assert!(rows.iter().all(|row| row.version == revision));
    assert_eq!(rows[0].description, "");
    assert_eq!(rows[0].model.as_deref(), Some("model-z"));
    assert_eq!(rows[0].small_fast_model.as_deref(), Some("fast-z"));
    assert_eq!(rows[0].provider_type.as_deref(), Some("third_party_model"));
    assert_eq!(rows[0].provider.as_deref(), Some("fixture"));
    assert_eq!(rows[0].account.as_deref(), Some("account"));
    assert_eq!(
        rows[0].tags.as_deref(),
        Some(["one".to_string(), "two".to_string()].as_slice())
    );
    assert_eq!(rows[0].usage_count, 7);
    assert!(!rows[0].enabled);
    assert!(rows[0].is_default && !rows[0].is_current);
    assert!(rows[1].is_current && !rows[1].is_default);
    let source = manager.load().unwrap();
    assert_eq!(
        rows[0].auth_token,
        source.sections["z-last"]
            .auth_token
            .as_ref()
            .unwrap()
            .to_string()
    );
    assert!(!format!("{rows:?}").contains(SECRET));
    assert_eq!(
        base::read_file_current_profile_with_paths(&profiles, &registry, "claude")
            .unwrap()
            .current
            .as_deref(),
        Some("a-first")
    );
    assert_eq!(manager.snapshot().unwrap().version, revision);
    assert_unchanged(temp.path(), &before);
    let settled = fingerprint(temp.path());
    assert_eq!(service.list().unwrap(), rows);
    assert_unchanged(temp.path(), &settled);
}

#[test]
fn config_list_valid_file_marker_wins_missing_registry_is_not_created() {
    if isolated("config_list_valid_file_marker_wins_missing_registry_is_not_created") {
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    let (profiles, registry) = paths(temp.path());
    let manager = ConfigManager::new(&profiles);
    manager
        .replace_toml(&PROFILES.replace("ghost", "z-last"), "")
        .unwrap();
    let service = ConfigReadService::from_paths(profiles, registry.clone());
    let before = fingerprint(temp.path());
    assert!(service.list().unwrap()[0].is_current);
    assert!(!registry.exists());
    assert_unchanged(temp.path(), &before);
    fs::write(&registry, "[claude]\ncurrent_profile = 'a-first'\n").unwrap();
    let before = fingerprint(temp.path());
    assert!(service.list().unwrap()[0].is_current);
    assert_unchanged(temp.path(), &before);
}

#[test]
fn config_list_empty_catalog_is_success_but_missing_is_error() {
    if isolated("config_list_empty_catalog_is_success_but_missing_is_error") {
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    let (profiles, registry) = paths(temp.path());
    let service = ConfigReadService::from_paths(profiles.clone(), registry);
    assert!(matches!(service.list(), Err(CcrError::ConfigMissing(_))));
    assert!(fingerprint(temp.path()).is_empty());
    ConfigManager::new(profiles)
        .replace_toml("default_config = ''\ncurrent_config = ''\n", "")
        .unwrap();
    let before = fingerprint(temp.path());
    assert!(service.list().unwrap().is_empty());
    assert_unchanged(temp.path(), &before);
}

#[test]
fn config_list_does_not_migrate_legacy_or_repair_corrupt_catalog() {
    let temp = tempfile::tempdir().unwrap();
    let (profiles, registry) = paths(temp.path());
    let manager = ConfigManager::new(&profiles);
    let service = ConfigReadService::from_paths(profiles.clone(), registry);
    fs::write(&profiles, PROFILES).unwrap();
    let before = fingerprint(temp.path());
    assert!(
        service
            .list()
            .unwrap_err()
            .to_string()
            .contains("配置尚未迁移")
    );
    assert_unchanged(temp.path(), &before);
    fs::remove_file(&profiles).unwrap();
    fs::write(manager.profiles_db(), b"not a database").unwrap();
    let before = fingerprint(temp.path());
    assert!(service.list().is_err());
    assert_unchanged(temp.path(), &before);
}

#[test]
fn config_list_propagates_registry_parse_error_without_repair() {
    if isolated("config_list_propagates_registry_parse_error_without_repair") {
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    let (profiles, registry) = paths(temp.path());
    ConfigManager::new(&profiles)
        .replace_toml(PROFILES, "")
        .unwrap();
    fs::write(&registry, "[claude\ninvalid").unwrap();
    let before = fingerprint(temp.path());
    assert!(
        ConfigReadService::from_paths(profiles, registry)
            .list()
            .is_err()
    );
    assert_unchanged(temp.path(), &before);
}

/// Explicit evidence-data generator, never part of normal acceptance test counts.
#[test]
#[ignore = "run explicitly to create a synthetic native verification fixture"]
fn seed_native_fixture() {
    use ccr_config::managers::{PlatformConfigManager, UnifiedConfig};
    let requested =
        PathBuf::from(std::env::var_os("CCR_APP_FIXTURE_OUTPUT").expect("fixture output required"));
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .canonicalize()
        .unwrap();
    let allowed = repo
        .join("outputs/desktop/services")
        .canonicalize()
        .expect("create evidence directory first");
    assert!(requested.is_absolute());
    assert_eq!(requested.parent().unwrap().canonicalize().unwrap(), allowed);
    assert!(
        !requested.exists(),
        "refusing to overwrite any existing fixture"
    );
    assert_eq!(
        std::env::var_os("CCR_LOCK_DIR").map(PathBuf::from),
        Some(requested.join("locks"))
    );
    fs::create_dir(&requested).unwrap();
    let (profiles, registry) = paths(&requested);
    let manager = ConfigManager::new(profiles);
    let revision = manager.replace_toml(PROFILES, "").unwrap();
    let mut unified = UnifiedConfig::default();
    unified.get_platform_mut("claude").unwrap().current_profile = Some("a-first".into());
    PlatformConfigManager::new(&registry)
        .save(&unified)
        .unwrap();
    let hash = |path: &Path| ccr_core::core::content_version_token(&fs::read(path).unwrap());
    let manifest = format!(
        "fixture = 'CCR GPUI services synthetic evidence'\nschema = 1\nrevision = '{revision}'\nhash_algorithm = 'blake3'\nprofiles_db_hash = '{}'\nregistry_hash = '{}'\ncredentials = 'synthetic only; display must mask'\n",
        hash(manager.profiles_db()),
        hash(&registry)
    );
    fs::write(requested.join("fixture-manifest.toml"), manifest).unwrap();
    println!("synthetic fixture: {}", requested.display());
}
