#![allow(clippy::unwrap_used)]
use super::*;
use ccr_app::runtime::{Platform, RuntimeOwner};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn source(root: &Path) -> ConfigSource {
    ConfigSource::Fixture {
        profiles: root.join("profiles.toml"),
        registry: root.join("registry.toml"),
        delay: Duration::ZERO,
    }
}

fn revision(source: &ConfigSource) -> String {
    document(source).unwrap().read().unwrap().token
}

fn config(root: &Path, url: &str) {
    let mut config = SyncFoldersConfig::default();
    config.webdav.url = url.into();
    config.webdav.username = "fixture-user".into();
    config.webdav.password = "fixture-password".into();
    config.folders.push(
        ccr_sync::SyncFolder::builder()
            .name("fixture")
            .local_path(root.join("data.txt").to_str().unwrap())
            .remote_path("/data.txt")
            .build()
            .unwrap(),
    );
    std::fs::write(
        root.join("sync_folders.toml"),
        toml::to_string(&config).unwrap(),
    )
    .unwrap();
}

#[test]
fn settings_preserve_unknown_fields_password_and_compare_and_swap() {
    let root = tempfile::tempdir().unwrap();
    let source = source(root.path());
    config(root.path(), "https://dav.example.test/");
    let path = root.path().join("sync_folders.toml");
    let mut value: toml::Value = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    value["webdav"]
        .as_table_mut()
        .unwrap()
        .insert("extension".into(), "keep".into());
    value
        .as_table_mut()
        .unwrap()
        .insert("future".into(), 42.into());
    std::fs::write(&path, toml::to_string(&value).unwrap()).unwrap();
    let old = revision(&source);
    let patch = |revision, password| SettingsPatch {
        revision,
        enabled: false,
        url: None,
        username: Some("updated-user".into()),
        base_remote_path: None,
        password,
    };
    assert_eq!(
        save(&source, patch(old.clone(), PasswordUpdate::Keep)),
        Ok(SyncOutcome::Saved)
    );
    let stored: toml::Value = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(
        stored["webdav"]["password"].as_str(),
        Some("fixture-password")
    );
    assert_eq!(stored["webdav"]["extension"].as_str(), Some("keep"));
    assert_eq!(stored["future"].as_integer(), Some(42));
    assert_eq!(stored["folders"], value["folders"]);
    let before = std::fs::read(&path).unwrap();
    assert_eq!(
        save(&source, patch(old, PasswordUpdate::Clear)),
        Err(SyncError::Conflict)
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(
        save(
            &source,
            patch(
                revision(&source),
                PasswordUpdate::Replace("new-secret".into())
            )
        ),
        Ok(SyncOutcome::Saved)
    );
    assert_eq!(
        SyncFolderManager::new(&path)
            .load_config()
            .unwrap()
            .webdav
            .password
            .expose(),
        "new-secret"
    );
    assert_eq!(
        save(&source, patch(revision(&source), PasswordUpdate::Clear)),
        Ok(SyncOutcome::Saved)
    );
    assert!(
        SyncFolderManager::new(&path)
            .load_config()
            .unwrap()
            .webdav
            .password
            .is_empty()
    );
    assert!(
        root.path()
            .join("backups")
            .read_dir()
            .unwrap()
            .next()
            .is_some()
    );
}

#[test]
fn operations_are_local_owned_and_fixture_network_is_refused_after_detach() {
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let services = Services::new(owner.handle(), source(root.path()));
    let key = RequestKey {
        environment: EnvironmentId::Local("fixture".into()),
        platform: Platform::Claude,
        generation: 1,
        request_id: 1,
    };
    let subscription = services.observe_sync();
    services
        .start_sync(
            key.clone(),
            Operation::Test {
                revision: String::new(),
            },
        )
        .unwrap();
    drop(subscription);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while services.sync_operation_state().busy && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    let retained = services.sync_operation_state();
    assert!(!retained.busy);
    assert!(matches!(
        retained.result,
        Some(Err(SyncError::FixtureNetwork))
    ));
    assert_eq!(retained.environment, Some(key.environment.clone()));
    let remote = RequestKey {
        environment: EnvironmentId::Ssh("remote".into()),
        ..key.clone()
    };
    assert_eq!(
        services.start_sync(
            remote,
            Operation::Test {
                revision: String::new()
            }
        ),
        Err(SyncError::Unsupported)
    );
    owner.shutdown();
    assert_eq!(
        services.start_sync(
            key,
            Operation::Test {
                revision: String::new()
            }
        ),
        Err(SyncError::Interrupted)
    );
    assert!(!services.sync_operation_state().busy);
}

async fn server(
    status: &'static str,
    body: &'static [u8],
    extra_length: usize,
) -> (String, tokio::task::JoinHandle<Vec<u8>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        let mut chunk = [0; 4096];
        let (end, length) = loop {
            let count = socket.read(&mut chunk).await.unwrap();
            assert_ne!(count, 0);
            request.extend_from_slice(&chunk[..count]);
            if let Some(end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                let length = String::from_utf8_lossy(&request[..end])
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                break (end + 4, length);
            }
        };
        while request.len() < end + length {
            let count = socket.read(&mut chunk).await.unwrap();
            assert_ne!(count, 0);
            request.extend_from_slice(&chunk[..count]);
        }
        let headers = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len() + extra_length
        );
        socket.write_all(headers.as_bytes()).await.unwrap();
        socket.write_all(body).await.unwrap();
        socket.shutdown().await.unwrap();
        request
    });
    (format!("http://{address}/"), task)
}

#[tokio::test]
async fn explicit_loopback_push_and_transactional_pull_use_the_canonical_folder() {
    let root = tempfile::tempdir().unwrap();
    let source = source(root.path());
    let local = root.path().join("data.txt");
    std::fs::write(&local, "original bytes").unwrap();
    let (url, request) = server("201 Created", b"", 0).await;
    config(root.path(), &url);
    assert_eq!(
        execute(
            &source,
            Operation::Push {
                revision: revision(&source),
                folder: 0
            },
            true
        )
        .await,
        Ok(SyncOutcome::Pushed)
    );
    let request = request.await.unwrap();
    assert!(request.starts_with(b"PUT /data.txt "));
    assert!(request.ends_with(b"original bytes"));

    let (url, request) = server("200 OK", b"downloaded bytes", 0).await;
    config(root.path(), &url);
    let result = execute(
        &source,
        Operation::Pull {
            revision: revision(&source),
            folder: 0,
        },
        true,
    )
    .await
    .unwrap();
    assert!(request.await.unwrap().starts_with(b"GET /data.txt "));
    let SyncOutcome::Pulled {
        backup: Some(backup),
    } = result
    else {
        panic!("backup missing")
    };
    assert_eq!(std::fs::read_to_string(backup).unwrap(), "original bytes");
    assert_eq!(std::fs::read(&local).unwrap(), b"downloaded bytes");

    let (url, request) = server("200 OK", b"partial", 100).await;
    config(root.path(), &url);
    assert_eq!(
        execute(
            &source,
            Operation::Pull {
                revision: revision(&source),
                folder: 0
            },
            true
        )
        .await,
        Err(SyncError::Operation)
    );
    request.await.unwrap();
    assert_eq!(std::fs::read(&local).unwrap(), b"downloaded bytes");
}

#[test]
fn folder_crud_preserves_extensions_password_and_rejects_stale_identity() {
    let root = tempfile::tempdir().unwrap();
    let source = source(root.path());
    config(root.path(), "https://dav.example.test/");
    let path = root.path().join("sync_folders.toml");
    let mut initial: toml::Value =
        toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    initial["webdav"]
        .as_table_mut()
        .unwrap()
        .insert("future".into(), "preserve".into());
    initial["folders"][0]
        .as_table_mut()
        .unwrap()
        .insert("future".into(), "folder-extension".into());
    initial["folders"][0].as_table_mut().unwrap().insert(
        "exclude_patterns".into(),
        vec![toml::Value::String("private/**".into())].into(),
    );
    std::fs::write(&path, toml::to_string(&initial).unwrap()).unwrap();
    let stale = revision(&source);
    let new_folder = FolderPatch {
        name: Some("extra".into()),
        local_path: Some(root.path().join("extra.txt").to_string_lossy().into_owned()),
        remote_path: Some("/extra.txt".into()),
    };
    assert_eq!(
        save_folder(&source, &stale, FolderChange::Add(new_folder)),
        Ok(SyncOutcome::FolderSaved)
    );
    let identity = ccr_core::core::content_version_token(b"fixture");
    assert_eq!(
        save_folder(
            &source,
            &revision(&source),
            FolderChange::Edit {
                identity: identity.clone(),
                patch: FolderPatch {
                    name: Some("renamed".into()),
                    local_path: None,
                    remote_path: None
                }
            }
        ),
        Ok(SyncOutcome::FolderSaved)
    );
    let renamed = ccr_core::core::content_version_token(b"renamed");
    assert_eq!(
        save_folder(
            &source,
            &revision(&source),
            FolderChange::Enabled {
                identity: renamed.clone(),
                enabled: false
            }
        ),
        Ok(SyncOutcome::FolderSaved)
    );
    let current: toml::Value = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(current["webdav"], initial["webdav"]);
    assert_eq!(
        current["folders"][0]["exclude_patterns"],
        initial["folders"][0]["exclude_patterns"]
    );
    assert_eq!(
        current["folders"][0]["future"].as_str(),
        Some("folder-extension")
    );
    assert_eq!(
        current["folders"][0]["local_path"],
        initial["folders"][0]["local_path"]
    );
    assert_eq!(current["folders"][0]["enabled"].as_bool(), Some(false));
    assert!(
        root.path()
            .join("backups")
            .read_dir()
            .unwrap()
            .next()
            .is_some()
    );
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(
        save_folder(
            &source,
            &stale,
            FolderChange::Delete {
                identity: renamed.clone()
            }
        ),
        Err(SyncError::Conflict)
    );
    assert_eq!(
        save_folder(
            &source,
            &revision(&source),
            FolderChange::Delete { identity }
        ),
        Err(SyncError::Conflict)
    );
    assert_eq!(
        save_folder(
            &source,
            &revision(&source),
            FolderChange::Add(FolderPatch {
                name: Some("escape".into()),
                local_path: Some("~/.claude".into()),
                remote_path: Some("/escape".into())
            })
        ),
        Err(SyncError::Invalid)
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(
        save_folder(
            &source,
            &revision(&source),
            FolderChange::Delete { identity: renamed }
        ),
        Ok(SyncOutcome::FolderSaved)
    );
    let final_value: toml::Value =
        toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(final_value["folders"].as_array().unwrap().len(), 1);
    assert_eq!(final_value["folders"][0]["name"].as_str(), Some("extra"));
    assert_eq!(final_value["webdav"], initial["webdav"]);
}

// In-process WebDAV fixture. Only the explicitly bound loopback URL is accepted.
async fn asset_server() -> (
    String,
    Arc<Mutex<Option<Vec<u8>>>>,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let stored = Arc::new(Mutex::new(None::<Vec<u8>>));
    let remote = stored.clone();
    let task = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut chunk = [0; 4096];
            let (end, length) = loop {
                let count = socket.read(&mut chunk).await.unwrap();
                assert_ne!(count, 0);
                request.extend_from_slice(&chunk[..count]);
                if let Some(end) = request.windows(4).position(|p| p == b"\r\n\r\n") {
                    let length = String::from_utf8_lossy(&request[..end])
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    break (end + 4, length);
                }
            };
            while request.len() < end + length {
                let count = socket.read(&mut chunk).await.unwrap();
                assert_ne!(count, 0);
                request.extend_from_slice(&chunk[..count]);
            }
            let first = String::from_utf8_lossy(&request[..end]);
            let target = first.split_whitespace().nth(1).unwrap();
            let (status, body) = if first.starts_with("MKCOL ") {
                assert!(target.trim_end_matches('/').ends_with("/claude"));
                ("201 Created", Vec::new())
            } else if first.starts_with("PUT ") {
                assert!(target.ends_with("/claude/settings.json"));
                *remote.lock().unwrap() = Some(request[end..end + length].to_vec());
                ("201 Created", Vec::new())
            } else {
                assert!(first.starts_with("GET "));
                assert!(target.ends_with("/claude/settings.json"));
                match remote.lock().unwrap().clone() {
                    Some(bytes) => ("200 OK", bytes),
                    None => ("404 Not Found", Vec::new()),
                }
            };
            let header = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            socket.write_all(header.as_bytes()).await.unwrap();
            socket.write_all(&body).await.unwrap();
            socket.shutdown().await.unwrap();
        }
    });
    (format!("http://{address}/"), stored, task)
}
#[tokio::test]
async fn sensitive_asset_v2_roundtrip_and_explicit_v1_import_preserve_local_data() {
    let root = tempfile::tempdir().unwrap();
    let source = source(root.path());
    let (url, stored, server) = asset_server().await;
    config(root.path(), &url);
    let roots = super::super::super::resources::roots(&source).unwrap();
    let local = roots.home.join(".claude/settings.json");
    std::fs::create_dir_all(local.parent().unwrap()).unwrap();
    std::fs::write(&local, b"fixture-secret-auth").unwrap();
    let request = |direction, force, password: &str, import| {
        Operation::Assets(AssetRequest {
            revision: revision(&source),
            id: Some("claude-settings".into()),
            direction,
            force,
            passphrase: Some(password.to_string().into()),
            import_plaintext_v1: import,
        })
    };
    let result = execute(
        &source,
        request(AssetDirection::Push, false, "fixture-passphrase", false),
        true,
    )
    .await
    .unwrap();
    assert!(matches!(result,SyncOutcome::Assets(ref rows) if rows[0].result==Ok(None)));
    let encrypted = stored.lock().unwrap().clone().unwrap();
    assert!(
        !encrypted
            .windows(b"fixture-secret-auth".len())
            .any(|w| w == b"fixture-secret-auth")
    );
    std::fs::write(&local, b"local-edit").unwrap();
    let result = execute(
        &source,
        request(AssetDirection::Pull, true, "wrong-passphrase", false),
        true,
    )
    .await
    .unwrap();
    assert!(
        matches!(result,SyncOutcome::Assets(ref rows) if rows[0].result==Err(SyncError::AssetDecrypt))
    );
    assert_eq!(std::fs::read(&local).unwrap(), b"local-edit");
    let result = execute(
        &source,
        request(AssetDirection::Pull, true, "fixture-passphrase", false),
        true,
    )
    .await
    .unwrap();
    let SyncOutcome::Assets(rows) = result else {
        panic!("asset report expected")
    };
    let backup = rows[0].result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(std::fs::read(backup).unwrap(), b"local-edit");
    assert_eq!(std::fs::read(&local).unwrap(), b"fixture-secret-auth");
    *stored.lock().unwrap() = Some(b"legacy-plaintext".to_vec());
    let result = execute(
        &source,
        request(AssetDirection::Pull, true, "fixture-passphrase", false),
        true,
    )
    .await
    .unwrap();
    assert!(
        matches!(result,SyncOutcome::Assets(ref rows) if rows[0].result==Err(SyncError::AssetLegacy))
    );
    assert_eq!(std::fs::read(&local).unwrap(), b"fixture-secret-auth");
    let result = execute(
        &source,
        request(AssetDirection::Pull, true, "fixture-passphrase", true),
        true,
    )
    .await
    .unwrap();
    let SyncOutcome::Assets(rows) = result else {
        panic!("asset report expected")
    };
    let backup = rows[0].result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(std::fs::read(backup).unwrap(), b"fixture-secret-auth");
    assert_eq!(std::fs::read(&local).unwrap(), b"legacy-plaintext");
    assert_eq!(
        stored.lock().unwrap().as_deref(),
        Some(b"legacy-plaintext".as_slice())
    );
    server.abort();
}
