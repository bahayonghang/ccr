//! Typed data models for the llmusage install flow.
//!
//! All types are `Serialize`/`Deserialize` for Tauri IPC and JSON round-trip.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────────────────────
// AttemptId
// ──────────────────────────────────────────────────────────────────────────────

/// Unique identifier for a single install attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
#[serde(transparent)]
pub struct AttemptId(#[cfg_attr(feature = "ts", ts(type = "string"))] pub uuid::Uuid);

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

/// Opaque, single-use identifier for a backend-owned install plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
#[serde(transparent)]
pub struct PlanId(#[cfg_attr(feature = "ts", ts(type = "string"))] pub uuid::Uuid);

impl PlanId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for PlanId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PlanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Platform & PackageManager
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
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
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
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
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
pub struct HostCapabilities {
    pub platform: Platform,
    pub has_cargo: bool,
    pub has_homebrew: bool,
    pub has_scoop: bool,
    pub has_winget: bool,
    #[serde(default)]
    pub cargo_path: Option<PathBuf>,
    #[serde(default)]
    pub homebrew_path: Option<PathBuf>,
}

// ──────────────────────────────────────────────────────────────────────────────
// DetectionResult
// ──────────────────────────────────────────────────────────────────────────────

/// Result of probing for the `llmusage` binary on the host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
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
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
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
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
#[serde(tag = "kind")]
pub enum DataRootWarning {
    #[serde(rename = "data_root_missing")]
    DataRootMissing { path: PathBuf },
}

// ──────────────────────────────────────────────────────────────────────────────
// InstallPlanView / PlanOutcome
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
#[serde(tag = "kind")]
pub enum PlanOutcome {
    #[serde(rename = "plan")]
    Plan(InstallPlanView),
    #[serde(rename = "unsupported")]
    Unsupported { reason: UnsupportedReason },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
#[serde(rename_all = "snake_case")]
pub enum UnsupportedReason {
    NoPackageManager,
    ElevationRequired,
}

/// Audit-safe description of a backend-owned install plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
pub struct InstallPlanView {
    pub plan_id: PlanId,
    pub platform: Platform,
    pub package_manager: PackageManager,
    pub action: InstallActionKind,
    pub expected_effects: Vec<String>,
    pub elevation_required: bool,
    pub duration_class: DurationClass,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub expires_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
#[serde(rename_all = "lowercase")]
pub enum DurationClass {
    Fast,
    Medium,
    Slow,
}

/// Renderer-safe name of the closed install action selected by the backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
#[serde(rename_all = "snake_case")]
pub enum InstallActionKind {
    Cargo,
    Homebrew,
    Scoop,
    Winget,
}

/// Executable capability kept inside the backend crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InstallAction {
    Cargo { cargo_path: PathBuf },
    Homebrew { homebrew_path: PathBuf },
    Scoop,
    Winget,
}

impl InstallAction {
    pub(crate) fn kind(&self) -> InstallActionKind {
        match self {
            Self::Cargo { .. } => InstallActionKind::Cargo,
            Self::Homebrew { .. } => InstallActionKind::Homebrew,
            Self::Scoop => InstallActionKind::Scoop,
            Self::Winget => InstallActionKind::Winget,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// InstallEvent
// ──────────────────────────────────────────────────────────────────────────────

/// Events emitted during an install attempt. Delivered in total order per attempt.
/// `Started` precedes all others; exactly one terminal event is emitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
#[serde(tag = "type")]
pub enum InstallEvent {
    #[serde(rename = "started")]
    Started {
        attempt_id: AttemptId,
        plan: InstallPlanView,
    },
    #[serde(rename = "log")]
    Log {
        attempt_id: AttemptId,
        stream: LogStream,
        line: String,
        #[cfg_attr(feature = "ts", ts(as = "f64"))]
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
        #[cfg_attr(feature = "ts", ts(as = "f64"))]
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
        #[cfg_attr(feature = "ts", ts(as = "f64"))]
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
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
#[serde(rename_all = "lowercase")]
pub enum LogStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStage {
    Resolving,
    Downloading,
    Compiling,
    Installing,
    Finalizing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
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
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
#[serde(tag = "kind")]
pub enum CancelResult {
    #[serde(rename = "cancelled")]
    Cancelled {
        attempt_id: AttemptId,
        #[cfg_attr(feature = "ts", ts(as = "f64"))]
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
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
pub struct RingBufferSnapshot {
    pub attempt_id: Option<AttemptId>,
    pub logs: Vec<InstallEvent>,
    pub terminal: Option<InstallEvent>,
}

// ──────────────────────────────────────────────────────────────────────────────
// ManualCatalog
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
pub struct ManualCommand {
    pub platform: Platform,
    pub package_manager: Option<PackageManager>,
    pub title: String,
    pub command_line: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/install/")
)]
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

    #[error("[{code}] install plan `{plan_id}` is unavailable")]
    #[serde(rename = "plan_unavailable")]
    PlanUnavailable {
        code: InstallPlanConsumeErrorCode,
        plan_id: PlanId,
    },

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

/// Stable codes for failures while consuming an opaque install plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallPlanConsumeErrorCode {
    Unknown,
    Expired,
    Reused,
    HostMismatch,
}

impl std::fmt::Display for InstallPlanConsumeErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = match self {
            Self::Unknown => "install_plan_unknown",
            Self::Expired => "install_plan_expired",
            Self::Reused => "install_plan_reused",
            Self::HostMismatch => "install_plan_host_mismatch",
        };
        f.write_str(code)
    }
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

    fn install_plan_view(platform: Platform, package_manager: PackageManager) -> InstallPlanView {
        let action = match package_manager {
            PackageManager::Cargo => InstallActionKind::Cargo,
            PackageManager::Homebrew => InstallActionKind::Homebrew,
            PackageManager::Scoop => InstallActionKind::Scoop,
            PackageManager::Winget => InstallActionKind::Winget,
        };
        InstallPlanView {
            plan_id: PlanId::new(),
            platform,
            package_manager,
            action,
            expected_effects: vec!["Install llmusage".to_string()],
            elevation_required: false,
            duration_class: DurationClass::Medium,
            expires_at_ms: 1_700_000_120_000,
        }
    }

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
    fn renderer_cannot_forge_command_or_modify_args_and_envs() {
        let plan = install_plan_view(Platform::Macos, PackageManager::Homebrew);
        round_trip(&plan);

        let json = serde_json::to_value(&plan).expect("serialize plan view");
        let object = json.as_object().expect("plan view object");
        assert!(
            object
                .get("plan_id")
                .is_some_and(serde_json::Value::is_string)
        );
        assert!(!object.contains_key("command"));
        assert!(!object.contains_key("args"));
        assert!(!object.contains_key("envs"));
    }

    #[test]
    fn install_event_started_round_trip() {
        let ev = InstallEvent::Started {
            attempt_id: AttemptId::new(),
            plan: install_plan_view(Platform::Linux, PackageManager::Cargo),
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
        round_trip(&InstallFlowError::PlanUnavailable {
            code: InstallPlanConsumeErrorCode::Reused,
            plan_id: PlanId::new(),
        });
        round_trip(&InstallFlowError::Internal {
            message: "unexpected".to_string(),
        });
    }

    #[test]
    fn plan_outcome_round_trip() {
        let plan = PlanOutcome::Plan(install_plan_view(Platform::Windows, PackageManager::Winget));
        round_trip(&plan);

        let unsupported = PlanOutcome::Unsupported {
            reason: UnsupportedReason::NoPackageManager,
        };
        round_trip(&unsupported);
    }
}
