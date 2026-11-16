//! Session authority for the environment I/O seam; never reconnects by host ID.
use super::*;

#[derive(Clone)]
pub(crate) struct IoSession(Session, RemotePosixPath);

impl IoSession {
    pub(crate) fn home(&self) -> RemotePosixPath {
        self.1.clone()
    }
    pub(crate) fn with_home(&self, home: RemotePosixPath) -> Self {
        Self(self.0.clone(), home)
    }
}

impl Workspace {
    pub(crate) async fn detect_execution_io(
        &self,
        authority: &IoSession,
        cancel: CancellationToken,
    ) -> Result<Vec<detect::CliStatus>, Error> {
        self.check_io(authority)?;
        self.detect_execution_cli(
            &detect::CliSession::from_io(&authority.0.id, authority.0.session_id),
            cancel,
        )
        .await
    }
    pub(crate) async fn configuration_lock(
        &self,
        authority: &IoSession,
        cancel: CancellationToken,
    ) -> Result<tokio::sync::OwnedMutexGuard<()>, Error> {
        self.check_io(authority)?;
        let lock = self
            .writes
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .entry(format!("{}:configuration-batch", authority.0.id))
            .or_default()
            .clone();
        let guard = bounded(cancel, async { Ok(lock.lock_owned().await) }).await?;
        self.check_io(authority)?;
        Ok(guard)
    }
    pub(crate) fn capture_io(&self, id: &str) -> Result<IoSession, Error> {
        let session = self.current(id)?;
        let home = RemotePosixPath::root(session.host.remote_home.as_deref())
            .map_err(|_| "ssh_invalid_remote_path")?;
        Ok(IoSession(session, home))
    }

    pub(crate) fn check_io(&self, authority: &IoSession) -> Result<(), Error> {
        if self.current(&authority.0.id)?.session_id != authority.0.session_id {
            return Err("ssh_session_changed");
        }
        Ok(())
    }

    pub(crate) async fn read_io(
        &self,
        authority: &IoSession,
        relative: &str,
        cancel: CancellationToken,
    ) -> Result<Option<Vec<u8>>, Error> {
        self.check_io(authority)?;
        let path = authority
            .home()
            .join_relative(relative)
            .map_err(|_| "ssh_invalid_remote_path")?;
        let result = transport::read_bounded(cancel, authority.0.connection.download(path)).await;
        match result {
            Ok(content) => {
                self.check_io(authority)?;
                Ok(content)
            }
            Err("ssh_cancelled") => Err("ssh_cancelled"),
            Err(error) => {
                self.revoke(authority.0.session_id).await?;
                Err(error)
            }
        }
    }

    pub(crate) async fn read_session_io(
        &self,
        authority: &IoSession,
        relative: &str,
        cancel: CancellationToken,
    ) -> Result<Option<Vec<u8>>, Error> {
        self.check_io(authority)?;
        let path = authority
            .home()
            .join_relative(relative)
            .map_err(|_| "ssh_invalid_remote_path")?;
        let result =
            transport::read_bounded(cancel, authority.0.connection.download_session(path)).await;
        match result {
            Ok(content) => {
                self.check_io(authority)?;
                Ok(content)
            }
            Err(error @ ("ssh_cancelled" | "ssh_file_too_large" | "ssh_invalid_remote_path")) => {
                Err(error)
            }
            Err(error) => {
                self.revoke(authority.0.session_id).await?;
                Err(error)
            }
        }
    }

    pub(crate) async fn write_io(
        &self,
        authority: &IoSession,
        relative: &str,
        expected: &str,
        content: Option<&[u8]>,
        cancel: CancellationToken,
    ) -> Result<(), Error> {
        self.check_io(authority)?;
        let path = authority
            .home()
            .join_relative(relative)
            .map_err(|_| "ssh_invalid_remote_path")?;
        let lock = self
            .writes
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .entry(format!("{}:{}", authority.0.id, path.as_str()))
            .or_default()
            .clone();
        let _guard = bounded(cancel.clone(), async { Ok(lock.lock().await) }).await?;
        self.check_io(authority)?;
        let result = async {
            let previous = bounded(
                cancel.clone(),
                authority.0.connection.download(path.clone()),
            )
            .await?;
            let token = previous
                .as_ref()
                .map(|s| content_version_token(s))
                .unwrap_or_default();
            if token != expected {
                return Err("ssh_conflict");
            }
            if let Some(previous) = &previous {
                let backup = path
                    .with_suffix(&format!(".bak.{}", uuid::Uuid::new_v4()))
                    .map_err(|_| "ssh_invalid_remote_path")?;
                authority
                    .0
                    .connection
                    .upload(backup, previous.clone(), cancel.clone())
                    .await?;
            }
            self.check_io(authority)?;
            match content {
                Some(content) => {
                    authority
                        .0
                        .connection
                        .upload(path, content.into(), cancel)
                        .await?
                }
                None if previous.is_some() => {
                    bounded(cancel, authority.0.connection.remove(path)).await?;
                }
                None => {}
            }
            self.check_io(authority)
        }
        .await;
        if result.as_ref().is_err_and(|e| *e != "ssh_conflict") {
            self.revoke(authority.0.session_id).await?;
        }
        result
    }

    pub(crate) async fn directory_io(
        &self,
        authority: &IoSession,
        root: RemotePosixPath,
        cancel: CancellationToken,
    ) -> Result<bool, Error> {
        self.check_io(authority)?;
        let exists = bounded(cancel, authority.0.connection.is_directory(root)).await?;
        self.check_io(authority)?;
        Ok(exists)
    }
    pub(crate) async fn entries_io(
        &self,
        authority: &IoSession,
        relative: &str,
        cancel: CancellationToken,
    ) -> Result<Option<Vec<super::super::io::DirectoryEntry>>, Error> {
        self.check_io(authority)?;
        let path = authority
            .home()
            .join_relative(relative)
            .map_err(|_| "ssh_invalid_remote_path")?;
        let result = transport::read_bounded(cancel, authority.0.connection.entries(path)).await;
        match result {
            Ok(entries) => {
                self.check_io(authority)?;
                Ok(entries)
            }
            Err(error @ ("ssh_cancelled" | "ssh_invalid_remote_path")) => Err(error),
            Err(error) => {
                self.revoke(authority.0.session_id).await?;
                Err(error)
            }
        }
    }
    pub(crate) async fn list_io(
        &self,
        authority: &IoSession,
        relative: &str,
        cancel: CancellationToken,
    ) -> Result<Vec<String>, Error> {
        self.check_io(authority)?;
        let path = authority
            .home()
            .join_relative(relative)
            .map_err(|_| "ssh_invalid_remote_path")?;
        let result = transport::read_bounded(cancel, authority.0.connection.list(path)).await;
        match result {
            Ok(names) => {
                self.check_io(authority)?;
                Ok(names)
            }
            Err("ssh_cancelled") => Err("ssh_cancelled"),
            Err(error) => {
                self.revoke(authority.0.session_id).await?;
                Err(error)
            }
        }
    }
    pub(crate) async fn metadata_io(
        &self,
        authority: &IoSession,
        relative: &str,
        cancel: CancellationToken,
    ) -> Result<Option<(u64, Option<u64>)>, Error> {
        self.check_io(authority)?;
        let path = authority
            .home()
            .join_relative(relative)
            .map_err(|_| "ssh_invalid_remote_path")?;
        let result = bounded(cancel, authority.0.connection.metadata(path)).await?;
        self.check_io(authority)?;
        Ok(result)
    }

    pub(crate) async fn exec_io(
        &self,
        authority: &IoSession,
        command: String,
        timeout: std::time::Duration,
        max_bytes: usize,
        cancel: CancellationToken,
        observer: Option<super::super::io::OutputObserver>,
    ) -> Result<(bool, Vec<u8>, Vec<u8>), Error> {
        self.check_io(authority)?;
        let result = tokio::select! {
            biased;
            _ = cancel.cancelled() => Err("ssh_cancelled"),
            result = tokio::time::timeout(timeout, authority.0.connection.exec_bounded(command, max_bytes, observer)) => result.unwrap_or(Err("ssh_network_error")),
        };
        if result.is_err() {
            self.revoke(authority.0.session_id).await?;
        }
        match result {
            Ok(output) => {
                self.check_io(authority)?;
                Ok((output.success, output.stdout, output.stderr))
            }
            Err(error) => Err(error),
        }
    }
    pub(crate) async fn probe_io(
        &self,
        authority: &IoSession,
        command: String,
        timeout: std::time::Duration,
        max_bytes: usize,
        cancel: CancellationToken,
    ) -> Result<(bool, Vec<u8>, Vec<u8>), Error> {
        self.check_io(authority)?;
        let result = transport::read_with_timeout(
            cancel,
            authority
                .0
                .connection
                .exec_bounded(command, max_bytes, None),
            timeout,
        )
        .await;
        match result {
            Ok(output) => {
                self.check_io(authority)?;
                Ok((output.success, output.stdout, output.stderr))
            }
            Err("ssh_cancelled") => Err("ssh_cancelled"),
            Err(error) => {
                self.revoke(authority.0.session_id).await?;
                Err(error)
            }
        }
    }
}
