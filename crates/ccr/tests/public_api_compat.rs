use std::mem::size_of;

#[test]
fn legacy_public_paths_remain_available() {
    // Compatibility contract: the root re-export bridge is legacy-only, but
    // existing downstream imports must remain warning-free until a documented
    // next-major breaking plan exists.
    let _legacy_switch_platform = ccr::application::switch_platform;
    assert!(size_of::<ccr::application::SwitchPlatformRequest>() > 0);
    assert!(size_of::<ccr::commands::ImportMode>() > 0);
    assert!(size_of::<ccr::managers::ConfigManager>() > 0);
    let _ = ccr::models::OpenAiAuthMethod::Api;

    assert!(size_of::<ccr::models::prompt::PromptPreset>() > 0);
    assert!(size_of::<ccr::models::mcp_preset::McpServerSpec>() > 0);
    assert!(size_of::<ccr::models::skills::SkillOperationResponse>() > 0);
    assert!(size_of::<ccr::services::codex_session_service::CodexSessionInventory>() > 0);
    assert!(size_of::<ccr::sessions::SessionFilter>() > 0);
    assert!(size_of::<ccr::sessions::SessionIndexer>() > 0);
    assert!(size_of::<ccr::sessions::SessionSummary>() > 0);
}

#[test]
fn stable_prelude_paths_remain_available() {
    use ccr::prelude::{
        BackupService, CcrError, ConfigManager, ConfigService, HistoryManager, HistoryService,
        Platform, PlatformConfig, PlatformConfigEntry, PlatformConfigManager, PlatformPaths,
        ProfileConfig, Result, SettingsManager, SettingsService, UnifiedConfig, ValidateService,
        create_platform,
    };

    let _ = create_platform as fn(Platform) -> Result<std::sync::Arc<dyn PlatformConfig>>;

    assert!(size_of::<CcrError>() > 0);
    assert!(size_of::<Platform>() > 0);
    assert!(size_of::<PlatformPaths>() > 0);
    assert!(size_of::<ProfileConfig>() > 0);
    assert!(size_of::<PlatformConfigEntry>() > 0);
    assert!(size_of::<UnifiedConfig>() > 0);
    assert!(size_of::<ConfigManager>() > 0);
    assert!(size_of::<PlatformConfigManager>() > 0);
    assert!(size_of::<SettingsManager>() > 0);
    assert!(size_of::<HistoryManager>() > 0);
    assert!(size_of::<ConfigService>() > 0);
    assert!(size_of::<SettingsService>() > 0);
    assert!(size_of::<HistoryService>() > 0);
    assert!(size_of::<BackupService>() > 0);
    assert!(size_of::<ValidateService>() > 0);
}

#[test]
fn crate_root_public_reexport_snapshot_is_intentional() {
    const EXPECTED_PUBLIC_SURFACE: &[&str] = &[
        "pub mod cli;",
        "/// Compatibility-only bridge for historical root imports.",
        "///",
        "/// Keep this broad `ccr_cli` re-export available for existing consumers, but",
        "/// route new stable API through [`crate::prelude`] unless a breaking-release",
        "/// plan intentionally widens the crate root surface.",
        "pub use ccr_cli::{application, commands, managers, models, platforms, services, sync};",
        "#[cfg(feature = \"tui\")]",
        "pub use ccr_tui::tui;",
        "pub mod prelude {",
        "pub use crate::{",
        "BackupService, CcrError, ConfigManager, ConfigService, HistoryManager, HistoryService,",
        "Platform, PlatformConfig, PlatformConfigEntry, PlatformConfigManager, PlatformPaths,",
        "ProfileConfig, Result, SettingsManager, SettingsService, UnifiedConfig, ValidateService,",
        "create_platform,",
        "};",
        "}",
        "pub use ccr_core::{",
        "AutoCompletable, CONFIG_LOCK, CcrError, ColorOutput, FileLock, LockManager, Result,",
        "Validatable, init_file_only_logger, init_logger, mask_if_sensitive, mask_sensitive,",
        "};",
        "pub use managers::{",
        "BudgetManager, CcsConfig, ClaudeSettings, ConfigManager, ConfigSection, CostTracker,",
        "GlobalSettings, HistoryManager, PlatformConfigEntry, PlatformConfigManager, PricingManager,",
        "ProviderType, SettingsManager, TempOverride, TempOverrideManager, UnifiedConfig,",
        "};",
        "pub use models::skills::{",
        "MarketplaceListResponse, MarketplaceSkill, NpxPlatformSupport, NpxStatus, SkillContent,",
        "SkillFileContent, SkillFileEntry, SkillInstallMeta, SkillInstallMode, SkillInstallStrategy,",
        "SkillInstallationRecord, SkillLifecycleSummary, SkillOrigin, SkillPlatformConfig,",
        "SkillPlatformSummary, SkillRecord, SkillSourceHealth, SkillSourceRecord,",
        "SkillSourceSkillRecord, SkillSourceType, SkillTargetRecord, SkillTargetStatus,",
        "SkillsInstallCommandPreview, SkillsInstallRequest, SkillsInstallReviewResponse,",
        "SkillsInstallReviewSource, SkillsInstallReviewTarget, SkillsInventoryQuery,",
        "SkillsInventoryResponse, SkillsNpxCapabilities, SkillsOnboardingCandidate,",
        "SkillsSourceManifest, SkillsSyncRequest,",
        "};",
        "pub use models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};",
        "pub use models::stats::{",
        "Cost, CostRecord, CostStats, DailyCost, ModelPricing, TimeRange, TokenStats, TokenUsage,",
        "};",
        "pub use models::budget::{",
        "BudgetConfig, BudgetLimits, BudgetPeriod, BudgetStatus, BudgetWarning, LimitAction, PeriodCosts,",
        "};",
        "pub use models::pricing::PricingConfig;",
        "pub use platforms::{",
        "PlatformDetector, PlatformInfo, PlatformRegistry, PlatformStatus, create_platform,",
        "};",
        "pub use services::{",
        "BackupService, ConfigService, HistoryService, SettingsService, SkillsService, ValidateService,",
        "};",
        "pub use ccr_store::sessions;",
        "pub use ccr_store::{",
        "Database, Session, SessionEvent, SessionFilter, SessionIndexer, SessionStats, SessionStore,",
        "SessionSummary,",
        "};",
        "pub use sync::{",
        "FolderStats, SyncConfig, SyncConfigManager, SyncContentSelection, SyncContentSelector,",
        "SyncContentType, SyncFolder, SyncFolderManager, SyncFoldersConfig, SyncService, WebDavConfig,",
        "expand_path, get_ccr_sync_path,",
        "};",
    ];

    let source = include_str!("../src/lib.rs");
    let public_surface = collect_public_surface(source);

    assert_eq!(
        public_surface, EXPECTED_PUBLIC_SURFACE,
        "crates/ccr root public surface changed; prefer ccr::prelude for stable API additions, \
         keep legacy re-export changes intentional, and update this snapshot with rationale"
    );
}

fn collect_public_surface(source: &str) -> Vec<&str> {
    let lines: Vec<&str> = source.lines().collect();
    let mut public_surface = Vec::new();
    let mut pending_attributes = Vec::new();
    let mut started = false;
    let mut line_index = 0;

    while line_index < lines.len() {
        let trimmed = lines[line_index].trim();

        if !started {
            if trimmed == "pub mod cli;" {
                started = true;
            } else {
                line_index += 1;
                continue;
            }
        }

        if trimmed.is_empty() {
            pending_attributes.clear();
            line_index += 1;
            continue;
        }

        if trimmed.starts_with("//!") {
            line_index += 1;
            continue;
        }

        if trimmed.starts_with("/// Compatibility-only bridge") {
            pending_attributes.push(trimmed);
            line_index += 1;

            while line_index < lines.len() {
                let doc_line = lines[line_index].trim();

                if !doc_line.starts_with("///") {
                    break;
                }

                pending_attributes.push(doc_line);
                line_index += 1;
            }

            continue;
        }

        if trimmed.starts_with("//") {
            line_index += 1;
            continue;
        }

        if trimmed.starts_with("#[") {
            pending_attributes.push(trimmed);
            line_index += 1;
            continue;
        }

        if trimmed.starts_with("pub use ") || trimmed.starts_with("pub mod ") {
            public_surface.append(&mut pending_attributes);
            public_surface.push(trimmed);

            let mut depth = brace_delta(trimmed);
            line_index += 1;

            if trimmed.ends_with(';') && depth == 0 {
                continue;
            }

            while line_index < lines.len() {
                let block_line = lines[line_index].trim();

                if !block_line.is_empty() && !block_line.starts_with("//") {
                    public_surface.push(block_line);
                    depth += brace_delta(block_line);
                }

                line_index += 1;

                if depth == 0
                    && public_surface
                        .last()
                        .is_some_and(|line| line.ends_with(';') || *line == "}")
                {
                    break;
                }
            }

            continue;
        }

        pending_attributes.clear();
        line_index += 1;
    }

    public_surface
}

fn brace_delta(line: &str) -> isize {
    line.matches('{').count() as isize - line.matches('}').count() as isize
}
