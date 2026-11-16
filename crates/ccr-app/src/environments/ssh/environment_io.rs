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
    pub(crate) fn capture_io(&self, id: &str) -> Result<IoSession, Error> {
        let session = self.current(id)?;
        let home = RemotePosixPath::root(session.host.remote_home.as_deref())
            .map_err(|_| "ssh_invalid_remote_path")?;
        Ok(IoSession(session, home))
    }

    fn check_io(&self, authority: &IoSession) -> Result<(), Error> {
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
    ) -> Result<Option<String>, Error> {
        self.check_io(authority)?;
        let path = authority
            .home()
            .join_relative(relative)
            .map_err(|_| "ssh_invalid_remote_path")?;
        let result = bounded(cancel, authority.0.connection.download(path)).await;
        match result {
            Ok(content) => {
                self.check_io(authority)?;
                Ok(content)
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
        content: &str,
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
                .map(|s| content_version_token(s.as_bytes()))
                .unwrap_or_default();
            if token != expected {
                return Err("ssh_conflict");
            }
            if let Some(previous) = previous {
                let backup = path
                    .with_suffix(&format!(".bak.{}", uuid::Uuid::new_v4()))
                    .map_err(|_| "ssh_invalid_remote_path")?;
                authority
                    .0
                    .connection
                    .upload(backup, previous, cancel.clone())
                    .await?;
            }
            self.check_io(authority)?;
            authority
                .0
                .connection
                .upload(path, content.into(), cancel)
                .await?;
            self.check_io(authority)
        }
        .await;
        if result.as_ref().is_err_and(|e| *e != "ssh_conflict") {
            self.revoke(authority.0.session_id).await?;
        }
        result
    }

    pub(crate) async fn exec_io(
        &self,
        authority: &IoSession,
        command: String,
        timeout: std::time::Duration,
        cancel: CancellationToken,
    ) -> Result<(bool, Vec<u8>, Vec<u8>), Error> {
        self.check_io(authority)?;
        let result = tokio::select! {
            biased;
            _ = cancel.cancelled() => Err("ssh_cancelled"),
            result = tokio::time::timeout(timeout, authority.0.connection.exec(command)) => result.unwrap_or(Err("ssh_network_error")),
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
}
