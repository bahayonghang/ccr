//! Captured environment I/O: local OS paths never represent remote POSIX paths.
//!
//! Resolve before queueing work. Snapshots bind the environment, path and captured
//! authority; reconnecting cannot retarget queued requests. Callers validate their
//! document schema and mask secrets before display. Contents and process output
//! intentionally implement no `Debug` and errors never contain their contents.
//!
//! Writes serialize CAS, backup and replacement per path. Local writes use the
//! guarded writer's cross-process lock, private backup and atomic replacement.
//! SSH writes serialize within this workspace and require atomic SFTP replacement;
//! an unrelated remote writer can still race the version check. Backup must finish
//! before replacement; cancellation cannot roll back a replacement already sent.

use super::{
    EnvironmentPolicy,
    ssh::{IoSession, Workspace},
    ssh_security::{RemotePosixPath, posix_single_quote},
};
use crate::jobs::process::{ProcessDescriptor, ProcessGateway};
use crate::runtime::{EnvironmentId, Platform};
use ccr_core::core::{BackupPolicy, content_version_token};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio_util::sync::CancellationToken;

const MAX_CONTENT: usize = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Unsupported,
    InvalidPath,
    Io,
    TooLarge,
    Conflict,
    Cancelled,
    StaleAuthority,
    Process,
    Busy,
    Partial,
    UnrepresentablePath,
    Ssh(&'static str),
}

/// Directory identity without reading file bodies or following links.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
    Unsupported,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryEntry {
    pub name: String,
    pub kind: EntryKind,
}

/// OS-local roots are provided explicitly, including isolated fixture locks.
#[derive(Clone)]
pub struct LocalRoots {
    home: PathBuf,
    locks: PathBuf,
    platforms: Vec<(Platform, PathBuf)>,
}
impl LocalRoots {
    pub fn new(home: PathBuf, locks: PathBuf) -> Result<Self, Error> {
        if !home.is_absolute() || !locks.is_absolute() {
            return Err(Error::InvalidPath);
        }
        Ok(Self {
            home,
            locks,
            platforms: Vec::new(),
        })
    }

    /// Inject an already captured local platform override (for example CODEX_HOME).
    /// No process environment is read or changed by this module.
    pub fn with_platform(mut self, platform: Platform, root: PathBuf) -> Result<Self, Error> {
        if !root.is_absolute() {
            return Err(Error::InvalidPath);
        }
        self.platforms.retain(|(p, _)| *p != platform);
        self.platforms.push((platform, root));
        Ok(self)
    }
}

/// Path types stay distinct even when a remote path resembles a local OS path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvironmentPath {
    Local(PathBuf),
    Ssh(RemotePosixPath),
}

/// Environment-aware roots for new adapters. The CLI's existing PlatformRoots
/// retains its local domain-service semantics and environment-variable overrides.
#[derive(Clone)]
pub struct PlatformRoots {
    home: EnvironmentPath,
    local_platforms: Vec<(Platform, PathBuf)>,
}
impl PlatformRoots {
    pub fn home(&self) -> &EnvironmentPath {
        &self.home
    }

    pub fn relative(&self, relative: &str) -> Result<EnvironmentPath, Error> {
        validate_relative(relative)?;
        match &self.home {
            EnvironmentPath::Local(home) => Ok(EnvironmentPath::Local(home.join(relative))),
            EnvironmentPath::Ssh(home) => home
                .join_relative(relative)
                .map(EnvironmentPath::Ssh)
                .map_err(|_| Error::InvalidPath),
        }
    }

    pub fn platform(&self, platform: Platform) -> Result<EnvironmentPath, Error> {
        if let Some((_, path)) = self.local_platforms.iter().find(|(p, _)| *p == platform) {
            return Ok(EnvironmentPath::Local(path.clone()));
        }
        self.relative(match platform {
            Platform::Claude => ".claude",
            Platform::Codex => ".codex",
            Platform::Grok => ".grok",
            Platform::Gemini => ".gemini/antigravity-cli",
            Platform::OpenCode => ".config/opencode",
            Platform::Qwen => ".qwen",
            Platform::Droid => ".factory",
        })
    }
}

#[derive(Clone)]
enum Adapter {
    Local(LocalRoots),
    Ssh {
        workspace: Arc<Workspace>,
        authority: IoSession,
    },
}

#[derive(Clone)]
pub struct EnvironmentIo {
    environment: EnvironmentId,
    identity: uuid::Uuid,
    roots: PlatformRoots,
    adapter: Adapter,
}

#[derive(Clone)]
pub struct Snapshot {
    pub content: String,
    pub exists: bool,
    token: String,
    relative: String,
    identity: uuid::Uuid,
}
/// Opaque binary document authority; never format or serialize its contents.
#[derive(Clone)]
pub struct ByteSnapshot {
    pub content: Vec<u8>,
    pub exists: bool,
    token: String,
    relative: String,
    identity: uuid::Uuid,
}
impl ByteSnapshot {
    pub fn version(&self) -> &str {
        &self.token
    }
}
impl Snapshot {
    pub fn version(&self) -> &str {
        &self.token
    }
}

pub struct ProcessOutput {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
pub type OutputObserver = Arc<dyn Fn(bool, &[u8]) + Send + Sync>;

impl EnvironmentIo {
    pub fn environment(&self) -> &EnvironmentId {
        &self.environment
    }
    /// Check the captured connection without acquiring authority from a reconnect.
    pub fn check_authority(&self) -> Result<(), Error> {
        match &self.adapter {
            Adapter::Ssh {
                workspace,
                authority,
            } => workspace.check_io(authority).map_err(Error::Ssh),
            Adapter::Local(_) => Ok(()),
        }
    }
    /// Reuse the fixed CLI detector under this admission's captured connection.
    pub async fn detect_execution_cli(
        &self,
        cancel: CancellationToken,
    ) -> Result<Vec<super::ssh::CliStatus>, Error> {
        match &self.adapter {
            Adapter::Ssh {
                workspace,
                authority,
            } => workspace
                .detect_execution_io(authority, cancel)
                .await
                .map_err(Error::Ssh),
            _ => Err(Error::Unsupported),
        }
    }
    /// Capture a typed POSIX root for read-only source inventories outside HOME.
    pub fn at_remote_root(&self, home: RemotePosixPath) -> Result<Self, Error> {
        match &self.adapter {
            Adapter::Ssh {
                workspace,
                authority,
            } => Ok(Self {
                environment: self.environment.clone(),
                identity: uuid::Uuid::new_v4(),
                roots: PlatformRoots {
                    home: EnvironmentPath::Ssh(home.clone()),
                    local_platforms: Vec::new(),
                },
                adapter: Adapter::Ssh {
                    workspace: workspace.clone(),
                    authority: authority.with_home(home),
                },
            }),
            _ => Err(Error::Unsupported),
        }
    }
    /// Capture an existing, unlinked project directory using metadata only.
    pub async fn for_project(
        &self,
        root: RemotePosixPath,
        cancel: CancellationToken,
    ) -> Result<Self, Error> {
        if !root.as_str().starts_with('/') {
            return Err(Error::InvalidPath);
        }
        let Adapter::Ssh {
            workspace,
            authority,
        } = &self.adapter
        else {
            return Err(Error::Unsupported);
        };
        if !workspace
            .directory_io(authority, root.clone(), cancel)
            .await
            .map_err(Error::Ssh)?
        {
            return Err(Error::InvalidPath);
        }
        self.at_remote_root(root)
    }
    pub async fn metadata(
        &self,
        relative: &str,
        cancel: CancellationToken,
    ) -> Result<Option<(u64, Option<u64>)>, Error> {
        self.roots.relative(relative)?;
        check_cancel(&cancel)?;
        match &self.adapter {
            Adapter::Ssh {
                workspace,
                authority,
            } => workspace
                .metadata_io(authority, relative, cancel)
                .await
                .map_err(Error::Ssh),
            Adapter::Local(_) => Err(Error::Unsupported),
        }
    }
    /// Serialize a remote configuration publication without retaining a read lock
    /// for the lifetime of an editor. Individual files keep their own CAS locks.
    pub async fn configuration_lock(
        &self,
        cancel: CancellationToken,
    ) -> Result<tokio::sync::OwnedMutexGuard<()>, Error> {
        match &self.adapter {
            Adapter::Ssh {
                workspace,
                authority,
            } => workspace
                .configuration_lock(authority, cancel)
                .await
                .map_err(Error::Ssh),
            Adapter::Local(_) => Err(Error::Unsupported),
        }
    }
    /// Captures exactly the requested environment. SSH never evaluates the local
    /// roots closure; a missing or stale connection never falls back to local I/O.
    pub fn resolve(
        environment: &EnvironmentId,
        local: impl FnOnce() -> Result<LocalRoots, Error>,
        ssh: Option<Arc<Workspace>>,
    ) -> Result<Self, Error> {
        EnvironmentPolicy::FollowsEnvironment
            .admit(environment)
            .map_err(|_| Error::Unsupported)?;
        let (home, adapter) = match environment {
            EnvironmentId::Local(_) => {
                let roots = local()?;
                (
                    EnvironmentPath::Local(roots.home.clone()),
                    Adapter::Local(roots),
                )
            }
            EnvironmentId::Ssh(id) => {
                let workspace = ssh.ok_or(Error::Ssh("ssh_not_connected"))?;
                let authority = workspace.capture_io(id).map_err(Error::Ssh)?;
                (
                    EnvironmentPath::Ssh(authority.home()),
                    Adapter::Ssh {
                        workspace,
                        authority,
                    },
                )
            }
            EnvironmentId::Wsl(_) => return Err(Error::Unsupported),
        };
        let local_platforms = match &adapter {
            Adapter::Local(roots) => roots.platforms.clone(),
            Adapter::Ssh { .. } => Vec::new(),
        };
        Ok(Self {
            environment: environment.clone(),
            identity: uuid::Uuid::new_v4(),
            roots: PlatformRoots {
                home,
                local_platforms,
            },
            adapter,
        })
    }

    pub fn roots(&self) -> &PlatformRoots {
        &self.roots
    }

    /// Scope I/O to a platform's captured root. Local overrides may be outside
    /// HOME; SSH always uses the connected host's validated POSIX root.
    pub fn for_platform(&self, platform: Platform) -> Result<Self, Error> {
        let home = self.roots.platform(platform)?;
        let adapter = match (&self.adapter, &home) {
            (Adapter::Local(roots), EnvironmentPath::Local(home)) => {
                Adapter::Local(LocalRoots::new(home.clone(), roots.locks.clone())?)
            }
            (
                Adapter::Ssh {
                    workspace,
                    authority,
                },
                EnvironmentPath::Ssh(home),
            ) => Adapter::Ssh {
                workspace: workspace.clone(),
                authority: authority.with_home(home.clone()),
            },
            _ => return Err(Error::InvalidPath),
        };
        Ok(Self {
            environment: self.environment.clone(),
            identity: uuid::Uuid::new_v4(),
            roots: PlatformRoots {
                home,
                local_platforms: Vec::new(),
            },
            adapter,
        })
    }

    /// Explicit local cache only: deriving a path performs no I/O. Opaque host
    /// IDs are hashed, so traversal or platform separators cannot escape the root.
    pub fn cache_root(&self, root: &Path) -> Result<PathBuf, Error> {
        if !root.is_absolute() {
            return Err(Error::InvalidPath);
        }
        let (kind, id) = match &self.environment {
            EnvironmentId::Local(id) => ("local", id),
            EnvironmentId::Ssh(id) => ("ssh", id),
            EnvironmentId::Wsl(_) => return Err(Error::Unsupported),
        };
        let digest: String = Sha256::digest(id.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Ok(root.join(kind).join(digest))
    }

    /// Local filesystem work runs on a blocking worker and is joined even when
    /// cancellation arrives, so no detached writer outlives its terminal result.
    pub async fn read(&self, relative: &str, cancel: CancellationToken) -> Result<Snapshot, Error> {
        let bytes = self.read_bytes(relative, cancel).await?;
        Ok(Snapshot {
            content: String::from_utf8(bytes.content).map_err(|_| Error::Io)?,
            exists: bytes.exists,
            token: bytes.token,
            relative: bytes.relative,
            identity: bytes.identity,
        })
    }

    pub async fn read_bytes(
        &self,
        relative: &str,
        cancel: CancellationToken,
    ) -> Result<ByteSnapshot, Error> {
        self.roots.relative(relative)?;
        check_cancel(&cancel)?;
        let content = match &self.adapter {
            Adapter::Local(roots) => {
                let path = roots.home.join(relative);
                tokio::task::spawn_blocking(move || {
                    let file = match std::fs::File::open(path) {
                        Ok(file) => file,
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                            return Ok(None);
                        }
                        Err(_) => return Err(Error::Io),
                    };
                    let mut bytes = Vec::new();
                    file.take(MAX_CONTENT as u64 + 1)
                        .read_to_end(&mut bytes)
                        .map_err(|_| Error::Io)?;
                    if bytes.len() > MAX_CONTENT {
                        return Err(Error::TooLarge);
                    }
                    Ok(Some(bytes))
                })
                .await
                .map_err(|_| Error::Io)??
            }
            Adapter::Ssh {
                workspace,
                authority,
            } => workspace
                .read_io(authority, relative, cancel.clone())
                .await
                .map_err(Error::Ssh)?,
        };
        check_cancel(&cancel)?;
        if content.as_ref().is_some_and(|s| s.len() > MAX_CONTENT) {
            return Err(Error::TooLarge);
        }
        Ok(ByteSnapshot {
            token: content
                .as_ref()
                .map(|s| content_version_token(s))
                .unwrap_or_default(),
            exists: content.is_some(),
            content: content.unwrap_or_default(),
            relative: relative.into(),
            identity: self.identity,
        })
    }

    /// Read a session snapshot under the separate 64 MiB session-operation budget.
    /// Configuration reads and all writes retain their 2 MiB safety limit.
    pub async fn read_session_bytes(
        &self,
        relative: &str,
        cancel: CancellationToken,
    ) -> Result<Vec<u8>, Error> {
        self.roots.relative(relative)?;
        let Adapter::Ssh {
            workspace,
            authority,
        } = &self.adapter
        else {
            return Err(Error::Unsupported);
        };
        check_cancel(&cancel)?;
        let before = self
            .metadata(relative, cancel.clone())
            .await?
            .ok_or(Error::Conflict)?;
        if before.0 > 64 * 1024 * 1024 {
            return Err(Error::TooLarge);
        }
        let bytes = workspace
            .read_session_io(authority, relative, cancel.clone())
            .await
            .map_err(Error::Ssh)?
            .ok_or(Error::Conflict)?;
        if bytes.len() > 64 * 1024 * 1024 {
            return Err(Error::TooLarge);
        }
        let after = self.metadata(relative, cancel.clone()).await?;
        check_cancel(&cancel)?;
        if after != Some(before) || bytes.len() as u64 != before.0 {
            return Err(Error::Conflict);
        }
        Ok(bytes)
    }

    /// Schema validation belongs to the calling domain. This operation owns
    /// version conflict detection, backup ordering and private atomic replacement.
    pub async fn write(
        &self,
        expected: &Snapshot,
        content: &str,
        cancel: CancellationToken,
    ) -> Result<Snapshot, Error> {
        let bytes = ByteSnapshot {
            content: expected.content.as_bytes().to_vec(),
            exists: expected.exists,
            token: expected.token.clone(),
            relative: expected.relative.clone(),
            identity: expected.identity,
        };
        self.write_bytes(&bytes, Some(content.as_bytes()), cancel)
            .await?;
        Ok(Snapshot {
            content: content.into(),
            token: content_version_token(content.as_bytes()),
            exists: true,
            ..expected.clone()
        })
    }

    /// Replace or delete a captured binary file with the same CAS and backup policy.
    /// Remote deletion is serialized with replacement and backs up the preimage.
    pub async fn write_bytes(
        &self,
        expected: &ByteSnapshot,
        content: Option<&[u8]>,
        cancel: CancellationToken,
    ) -> Result<(), Error> {
        if expected.identity != self.identity {
            return Err(Error::StaleAuthority);
        }
        check_cancel(&cancel)?;
        if content.is_some_and(|content| content.len() > MAX_CONTENT) {
            return Err(Error::TooLarge);
        }
        match &self.adapter {
            Adapter::Local(roots) => {
                let path = roots.home.join(&expected.relative);
                let locks = roots.locks.clone();
                let token = expected.token.clone();
                let content = content.map(<[u8]>::to_vec);
                let cancel = cancel.clone();
                let result = tokio::task::spawn_blocking(move || {
                    check_cancel(&cancel)?;
                    let content = content.ok_or(Error::Unsupported)?;
                    ccr_core::core::LockManager::with_scoped_directory(&locks, || {
                        ccr_core::core::write_guarded_versioned(
                            &path,
                            &content,
                            &token,
                            &ccr_core::core::WriteOptions {
                                backup: BackupPolicy::SameDir { tag: None },
                                secret: true,
                                ..Default::default()
                            },
                        )
                    })
                    .map_err(|_| Error::Io)
                })
                .await
                .map_err(|_| Error::Io)??;
                if result == ccr_core::core::VersionedWriteOutcome::Conflict {
                    return Err(Error::Conflict);
                }
            }
            Adapter::Ssh {
                workspace,
                authority,
            } => workspace
                .write_io(
                    authority,
                    &expected.relative,
                    &expected.token,
                    content,
                    cancel,
                )
                .await
                .map_err(|e| {
                    if e == "ssh_conflict" {
                        Error::Conflict
                    } else {
                        Error::Ssh(e)
                    }
                })?,
        }
        Ok(())
    }

    /// Enumerate resource children, retaining unsupported links as health records.
    pub async fn entries(
        &self,
        relative: &str,
        cancel: CancellationToken,
    ) -> Result<Option<Vec<DirectoryEntry>>, Error> {
        self.roots.relative(relative)?;
        check_cancel(&cancel)?;
        let Adapter::Ssh {
            workspace,
            authority,
        } = &self.adapter
        else {
            return Err(Error::Unsupported);
        };
        let entries = workspace
            .entries_io(authority, relative, cancel.clone())
            .await
            .map_err(Error::Ssh)?;
        check_cancel(&cancel)?;
        Ok(entries)
    }

    /// List validated child names without interpreting a remote path as an OS path.
    pub async fn list(
        &self,
        relative: &str,
        cancel: CancellationToken,
    ) -> Result<Vec<String>, Error> {
        self.roots.relative(relative)?;
        check_cancel(&cancel)?;
        let names = match &self.adapter {
            Adapter::Local(roots) => {
                let path = roots.home.join(relative);
                tokio::task::spawn_blocking(move || {
                    let entries = match std::fs::read_dir(path) {
                        Ok(entries) => entries,
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                            return Ok(Vec::new());
                        }
                        Err(_) => return Err(Error::Io),
                    };
                    entries
                        .map(|entry| {
                            entry
                                .map_err(|_| Error::Io)?
                                .file_name()
                                .into_string()
                                .map_err(|_| Error::InvalidPath)
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .await
                .map_err(|_| Error::Io)??
            }
            Adapter::Ssh {
                workspace,
                authority,
            } => workspace
                .list_io(authority, relative, cancel.clone())
                .await
                .map_err(Error::Ssh)?,
        };
        for name in &names {
            if name.contains('/') {
                return Err(Error::InvalidPath);
            }
            validate_relative(name)?;
        }
        check_cancel(&cancel)?;
        Ok(names)
    }

    /// The backend domain supplies its existing local process capability and the
    /// matching remote executable. Each remote argument is separately POSIX quoted;
    /// no shell fragments, local executable discovery or implicit SSH reconnect run.
    pub async fn probe(
        &self,
        descriptor: &ProcessDescriptor,
        remote_program: &str,
        args: &[String],
        cancel: CancellationToken,
    ) -> Result<ProcessOutput, Error> {
        if !matches!(
            descriptor.capability().id(),
            "ccr_version_probe" | "cli_probe"
        ) {
            return Err(Error::Unsupported);
        }
        check_cancel(&cancel)?;
        if remote_program.is_empty()
            || std::iter::once(remote_program)
                .chain(args.iter().map(String::as_str))
                .any(|arg| arg.contains('\0'))
        {
            return Err(Error::InvalidPath);
        }
        match &self.adapter {
            Adapter::Ssh {
                workspace,
                authority,
            } => {
                let command = std::iter::once(remote_program)
                    .chain(args.iter().map(String::as_str))
                    .map(posix_single_quote)
                    .collect::<Vec<_>>()
                    .join(" ");
                let (success, stdout, stderr) = workspace
                    .probe_io(
                        authority,
                        command,
                        descriptor.timeout(),
                        descriptor.max_bytes_per_stream(),
                        cancel,
                    )
                    .await
                    .map_err(Error::Ssh)?;
                Ok(ProcessOutput {
                    success,
                    stdout,
                    stderr,
                })
            }
            _ => self.execute(descriptor, remote_program, args, cancel).await,
        }
    }
    pub async fn execute(
        &self,
        descriptor: &ProcessDescriptor,
        remote_program: &str,
        args: &[String],
        cancel: CancellationToken,
    ) -> Result<ProcessOutput, Error> {
        self.execute_observed(descriptor, remote_program, args, cancel, None)
            .await
    }
    /// Observe bounded remote chunks. The consumer buffers complete lines before
    /// redaction; the callback cannot choose commands or change process budgets.
    pub async fn execute_observed(
        &self,
        descriptor: &ProcessDescriptor,
        remote_program: &str,
        args: &[String],
        cancel: CancellationToken,
        observer: Option<OutputObserver>,
    ) -> Result<ProcessOutput, Error> {
        check_cancel(&cancel)?;
        if remote_program.is_empty()
            || std::iter::once(remote_program)
                .chain(args.iter().map(String::as_str))
                .any(|s| s.contains('\0'))
        {
            return Err(Error::InvalidPath);
        }
        match &self.adapter {
            Adapter::Local(_) => {
                let mut command =
                    ProcessGateway::command(descriptor).map_err(|_| Error::Process)?;
                command.args(args);
                let output = ProcessGateway::execute_command_cancellable(
                    command,
                    descriptor,
                    None,
                    cancel.clone(),
                )
                .await
                .map_err(|_| Error::Process)?;
                check_cancel(&cancel)?;
                if output.timed_out || output.stdout_truncated || output.stderr_truncated {
                    return Err(Error::Process);
                }
                Ok(ProcessOutput {
                    success: output.status.success(),
                    stdout: output.stdout,
                    stderr: output.stderr,
                })
            }
            Adapter::Ssh {
                workspace,
                authority,
            } => {
                let command = std::iter::once(remote_program)
                    .chain(args.iter().map(String::as_str))
                    .map(posix_single_quote)
                    .collect::<Vec<_>>()
                    .join(" ");
                let (success, stdout, stderr) = workspace
                    .exec_io(
                        authority,
                        command,
                        descriptor.timeout(),
                        descriptor.max_bytes_per_stream(),
                        cancel,
                        observer,
                    )
                    .await
                    .map_err(Error::Ssh)?;
                Ok(ProcessOutput {
                    success,
                    stdout,
                    stderr,
                })
            }
        }
    }
}

fn validate_relative(relative: &str) -> Result<(), Error> {
    RemotePosixPath::root(Some("/"))
        .and_then(|root| root.join_relative(relative))
        .map(|_| ())
        .map_err(|_| Error::InvalidPath)
}
fn check_cancel(cancel: &CancellationToken) -> Result<(), Error> {
    if cancel.is_cancelled() {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
