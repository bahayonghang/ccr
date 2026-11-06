//! Saved-host sessions and SFTP documents shared by native adapters.
//! Identity paths remain backend-only; no operation falls back to local files.
use super::ssh_security::*;
use crate::jobs::process::{ProcessDescriptor, ProcessGateway};
use ccr_core::core::content_version_token;
use ccr_db::{
    database::repositories::ssh_repo,
    models::ssh::{SshHost, SshKnownHost},
};
use std::{
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;
mod detect;
pub use detect::{CliSession, CliStatus};

pub type Error = &'static str;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Host {
    pub id: String,
    pub name: String,
}
#[derive(Clone)]
pub struct HostInput {
    pub id: Option<String>,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub remote_home: String,
}

#[derive(Clone, Debug)]
pub struct Probe {
    pub challenge_id: String,
    pub host: String,
    pub port: u16,
    pub fingerprint: String,
    pub stored_fingerprint: Option<String>,
    pub status: HostKeyStatus,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigFile {
    Claude,
    Codex,
    Gemini,
    OpenCode,
}
impl ConfigFile {
    pub const ALL: [Self; 4] = [Self::Claude, Self::Codex, Self::Gemini, Self::OpenCode];
    pub fn path(self) -> &'static str {
        match self {
            Self::Claude => ".claude/settings.json",
            Self::Codex => ".codex/config.toml",
            Self::Gemini => ".gemini/antigravity-cli/settings.json",
            Self::OpenCode => ".opencode/opencode.json",
        }
    }
}
#[derive(Clone)]
pub struct Document {
    pub environment: String,
    pub file: ConfigFile,
    pub content: String,
    pub version: String,
    session_id: uuid::Uuid,
}
#[derive(Clone)]
struct Session {
    id: String,
    host: SshHost,
    session_id: uuid::Uuid,
}
struct Output {
    success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}
type RunFuture<'a> = Pin<Box<dyn Future<Output = Result<Output, Error>> + Send + 'a>>;
trait Transport: Send + Sync {
    fn run<'a>(
        &'a self,
        command: tokio::process::Command,
        descriptor: ProcessDescriptor,
        input: Option<Vec<u8>>,
        cancel: CancellationToken,
    ) -> RunFuture<'a>;
}
struct OpenSsh;
impl Transport for OpenSsh {
    fn run<'a>(
        &'a self,
        command: tokio::process::Command,
        descriptor: ProcessDescriptor,
        input: Option<Vec<u8>>,
        cancel: CancellationToken,
    ) -> RunFuture<'a> {
        Box::pin(async move {
            let out = ProcessGateway::execute_command_cancellable(
                command,
                &descriptor,
                input,
                cancel.clone(),
            )
            .await
            .map_err(|error| {
                if error == "process_cancelled" {
                    "ssh_cancelled"
                } else if error.contains("_wait_failed:") {
                    "ssh_cleanup_failed"
                } else {
                    "ssh_network_error"
                }
            })?;
            if cancel.is_cancelled() && !out.status.success() {
                return Err("ssh_cancelled");
            }
            if out.timed_out || out.stdout_truncated || out.stderr_truncated {
                return Err("ssh_network_error");
            }
            Ok(Output {
                success: out.status.success(),
                stdout: out.stdout,
                stderr: out.stderr,
            })
        })
    }
}
pub struct Workspace {
    database: Option<PathBuf>,
    known_hosts: PathBuf,
    lock_directory: Option<PathBuf>,
    trust: SshTrustService,
    session: Mutex<Option<Session>>,
    #[cfg(any(test, feature = "test-fixtures"))]
    synthetic: bool,
    serial: tokio::sync::Mutex<()>,
    transport: Arc<dyn Transport>,
}
impl Workspace {
    pub fn new(database: Option<PathBuf>, known_hosts: PathBuf) -> Self {
        Self {
            database,
            known_hosts,
            lock_directory: None,
            #[cfg(any(test, feature = "test-fixtures"))]
            synthetic: false,
            trust: SshTrustService::new(),
            session: Mutex::new(None),
            serial: tokio::sync::Mutex::new(()),
            transport: Arc::new(OpenSsh),
        }
    }
    fn connection(&self, write: bool) -> Result<rusqlite::Connection, Error> {
        let path = self.database.as_ref().ok_or("ssh_database_unavailable")?;
        ssh_repo::open_host_store(path, write).map_err(|_| "ssh_database_unavailable")
    }
    pub fn hosts(&self) -> Result<Vec<Host>, Error> {
        if let Some(path) = &self.database {
            if !path.try_exists().map_err(|_| "ssh_database_unavailable")? {
                return Ok(Vec::new());
            }
        } else {
            return Ok(Vec::new());
        }
        ssh_repo::list_hosts(&self.connection(false)?)
            .map(|rows| {
                rows.into_iter()
                    .map(|row| Host {
                        id: row.id,
                        name: row.name,
                    })
                    .collect()
            })
            .map_err(|_| "ssh_database_unavailable")
    }
    pub fn host_details(&self, id: &str) -> Result<HostInput, Error> {
        let host = self.host(id)?;
        Ok(HostInput {
            id: Some(host.id),
            name: host.name,
            host: host.host,
            port: host.port,
            user: host.username,
            remote_home: host.remote_home.unwrap_or_else(|| "~".into()),
        })
    }
    pub async fn save_host(&self, input: HostInput) -> Result<Host, Error> {
        let _guard = self.serial.lock().await;
        if input.name.trim().is_empty()
            || input.name.len() > 128
            || input.name.chars().any(char::is_control)
        {
            return Err("ssh_invalid_target");
        }
        let target = SshTarget::new(&input.host, input.port, Some(&input.user), None)
            .map_err(|_| "ssh_invalid_target")?;
        RemotePosixPath::root(Some(&input.remote_home)).map_err(|_| "ssh_invalid_remote_path")?;
        let now = chrono::Utc::now();
        let mut row = match input.id {
            Some(id) => self.host(&id)?,
            None => SshHost {
                id: uuid::Uuid::new_v4().to_string(),
                name: String::new(),
                host: String::new(),
                port: 22,
                username: String::new(),
                identity_file: None,
                remote_home: None,
                created_at: now,
                updated_at: now,
                last_connected_at: None,
            },
        };
        row.name = input.name.trim().into();
        row.host = target.host().into();
        row.port = target.port();
        row.username = input.user;
        row.remote_home = Some(input.remote_home);
        row.updated_at = now;
        let conn = self.connection(true)?;
        if ssh_repo::get_host(&conn, &row.id)
            .map_err(|_| "ssh_database_unavailable")?
            .is_some()
        {
            if !ssh_repo::update_host(&conn, &row).map_err(|_| "ssh_database_unavailable")? {
                return Err("ssh_host_missing");
            }
        } else {
            ssh_repo::insert_host(&conn, &row).map_err(|_| "ssh_database_unavailable")?;
        }
        if self.connected(&row.id) {
            self.clear();
        }
        Ok(Host {
            id: row.id,
            name: row.name,
        })
    }
    pub async fn delete_host(&self, id: &str) -> Result<(), Error> {
        let _guard = self.serial.lock().await;
        if !ssh_repo::delete_host(&self.connection(true)?, id)
            .map_err(|_| "ssh_database_unavailable")?
        {
            return Err("ssh_host_missing");
        }
        if self.connected(id) {
            self.clear();
        }
        Ok(())
    }
    fn host(&self, id: &str) -> Result<SshHost, Error> {
        ssh_repo::get_host(&self.connection(false)?, id)
            .map_err(|_| "ssh_database_unavailable")?
            .ok_or("ssh_host_missing")
    }
    fn target(host: &SshHost) -> Result<SshTarget, Error> {
        SshTarget::new(
            &host.host,
            host.port,
            Some(&host.username),
            host.identity_file.as_deref(),
        )
        .map_err(|_| "ssh_invalid_target")
    }
    fn current(&self, id: &str) -> Result<Session, Error> {
        self.session
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .filter(|s| s.id == id)
            .cloned()
            .ok_or("ssh_not_connected")
    }
    fn clear(&self) {
        *self.session.lock().unwrap_or_else(|p| p.into_inner()) = None;
    }
    /// Held logical session authority, not TCP liveness. No lookup or transport.
    pub fn retained_session_count(&self) -> usize {
        usize::from(
            self.session
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .is_some(),
        )
    }
    pub fn connected(&self, id: &str) -> bool {
        self.current(id).is_ok()
    }
    pub async fn disconnect(&self) {
        let _guard = self.serial.lock().await;
        self.clear();
    }
    async fn scan(
        &self,
        host: &SshHost,
        cancel: CancellationToken,
    ) -> Result<ScannedHostKey, Error> {
        if cancel.is_cancelled() {
            return Err("ssh_cancelled");
        }
        let target = Self::target(host)?;
        let descriptor = ProcessDescriptor::ssh_keyscan();
        let mut command = ProcessGateway::command(&descriptor).map_err(|_| "ssh_network_error")?;
        command.args([
            "-T",
            "5",
            "-p",
            &target.port().to_string(),
            "-t",
            "ed25519,ecdsa,rsa",
            target.host(),
        ]);
        let output = self
            .transport
            .run(command, descriptor, None, cancel)
            .await?;
        if !output.success {
            return Err("ssh_network_error");
        }
        parse_keyscan_output(
            &String::from_utf8_lossy(&output.stdout),
            target.host(),
            target.port(),
        )
        .map_err(|_| "ssh_host_key_invalid")
    }
    pub async fn probe(&self, id: &str, cancel: CancellationToken) -> Result<Probe, Error> {
        let _guard = self.serial.lock().await;
        self.clear();
        let host = self.host(id)?;
        let target = Self::target(&host)?;
        let key = self.scan(&host, cancel.clone()).await?;
        if cancel.is_cancelled() {
            return Err("ssh_cancelled");
        }
        let stored =
            ssh_repo::get_known_host(&self.connection(false)?, target.host(), target.port())
                .map_err(|_| "ssh_database_unavailable")?
                .map(|v| v.fingerprint);
        let status = classify_host_key(stored.as_deref(), &key.fingerprint);
        let fingerprint = key.fingerprint.clone();
        let challenge_id = self
            .trust
            .register(target.host().into(), target.port(), key);
        Ok(Probe {
            challenge_id,
            host: target.host().into(),
            port: target.port(),
            fingerprint,
            stored_fingerprint: stored,
            status,
        })
    }
    /// Confirmation consumes only the backend challenge, never renderer key material.
    pub async fn confirm(
        &self,
        challenge_id: &str,
        cancel: CancellationToken,
    ) -> Result<(), Error> {
        let _guard = self.serial.lock().await;
        // Changing trust cannot retain write authority from an earlier handshake.
        self.clear();
        if cancel.is_cancelled() {
            return Err("ssh_cancelled");
        }
        let key = self
            .trust
            .consume(challenge_id)
            .map_err(|_| "ssh_challenge_invalid")?;
        persist_known_host_with_locks(
            self.known_hosts.clone(),
            key.host.clone(),
            key.port,
            key.key_type.clone(),
            key.key_data,
            self.lock_directory.clone(),
        )
        .await
        .map_err(|_| "ssh_trust_write_failed")?;
        ssh_repo::upsert_known_host(
            &self.connection(true)?,
            &SshKnownHost {
                host: key.host,
                port: key.port,
                key_type: key.key_type,
                fingerprint: key.fingerprint,
                confirmed_at: chrono::Utc::now(),
            },
        )
        .map_err(|_| "ssh_trust_write_failed")
    }
    pub async fn connect(&self, id: &str, cancel: CancellationToken) -> Result<(), Error> {
        let _guard = self.serial.lock().await;
        self.clear();
        let host = self.host(id)?;
        let target = Self::target(&host)?;
        RemotePosixPath::root(host.remote_home.as_deref())
            .map_err(|_| "ssh_invalid_remote_path")?;
        let key = self.scan(&host, cancel.clone()).await?;
        let stored =
            ssh_repo::get_known_host(&self.connection(false)?, target.host(), target.port())
                .map_err(|_| "ssh_database_unavailable")?
                .map(|v| v.fingerprint);
        match classify_host_key(stored.as_deref(), &key.fingerprint) {
            HostKeyStatus::Matched => {}
            HostKeyStatus::Mismatch => return Err("ssh_host_key_mismatch"),
            _ => return Err("ssh_host_key_untrusted"),
        }
        if cancel.is_cancelled() {
            return Err("ssh_cancelled");
        }
        persist_known_host_with_locks(
            self.known_hosts.clone(),
            target.host().into(),
            target.port(),
            key.key_type,
            key.key_data,
            self.lock_directory.clone(),
        )
        .await
        .map_err(|_| "ssh_trust_write_failed")?;
        let nonce = format!("__CCR_SSH_OK__{}", uuid::Uuid::new_v4());
        let mut command = target
            .ssh_command(&self.known_hosts, 10)
            .map_err(|_| "ssh_invalid_target")?;
        command.arg(format!("printf '%s\\n' {}", posix_single_quote(&nonce)));
        let out = self
            .transport
            .run(command, ProcessDescriptor::openssh(), None, cancel.clone())
            .await?;
        if cancel.is_cancelled() {
            return Err("ssh_cancelled");
        }
        if !out.success || String::from_utf8_lossy(&out.stdout).trim() != nonce {
            return Err(classify_ssh_failure(&String::from_utf8_lossy(&out.stderr)).code());
        }
        *self.session.lock().unwrap_or_else(|p| p.into_inner()) = Some(Session {
            id: id.into(),
            host,
            session_id: uuid::Uuid::new_v4(),
        });
        Ok(())
    }
    fn path(session: &Session, file: ConfigFile) -> Result<RemotePosixPath, Error> {
        RemotePosixPath::root(session.host.remote_home.as_deref())
            .and_then(|p| p.join_relative(file.path()))
            .map_err(|_| "ssh_invalid_remote_path")
    }
    async fn sftp(
        &self,
        session: &Session,
        batch: String,
        cancel: CancellationToken,
    ) -> Result<(), Error> {
        if cancel.is_cancelled() {
            return Err("ssh_cancelled");
        }
        let command = Self::target(&session.host)?
            .sftp_command(&self.known_hosts)
            .map_err(|_| "ssh_invalid_target")?;
        let output = self
            .transport
            .run(
                command,
                ProcessDescriptor::sftp(),
                Some(batch.into_bytes()),
                cancel,
            )
            .await?;
        if !output.success {
            return Err(classify_ssh_failure(&String::from_utf8_lossy(&output.stderr)).code());
        }
        Ok(())
    }
    async fn download(
        &self,
        session: &Session,
        path: &RemotePosixPath,
        cancel: CancellationToken,
    ) -> Result<String, Error> {
        let temp = tempfile::tempdir().map_err(|_| "ssh_local_staging_failed")?;
        let local = temp.path().join("download");
        self.sftp(session, sftp_read_batch(path, &local), cancel)
            .await?;
        // Bound a remote file before allocating its contents.
        if std::fs::metadata(&local)
            .map_err(|_| "ssh_read_failed")?
            .len()
            > 2 * 1024 * 1024
        {
            return Err("ssh_file_too_large");
        }
        std::fs::read_to_string(local).map_err(|_| "ssh_read_failed")
    }
    async fn upload(
        &self,
        session: &Session,
        path: &RemotePosixPath,
        content: &str,
        cancel: CancellationToken,
    ) -> Result<(), Error> {
        let temp = tempfile::tempdir().map_err(|_| "ssh_local_staging_failed")?;
        let local = temp.path().join("upload");
        let stage = || {
            ccr_core::core::write_guarded(
                &local,
                content.as_bytes(),
                &ccr_core::core::WriteOptions {
                    secret: true,
                    ..Default::default()
                },
            )
        };
        match &self.lock_directory {
            Some(directory) => {
                ccr_core::core::lock::LockManager::with_scoped_directory(directory, stage)
            }
            None => stage(),
        }
        .map_err(|_| "ssh_local_staging_failed")?;
        let plan = sftp_write_plan(path, &local, uuid::Uuid::new_v4())
            .map_err(|_| "ssh_invalid_remote_path")?;
        let result = async {
            self.sftp(session, plan.upload_batch, cancel.clone())
                .await?;
            self.sftp(session, plan.rename_batch, cancel)
                .await
                .map_err(|_| "ssh_write_outcome_unknown")
        }
        .await;
        if result.is_err() {
            let cleanup = self
                .sftp(session, plan.cleanup_batch, CancellationToken::new())
                .await;
            if cleanup == Err("ssh_cleanup_failed") {
                return Err("ssh_cleanup_failed");
            }
        }
        result
    }
    pub async fn read(
        &self,
        id: &str,
        file: ConfigFile,
        cancel: CancellationToken,
    ) -> Result<Document, Error> {
        let _guard = self.serial.lock().await;
        let session = self.current(id)?;
        let result = self
            .download(&session, &Self::path(&session, file)?, cancel)
            .await;
        match result {
            Ok(content) => Ok(Document {
                environment: id.into(),
                file,
                version: content_version_token(content.as_bytes()),
                content,
                session_id: session.session_id,
            }),
            Err(error) => {
                self.clear();
                Err(error)
            }
        }
    }
    pub async fn save(
        &self,
        document: &Document,
        content: &str,
        cancel: CancellationToken,
    ) -> Result<Document, Error> {
        let _guard = self.serial.lock().await;
        let session = self.current(&document.environment)?;
        if document.session_id != session.session_id {
            return Err("ssh_session_changed");
        }
        if content.len() > 2 * 1024 * 1024 {
            return Err("ssh_file_too_large");
        }
        match document.file {
            ConfigFile::Codex => {
                content
                    .parse::<toml::Value>()
                    .map_err(|_| "ssh_invalid_document")?;
            }
            _ => {
                serde_json::from_str::<serde_json::Value>(content)
                    .map_err(|_| "ssh_invalid_document")?;
            }
        }
        let path = Self::path(&session, document.file)?;
        let previous = match self.download(&session, &path, cancel.clone()).await {
            Ok(value) => value,
            Err(error) => {
                self.clear();
                return Err(error);
            }
        };
        if content_version_token(previous.as_bytes()) != document.version {
            return Err("ssh_conflict");
        }
        let backup = RemotePosixPath::root(session.host.remote_home.as_deref())
            .and_then(|root| {
                root.join_relative(&format!(
                    "{}.bak.{}",
                    document.file.path(),
                    uuid::Uuid::new_v4()
                ))
            })
            .map_err(|_| "ssh_invalid_remote_path")?;
        if let Err(error) = self
            .upload(&session, &backup, &previous, cancel.clone())
            .await
        {
            self.clear();
            return Err(error);
        }
        if let Err(error) = self.upload(&session, &path, content, cancel).await {
            self.clear();
            return Err(error);
        }
        Ok(Document {
            content: content.into(),
            version: content_version_token(content.as_bytes()),
            ..document.clone()
        })
    }
}

#[cfg(any(test, feature = "test-fixtures"))]
pub mod fixture;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, Workspace, Arc<fixture::FixtureTransport>) {
        let root = tempfile::tempdir().unwrap();
        let database = root.path().join("ssh.db");
        let conn = rusqlite::Connection::open(&database).unwrap();
        conn.execute_batch("CREATE TABLE ssh_hosts (id TEXT PRIMARY KEY,name TEXT,host TEXT,port INTEGER,username TEXT,identity_file TEXT,remote_home TEXT,created_at TEXT,updated_at TEXT,last_connected_at TEXT);CREATE TABLE ssh_known_hosts (host TEXT,port INTEGER,key_type TEXT,fingerprint TEXT,confirmed_at TEXT,PRIMARY KEY(host,port));").unwrap();
        for id in ["one", "two"] {
            ssh_repo::insert_host(
                &conn,
                &SshHost {
                    id: id.into(),
                    name: id.into(),
                    host: format!("{id}.example"),
                    port: 22,
                    username: "fixture".into(),
                    identity_file: None,
                    remote_home: Some("/home/fixture".into()),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    last_connected_at: None,
                },
            )
            .unwrap();
        }
        let (owner, transport) = Workspace::fixture(database, root.path().join("ssh/known_hosts"));
        transport.set_file(
            "one.example",
            "/home/fixture/.claude/settings.json",
            r#"{"one":1}"#,
        );
        transport.set_file(
            "two.example",
            "/home/fixture/.claude/settings.json",
            r#"{"two":2}"#,
        );
        (root, owner, transport)
    }
    async fn connect(owner: &Workspace, id: &str) {
        let cancel = CancellationToken::new();
        let probe = owner.probe(id, cancel.clone()).await.unwrap();
        owner
            .confirm(&probe.challenge_id, cancel.clone())
            .await
            .unwrap();
        owner.connect(id, cancel).await.unwrap();
    }
    #[tokio::test]
    async fn retained_session_metrics_observe_only_memory_and_follow_real_owner_transitions() {
        struct NoTransport;
        impl Transport for NoTransport {
            fn run<'a>(
                &'a self,
                _: tokio::process::Command,
                _: ProcessDescriptor,
                _: Option<Vec<u8>>,
                _: CancellationToken,
            ) -> RunFuture<'a> {
                panic!("metrics must never invoke transport");
            }
        }
        let (_root, mut owner, _transport) = fixture();
        assert_eq!(owner.retained_session_count(), 0);
        assert!(
            owner
                .connect("one", CancellationToken::new())
                .await
                .is_err()
        );
        assert_eq!(owner.retained_session_count(), 0);
        connect(&owner, "one").await;
        // Removing lookup authority and poisoning transport must not change the
        // existing retained capability or turn a metric read into another probe.
        owner.database = None;
        owner.transport = Arc::new(NoTransport);
        assert_eq!(owner.retained_session_count(), 1);
        assert_eq!(owner.retained_session_count(), 1);
        owner.disconnect().await;
        assert_eq!(owner.retained_session_count(), 0);
    }
    #[tokio::test]
    async fn untrusted_changed_and_cancelled_connections_never_become_active() {
        let (root, owner, transport) = fixture();
        let cancel = CancellationToken::new();
        assert_eq!(
            owner.connect("one", cancel.clone()).await,
            Err("ssh_host_key_untrusted")
        );
        assert!(!root.path().join("ssh/known_hosts").exists());
        connect(&owner, "one").await;
        assert!(owner.connected("one"));
        assert!(owner.lock_directory.as_ref().unwrap().is_dir());
        transport.change_key();
        assert_eq!(
            owner.connect("one", cancel.clone()).await,
            Err("ssh_host_key_mismatch")
        );
        assert!(!owner.connected("one"));
        cancel.cancel();
        assert_eq!(owner.connect("one", cancel).await, Err("ssh_cancelled"));
    }
    #[tokio::test]
    async fn remote_documents_bind_session_and_write_backup_before_target() {
        let (_root, owner, transport) = fixture();
        connect(&owner, "one").await;
        let cancel = CancellationToken::new();
        let original = owner
            .read("one", ConfigFile::Claude, cancel.clone())
            .await
            .unwrap();
        let saved = owner
            .save(&original, r#"{"one":3}"#, cancel.clone())
            .await
            .unwrap();
        assert_eq!(
            transport
                .file("one.example", "/home/fixture/.claude/settings.json")
                .as_deref(),
            Some(r#"{"one":3}"#)
        );
        assert_eq!(
            transport
                .file("two.example", "/home/fixture/.claude/settings.json")
                .as_deref(),
            Some(r#"{"two":2}"#)
        );
        assert!(
            transport
                .batches()
                .iter()
                .any(|b| b.starts_with("rename ") && b.contains(".bak."))
        );
        assert!(matches!(
            owner.save(&original, r#"{"one":4}"#, cancel.clone()).await,
            Err("ssh_conflict")
        ));
        connect(&owner, "one").await;
        assert!(matches!(
            owner.save(&saved, r#"{"one":5}"#, cancel).await,
            Err("ssh_session_changed")
        ));
    }
    #[tokio::test]
    async fn confirming_changed_trust_revokes_loaded_document_authority() {
        let (_root, owner, transport) = fixture();
        connect(&owner, "one").await;
        let cancel = CancellationToken::new();
        let document = owner
            .read("one", ConfigFile::Claude, cancel.clone())
            .await
            .unwrap();
        transport.change_key();
        let probe = owner.probe("one", cancel.clone()).await.unwrap();
        assert_eq!(probe.status, HostKeyStatus::Mismatch);
        assert!(!owner.connected("one"));
        owner
            .confirm(&probe.challenge_id, cancel.clone())
            .await
            .unwrap();
        assert!(matches!(
            owner.save(&document, "{}", cancel.clone()).await,
            Err("ssh_not_connected")
        ));
        owner.connect("one", cancel.clone()).await.unwrap();
        assert!(matches!(
            owner.save(&document, "{}", cancel).await,
            Err("ssh_session_changed")
        ));
        assert_eq!(
            transport
                .file("one.example", "/home/fixture/.claude/settings.json")
                .as_deref(),
            Some(r#"{"one":1}"#)
        );
    }
    #[tokio::test]
    async fn cancelled_upload_does_not_hide_cleanup_process_failure() {
        let (_root, owner, transport) = fixture();
        connect(&owner, "one").await;
        let cancel = CancellationToken::new();
        let document = owner
            .read("one", ConfigFile::Claude, cancel.clone())
            .await
            .unwrap();
        transport.fail_with("put", "ssh_cancelled");
        transport.fail_with("-rm", "ssh_cleanup_failed");
        assert!(matches!(
            owner.save(&document, "{}", cancel).await,
            Err("ssh_cleanup_failed")
        ));
        assert!(!owner.connected("one"));
        assert_eq!(
            transport
                .file("one.example", "/home/fixture/.claude/settings.json")
                .as_deref(),
            Some(r#"{"one":1}"#)
        );
    }
    #[tokio::test]
    async fn failed_upload_leaves_target_unchanged_and_cleans_stage() {
        let (_root, owner, transport) = fixture();
        connect(&owner, "one").await;
        let cancel = CancellationToken::new();
        let original = owner
            .read("one", ConfigFile::Claude, cancel.clone())
            .await
            .unwrap();
        transport.fail("put");
        assert!(owner.save(&original, r#"{"one":3}"#, cancel).await.is_err());
        assert_eq!(
            transport
                .file("one.example", "/home/fixture/.claude/settings.json")
                .as_deref(),
            Some(r#"{"one":1}"#)
        );
        assert!(transport.batches().last().unwrap().starts_with("-rm "));
        assert!(!owner.connected("one"));
    }
    #[tokio::test]
    async fn fixed_cli_detection_requires_active_session_and_keeps_strict_commands() {
        let (_root, owner, transport) = fixture();
        assert!(matches!(owner.cli_session("one"), Err("ssh_not_connected")));
        assert!(transport.cli_requests().is_empty());
        connect(&owner, "one").await;
        transport.set_cli_output("claude", "/usr/local/bin/claude\n");
        transport.set_cli_output("agy", "/home/fixture/bin/agy\n");
        let authority = owner.cli_session("one").unwrap();
        let rows = owner
            .detect_cli(&authority, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(
            rows.iter().map(|r| r.name).collect::<Vec<_>>(),
            ["claude", "codex", "gemini", "opencode"]
        );
        assert_eq!(rows[0].path.as_deref(), Some("/usr/local/bin/claude"));
        assert!(rows[0].installed);
        assert!(!rows[1].installed);
        assert!(rows[1].path.is_none());
        assert!(rows[2].installed);
        let requests = transport.cli_requests();
        assert_eq!(requests.len(), 4);
        for (args, (_, tool)) in requests.iter().zip(detect::CLI_TOOLS) {
            assert_eq!(
                args.last().unwrap(),
                &format!("command -v '{}' || true", tool)
            );
            for required in [
                "BatchMode=yes".into(),
                "StrictHostKeyChecking=yes".into(),
                "GlobalKnownHostsFile=none".into(),
                format!("UserKnownHostsFile={}", owner.known_hosts.display()),
            ] {
                assert!(args.contains(&required));
            }
        }
        connect(&owner, "one").await;
        assert_eq!(
            owner.detect_cli(&authority, CancellationToken::new()).await,
            Err("ssh_session_changed")
        );
        assert_eq!(transport.cli_requests().len(), 4);
        assert!(owner.connected("one"));
        owner.disconnect().await;
        assert_eq!(
            owner.detect_cli(&authority, CancellationToken::new()).await,
            Err("ssh_not_connected")
        );
    }
    #[tokio::test]
    async fn malformed_cli_output_and_cancel_do_not_publish_installed_rows() {
        let (_root, owner, transport) = fixture();
        connect(&owner, "one").await;
        transport.set_cli_output("claude", "banner\n/usr/bin/claude");
        let authority = owner.cli_session("one").unwrap();
        assert_eq!(
            owner.detect_cli(&authority, CancellationToken::new()).await,
            Err("ssh_cli_output_invalid")
        );
        assert!(!owner.connected("one"));
        connect(&owner, "one").await;
        transport.set_cli_output("claude", "/usr/bin/claude\n");
        transport.delay_cli(std::time::Duration::from_millis(100));
        let authority = owner.cli_session("one").unwrap();
        let cancel = CancellationToken::new();
        let token = cancel.clone();
        let before = transport.cli_requests().len();
        let (detected, ()) = tokio::join!(owner.detect_cli(&authority, cancel), async {
            tokio::time::timeout(std::time::Duration::from_secs(2), async {
                while transport.cli_requests().len() == before {
                    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                }
            })
            .await
            .unwrap();
            token.cancel();
        });
        assert_eq!(detected, Err("ssh_cancelled"));
        assert!(!owner.connected("one"));
        assert_eq!(transport.cli_requests().len(), before + 1);
    }
}
