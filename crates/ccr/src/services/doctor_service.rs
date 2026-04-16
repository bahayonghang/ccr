// 🩺 CCR Doctor 服务
// 聚合本地环境、平台配置、当前 profile、认证状态与可选在线探活。

use crate::managers::conflict_checker::{Conflict, ConflictChecker, ConflictSeverity};
use crate::managers::{CcsConfig, ClaudeSettings, PlatformConfigManager, UnifiedConfig};
use crate::models::{AuthStateStatus, Platform, PlatformPaths, ProfileConfig};
use crate::platforms::claude::ClaudePlatform;
use crate::platforms::droid::DroidSettings;
use crate::platforms::gemini::GeminiSettings;
use crate::platforms::{CodexPlatform, PlatformDetector, create_platform};
use crate::services::health_check::{HealthCheckResult, HealthCheckService, HealthStatus};
use crate::services::{ClaudeAuthService, CodexAuthService};
use ccr_config::platforms::base;
use ccr_core::Validatable;
use futures::future::BoxFuture;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DoctorStatus {
    Ok,
    Warn,
    Fail,
    Skip,
}

impl DoctorStatus {
    pub fn label(&self) -> &'static str {
        match self {
            DoctorStatus::Ok => "[OK]",
            DoctorStatus::Warn => "[WARN]",
            DoctorStatus::Fail => "[FAIL]",
            DoctorStatus::Skip => "[SKIP]",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DoctorSummary {
    pub passed: usize,
    pub warnings: usize,
    pub failed: usize,
    pub skipped: usize,
}

impl DoctorSummary {
    fn record(&mut self, status: DoctorStatus) {
        match status {
            DoctorStatus::Ok => self.passed += 1,
            DoctorStatus::Warn => self.warnings += 1,
            DoctorStatus::Fail => self.failed += 1,
            DoctorStatus::Skip => self.skipped += 1,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorCheck {
    pub id: String,
    pub status: DoctorStatus,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<String>,
}

impl DoctorCheck {
    fn new(id: impl Into<String>, status: DoctorStatus, summary: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status,
            summary: summary.into(),
            path: None,
            detail: None,
            recommendation: None,
        }
    }

    fn ok(id: impl Into<String>, summary: impl Into<String>) -> Self {
        Self::new(id, DoctorStatus::Ok, summary)
    }

    fn warn(id: impl Into<String>, summary: impl Into<String>) -> Self {
        Self::new(id, DoctorStatus::Warn, summary)
    }

    fn fail(id: impl Into<String>, summary: impl Into<String>) -> Self {
        Self::new(id, DoctorStatus::Fail, summary)
    }

    fn skip(id: impl Into<String>, summary: impl Into<String>) -> Self {
        Self::new(id, DoctorStatus::Skip, summary)
    }

    fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    fn with_recommendation(mut self, recommendation: impl Into<String>) -> Self {
        self.recommendation = Some(recommendation.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub scope: String,
    pub online: bool,
    pub summary: DoctorSummary,
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    fn new(scope: impl Into<String>, online: bool) -> Self {
        Self {
            scope: scope.into(),
            online,
            summary: DoctorSummary::default(),
            checks: Vec::new(),
        }
    }

    fn push(&mut self, check: DoctorCheck) {
        self.summary.record(check.status);
        self.checks.push(check);
    }

    pub fn has_failures(&self) -> bool {
        self.summary.failed > 0
    }
}

#[derive(Debug, Clone)]
pub struct DoctorRunOptions {
    pub online: bool,
    pub all_platforms: bool,
    pub platform: Option<String>,
}

pub trait DoctorProviderProbe: Send + Sync {
    fn check(
        &self,
        name: String,
        config: crate::managers::ConfigSection,
    ) -> BoxFuture<'static, HealthCheckResult>;
}

struct LiveDoctorProviderProbe {
    service: Arc<HealthCheckService>,
}

impl Default for LiveDoctorProviderProbe {
    fn default() -> Self {
        Self {
            service: Arc::new(HealthCheckService::new()),
        }
    }
}

impl DoctorProviderProbe for LiveDoctorProviderProbe {
    fn check(
        &self,
        name: String,
        config: crate::managers::ConfigSection,
    ) -> BoxFuture<'static, HealthCheckResult> {
        let service = Arc::clone(&self.service);
        Box::pin(async move { service.check(&name, &config).await })
    }
}

struct GlobalDoctorContext {
    unified: Option<UnifiedConfig>,
    current_platform: Option<Platform>,
    configured_platforms: Vec<Platform>,
}

struct ResolvedScope {
    label: String,
    targets: Vec<Platform>,
}

struct CurrentProfileResolution {
    check: DoctorCheck,
    effective_profile: Option<String>,
}

pub struct DoctorService {
    provider_probe: Arc<dyn DoctorProviderProbe>,
}

impl Default for DoctorService {
    fn default() -> Self {
        Self::new()
    }
}

impl DoctorService {
    pub fn new() -> Self {
        Self {
            provider_probe: Arc::new(LiveDoctorProviderProbe::default()),
        }
    }

    #[cfg(test)]
    pub fn with_provider_probe(provider_probe: Arc<dyn DoctorProviderProbe>) -> Self {
        Self { provider_probe }
    }

    pub async fn run(&self, options: &DoctorRunOptions) -> DoctorReport {
        let mut report = DoctorReport::new("global diagnostics", options.online);
        let context = self.collect_global_checks(&mut report);
        let scope = self.resolve_scope(options, &context);
        report.scope = scope.label;

        if options.all_platforms && scope.targets.is_empty() {
            report.push(
                DoctorCheck::warn(
                    "scope.targets",
                    "No configured platforms were found for --all-platforms.",
                )
                .with_recommendation(
                    "Create or register at least one platform profile and rerun doctor.",
                ),
            );
        }

        for platform in scope.targets {
            self.inspect_platform(
                &mut report,
                context.unified.as_ref(),
                platform,
                options.online,
            )
            .await;
        }

        report
    }

    fn collect_global_checks(&self, report: &mut DoctorReport) -> GlobalDoctorContext {
        let mut context = GlobalDoctorContext {
            unified: None,
            current_platform: None,
            configured_platforms: Vec::new(),
        };

        let platform_manager = match PlatformConfigManager::with_default() {
            Ok(manager) => manager,
            Err(error) => {
                report.push(
                    DoctorCheck::fail(
                        "global.ccr_root",
                        "Unable to resolve the CCR root directory.",
                    )
                    .with_detail(error.to_string())
                    .with_recommendation(
                        "Check your home directory environment and rerun `ccr init` if needed.",
                    ),
                );
                self.collect_conflict_check(report);
                return context;
            }
        };

        let root_dir = platform_manager
            .config_path()
            .parent()
            .map(|path| path.to_path_buf());
        match root_dir {
            Some(root) if root.exists() => {
                report.push(
                    DoctorCheck::ok("global.ccr_root", "CCR root directory is present.")
                        .with_path(root.display().to_string()),
                );
            }
            Some(root) => {
                report.push(
                    DoctorCheck::fail("global.ccr_root", "CCR root directory is missing.")
                        .with_path(root.display().to_string())
                        .with_recommendation("Run `ccr init` to create the CCR workspace."),
                );
            }
            None => {
                report.push(
                    DoctorCheck::fail(
                        "global.ccr_root",
                        "Unable to derive the CCR root directory from config.toml.",
                    )
                    .with_recommendation(
                        "Check the CCR_ROOT environment variable or rerun `ccr init`.",
                    ),
                );
            }
        }

        let registry_path = platform_manager.config_path().to_path_buf();
        if !registry_path.exists() {
            report.push(
                DoctorCheck::fail("global.registry", "CCR registry file is missing.")
                    .with_path(registry_path.display().to_string())
                    .with_recommendation("Run `ccr init` to create ~/.ccr/config.toml."),
            );
        } else {
            match platform_manager.load() {
                Ok(unified) => {
                    report.push(
                        DoctorCheck::ok("global.registry", "CCR registry file is readable.")
                            .with_path(registry_path.display().to_string()),
                    );
                    context.unified = Some(unified);
                }
                Err(error) => {
                    report.push(
                        DoctorCheck::fail(
                            "global.registry",
                            "CCR registry file could not be parsed.",
                        )
                        .with_path(registry_path.display().to_string())
                        .with_detail(error.to_string())
                        .with_recommendation(
                            "Fix ~/.ccr/config.toml or regenerate it with `ccr init`.",
                        ),
                    );
                }
            }
        }

        match context.unified.as_ref() {
            Some(unified) => match Platform::from_str(&unified.current_platform) {
                Ok(platform) if platform.is_implemented() => {
                    context.current_platform = Some(platform);
                    report.push(
                        DoctorCheck::ok(
                            "global.current_platform",
                            format!("Current platform is {}.", platform.short_name()),
                        )
                        .with_detail(format!(
                            "Registry entry points to {}.",
                            platform.display_name()
                        )),
                    );
                }
                Ok(platform) => {
                    report.push(
                        DoctorCheck::fail(
                            "global.current_platform",
                            format!(
                                "Current platform '{}' is not implemented.",
                                platform.short_name()
                            ),
                        )
                        .with_recommendation(
                            "Switch to an implemented platform such as claude, codex, gemini, or droid.",
                        ),
                    );
                }
                Err(error) => {
                    report.push(
                        DoctorCheck::fail(
                            "global.current_platform",
                            "Current platform could not be parsed from the registry.",
                        )
                        .with_detail(error.to_string())
                        .with_recommendation(
                            "Update ~/.ccr/config.toml to a supported platform name.",
                        ),
                    );
                }
            },
            None => {
                report.push(
                    DoctorCheck::fail(
                        "global.current_platform",
                        "Current platform could not be resolved because the registry is unavailable.",
                    )
                    .with_recommendation("Restore ~/.ccr/config.toml before rerunning doctor."),
                );
            }
        }

        let detector = PlatformDetector::new();
        let detected_platforms = match detector.detect_configured_platforms() {
            Ok(platforms) => platforms,
            Err(error) => {
                report.push(
                    DoctorCheck::fail(
                        "global.configured_platforms",
                        "Configured platforms could not be discovered.",
                    )
                    .with_detail(error.to_string())
                    .with_recommendation("Inspect the platform profiles under ~/.ccr/platforms/*."),
                );
                Vec::new()
            }
        };

        let (configured_platforms, unknown_registry_platforms) =
            Self::merge_configured_platforms(context.unified.as_ref(), detected_platforms);
        context.configured_platforms = configured_platforms.clone();

        if configured_platforms.is_empty() {
            report.push(
                DoctorCheck::warn(
                    "global.configured_platforms",
                    "No configured platforms were detected.",
                )
                .with_recommendation(
                    "Create a platform profile or run `ccr init` before rerunning doctor.",
                ),
            );
        } else if unknown_registry_platforms.is_empty() {
            report.push(
                DoctorCheck::ok(
                    "global.configured_platforms",
                    format!(
                        "Configured platforms: {}.",
                        Self::format_platform_names(&configured_platforms)
                    ),
                )
                .with_detail(format!(
                    "Discovered {} configured platform(s).",
                    configured_platforms.len()
                )),
            );
        } else {
            report.push(
                DoctorCheck::warn(
                    "global.configured_platforms",
                    format!(
                        "Configured platforms: {}.",
                        Self::format_platform_names(&configured_platforms)
                    ),
                )
                .with_detail(format!(
                    "Unknown registry entries: {}.",
                    unknown_registry_platforms.join(", ")
                ))
                .with_recommendation(
                    "Remove or rename unsupported platform entries from ~/.ccr/config.toml.",
                ),
            );
        }

        self.collect_conflict_check(report);

        context
    }

    fn collect_conflict_check(&self, report: &mut DoctorReport) {
        match ConflictChecker::new().check_conflicts() {
            Ok(conflict_report) => {
                let critical: Vec<&Conflict> = conflict_report
                    .conflicts
                    .iter()
                    .filter(|conflict| conflict.severity == ConflictSeverity::Critical)
                    .collect();
                let warnings: Vec<&Conflict> = conflict_report
                    .conflicts
                    .iter()
                    .filter(|conflict| conflict.severity == ConflictSeverity::Warning)
                    .collect();
                let info: Vec<&Conflict> = conflict_report
                    .conflicts
                    .iter()
                    .filter(|conflict| conflict.severity == ConflictSeverity::Info)
                    .collect();

                if critical.is_empty()
                    && warnings.is_empty()
                    && info.is_empty()
                    && conflict_report.warnings.is_empty()
                {
                    report.push(DoctorCheck::ok(
                        "global.conflicts",
                        "No local cross-platform conflicts were detected.",
                    ));
                    return;
                }

                if !critical.is_empty() {
                    report.push(
                        DoctorCheck::fail(
                            "global.conflicts",
                            format!("Found {} critical local conflict(s).", critical.len()),
                        )
                        .with_detail(Self::format_conflict_detail(
                            &critical,
                            &conflict_report.warnings,
                        ))
                        .with_recommendation(Self::first_conflict_recommendation(&critical)),
                    );
                    return;
                }

                report.push(
                    DoctorCheck::warn(
                        "global.conflicts",
                        format!(
                            "Found {} warning/info conflict(s) and {} collection warning(s).",
                            warnings.len() + info.len(),
                            conflict_report.warnings.len()
                        ),
                    )
                    .with_detail(Self::format_conflict_detail(
                        &warnings,
                        &conflict_report.warnings,
                    ))
                    .with_recommendation(
                        warnings
                            .first()
                            .map(|conflict| conflict.suggestion.clone())
                            .unwrap_or_else(|| {
                                "Review overlapping local settings before switching platforms."
                                    .to_string()
                            }),
                    ),
                );
            }
            Err(error) => {
                report.push(
                    DoctorCheck::warn(
                        "global.conflicts",
                        "Conflict scan could not inspect every platform setting file.",
                    )
                    .with_detail(error.to_string())
                    .with_recommendation(
                        "Inspect local platform settings manually if conflicts are suspected.",
                    ),
                );
            }
        }
    }

    fn resolve_scope(
        &self,
        options: &DoctorRunOptions,
        context: &GlobalDoctorContext,
    ) -> ResolvedScope {
        if let Some(platform_name) = options.platform.as_deref()
            && let Ok(platform) = Platform::from_str(platform_name)
        {
            return ResolvedScope {
                label: format!("global + platform ({})", platform.short_name()),
                targets: vec![platform],
            };
        }

        if options.all_platforms {
            return ResolvedScope {
                label: "global + all configured platforms".to_string(),
                targets: context.configured_platforms.clone(),
            };
        }

        ResolvedScope {
            label: context
                .current_platform
                .map(|platform| format!("global + current platform ({})", platform.short_name()))
                .unwrap_or_else(|| "global + current platform".to_string()),
            targets: context.current_platform.into_iter().collect(),
        }
    }

    async fn inspect_platform(
        &self,
        report: &mut DoctorReport,
        unified: Option<&UnifiedConfig>,
        platform: Platform,
        online: bool,
    ) {
        let platform_name = platform.short_name();
        if platform == Platform::Qwen {
            report.push(
                DoctorCheck::skip(
                    format!("platform.{platform_name}.support"),
                    "Qwen is not implemented yet; doctor skips platform-specific checks.",
                )
                .with_recommendation(
                    "Switch to an implemented platform or add Qwen support first.",
                ),
            );
            return;
        }

        let paths = match PlatformPaths::new(platform) {
            Ok(paths) => paths,
            Err(error) => {
                report.push(
                    DoctorCheck::fail(
                        format!("platform.{platform_name}.paths"),
                        format!("{} paths could not be resolved.", platform.display_name()),
                    )
                    .with_detail(error.to_string()),
                );
                return;
            }
        };

        let platform_impl = match create_platform(platform) {
            Ok(instance) => instance,
            Err(error) => {
                report.push(
                    DoctorCheck::fail(
                        format!("platform.{platform_name}.instance"),
                        format!(
                            "{} checks could not be initialized.",
                            platform.display_name()
                        ),
                    )
                    .with_detail(error.to_string()),
                );
                return;
            }
        };

        let profiles = if !paths.profiles_file.exists() {
            report.push(
                DoctorCheck::fail(
                    format!("platform.{platform_name}.profiles_file"),
                    format!("{} profiles file is missing.", platform.display_name()),
                )
                .with_path(paths.profiles_file.display().to_string())
                .with_recommendation(format!(
                    "Create {} profiles under {} before rerunning doctor.",
                    platform.display_name(),
                    paths.profiles_file.display()
                )),
            );
            Default::default()
        } else {
            match platform_impl.load_profiles() {
                Ok(profiles) => {
                    report.push(
                        DoctorCheck::ok(
                            format!("platform.{platform_name}.profiles_file"),
                            format!(
                                "{} profiles file is readable ({} profile(s)).",
                                platform.display_name(),
                                profiles.len()
                            ),
                        )
                        .with_path(paths.profiles_file.display().to_string()),
                    );
                    profiles
                }
                Err(error) => {
                    report.push(
                        DoctorCheck::fail(
                            format!("platform.{platform_name}.profiles_file"),
                            format!(
                                "{} profiles file could not be parsed.",
                                platform.display_name()
                            ),
                        )
                        .with_path(paths.profiles_file.display().to_string())
                        .with_detail(error.to_string())
                        .with_recommendation("Fix the profiles TOML before rerunning doctor."),
                    );
                    Default::default()
                }
            }
        };

        let current_profile_resolution =
            self.resolve_current_profile(unified, platform, &paths.profiles_file, &profiles);
        let current_profile = current_profile_resolution.effective_profile.clone();
        report.push(current_profile_resolution.check);

        report.push(
            DoctorCheck::ok(
                format!("platform.{platform_name}.settings_path"),
                format!("{} settings path resolved.", platform.display_name()),
            )
            .with_path(platform_impl.get_settings_path().display().to_string()),
        );

        if let Some(profile_name) = current_profile.as_deref() {
            if let Some(profile) = profiles.get(profile_name) {
                match platform_impl.validate_profile(profile) {
                    Ok(_) => {
                        report.push(
                            DoctorCheck::ok(
                                format!("platform.{platform_name}.profile_validation"),
                                format!(
                                    "Current {} profile '{}' passed local validation.",
                                    platform_name, profile_name
                                ),
                            )
                            .with_detail(Self::profile_validation_detail(platform, profile)),
                        );
                    }
                    Err(error) => {
                        report.push(
                            DoctorCheck::fail(
                                format!("platform.{platform_name}.profile_validation"),
                                format!(
                                    "Current {} profile '{}' is invalid.",
                                    platform_name, profile_name
                                ),
                            )
                            .with_detail(error.to_string())
                            .with_recommendation(
                                "Fix the current profile fields before rerunning doctor.",
                            ),
                        );
                    }
                }

                report.push(self.validate_settings_file(
                    platform,
                    &platform_impl.get_settings_path(),
                    Some(profile),
                ));
                report.push(self.validate_runtime_health(platform, profile));
                if online {
                    report.push(
                        self.run_online_provider_check(platform, profile_name, profile)
                            .await,
                    );
                }
            }
        } else {
            report.push(self.validate_settings_file(
                platform,
                &platform_impl.get_settings_path(),
                None,
            ));
            report.push(
                DoctorCheck::skip(
                    format!("platform.{platform_name}.runtime_auth"),
                    format!(
                        "Skipped {} runtime/auth validation because no current profile is resolved.",
                        platform.display_name()
                    ),
                )
                .with_recommendation(format!(
                    "Set a current {} profile and rerun doctor.",
                    platform_name
                )),
            );
            if online {
                report.push(
                    DoctorCheck::skip(
                        format!("platform.{platform_name}.provider_probe"),
                        format!(
                            "Skipped {} online provider probe because no current profile is resolved.",
                            platform.display_name()
                        ),
                    )
                    .with_recommendation(format!(
                        "Set a current {} profile before using `ccr doctor --online`.",
                        platform_name
                    )),
                );
            }
        }
    }

    fn resolve_current_profile(
        &self,
        unified: Option<&UnifiedConfig>,
        platform: Platform,
        profiles_path: &Path,
        profiles: &indexmap::IndexMap<String, ProfileConfig>,
    ) -> CurrentProfileResolution {
        let platform_name = platform.short_name();
        let registry_current = unified.and_then(|config| {
            config
                .get_platform_profile(platform_name)
                .ok()
                .flatten()
                .map(str::to_string)
        });
        let file_current = Self::read_profiles_current(profiles_path);

        let registry_valid = registry_current
            .as_ref()
            .filter(|name| profiles.contains_key(name.as_str()))
            .cloned();
        let file_valid = file_current
            .as_ref()
            .filter(|name| profiles.contains_key(name.as_str()))
            .cloned();

        let effective_profile = file_valid.clone().or(registry_valid.clone());
        let id = format!("platform.{platform_name}.current_profile");

        let (status, summary, detail, recommendation) = match effective_profile.as_deref() {
            Some(profile_name) if file_valid.is_some() && registry_valid.is_some() => {
                if file_valid != registry_valid {
                    (
                        DoctorStatus::Warn,
                        format!(
                            "Current {} profile resolved to '{}' from profiles.toml, but the registry points elsewhere.",
                            platform_name, profile_name
                        ),
                        Some(format!(
                            "profiles.toml current_config = {:?}, registry current_profile = {:?}.",
                            file_current, registry_current
                        )),
                        Some(format!(
                            "Re-apply or switch the {} profile to realign registry and profiles.toml.",
                            platform_name
                        )),
                    )
                } else {
                    (
                        DoctorStatus::Ok,
                        format!(
                            "Current {} profile resolved to '{}'.",
                            platform_name, profile_name
                        ),
                        Some("Registry and profiles.toml agree on the active profile.".to_string()),
                        None,
                    )
                }
            }
            Some(profile_name) if file_valid.is_some() => (
                DoctorStatus::Warn,
                format!(
                    "Current {} profile resolved to '{}' from profiles.toml.",
                    platform_name, profile_name
                ),
                Some(format!(
                    "Registry current_profile = {:?}.",
                    registry_current
                )),
                Some(format!(
                    "Update the registry by re-applying the active {} profile.",
                    platform_name
                )),
            ),
            Some(profile_name) => (
                DoctorStatus::Warn,
                format!(
                    "Current {} profile resolved to '{}' from the registry.",
                    platform_name, profile_name
                ),
                Some(format!(
                    "profiles.toml current_config = {:?}.",
                    file_current
                )),
                Some(format!(
                    "Update profiles.toml by re-applying the active {} profile.",
                    platform_name
                )),
            ),
            None => (
                DoctorStatus::Fail,
                format!(
                    "No valid current {} profile could be resolved.",
                    platform_name
                ),
                Some(format!(
                    "profiles.toml current_config = {:?}, registry current_profile = {:?}.",
                    file_current, registry_current
                )),
                Some(format!(
                    "Set a current {} profile and rerun doctor.",
                    platform_name
                )),
            ),
        };

        let mut check = DoctorCheck::new(id, status, summary);
        if let Some(detail) = detail {
            check = check.with_detail(detail);
        }
        if let Some(recommendation) = recommendation {
            check = check.with_recommendation(recommendation);
        }

        CurrentProfileResolution {
            check,
            effective_profile,
        }
    }

    fn validate_settings_file(
        &self,
        platform: Platform,
        settings_path: &Path,
        current_profile: Option<&ProfileConfig>,
    ) -> DoctorCheck {
        let id = format!("platform.{}.settings_file", platform.short_name());
        if !settings_path.exists() {
            return DoctorCheck::fail(
                id,
                format!("{} settings file is missing.", platform.display_name()),
            )
            .with_path(settings_path.display().to_string())
            .with_recommendation(
                "Apply a profile or create the settings file before rerunning doctor.",
            );
        }

        match platform {
            Platform::Claude => {
                let content = match fs::read_to_string(settings_path) {
                    Ok(content) => content,
                    Err(error) => {
                        return DoctorCheck::fail(id, "Claude settings file could not be read.")
                            .with_path(settings_path.display().to_string())
                            .with_detail(error.to_string());
                    }
                };

                let settings: ClaudeSettings = match serde_json::from_str(&content) {
                    Ok(settings) => settings,
                    Err(error) => {
                        return DoctorCheck::fail(id, "Claude settings file could not be parsed.")
                            .with_path(settings_path.display().to_string())
                            .with_detail(error.to_string())
                            .with_recommendation(
                                "Fix ~/.claude/settings.json before rerunning doctor.",
                            );
                    }
                };

                let validation = match current_profile {
                    Some(profile)
                        if ClaudePlatform::profile_auth_mode(profile)
                            == crate::models::ClaudeProfileAuthMode::ApiKey =>
                    {
                        settings.validate_api_key_mode()
                    }
                    _ => settings.validate(),
                };

                match validation {
                    Ok(_) => DoctorCheck::ok(id, "Claude settings file is valid.")
                        .with_path(settings_path.display().to_string()),
                    Err(error) => DoctorCheck::fail(id, "Claude settings file is invalid.")
                        .with_path(settings_path.display().to_string())
                        .with_detail(error.to_string())
                        .with_recommendation(
                            "Fix the ANTHROPIC_* overrides or re-apply the active Claude profile.",
                        ),
                }
            }
            Platform::Codex => {
                let content = match fs::read_to_string(settings_path) {
                    Ok(content) => content,
                    Err(error) => {
                        return DoctorCheck::fail(id, "Codex config.toml could not be read.")
                            .with_path(settings_path.display().to_string())
                            .with_detail(error.to_string());
                    }
                };

                match toml::from_str::<toml::Value>(&content) {
                    Ok(_) => DoctorCheck::ok(id, "Codex config.toml is readable.")
                        .with_path(settings_path.display().to_string()),
                    Err(error) => DoctorCheck::fail(id, "Codex config.toml could not be parsed.")
                        .with_path(settings_path.display().to_string())
                        .with_detail(error.to_string())
                        .with_recommendation("Fix ~/.codex/config.toml before rerunning doctor."),
                }
            }
            Platform::Gemini => {
                let content = match fs::read_to_string(settings_path) {
                    Ok(content) => content,
                    Err(error) => {
                        return DoctorCheck::fail(id, "Gemini settings file could not be read.")
                            .with_path(settings_path.display().to_string())
                            .with_detail(error.to_string());
                    }
                };

                match serde_json::from_str::<GeminiSettings>(&content) {
                    Ok(settings) => match settings.validate() {
                        Ok(_) => DoctorCheck::ok(id, "Gemini settings file is valid.")
                            .with_path(settings_path.display().to_string()),
                        Err(error) => DoctorCheck::fail(id, "Gemini settings file is invalid.")
                            .with_path(settings_path.display().to_string())
                            .with_detail(error.to_string())
                            .with_recommendation(
                                "Fix the Gemini settings JSON or re-apply the active profile.",
                            ),
                    },
                    Err(error) => DoctorCheck::fail(
                        id,
                        "Gemini settings file could not be parsed.",
                    )
                    .with_path(settings_path.display().to_string())
                    .with_detail(error.to_string())
                    .with_recommendation(
                        "Fix ~/.ccr/platforms/gemini/settings.json before rerunning doctor.",
                    ),
                }
            }
            Platform::Droid => {
                let content = match fs::read_to_string(settings_path) {
                    Ok(content) => content,
                    Err(error) => {
                        return DoctorCheck::fail(id, "Droid settings file could not be read.")
                            .with_path(settings_path.display().to_string())
                            .with_detail(error.to_string());
                    }
                };

                match serde_json::from_str::<DroidSettings>(&content) {
                    Ok(_) => DoctorCheck::ok(id, "Droid settings file is readable.")
                        .with_path(settings_path.display().to_string()),
                    Err(error) => DoctorCheck::fail(id, "Droid settings file could not be parsed.")
                        .with_path(settings_path.display().to_string())
                        .with_detail(error.to_string())
                        .with_recommendation(
                            "Fix ~/.factory/settings.json before rerunning doctor.",
                        ),
                }
            }
            Platform::Qwen => DoctorCheck::skip(id, "Qwen settings validation is skipped."),
        }
    }

    fn validate_runtime_health(&self, platform: Platform, profile: &ProfileConfig) -> DoctorCheck {
        let id = format!("platform.{}.runtime_auth", platform.short_name());
        match platform {
            Platform::Claude => {
                let auth_mode = ClaudePlatform::profile_auth_mode(profile);
                match ClaudeAuthService::new() {
                    Ok(service) => match service.read_auth_snapshot() {
                        Ok(snapshot) => match auth_mode {
                            crate::models::ClaudeProfileAuthMode::Subscription => {
                                if snapshot.runtime_usable && snapshot.current_info.is_some() {
                                    DoctorCheck::ok(
                                        id,
                                        "Claude subscription runtime credentials are usable.",
                                    )
                                } else if snapshot.current_info.is_some() {
                                    DoctorCheck::fail(
                                        id,
                                        "Claude subscription credentials are expired or pending refresh.",
                                    )
                                    .with_recommendation("Refresh or log in to Claude Code before rerunning doctor.")
                                } else {
                                    DoctorCheck::fail(
                                        id,
                                        "Claude subscription credentials were not detected.",
                                    )
                                    .with_recommendation("Log in to Claude Code or switch the profile to API key mode.")
                                }
                            }
                            crate::models::ClaudeProfileAuthMode::ApiKey => DoctorCheck::ok(
                                id,
                                format!(
                                    "Claude profile uses API key mode ({}).",
                                    ClaudePlatform::profile_auth_source(profile)
                                ),
                            ),
                        },
                        Err(error) => {
                            DoctorCheck::fail(id, "Claude runtime auth snapshot could not be read.")
                                .with_detail(error.to_string())
                        }
                    },
                    Err(error) => {
                        DoctorCheck::fail(id, "Claude auth service could not be initialized.")
                            .with_detail(error.to_string())
                    }
                }
            }
            Platform::Codex => {
                let auth_mode = CodexPlatform::profile_auth_mode(profile);
                let auth_source = CodexPlatform::profile_auth_source(profile);
                match CodexAuthService::new() {
                    Ok(service) => {
                        let auth_state = service.get_auth_state();
                        if auth_mode.uses_openai_auth() {
                            if auth_state.status == AuthStateStatus::Valid {
                                DoctorCheck::ok(
                                    id,
                                    format!(
                                        "Codex runtime auth is ready for {} mode.",
                                        auth_mode.as_str()
                                    ),
                                )
                                .with_detail(format!(
                                    "store = {}, source = {}.",
                                    auth_state.store.as_str(),
                                    auth_source
                                ))
                            } else {
                                DoctorCheck::fail(
                                    id,
                                    "Codex runtime auth is not ready for the current profile.",
                                )
                                .with_detail(format!(
                                    "store = {}, reason = {}.",
                                    auth_state.store.as_str(),
                                    auth_state.reason
                                ))
                                .with_recommendation(
                                    "Log in to Codex or switch cli_auth_credentials_store to file before rerunning doctor.",
                                )
                            }
                        } else {
                            DoctorCheck::ok(
                                id,
                                format!(
                                    "Codex profile uses {} mode and does not require managed OpenAI auth.",
                                    auth_mode.as_str()
                                ),
                            )
                            .with_detail(format!(
                                "store = {}, source = {}.",
                                auth_state.store.as_str(),
                                auth_source
                            ))
                        }
                    }
                    Err(error) => {
                        DoctorCheck::fail(id, "Codex auth service could not be initialized.")
                            .with_detail(error.to_string())
                    }
                }
            }
            Platform::Gemini => DoctorCheck::ok(
                id,
                "Gemini local runtime health is covered by profile and settings validation.",
            ),
            Platform::Droid => DoctorCheck::ok(
                id,
                "Droid local runtime health is covered by profile and settings validation.",
            ),
            Platform::Qwen => DoctorCheck::skip(id, "Qwen runtime validation is skipped."),
        }
    }

    async fn run_online_provider_check(
        &self,
        platform: Platform,
        profile_name: &str,
        profile: &ProfileConfig,
    ) -> DoctorCheck {
        let id = format!("platform.{}.provider_probe", platform.short_name());
        let section = match base::profile_to_section(profile) {
            Ok(section) => section,
            Err(error) => {
                return DoctorCheck::skip(
                    id,
                    format!(
                        "Skipped {} online provider probe because the current profile could not be converted.",
                        platform.display_name()
                    ),
                )
                .with_detail(error.to_string());
            }
        };

        if !Self::is_probeable_section(&section) {
            return DoctorCheck::skip(
                id,
                format!(
                    "Skipped {} online provider probe because the current profile has no probeable endpoint.",
                    platform.display_name()
                ),
            )
            .with_recommendation(
                "Use a profile with base_url + auth_token (or Claude subscription mode) before using `--online`.",
            );
        }

        let result = self
            .provider_probe
            .check(profile_name.to_string(), section)
            .await;

        let mut check = match result.status {
            HealthStatus::Healthy => DoctorCheck::ok(
                id,
                format!(
                    "{} online provider probe succeeded for '{}'.",
                    platform.display_name(),
                    profile_name
                ),
            ),
            HealthStatus::Degraded => DoctorCheck::warn(
                id,
                format!(
                    "{} online provider probe is degraded for '{}'.",
                    platform.display_name(),
                    profile_name
                ),
            ),
            HealthStatus::Unknown => DoctorCheck::warn(
                id,
                format!(
                    "{} online provider probe returned an unknown state for '{}'.",
                    platform.display_name(),
                    profile_name
                ),
            ),
            HealthStatus::Unhealthy => DoctorCheck::fail(
                id,
                format!(
                    "{} online provider probe failed for '{}'.",
                    platform.display_name(),
                    profile_name
                ),
            ),
        }
        .with_path(result.base_url.clone());

        let detail = Self::format_probe_detail(&result);
        if !detail.is_empty() {
            check = check.with_detail(detail);
        }

        if matches!(
            result.status,
            HealthStatus::Degraded | HealthStatus::Unhealthy
        ) || result.error.is_some()
        {
            check = check.with_recommendation(
                "Check the profile base_url, auth_token, model, and network reachability, then rerun `ccr doctor --online`.",
            );
        }

        check
    }

    fn merge_configured_platforms(
        unified: Option<&UnifiedConfig>,
        detected_platforms: Vec<Platform>,
    ) -> (Vec<Platform>, Vec<String>) {
        let mut merged = detected_platforms;
        let mut unknown = Vec::new();

        if let Some(unified) = unified {
            for name in unified.platforms.keys() {
                match Platform::from_str(name) {
                    Ok(platform) => Self::push_unique_platform(&mut merged, platform),
                    Err(_) => unknown.push(name.clone()),
                }
            }
        }

        merged.sort_by_key(|platform| Self::platform_order(*platform));
        (merged, unknown)
    }

    fn push_unique_platform(platforms: &mut Vec<Platform>, platform: Platform) {
        if !platforms.contains(&platform) {
            platforms.push(platform);
        }
    }

    fn platform_order(platform: Platform) -> usize {
        Platform::all()
            .iter()
            .position(|candidate| *candidate == platform)
            .unwrap_or(usize::MAX)
    }

    fn format_platform_names(platforms: &[Platform]) -> String {
        platforms
            .iter()
            .map(Platform::short_name)
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn read_profiles_current(profiles_path: &Path) -> Option<String> {
        if !profiles_path.exists() {
            return None;
        }

        let content = fs::read_to_string(profiles_path).ok()?;
        let parsed = toml::from_str::<CcsConfig>(&content).ok()?;
        let current = parsed.current_config.trim();
        (!current.is_empty()).then(|| current.to_string())
    }

    fn format_conflict_detail(conflicts: &[&Conflict], warnings: &[String]) -> String {
        let mut parts = conflicts
            .iter()
            .map(|conflict| {
                let platforms = conflict
                    .platforms
                    .iter()
                    .map(|value| format!("{}={}", value.platform, value.value))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} ({})", conflict.key, platforms)
            })
            .collect::<Vec<_>>();

        if !warnings.is_empty() {
            parts.extend(warnings.iter().cloned());
        }

        parts.join(" | ")
    }

    fn first_conflict_recommendation(conflicts: &[&Conflict]) -> String {
        conflicts
            .first()
            .map(|conflict| conflict.suggestion.clone())
            .unwrap_or_else(|| {
                "Review overlapping local settings and align them before rerunning doctor."
                    .to_string()
            })
    }

    fn profile_validation_detail(platform: Platform, profile: &ProfileConfig) -> String {
        match platform {
            Platform::Claude => format!(
                "auth_mode = {}, source = {}.",
                ClaudePlatform::profile_auth_mode(profile).as_str(),
                ClaudePlatform::profile_auth_source(profile)
            ),
            Platform::Codex => format!(
                "auth_mode = {}, source = {}.",
                CodexPlatform::profile_auth_mode(profile).as_str(),
                CodexPlatform::profile_auth_source(profile)
            ),
            Platform::Gemini | Platform::Droid | Platform::Qwen => profile
                .model
                .as_deref()
                .map(|model| format!("model = {}.", model))
                .unwrap_or_else(|| "No explicit model configured.".to_string()),
        }
    }

    fn is_probeable_section(section: &crate::managers::ConfigSection) -> bool {
        if section
            .other
            .get("auth_mode")
            .and_then(toml::Value::as_str)
            .is_some_and(|value| value == "subscription")
        {
            return true;
        }

        section
            .base_url
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
            && section
                .auth_token
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
    }

    fn format_probe_detail(result: &HealthCheckResult) -> String {
        let mut parts = Vec::new();
        if let Some(latency) = result.latency_ms {
            parts.push(format!("latency = {} ms", latency));
        }
        if !result.model_available {
            parts.push("configured model is not available".to_string());
        }
        if let Some(error) = result
            .error
            .as_ref()
            .filter(|error| !error.trim().is_empty())
        {
            parts.push(format!("error = {}", error));
        }
        if !result.available_models.is_empty() {
            parts.push(format!(
                "available_models = {}",
                result.available_models.join(", ")
            ));
        }
        parts.join(" | ")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::managers::{
        CcsConfig, ConfigManager, ConfigSection, GlobalSettings, PlatformConfigEntry,
        PlatformConfigManager,
    };
    use indexmap::IndexMap;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{LazyLock, Mutex};
    use tempfile::tempdir;

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    struct CountingProbe {
        calls: Arc<AtomicUsize>,
        status: HealthStatus,
    }

    impl CountingProbe {
        fn new(calls: Arc<AtomicUsize>, status: HealthStatus) -> Self {
            Self { calls, status }
        }
    }

    impl DoctorProviderProbe for CountingProbe {
        fn check(
            &self,
            name: String,
            config: crate::managers::ConfigSection,
        ) -> BoxFuture<'static, HealthCheckResult> {
            let calls = Arc::clone(&self.calls);
            let status = self.status.clone();
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                HealthCheckResult {
                    provider_name: name,
                    base_url: config
                        .base_url
                        .unwrap_or_else(|| "https://api.example.com".to_string()),
                    status,
                    latency_ms: Some(12),
                    error: None,
                    model_available: true,
                    available_models: vec!["test-model".to_string()],
                }
            })
        }
    }

    struct TestEnv {
        _guard: std::sync::MutexGuard<'static, ()>,
        root: std::path::PathBuf,
        home: std::path::PathBuf,
        previous_env: Vec<(String, Option<String>)>,
        _temp_dir: tempfile::TempDir,
    }

    impl TestEnv {
        fn new() -> Self {
            let guard = ENV_LOCK.lock().unwrap();
            let temp_dir = tempdir().unwrap();
            let home = temp_dir.path().join("home");
            let root = home.join(".ccr");
            fs::create_dir_all(&home).unwrap();
            fs::create_dir_all(&root).unwrap();

            let mut env = Self {
                _guard: guard,
                root,
                home,
                previous_env: Vec::new(),
                _temp_dir: temp_dir,
            };
            env.set_env("CCR_ROOT", env.root.display().to_string());
            env.set_env(
                "CCR_SETTINGS_PATH",
                env.home
                    .join(".claude")
                    .join("settings.json")
                    .display()
                    .to_string(),
            );
            env.set_env(
                "CCR_BACKUP_DIR",
                env.home
                    .join(".claude")
                    .join("backups")
                    .display()
                    .to_string(),
            );
            env.set_env(
                "CCR_CODEX_DIR",
                env.home.join(".codex").display().to_string(),
            );
            env.set_env(
                "CLAUDE_CONFIG_DIR",
                env.home.join(".claude").display().to_string(),
            );
            env.set_env("HOME", env.home.display().to_string());
            env.set_env("USERPROFILE", env.home.display().to_string());
            env
        }

        fn set_env(&mut self, key: &str, value: String) {
            self.previous_env
                .push((key.to_string(), std::env::var(key).ok()));
            unsafe { std::env::set_var(key, value) };
        }

        fn write_unified_config(&self, current_platform: &str, current_profile: &str) {
            let manager = PlatformConfigManager::new(self.root.join("config.toml"));
            let mut unified = UnifiedConfig {
                default_platform: current_platform.to_string(),
                current_platform: current_platform.to_string(),
                platforms: IndexMap::new(),
            };
            unified.platforms.insert(
                current_platform.to_string(),
                PlatformConfigEntry {
                    enabled: true,
                    current_profile: Some(current_profile.to_string()),
                    description: None,
                    last_used: None,
                },
            );
            manager.save(&unified).unwrap();
        }

        fn write_claude_profile(&self, name: &str) {
            let manager = ConfigManager::new(
                self.root
                    .join("platforms")
                    .join("claude")
                    .join("profiles.toml"),
            );
            let mut config = CcsConfig {
                default_config: name.to_string(),
                current_config: name.to_string(),
                settings: GlobalSettings::default(),
                sections: IndexMap::new(),
            };
            config.sections.insert(
                name.to_string(),
                ConfigSection {
                    description: Some("Doctor test".to_string()),
                    base_url: Some("https://api.example.com".to_string()),
                    auth_token: Some("sk-test-token".to_string()),
                    model: Some("test-model".to_string()),
                    small_fast_model: None,
                    provider: Some("example".to_string()),
                    provider_type: None,
                    account: None,
                    tags: None,
                    usage_count: Some(0),
                    enabled: Some(true),
                    other: IndexMap::new(),
                },
            );
            manager.save(&config).unwrap();
        }

        fn write_claude_settings(&self) {
            let settings_path = self.home.join(".claude").join("settings.json");
            if let Some(parent) = settings_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(
                settings_path,
                serde_json::to_string_pretty(&ClaudeSettings {
                    env: HashMap::from([
                        (
                            "ANTHROPIC_BASE_URL".to_string(),
                            "https://api.example.com".to_string(),
                        ),
                        (
                            "ANTHROPIC_AUTH_TOKEN".to_string(),
                            "sk-test-token".to_string(),
                        ),
                        ("ANTHROPIC_MODEL".to_string(), "test-model".to_string()),
                    ]),
                    other: HashMap::new(),
                })
                .unwrap(),
            )
            .unwrap();
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            while let Some((key, previous)) = self.previous_env.pop() {
                unsafe {
                    match previous {
                        Some(value) => std::env::set_var(key, value),
                        None => std::env::remove_var(key),
                    }
                }
            }
        }
    }

    #[test]
    fn doctor_summary_tracks_status_counts() {
        let mut report = DoctorReport::new("scope", false);
        report.push(DoctorCheck::ok("a", "ok"));
        report.push(DoctorCheck::warn("b", "warn"));
        report.push(DoctorCheck::fail("c", "fail"));
        report.push(DoctorCheck::skip("d", "skip"));

        assert_eq!(report.summary.passed, 1);
        assert_eq!(report.summary.warnings, 1);
        assert_eq!(report.summary.failed, 1);
        assert_eq!(report.summary.skipped, 1);
        assert!(report.has_failures());
    }

    #[test]
    fn critical_conflicts_map_to_failed_doctor_checks() {
        let critical = Conflict {
            key: "apiKey".to_string(),
            platforms: vec![],
            severity: ConflictSeverity::Critical,
            suggestion: "Unify apiKey".to_string(),
        };

        let detail = DoctorService::format_conflict_detail(&[&critical], &[]);
        assert!(detail.contains("apiKey"));
        assert_eq!(
            DoctorService::first_conflict_recommendation(&[&critical]),
            "Unify apiKey"
        );
    }

    #[tokio::test]
    async fn doctor_service_skips_online_probe_by_default() {
        let env = TestEnv::new();
        env.write_unified_config("claude", "main");
        env.write_claude_profile("main");
        env.write_claude_settings();

        let calls = Arc::new(AtomicUsize::new(0));
        let service = DoctorService::with_provider_probe(Arc::new(CountingProbe::new(
            Arc::clone(&calls),
            HealthStatus::Healthy,
        )));

        let report = service
            .run(&DoctorRunOptions {
                online: false,
                all_platforms: false,
                platform: None,
            })
            .await;

        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(report.summary.failed, 0);
    }

    #[tokio::test]
    async fn doctor_service_calls_online_probe_when_enabled() {
        let env = TestEnv::new();
        env.write_unified_config("claude", "main");
        env.write_claude_profile("main");
        env.write_claude_settings();

        let calls = Arc::new(AtomicUsize::new(0));
        let service = DoctorService::with_provider_probe(Arc::new(CountingProbe::new(
            Arc::clone(&calls),
            HealthStatus::Healthy,
        )));

        let report = service
            .run(&DoctorRunOptions {
                online: true,
                all_platforms: false,
                platform: None,
            })
            .await;

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.id == "platform.claude.provider_probe")
        );
    }
}
