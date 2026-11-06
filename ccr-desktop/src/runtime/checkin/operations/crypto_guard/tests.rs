#![allow(clippy::unwrap_used)]
use super::*;
use ccr_app::runtime::RuntimeOwner;
use ccr_core::Secret;

#[test]
fn captured_empty_storage_allows_first_key_but_existing_rows_require_original_key() {
    let root = tempfile::tempdir().unwrap();
    let runtime = RuntimeOwner::new().unwrap();
    let services = Services::new(
        runtime.handle(),
        ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        },
    );
    let storage = services
        .checkin_operations
        .storage(&services.source)
        .unwrap();
    assert_eq!(storage.guard_crypto_initialization(), Ok(()));
    assert!(!storage.crypto.join("crypto.key").exists());
    let provider = storage
        .providers()
        .create(CreateProviderRequest {
            name: "controlled".into(),
            base_url: "http://127.0.0.1:1".into(),
            checkin_path: Some("/checkin".into()),
            balance_path: Some("/balance".into()),
            user_info_path: Some("/user".into()),
            auth_header: None,
            auth_prefix: None,
            builtin_id: None,
        })
        .unwrap();
    let account = storage
        .scoped(|| {
            storage.guard_crypto_initialization()?;
            storage
                .accounts()
                .create(CreateAccountRequest {
                    provider_id: provider.id,
                    name: "existing".into(),
                    cookies_json: Secret::from(r#"{"session":"controlled-original"}"#),
                    api_user: "7".into(),
                    extra_config: "{}".into(),
                })
                .map_err(|_| OperationError::Storage)
        })
        .unwrap();
    let key = storage.crypto.join("crypto.key");
    assert!(key.is_file());
    let encrypted = account.cookies_json_encrypted;
    let original = std::fs::read(&key).unwrap();
    std::fs::remove_file(&key).unwrap();
    assert_eq!(
        storage.guard_crypto_initialization(),
        Err(OperationError::CryptoKeyUnavailable)
    );
    assert!(storage.accounts().get_cookies_json(&account.id).is_err());
    assert!(storage.accounts().get_info(&account.id).is_err());
    assert!(!key.exists());
    std::fs::write(&key, b"malformed-owned-fixture-key").unwrap();
    assert_eq!(
        storage.guard_crypto_initialization(),
        Err(OperationError::CryptoKeyUnavailable)
    );
    assert_eq!(std::fs::read(&key).unwrap(), b"malformed-owned-fixture-key");
    assert_eq!(
        storage
            .accounts()
            .get(&account.id)
            .unwrap()
            .cookies_json_encrypted,
        encrypted
    );
    std::fs::write(&key, original).unwrap();
    assert_eq!(storage.require_existing_crypto(), Ok(()));
    runtime.shutdown();
}

#[test]
fn missing_or_malformed_existing_keys_block_native_mutation_transfer_and_requests() {
    let root = tempfile::tempdir().unwrap();
    let runtime = RuntimeOwner::new().unwrap();
    let source = ConfigSource::Fixture {
        profiles: root.path().join("profiles.toml"),
        registry: root.path().join("registry.toml"),
        delay: Duration::ZERO,
    };
    let services = Services::new(runtime.handle(), source.clone());
    let storage = services.checkin_operations.storage(&source).unwrap();
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let provider = storage
        .providers()
        .create(CreateProviderRequest {
            name: "controlled".into(),
            base_url: format!("http://{}", listener.local_addr().unwrap()),
            checkin_path: Some("/checkin".into()),
            balance_path: Some("/balance".into()),
            user_info_path: Some("/user".into()),
            auth_header: None,
            auth_prefix: None,
            builtin_id: None,
        })
        .unwrap();
    let account_request = || CreateAccountRequest {
        provider_id: provider.id.clone(),
        name: "controlled".into(),
        cookies_json: Secret::from(r#"{"session":"controlled-original"}"#),
        api_user: "7".into(),
        extra_config: "{}".into(),
    };
    let account = storage
        .scoped(|| {
            storage
                .accounts()
                .create(account_request())
                .map_err(|_| OperationError::Storage)
        })
        .unwrap();
    let encrypted = account.cookies_json_encrypted.clone();
    let key = storage.crypto.join("crypto.key");
    let original_key = std::fs::read(&key).unwrap();
    let manager = ccr_checkin::ExportManager::with_db(&storage.crypto, storage.db.clone());
    let source_file = root.path().join("reviewed.json");
    manager.export_file(&source_file).unwrap();
    let preview = Arc::new(manager.preview_file(&source_file).unwrap());
    let owner = CheckinOperations {
        loopback_fixture: true,
        ..Default::default()
    };
    let wait = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    for malformed in [false, true] {
        if malformed {
            std::fs::write(&key, b"damaged-controlled-key").unwrap();
        } else {
            std::fs::remove_file(&key).unwrap();
        }
        let export = root.path().join(if malformed {
            "malformed-export.json"
        } else {
            "missing-export.json"
        });
        let operations = vec![
            Operation::AddAccount(account_request()),
            Operation::UpdateAccount(
                account.id.clone(),
                UpdateAccountRequest {
                    name: Some("must-not-update".into()),
                    cookies_json: Some(Secret::from(r#"{"session":"replacement"}"#)),
                    api_user: None,
                    enabled: None,
                    extra_config: None,
                },
            ),
            Operation::Extended(ExtendedCommand::Export(export.clone())),
            Operation::Extended(ExtendedCommand::Import {
                session: preview.clone(),
                overwrite: true,
            }),
            Operation::Checkin(account.id.clone()),
            Operation::Balance(account.id.clone()),
            Operation::Extended(ExtendedCommand::Inspect(InspectionCommand::Connection(
                account.id.clone(),
            ))),
            Operation::Extended(ExtendedCommand::Inspect(InspectionCommand::Waf(
                account.id.clone(),
            ))),
        ];
        for operation in operations {
            assert_eq!(
                wait.block_on(execute(&storage, &source, &owner, operation))
                    .err(),
                Some(OperationError::CryptoKeyUnavailable)
            );
            assert_eq!(
                storage
                    .accounts()
                    .get(&account.id)
                    .unwrap()
                    .cookies_json_encrypted,
                encrypted
            );
            assert_eq!(storage.accounts().load_all().unwrap().len(), 1);
            if malformed {
                assert_eq!(std::fs::read(&key).unwrap(), b"damaged-controlled-key");
            } else {
                assert!(!key.exists());
            }
            assert!(!export.exists());
            assert!(!storage.crypto.join("backups").exists());
        }
        // Controlled adapter driver: captured temporary storage, no provider call.
        assert_eq!(
            wait.block_on(execute(
                &storage,
                &ConfigSource::Local,
                &owner,
                Operation::Extended(ExtendedCommand::Recharge {
                    account: account.id.clone(),
                    code: Secret::from("controlled-code")
                })
            ))
            .err(),
            Some(OperationError::CryptoKeyUnavailable)
        );
        assert!(
            matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
        // Metadata and pure CDK-extra edits must not require decryption.
        let metadata = UpdateAccountRequest {
            name: Some("metadata-only".into()),
            cookies_json: None,
            api_user: None,
            enabled: None,
            extra_config: None,
        };
        assert_eq!(
            wait.block_on(execute(
                &storage,
                &source,
                &owner,
                Operation::UpdateAccount(account.id.clone(), metadata)
            ))
            .map(|value| value.0)
            .ok(),
            Some(Outcome::Saved)
        );
        assert!(
            wait.block_on(execute(
                &storage,
                &source,
                &owner,
                Operation::Extended(ExtendedCommand::CdkCredentials {
                    account: account.id.clone(),
                    patch: Secret::from(r#"{"x666_access_token":"controlled-extra"}"#)
                })
            ))
            .is_ok()
        );
        assert!(
            wait.block_on(execute(
                &storage,
                &source,
                &owner,
                Operation::Extended(ExtendedCommand::Stats)
            ))
            .is_ok()
        );
        assert_eq!(
            storage
                .accounts()
                .get(&account.id)
                .unwrap()
                .cookies_json_encrypted,
            encrypted
        );
        if malformed {
            assert_eq!(std::fs::read(&key).unwrap(), b"damaged-controlled-key");
        } else {
            assert!(!key.exists());
        }
        std::fs::write(&key, &original_key).unwrap();
    }
    // The native key guard must not bypass the original reviewed DB revision.
    assert_eq!(
        wait.block_on(execute(
            &storage,
            &source,
            &owner,
            Operation::Extended(ExtendedCommand::Import {
                session: preview,
                overwrite: true
            })
        ))
        .err(),
        Some(OperationError::ImportConflict)
    );
    assert!(!storage.crypto.join("backups").exists());
    let refreshed = Arc::new(manager.preview_file(&source_file).unwrap());
    assert!(
        wait.block_on(execute(
            &storage,
            &source,
            &owner,
            Operation::Extended(ExtendedCommand::Import {
                session: refreshed,
                overwrite: false
            })
        ))
        .is_ok()
    );
    assert!(storage.crypto.join("backups").is_dir());
    drop(wait);
    runtime.shutdown();
}
