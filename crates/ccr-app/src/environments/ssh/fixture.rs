//! In-memory persistent SSH sessions. No sockets, processes or credentials are used.
use super::*;
use base64::Engine;
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
#[derive(Default)]
pub struct FixtureTransport {
    files: Mutex<HashMap<(String, String), String>>,
    key: Mutex<String>,
    failure: Mutex<VecDeque<(&'static str, Error)>>,
    batches: Mutex<Vec<String>>,
    cli_outputs: Mutex<HashMap<String, Vec<u8>>>,
    cli_requests: Mutex<Vec<Vec<String>>>,
    cli_delay: Mutex<std::time::Duration>,
    handshakes: AtomicUsize,
    unsupported: AtomicBool,
}
impl FixtureTransport {
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
        self.files
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert((host.into(), path.into()), text.into());
    }
    pub fn file(&self, host: &str, path: &str) -> Option<String> {
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
    fn operation(&self, op: &str, path: &str) -> Result<(), Error> {
        self.batches
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(format!("{op} {path}"));
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
    ) -> RunFuture<'a, Arc<dyn Connection>> {
        Box::pin(async move {
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
            self.closed.store(true, Ordering::SeqCst);
            Ok(())
        })
    }
    fn exec(&self, command: String) -> RunFuture<'_, Output> {
        Box::pin(async move {
            if self.is_closed() {
                return Err("ssh_network_error");
            }
            let stdout = if command == "printf '%s' \"$HOME\"" {
                b"/home/fixture".to_vec()
            } else if let Some(path) = command
                .strip_prefix("cat -- '")
                .and_then(|v| v.strip_suffix('\''))
            {
                self.transport
                    .file(&self.host, path)
                    .ok_or("ssh_read_failed")?
                    .into_bytes()
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
    fn upload(
        &self,
        path: RemotePosixPath,
        content: String,
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
                    .set_file(&self.host, stage.as_str(), &content);
                self.transport
                    .operation("rename", path.as_str())
                    .map_err(|_| "ssh_write_outcome_unknown")?;
                self.transport.set_file(&self.host, path.as_str(), &content);
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
}
