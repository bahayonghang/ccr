//! Read-only diagnostics. Platform adapters own profile and auth rules.

use crate::managers::{ConfigManager, PlatformConfigManager, UnifiedConfig};
use crate::models::{Platform, PlatformConfig, ProfileConfig};
use crate::platforms::create_platform;
use crate::services::doctor_service::{DoctorService, DoctorStatus};
use ccr_config::managers::config_validator::ConfigValidator;
use ccr_config::platforms::base::{resolve_file_current_profile, section_to_profile};
use ccr_core::core::error::{CcrError, exit_codes};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCategory {
    Valid,
    Missing,
    Corrupt,
    Unreadable,
    Invalid,
    Disabled,
    Inactive,
    Runtime,
}

impl DiagnosticCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Missing => "missing",
            Self::Corrupt => "corrupt",
            Self::Unreadable => "unreadable",
            Self::Invalid => "invalid",
            Self::Disabled => "disabled",
            Self::Inactive => "inactive",
            Self::Runtime => "runtime",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticCheck {
    pub platform: Option<Platform>,
    pub target: String,
    pub severity: DiagnosticSeverity,
    pub category: DiagnosticCategory,
    /// Contains no source document, credential, or rejected field value.
    pub message: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DiagnosticReport {
    pub checks: Vec<DiagnosticCheck>,
}

impl DiagnosticReport {
    pub fn has_errors(&self) -> bool {
        self.checks
            .iter()
            .any(|check| check.severity == DiagnosticSeverity::Error)
    }

    /// The first error in registry -> Claude -> Codex -> Grok order wins.
    /// Codes come from CcrError; informational and warning checks never fail.
    pub fn exit_code(&self) -> i32 {
        self.checks
            .iter()
            .find(|check| check.severity == DiagnosticSeverity::Error)
            .map_or(0, |check| check.exit_code)
    }

    fn push(
        &mut self,
        platform: Option<Platform>,
        target: impl Into<String>,
        severity: DiagnosticSeverity,
        category: DiagnosticCategory,
        message: impl Into<String>,
        exit_code: i32,
    ) {
        self.checks.push(DiagnosticCheck {
            platform,
            target: target.into(),
            severity,
            category,
            message: message.into(),
            exit_code,
        });
    }

    fn error(&mut self, platform: Option<Platform>, target: &str, error: CcrError) {
        let category = match &error {
            CcrError::ConfigMissing(_) | CcrError::SettingsMissing(_) => {
                DiagnosticCategory::Missing
            }
            CcrError::ConfigFormatInvalid(_) | CcrError::TomlError(_) | CcrError::JsonError(_) => {
                DiagnosticCategory::Corrupt
            }
            CcrError::FileIoError(_) | CcrError::IoError(_) => DiagnosticCategory::Unreadable,
            _ => DiagnosticCategory::Invalid,
        };
        let severity = if category == DiagnosticCategory::Missing {
            DiagnosticSeverity::Warning
        } else {
            DiagnosticSeverity::Error
        };
        self.push(
            platform,
            target,
            severity,
            category,
            format!(
                "{target}: {}. Inspect the indicated file or profile.",
                category.as_str()
            ),
            error.exit_code(),
        );
    }
}

/// Collect every supported profile platform without initializing any file.
pub fn diagnose() -> DiagnosticReport {
    let mut report = DiagnosticReport::default();
    let registry = match PlatformConfigManager::with_default().and_then(|manager| {
        // try_exists propagates access failures instead of converting them to absence.
        if !manager.config_path().try_exists()? {
            return Err(CcrError::ConfigMissing("registry".into()));
        }
        manager.load()
    }) {
        Ok(registry) => Some(registry),
        Err(error) => {
            report.error(None, "registry", error);
            None
        }
    };
    for &platform in Platform::auth_profile_supported() {
        inspect_profiles(&mut report, platform, registry.as_ref());
    }
    report
}

fn inspect_profiles(
    report: &mut DiagnosticReport,
    platform: Platform,
    registry: Option<&UnifiedConfig>,
) {
    let instance = match create_platform(platform) {
        Ok(instance) => instance,
        Err(error) => {
            report.error(Some(platform), "platform", error);
            return;
        }
    };
    let config = match ConfigManager::for_platform(platform.short_name())
        .and_then(|manager| manager.load())
    {
        Ok(config) => config,
        Err(error) => {
            report.error(Some(platform), "profiles", error);
            inspect_runtime(report, platform, instance.as_ref(), None);
            return;
        }
    };
    let registry_current = registry
        .and_then(|registry| registry.platforms.get(platform.short_name()))
        .and_then(|entry| entry.current_profile.as_deref())
        .filter(|name| !name.is_empty());
    let file_current =
        (!config.current_config.is_empty()).then_some(config.current_config.as_str());
    let claude_current = if platform == Platform::Claude {
        let profiles = config
            .sections
            .iter()
            .map(|(name, section)| (name.clone(), section_to_profile(section)))
            .collect();
        Some(resolve_file_current_profile(
            &profiles,
            file_current,
            registry_current,
        ))
    } else {
        None
    };
    // Reuse Claude's valid file -> valid registry contract. Keep an unresolved
    // explicit marker so the missing-target error remains distinct from inactive.
    let intended = if let Some(resolution) = &claude_current {
        resolution
            .current
            .as_deref()
            .or(file_current)
            .or(registry_current)
    } else {
        registry_current.or(file_current)
    };
    if let Some(current) = claude_current
        .as_ref()
        .and_then(|resolution| resolution.current.as_deref())
        && [file_current, registry_current]
            .into_iter()
            .flatten()
            .any(|marker| marker != current)
    {
        report.push(
            Some(platform),
            "current",
            DiagnosticSeverity::Warning,
            DiagnosticCategory::Invalid,
            "Stored current markers disagree or reference an absent profile. The valid Claude marker was selected without repair.",
            0,
        );
    }
    for (name, section) in &config.sections {
        let profile = section_to_profile(section);
        match ConfigValidator::new().validate_section_with(section, |section| {
            instance.validate_profile(&section_to_profile(section))
        }) {
            Ok(()) => {
                let disabled = !profile.is_enabled();
                let severity = if disabled && intended == Some(name.as_str()) {
                    DiagnosticSeverity::Error
                } else if disabled {
                    DiagnosticSeverity::Warning
                } else {
                    DiagnosticSeverity::Info
                };
                let category = if disabled {
                    DiagnosticCategory::Disabled
                } else {
                    DiagnosticCategory::Valid
                };
                report.push(
                    Some(platform),
                    name,
                    severity,
                    category,
                    if disabled {
                        "Profile is disabled; activation is unavailable."
                    } else {
                        "Profile passed platform/auth-mode validation."
                    },
                    if severity == DiagnosticSeverity::Error {
                        exit_codes::VALIDATION_ERROR
                    } else {
                        0
                    },
                );
            }
            Err(error) => report.error(Some(platform), name, error),
        }
    }
    let Some(name) = intended else {
        report.push(
            Some(platform),
            "current",
            DiagnosticSeverity::Warning,
            DiagnosticCategory::Inactive,
            "No current profile is configured. Stored profiles remain inactive.",
            0,
        );
        // Existing runtime files still need syntactic/read validation.
        inspect_runtime(report, platform, instance.as_ref(), None);
        return;
    };
    let Some(section) = config.sections.get(name) else {
        report.push(
            Some(platform),
            "current",
            DiagnosticSeverity::Error,
            DiagnosticCategory::Invalid,
            "Recorded current profile does not exist.",
            exit_codes::PROFILE_NOT_FOUND,
        );
        inspect_runtime(report, platform, instance.as_ref(), None);
        return;
    };
    let profile = section_to_profile(section);
    if !profile.is_enabled() || instance.validate_profile(&profile).is_err() {
        inspect_runtime(report, platform, instance.as_ref(), None);
        return;
    }
    // Classify the runtime file before platform current resolution also reads
    // it; otherwise an adapter can replace the precise IO/parser error code.
    if !inspect_runtime(report, platform, instance.as_ref(), Some(&profile)) {
        return;
    }
    match instance.get_current_profile() {
        Ok(Some(_)) => {}
        Ok(None) => report.push(
            Some(platform),
            "current",
            DiagnosticSeverity::Warning,
            DiagnosticCategory::Inactive,
            "Recorded profile is not active in the runtime. Re-apply explicitly if intended.",
            0,
        ),
        Err(error) => report.error(Some(platform), "current", error),
    }
}

fn inspect_runtime(
    report: &mut DiagnosticReport,
    platform: Platform,
    instance: &dyn PlatformConfig,
    profile: Option<&ProfileConfig>,
) -> bool {
    let service = DoctorService::new();
    let path = instance.get_settings_path();
    // Read directly so access/parse errors never enter a missing-file branch.
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            report.push(Some(platform), "settings", DiagnosticSeverity::Warning, DiagnosticCategory::Missing, "Runtime settings are not configured. Apply a profile explicitly to initialize them.", 0);
            return false;
        }
        Err(error) => {
            report.error(
                Some(platform),
                "settings",
                CcrError::FileIoError(error.to_string()),
            );
            return false;
        }
    };
    let syntax_valid = match platform {
        Platform::Claude => {
            serde_json::from_slice::<crate::managers::ClaudeSettings>(&bytes).is_ok()
        }
        _ => std::str::from_utf8(&bytes)
            .ok()
            .and_then(|text| toml::from_str::<toml::Table>(text).ok())
            .is_some(),
    };
    if !syntax_valid {
        report.push(
            Some(platform),
            "settings",
            DiagnosticSeverity::Error,
            DiagnosticCategory::Corrupt,
            "Runtime settings cannot be parsed.",
            if platform == Platform::Claude {
                exit_codes::JSON_ERROR
            } else {
                exit_codes::TOML_ERROR
            },
        );
        return false;
    }
    let mut checks = vec![service.validate_settings_file(platform, &path, profile)];
    if let Some(profile) = profile {
        checks.push(service.validate_runtime_health(platform, profile));
    }
    for check in checks {
        let severity = match check.status {
            DoctorStatus::Fail => DiagnosticSeverity::Error,
            DoctorStatus::Warn => DiagnosticSeverity::Warning,
            _ => DiagnosticSeverity::Info,
        };
        // Keep doctor summaries, which contain no credential values. Detailed
        // parser errors and provider-specific rejected values are not copied.
        report.push(
            Some(platform),
            check.id,
            severity,
            DiagnosticCategory::Runtime,
            check.summary,
            if severity == DiagnosticSeverity::Error {
                exit_codes::VALIDATION_ERROR
            } else {
                0
            },
        );
    }
    true
}
