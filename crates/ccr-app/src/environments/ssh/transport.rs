use super::*;
use russh::{
    ChannelMsg, client,
    keys::{PublicKeyOrCertificate, agent::client::AgentClient},
};
use russh_sftp::{
    client::RawSftpSession,
    protocol::{FileAttributes, OpenFlags, Packet, StatusCode},
};
use std::time::Duration;
mod authentication;

pub(super) trait Transport: Send + Sync {
    fn probe<'a>(
        &'a self,
        host: &'a SshHost,
        stored: Option<&'a SshKnownHost>,
    ) -> RunFuture<'a, ScannedHostKey>;
    fn connect<'a>(
        &'a self,
        host: &'a SshHost,
        stored: &'a SshKnownHost,
        auth: &'a Authentication,
    ) -> RunFuture<'a, Arc<dyn Connection>>;
}
pub(super) trait Connection: Send + Sync {
    fn is_closed(&self) -> bool;
    fn disconnect(&self) -> RunFuture<'_, ()>;
    fn exec(&self, command: String) -> RunFuture<'_, Output>;
    fn download(&self, path: RemotePosixPath) -> RunFuture<'_, Option<Vec<u8>>>;
    fn upload(
        &self,
        path: RemotePosixPath,
        content: Vec<u8>,
        cancel: CancellationToken,
    ) -> RunFuture<'_, ()>;
    fn list(&self, path: RemotePosixPath) -> RunFuture<'_, Vec<String>>;
    fn remove(&self, path: RemotePosixPath) -> RunFuture<'_, ()>;
    fn metadata(&self, path: RemotePosixPath) -> RunFuture<'_, Option<(u64, Option<u64>)>>;
}
pub(super) async fn bounded<T>(
    cancel: CancellationToken,
    work: impl Future<Output = Result<T, Error>>,
) -> Result<T, Error> {
    tokio::select! { biased; _ = cancel.cancelled() => Err("ssh_cancelled"), result = tokio::time::timeout(Duration::from_secs(10), work) => result.map_err(|_| "ssh_network_error")? }
}
struct Verifier {
    host: String,
    port: u16,
    trusted: Option<String>,
    observed: Arc<Mutex<Option<ScannedHostKey>>>,
}
impl client::Handler for Verifier {
    type Error = russh::Error;
    async fn check_server_key(
        &mut self,
        key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        let PublicKeyOrCertificate::PublicKey { key, .. } = key else {
            return Ok(false);
        };
        let text = key.to_openssh().map_err(|_| russh::Error::UnknownKey)?;
        let key = parse_keyscan_output(
            &format!("[{}]:{} {}", self.host, self.port, text),
            &self.host,
            self.port,
        )
        .map_err(|_| russh::Error::UnknownKey)?;
        let accepted = self.trusted.as_ref().is_some_and(|s| *s == key.fingerprint);
        *self.observed.lock().unwrap_or_else(|p| p.into_inner()) = Some(key);
        Ok(accepted)
    }
}
fn config(stored: Option<&SshKnownHost>) -> client::Config {
    let mut config = client::Config {
        keepalive_interval: Some(Duration::from_secs(15)),
        keepalive_max: 3,
        ..Default::default()
    };
    if let Some(stored) = stored {
        let algorithms = config
            .preferred
            .key
            .iter()
            .filter(|a| {
                a.as_str() == stored.key_type
                    || (stored.key_type == "ssh-rsa" && a.as_str().starts_with("rsa-sha2-"))
            })
            .cloned()
            .collect::<Vec<_>>();
        config.preferred.key = algorithms.into();
    }
    config
}
pub(super) struct RusshTransport;
impl Transport for RusshTransport {
    fn probe<'a>(
        &'a self,
        host: &'a SshHost,
        stored: Option<&'a SshKnownHost>,
    ) -> RunFuture<'a, ScannedHostKey> {
        Box::pin(async move {
            let observed = Arc::new(Mutex::new(None));
            let handler = Verifier {
                host: host.host.clone(),
                port: host.port,
                trusted: None,
                observed: observed.clone(),
            };
            // probe 必须在主机密钥回调拒绝，不进入认证，也不写信任库。
            let _ = client::connect(
                Arc::new(config(stored)),
                (host.host.as_str(), host.port),
                handler,
            )
            .await;
            observed
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .take()
                .ok_or("ssh_network_error")
        })
    }
    fn connect<'a>(
        &'a self,
        host: &'a SshHost,
        stored: &'a SshKnownHost,
        auth: &'a Authentication,
    ) -> RunFuture<'a, Arc<dyn Connection>> {
        Box::pin(async move {
            let observed = Arc::new(Mutex::new(None));
            let handler = Verifier {
                host: host.host.clone(),
                port: host.port,
                trusted: Some(stored.fingerprint.clone()),
                observed: observed.clone(),
            };
            let handle = client::connect(
                Arc::new(config(Some(stored))),
                (host.host.as_str(), host.port),
                handler,
            )
            .await
            .map_err(|_| {
                if observed
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .as_ref()
                    .is_some_and(|k| k.fingerprint != stored.fingerprint)
                {
                    "ssh_host_key_mismatch"
                } else {
                    "ssh_network_error"
                }
            })?;
            // RAII 守卫在认证取消/超时/失败时也显式发送断开。
            let mut connection = LiveConnection(Arc::new(handle));
            if auth.method != AuthMethod::Agent {
                authentication::authenticate(&mut connection, host, auth).await?;
                return Ok(Arc::new(connection) as Arc<dyn Connection>);
            }
            let mut failure = "ssh_auth_failed";
            #[cfg(unix)]
            let mut agent = AgentClient::connect_env()
                .await
                .map_err(|_| "ssh_auth_unsupported")?
                .dynamic();
            #[cfg(windows)]
            let mut agent = {
                let path = std::env::var_os("SSH_AUTH_SOCK")
                    .unwrap_or_else(|| r"\\.\pipe\openssh-ssh-agent".into());
                match tokio::time::timeout(
                    Duration::from_secs(2),
                    AgentClient::connect_named_pipe(path),
                )
                .await
                {
                    Ok(Ok(agent)) => agent.dynamic(),
                    _ => AgentClient::connect_pageant()
                        .await
                        .map_err(|_| "ssh_auth_unsupported")?
                        .dynamic(),
                }
            };
            if let Some(binding) = connection.0.agent_session_binding() {
                // 旧 agent 不支持此扩展时仍允许普通密钥；受限密钥由 agent 自身拒绝。
                let _ = agent.bind_session(binding, false).await;
            }
            let keys = agent
                .request_identities()
                .await
                .map_err(|_| "ssh_auth_unsupported")?;
            let hash = connection
                .0
                .best_supported_rsa_hash()
                .await
                .map_err(|_| "ssh_network_error")?
                .flatten();
            for identity in keys {
                let handle = Arc::get_mut(&mut connection.0).ok_or("ssh_auth_unsupported")?;
                let result = match identity {
                    russh::keys::agent::AgentIdentity::PublicKey { key, .. } => {
                        handle
                            .authenticate_publickey_with(&host.username, key, hash, &mut agent)
                            .await
                    }
                    russh::keys::agent::AgentIdentity::Certificate { certificate, .. } => {
                        handle
                            .authenticate_certificate_with(
                                &host.username,
                                certificate,
                                hash,
                                &mut agent,
                            )
                            .await
                    }
                };
                if let Ok(result) = result {
                    if result.success() {
                        return Ok(Arc::new(connection) as Arc<dyn Connection>);
                    }
                    failure = authentication::failure(result);
                }
            }
            Err(failure)
        })
    }
}
struct LiveConnection(Arc<client::Handle<Verifier>>);
impl Drop for LiveConnection {
    fn drop(&mut self) {
        // 认证取消时异步发送断开；正常退出由 Workspace 在 runtime 停止前显式等待。
        if !self.0.is_closed()
            && let Ok(runtime) = tokio::runtime::Handle::try_current()
        {
            let handle = self.0.clone();
            runtime.spawn(async move {
                let _ = tokio::time::timeout(
                    Duration::from_secs(10),
                    handle.disconnect(russh::Disconnect::ByApplication, "", "en"),
                )
                .await;
            });
        }
    }
}
impl Connection for LiveConnection {
    fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
    fn disconnect(&self) -> RunFuture<'_, ()> {
        Box::pin(async {
            self.0
                .disconnect(russh::Disconnect::ByApplication, "", "en")
                .await
                .map_err(|_| "ssh_cleanup_failed")
        })
    }
    fn exec(&self, command: String) -> RunFuture<'_, Output> {
        Box::pin(async move {
            let mut channel = self
                .0
                .channel_open_session()
                .await
                .map_err(|_| "ssh_network_error")?;
            let result = async {
                channel
                    .exec(true, command)
                    .await
                    .map_err(|_| "ssh_network_error")?;
                channel.eof().await.map_err(|_| "ssh_network_error")?;
                let mut output = Output {
                    success: false,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                };
                while let Some(message) = channel.wait().await {
                    let (target, data) = match message {
                        ChannelMsg::Data { data } => (&mut output.stdout, data),
                        ChannelMsg::ExtendedData { data, .. } => (&mut output.stderr, data),
                        ChannelMsg::ExitStatus { exit_status } => {
                            output.success = exit_status == 0;
                            continue;
                        }
                        _ => continue,
                    };
                    if target.len() + data.len() > 2 * 1024 * 1024 {
                        return Err("ssh_file_too_large");
                    }
                    target.extend_from_slice(&data);
                }
                Ok(output)
            }
            .await;
            let _ = channel.close().await;
            result
        })
    }
    fn download(&self, path: RemotePosixPath) -> RunFuture<'_, Option<Vec<u8>>> {
        Box::pin(async move {
            let channel = self
                .0
                .channel_open_session()
                .await
                .map_err(|_| "ssh_network_error")?;
            channel
                .request_subsystem(true, "sftp")
                .await
                .map_err(|_| "ssh_network_error")?;
            let raw = RawSftpSession::new(channel.into_stream());
            raw.init().await.map_err(|_| "ssh_network_error")?;
            read_sftp(&raw, &path).await
        })
    }
    fn upload(
        &self,
        path: RemotePosixPath,
        content: Vec<u8>,
        cancel: CancellationToken,
    ) -> RunFuture<'_, ()> {
        Box::pin(async move {
            let raw = bounded(cancel.clone(), async {
                let channel = self
                    .0
                    .channel_open_session()
                    .await
                    .map_err(|_| "ssh_network_error")?;
                channel
                    .request_subsystem(true, "sftp")
                    .await
                    .map_err(|_| "ssh_network_error")?;
                let raw = RawSftpSession::new(channel.into_stream());
                let version = raw.init().await.map_err(|_| "ssh_network_error")?;
                if !version.extensions.contains_key("posix-rename@openssh.com") {
                    return Err("ssh_write_outcome_unknown");
                }
                Ok(raw)
            })
            .await?;
            let stage = path
                .with_suffix(&format!(".ccr-tmp-{}", uuid::Uuid::new_v4()))
                .map_err(|_| "ssh_invalid_remote_path")?;
            let mut open_handle = None;
            let mut committing = false;
            let result = bounded(cancel, async {
                for parent in path.parent_directories() {
                    let _ = raw
                        .mkdir(
                            parent,
                            FileAttributes {
                                permissions: Some(0o700),
                                ..Default::default()
                            },
                        )
                        .await;
                }
                let handle = raw
                    .open(
                        stage.as_str(),
                        OpenFlags::WRITE | OpenFlags::CREATE | OpenFlags::EXCLUDE,
                        FileAttributes {
                            permissions: Some(0o600),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|_| "ssh_network_error")?;
                open_handle = Some(handle.handle.clone());
                for (index, bytes) in content.chunks(32768).enumerate() {
                    raw.write(&handle.handle, (index * 32768) as u64, bytes.to_vec())
                        .await
                        .map_err(|_| "ssh_network_error")?;
                }
                raw.close(handle.handle)
                    .await
                    .map_err(|_| "ssh_network_error")?;
                open_handle = None;
                committing = true;
                atomic_replace(&raw, stage.as_str(), path.as_str()).await
            })
            .await;
            let result = if committing && result.is_err() {
                Err("ssh_write_outcome_unknown")
            } else {
                result
            };
            if result.is_err() {
                let cleanup = bounded(CancellationToken::new(), async {
                    if let Some(handle) = open_handle {
                        let _ = raw.close(handle).await;
                    }
                    match raw.remove(stage.as_str()).await {
                        Ok(_) => Ok(()),
                        Err(russh_sftp::client::error::Error::Status(status))
                            if status.status_code == StatusCode::NoSuchFile =>
                        {
                            Ok(())
                        }
                        Err(_) => Err("ssh_cleanup_failed"),
                    }
                })
                .await;
                if cleanup.is_err() {
                    return Err("ssh_cleanup_failed");
                }
            }
            result
        })
    }
    fn list(&self, path: RemotePosixPath) -> RunFuture<'_, Vec<String>> {
        Box::pin(async move {
            let raw = self.sftp().await?;
            let handle = match raw.opendir(path.as_str()).await {
                Ok(handle) => handle.handle,
                Err(russh_sftp::client::error::Error::Status(status))
                    if status.status_code == StatusCode::NoSuchFile =>
                {
                    return Ok(Vec::new());
                }
                Err(_) => return Err("ssh_read_failed"),
            };
            let result = async {
                let mut names = Vec::new();
                loop {
                    match raw.readdir(&handle).await {
                        Ok(batch) => {
                            for entry in batch.files {
                                if matches!(entry.filename.as_str(), "." | "..") {
                                    continue;
                                }
                                if entry.attrs.is_symlink()
                                    || (!entry.attrs.is_regular() && !entry.attrs.is_dir())
                                {
                                    return Err("ssh_invalid_remote_path");
                                }
                                names.push(entry.filename);
                                if names.len() > 4096 {
                                    return Err("ssh_file_too_large");
                                }
                            }
                        }
                        Err(russh_sftp::client::error::Error::Status(status))
                            if status.status_code == StatusCode::Eof =>
                        {
                            break;
                        }
                        Err(_) => return Err("ssh_read_failed"),
                    }
                }
                Ok(names)
            }
            .await;
            raw.close(handle).await.map_err(|_| "ssh_cleanup_failed")?;
            result
        })
    }
    fn remove(&self, path: RemotePosixPath) -> RunFuture<'_, ()> {
        Box::pin(async move {
            self.sftp()
                .await?
                .remove(path.as_str())
                .await
                .map(|_| ())
                .map_err(|_| "ssh_write_outcome_unknown")
        })
    }
    fn metadata(&self, path: RemotePosixPath) -> RunFuture<'_, Option<(u64, Option<u64>)>> {
        Box::pin(async move {
            match self.sftp().await?.lstat(path.as_str()).await {
                Ok(attrs) if attrs.attrs.is_regular() => Ok(Some((
                    attrs.attrs.size.unwrap_or_default(),
                    attrs.attrs.mtime.map(|time| u64::from(time) * 1000),
                ))),
                Ok(_) => Err("ssh_invalid_remote_path"),
                Err(russh_sftp::client::error::Error::Status(status))
                    if status.status_code == StatusCode::NoSuchFile =>
                {
                    Ok(None)
                }
                Err(_) => Err("ssh_read_failed"),
            }
        })
    }
}
impl LiveConnection {
    async fn sftp(&self) -> Result<RawSftpSession, Error> {
        let channel = self
            .0
            .channel_open_session()
            .await
            .map_err(|_| "ssh_network_error")?;
        channel
            .request_subsystem(true, "sftp")
            .await
            .map_err(|_| "ssh_network_error")?;
        let raw = RawSftpSession::new(channel.into_stream());
        raw.init().await.map_err(|_| "ssh_network_error")?;
        Ok(raw)
    }
}
async fn read_sftp(raw: &RawSftpSession, path: &RemotePosixPath) -> Result<Option<Vec<u8>>, Error> {
    let handle = match raw
        .open(path.as_str(), OpenFlags::READ, FileAttributes::default())
        .await
    {
        Ok(handle) => handle.handle,
        Err(russh_sftp::client::error::Error::Status(status))
            if status.status_code == StatusCode::NoSuchFile =>
        {
            return Ok(None);
        }
        Err(_) => return Err("ssh_read_failed"),
    };
    let result = async {
        let mut bytes = Vec::new();
        loop {
            match raw.read(&handle, bytes.len() as u64, 32768).await {
                Ok(data) if !data.data.is_empty() => {
                    if bytes.len() + data.data.len() > 2 * 1024 * 1024 {
                        return Err("ssh_file_too_large");
                    }
                    bytes.extend_from_slice(&data.data);
                }
                Ok(_) => return Err("ssh_read_failed"),
                Err(russh_sftp::client::error::Error::Status(status))
                    if status.status_code == StatusCode::Eof =>
                {
                    break;
                }
                Err(_) => return Err("ssh_read_failed"),
            }
        }
        Ok(Some(bytes))
    }
    .await;
    raw.close(handle).await.map_err(|_| "ssh_cleanup_failed")?;
    result
}

async fn atomic_replace(raw: &RawSftpSession, old: &str, new: &str) -> Result<(), Error> {
    match raw
        .extended("posix-rename@openssh.com", rename_payload(old, new))
        .await
    {
        Ok(Packet::Status(status)) if status.status_code == StatusCode::Ok => Ok(()),
        _ => Err("ssh_write_outcome_unknown"),
    }
}

#[cfg(test)]
mod tests;

fn rename_payload(old: &str, new: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for path in [old, new] {
        bytes.extend_from_slice(&(path.len() as u32).to_be_bytes());
        bytes.extend_from_slice(path.as_bytes());
    }
    bytes
}
