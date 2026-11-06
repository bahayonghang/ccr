//! In-memory OpenSSH stand-in. No process, socket, user home, or identity file is opened.
use super::*;
use base64::Engine;
use std::collections::{HashMap, VecDeque};
#[derive(Default)]
pub struct FixtureTransport {
    files: Mutex<HashMap<(String, String), String>>,
    key: Mutex<String>,
    failure: Mutex<VecDeque<(&'static str, Error)>>,
    batches: Mutex<Vec<String>>,
}
impl FixtureTransport {
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
}
impl Workspace {
    pub fn is_fixture(&self) -> bool {
        self.synthetic
    }

    pub fn fixture(database: PathBuf, known_hosts: PathBuf) -> (Self, Arc<FixtureTransport>) {
        let transport = Arc::new(FixtureTransport::default());
        let mut owner = Self::new(Some(database), known_hosts);
        owner.transport = transport.clone();
        owner.synthetic = true;
        (owner, transport)
    }
}
impl Transport for FixtureTransport {
    fn run<'a>(
        &'a self,
        command: tokio::process::Command,
        descriptor: ProcessDescriptor,
        input: Option<Vec<u8>>,
        cancel: CancellationToken,
    ) -> RunFuture<'a> {
        Box::pin(async move {
            if cancel.is_cancelled() {
                return Err("ssh_cancelled");
            }
            let args = command
                .as_std()
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            let last = args.last().ok_or("fixture_bad_command")?;
            let stdout = match descriptor.capability().id() {
                "ssh_keyscan" => {
                    let key = self.key.lock().unwrap_or_else(|p| p.into_inner());
                    let bytes = if key.is_empty() {
                        "fixture-host-key"
                    } else {
                        key.as_str()
                    };
                    format!(
                        "{last} ssh-ed25519 {}\n",
                        base64::engine::general_purpose::STANDARD.encode(bytes)
                    )
                    .into_bytes()
                }
                "openssh" => {
                    let nonce = last
                        .strip_prefix("printf '%s\\n' '")
                        .and_then(|v| v.strip_suffix('\''))
                        .ok_or("fixture_bad_handshake")?;
                    nonce.as_bytes().to_vec()
                }
                "sftp" => {
                    let host = last
                        .rsplit('@')
                        .next()
                        .ok_or("fixture_bad_target")?
                        .to_string();
                    let batch = String::from_utf8(input.ok_or("fixture_missing_input")?)
                        .map_err(|_| "fixture_bad_input")?;
                    self.batches
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .push(batch.clone());
                    for line in batch.lines() {
                        let operation =
                            line.split_whitespace().next().ok_or("fixture_bad_batch")?;
                        let mut failure = self.failure.lock().unwrap_or_else(|p| p.into_inner());
                        if let Some(&(expected, error)) = failure.front()
                            && expected == operation
                        {
                            failure.pop_front();
                            return Err(error);
                        }
                        drop(failure);
                        let fields = line.split('"').skip(1).step_by(2).collect::<Vec<_>>();
                        let mut files = self.files.lock().unwrap_or_else(|p| p.into_inner());
                        match operation {
                            "-mkdir" => {}
                            "get" => {
                                let source = files
                                    .get(&(host.clone(), fields[0].into()))
                                    .ok_or("ssh_read_failed")?;
                                std::fs::write(fields[1], source).map_err(|_| "fixture_io")?;
                            }
                            "put" => {
                                files.insert(
                                    (host.clone(), fields[1].into()),
                                    std::fs::read_to_string(fields[0]).map_err(|_| "fixture_io")?,
                                );
                            }
                            "rename" => {
                                let value = files
                                    .remove(&(host.clone(), fields[0].into()))
                                    .ok_or("fixture_missing_upload")?;
                                files.insert((host.clone(), fields[1].into()), value);
                            }
                            "-rm" => {
                                files.remove(&(host.clone(), fields[0].into()));
                            }
                            _ => return Err("fixture_bad_batch"),
                        }
                    }
                    Vec::new()
                }
                _ => return Err("fixture_unexpected_capability"),
            };
            Ok(Output {
                success: true,
                stdout,
                stderr: Vec::new(),
            })
        })
    }
}
