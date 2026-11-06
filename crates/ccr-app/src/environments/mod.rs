//! Local process, WSL probe, and saved SSH host labels for desktop shells.
mod ssh_session;
mod ssh_trust;
mod surface;
mod wsl;

use std::path::PathBuf;

pub use ssh_session::{
    AUTH_UNSUPPORTED, CANCELLED_BEFORE_CONNECT, ConnectFacts, ConnectKind, DISCONNECT_NOTHING,
    DisconnectReport, HOST_KEY_MISMATCH, ProcessHostTrust, RECONNECT_UNTRUSTED, ReconnectGate,
    allow_reconnect, allow_reconnect_target, boundary_before_connect, disconnect_error,
    password_only, plan_disconnect, reconnect_gate,
};
pub use ssh_trust::{HostKeyDecision, HostKeyStatus, classify_host_key, decide_host_key};
pub use surface::{
    EnvironmentSurface, LOCAL_ONLY, SurfaceSlot, admit_local_only, read_local_source,
};
pub use wsl::{
    WslDistro, WslExecProbe, WslFileIntent, WslFileTarget, WslListError, WslProbe, WslState,
    WslVersion, classify_exec, decode_wsl_output, list_installed_distros, parse_wsl_list,
    plan_wsl_file, probe_wsl, resolve_wsl_file,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunningEnvironment {
    /// This desktop process is the local execution environment.
    Local,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityState {
    Unsupported,
    NotRun,
    NotConnected,
    Available,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellCapabilities {
    pub tray: CapabilityState,
    pub hide_restore: CapabilityState,
    pub notifications: CapabilityState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SshNames {
    NotConnected,
    Unreadable,
    Empty,
    Names(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentReport {
    pub running: RunningEnvironment,
    pub os: &'static str,
    pub wsl: WslProbe,
    pub ssh: SshNames,
    pub shell: ShellCapabilities,
}

pub struct InspectOptions {
    pub query_wsl: bool,
    pub ssh_database: Option<PathBuf>,
}

impl InspectOptions {
    /// Demo and fixture shells do not query WSL or the user database.
    pub fn fixture() -> Self {
        Self {
            query_wsl: false,
            ssh_database: None,
        }
    }

    /// The running desktop may list WSL and saved host names. It does not connect.
    pub fn live() -> Self {
        Self {
            query_wsl: cfg!(windows),
            ssh_database: ccr_db::database::existing_ui_database_file(),
        }
    }
}

pub fn running_environment() -> RunningEnvironment {
    RunningEnvironment::Local
}

pub fn host_os_name() -> &'static str {
    match std::env::consts::OS {
        "windows" => "Windows",
        "macos" => "macOS",
        "linux" => "Linux",
        other => other,
    }
}

/// gpui-kit 0.7.3 re-exports gpui-pre 0.3.9.
/// That stack has no tray type. Windows `Platform::hide` is empty and has no restore.
/// System notifications exist on the app context; delivery still has to be confirmed by the OS.
pub fn shell_capabilities() -> ShellCapabilities {
    ShellCapabilities {
        tray: CapabilityState::Unsupported,
        hide_restore: if cfg!(windows) {
            CapabilityState::Unsupported
        } else {
            CapabilityState::NotRun
        },
        notifications: CapabilityState::Available,
    }
}

pub fn inspect(options: InspectOptions, is_cancelled: &impl Fn() -> bool) -> EnvironmentReport {
    let wsl = if options.query_wsl {
        probe_wsl(is_cancelled)
    } else if cfg!(windows) {
        WslProbe::NotQueried
    } else {
        WslProbe::Unsupported
    };
    EnvironmentReport {
        running: running_environment(),
        os: host_os_name(),
        wsl,
        ssh: ssh_names(options.ssh_database.as_deref()),
        shell: shell_capabilities(),
    }
}

fn ssh_names(path: Option<&std::path::Path>) -> SshNames {
    let Some(path) = path else {
        return SshNames::NotConnected;
    };
    match ccr_db::database::repositories::ssh_repo::read_saved_host_names(path) {
        ccr_db::database::repositories::ssh_repo::SavedHostNames::Unavailable => {
            SshNames::Unreadable
        }
        ccr_db::database::repositories::ssh_repo::SavedHostNames::Names(names)
            if names.is_empty() =>
        {
            SshNames::Empty
        }
        ccr_db::database::repositories::ssh_repo::SavedHostNames::Names(names) => {
            SshNames::Names(names)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CapabilityState, InspectOptions, RunningEnvironment, SshNames, WslProbe, inspect,
        shell_capabilities,
    };

    #[test]
    #[ignore = "queries the local wsl.exe and does not open a network connection"]
    fn live_wsl_probe_prints_classified_result() {
        let probe = super::probe_wsl(&|| false);
        eprintln!("wsl-probe={probe:?}");
    }

    #[test]
    fn fixture_report_is_the_local_process_and_does_not_invent_a_distro() {
        let report = inspect(InspectOptions::fixture(), &|| false);
        assert_eq!(report.running, RunningEnvironment::Local);
        assert_eq!(report.os, super::host_os_name());
        assert!(matches!(
            report.wsl,
            WslProbe::NotQueried | WslProbe::Unsupported
        ));
        assert_eq!(report.ssh, SshNames::NotConnected);
        assert_eq!(report.shell.tray, CapabilityState::Unsupported);
        if cfg!(windows) {
            assert_eq!(report.wsl, WslProbe::NotQueried);
            assert_eq!(report.shell.hide_restore, CapabilityState::Unsupported);
        } else {
            assert_eq!(report.wsl, WslProbe::Unsupported);
            assert_eq!(report.shell.hide_restore, CapabilityState::NotRun);
        }
        assert_eq!(
            shell_capabilities().notifications,
            CapabilityState::Available
        );
    }

    #[test]
    fn unreadable_ssh_database_is_not_reported_as_a_name_list() {
        let dir = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
        let path = dir.path().join("not-a-database");
        std::fs::write(&path, b"not sqlite").unwrap_or_else(|error| panic!("{error}"));
        let report = inspect(
            InspectOptions {
                query_wsl: false,
                ssh_database: Some(path),
            },
            &|| false,
        );
        assert_eq!(report.ssh, SshNames::Unreadable);
    }
}
