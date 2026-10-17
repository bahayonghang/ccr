#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{
    CatalogTransaction, MigrationFailpoint, document_probe, fail_next_catalog_commit,
    increment_usage_count, migrate_paths, profile_names, profile_probe, schema_user_version,
};
use crate::managers::config::ConfigManager;
use crate::models::{Platform, PlatformPaths};
use crate::platforms::base::{self, update_current_config};
use crate::test_support::TestCcrEnv;
use ccr_core::CcrError;
use std::fs;
use std::path::{Path, PathBuf};

fn paths(platform: Platform) -> PlatformPaths {
    PlatformPaths::new(platform).unwrap()
}

fn write_legacy(platform: Platform, content: &str) -> PathBuf {
    let profiles = paths(platform);
    fs::create_dir_all(&profiles.platform_dir).unwrap();
    fs::write(&profiles.profiles_file, content).unwrap();
    profiles.profiles_file
}

fn assert_not_migrated(error: &CcrError) {
    let message = error.to_string();
    assert!(message.contains("尚未迁移"), "{message}");
    assert!(!message.contains("secret-sentinel"), "{message}");
}

#[test]
fn migrate_creates_nothing_when_catalog_and_legacy_are_absent() {
    let env = TestCcrEnv::new();
    let before = fs::read_dir(env.root()).unwrap().count();
    super::migrate_catalogs().unwrap();
    assert_eq!(fs::read_dir(env.root()).unwrap().count(), before);
    assert!(!paths(Platform::Claude).profiles_db().exists());
    assert!(!paths(Platform::Qwen).platform_dir.exists());
}

#[test]
fn direct_load_of_legacy_toml_is_not_migrated_and_creates_no_database() {
    let _env = TestCcrEnv::new();
    let legacy = write_legacy(
        Platform::Claude,
        "[main]\ndescription = \"kept\"\nauth_token = \"secret-sentinel\"\n",
    );
    let manager = ConfigManager::for_platform("claude").unwrap();
    assert_not_migrated(&manager.load().unwrap_err());
    assert_not_migrated(
        &manager
            .snapshot()
            .err()
            .expect("legacy toml must not be read"),
    );
    assert_not_migrated(&base::load_profiles_from_toml(&legacy).unwrap_err());
    assert!(!manager.profiles_db().exists());
    assert!(legacy.is_file());
}

#[test]
fn migration_imports_five_platforms_and_skips_qwen() {
    let _env = TestCcrEnv::new();
    write_legacy(
        Platform::Claude,
        "[beta]\ndescription = \"second\"\nusage_count = 2\nenabled = true\n[alpha]\ndescription = \"first\"\nusage_count = 1\n",
    );
    write_legacy(Platform::Codex, "[codex]\nauth_mode = \"chatgpt\"\n");
    write_legacy(
        Platform::Grok,
        "[grok]\napi_key = \"secret-sentinel\"\nbase_url = \"https://api.x.ai/v1\"\n",
    );
    write_legacy(
        Platform::Gemini,
        "[gemini]\nauth_token = \"gemini-token\"\nbase_url = \"https://generativelanguage.googleapis.com\"\n",
    );
    write_legacy(
        Platform::Droid,
        "[droid]\ndisplay_name = \"Factory\"\nusage_count = 4\n",
    );
    let qwen = paths(Platform::Qwen);
    fs::create_dir_all(&qwen.platform_dir).unwrap();
    fs::write(&qwen.profiles_file, "[qwen]\ndescription = \"untouched\"\n").unwrap();

    super::migrate_catalogs().unwrap();

    for platform in [
        Platform::Claude,
        Platform::Codex,
        Platform::Grok,
        Platform::Gemini,
        Platform::Droid,
    ] {
        let profiles = paths(platform);
        assert!(!profiles.profiles_file.exists(), "{}", platform);
        assert!(profiles.profiles_db().is_file(), "{}", platform);
        assert!(!super::attempt_file(&profiles.profiles_db()).exists());
        let document = document_probe(&profiles.profiles_db()).unwrap();
        assert_eq!(document.phase, "source_retired");
        assert!(document.migrated_at.is_some());
        assert_eq!(schema_user_version(&profiles.profiles_db()).unwrap(), 1);
    }
    assert_eq!(
        profile_names(&paths(Platform::Claude).profiles_db()).unwrap(),
        vec!["beta".to_string(), "alpha".to_string()]
    );
    let grok = profile_probe(&paths(Platform::Grok).profiles_db(), "grok")
        .unwrap()
        .unwrap();
    assert!(grok.auth_token.is_none());
    assert!(grok.extra_toml.contains("api_key"));
    assert!(grok.extra_toml.contains("secret-sentinel"));
    let gemini = profile_probe(&paths(Platform::Gemini).profiles_db(), "gemini")
        .unwrap()
        .unwrap();
    assert!(gemini.usage_count.is_none());
    assert!(gemini.enabled.is_none());
    assert!(qwen.profiles_file.is_file());
    assert!(!qwen.profiles_db().exists());
}

#[test]
fn migration_preserves_null_markers_datetime_and_sort_order() {
    let _env = TestCcrEnv::new();
    write_legacy(
        Platform::Claude,
        "[zeta]\ndescription = \"later\"\ncustom_timestamp = 2026-01-01T00:00:00Z\nusage_count = 3\n[alpha]\ndescription = \"earlier\"\n",
    );
    migrate_paths(&paths(Platform::Claude), None).unwrap();
    let database = paths(Platform::Claude).profiles_db();
    let document = document_probe(&database).unwrap();
    assert!(document.current.is_none());
    assert!(document.default_config.is_none());
    assert!(document.settings_toml.is_none());
    assert_eq!(
        profile_names(&database).unwrap(),
        vec!["zeta".to_string(), "alpha".to_string()]
    );
    let zeta = profile_probe(&database, "zeta").unwrap().unwrap();
    assert_eq!(zeta.sort_order, 0);
    assert_eq!(zeta.usage_count, Some(3));
    let alpha = profile_probe(&database, "alpha").unwrap().unwrap();
    assert_eq!(alpha.sort_order, 1);
    assert!(alpha.usage_count.is_none());
    assert!(alpha.enabled.is_none());

    let loaded = ConfigManager::for_platform("claude")
        .unwrap()
        .load()
        .unwrap();
    let original = loaded.sections["zeta"].other["custom_timestamp"].clone();
    assert!(matches!(original, toml::Value::Datetime(_)));
    ConfigManager::for_platform("claude")
        .unwrap()
        .mutate(|config| {
            config.get_section_mut("zeta")?.description = Some("updated".into());
            Ok(())
        })
        .unwrap();
    let after = ConfigManager::for_platform("claude")
        .unwrap()
        .load()
        .unwrap();
    assert_eq!(after.sections["zeta"].other["custom_timestamp"], original);
    assert!(document_probe(&database).unwrap().current.is_none());
}

#[test]
fn empty_current_stays_distinct_from_an_absent_marker() {
    let _env = TestCcrEnv::new();
    write_legacy(
        Platform::Codex,
        "default_config = \"main\"\ncurrent_config = \"\"\n\n[main]\ndescription = \"present\"\n",
    );
    migrate_paths(&paths(Platform::Codex), None).unwrap();
    let database = paths(Platform::Codex).profiles_db();
    let document = document_probe(&database).unwrap();
    assert_eq!(document.current.as_deref(), Some(""));
    assert_eq!(document.default_config.as_deref(), Some("main"));
    let loaded = ConfigManager::for_platform("codex")
        .unwrap()
        .load()
        .unwrap();
    assert!(loaded.current_config.is_empty());
    ConfigManager::for_platform("codex")
        .unwrap()
        .mutate(|config| {
            config.get_section_mut("main")?.description = Some("edited".into());
            Ok(())
        })
        .unwrap();
    assert_eq!(
        document_probe(&database).unwrap().current.as_deref(),
        Some("")
    );
}

#[test]
fn migration_failpoints_keep_the_authoritative_state_machine() {
    let _env = TestCcrEnv::new();
    let profiles = paths(Platform::Claude);
    let body =
        "[main]\ndescription = \"stable\"\nauth_token = \"secret-sentinel\"\nusage_count = 7\n";

    write_legacy(Platform::Claude, body);
    let error = migrate_paths(
        &profiles,
        Some(MigrationFailpoint::BeforeAuthoritativeCommit),
    )
    .unwrap_err();
    assert!(error.to_string().contains("权威提交前"));
    assert!(profiles.profiles_file.is_file());
    assert!(!profiles.profiles_db().exists());
    assert!(!super::attempt_file(&profiles.profiles_db()).exists());
    assert!(!super::with_suffix(&profiles.profiles_db(), "-wal").exists());

    write_legacy(Platform::Claude, body);
    let error = migrate_paths(
        &profiles,
        Some(MigrationFailpoint::AfterAuthoritativeCommitBeforeMove),
    )
    .unwrap_err();
    assert!(error.to_string().contains("移动遗留配置前"));
    assert!(profiles.profiles_file.is_file());
    let document = document_probe(&profiles.profiles_db()).unwrap();
    assert_eq!(document.phase, "data_authoritative");
    assert!(document.migrated_at.is_none());
    assert_eq!(
        profile_probe(&profiles.profiles_db(), "main")
            .unwrap()
            .unwrap()
            .usage_count,
        Some(7)
    );
    fs::write(
        &profiles.profiles_file,
        "[other]\ndescription = \"replacement\"\n",
    )
    .unwrap();
    migrate_paths(&profiles, None).unwrap();
    assert!(!profiles.profiles_file.exists());
    assert_eq!(
        document_probe(&profiles.profiles_db()).unwrap().phase,
        "source_retired"
    );
    assert!(
        profile_probe(&profiles.profiles_db(), "main")
            .unwrap()
            .is_some()
    );
    assert!(
        profile_probe(&profiles.profiles_db(), "other")
            .unwrap()
            .is_none()
    );

    fs::remove_dir_all(&profiles.platform_dir).unwrap();
    fs::remove_dir_all(&profiles.backups_dir).ok();
    write_legacy(Platform::Claude, body);
    let error = migrate_paths(&profiles, Some(MigrationFailpoint::DuringMove)).unwrap_err();
    assert!(error.to_string().contains("移动遗留配置时"));
    assert!(profiles.profiles_file.is_file());
    assert!(profiles.backups_dir.is_dir());
    assert_eq!(
        document_probe(&profiles.profiles_db()).unwrap().phase,
        "data_authoritative"
    );

    fs::remove_dir_all(&profiles.platform_dir).unwrap();
    fs::remove_dir_all(&profiles.backups_dir).unwrap();
    write_legacy(Platform::Claude, body);
    let error = migrate_paths(&profiles, Some(MigrationFailpoint::MoveFails)).unwrap_err();
    assert!(error.to_string().contains("移动遗留配置失败"));
    assert!(profiles.profiles_file.is_file());
    assert!(!profiles.backups_dir.exists());
    migrate_paths(&profiles, None).unwrap();
    assert!(!profiles.profiles_file.exists());
    assert_eq!(
        document_probe(&profiles.profiles_db()).unwrap().phase,
        "source_retired"
    );

    fs::remove_dir_all(&profiles.platform_dir).unwrap();
    fs::remove_dir_all(&profiles.backups_dir).unwrap();
    write_legacy(Platform::Claude, body);
    let error = migrate_paths(
        &profiles,
        Some(MigrationFailpoint::AfterMoveBeforeSourceRetired),
    )
    .unwrap_err();
    assert!(error.to_string().contains("停用遗留配置前"));
    assert!(!profiles.profiles_file.exists());
    let revision = document_probe(&profiles.profiles_db()).unwrap().revision;
    assert_eq!(
        document_probe(&profiles.profiles_db()).unwrap().phase,
        "data_authoritative"
    );
    migrate_paths(&profiles, None).unwrap();
    let retired = document_probe(&profiles.profiles_db()).unwrap();
    assert_eq!(retired.phase, "source_retired");
    assert_eq!(retired.revision, revision);
    assert!(retired.migrated_at.is_some());
}

#[test]
fn existing_database_without_attempt_marker_is_left_untouched() {
    let _env = TestCcrEnv::new();
    let profiles = paths(Platform::Droid);
    fs::create_dir_all(&profiles.platform_dir).unwrap();
    fs::write(profiles.profiles_db(), b"not-a-database").unwrap();
    fs::write(
        &profiles.profiles_file,
        "[main]\ndescription = \"legacy\"\n",
    )
    .unwrap();
    let before = fs::read(profiles.profiles_db()).unwrap();
    let error = migrate_paths(&profiles, None).unwrap_err();
    assert!(!error.to_string().contains("legacy"));
    assert_eq!(fs::read(profiles.profiles_db()).unwrap(), before);
    assert!(profiles.profiles_file.is_file());
}

#[test]
fn incomplete_attempt_database_is_recycled_and_reimported() {
    let _env = TestCcrEnv::new();
    let profiles = paths(Platform::Gemini);
    fs::create_dir_all(&profiles.platform_dir).unwrap();
    fs::write(profiles.profiles_db(), b"incomplete").unwrap();
    fs::write(
        super::attempt_file(&profiles.profiles_db()),
        "previous-attempt",
    )
    .unwrap();
    fs::write(
        &profiles.profiles_file,
        "[gemini]\ndescription = \"imported\"\n",
    )
    .unwrap();
    migrate_paths(&profiles, None).unwrap();
    assert!(!profiles.profiles_file.exists());
    assert_eq!(
        profile_probe(&profiles.profiles_db(), "gemini")
            .unwrap()
            .unwrap()
            .extra_toml,
        ""
    );
    let loaded = ConfigManager::for_platform("gemini")
        .unwrap()
        .load()
        .unwrap();
    assert_eq!(
        loaded.sections["gemini"].description.as_deref(),
        Some("imported")
    );
}

#[test]
fn source_retired_reads_do_not_open_legacy_toml() {
    let _env = TestCcrEnv::new();
    write_legacy(Platform::Claude, "[main]\ndescription = \"from-db\"\n");
    migrate_paths(&paths(Platform::Claude), None).unwrap();
    let profiles = paths(Platform::Claude);
    fs::write(&profiles.profiles_file, "this is not toml [").unwrap();
    let loaded = base::load_profiles_from_toml(&profiles.profiles_file).unwrap();
    assert_eq!(loaded["main"].description.as_deref(), Some("from-db"));
    assert_eq!(
        base::load_current_profile_marker(&profiles.profiles_file).unwrap(),
        None
    );
}

#[test]
fn catalog_transaction_joins_inner_load_mutate_and_current_update() {
    let _env = TestCcrEnv::new();
    write_legacy(
        Platform::Claude,
        "default_config = \"main\"\ncurrent_config = \"main\"\n\n[main]\ndescription = \"before\"\nenabled = false\nusage_count = 2\n",
    );
    migrate_paths(&paths(Platform::Claude), None).unwrap();
    let manager = ConfigManager::for_platform("claude").unwrap();
    let database = manager.profiles_db().to_path_buf();
    let legacy = manager.config_path().to_path_buf();
    let error = CatalogTransaction::execute(&database, |_| {
        manager.mutate(|config| {
            let mut section = config.remove_section("main")?;
            section.enabled = Some(true);
            section.description = Some("renamed-visible".into());
            config.sections.insert("renamed".into(), section);
            config.current_config.clear();
            config.default_config = "renamed".into();
            Ok(())
        })?;
        let visible = manager.load()?;
        assert!(visible.sections.contains_key("renamed"));
        assert_eq!(visible.sections["renamed"].enabled, Some(true));
        assert!(!visible.sections.contains_key("main"));
        update_current_config(&legacy, "renamed")?;
        assert_eq!(
            base::load_current_profile_marker(&legacy)?.as_deref(),
            Some("renamed")
        );
        let conflict = manager.mutate(|config| {
            super::bump_revision_for_test(&database)?;
            config.get_section_mut("renamed")?.description = Some("rejected".into());
            Ok(())
        });
        assert!(conflict.unwrap_err().to_string().contains("版本冲突"));
        Err::<(), _>(CcrError::ConfigError("rollback fixture".into()))
    })
    .unwrap_err();
    assert!(error.to_string().contains("rollback fixture"));
    let restored = manager.load().unwrap();
    assert!(restored.sections.contains_key("main"));
    assert_eq!(restored.sections["main"].enabled, Some(false));
    assert_eq!(restored.sections["main"].usage_count, Some(2));
    assert_eq!(
        base::load_current_profile_marker(&legacy)
            .unwrap()
            .as_deref(),
        Some("main")
    );
}

#[test]
fn no_op_mutate_does_not_advance_revision() {
    let _env = TestCcrEnv::new();
    write_legacy(Platform::Claude, "[main]\ndescription = \"same\"\n");
    migrate_paths(&paths(Platform::Claude), None).unwrap();
    let manager = ConfigManager::for_platform("claude").unwrap();
    let before = document_probe(manager.profiles_db()).unwrap().revision;
    manager.mutate(|_| Ok(())).unwrap();
    assert_eq!(
        document_probe(manager.profiles_db()).unwrap().revision,
        before
    );
}

#[test]
fn catalog_files_are_owner_only_and_debug_hides_secrets() {
    let _env = TestCcrEnv::new();
    write_legacy(
        Platform::Claude,
        "[main]\nauth_token = \"secret-sentinel\"\ndescription = \"hidden\"\n",
    );
    migrate_paths(&paths(Platform::Claude), None).unwrap();
    let database = paths(Platform::Claude).profiles_db();
    assert_owner_only(&database);
    for suffix in ["-wal", "-shm"] {
        let path = super::with_suffix(&database, suffix);
        if path.exists() {
            assert_owner_only(&path);
        }
    }
    let loaded = ConfigManager::for_platform("claude")
        .unwrap()
        .load()
        .unwrap();
    let debug = format!("{:?}", loaded.sections["main"]);
    assert!(!debug.contains("secret-sentinel"));
    let connection = rusqlite::Connection::open(&database).unwrap();
    assert!(
        connection
            .execute(
                "UPDATE document SET migration_phase = 'nope' WHERE id = 1",
                [],
            )
            .is_err()
    );
}

#[test]
fn usage_increment_promotes_null_and_saturates() {
    let _env = TestCcrEnv::new();
    write_legacy(
        Platform::Claude,
        "[fresh]\ndescription = \"new\"\n\n[full]\ndescription = \"cap\"\nusage_count = 4294967295\n",
    );
    migrate_paths(&paths(Platform::Claude), None).unwrap();
    let database = paths(Platform::Claude).profiles_db();
    increment_usage_count(&database, "fresh").unwrap();
    increment_usage_count(&database, "full").unwrap();
    let fresh = profile_probe(&database, "fresh").unwrap().unwrap();
    let full = profile_probe(&database, "full").unwrap().unwrap();
    assert_eq!(fresh.usage_count, Some(1));
    assert_eq!(full.usage_count, Some(4_294_967_295));
}

#[test]
fn commit_failure_rolls_back_the_open_transaction() {
    let _env = TestCcrEnv::new();
    write_legacy(Platform::Claude, "[main]\ndescription = \"before\"\n");
    migrate_paths(&paths(Platform::Claude), None).unwrap();
    let database = paths(Platform::Claude).profiles_db();
    let before = document_probe(&database).unwrap().revision;
    fail_next_catalog_commit();
    let error = CatalogTransaction::execute(&database, |transaction| {
        ConfigManager::for_platform("claude")?.mutate(|config| {
            config.current_config = "main".into();
            Ok(())
        })?;
        transaction.store_applied_operation("op-1", "payload")?;
        Ok(())
    })
    .unwrap_err();
    assert!(error.to_string().contains("目录提交失败"), "{error}");
    assert_eq!(document_probe(&database).unwrap().revision, before);
    assert!(
        super::load_applied_operation(&database, "op-1")
            .unwrap()
            .is_none()
    );
    assert!(document_probe(&database).unwrap().current.is_none());
}

#[test]
fn canonical_toml_round_trips_markers_and_datetime_extras() {
    let _env = TestCcrEnv::new();
    let manager = ConfigManager::for_platform("claude").unwrap();
    let source = "
note = \"kept\"
retired_at = 2024-01-02T03:04:05Z

[demo]
model = \"m\"
auth_token = \"secret-sentinel\"
enabled = false
usage_count = 0
seen_at = 2024-05-06T07:08:09Z
";
    let revision = manager.replace_toml(source, "").unwrap();
    let (rendered, token) = manager.canonical_toml().unwrap().unwrap();
    assert_eq!(token, revision);
    assert!(!rendered.contains("current_config"), "{rendered}");
    assert!(!rendered.contains("default_config"), "{rendered}");
    assert!(rendered.contains("note = \"kept\""), "{rendered}");
    assert!(rendered.contains("2024-01-02T03:04:05Z"), "{rendered}");
    assert!(rendered.contains("2024-05-06T07:08:09Z"), "{rendered}");
    assert!(rendered.contains("usage_count = 0"), "{rendered}");
    assert!(rendered.contains("enabled = false"), "{rendered}");
    assert!(!manager.config_path().exists());
    assert!(manager.profiles_db().is_file());

    let again = manager.replace_toml(&rendered, &token).unwrap();
    let (rendered_again, token_again) = manager.canonical_toml().unwrap().unwrap();
    assert_eq!(rendered_again, rendered);
    assert_eq!(token_again, again);
    assert_ne!(token_again, token);
    assert_eq!(
        document_probe(manager.profiles_db()).unwrap().phase,
        "source_retired"
    );

    let empty = "current_config = \"\"\ndefault_config = \"\"\n\n[demo]\nmodel = \"m\"\n";
    let empty_revision = manager.replace_toml(empty, &token_again).unwrap();
    let (empty_text, empty_token) = manager.canonical_toml().unwrap().unwrap();
    assert_eq!(empty_token, empty_revision);
    assert!(empty_text.contains("current_config = \"\""), "{empty_text}");
    assert!(empty_text.contains("default_config = \"\""), "{empty_text}");
    let conflict = manager.replace_toml(empty, "stale").unwrap_err();
    assert!(conflict.to_string().contains("配置版本冲突"), "{conflict}");
    let (kept, kept_token) = manager.canonical_toml().unwrap().unwrap();
    assert_eq!(kept, empty_text);
    assert_eq!(kept_token, empty_revision);
    assert!(!manager.config_path().exists());

    let legacy = write_legacy(Platform::Codex, "[codex]\nmodel = \"x\"\n");
    let codex = ConfigManager::for_platform("codex").unwrap();
    assert_not_migrated(&codex.canonical_toml().unwrap_err());
    assert_not_migrated(
        &codex
            .replace_toml("[codex]\nmodel = \"y\"\n", "")
            .unwrap_err(),
    );
    assert!(legacy.is_file());
    assert!(!codex.profiles_db().exists());

    let gemini = ConfigManager::for_platform("gemini").unwrap();
    assert!(gemini.canonical_toml().unwrap().is_none());
    let missing_token = gemini
        .replace_toml("[gemini]\nmodel = \"m\"\n", "stale")
        .unwrap_err();
    assert!(
        missing_token.to_string().contains("配置版本冲突"),
        "{missing_token}"
    );
    assert!(!gemini.profiles_db().exists());
    assert!(!gemini.config_path().exists());
}

#[test]
fn replace_toml_joins_the_open_transaction() {
    let _env = TestCcrEnv::new();
    let manager = ConfigManager::for_platform("claude").unwrap();
    let revision = manager.replace_toml("[main]\nmodel = \"a\"\n", "").unwrap();
    let database = manager.profiles_db().to_path_buf();
    let error = CatalogTransaction::execute(&database, |_| -> crate::Result<()> {
        manager.replace_toml("[main]\nmodel = \"b\"\n", &revision)?;
        Err(CcrError::ConfigError("stop".into()))
    })
    .unwrap_err();
    assert!(error.to_string().contains("stop"), "{error}");
    let (text, token) = manager.canonical_toml().unwrap().unwrap();
    assert_eq!(token, revision);
    assert!(text.contains("model = \"a\""), "{text}");
    assert!(!text.contains("model = \"b\""), "{text}");
    assert!(!manager.config_path().exists());
}

fn assert_owner_only(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600,
            "{}",
            path.display()
        );
    }
    #[cfg(windows)]
    assert_windows_owner_only_dacl(path);
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        eprintln!("NOT_RUN owner-only permission assertion");
    }
}

#[cfg(windows)]
fn assert_windows_owner_only_dacl(path: &Path) {
    let descriptor = windows_dacl(path);
    let control = u16::from_le_bytes(descriptor[2..4].try_into().unwrap());
    assert_ne!(control & 0x1000, 0, "the DACL must be protected");
    let offset = u32::from_le_bytes(descriptor[16..20].try_into().unwrap()) as usize;
    let ace_count = u16::from_le_bytes(descriptor[offset + 4..offset + 6].try_into().unwrap());
    assert_eq!(ace_count, 1, "only the process user receives access");
    let ace = &descriptor[offset + 8..];
    assert_eq!(ace[0], 0, "the single ACE must allow access");
    assert_eq!(ace[1], 0, "the ACE must not be inherited");
    let ace_size = u16::from_le_bytes(ace[2..4].try_into().unwrap()) as usize;
    let mask = u32::from_le_bytes(ace[4..8].try_into().unwrap());
    assert_eq!(mask, 0x001f_01ff, "the process user receives full control");
    assert!(
        ace[8..ace_size] == windows_process_user_sid(),
        "the ACE must match the process token user"
    );
}

#[cfg(windows)]
fn windows_dacl(path: &Path) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt;
    let wide = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn GetFileSecurityW(
            file_name: *const u16,
            requested_information: u32,
            security_descriptor: *mut std::ffi::c_void,
            length: u32,
            length_needed: *mut u32,
        ) -> i32;
    }
    let mut needed = 0_u32;
    unsafe {
        GetFileSecurityW(
            wide.as_ptr(),
            0x0000_0004,
            std::ptr::null_mut(),
            0,
            &mut needed,
        );
    }
    assert_ne!(needed, 0);
    let mut descriptor = vec![0_u8; needed as usize];
    let ok = unsafe {
        GetFileSecurityW(
            wide.as_ptr(),
            0x0000_0004,
            descriptor.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    };
    assert_ne!(ok, 0);
    descriptor
}

#[cfg(windows)]
fn windows_process_user_sid() -> Vec<u8> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    }
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn OpenProcessToken(
            process: *mut std::ffi::c_void,
            access: u32,
            token: *mut *mut std::ffi::c_void,
        ) -> i32;
        fn GetTokenInformation(
            token: *mut std::ffi::c_void,
            class: u32,
            data: *mut std::ffi::c_void,
            length: u32,
            needed: *mut u32,
        ) -> i32;
        fn GetLengthSid(sid: *const std::ffi::c_void) -> u32;
    }
    unsafe {
        let mut token = std::ptr::null_mut();
        assert_ne!(OpenProcessToken(GetCurrentProcess(), 0x0008, &mut token), 0);
        let mut needed = 0;
        GetTokenInformation(token, 1, std::ptr::null_mut(), 0, &mut needed);
        let mut info = vec![0_usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
        let ok = GetTokenInformation(token, 1, info.as_mut_ptr().cast(), needed, &mut needed);
        CloseHandle(token);
        assert_ne!(ok, 0);
        let sid = *(info.as_ptr().cast::<*const std::ffi::c_void>());
        std::slice::from_raw_parts(sid.cast::<u8>(), GetLengthSid(sid) as usize).to_vec()
    }
}

#[test]
fn snapshot_installs_into_a_new_file_and_failed_replace_keeps_revision() {
    let env = TestCcrEnv::new();
    let profiles = paths(Platform::Claude);
    fs::create_dir_all(&profiles.platform_dir).unwrap();
    let revision = ConfigManager::new(&profiles.profiles_file)
        .replace_toml("[demo]\nauth_token = \"secret-sentinel\"\n", "")
        .unwrap();
    let bytes = super::snapshot_profile_catalog(&profiles.profiles_db()).unwrap();
    assert!(!bytes.is_empty());
    assert_eq!(
        document_probe(&profiles.profiles_db()).unwrap().revision,
        revision
    );
    assert!(!profiles.profiles_file.exists());

    let staging = env.root().join("staging.db");
    fs::write(&staging, &bytes).unwrap();
    let fresh = env.root().join("fresh.db");
    super::install_profile_catalog(&fresh, &staging).unwrap();
    assert_eq!(document_probe(&fresh).unwrap().revision, revision);

    let source = env.root().join("source");
    fs::create_dir_all(&source).unwrap();
    let source_revision = ConfigManager::new(source.join("profiles.toml"))
        .replace_toml("[other]\nauth_token = \"secret-sentinel\"\n", "")
        .unwrap();
    assert_ne!(source_revision, revision);
    fs::write(
        &staging,
        super::snapshot_profile_catalog(&source.join("profiles.db")).unwrap(),
    )
    .unwrap();
    let open = rusqlite::Connection::open(profiles.profiles_db()).unwrap();
    open.execute(
        "INSERT INTO applied_operations (operation_id, payload) VALUES ('op-1', 'kept')",
        [],
    )
    .unwrap();
    super::fail_next_profile_catalog_install();
    let error = super::install_profile_catalog(&profiles.profiles_db(), &staging).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("配置目录安装提交失败"), "{message}");
    assert!(!message.contains("secret-sentinel"), "{message}");
    assert_eq!(
        document_probe(&profiles.profiles_db()).unwrap().revision,
        revision
    );
    let payload: String = open
        .query_row(
            "SELECT payload FROM applied_operations WHERE operation_id = 'op-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(payload, "kept");
}

#[test]
fn live_read_sees_uncheckpointed_wal_commit() {
    let _env = TestCcrEnv::new();
    write_legacy(Platform::Claude, "[main]\ndescription = \"kept\"\n");
    super::migrate_catalogs().unwrap();
    let profiles = paths(Platform::Claude);
    let before = document_probe(&profiles.profiles_db()).unwrap();
    let writer = rusqlite::Connection::open(profiles.profiles_db()).unwrap();
    writer
        .execute_batch("PRAGMA wal_autocheckpoint = 0")
        .unwrap();
    writer.execute_batch("BEGIN IMMEDIATE").unwrap();
    writer
        .execute(
            "UPDATE document SET current_config = 'wal-current', revision = 'rev-wal' WHERE id = 1",
            [],
        )
        .unwrap();
    writer.execute_batch("COMMIT").unwrap();
    let wal = profiles.profiles_db().with_file_name("profiles.db-wal");
    assert!(wal.is_file());
    assert!(fs::metadata(&wal).unwrap().len() > 0);

    let manager = ConfigManager::for_platform("claude").unwrap();
    let loaded = manager.load().unwrap();
    assert_eq!(loaded.current_config, "wal-current");
    let (toml, revision) = manager.canonical_toml().unwrap().unwrap();
    assert_eq!(revision, "rev-wal");
    assert!(toml.contains("wal-current"));

    let uri = super::sqlite_read_uri(&profiles.profiles_db()).unwrap();
    let stale = rusqlite::Connection::open_with_flags(
        &uri,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .unwrap();
    let current: Option<String> = stale
        .query_row(
            "SELECT current_config FROM document WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(current, before.current);
    drop(writer);
}
