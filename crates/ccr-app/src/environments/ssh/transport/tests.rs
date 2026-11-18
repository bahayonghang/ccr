#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use base64::Engine;
use russh::client::Handler;

mod upload;

#[tokio::test]
async fn callback_rejects_probe_and_changed_keys_and_preserves_openssh_fingerprint() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&11u32.to_be_bytes());
    bytes.extend_from_slice(b"ssh-ed25519");
    bytes.extend_from_slice(&32u32.to_be_bytes());
    bytes.extend_from_slice(&[7u8; 32]);
    let text = format!(
        "ssh-ed25519 {}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    );
    let public = russh::keys::PublicKey::from_openssh(&text).unwrap();
    let fingerprint = public.fingerprint(russh::keys::HashAlg::Sha256).to_string();
    let public = PublicKeyOrCertificate::PublicKey {
        key: public,
        hash_alg: None,
    };
    for port in [22, 2222] {
        let observed = Arc::new(Mutex::new(None));
        let mut handler = Verifier {
            host: "fixture.invalid".into(),
            port,
            trusted: None,
            observed: observed.clone(),
        };
        assert!(!handler.check_server_key(&public).await.unwrap());
        assert_eq!(
            observed.lock().unwrap().as_ref().unwrap().fingerprint,
            fingerprint
        );
        handler.trusted = Some(fingerprint.clone());
        assert!(handler.check_server_key(&public).await.unwrap());
        handler.trusted = Some("SHA256:changed".into());
        assert!(!handler.check_server_key(&public).await.unwrap());
    }
}

type Requests = Arc<Mutex<Vec<(String, Vec<u8>)>>>;
struct RenameServer {
    unsupported: bool,
    observed: Requests,
}
impl russh_sftp::server::Handler for RenameServer {
    type Error = StatusCode;
    fn unimplemented(&self) -> Self::Error {
        StatusCode::OpUnsupported
    }
    async fn extended(
        &mut self,
        id: u32,
        request: String,
        data: Vec<u8>,
    ) -> Result<Packet, Self::Error> {
        self.observed.lock().unwrap().push((request, data));
        if self.unsupported {
            return Err(StatusCode::OpUnsupported);
        }
        Ok(Packet::Status(russh_sftp::protocol::Status {
            id,
            status_code: StatusCode::Ok,
            error_message: String::new(),
            language_tag: String::new(),
        }))
    }
}
#[tokio::test]
async fn raw_sftp_extended_encodes_two_strings_and_fails_closed_on_unsupported() {
    for unsupported in [false, true] {
        let (client, server_stream) = tokio::io::duplex(4096);
        let observed = Arc::new(Mutex::new(Vec::new()));
        let server = RenameServer {
            unsupported,
            observed: observed.clone(),
        };
        let task = tokio::spawn(russh_sftp::server::run(server_stream, server));
        let raw = RawSftpSession::new(client);
        raw.init().await.unwrap();
        let result = atomic_replace(&raw, "/tmp/合成", "/config/target").await;
        assert_eq!(
            result,
            if unsupported {
                Err("ssh_write_outcome_unknown")
            } else {
                Ok(())
            }
        );
        let requests = observed.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].0, "posix-rename@openssh.com");
        assert_eq!(requests[0].1, rename_payload("/tmp/合成", "/config/target"));
        assert_eq!(&requests[0].1[..4], &11u32.to_be_bytes());
        drop(requests);
        drop(raw);
        task.abort();
    }
}
#[tokio::test(start_paused = true)]
async fn total_timeout_and_cancel_are_bounded() {
    assert_eq!(
        bounded::<()>(CancellationToken::new(), std::future::pending()).await,
        Err("ssh_network_error")
    );
    let token = CancellationToken::new();
    token.cancel();
    assert_eq!(
        bounded::<()>(token, async { Ok(()) }).await,
        Err("ssh_cancelled")
    );
}

struct ReadServer {
    link: Option<&'static str>,
    opened: Arc<std::sync::atomic::AtomicBool>,
    content: Option<Vec<u8>>,
    closed: Arc<std::sync::atomic::AtomicBool>,
}
impl russh_sftp::server::Handler for ReadServer {
    type Error = StatusCode;
    async fn lstat(
        &mut self,
        id: u32,
        path: String,
    ) -> Result<russh_sftp::protocol::Attrs, StatusCode> {
        let permissions = if self.link == Some(path.as_str()) {
            0o120777
        } else if matches!(path.as_str(), "/home" | "/home/fixture") {
            0o040700
        } else if path == "/home/fixture/settings.json" && self.content.is_some() {
            0o100600
        } else {
            return Err(StatusCode::NoSuchFile);
        };
        Ok(russh_sftp::protocol::Attrs {
            id,
            attrs: FileAttributes {
                permissions: Some(permissions),
                ..Default::default()
            },
        })
    }
    fn unimplemented(&self) -> Self::Error {
        StatusCode::OpUnsupported
    }
    async fn open(
        &mut self,
        id: u32,
        filename: String,
        flags: OpenFlags,
        _: FileAttributes,
    ) -> Result<russh_sftp::protocol::Handle, StatusCode> {
        self.opened.store(true, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(filename, "/home/fixture/settings.json");
        assert_eq!(flags.bits(), OpenFlags::READ.bits());
        self.content.as_ref().ok_or(StatusCode::NoSuchFile)?;
        Ok(russh_sftp::protocol::Handle {
            id,
            handle: "read-handle".into(),
        })
    }
    async fn read(
        &mut self,
        id: u32,
        handle: String,
        offset: u64,
        length: u32,
    ) -> Result<russh_sftp::protocol::Data, StatusCode> {
        assert_eq!(handle, "read-handle");
        let content = self.content.as_ref().unwrap();
        if offset as usize >= content.len() {
            return Err(StatusCode::Eof);
        }
        let end = (offset as usize + length as usize).min(content.len());
        Ok(russh_sftp::protocol::Data {
            id,
            data: content[offset as usize..end].to_vec(),
        })
    }
    async fn close(
        &mut self,
        id: u32,
        _: String,
    ) -> Result<russh_sftp::protocol::Status, StatusCode> {
        self.closed.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(russh_sftp::protocol::Status {
            id,
            status_code: StatusCode::Ok,
            error_message: String::new(),
            language_tag: String::new(),
        })
    }
}

#[tokio::test]
async fn production_sftp_reader_handles_missing_empty_binary_chunks_limit_and_closes_handles() {
    for content in [
        None,
        Some(Vec::new()),
        Some(vec![b'x'; 40000]),
        Some(vec![255]),
        Some(vec![b'x'; 2 * 1024 * 1024 + 1]),
    ] {
        let (client, stream) = tokio::io::duplex(65536);
        let closed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let task = tokio::spawn(russh_sftp::server::run(
            stream,
            ReadServer {
                link: None,
                opened: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                content: content.clone(),
                closed: closed.clone(),
            },
        ));
        let raw = RawSftpSession::new(client);
        raw.init().await.unwrap();
        let result = read_sftp(
            &raw,
            &RemotePosixPath::root(Some("/home/fixture/settings.json")).unwrap(),
        )
        .await;
        match &content {
            None => assert_eq!(result, Ok(None)),
            Some(bytes) if bytes.len() > 2 * 1024 * 1024 => {
                assert_eq!(result, Err("ssh_file_too_large"))
            }
            Some(bytes) => assert_eq!(result, Ok(Some(bytes.clone()))),
        }
        assert_eq!(
            closed.load(std::sync::atomic::Ordering::SeqCst),
            content.is_some()
        );
        drop(raw);
        task.abort();
    }
}

#[tokio::test]
async fn production_sftp_reader_refuses_linked_leaf_and_ancestors_before_open() {
    for link in ["/home", "/home/fixture", "/home/fixture/settings.json"] {
        let (client, stream) = tokio::io::duplex(65536);
        let opened = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let task = tokio::spawn(russh_sftp::server::run(
            stream,
            ReadServer {
                link: Some(link),
                opened: opened.clone(),
                content: Some(b"private".to_vec()),
                closed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            },
        ));
        let raw = RawSftpSession::new(client);
        raw.init().await.unwrap();
        assert_eq!(
            read_sftp(
                &raw,
                &RemotePosixPath::root(Some("/home/fixture/settings.json")).unwrap()
            )
            .await,
            Err("ssh_invalid_remote_path")
        );
        assert!(!opened.load(std::sync::atomic::Ordering::SeqCst));
        drop(raw);
        task.abort();
    }
}
#[tokio::test(start_paused = true)]
async fn cancelled_read_cleanup_uses_the_original_deadline_and_discards_late_bytes() {
    for completes in [false, true] {
        let started = tokio::time::Instant::now();
        let cancel = CancellationToken::new();
        let cancel_worker = cancel.clone();
        let (result, ()) = tokio::join!(
            read_bounded(cancel_worker, async {
                if completes {
                    tokio::time::sleep(Duration::from_secs(6)).await;
                    Ok(Some(b"late private data".to_vec()))
                } else {
                    std::future::pending::<Result<Option<Vec<u8>>, Error>>().await
                }
            }),
            async {
                tokio::time::sleep(Duration::from_secs(3)).await;
                cancel.cancel();
            }
        );
        assert_eq!(
            result,
            Err(if completes {
                "ssh_cancelled"
            } else {
                "ssh_cleanup_failed"
            })
        );
        assert_eq!(
            started.elapsed(),
            Duration::from_secs(if completes { 6 } else { 10 })
        );
    }
}
