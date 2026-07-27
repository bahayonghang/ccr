//! Platform-aware install plan generation.
//!
//! `generate_plan()` is a pure function over `(DetectionResult, HostCapabilities)`.
//! It selects a closed backend action; executable details never enter a wire DTO.

use crate::services::install_types::{
    DetectionResult, DurationClass, HostCapabilities, InstallAction, InstallFlowError,
    PackageManager, Platform, UnsupportedReason,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GeneratedInstallPlan {
    pub action: InstallAction,
    pub platform: Platform,
    pub package_manager: PackageManager,
    pub duration_class: DurationClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GeneratedPlanOutcome {
    Plan(GeneratedInstallPlan),
    Unsupported { reason: UnsupportedReason },
}

/// Generate an install plan for the current host.
///
/// This is a pure function: no I/O, no side effects.
pub(crate) fn generate_plan(
    _detection: &DetectionResult,
    caps: &HostCapabilities,
) -> Result<GeneratedPlanOutcome, InstallFlowError> {
    Ok(match caps.platform {
        Platform::Macos => plan_macos(caps),
        Platform::Linux => plan_linux(caps),
        Platform::Windows => plan_windows(caps),
    })
}

fn plan_macos(caps: &HostCapabilities) -> GeneratedPlanOutcome {
    if let Some(homebrew_path) = caps.homebrew_path.as_deref() {
        GeneratedPlanOutcome::Plan(GeneratedInstallPlan {
            action: InstallAction::Homebrew {
                homebrew_path: homebrew_path.to_path_buf(),
            },
            platform: Platform::Macos,
            package_manager: PackageManager::Homebrew,
            duration_class: DurationClass::Medium,
        })
    } else if let Some(cargo_path) = caps.cargo_path.as_deref() {
        GeneratedPlanOutcome::Plan(make_cargo_plan_with_path(Platform::Macos, cargo_path))
    } else {
        GeneratedPlanOutcome::Unsupported {
            reason: UnsupportedReason::NoPackageManager,
        }
    }
}

fn plan_linux(caps: &HostCapabilities) -> GeneratedPlanOutcome {
    if let Some(cargo_path) = caps.cargo_path.as_deref() {
        GeneratedPlanOutcome::Plan(make_cargo_plan_with_path(Platform::Linux, cargo_path))
    } else {
        GeneratedPlanOutcome::Unsupported {
            reason: UnsupportedReason::NoPackageManager,
        }
    }
}

fn plan_windows(caps: &HostCapabilities) -> GeneratedPlanOutcome {
    if caps.has_winget {
        GeneratedPlanOutcome::Plan(GeneratedInstallPlan {
            action: InstallAction::Winget,
            platform: Platform::Windows,
            package_manager: PackageManager::Winget,
            duration_class: DurationClass::Fast,
        })
    } else if caps.has_scoop {
        GeneratedPlanOutcome::Plan(GeneratedInstallPlan {
            action: InstallAction::Scoop,
            platform: Platform::Windows,
            package_manager: PackageManager::Scoop,
            duration_class: DurationClass::Fast,
        })
    } else if let Some(cargo_path) = caps.cargo_path.as_deref() {
        GeneratedPlanOutcome::Plan(make_cargo_plan_with_path(Platform::Windows, cargo_path))
    } else {
        GeneratedPlanOutcome::Unsupported {
            reason: UnsupportedReason::NoPackageManager,
        }
    }
}

fn make_cargo_plan_with_path(
    platform: Platform,
    cargo_path: &std::path::Path,
) -> GeneratedInstallPlan {
    GeneratedInstallPlan {
        action: InstallAction::Cargo {
            cargo_path: cargo_path.to_path_buf(),
        },
        platform,
        package_manager: PackageManager::Cargo,
        duration_class: DurationClass::Slow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn caps(
        platform: Platform,
        cargo: bool,
        brew: bool,
        scoop: bool,
        winget: bool,
        cargo_path: Option<&'static str>,
        homebrew_path: Option<&'static str>,
    ) -> HostCapabilities {
        HostCapabilities {
            platform,
            has_cargo: cargo,
            has_homebrew: brew,
            has_scoop: scoop,
            has_winget: winget,
            cargo_path: cargo_path.map(PathBuf::from),
            homebrew_path: homebrew_path.map(PathBuf::from),
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
        let result = generate_plan(
            &absent(),
            &caps(
                Platform::Macos,
                true,
                true,
                false,
                false,
                Some("/Users/example/.cargo/bin/cargo"),
                Some("/opt/homebrew/bin/brew"),
            ),
        )
        .expect("should not error");
        match result {
            GeneratedPlanOutcome::Plan(plan) => {
                assert_eq!(plan.package_manager, PackageManager::Homebrew);
                assert_eq!(
                    plan.action,
                    InstallAction::Homebrew {
                        homebrew_path: PathBuf::from("/opt/homebrew/bin/brew")
                    }
                );
            }
            _ => panic!("expected Plan"),
        }
    }

    #[test]
    fn macos_ignores_homebrew_bool_without_resolved_path() {
        let result = generate_plan(
            &absent(),
            &caps(Platform::Macos, false, true, false, false, None, None),
        )
        .expect("should not error");

        assert_eq!(
            result,
            GeneratedPlanOutcome::Unsupported {
                reason: UnsupportedReason::NoPackageManager
            }
        );
    }

    #[test]
    fn macos_falls_back_to_cargo() {
        let result = generate_plan(
            &absent(),
            &caps(
                Platform::Macos,
                true,
                false,
                false,
                false,
                Some("/Users/example/.cargo/bin/cargo"),
                None,
            ),
        )
        .expect("should not error");
        match result {
            GeneratedPlanOutcome::Plan(plan) => {
                assert_eq!(plan.package_manager, PackageManager::Cargo);
                assert_eq!(
                    plan.action,
                    InstallAction::Cargo {
                        cargo_path: PathBuf::from("/Users/example/.cargo/bin/cargo")
                    }
                );
                assert_eq!(plan.duration_class, DurationClass::Slow);
            }
            _ => panic!("expected Plan"),
        }
    }

    #[test]
    fn macos_ignores_cargo_bool_without_resolved_path() {
        let result = generate_plan(
            &absent(),
            &caps(Platform::Macos, true, false, false, false, None, None),
        )
        .expect("should not error");

        assert_eq!(
            result,
            GeneratedPlanOutcome::Unsupported {
                reason: UnsupportedReason::NoPackageManager
            }
        );
    }

    #[test]
    fn macos_unsupported_no_pm() {
        let result = generate_plan(
            &absent(),
            &caps(Platform::Macos, false, false, false, false, None, None),
        )
        .expect("should not error");
        assert_eq!(
            result,
            GeneratedPlanOutcome::Unsupported {
                reason: UnsupportedReason::NoPackageManager
            }
        );
    }

    // ── Linux ────────────────────────────────────────────────────────────────

    #[test]
    fn linux_uses_cargo() {
        let result = generate_plan(
            &absent(),
            &caps(
                Platform::Linux,
                true,
                true,
                false,
                false,
                Some("/usr/bin/cargo"),
                None,
            ),
        )
        .expect("should not error");
        match result {
            GeneratedPlanOutcome::Plan(plan) => {
                assert_eq!(plan.package_manager, PackageManager::Cargo);
                assert_eq!(plan.platform, Platform::Linux);
            }
            _ => panic!("expected Plan"),
        }
    }

    #[test]
    fn linux_unsupported_no_cargo() {
        let result = generate_plan(
            &absent(),
            &caps(Platform::Linux, false, true, false, false, None, None),
        )
        .expect("should not error");
        assert_eq!(
            result,
            GeneratedPlanOutcome::Unsupported {
                reason: UnsupportedReason::NoPackageManager
            }
        );
    }

    // ── Windows ──────────────────────────────────────────────────────────────

    #[test]
    fn windows_prefers_winget() {
        let result = generate_plan(
            &absent(),
            &caps(
                Platform::Windows,
                true,
                false,
                true,
                true,
                Some("C:\\Users\\user\\.cargo\\bin\\cargo"),
                None,
            ),
        )
        .expect("should not error");
        match result {
            GeneratedPlanOutcome::Plan(plan) => {
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
            &caps(
                Platform::Windows,
                true,
                false,
                true,
                false,
                Some("C:\\Users\\user\\.cargo\\bin\\cargo"),
                None,
            ),
        )
        .expect("should not error");
        match result {
            GeneratedPlanOutcome::Plan(plan) => {
                assert_eq!(plan.package_manager, PackageManager::Scoop);
            }
            _ => panic!("expected Plan"),
        }
    }

    #[test]
    fn windows_falls_back_to_cargo() {
        let result = generate_plan(
            &absent(),
            &caps(
                Platform::Windows,
                true,
                false,
                false,
                false,
                Some("C:\\Users\\user\\.cargo\\bin\\cargo"),
                None,
            ),
        )
        .expect("should not error");
        match result {
            GeneratedPlanOutcome::Plan(plan) => {
                assert_eq!(plan.package_manager, PackageManager::Cargo);
            }
            _ => panic!("expected Plan"),
        }
    }

    #[test]
    fn windows_unsupported_no_pm() {
        let result = generate_plan(
            &absent(),
            &caps(Platform::Windows, false, false, false, false, None, None),
        )
        .expect("should not error");
        assert_eq!(
            result,
            GeneratedPlanOutcome::Unsupported {
                reason: UnsupportedReason::NoPackageManager
            }
        );
    }

    #[test]
    fn all_generated_plans_select_a_closed_action() {
        let platforms = [
            caps(
                Platform::Macos,
                true,
                true,
                false,
                false,
                Some("/opt/homebrew/bin/brew"),
                Some("/opt/homebrew/bin/brew"),
            ),
            caps(
                Platform::Macos,
                true,
                false,
                false,
                false,
                Some("/usr/bin/cargo"),
                None,
            ),
            caps(
                Platform::Linux,
                true,
                false,
                false,
                false,
                Some("/usr/bin/cargo"),
                None,
            ),
            caps(
                Platform::Windows,
                true,
                false,
                false,
                true,
                Some("C:\\Users\\user\\.cargo\\bin\\cargo"),
                None,
            ),
            caps(
                Platform::Windows,
                true,
                false,
                true,
                false,
                Some("C:\\Users\\user\\.cargo\\bin\\cargo"),
                None,
            ),
            caps(
                Platform::Windows,
                true,
                false,
                false,
                false,
                Some("C:\\Users\\user\\.cargo\\bin\\cargo"),
                None,
            ),
        ];

        for c in &platforms {
            if let GeneratedPlanOutcome::Plan(plan) =
                generate_plan(&absent(), c).expect("should not error")
            {
                assert_eq!(
                    plan.action.kind(),
                    match plan.package_manager {
                        PackageManager::Cargo =>
                            crate::services::install_types::InstallActionKind::Cargo,
                        PackageManager::Homebrew =>
                            crate::services::install_types::InstallActionKind::Homebrew,
                        PackageManager::Scoop =>
                            crate::services::install_types::InstallActionKind::Scoop,
                        PackageManager::Winget =>
                            crate::services::install_types::InstallActionKind::Winget,
                    }
                );
            }
        }
    }
}
