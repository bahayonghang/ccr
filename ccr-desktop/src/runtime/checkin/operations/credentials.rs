//! Editor-scoped plaintext; never enters the retained management snapshot.
use super::*;
use ccr_app::environments::EnvironmentPolicy;
use ccr_app::runtime::{AdmissionError, TaskSubscription};
use ccr_core::Secret;

/// Deliberately no Debug/Serialize: only the selected editor consumes this.
#[derive(Clone)]
pub struct AccountCredentials {
    pub account_id: String,
    pub name: String,
    pub cookies: Secret,
    pub api_user: Secret,
}

impl Services {
    pub fn checkin_account_credentials(
        &self,
        key: RequestKey,
        account_id: String,
    ) -> Result<TaskSubscription<AccountCredentials, OperationError>, AdmissionError> {
        let local = EnvironmentPolicy::Independent
            .admit(&key.environment)
            .is_ok();
        let source = self.source.clone();
        let owner = self.checkin_operations.clone();
        self.executor.read(key, move |cancel, _| {
            if !local {
                return Err(OperationError::Unsupported);
            }
            if cancel.is_cancelled() {
                return Err(OperationError::Interrupted);
            }
            if owner
                .state
                .lock()
                .map_err(|_| OperationError::Storage)?
                .busy
            {
                return Err(OperationError::Busy);
            }
            let path = database_path(&source)
                .map_err(|_| OperationError::Storage)?
                .ok_or(OperationError::Invalid)?;
            let conn = open_reader(&path).map_err(|_| OperationError::Storage)?;
            let account = checkin_repo::get_account_by_id(&conn, &account_id)
                .map_err(|_| OperationError::Storage)?
                .ok_or(OperationError::Invalid)?;
            let crypto_dir = match &source {
                ConfigSource::Local => {
                    CheckinService::default_checkin_dir().map_err(|_| OperationError::Storage)?
                }
                ConfigSource::Benchmark(layout) => layout.path("crypto/checkin"),
                ConfigSource::Fixture { .. } => {
                    let dir = path
                        .parent()
                        .ok_or(OperationError::Storage)?
                        .join("checkin-crypto");
                    reject_links(&dir)?;
                    dir
                }
            };
            let crypto = ccr_checkin::core::crypto::CryptoManager::load_existing(&crypto_dir)
                .map_err(|_| OperationError::CryptoKeyUnavailable)?;
            let cookies = crypto
                .decrypt(&account.cookies_json_encrypted)
                .map_err(|_| OperationError::Credentials)?;
            let api_user = account.api_user;
            if cancel.is_cancelled() {
                return Err(OperationError::Interrupted);
            }
            Ok(AccountCredentials {
                account_id,
                name: account.name,
                cookies,
                api_user: api_user.into(),
            })
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_app::runtime::{Platform, RuntimeOwner, TaskEvent};

    #[test]
    fn editing_a_vanished_account_does_not_initialize_storage() {
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
        let mut read = services
            .checkin_account_credentials(
                RequestKey {
                    environment: EnvironmentId::Local("application".into()),
                    platform: Platform::Claude,
                    generation: 1,
                    request_id: 1,
                },
                "vanished".into(),
            )
            .unwrap();
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            loop {
                if let TaskEvent::Terminal(result) = read.next().await.unwrap() {
                    assert!(matches!(
                        result.as_ref(),
                        Completion::Failed(OperationError::Invalid)
                    ));
                    break;
                }
            }
        });
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
        runtime.shutdown();
    }

    #[test]
    fn empty_fixture_transfer_keeps_crypto_creation_and_locks_captured() {
        let runtime = RuntimeOwner::new().unwrap();
        let source_root = tempfile::tempdir().unwrap();
        let target_root = tempfile::tempdir().unwrap();
        let services = |root: &Path| {
            Services::new(
                runtime.handle(),
                ConfigSource::Fixture {
                    profiles: root.join("profiles.toml"),
                    registry: root.join("registry.toml"),
                    delay: Duration::ZERO,
                },
            )
        };
        let source = services(source_root.path());
        let target = services(target_root.path());
        let run = |services: &Services, operation, generation| {
            services
                .start_checkin(
                    RequestKey {
                        environment: EnvironmentId::Local("fixture".into()),
                        platform: Platform::Claude,
                        generation,
                        request_id: generation,
                    },
                    operation,
                )
                .unwrap();
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            loop {
                let state = services.checkin_operation_state();
                if !state.busy {
                    assert_eq!(state.result, Some(Ok(Outcome::Extended)));
                    return state;
                }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(5));
            }
        };
        // An existing database can be exported without running startup migrations.
        let conn = rusqlite::Connection::open(source_root.path().join("checkin.db")).unwrap();
        conn.execute_batch(database::schema::CREATE_TABLES_SQL)
            .unwrap();
        drop(conn);
        let db_path = source_root.path().join("checkin.db");
        let initial = std::fs::read(&db_path).unwrap();
        run(
            &source,
            Operation::Extended(ExtendedCommand::ExportStats(
                source_root.path().join("today.json"),
            )),
            0,
        );
        assert!(!source_root.path().join("checkin-crypto").exists());
        let export = source_root.path().join("empty-export.json");
        run(
            &source,
            Operation::Extended(ExtendedCommand::Export(export.clone())),
            1,
        );
        assert!(!source_root.path().join("checkin-migration-home").exists());
        assert_eq!(std::fs::read(db_path).unwrap(), initial);
        let copied = target_root.path().join("empty-import.json");
        std::fs::copy(export, &copied).unwrap();
        let key = RequestKey {
            environment: EnvironmentId::Local("fixture".into()),
            platform: Platform::Claude,
            generation: 2,
            request_id: 2,
        };
        let mut read = target.checkin_import_preview(key, copied).unwrap();
        let session = tokio::runtime::Runtime::new().unwrap().block_on(async {
            loop {
                if let TaskEvent::Terminal(result) = read.next().await.unwrap() {
                    if let Completion::Success(preview) = result.as_ref() {
                        break preview.clone();
                    }
                    panic!("preview failed");
                }
            }
        });
        assert!(!target_root.path().join("checkin.db").exists());
        assert!(
            !target_root
                .path()
                .join("checkin-crypto/crypto.key")
                .exists()
        );
        run(
            &target,
            Operation::Extended(ExtendedCommand::Import {
                session: session.clone(),
                overwrite: false,
            }),
            3,
        );
        for root in [source_root.path(), target_root.path()] {
            assert!(root.join("checkin-crypto/crypto.key").is_file());
            assert!(root.join("checkin-locks").is_dir());
        }
        runtime.shutdown();
    }

    #[tokio::test]
    async fn credential_read_is_selected_scoped_and_never_retained() {
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
        let provider = storage
            .providers()
            .create(CreateProviderRequest {
                name: "fixture".into(),
                base_url: "https://example.invalid".into(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
                builtin_id: None,
            })
            .unwrap();
        let account = storage
            .scoped(|| {
                storage
                    .accounts()
                    .create(CreateAccountRequest {
                        provider_id: provider.id,
                        name: "selected".into(),
                        cookies_json: r#"{"session":"editor-only-secret"}"#.into(),
                        api_user: "editor-only-user".into(),
                        extra_config: "{}".into(),
                    })
                    .map_err(|_| OperationError::Storage)
            })
            .unwrap();
        let encrypted = account.cookies_json_encrypted.clone();
        for (environment, id, expected) in [
            (
                EnvironmentId::Local("fixture".into()),
                account.id.clone(),
                None,
            ),
            (
                EnvironmentId::Ssh("fixture".into()),
                account.id.clone(),
                None,
            ),
            (
                EnvironmentId::Local("fixture".into()),
                "missing".into(),
                Some(OperationError::Invalid),
            ),
        ] {
            let mut read = services
                .checkin_account_credentials(
                    RequestKey {
                        environment,
                        platform: Platform::Claude,
                        generation: 1,
                        request_id: 1,
                    },
                    id,
                )
                .unwrap();
            loop {
                let event = read.next().await.unwrap();
                if let TaskEvent::Terminal(result) = event {
                    match (result.as_ref(), expected) {
                        (Completion::Success(value), None) => {
                            assert_eq!(value.account_id, account.id);
                            assert_eq!(value.name, "selected");
                            assert_eq!(
                                value.cookies.expose(),
                                r#"{"session":"editor-only-secret"}"#
                            );
                            assert_eq!(value.api_user.expose(), "editor-only-user");
                        }
                        (Completion::Failed(actual), Some(expected)) => {
                            assert_eq!(*actual, expected)
                        }
                        _ => panic!("unexpected credential outcome"),
                    }
                    break;
                }
            }
        }
        assert_eq!(
            storage
                .accounts()
                .get(&account.id)
                .unwrap()
                .cookies_json_encrypted,
            encrypted
        );
        let retained = services.checkin_operation_state();
        assert!(
            retained.snapshot.is_none() && retained.result.is_none() && retained.extended.is_none()
        );
        std::fs::remove_file(storage.crypto.join("crypto.key")).unwrap();
        let mut read = services
            .checkin_account_credentials(
                RequestKey {
                    environment: EnvironmentId::Local("fixture".into()),
                    platform: Platform::Claude,
                    generation: 2,
                    request_id: 2,
                },
                account.id.clone(),
            )
            .unwrap();
        loop {
            if let TaskEvent::Terminal(result) = read.next().await.unwrap() {
                assert!(matches!(
                    result.as_ref(),
                    Completion::Failed(OperationError::CryptoKeyUnavailable)
                ));
                break;
            }
        }
        assert!(!storage.crypto.join("crypto.key").exists());
        let metadata_only = execute(
            &storage,
            &services.source,
            &services.checkin_operations,
            Operation::UpdateAccount(
                account.id.clone(),
                ccr_db::models::checkin::UpdateAccountRequest {
                    name: Some("metadata-preserved".into()),
                    cookies_json: None,
                    api_user: None,
                    enabled: None,
                    extra_config: None,
                },
            ),
        )
        .await;
        assert!(
            metadata_only.is_ok(),
            "metadata-only editor save must preserve encrypted credentials without a key"
        );
        assert!(!storage.crypto.join("crypto.key").exists());
        let unchanged = storage.accounts().get(&account.id).unwrap();
        assert_eq!(unchanged.name, "metadata-preserved");
        assert_eq!(unchanged.cookies_json_encrypted, encrypted);
        let replacement = execute(
            &storage,
            &services.source,
            &services.checkin_operations,
            Operation::UpdateAccount(
                account.id.clone(),
                ccr_db::models::checkin::UpdateAccountRequest {
                    name: Some("must-not-save".into()),
                    cookies_json: Some(r#"{"session":"replacement"}"#.into()),
                    api_user: None,
                    enabled: None,
                    extra_config: None,
                },
            ),
        )
        .await;
        assert!(
            replacement.is_err(),
            "credential replacement requires the original key"
        );
        assert!(!storage.crypto.join("crypto.key").exists());
        assert_eq!(
            storage
                .accounts()
                .get(&account.id)
                .unwrap()
                .cookies_json_encrypted,
            encrypted
        );
        tokio::task::spawn_blocking(move || runtime.shutdown())
            .await
            .unwrap();
    }
}
