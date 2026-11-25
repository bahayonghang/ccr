#![allow(clippy::unwrap_used)]
use super::*;
use crate::runtime::sync::test_support::asset_server;
use ccr_app::runtime::{Platform, RuntimeOwner};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct SshFixture {
    root: tempfile::TempDir,
    owner: RuntimeOwner,
    services: Services,
    workspace: Arc<ccr_app::environments::ssh::Workspace>,
    transport: Arc<ccr_app::environments::ssh::fixture::FixtureTransport>,
    id: String,
}
impl SshFixture {
    async fn new(url: &str) -> Self {
        use ccr_app::environments::ssh::{HostInput, Workspace};
        use tokio_util::sync::CancellationToken;
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let mut services = Services::new(owner.handle(), source(root.path()));
        services.sync_operations = Arc::new(SyncOperations {
            loopback_fixture: true,
            ..Default::default()
        });
        let (workspace, transport) =
            Workspace::fixture(root.path().join("ssh.db"), root.path().join("known_hosts"));
        let workspace = Arc::new(workspace);
        services
            .ssh
            .set(workspace.clone())
            .unwrap_or_else(|_| panic!("ssh owner"));
        let host = workspace
            .save_host(HostInput {
                name: "Sync fixture".into(),
                host: "sync.fixture.invalid".into(),
                user: "fixture".into(),
                port: 22,
                remote_home: "/srv/sync".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let probe = workspace
            .probe(&host.id, CancellationToken::new())
            .await
            .unwrap();
        workspace
            .confirm(&probe.challenge_id, CancellationToken::new())
            .await
            .unwrap();
        workspace
            .connect(&host.id, CancellationToken::new())
            .await
            .unwrap();
        services.select_environment(EnvironmentId::Ssh(host.id.clone()));
        let fixture = Self {
            root,
            owner,
            services,
            workspace,
            transport,
            id: host.id,
        };
        let mut config = SyncFoldersConfig::default();
        config.webdav.enabled = true;
        config.webdav.url = url.into();
        config.webdav.username = "synthetic-user".into();
        config.webdav.password = "synthetic-password".into();
        config.folders.push(ccr_sync::SyncFolder {
            name: "shared".into(),
            description: String::new(),
            local_path: "~/.claude/shared".into(),
            remote_path: "/data/".into(),
            enabled: true,
            auto_sync: false,
            exclude_patterns: Vec::new(),
        });
        fixture.set(".ccr/sync_folders.toml", &toml::to_string(&config).unwrap());
        std::fs::write(
            fixture.root.path().join("sync_folders.toml"),
            "local poison",
        )
        .unwrap();
        fixture
    }
    fn set(&self, path: &str, content: &str) {
        self.transport.set_file(
            "sync.fixture.invalid",
            &format!("/srv/sync/{path}"),
            content,
        );
    }
    fn key(&self) -> RequestKey {
        RequestKey {
            environment: EnvironmentId::Ssh(self.id.clone()),
            platform: Platform::Claude,
            generation: 1,
            request_id: 1,
        }
    }
    fn revision(&self) -> String {
        ccr_core::core::content_version_token(
            self.transport
                .file("sync.fixture.invalid", "/srv/sync/.ccr/sync_folders.toml")
                .unwrap()
                .as_bytes(),
        )
    }
    async fn run(&self, operation: Operation) -> Result<SyncOutcome, SyncError> {
        self.services.start_sync(self.key(), operation)?;
        let mut updates = self.services.observe_sync();
        tokio::time::timeout(Duration::from_secs(15), async {
            while updates.borrow().busy {
                updates.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        self.services.sync_operation_state().result.clone().unwrap()
    }
    async fn finish(self) {
        self.workspace.disconnect().await;
        tokio::task::spawn_blocking(move || self.owner.shutdown())
            .await
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(self.root.path().join("sync_folders.toml")).unwrap(),
            "local poison"
        );
    }
}

#[tokio::test]
async fn ssh_sync_sensitive_assets_use_original_envelope_and_remote_cas_backup() {
    let server = asset_server().await;
    let stored = &server.settings;
    let f = SshFixture::new(&server.url).await;
    f.set(
        ".claude/settings.json",
        "{\"apiKey\":\"synthetic-remote-secret\"}",
    );
    let request = |revision, direction| {
        Operation::Assets(AssetRequest {
            revision,
            id: Some("claude-settings".into()),
            direction,
            force: true,
            passphrase: Some(Secret::from("synthetic-passphrase")),
            import_plaintext_v1: false,
        })
    };
    assert!(
        matches!(f.run(request(f.revision(), AssetDirection::Push)).await, Ok(SyncOutcome::Assets(rows)) if rows.len()==1 && rows[0].result.is_ok())
    );
    assert!(
        !String::from_utf8_lossy(stored.lock().unwrap().as_ref().unwrap())
            .contains("synthetic-remote-secret")
    );
    f.set(".claude/settings.json", "{\"changed\":true}");
    assert!(
        matches!(f.run(request(f.revision(), AssetDirection::Pull)).await, Ok(SyncOutcome::Assets(rows)) if rows[0].result.is_ok())
    );
    assert!(
        f.transport
            .file("sync.fixture.invalid", "/srv/sync/.claude/settings.json")
            .unwrap()
            .contains("synthetic-remote-secret")
    );
    assert!(
        f.transport
            .batches()
            .iter()
            .any(|batch| batch.starts_with("rename /srv/sync/.claude/settings.json.bak."))
    );
    assert_eq!(
        f.run(request("stale".into(), AssetDirection::Push)).await,
        Err(SyncError::Conflict)
    );
    let database = Operation::Assets(AssetRequest {
        revision: f.revision(),
        id: Some("ccr-platforms".into()),
        direction: AssetDirection::Pull,
        force: true,
        passphrase: Some(Secret::from("synthetic-passphrase")),
        import_plaintext_v1: false,
    });
    assert!(
        matches!(f.run(database).await, Ok(SyncOutcome::Assets(rows)) if rows[0].result==Err(SyncError::RemoteDatabase))
    );
    drop(server);
    f.finish().await;
}

async fn write_only_server() -> (
    String,
    Arc<Mutex<Vec<Vec<u8>>>>,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let output = requests.clone();
    let task = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut chunk = [0; 4096];
            let (end, length) = loop {
                let size = socket.read(&mut chunk).await.unwrap();
                assert!(size > 0);
                request.extend_from_slice(&chunk[..size]);
                if let Some(end) = request.windows(4).position(|b| b == b"\r\n\r\n") {
                    let length = String::from_utf8_lossy(&request[..end])
                        .lines()
                        .find_map(|line| {
                            let (key, value) = line.split_once(':')?;
                            key.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    break (end + 4, length);
                }
            };
            while request.len() < end + length {
                let size = socket.read(&mut chunk).await.unwrap();
                assert!(size > 0);
                request.extend_from_slice(&chunk[..size]);
            }
            assert!(request.starts_with(b"PUT ") || request.starts_with(b"MKCOL "));
            output.lock().unwrap().push(request);
            socket
                .write_all(
                    b"HTTP/1.1 201 Created\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await
                .unwrap();
            socket.shutdown().await.unwrap();
        }
    });
    (format!("http://{address}/"), requests, task)
}

#[tokio::test]
async fn ssh_sync_ordinary_directory_empty_source_and_database_refusal_match_real_metadata() {
    let (url, requests, server) = write_only_server().await;
    let f = SshFixture::new(&url).await;
    f.set(".claude/shared/notes.md", "remote folder input");
    let io = f
        .services
        .remote_configuration_io(&f.key().environment)
        .unwrap();
    assert!(matches!(
        io.metadata(".claude/shared", tokio_util::sync::CancellationToken::new())
            .await,
        Err(ccr_app::environments::io::Error::Ssh(
            "ssh_invalid_remote_path"
        ))
    ));
    assert_eq!(
        f.run(Operation::Push {
            revision: f.revision(),
            folder: 0
        })
        .await,
        Ok(SyncOutcome::Pushed)
    );
    assert!(
        requests
            .lock()
            .unwrap()
            .iter()
            .any(|request| String::from_utf8_lossy(request).contains("remote folder input"))
    );
    f.transport.set_directory("/srv/sync/.claude/empty");
    let old = f.revision();
    let change = FolderChange::Edit {
        identity: ccr_core::core::content_version_token(b"shared"),
        patch: FolderPatch {
            name: None,
            local_path: Some("~/.claude/empty".into()),
            remote_path: None,
        },
    };
    assert_eq!(
        f.run(Operation::Folder {
            revision: old,
            change
        })
        .await,
        Ok(SyncOutcome::FolderSaved)
    );
    assert_eq!(
        f.run(Operation::Push {
            revision: f.revision(),
            folder: 0
        })
        .await,
        Ok(SyncOutcome::Pushed)
    );
    f.set(".claude/empty/profiles.db", "must not rename live DB");
    let before = requests.lock().unwrap().len();
    assert_eq!(
        f.run(Operation::Push {
            revision: f.revision(),
            folder: 0
        })
        .await,
        Err(SyncError::RemoteDatabase)
    );
    assert_eq!(requests.lock().unwrap().len(), before);
    server.abort();
    f.finish().await;
}

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
        Err(SyncError::Remote)
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

#[tokio::test]
async fn selected_folder_failure_retains_its_row_and_reports_an_error_event() {
    use crate::runtime::monitoring::Requests;
    use ccr_app::runtime::TaskEvent;
    use ccr_types::{MonitoringFeedQuery, MonitoringLevel};

    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let source = source(root.path());
    config(root.path(), "https://fixture.invalid/");
    let revision = revision(&source);
    let services = Services::new(owner.handle(), source);
    let key = RequestKey {
        environment: services.environment(),
        platform: Platform::Claude,
        generation: 1,
        request_id: 1,
    };
    let mut updates = services.observe_sync();
    services
        .start_sync(
            key.clone(),
            Operation::TransferFolders {
                revision,
                upload: true,
                selected: Some(0),
            },
        )
        .unwrap();
    let result = tokio::time::timeout(
        Duration::from_secs(10),
        updates.wait_for(|state| state.result.is_some()),
    )
    .await
    .unwrap()
    .unwrap()
    .result
    .clone();
    assert_eq!(
        result,
        Some(Ok(SyncOutcome::Batch(vec![FolderResult {
            name: "fixture".into(),
            result: Err(SyncError::FixtureNetwork),
        }])))
    );

    let mut feed = Requests::new(&services)
        .refresh(
            RequestKey {
                request_id: 2,
                ..key
            },
            MonitoringFeedQuery {
                channel: Some("sync".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let report = tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(event) = feed.next().await {
            if let TaskEvent::Terminal(result) = event {
                let Completion::Success(report) = result.as_ref() else {
                    panic!("monitoring read failed");
                };
                return report.clone();
            }
        }
        panic!("monitoring read had no result");
    })
    .await
    .unwrap();
    assert_eq!(report.entries.len(), 1);
    assert_eq!(report.entries[0].level, MonitoringLevel::Error);
    assert_eq!(report.entries[0].event_type, "sync.operation.failed");
    tokio::task::spawn_blocking(move || owner.shutdown())
        .await
        .unwrap();
}

async fn read_dav_request(socket: &mut tokio::net::TcpStream) -> Vec<u8> {
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
    request
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
        let request = read_dav_request(&mut socket).await;
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
async fn local_asset_path_failure_keeps_known_webdav_evidence() {
    let root = tempfile::tempdir().unwrap();
    let response = br#"<d:multistatus xmlns:d="DAV:"><d:response><d:href>/</d:href><d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response></d:multistatus>"#;
    let (url, request) = server("207 Multi-Status", response, 0).await;
    config(root.path(), &url);
    let source = ConfigSource::Fixture {
        profiles: root.path().join("invalid\0directory").join("profiles.toml"),
        registry: root.path().join("registry.toml"),
        delay: Duration::ZERO,
    };
    let outcome = execute(
        &source,
        Operation::AssetInventory {
            revision: revision(&source),
            probe: true,
        },
        true,
    )
    .await
    .unwrap();
    let SyncOutcome::AssetInventory(rows) = outcome else {
        panic!("expected WebDAV inventory despite unreadable local paths");
    };
    let platform = rows.iter().find(|row| row.id == "ccr-platforms").unwrap();
    assert_eq!(platform.local_path, None);
    assert_eq!(platform.local_exists, None);
    assert_eq!(platform.remote_exists, Some(true));
    let request = request.await.unwrap();
    assert!(String::from_utf8_lossy(&request).starts_with("PROPFIND "));
}

#[tokio::test]
async fn ssh_asset_inventory_scopes_webdav_evidence_and_keeps_incomplete_checks_unknown() {
    use std::sync::atomic::{AtomicU8, Ordering};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let fixture = SshFixture::new(&url).await;
    let config_path = ".ccr/sync_folders.toml";
    let config = fixture
        .transport
        .file("sync.fixture.invalid", "/srv/sync/.ccr/sync_folders.toml")
        .unwrap();
    fixture.set(config_path, "[broken WebDAV");
    fixture.set(".claude/CLAUDE.md", "# Synthetic instructions");

    let SyncOutcome::AssetInventory(environment) = fixture
        .run(Operation::AssetInventory {
            revision: String::new(),
            probe: false,
        })
        .await
        .unwrap()
    else {
        panic!("expected current environment inventory");
    };
    assert_eq!(environment.len(), 5);
    assert!(environment.iter().all(|row| row.remote_exists.is_none()));
    let instructions = environment
        .iter()
        .find(|row| row.id == "claude-memory")
        .unwrap();
    assert_eq!(instructions.local_exists, Some(true));
    assert_eq!(
        instructions.local_path.as_deref(),
        Some("/srv/sync/.claude/CLAUDE.md")
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(25), listener.accept())
            .await
            .is_err()
    );

    fixture.set(config_path, &config);
    let failure_mode = Arc::new(AtomicU8::new(0));
    let server_mode = failure_mode.clone();
    let remote_config = fixture.transport.clone();
    let changed_config = format!("{config}\n# Edited while checking WebDAV\n");
    let server = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let request = read_dav_request(&mut socket).await;
            let request = String::from_utf8_lossy(&request);
            let target = request.split_whitespace().nth(1).unwrap();
            let mode = server_mode.load(Ordering::Acquire);
            if mode == 3 {
                remote_config.set_file(
                    "sync.fixture.invalid",
                    "/srv/sync/.ccr/sync_folders.toml",
                    &changed_config,
                );
            }
            let status = match (mode, target) {
                (2, _) => "503 Service Unavailable",
                (1, path) if path.ends_with("/codex/config.toml") => "503 Service Unavailable",
                (_, path) if path.ends_with("/claude/CLAUDE.md") => "200 OK",
                _ => "404 Not Found",
            };
            let response =
                format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            socket.write_all(response.as_bytes()).await.unwrap();
            socket.shutdown().await.unwrap();
        }
    });
    let revision = fixture.revision();
    let probe = || Operation::AssetInventory {
        revision: revision.clone(),
        probe: true,
    };
    for (mode, expected) in [(0, [1, 4, 0]), (1, [1, 3, 1])] {
        failure_mode.store(mode, Ordering::Release);
        let SyncOutcome::AssetInventory(remote) = fixture.run(probe()).await.unwrap() else {
            panic!("expected WebDAV inventory");
        };
        assert_eq!(
            [Some(true), Some(false), None].map(|state| {
                remote
                    .iter()
                    .filter(|row| row.remote_exists == state)
                    .count()
            }),
            expected
        );
        assert_eq!(
            remote
                .iter()
                .find(|row| row.id == "claude-memory")
                .unwrap()
                .remote_exists,
            Some(true)
        );
        let state = fixture.services.sync_operation_state();
        let active = fixture.key().environment;
        assert_eq!(state.asset_inventory(&active, Some(&revision)), Some(true));
        assert_eq!(state.asset_inventory(&active, Some("new revision")), None);
        assert_eq!(
            state
                .visible_in(&EnvironmentId::Local("fixture".into()))
                .result,
            None
        );
    }

    failure_mode.store(2, Ordering::Release);
    assert_eq!(fixture.run(probe()).await, Err(SyncError::Operation));
    assert_eq!(
        fixture.services.sync_operation_state().result,
        Some(Err(SyncError::Operation))
    );

    failure_mode.store(3, Ordering::Release);
    assert_eq!(fixture.run(probe()).await, Err(SyncError::Conflict));
    server.abort();
    fixture.finish().await;
}
#[tokio::test]
async fn explicit_connection_test_keeps_the_shared_lock_and_scopes_its_result() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let fixture = SshFixture::new(&url).await;
    let revision = fixture.revision();
    let key = fixture.key();
    let local = RequestKey {
        environment: EnvironmentId::Local("local".into()),
        ..key.clone()
    };
    fixture
        .services
        .start_sync(
            key.clone(),
            Operation::Test {
                revision: revision.clone(),
            },
        )
        .unwrap();
    let (mut socket, _) = tokio::time::timeout(Duration::from_secs(10), listener.accept())
        .await
        .unwrap()
        .unwrap();
    let elsewhere = fixture
        .services
        .sync_operation_state()
        .visible_in(&local.environment);
    assert!(elsewhere.busy && elsewhere.kind.is_none() && elsewhere.result.is_none());
    assert_eq!(
        fixture.services.start_sync(
            local.clone(),
            Operation::Test {
                revision: revision.clone()
            }
        ),
        Err(SyncError::Busy)
    );
    let mut request = Vec::new();
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        assert_ne!(socket.read_buf(&mut request).await.unwrap(), 0);
    }
    assert!(request.starts_with(b"PROPFIND / "));
    let body = r#"<d:multistatus xmlns:d="DAV:"><d:response><d:href>/</d:href><d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response></d:multistatus>"#;
    let response = format!(
        "HTTP/1.1 207 Multi-Status\r\nContent-Type: application/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    socket.write_all(response.as_bytes()).await.unwrap();
    socket.shutdown().await.unwrap();
    let mut updates = fixture.services.observe_sync();
    let completed = tokio::time::timeout(
        Duration::from_secs(10),
        updates.wait_for(|state| !state.busy),
    )
    .await
    .unwrap()
    .unwrap()
    .clone();
    assert_eq!(completed.result, Some(Ok(SyncOutcome::Connected)));
    assert_eq!(
        completed.connection_test(&key.environment, &revision),
        Some(ConnectionTestStatus::Succeeded)
    );
    assert_eq!(
        completed.connection_test(&local.environment, &revision),
        None
    );
    assert_eq!(completed.connection_test(&key.environment, "changed"), None);
    assert!(
        fixture
            .run(Operation::AssetInventory {
                revision: revision.clone(),
                probe: false
            })
            .await
            .is_ok()
    );
    assert_eq!(
        fixture
            .services
            .sync_operation_state()
            .connection_test(&key.environment, &revision),
        Some(ConnectionTestStatus::Succeeded)
    );
    fixture.workspace.disconnect().await;
    assert_eq!(
        fixture.services.start_sync(
            key.clone(),
            Operation::Test {
                revision: revision.clone()
            },
        ),
        Err(SyncError::Remote)
    );
    let failed = fixture.services.sync_operation_state();
    assert!(!failed.busy);
    assert_eq!(
        failed.connection_test(&key.environment, &revision),
        Some(ConnectionTestStatus::Failed(SyncError::Remote))
    );
    fixture.finish().await;
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

#[tokio::test]
async fn sensitive_asset_v2_roundtrip_and_explicit_v1_import_preserve_local_data() {
    let root = tempfile::tempdir().unwrap();
    let source = source(root.path());
    let server = asset_server().await;
    let stored = &server.settings;
    config(root.path(), &server.url);
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
    drop(server);
}

#[tokio::test]
async fn folder_workbench_rejects_empty_ssh_batch_instead_of_reporting_success() {
    let f = SshFixture::new("https://fixture.invalid/").await;
    assert_eq!(
        f.run(Operation::Folder {
            revision: f.revision(),
            change: FolderChange::Enabled {
                identity: ccr_core::core::content_version_token(b"shared"),
                enabled: false,
            },
        })
        .await,
        Ok(SyncOutcome::FolderSaved)
    );
    assert_eq!(
        f.run(Operation::TransferFolders {
            revision: f.revision(),
            upload: true,
            selected: None,
        })
        .await,
        Err(SyncError::Invalid)
    );
    f.finish().await;
}

#[tokio::test]
async fn folder_workbench_ssh_download_reports_partial_publication_refusal_and_completion() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let f = SshFixture::new(&url).await;
    f.set(".claude/shared/one.txt", "old-one");
    f.set(".claude/shared/two.txt", "old-two");
    f.set(".claude/good.txt", "old-good");
    f.set(".claude/profiles.db", "fixture-db");
    f.transport.fail_path(
        "rename",
        "/srv/sync/.claude/shared/two.txt",
        "ssh_write_outcome_unknown",
    );
    for (name, local, remote) in [
        ("database", "~/.claude/profiles.db", "/db"),
        ("good", "~/.claude/good.txt", "/good.txt"),
    ] {
        assert_eq!(
            f.run(Operation::Folder {
                revision: f.revision(),
                change: FolderChange::Add(FolderPatch {
                    name: Some(name.into()),
                    local_path: Some(local.into()),
                    remote_path: Some(remote.into()),
                }),
            })
            .await,
            Ok(SyncOutcome::FolderSaved)
        );
    }

    // Complete the independent folder before the partial write revokes SSH authority.
    let current = f
        .transport
        .file("sync.fixture.invalid", "/srv/sync/.ccr/sync_folders.toml")
        .unwrap();
    let mut reordered: SyncFoldersConfig = toml::from_str(&current).unwrap();
    reordered.folders.reverse();
    f.set(
        ".ccr/sync_folders.toml",
        &toml::to_string(&reordered).unwrap(),
    );

    let webdav = tokio::spawn(async move {
        for _ in 0..4 {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                assert_ne!(socket.read_buf(&mut request).await.unwrap(), 0);
            }
            let (status, body) = if request.starts_with(b"PROPFIND /data ") {
                let body = ["/data/one.txt", "/data/two.txt"].map(|path| format!(
                    r#"<D:response><D:href>{path}</D:href><D:propstat><D:prop><D:resourcetype/><D:getcontentlength>8</D:getcontentlength><D:getlastmodified>Sun, 26 Jul 2026 00:00:00 GMT</D:getlastmodified></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response>"#,
                )).concat();
                (
                    "207 Multi-Status",
                    format!(r#"<D:multistatus xmlns:D="DAV:">{body}</D:multistatus>"#),
                )
            } else {
                let content = [
                    (b"GET /data/one.txt ".as_slice(), "new-one!"),
                    (b"GET /data/two.txt ".as_slice(), "new-two!"),
                    (b"GET /good.txt ".as_slice(), "new-good"),
                ]
                .into_iter()
                .find_map(|(line, content)| request.starts_with(line).then_some(content))
                .unwrap_or_else(|| {
                    let line = request
                        .split(|byte| *byte == b'\r')
                        .next()
                        .unwrap_or_default();
                    panic!(
                        "Unexpected loopback request: {}",
                        String::from_utf8_lossy(line)
                    );
                });
                ("200 OK", content.to_owned())
            };
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
            socket.shutdown().await.unwrap();
        }
    });
    let result = f
        .run(Operation::TransferFolders {
            revision: f.revision(),
            selected: None,
            upload: false,
        })
        .await;
    assert_eq!(
        result,
        Ok(SyncOutcome::Batch(vec![
            FolderResult {
                name: "good".into(),
                result: Ok(None)
            },
            FolderResult {
                name: "database".into(),
                result: Err(SyncError::RemoteDatabase)
            },
            FolderResult {
                name: "shared".into(),
                result: Err(SyncError::RemotePartial)
            },
        ]))
    );
    webdav.await.unwrap();
    for (path, content) in [
        ("/srv/sync/.claude/shared/one.txt", "new-one!"),
        ("/srv/sync/.claude/shared/two.txt", "old-two"),
        ("/srv/sync/.claude/good.txt", "new-good"),
        ("/srv/sync/.claude/profiles.db", "fixture-db"),
    ] {
        assert_eq!(
            f.transport.file("sync.fixture.invalid", path).as_deref(),
            Some(content)
        );
    }
    assert!(
        f.transport
            .batches()
            .iter()
            .any(|batch| batch.starts_with("rename /srv/sync/.claude/shared/one.txt.bak."))
    );
    f.finish().await;
}
