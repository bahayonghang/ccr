use super::*;
use russh::{Channel, ChannelId, server};
use russh_sftp::protocol::{Attrs, Status, Version};
use tokio::sync::Notify;

struct SftpPeer {
    entered: Arc<Notify>,
    events: Arc<Mutex<Vec<String>>>,
    failure: &'static str,
    stage: Option<String>,
}
impl russh_sftp::server::Handler for SftpPeer {
    type Error = StatusCode;

    fn unimplemented(&self) -> Self::Error {
        StatusCode::OpUnsupported
    }

    async fn init(&mut self, _: u32, _: HashMap<String, String>) -> Result<Version, StatusCode> {
        let mut version = Version::new();
        if self.failure != "unsupported" {
            version
                .extensions
                .insert("posix-rename@openssh.com".into(), "1".into());
        }
        Ok(version)
    }

    async fn lstat(&mut self, id: u32, path: String) -> Result<Attrs, StatusCode> {
        self.events.lock().unwrap().push(format!("lstat {path}"));
        if self.failure == "stalled-path" {
            self.entered.notify_one();
            return std::future::pending().await;
        }
        Ok(Attrs {
            id,
            attrs: FileAttributes {
                permissions: Some(if self.failure == "linked" {
                    0o120777
                } else {
                    0o040700
                }),
                ..Default::default()
            },
        })
    }

    async fn mkdir(
        &mut self,
        id: u32,
        _: String,
        attrs: FileAttributes,
    ) -> Result<Status, StatusCode> {
        assert_eq!(attrs.permissions, Some(0o700));
        self.events.lock().unwrap().push("mkdir".into());
        Ok(ok(id))
    }

    async fn open(
        &mut self,
        id: u32,
        path: String,
        flags: OpenFlags,
        attrs: FileAttributes,
    ) -> Result<russh_sftp::protocol::Handle, StatusCode> {
        assert!(path.starts_with("/home/fixture/settings.json.ccr-tmp-"));
        assert_eq!(
            flags.bits(),
            (OpenFlags::WRITE | OpenFlags::CREATE | OpenFlags::EXCLUDE).bits()
        );
        assert_eq!(attrs.permissions, Some(0o600));
        self.stage = Some(path);
        self.events.lock().unwrap().push("open".into());
        Ok(russh_sftp::protocol::Handle {
            id,
            handle: "synthetic-write".into(),
        })
    }

    async fn write(
        &mut self,
        id: u32,
        handle: String,
        offset: u64,
        data: Vec<u8>,
    ) -> Result<Status, StatusCode> {
        assert_eq!(handle, "synthetic-write");
        assert_eq!(offset, 0);
        assert_eq!(data, b"synthetic content");
        self.events.lock().unwrap().push("write".into());
        if matches!(self.failure, "write" | "cleanup") {
            return Err(StatusCode::Failure);
        }
        Ok(ok(id))
    }

    async fn close(&mut self, id: u32, handle: String) -> Result<Status, StatusCode> {
        assert_eq!(handle, "synthetic-write");
        self.events.lock().unwrap().push("close".into());
        Ok(ok(id))
    }

    async fn extended(
        &mut self,
        id: u32,
        request: String,
        data: Vec<u8>,
    ) -> Result<Packet, StatusCode> {
        assert_eq!(request, "posix-rename@openssh.com");
        assert_eq!(
            data,
            rename_payload(
                self.stage.as_deref().unwrap(),
                "/home/fixture/settings.json"
            )
        );
        self.events.lock().unwrap().push("rename".into());
        if self.failure == "rename" {
            return Err(StatusCode::Failure);
        }
        Ok(Packet::Status(ok(id)))
    }

    async fn remove(&mut self, _: u32, path: String) -> Result<Status, StatusCode> {
        assert_eq!(Some(path.as_str()), self.stage.as_deref());
        self.events.lock().unwrap().push("remove".into());
        Err(if self.failure == "cleanup" {
            StatusCode::PermissionDenied
        } else {
            StatusCode::NoSuchFile
        })
    }
}

fn ok(id: u32) -> Status {
    Status {
        id,
        status_code: StatusCode::Ok,
        error_message: String::new(),
        language_tag: String::new(),
    }
}

struct SshPeer {
    channel: Option<Channel<server::Msg>>,
    sftp: Option<SftpPeer>,
    tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}
impl server::Handler for SshPeer {
    type Error = russh::Error;

    async fn auth_none(&mut self, _: &str) -> Result<server::Auth, Self::Error> {
        Ok(server::Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<server::Msg>,
        reply: server::ChannelOpenHandle,
        _: &mut server::Session,
    ) -> Result<(), Self::Error> {
        self.channel = Some(channel);
        reply.accept().await;
        Ok(())
    }

    async fn subsystem_request(
        &mut self,
        id: ChannelId,
        name: &str,
        session: &mut server::Session,
    ) -> Result<(), Self::Error> {
        assert_eq!(name, "sftp");
        session.channel_success(id)?;
        let channel = self.channel.take().unwrap();
        let peer = self.sftp.take().unwrap();
        self.tasks.lock().unwrap().push(tokio::spawn(async move {
            russh_sftp::server::run(channel.into_stream(), peer).await;
        }));
        Ok(())
    }
}

struct Peer {
    connection: LiveConnection,
    server: server::RunningSession<SshPeer>,
    entered: Arc<Notify>,
    events: Arc<Mutex<Vec<String>>>,
    tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

impl Peer {
    async fn finish(self) {
        let tasks = std::mem::take(&mut *self.tasks.lock().unwrap());
        for task in tasks {
            task.abort();
            if let Err(error) = task.await {
                assert!(error.is_cancelled());
            }
        }
        self.server
            .handle()
            .disconnect(russh::Disconnect::ByApplication, "".into(), "en".into())
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(1), self.server)
            .await
            .unwrap()
            .unwrap();
    }
}

async fn peer(failure: &'static str) -> Peer {
    use russh::keys::ssh_key::{
        PrivateKey,
        private::{Ed25519Keypair, KeypairData},
    };
    let key = PrivateKey::new(
        KeypairData::Ed25519(Ed25519Keypair::from_seed(&[7; 32])),
        "synthetic server",
    )
    .unwrap();
    let fingerprint = key
        .public_key()
        .fingerprint(russh::keys::HashAlg::Sha256)
        .to_string();
    let config = Arc::new(server::Config {
        keys: vec![key],
        ..Default::default()
    });
    let entered = Arc::new(Notify::new());
    let events = Arc::new(Mutex::new(Vec::new()));
    let tasks = Arc::new(Mutex::new(Vec::new()));
    let peer = SshPeer {
        channel: None,
        sftp: Some(SftpPeer {
            entered: entered.clone(),
            events: events.clone(),
            failure,
            stage: None,
        }),
        tasks: tasks.clone(),
    };
    let (client_stream, server_stream) = tokio::io::duplex(65536);
    let (server, handle) = tokio::join!(
        server::run_stream(config, server_stream, peer),
        client::connect_stream(
            Arc::new(client::Config::default()),
            client_stream,
            Verifier {
                host: "fixture.invalid".into(),
                port: 22,
                trusted: Some(fingerprint),
                observed: Arc::new(Mutex::new(None)),
            },
        )
    );
    let mut handle = handle.unwrap();
    assert!(handle.authenticate_none("fixture").await.unwrap().success());
    let server = server.unwrap();
    Peer {
        connection: LiveConnection(Arc::new(handle)),
        server,
        entered,
        events,
        tasks,
    }
}

#[tokio::test]
async fn production_upload_cancels_during_path_checks_before_creating_a_stage() {
    let peer = peer("stalled-path").await;
    let cancel = CancellationToken::new();
    let result = {
        let upload = peer.connection.upload(
            RemotePosixPath::root(Some("/home/fixture/settings.json")).unwrap(),
            b"synthetic content".to_vec(),
            cancel.clone(),
        );
        tokio::pin!(upload);
        tokio::select! {
            _ = peer.entered.notified() => {},
            result = &mut upload => panic!("upload ended before path check: {result:?}"),
        }
        cancel.cancel();
        tokio::time::timeout(Duration::from_secs(1), &mut upload).await
    };
    let events = peer.events.lock().unwrap().clone();
    peer.finish().await;
    assert_eq!(result, Ok(Err("ssh_cancelled")));
    assert_eq!(events, ["lstat /home"]);
}

#[tokio::test]
async fn production_upload_orders_staging_close_and_atomic_rename_and_preserves_errors() {
    for (failure, expected, operations) in [
        ("none", Ok(()), vec!["open", "write", "close", "rename"]),
        ("unsupported", Err("ssh_write_outcome_unknown"), vec![]),
        ("linked", Err("ssh_invalid_remote_path"), vec![]),
        (
            "write",
            Err("ssh_network_error"),
            vec!["open", "write", "close", "remove"],
        ),
        (
            "rename",
            Err("ssh_write_outcome_unknown"),
            vec!["open", "write", "close", "rename", "remove"],
        ),
        (
            "cleanup",
            Err("ssh_cleanup_failed"),
            vec!["open", "write", "close", "remove"],
        ),
    ] {
        let peer = peer(failure).await;
        let result = peer
            .connection
            .upload(
                RemotePosixPath::root(Some("/home/fixture/settings.json")).unwrap(),
                b"synthetic content".to_vec(),
                CancellationToken::new(),
            )
            .await;
        let events = peer
            .events
            .lock()
            .unwrap()
            .iter()
            .filter(|event| !event.starts_with("lstat ") && *event != "mkdir")
            .cloned()
            .collect::<Vec<_>>();
        peer.finish().await;
        assert_eq!(result, expected, "{failure}");
        assert_eq!(events, operations, "{failure}");
    }
}
