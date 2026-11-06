//! Closed local tool probes and manual Cargo update shared by both desktop adapters.
pub mod bridge;
use crate::jobs::process::{CappedProcessOutput, ProcessDescriptor, ProcessGateway};
use std::{
    ffi::OsString,
    path::Path,
    time::{Duration, Instant},
};

pub const VERSION_TOOLS: [(&str, &str); 5] = [
    ("ccr", "ccr"),
    ("claude", "claude"),
    ("codex", "codex"),
    ("gemini", "agy"),
    ("grok", "grok"),
];

#[derive(Clone, Debug)]
pub struct CliVersion {
    pub platform: String,
    pub installed: bool,
    pub version: Option<String>,
    pub status: &'static str,
    pub elapsed_ms: u64,
}

/// The adapter supplies a detected CLI path; only the closed tool set gets a descriptor.
pub async fn probe_version(
    tool: &'static str,
    program: impl AsRef<Path>,
    timeout_ms: u64,
) -> CliVersion {
    let started = Instant::now();
    let mut entry = CliVersion {
        platform: tool.into(),
        installed: false,
        version: None,
        status: "error",
        elapsed_ms: 0,
    };
    let Ok(descriptor) =
        ProcessDescriptor::cli_probe(tool, program.as_ref(), Duration::from_millis(timeout_ms))
    else {
        entry.status = "not_installed";
        return entry;
    };
    match ProcessGateway::execute(&descriptor, &[OsString::from("--version")]).await {
        Ok(output) if output.timed_out => entry.status = "timeout",
        Ok(output) if !output.stdout_truncated && !output.stderr_truncated => {
            let line = first_version_line(&output);
            if output.status.success() && line.is_some() {
                entry.installed = true;
                entry.status = "ok";
                entry.version = line;
            }
        }
        _ => {}
    }
    entry.elapsed_ms = started.elapsed().as_millis() as u64;
    entry
}
fn first_version_line(output: &CappedProcessOutput) -> Option<String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    stdout
        .lines()
        .find(|line| !line.trim().is_empty())
        .or_else(|| stderr.lines().find(|line| !line.trim().is_empty()))
        .map(|line| line.trim().into())
}

#[derive(Clone, Debug)]
pub struct UpdateReport {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub truncated: bool,
}
impl From<CappedProcessOutput> for UpdateReport {
    fn from(output: CappedProcessOutput) -> Self {
        let truncated = output.stdout_truncated || output.stderr_truncated;
        Self {
            success: output.status.success() && !output.timed_out && !truncated,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            timed_out: output.timed_out,
            truncated,
        }
    }
}
/// Manual user-Cargo installation. This does not replace a packaged desktop sidecar.
pub async fn update_ccr_for_root(root: &Path) -> Result<UpdateReport, String> {
    if !root.is_absolute() {
        return Err("install_root_must_be_absolute".into());
    }
    ProcessGateway::execute(
        &ProcessDescriptor::ccr_update(),
        &[
            OsString::from("install"),
            OsString::from("ccr"),
            OsString::from("--root"),
            root.as_os_str().into(),
        ],
    )
    .await
    .map(Into::into)
}
pub async fn update_ccr() -> Result<UpdateReport, String> {
    ProcessGateway::execute(
        &ProcessDescriptor::ccr_update(),
        &[OsString::from("install"), OsString::from("ccr")],
    )
    .await
    .map(Into::into)
}

#[cfg(test)]
mod tests;
