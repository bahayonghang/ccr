use super::*;
use ccr_app::{
    environments::io::EnvironmentIo, jobs::process::ProcessDescriptor, runtime::EnvironmentId,
};
use ccr_cli::services::{
    install_service::{BackendFuture, InstallBackend, InstallInvocation, InstallOutput},
    install_types::{AbsentReason, Platform},
};
use tokio_util::sync::CancellationToken;

struct Backend(EnvironmentIo);
fn failure() -> InstallFlowError {
    InstallFlowError::Internal {
        message: "ssh_install_unavailable".into(),
    }
}
impl InstallBackend for Backend {
    fn inspect(
        &self,
        cancel: CancellationToken,
    ) -> BackendFuture<'_, (DetectionResult, HostCapabilities)> {
        Box::pin(async move {
            let tools = self
                .0
                .detect_execution_cli(cancel.clone())
                .await
                .map_err(|_| failure())?;
            let path = |name| {
                tools
                    .iter()
                    .find(|tool| tool.name == name)
                    .and_then(|tool| tool.path.as_ref())
                    .cloned()
            };
            let output = self
                .0
                .probe(
                    &ProcessDescriptor::ccr_version_probe(),
                    "uname",
                    &["-s".into()],
                    cancel.clone(),
                )
                .await
                .map_err(|_| failure())?;
            if !output.success {
                return Err(failure());
            }
            let platform = match std::str::from_utf8(&output.stdout)
                .map_err(|_| failure())?
                .trim()
            {
                "Linux" => Platform::Linux,
                "Darwin" => Platform::Macos,
                _ => return Err(failure()),
            };
            let detection = match path("llmusage") {
                Some(program) => {
                    let output = self
                        .0
                        .probe(
                            &ProcessDescriptor::ccr_version_probe(),
                            &program,
                            &["--version".into()],
                            cancel,
                        )
                        .await
                        .map_err(|_| failure())?;
                    if !output.success {
                        return Err(failure());
                    }
                    // 整行脱敏先于任何版本展示；不把banner或stderr解析成版本。
                    let _safe_stdout = safe_line(&String::from_utf8_lossy(&output.stdout));
                    DetectionResult::Available {
                        path: program.into(),
                        version: None,
                        data_root_warning: None,
                    }
                }
                None => DetectionResult::Absent {
                    reason: AbsentReason::NotOnPath,
                    data_root_warning: None,
                },
            };
            let cargo = path("cargo");
            let brew = path("brew");
            Ok((
                detection,
                HostCapabilities {
                    platform,
                    has_cargo: cargo.is_some(),
                    has_homebrew: brew.is_some(),
                    has_scoop: false,
                    has_winget: false,
                    cargo_path: cargo.map(Into::into),
                    homebrew_path: brew.map(Into::into),
                },
            ))
        })
    }
    fn execute(
        &self,
        invocation: InstallInvocation,
        cancel: CancellationToken,
    ) -> BackendFuture<'_, InstallOutput> {
        Box::pin(async move {
            let program = invocation.program().to_str().ok_or_else(failure)?;
            if !program.starts_with('/') || program.contains('\\') {
                return Err(failure());
            }
            let args = invocation
                .args()
                .iter()
                .map(|arg| (*arg).to_owned())
                .collect::<Vec<_>>();
            let descriptor = ProcessDescriptor::llmusage(program).map_err(|_| failure())?;
            let output = self
                .0
                .execute(&descriptor, program, &args, cancel)
                .await
                .map_err(|_| failure())?;
            Ok(InstallOutput {
                success: output.success,
                stdout: output.stdout,
                stderr: output.stderr,
            })
        })
    }
}
impl InstallRequests {
    pub(super) fn owner(&self, environment: &EnvironmentId) -> Result<Arc<InstallOwner>, Error> {
        if matches!(environment, EnvironmentId::Local(_)) {
            return Ok(self.0.install.clone());
        }
        let io = self
            .0
            .remote_configuration_io(environment)
            .map_err(|_| Error::Unsupported)?;
        let EnvironmentId::Ssh(id) = environment else {
            return Err(Error::Unsupported);
        };
        let mut children = self.0.install.remote.lock().map_err(|_| Error::Operation)?;
        if let Some(owner) = children.get(id)
            && owner
                .io
                .as_ref()
                .is_some_and(|io| io.check_authority().is_ok())
        {
            return Ok(owner.clone());
        }
        let mut owner =
            InstallOwner::with_service(InstallService::for_remote(Arc::new(Backend(io.clone()))));
        owner.io = Some(io);
        let owner = Arc::new(owner);
        children.insert(id.clone(), owner.clone());
        Ok(owner)
    }
}
