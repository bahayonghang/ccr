//! Platform-aware install plan generation.
//!
//! `generate_plan()` is a pure function over `(DetectionResult, HostCapabilities)`.
//! It produces a total decision over `{ macOS, Linux, Windows } × { cargo, homebrew,
//! scoop, winget, none }`, always returning either `Plan(InstallPlan)` or
//! `Unsupported { reason }`.

use std::collections::BTreeMap;

use crate::services::install_types::{
    DetectionResult, DurationClass, HostCapabilities, InstallFlowError, InstallPlan,
    PackageManager, PlanOutcome, Platform, UnsupportedReason,
};

/// The official llmusage crate name used in package registries.
const LLMUSAGE_CRATE_NAME: &str = "llmusage";

/// Elevation-related tokens that must never appear in generated commands.
const ELEVATION_TOKENS: &[&str] = &["sudo", "doas", "su", "runas"];

/// Elevation substring patterns (for Windows PowerShell).
const ELEVATION_SUBSTRINGS: &[&str] = &["Start-Process -Verb RunAs"];

/// Generate an install plan for the current host.
///
/// This is a pure function: no I/O, no side effects.
pub fn generate_plan(
    _detection: &DetectionResult,
    caps: &HostCapabilities,
) -> Result<PlanOutcome, InstallFlowError> {
    let outcome = match caps.platform {
        Platform::Macos => plan_macos(caps),
        Platform::Linux => plan_linux(caps),
        Platform::Windows => plan_windows(caps),
    };

    // Safety check: validate the generated plan does not contain elevation tokens.
    if let PlanOutcome::Plan(ref plan) = outcome {
        validate_command_safety(&plan.command, &plan.args)?;
    }

    Ok(outcome)
}

fn plan_macos(caps: &HostCapabilities) -> PlanOutcome {
    if caps.has_homebrew {
        PlanOutcome::Plan(make_plan(
            Platform::Macos,
            PackageManager::Homebrew,
            "brew",
            &["install", LLMUSAGE_CRATE_NAME],
            DurationClass::Medium,
        ))
    } else if caps.has_cargo {
        PlanOutcome::Plan(make_cargo_plan(Platform::Macos))
    } else {
        PlanOutcome::Unsupported {
            reason: UnsupportedReason::NoPackageManager,
        }
    }
}

fn plan_linux(caps: &HostCapabilities) -> PlanOutcome {
    if caps.has_cargo {
        PlanOutcome::Plan(make_cargo_plan(Platform::Linux))
    } else {
        PlanOutcome::Unsupported {
            reason: UnsupportedReason::NoPackageManager,
        }
    }
}

fn plan_windows(caps: &HostCapabilities) -> PlanOutcome {
    if caps.has_winget {
        PlanOutcome::Plan(make_plan(
            Platform::Windows,
            PackageManager::Winget,
            "winget",
            &["install", "--id", LLMUSAGE_CRATE_NAME, "--source", "winget"],
            DurationClass::Fast,
        ))
    } else if caps.has_scoop {
        PlanOutcome::Plan(make_plan(
            Platform::Windows,
            PackageManager::Scoop,
            "scoop",
            &["install", LLMUSAGE_CRATE_NAME],
            DurationClass::Fast,
        ))
    } else if caps.has_cargo {
        PlanOutcome::Plan(make_cargo_plan(Platform::Windows))
    } else {
        PlanOutcome::Unsupported {
            reason: UnsupportedReason::NoPackageManager,
        }
    }
}

fn make_cargo_plan(platform: Platform) -> InstallPlan {
    InstallPlan {
        platform,
        package_manager: PackageManager::Cargo,
        command: "cargo".to_string(),
        args: vec![
            "install".to_string(),
            "--locked".to_string(),
            LLMUSAGE_CRATE_NAME.to_string(),
        ],
        envs: BTreeMap::new(),
        elevation_required: false,
        duration_class: DurationClass::Slow,
        plan_id: uuid::Uuid::new_v4(),
    }
}

fn make_plan(
    platform: Platform,
    pm: PackageManager,
    command: &str,
    args: &[&str],
    duration_class: DurationClass,
) -> InstallPlan {
    InstallPlan {
        platform,
        package_manager: pm,
        command: command.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
        envs: BTreeMap::new(),
        elevation_required: false,
        duration_class,
        plan_id: uuid::Uuid::new_v4(),
    }
}

/// Validate that a command does not contain elevation constructs.
///
/// Rejects commands whose binary name or arguments match known elevation tokens.
fn validate_command_safety(command: &str, args: &[String]) -> Result<(), InstallFlowError> {
    // Check the command binary itself
    let cmd_lower = command.to_lowercase();
    for token in ELEVATION_TOKENS {
        if cmd_lower == *token {
            return Err(InstallFlowError::ElevationRejected {
                token: token.to_string(),
            });
        }
    }

    // Check arguments for elevation tokens
    for arg in args {
        let arg_lower = arg.to_lowercase();
        for token in ELEVATION_TOKENS {
            if arg_lower == *token {
                return Err(InstallFlowError::ElevationRejected {
                    token: token.to_string(),
                });
            }
        }
    }

    // Check the full command line for elevation substrings
    let full_cmd = format!("{command} {}", args.join(" "));
    for pattern in ELEVATION_SUBSTRINGS {
        if full_cmd.contains(pattern) {
            return Err(InstallFlowError::ElevationRejected {
                token: pattern.to_string(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps(
        platform: Platform,
        cargo: bool,
        brew: bool,
        scoop: bool,
        winget: bool,
    ) -> HostCapabilities {
        HostCapabilities {
            platform,
            has_cargo: cargo,
            has_homebrew: brew,
            has_scoop: scoop,
            has_winget: winget,
        }
    }

    fn absent() -> DetectionResult {
        DetectionResult::Absent {
            reason: crate::services::install_types::AbsentReason::NotOnPath,
            data_root_warning: None,
        }
    }

    // ── macOS ────────────────────────────────────────────────────────────────

    #[test]
    fn macos_prefers_homebrew() {
        let result = generate_plan(&absent(), &caps(Platform::Macos, true, true, false, false))
            .expect("should not error");
        match result {
            PlanOutcome::Plan(plan) => {
                assert_eq!(plan.package_manager, PackageManager::Homebrew);
                assert_eq!(plan.command, "brew");
                assert!(!plan.elevation_required);
            }
            _ => panic!("expected Plan"),
        }
    }

    #[test]
    fn macos_falls_back_to_cargo() {
        let result = generate_plan(&absent(), &caps(Platform::Macos, true, false, false, false))
            .expect("should not error");
        match result {
            PlanOutcome::Plan(plan) => {
                assert_eq!(plan.package_manager, PackageManager::Cargo);
                assert_eq!(plan.duration_class, DurationClass::Slow);
            }
            _ => panic!("expected Plan"),
        }
    }

    #[test]
    fn macos_unsupported_no_pm() {
        let result = generate_plan(
            &absent(),
            &caps(Platform::Macos, false, false, false, false),
        )
        .expect("should not error");
        assert_eq!(
            result,
            PlanOutcome::Unsupported {
                reason: UnsupportedReason::NoPackageManager
            }
        );
    }

    // ── Linux ────────────────────────────────────────────────────────────────

    #[test]
    fn linux_uses_cargo() {
        let result = generate_plan(&absent(), &caps(Platform::Linux, true, true, false, false))
            .expect("should not error");
        match result {
            PlanOutcome::Plan(plan) => {
                assert_eq!(plan.package_manager, PackageManager::Cargo);
                assert_eq!(plan.platform, Platform::Linux);
            }
            _ => panic!("expected Plan"),
        }
    }

    #[test]
    fn linux_unsupported_no_cargo() {
        let result = generate_plan(&absent(), &caps(Platform::Linux, false, true, false, false))
            .expect("should not error");
        assert_eq!(
            result,
            PlanOutcome::Unsupported {
                reason: UnsupportedReason::NoPackageManager
            }
        );
    }

    // ── Windows ──────────────────────────────────────────────────────────────

    #[test]
    fn windows_prefers_winget() {
        let result = generate_plan(&absent(), &caps(Platform::Windows, true, false, true, true))
            .expect("should not error");
        match result {
            PlanOutcome::Plan(plan) => {
                assert_eq!(plan.package_manager, PackageManager::Winget);
                assert_eq!(plan.duration_class, DurationClass::Fast);
            }
            _ => panic!("expected Plan"),
        }
    }

    #[test]
    fn windows_falls_back_to_scoop() {
        let result = generate_plan(
            &absent(),
            &caps(Platform::Windows, true, false, true, false),
        )
        .expect("should not error");
        match result {
            PlanOutcome::Plan(plan) => {
                assert_eq!(plan.package_manager, PackageManager::Scoop);
            }
            _ => panic!("expected Plan"),
        }
    }

    #[test]
    fn windows_falls_back_to_cargo() {
        let result = generate_plan(
            &absent(),
            &caps(Platform::Windows, true, false, false, false),
        )
        .expect("should not error");
        match result {
            PlanOutcome::Plan(plan) => {
                assert_eq!(plan.package_manager, PackageManager::Cargo);
            }
            _ => panic!("expected Plan"),
        }
    }

    #[test]
    fn windows_unsupported_no_pm() {
        let result = generate_plan(
            &absent(),
            &caps(Platform::Windows, false, false, false, false),
        )
        .expect("should not error");
        assert_eq!(
            result,
            PlanOutcome::Unsupported {
                reason: UnsupportedReason::NoPackageManager
            }
        );
    }

    // ── Safety ───────────────────────────────────────────────────────────────

    #[test]
    fn rejects_sudo_command() {
        let err = validate_command_safety("sudo", &["cargo".to_string(), "install".to_string()])
            .expect_err("should reject sudo");
        assert!(matches!(err, InstallFlowError::ElevationRejected { .. }));
    }

    #[test]
    fn rejects_doas_in_args() {
        let err = validate_command_safety("cargo", &["doas".to_string()])
            .expect_err("should reject doas");
        assert!(matches!(err, InstallFlowError::ElevationRejected { .. }));
    }

    #[test]
    fn accepts_safe_command() {
        validate_command_safety("cargo", &["install".to_string(), "--locked".to_string()])
            .expect("should accept safe command");
    }

    // ── All plans have elevation_required = false ────────────────────────────

    #[test]
    fn all_generated_plans_non_elevated() {
        let platforms = [
            caps(Platform::Macos, true, true, false, false),
            caps(Platform::Macos, true, false, false, false),
            caps(Platform::Linux, true, false, false, false),
            caps(Platform::Windows, true, false, false, true),
            caps(Platform::Windows, true, false, true, false),
            caps(Platform::Windows, true, false, false, false),
        ];

        for c in &platforms {
            if let PlanOutcome::Plan(plan) = generate_plan(&absent(), c).expect("should not error")
            {
                assert!(
                    !plan.elevation_required,
                    "plan for {:?}/{:?} should not require elevation",
                    c.platform, plan.package_manager
                );
            }
        }
    }
}
