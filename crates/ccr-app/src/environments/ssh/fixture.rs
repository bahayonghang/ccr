//! In-memory persistent SSH sessions. No sockets, processes or system credential stores are used.
use super::*;
use base64::Engine;
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
type FixtureOutput = (bool, Vec<u8>, Vec<u8>);
#[derive(Default)]
pub struct FixtureTransport {
    files: Mutex<HashMap<(String, String), Vec<u8>>>,
    unsupported_paths: Mutex<std::collections::BTreeSet<String>>,
    directories: Mutex<std::collections::BTreeSet<String>>,
    key: Mutex<String>,
    failure: Mutex<VecDeque<(&'static str, Error)>>,
    path_failures: Mutex<VecDeque<(&'static str, String, Error)>>,
    batches: Mutex<Vec<String>>,
    cli_outputs: Mutex<HashMap<String, Vec<u8>>>,
    cli_requests: Mutex<Vec<Vec<String>>>,
    cli_delay: Mutex<std::time::Duration>,
    io_delay: Mutex<std::time::Duration>,
    io_started: AtomicUsize,
    home: Mutex<Option<String>>,
    exec_outputs: Mutex<HashMap<String, FixtureOutput>>,
    handshakes: AtomicUsize,
    unsupported: AtomicBool,
    unavailable_database_on_home: Mutex<Option<PathBuf>>,
}
impl FixtureTransport {
    pub fn delay_io(&self, delay: std::time::Duration) {
        *self.io_delay.lock().unwrap_or_else(|p| p.into_inner()) = delay;
    }
    pub fn io_started(&self) -> usize {
        self.io_started.load(Ordering::SeqCst)
    }
    async fn wait_io(&self) {
        self.io_started.fetch_add(1, Ordering::SeqCst);
        let delay = *self.io_delay.lock().unwrap_or_else(|p| p.into_inner());
        tokio::time::sleep(delay).await;
    }
    pub fn set_unsupported_path(&self, path: &str) {
        self.unsupported_paths
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(path.into());
    }
    pub fn set_directory(&self, path: &str) {
        self.directories
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(path.into());
    }
    pub fn set_exec_output(&self, command: &str, success: bool, stdout: &str, stderr: &str) {
        self.exec_outputs
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(
                command.into(),
                (
                    success,
                    stdout.as_bytes().to_vec(),
                    stderr.as_bytes().to_vec(),
                ),
            );
    }
    pub fn set_home(&self, home: &str) {
        *self.home.lock().unwrap_or_else(|p| p.into_inner()) = Some(home.into());
    }
    pub fn set_cli_output(&self, tool: &str, output: &str) {
        self.cli_outputs
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(tool.into(), output.as_bytes().to_vec());
    }
    pub fn cli_requests(&self) -> Vec<Vec<String>> {
        self.cli_requests
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }
    pub fn delay_cli(&self, delay: std::time::Duration) {
        *self.cli_delay.lock().unwrap_or_else(|p| p.into_inner()) = delay;
    }
    pub fn set_file(&self, host: &str, path: &str, text: &str) {
        self.set_bytes(host, path, text.as_bytes());
    }
    pub fn set_bytes(&self, host: &str, path: &str, bytes: &[u8]) {
        self.files
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert((host.into(), path.into()), bytes.into());
    }
    pub fn file(&self, host: &str, path: &str) -> Option<String> {
        self.bytes(host, path)
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }
    pub fn bytes(&self, host: &str, path: &str) -> Option<Vec<u8>> {
        self.files
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(&(host.into(), path.into()))
            .cloned()
    }
    pub fn change_key(&self) {
        *self.key.lock().unwrap_or_else(|p| p.into_inner()) = "changed-key".into();
    }
    pub fn fail(&self, operation: &'static str) {
        self.fail_with(operation, "ssh_network_error");
    }
    pub fn fail_with(&self, operation: &'static str, error: Error) {
        self.failure
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push_back((operation, error));
    }
    pub fn fail_path(&self, operation: &'static str, path: &str, error: Error) {
        self.path_failures
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push_back((operation, path.into(), error));
    }
    pub fn batches(&self) -> Vec<String> {
        self.batches
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }
    pub fn handshakes(&self) -> usize {
        self.handshakes.load(Ordering::SeqCst)
    }
    pub fn disable_posix_rename(&self) {
        self.unsupported.store(true, Ordering::SeqCst);
    }
    /// Make the host store unavailable after successful authentication and HOME detection.
    pub fn break_database_after_home(&self, path: PathBuf) {
        *self
            .unavailable_database_on_home
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(path);
    }
    fn operation(&self, op: &str, path: &str) -> Result<(), Error> {
        self.batches
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(format!("{op} {path}"));
        if self
            .unsupported_paths
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .iter()
            .any(|bad| path == bad || path.starts_with(&format!("{bad}/")))
        {
            return Err("ssh_invalid_remote_path");
        }
        let mut paths = self.path_failures.lock().unwrap_or_else(|p| p.into_inner());
        if paths
            .front()
            .is_some_and(|(operation, target, _)| *operation == op && target == path)
        {
            return Err(paths.pop_front().ok_or("fixture_failure_missing")?.2);
        }
        let mut errors = self.failure.lock().unwrap_or_else(|p| p.into_inner());
        if errors.front().is_some_and(|(expected, _)| *expected == op) {
            return Err(errors.pop_front().ok_or("fixture_failure_missing")?.1);
        }
        Ok(())
    }
    fn key(&self, host: &SshHost) -> Result<ScannedHostKey, Error> {
        let key = self.key.lock().unwrap_or_else(|p| p.into_inner());
        let data = base64::engine::general_purpose::STANDARD.encode(if key.is_empty() {
            "fixture-host-key"
        } else {
            &key
        });
        parse_keyscan_output(
            &format!("[{}]:{} ssh-ed25519 {data}", host.host, host.port),
            &host.host,
            host.port,
        )
        .map_err(|_| "ssh_host_key_invalid")
    }
}
struct FixtureConnector(Arc<FixtureTransport>);
impl Workspace {
    pub fn is_fixture(&self) -> bool {
        self.synthetic
    }
    pub fn fixture(database: PathBuf, known_hosts: PathBuf) -> (Self, Arc<FixtureTransport>) {
        let transport = Arc::new(FixtureTransport::default());
        let lock_directory = database.with_extension("locks");
        let mut owner = Self::new(Some(database), known_hosts);
        owner.lock_directory = Some(lock_directory);
        owner.transport = Arc::new(FixtureConnector(transport.clone()));
        owner.synthetic = true;
        owner.credentials = Credentials(Some(
            keyring_core::mock::Store::new().unwrap_or_else(|_| panic!("mock store")),
        ));
        (owner, transport)
    }
}
impl Transport for FixtureConnector {
    fn probe<'a>(
        &'a self,
        host: &'a SshHost,
        _: Option<&'a SshKnownHost>,
    ) -> RunFuture<'a, ScannedHostKey> {
        Box::pin(async move { self.0.key(host) })
    }
    fn connect<'a>(
        &'a self,
        host: &'a SshHost,
        stored: &'a SshKnownHost,
        auth: &'a Authentication,
    ) -> RunFuture<'a, Arc<dyn Connection>> {
        Box::pin(async move {
            self.0.operation(auth.method.as_str(), "authentication")?;
            if auth.method == AuthMethod::Password && auth.secret.is_none() {
                return Err("ssh_credentials_missing");
            }
            self.0.handshakes.fetch_add(1, Ordering::SeqCst);
            if self.0.key(host)?.fingerprint != stored.fingerprint {
                return Err("ssh_host_key_mismatch");
            }
            Ok(Arc::new(FixtureConnection {
                transport: self.0.clone(),
                host: host.host.clone(),
                closed: AtomicBool::new(false),
            }) as Arc<dyn Connection>)
        })
    }
}
struct FixtureConnection {
    transport: Arc<FixtureTransport>,
    host: String,
    closed: AtomicBool,
}
impl Connection for FixtureConnection {
    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }
    fn disconnect(&self) -> RunFuture<'_, ()> {
        Box::pin(async {
            self.transport.operation("disconnect", &self.host)?;
            self.closed.store(true, Ordering::SeqCst);
            Ok(())
        })
    }
    fn exec(&self, command: String) -> RunFuture<'_, Output> {
        Box::pin(async move {
            if self.is_closed() {
                return Err("ssh_network_error");
            }
            let fixture_output = self
                .transport
                .exec_outputs
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .get(&command)
                .cloned();
            if let Some((success, stdout, stderr)) = fixture_output {
                self.transport.operation("exec", &command)?;
                let delay = *self
                    .transport
                    .cli_delay
                    .lock()
                    .unwrap_or_else(|p| p.into_inner());
                tokio::time::sleep(delay).await;
                return Ok(Output {
                    success,
                    stdout,
                    stderr,
                });
            }
            let stdout = if command == "printf '%s' \"$HOME\"" {
                self.transport.operation("home", "")?;
                let path = self
                    .transport
                    .unavailable_database_on_home
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .take();
                if let Some(path) = path {
                    std::fs::remove_file(&path).map_err(|_| "ssh_database_unavailable")?;
                    std::fs::create_dir(path).map_err(|_| "ssh_database_unavailable")?;
                }
                self.transport
                    .home
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .as_deref()
                    .unwrap_or("/home/fixture")
                    .as_bytes()
                    .to_vec()
            } else {
                let tool = detect::CLI_TOOLS
                    .iter()
                    .find(|(_, tool)| {
                        command == format!("command -v {} || true", posix_single_quote(tool))
                    })
                    .ok_or("fixture_bad_command")?
                    .1;
                self.transport
                    .cli_requests
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .push(vec![command]);
                self.transport.operation("detect", tool)?;
                let delay = *self
                    .transport
                    .cli_delay
                    .lock()
                    .unwrap_or_else(|p| p.into_inner());
                tokio::time::sleep(delay).await;
                self.transport
                    .cli_outputs
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .get(tool)
                    .cloned()
                    .unwrap_or_default()
            };
            if stdout.len() > 2 * 1024 * 1024 {
                return Err("ssh_file_too_large");
            }
            Ok(Output {
                success: true,
                stdout,
                stderr: Vec::new(),
            })
        })
    }
    fn download(&self, path: RemotePosixPath) -> RunFuture<'_, Option<Vec<u8>>> {
        Box::pin(async move {
            if self.is_closed() {
                return Err("ssh_network_error");
            }
            self.transport.wait_io().await;
            self.transport.operation("get", path.as_str())?;
            let content = self.transport.bytes(&self.host, path.as_str());
            if content.as_ref().is_some_and(|s| s.len() > 2 * 1024 * 1024) {
                return Err("ssh_file_too_large");
            }
            Ok(content)
        })
    }
    fn upload(
        &self,
        path: RemotePosixPath,
        content: Vec<u8>,
        cancel: CancellationToken,
    ) -> RunFuture<'_, ()> {
        Box::pin(async move {
            if self.is_closed() {
                return Err("ssh_network_error");
            }
            if self.transport.unsupported.load(Ordering::SeqCst) {
                return Err("ssh_write_outcome_unknown");
            }
            let stage = path
                .with_suffix(".ccr-tmp-fixture")
                .map_err(|_| "ssh_invalid_remote_path")?;
            let result = bounded(cancel, async {
                self.transport.operation("put", stage.as_str())?;
                self.transport
                    .set_bytes(&self.host, stage.as_str(), &content);
                self.transport
                    .operation("rename", path.as_str())
                    .map_err(|_| "ssh_write_outcome_unknown")?;
                self.transport
                    .set_bytes(&self.host, path.as_str(), &content);
                Ok(())
            })
            .await;
            if result.is_err() {
                self.transport
                    .operation("-rm", stage.as_str())
                    .map_err(|_| "ssh_cleanup_failed")?;
            }
            self.transport
                .files
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .remove(&(self.host.clone(), stage.as_str().into()));
            result
        })
    }
    fn entries(
        &self,
        path: RemotePosixPath,
    ) -> RunFuture<'_, Option<Vec<super::super::io::DirectoryEntry>>> {
        Box::pin(async move {
            use super::super::io::{DirectoryEntry, EntryKind};
            if self.is_closed() {
                return Err("ssh_network_error");
            }
            self.transport.wait_io().await;
            self.transport.operation("list", path.as_str())?;
            let prefix = format!("{}/", path.as_str());
            let files = self
                .transport
                .files
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            let mut entries = std::collections::BTreeMap::new();
            for ((host, path), _) in files.iter().filter(|((host, _), _)| host == &self.host) {
                let _ = host;
                if let Some(relative) = path.strip_prefix(&prefix) {
                    if let Some((name, _)) = relative.split_once('/') {
                        entries.insert(name.to_owned(), EntryKind::Directory);
                    } else {
                        entries.insert(relative.to_owned(), EntryKind::File);
                    }
                }
            }
            for path in self
                .transport
                .unsupported_paths
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .iter()
            {
                if let Some(relative) = path.strip_prefix(&prefix)
                    && !relative.contains('/')
                {
                    entries.insert(relative.into(), EntryKind::Unsupported);
                }
            }
            let exists = !entries.is_empty()
                || self
                    .transport
                    .directories
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .contains(path.as_str());
            Ok(exists.then(|| {
                entries
                    .into_iter()
                    .map(|(name, kind)| DirectoryEntry { name, kind })
                    .collect()
            }))
        })
    }
    fn remove(&self, path: RemotePosixPath) -> RunFuture<'_, ()> {
        Box::pin(async move {
            if self.is_closed() {
                return Err("ssh_network_error");
            }
            self.transport.operation("remove", path.as_str())?;
            self.transport
                .files
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .remove(&(self.host.clone(), path.as_str().into()));
            Ok(())
        })
    }
    fn is_directory(&self, path: RemotePosixPath) -> RunFuture<'_, bool> {
        Box::pin(async move {
            if self.is_closed() {
                return Err("ssh_network_error");
            }
            self.transport.operation("stat-directory", path.as_str())?;
            Ok(self
                .transport
                .directories
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .contains(path.as_str())
                || self
                    .transport
                    .files
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .keys()
                    .any(|(host, file)| {
                        host == &self.host && file.starts_with(&format!("{}/", path.as_str()))
                    }))
        })
    }
    fn metadata(&self, path: RemotePosixPath) -> RunFuture<'_, Option<(u64, Option<u64>)>> {
        Box::pin(async move {
            if self.is_closed() {
                return Err("ssh_network_error");
            }
            self.transport.operation("stat", path.as_str())?;
            Ok(self
                .transport
                .bytes(&self.host, path.as_str())
                .map(|bytes| (bytes.len() as u64, None)))
        })
    }
}
