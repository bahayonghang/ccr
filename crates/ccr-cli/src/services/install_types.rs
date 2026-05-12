//! Typed data models for the llmusage install flow.
//!
//! All types are `Serialize`/`Deserialize` for Tauri IPC and JSON round-trip.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────────────────────
// AttemptId
// ──────────────────────────────────────────────────────────────────────────────

/// Unique identifier for a single install attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttemptId(pub uuid::Uuid);

impl AttemptId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for AttemptId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AttemptId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Platform & PackageManager
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Macos,
    Linux,
    Windows,
}

impl Platform {
    /// Detect the current host platform.
    pub fn current() -> Self {
        match std::env::consts::OS {
            "macos" => Self::Macos,
            "linux" => Self::Linux,
            "windows" => Self::Windows,
            _ => Self::Linux, // fallback
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Macos => write!(f, "macOS"),
            Self::Linux => write!(f, "Linux"),
            Self::Windows => write!(f, "Windows"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Cargo,
    Homebrew,
    Scoop,
    Winget,
}

impl std::fmt::Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cargo => write!(f, "cargo"),
            Self::Homebrew => write!(f, "homebrew"),
            Self::Scoop => write!(f, "scoop"),
            Self::Winget => write!(f, "winget"),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// HostCapabilities
// ──────────────────────────────────────────────────────────────────────────────

/// Describes what package managers are available on the host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostCapabilities {
    pub platform: Platform,
    pub has_cargo: bool,
    pub has_homebrew: bool,
    pub has_scoop: bool,
    pub has_winget: bool,
}

// ──────────────────────────────────────────────────────────────────────────────
// DetectionResult
// ──────────────────────────────────────────────────────────────────────────────

/// Result of probing for the `llmusage` binary on the host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum DetectionResult {
    #[serde(rename = "available")]
    Available {
        path: PathBuf,
        version: Option<String>,
        data_root_warning: Option<DataRootWarning>,
    },
    #[serde(rename = "absent")]
    Absent {
        reason: AbsentReason,
        data_root_warning: Option<DataRootWarning>,
    },
}

impl DetectionResult {
    /// Returns `true` if llmusage is available.
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AbsentReason {
    #[serde(rename = "not_on_path")]
    NotOnPath,
    #[serde(rename = "not_executable")]
    NotExecutable {
        exit_code: Option<i32>,
        stderr_excerpt: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum DataRootWarning {
    #[serde(rename = "data_root_missing")]
    DataRootMissing { path: PathBuf },
}

// ──────────────────────────────────────────────────────────────────────────────
// InstallPlan / PlanOutcome
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PlanOutcome {
    #[serde(rename = "plan")]
    Plan(InstallPlan),
    #[serde(rename = "unsupported")]
    Unsupported { reason: UnsupportedReason },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnsupportedReason {
    NoPackageManager,
    ElevationRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallPlan {
    pub platform: Platform,
    pub package_manager: PackageManager,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub envs: BTreeMap<String, String>,
    pub elevation_required: bool,
    pub duration_class: DurationClass,
    pub plan_id: uuid::Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DurationClass {
    Fast,
    Medium,
    Slow,
}

// ──────────────────────────────────────────────────────────────────────────────
// InstallEvent
// ──────────────────────────────────────────────────────────────────────────────

/// Events emitted during an install attempt. Delivered in total order per attempt.
/// `Started` precedes all others; exactly one terminal event is emitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InstallEvent {
    #[serde(rename = "started")]
    Started {
        attempt_id: AttemptId,
        plan: InstallPlan,
    },
    #[serde(rename = "log")]
    Log {
        attempt_id: AttemptId,
        stream: LogStream,
        line: String,
        seq: u64,
    },
    #[serde(rename = "progress")]
    Progress {
        attempt_id: AttemptId,
        stage: ProgressStage,
        detail: Option<String>,
    },
    #[serde(rename = "succeeded")]
    Succeeded {
        attempt_id: AttemptId,
        duration_ms: u64,
        installed_version: Option<String>,
    },
    #[serde(rename = "failed")]
    Failed {
        attempt_id: AttemptId,
        failure_kind: FailureKind,
        exit_code: Option<i32>,
        stderr_excerpt: Option<String>,
        error_message: String,
    },
    #[serde(rename = "cancelled")]
    Cancelled {
        attempt_id: AttemptId,
        requested_at_ms: u64,
    },
}

impl InstallEvent {
    /// Returns `true` if this is a terminal event (Succeeded, Failed, Cancelled).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Succeeded { .. } | Self::Failed { .. } | Self::Cancelled { .. }
        )
    }

    pub fn attempt_id(&self) -> AttemptId {
        match self {
            Self::Started { attempt_id, .. }
            | Self::Log { attempt_id, .. }
            | Self::Progress { attempt_id, .. }
            | Self::Succeeded { attempt_id, .. }
            | Self::Failed { attempt_id, .. }
            | Self::Cancelled { attempt_id, .. } => *attempt_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStage {
    Resolving,
    Downloading,
    Compiling,
    Installing,
    Finalizing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    SpawnFailed,
    NonZeroExit,
    InternalError,
}

// ──────────────────────────────────────────────────────────────────────────────
// CancelResult
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CancelResult {
    #[serde(rename = "cancelled")]
    Cancelled {
        attempt_id: AttemptId,
        requested_at_ms: u64,
    },
    #[serde(rename = "not_running")]
    NotRunning,
    #[serde(rename = "already_terminal")]
    AlreadyTerminal { attempt_id: AttemptId },
}

// ──────────────────────────────────────────────────────────────────────────────
// RingBufferSnapshot
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RingBufferSnapshot {
    pub attempt_id: Option<AttemptId>,
    pub logs: Vec<InstallEvent>,
    pub terminal: Option<InstallEvent>,
}

// ──────────────────────────────────────────────────────────────────────────────
// ManualCatalog
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualCommand {
    pub platform: Platform,
    pub package_manager: Option<PackageManager>,
    pub title: String,
    pub command_line: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualCatalog {
    pub entries: Vec<ManualCommand>,
    pub docs_url: String,
}

// ──────────────────────────────────────────────────────────────────────────────
// InstallFlowError
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(tag = "kind")]
pub enum InstallFlowError {
    #[error("install attempt already running")]
    #[serde(rename = "already_running")]
    AlreadyRunning,

    #[error("invalid payload on field `{field}`: {reason}")]
    #[serde(rename = "invalid_payload")]
    InvalidPayload { field: String, reason: String },

    #[error("plan rejected: elevation constructs are not allowed ({token})")]
    #[serde(rename = "elevation_rejected")]
    ElevationRejected { token: String },

    #[error("manual install catalog incomplete: missing {missing_platform}")]
    #[serde(rename = "manual_catalog_unavailable")]
    ManualCatalogUnavailable { missing_platform: Platform },

    #[error("unable to send cancel signal: {source_message}")]
    #[serde(rename = "cancel_send_failed")]
    CancelSendFailed { source_message: String },

    #[error("post-install detection still absent")]
    #[serde(rename = "post_install_still_absent")]
    PostInstallStillAbsent { hint: PostInstallHint },

    #[error("internal error: {message}")]
    #[serde(rename = "internal")]
    Internal { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostInstallHint {
    ReopenAppForPath,
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: serialize then deserialize, assert round-trip equality.
    fn round_trip<T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug>(
        value: &T,
    ) {
        let json = serde_json::to_string(value).expect("serialize");
        let back: T = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*value, back, "round-trip failed for: {json}");
    }

    #[test]
    fn detection_result_available_round_trip() {
        let val = DetectionResult::Available {
            path: PathBuf::from("/usr/local/bin/llmusage"),
            version: Some("0.5.3".to_string()),
            data_root_warning: None,
        };
        round_trip(&val);
    }

    #[test]
    fn detection_result_absent_round_trip() {
        let val = DetectionResult::Absent {
            reason: AbsentReason::NotOnPath,
            data_root_warning: Some(DataRootWarning::DataRootMissing {
                path: PathBuf::from("/home/user/.llmusage"),
            }),
        };
        round_trip(&val);
    }

    #[test]
    fn install_plan_round_trip() {
        let plan = InstallPlan {
            platform: Platform::Macos,
            package_manager: PackageManager::Homebrew,
            command: "brew".to_string(),
            args: vec!["install".to_string(), "llmusage".to_string()],
            envs: BTreeMap::new(),
            elevation_required: false,
            duration_class: DurationClass::Medium,
            plan_id: uuid::Uuid::new_v4(),
        };
        round_trip(&plan);
    }

    #[test]
    fn install_event_started_round_trip() {
        let ev = InstallEvent::Started {
            attempt_id: AttemptId::new(),
            plan: InstallPlan {
                platform: Platform::Linux,
                package_manager: PackageManager::Cargo,
                command: "cargo".to_string(),
                args: vec![
                    "install".to_string(),
                    "--locked".to_string(),
                    "llmusage".to_string(),
                ],
                envs: BTreeMap::new(),
                elevation_required: false,
                duration_class: DurationClass::Slow,
                plan_id: uuid::Uuid::new_v4(),
            },
        };
        round_trip(&ev);
    }

    #[test]
    fn install_event_log_round_trip() {
        let ev = InstallEvent::Log {
            attempt_id: AttemptId::new(),
            stream: LogStream::Stderr,
            line: "Compiling llmusage v0.5.3".to_string(),
            seq: 42,
        };
        round_trip(&ev);
    }

    #[test]
    fn install_event_succeeded_round_trip() {
        let ev = InstallEvent::Succeeded {
            attempt_id: AttemptId::new(),
            duration_ms: 12345,
            installed_version: Some("0.5.3".to_string()),
        };
        round_trip(&ev);
    }

    #[test]
    fn install_event_failed_round_trip() {
        let ev = InstallEvent::Failed {
            attempt_id: AttemptId::new(),
            failure_kind: FailureKind::NonZeroExit,
            exit_code: Some(101),
            stderr_excerpt: Some("error[E0433]: failed to resolve".to_string()),
            error_message: "cargo install failed".to_string(),
        };
        round_trip(&ev);
    }

    #[test]
    fn install_event_cancelled_round_trip() {
        let ev = InstallEvent::Cancelled {
            attempt_id: AttemptId::new(),
            requested_at_ms: 1700000000000,
        };
        round_trip(&ev);
    }

    #[test]
    fn cancel_result_round_trip() {
        round_trip(&CancelResult::NotRunning);
        round_trip(&CancelResult::Cancelled {
            attempt_id: AttemptId::new(),
            requested_at_ms: 999,
        });
        round_trip(&CancelResult::AlreadyTerminal {
            attempt_id: AttemptId::new(),
        });
    }

    #[test]
    fn install_flow_error_round_trip() {
        round_trip(&InstallFlowError::AlreadyRunning);
        round_trip(&InstallFlowError::InvalidPayload {
            field: "plan_id".to_string(),
            reason: "not a valid UUID".to_string(),
        });
        round_trip(&InstallFlowError::ElevationRejected {
            token: "sudo".to_string(),
        });
        round_trip(&InstallFlowError::Internal {
            message: "unexpected".to_string(),
        });
    }

    #[test]
    fn plan_outcome_round_trip() {
        let plan = PlanOutcome::Plan(InstallPlan {
            platform: Platform::Windows,
            package_manager: PackageManager::Winget,
            command: "winget".to_string(),
            args: vec![
                "install".to_string(),
                "--id".to_string(),
                "llmusage".to_string(),
            ],
            envs: BTreeMap::new(),
            elevation_required: false,
            duration_class: DurationClass::Fast,
            plan_id: uuid::Uuid::new_v4(),
        });
        round_trip(&plan);

        let unsupported = PlanOutcome::Unsupported {
            reason: UnsupportedReason::NoPackageManager,
        };
        round_trip(&unsupported);
    }
}
