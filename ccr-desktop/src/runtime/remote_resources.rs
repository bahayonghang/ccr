//! Captured SSH authority and bounded resource mirrors for native domain adapters.
use super::{ConfigSource, Services};
use ccr_app::{
    environments::io::{EnvironmentIo, Error},
    resources::environment::{Family, ResourceMirror},
    runtime::EnvironmentId,
};
use ccr_cli::application::platform_roots::PlatformRoots;
use std::{collections::BTreeSet, sync::Arc};
use tokio_util::sync::CancellationToken;
pub(super) type Remote = Option<Arc<ResourceMirror>>;
impl Services {
    pub(super) fn resource_io(
        &self,
        environment: &EnvironmentId,
    ) -> Result<Option<EnvironmentIo>, Error> {
        match environment {
            EnvironmentId::Local(_) => Ok(None),
            EnvironmentId::Ssh(_) => self.remote_configuration_io(environment).map(Some),
            _ => Err(Error::Unsupported),
        }
    }
}
pub(super) fn capture(
    io: Result<Option<EnvironmentIo>, Error>,
    family: Family,
    targets: BTreeSet<String>,
    cancel: CancellationToken,
) -> Result<Remote, Error> {
    match io? {
        None => Ok(None),
        Some(io) => tokio::runtime::Handle::current()
            .block_on(ResourceMirror::capture(io, family, targets, cancel))
            .map(Arc::new)
            .map(Some),
    }
}
pub(super) fn project_io(
    io: Result<Option<EnvironmentIo>, Error>,
    project: Option<&std::path::Path>,
    cancel: CancellationToken,
) -> Result<Option<EnvironmentIo>, Error> {
    match (io?, project) {
        (Some(io), Some(path)) => {
            let root = ccr_app::environments::ssh_security::RemotePosixPath::root(path.to_str())
                .map_err(|_| Error::InvalidPath)?;
            tokio::runtime::Handle::current()
                .block_on(io.for_project(root, cancel))
                .map(Some)
        }
        (io, _) => Ok(io),
    }
}
pub(super) fn roots(
    source: &ConfigSource,
    remote: &Remote,
) -> Result<PlatformRoots, super::resources::ResourceError> {
    if let Some(remote) = remote {
        Ok(super::remote_config::roots(&remote.mirror))
    } else {
        super::resources::roots(source)
    }
}
pub(super) fn publish(remote: &Remote, cancel: CancellationToken) -> Result<(), Error> {
    if let Some(remote) = remote {
        tokio::runtime::Handle::current().block_on(remote.publish(cancel))?;
    }
    Ok(())
}
pub fn message(error: Error) -> &'static str {
    match error {
        Error::Cancelled | Error::Ssh("ssh_cancelled") => {
            "Remote operation cancelled. Read the current remote files before retrying. / 远程操作已取消，重试前请重新读取远程文件。"
        }
        Error::Conflict => {
            "Remote source changed. Draft retained; reload before retrying. / 远程来源已变更，草稿保留；请重读后重试。"
        }
        Error::Partial | Error::Ssh("ssh_write_outcome_unknown" | "ssh_cleanup_failed") => {
            "Remote publication is partial or unknown. Read the host files and their preimage backups before retrying. / 远程发布部分完成或结果未知；重试前请检查主机文件与原始备份。"
        }
        Error::UnrepresentablePath | Error::InvalidPath | Error::Ssh("ssh_invalid_remote_path") => {
            "This remote path cannot be edited safely. Linked or non-portable names are retained as unsupported sources. / 此远程路径无法安全编辑；链接或不可表示名称保留为不支持的来源。"
        }
        Error::TooLarge => {
            "Remote resource inventory exceeds the bounded file or byte limit. / 远程资源清单超出文件或字节上限。"
        }
        _ => {
            "Remote resource access failed. Draft retained; reconnect and reload before retrying. / 远程资源访问失败，草稿保留；请重连并重新读取。"
        }
    }
}

#[cfg(test)]
mod tests;
