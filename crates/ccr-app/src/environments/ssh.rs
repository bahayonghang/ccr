//! Persistent saved-host sessions and versioned remote documents for native adapters.
//! Tauri continues to use the independent OpenSSH security API.
use super::ssh_security::*;
use ccr_core::core::content_version_token;
use ccr_db::{
    database::repositories::ssh_repo,
    models::ssh::{SshHost, SshKnownHost},
};
use std::{
    collections::HashMap,
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;
mod detect;
mod transport;
pub use detect::{CliSession, CliStatus};
use transport::{Connection, RusshTransport, Transport, bounded};
pub type Error = &'static str;
type RunFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'a>>;
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
    connection: Arc<dyn Connection>,
}
struct Output {
    success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}
pub struct Workspace {
    database: Option<PathBuf>,
    known_hosts: PathBuf,
    lock_directory: Option<PathBuf>,
    trust: SshTrustService,
    session: Mutex<Option<Session>>,
    #[cfg(any(test, feature = "test-fixtures"))]
    synthetic: bool,
    authentication: tokio::sync::Mutex<()>,
    writes: Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
    transport: Arc<dyn Transport>,
}
impl Workspace {
    pub fn new(database: Option<PathBuf>, known_hosts: PathBuf) -> Self {
        Self {
            database,
            known_hosts,
            lock_directory: None,
            trust: SshTrustService::new(),
            session: Mutex::new(None),
            #[cfg(any(test, feature = "test-fixtures"))]
            synthetic: false,
            authentication: tokio::sync::Mutex::new(()),
            writes: Mutex::new(HashMap::new()),
            transport: Arc::new(RusshTransport),
        }
    }
    fn connection(&self, write: bool) -> Result<rusqlite::Connection, Error> {
        ssh_repo::open_host_store(
            self.database.as_ref().ok_or("ssh_database_unavailable")?,
            write,
        )
        .map_err(|_| "ssh_database_unavailable")
    }
    pub fn hosts(&self) -> Result<Vec<Host>, Error> {
        let Some(path) = &self.database else {
            return Ok(Vec::new());
        };
        if !path.try_exists().map_err(|_| "ssh_database_unavailable")? {
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
        let _guard = self.authentication.lock().await;
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
        {
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
        }
        if self.current(&row.id).is_ok() {
            self.clear().await;
        }
        Ok(Host {
            id: row.id,
            name: row.name,
        })
    }
    pub async fn delete_host(&self, id: &str) -> Result<(), Error> {
        let _guard = self.authentication.lock().await;
        if !ssh_repo::delete_host(&self.connection(true)?, id)
            .map_err(|_| "ssh_database_unavailable")?
        {
            return Err("ssh_host_missing");
        }
        if self.current(id).is_ok() {
            self.clear().await;
        }
        Ok(())
    }
    fn current(&self, id: &str) -> Result<Session, Error> {
        self.session
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .filter(|s| s.id == id && !s.connection.is_closed())
            .cloned()
            .ok_or("ssh_not_connected")
    }
    async fn clear(&self) {
        let session = self
            .session
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take();
        if let Some(session) = session {
            let _ = bounded(CancellationToken::new(), session.connection.disconnect()).await;
        }
    }
    async fn revoke(&self, id: uuid::Uuid) {
        let session = {
            let mut current = self.session.lock().unwrap_or_else(|p| p.into_inner());
            if current.as_ref().is_some_and(|s| s.session_id == id) {
                current.take()
            } else {
                None
            }
        };
        if let Some(session) = session {
            let _ = bounded(CancellationToken::new(), session.connection.disconnect()).await;
        }
    }
    /// Returns retained authority without performing network or database access.
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
        let _guard = self.authentication.lock().await;
        self.clear().await;
    }
    fn known(&self, host: &SshHost) -> Result<Option<SshKnownHost>, Error> {
        ssh_repo::get_known_host(&self.connection(false)?, &host.host, host.port)
            .map_err(|_| "ssh_database_unavailable")
    }
    pub async fn probe(&self, id: &str, cancel: CancellationToken) -> Result<Probe, Error> {
        let _guard = bounded(cancel.clone(), async {
            Ok(self.authentication.lock().await)
        })
        .await?;
        self.clear().await;
        let host = self.host(id)?;
        Self::target(&host)?;
        let stored = self.known(&host)?;
        let key = bounded(cancel, self.transport.probe(&host, stored.as_ref())).await?;
        let stored_fingerprint = stored.map(|v| v.fingerprint);
        let status = classify_host_key(stored_fingerprint.as_deref(), &key.fingerprint);
        let fingerprint = key.fingerprint.clone();
        let challenge_id = self.trust.register(host.host.clone(), host.port, key);
        Ok(Probe {
            challenge_id,
            host: host.host,
            port: host.port,
            fingerprint,
            stored_fingerprint,
            status,
        })
    }
    /// Consumes a backend challenge; changed keys cannot replace existing trust.
    pub async fn confirm(
        &self,
        challenge_id: &str,
        cancel: CancellationToken,
    ) -> Result<(), Error> {
        let _guard = bounded(cancel.clone(), async {
            Ok(self.authentication.lock().await)
        })
        .await?;
        self.clear().await;
        if cancel.is_cancelled() {
            return Err("ssh_cancelled");
        }
        let key = self
            .trust
            .consume(challenge_id)
            .map_err(|_| "ssh_challenge_invalid")?;
        let stored = ssh_repo::get_known_host(&self.connection(false)?, &key.host, key.port)
            .map_err(|_| "ssh_database_unavailable")?;
        if stored.is_some_and(|s| s.fingerprint != key.fingerprint) {
            return Err("ssh_host_key_mismatch");
        }
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
        let _guard = bounded(cancel.clone(), async {
            Ok(self.authentication.lock().await)
        })
        .await?;
        self.clear().await;
        let mut host = self.host(id)?;
        Self::target(&host)?;
        RemotePosixPath::root(host.remote_home.as_deref())
            .map_err(|_| "ssh_invalid_remote_path")?;
        let stored = self.known(&host)?.ok_or("ssh_host_key_untrusted")?;
        let connection = bounded(cancel.clone(), self.transport.connect(&host, &stored)).await?;
        let home = bounded(
            cancel.clone(),
            connection.exec("printf '%s' \"$HOME\"".into()),
        )
        .await;
        let home = match home
            .and_then(|o| {
                if !o.success {
                    return Err("ssh_invalid_remote_path");
                }
                String::from_utf8(o.stdout).map_err(|_| "ssh_invalid_remote_path")
            })
            .and_then(|s| {
                if !s.starts_with('/') {
                    return Err("ssh_invalid_remote_path");
                }
                RemotePosixPath::root(Some(&s)).map_err(|_| "ssh_invalid_remote_path")?;
                Ok(s)
            }) {
            Ok(home) => home,
            Err(e) => {
                let _ = bounded(CancellationToken::new(), connection.disconnect()).await;
                return Err(e);
            }
        };
        if host.remote_home.as_deref().is_none_or(|s| s == "~") {
            host.remote_home = Some(home);
        }
        *self.session.lock().unwrap_or_else(|p| p.into_inner()) = Some(Session {
            id: id.into(),
            host,
            session_id: uuid::Uuid::new_v4(),
            connection,
        });
        Ok(())
    }
    fn path(session: &Session, file: ConfigFile) -> Result<RemotePosixPath, Error> {
        RemotePosixPath::root(session.host.remote_home.as_deref())
            .and_then(|p| p.join_relative(file.path()))
            .map_err(|_| "ssh_invalid_remote_path")
    }
    async fn download(
        &self,
        session: &Session,
        path: &RemotePosixPath,
        cancel: CancellationToken,
    ) -> Result<String, Error> {
        let output = bounded(
            cancel,
            session
                .connection
                .exec(format!("cat -- {}", posix_single_quote(path.as_str()))),
        )
        .await?;
        if !output.success {
            return Err("ssh_read_failed");
        }
        String::from_utf8(output.stdout).map_err(|_| "ssh_read_failed")
    }
    pub async fn read(
        &self,
        id: &str,
        file: ConfigFile,
        cancel: CancellationToken,
    ) -> Result<Document, Error> {
        let session = self.current(id)?;
        match self
            .download(&session, &Self::path(&session, file)?, cancel)
            .await
        {
            Ok(content) => Ok(Document {
                environment: id.into(),
                file,
                version: content_version_token(content.as_bytes()),
                content,
                session_id: session.session_id,
            }),
            Err(error) => {
                self.revoke(session.session_id).await;
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
        let session = self.current(&document.environment)?;
        let path = Self::path(&session, document.file)?;
        let lock = self
            .writes
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .entry(format!("{}:{}", session.id, path.as_str()))
            .or_default()
            .clone();
        let _guard = bounded(cancel.clone(), async { Ok(lock.lock().await) }).await?;
        if self.current(&document.environment)?.session_id != document.session_id
            || session.session_id != document.session_id
        {
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
        let result = async {
            let previous = self.download(&session, &path, cancel.clone()).await?;
            if content_version_token(previous.as_bytes()) != document.version {
                return Err("ssh_conflict");
            }
            let backup = path
                .with_suffix(&format!(".bak.{}", uuid::Uuid::new_v4()))
                .map_err(|_| "ssh_invalid_remote_path")?;
            session
                .connection
                .upload(backup, previous, cancel.clone())
                .await?;
            session
                .connection
                .upload(path, content.into(), cancel)
                .await?;
            Ok(Document {
                content: content.into(),
                version: content_version_token(content.as_bytes()),
                ..document.clone()
            })
        }
        .await;
        if result.as_ref().is_err_and(|e| *e != "ssh_conflict") {
            self.revoke(session.session_id).await;
        }
        result
    }
}
#[cfg(any(test, feature = "test-fixtures"))]
pub mod fixture;
#[cfg(test)]
mod tests;
