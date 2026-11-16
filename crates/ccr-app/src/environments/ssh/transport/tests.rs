#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use base64::Engine;
use russh::client::Handler;

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
