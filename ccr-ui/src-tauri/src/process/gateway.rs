//! Tauri process adapter. Shared execution owns capability and cleanup policy.
pub use ccr_app::jobs::process::{CappedProcessOutput, OwnedProcessRecord, ProcessDescriptor};
use ccr_app::jobs::process::{ProcessGateway as SharedGateway, SidecarContext};
use std::{ffi::OsString, path::Path};
use tokio::process::Command;

pub(crate) fn sidecar_context() -> SidecarContext {
    SidecarContext {
        current_exe: std::env::current_exe().ok(),
        repository_root: Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        allow_development: cfg!(debug_assertions),
        expected_sha256: option_env!("CCR_SIDECAR_SHA256").map(str::to_owned),
        expected_version: env!("CARGO_PKG_VERSION").into(),
    }
}

pub struct ProcessGateway;
impl ProcessGateway {
    pub fn command(descriptor: &ProcessDescriptor) -> Result<Command, String> {
        SharedGateway::command_with_context(descriptor, &sidecar_context())
    }
    pub async fn execute(
        descriptor: &ProcessDescriptor,
        args: &[OsString],
    ) -> Result<CappedProcessOutput, String> {
        SharedGateway::execute_with_context(descriptor, &sidecar_context(), args).await
    }
    pub fn open_oauth_url(raw_url: &str) -> Result<(), String> {
        SharedGateway::open_oauth_url(raw_url)
    }
    pub fn owned_processes_for_port(pids: &[u32], port: u16) -> Vec<OwnedProcessRecord> {
        SharedGateway::owned_processes_for_port(pids, port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sidecar_identity_is_captured_from_tauri_consumer() {
        let context = sidecar_context();
        assert_eq!(
            context.repository_root.canonicalize().expect("repository"),
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .expect("repository")
        );
        assert_eq!(context.expected_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(
            context.expected_sha256.as_deref(),
            option_env!("CCR_SIDECAR_SHA256")
        );
        assert_eq!(context.allow_development, cfg!(debug_assertions));
    }
}
