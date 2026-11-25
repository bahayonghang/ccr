use super::*;
use ccr_core::Secret;
use ccr_db::{
    database::{self, DbAccess},
    models::checkin::{CreateAccountRequest, CreateProviderRequest, UpdateAccountRequest},
};
fn apply(
    session: &ImportSession,
    db: &Path,
    crypto: &Path,
    overwrite: bool,
    open: impl FnOnce() -> Result<DbAccess>,
) -> Result<ImportReceipt> {
    ccr_core::core::LockManager::with_scoped_directory(&crypto.join("test-locks"), || {
        Ok(ExportManager::apply_file_at(
            session, db, crypto, overwrite, open,
        ))
    })
    .unwrap()
}

#[test]
fn preview_without_database_leaves_target_absent_and_binds_its_identity() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("destination");
    let db = target.join("db/checkin.db");
    let crypto = target.join("crypto");
    let source = root.path().join("source.json");
    let provider = crate::models::checkin::CheckinProvider::new(
        "Example".into(),
        "https://example.invalid".into(),
    );
    let data = ExportData::new(vec![provider], vec![]);
    std::fs::write(&source, serde_json::to_vec(&data).unwrap()).unwrap();
    let session = ExportManager::preview_file_at(&source, &db, &crypto).unwrap();
    assert!(session.target_missing());
    assert_eq!(session.preview.new_providers, 1);
    drop(session);
    assert!(!target.exists());
}

#[test]
fn confirmed_import_checks_target_and_source_before_initializing_and_reports_backup() {
    let root = tempfile::tempdir().unwrap();
    let db = root.path().join("db/checkin.db");
    let crypto = root.path().join("crypto");
    let source = root.path().join("source.json");
    let data = ExportData::new(
        vec![CheckinProvider::new(
            "Example".into(),
            "https://example.invalid".into(),
        )],
        vec![],
    );
    let bytes = serde_json::to_vec(&data).unwrap();
    std::fs::write(&source, &bytes).unwrap();
    let session = ExportManager::preview_file_at(&source, &db, &crypto).unwrap();
    let never_open = || panic!("stale input cannot initialize a database");
    assert!(matches!(
        apply(&session, &db, &root.path().join("other"), false, never_open),
        Err(ExportError::Conflict)
    ));
    assert!(matches!(
        apply(
            &session,
            &root.path().join("other.db"),
            &crypto,
            false,
            never_open
        ),
        Err(ExportError::Conflict)
    ));
    std::fs::write(&source, b"{}").unwrap();
    assert!(matches!(
        apply(&session, &db, &crypto, false, never_open),
        Err(ExportError::Conflict)
    ));
    assert!(!db.exists());
    std::fs::write(&source, &bytes).unwrap();
    std::fs::create_dir_all(db.parent().unwrap()).unwrap();
    std::fs::File::create_new(&db).unwrap();
    assert!(matches!(
        apply(&session, &db, &crypto, false, never_open),
        Err(ExportError::Conflict)
    ));
    std::fs::remove_file(&db).unwrap();
    let open = || {
        let pool = database::create_pool(&db, None).unwrap();
        pool.get()
            .unwrap()
            .execute_batch(database::schema::CREATE_TABLES_SQL)
            .unwrap();
        Ok(DbAccess::Pool(pool))
    };
    let result = apply(&session, &db, &crypto, false, open).unwrap();
    assert_eq!(result.counts.providers_imported, 1);
    assert!(crypto.join("backups").join(&result.backup_file).is_file());
    let reviewed = ExportManager::preview_file_at(&source, &db, &crypto).unwrap();
    assert!(!reviewed.target_missing());
    assert_eq!(reviewed.preview.conflicting_providers, 1);
    assert_eq!(
        apply(&reviewed, &db, &crypto, false, open)
            .unwrap()
            .counts
            .providers_skipped,
        1
    );
    assert_eq!(
        apply(&reviewed, &db, &crypto, true, open)
            .unwrap()
            .counts
            .providers_imported,
        1
    );
}

#[test]
fn first_import_open_failure_can_be_previewed_and_retried() {
    let root = tempfile::tempdir().unwrap();
    let db = root.path().join("db/checkin.db");
    let crypto = root.path().join("crypto");
    let source = root.path().join("config.json");
    std::fs::write(
        &source,
        serde_json::to_vec(&ExportData::new(vec![], vec![])).unwrap(),
    )
    .unwrap();
    let preview = ExportManager::preview_file_at(&source, &db, &crypto).unwrap();
    assert!(
        apply(&preview, &db, &crypto, false, || Err(
            ExportError::ImportFailed("fixture open failure".into())
        ))
        .is_err()
    );
    let retry = ExportManager::preview_file_at(&source, &db, &crypto).unwrap();
    assert!(
        apply(&retry, &db, &crypto, false, || Ok(DbAccess::Pool(
            database::create_pool(&db, None).unwrap()
        )))
        .is_ok()
    );
}

#[test]
fn failed_import_reports_exact_committed_counts_and_verified_recovery_file() {
    let root = tempfile::tempdir().unwrap();
    let db = root.path().join("checkin.db");
    let crypto = root.path().join("crypto");
    let pool = database::create_pool(&db, None).unwrap();
    let conn = pool.get().unwrap();
    conn.execute_batch(database::schema::CREATE_TABLES_SQL)
        .unwrap();
    conn.execute_batch("CREATE TRIGGER reject_second BEFORE INSERT ON checkin_providers WHEN NEW.name = 'Second' BEGIN SELECT RAISE(FAIL, 'fixture failure'); END;").unwrap();
    let providers = ["First", "Second"]
        .map(|name| CheckinProvider::new(name.into(), "https://example.invalid".into()));
    let source = root.path().join("source.json");
    std::fs::write(
        &source,
        serde_json::to_vec(&ExportData::new(providers.into(), vec![])).unwrap(),
    )
    .unwrap();
    let session = ExportManager::preview_file_at(&source, &db, &crypto).unwrap();
    let Err(ExportError::PartialImport(receipt)) = apply(&session, &db, &crypto, false, || {
        Ok(DbAccess::Pool(pool.clone()))
    }) else {
        panic!("partial receipt required");
    };
    assert_eq!(receipt.counts.providers_imported, 1);
    let backup = rusqlite::Connection::open_with_flags(
        crypto.join("backups").join(receipt.backup_file),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let rows = ccr_db::database::repositories::checkin_repo::get_all_providers;
    assert_eq!(rows(&conn).unwrap().len(), 1);
    assert!(rows(&backup).unwrap().is_empty());
}

#[test]
fn name_conflicts_keep_local_provider_identity_for_skip_and_overwrite() {
    let root = tempfile::tempdir().unwrap();
    let db = root.path().join("checkin.db");
    let crypto = root.path().join("crypto");
    let pool = database::create_pool(&db, None).unwrap();
    let conn = pool.get().unwrap();
    conn.execute_batch(database::schema::CREATE_TABLES_SQL)
        .unwrap();
    let local = CheckinProvider::new("Shared".into(), "https://old.invalid".into());
    let foreign = CheckinProvider::new("Shared".into(), "https://new.invalid".into());
    let repo = ProviderManager::with_db(DbAccess::Pool(pool.clone()));
    ccr_db::database::repositories::checkin_repo::insert_provider(&conn, &local).unwrap();
    let account = ExportAccount {
        id: "a".into(),
        provider_id: foreign.id.clone(),
        name: "Imported".into(),
        cookies_json: Secret::from(r#"{"cookie":"synthetic"}"#),
        cookies_json_encrypted: false,
        api_user: "1".into(),
        enabled: true,
        created_at: chrono::Utc::now(),
    };
    let source = root.path().join("source.json");
    std::fs::write(
        &source,
        serde_json::to_vec(&ExportData::new(vec![foreign], vec![account])).unwrap(),
    )
    .unwrap();
    for overwrite in [false, true] {
        let session = ExportManager::preview_file_at(&source, &db, &crypto).unwrap();
        let result = apply(&session, &db, &crypto, overwrite, || {
            Ok(DbAccess::Pool(pool.clone()))
        })
        .unwrap()
        .counts;
        assert_eq!(result.providers_imported, usize::from(overwrite));
        assert_eq!(result.providers_skipped, usize::from(!overwrite));
        assert_eq!(result.accounts_imported, 1);
        assert_eq!(
            repo.get(&local.id).unwrap().base_url,
            if overwrite {
                "https://new.invalid"
            } else {
                "https://old.invalid"
            }
        );
        assert_eq!(
            AccountManager::with_db(&crypto, DbAccess::Pool(pool.clone()))
                .get("a")
                .unwrap()
                .provider_id,
            local.id
        );
    }
}

#[test]
fn transfer_uses_scoped_database_preserves_extensions_and_refuses_stale_preview() {
    let root = tempfile::tempdir().unwrap();
    let crypto = root.path().join("crypto");
    let pool = database::create_pool(&root.path().join("source.db"), None).unwrap();
    {
        let conn = pool.get().unwrap();
        conn.execute_batch(database::schema::CREATE_TABLES_SQL)
            .unwrap();
    }
    let db = DbAccess::Pool(pool.clone());
    let provider = ProviderManager::with_db(db.clone())
        .create(CreateProviderRequest {
            name: "Fixture".into(),
            base_url: "https://fixture.invalid".into(),
            checkin_path: None,
            balance_path: None,
            user_info_path: None,
            auth_header: None,
            auth_prefix: None,
            builtin_id: None,
        })
        .unwrap();
    let accounts = AccountManager::with_db(&crypto, db.clone());
    let account = accounts
        .create(CreateAccountRequest {
            provider_id: provider.id,
            name: "Fixture account".into(),
            cookies_json: Secret::new(r#"{"session":"fixture-sensitive"}"#),
            api_user: "7".into(),
            extra_config: r#"{"future":"keep"}"#.into(),
        })
        .unwrap();
    let manager = ExportManager::with_db(&crypto, db);
    let export = root.path().join("config.json");
    manager.export_file(&export).unwrap();
    assert!(
        !std::fs::read_to_string(&export)
            .unwrap()
            .contains("fixture-sensitive")
    );
    let original = std::fs::read(&export).unwrap();
    assert!(matches!(
        manager.export_file(&export),
        Err(ExportError::DestinationExists)
    ));
    assert!(matches!(
        manager.export_stats_file(&export),
        Err(ExportError::DestinationExists)
    ));
    assert_eq!(std::fs::read(&export).unwrap(), original);
    let stats = root.path().join("today.json");
    manager.export_stats_file(&stats).unwrap();
    let stats: serde_json::Value = serde_json::from_slice(&std::fs::read(stats).unwrap()).unwrap();
    assert_eq!(stats["total_accounts"], 1);
    assert!(stats.get("accounts").is_none());
    let database_file = root.path().join("source.db");
    let captured = ExportManager::preview_file_at(&export, &database_file, &crypto).unwrap();
    let original_key = std::fs::read(crypto.join("crypto.key")).unwrap();
    let lost_during_open = apply(&captured, &database_file, &crypto, true, || {
        std::fs::remove_file(crypto.join("crypto.key")).unwrap();
        Ok(DbAccess::Pool(pool.clone()))
    });
    assert!(matches!(
        lost_during_open,
        Err(ExportError::ExistingKeyUnavailable | ExportError::CryptoError(_))
    ));
    assert!(!crypto.join("crypto.key").exists());
    assert!(!crypto.join("backups").exists());
    std::fs::write(crypto.join("crypto.key"), original_key).unwrap();
    let preview = Arc::new(manager.preview_file(&export).unwrap());
    accounts
        .update(
            &account.id,
            UpdateAccountRequest {
                name: Some("external winner".into()),
                cookies_json: None,
                api_user: None,
                enabled: None,
                extra_config: None,
            },
        )
        .unwrap();
    assert!(manager.import_preview(&preview, true).is_err());
    let preview = Arc::new(manager.preview_file(&export).unwrap());
    let result = manager.import_preview(&preview, true).unwrap();
    assert_eq!(result.accounts_imported, 1);
    assert_eq!(
        accounts.get(&account.id).unwrap().extra_config,
        r#"{"future":"keep"}"#
    );
    let backups: Vec<_> = std::fs::read_dir(crypto.join("backups")).unwrap().collect();
    assert_eq!(backups.len(), 1);
    let mut raw: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&export).unwrap()).unwrap();
    raw["unknown"] = serde_json::Value::Null;
    std::fs::write(&export, serde_json::to_vec(&raw).unwrap()).unwrap();
    assert!(manager.preview_file(&export).is_err());
    // A source readable at preview must still be decryptable before commit.
    let fresh = root.path().join("fresh.json");
    manager.export_file(&fresh).unwrap();
    let preview = Arc::new(manager.preview_file(&fresh).unwrap());
    let original_account = accounts.get(&account.id).unwrap();
    let foreign = root.path().join("foreign-crypto");
    CryptoManager::new(&foreign).unwrap();
    std::fs::copy(foreign.join("crypto.key"), crypto.join("crypto.key")).unwrap();
    assert!(manager.import_preview(&preview, true).is_err());
    assert!(manager.preview_file(&fresh).is_err());
    assert_eq!(
        accounts.get(&account.id).unwrap().cookies_json_encrypted,
        original_account.cookies_json_encrypted
    );
    assert_eq!(
        std::fs::read_dir(crypto.join("backups")).unwrap().count(),
        1
    );
}
