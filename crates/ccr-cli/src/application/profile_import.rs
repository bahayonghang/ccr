//! Shared `profile import` preview & CAS transaction service for Claude, Codex, and Grok.
//!
//! Owns 3-format input parsing, legacy masked-secret rejection, credential-pending disablement,
//! compatible local secret preservation, normalized spec equality, 6+1 bucket accounting,
//! active runtime protection + re-apply, Codex `profile_secrets.json` batch sync, and
//! `operation_lock -> WriteJournal -> CatalogTransaction` atomic commit.

use crate::application::profile_export::{
    ensure_supported_platform, inject_synthetic_secret_into_section, overlay_codex_secret_section,
    platform_secret_stdin_flag,
};
use crate::application::profile_lifecycle::{ProfileLifecycleContext, operation_paths_with};
use crate::models::{ClaudeProfileAuthMode, Platform, PlatformConfig};
use crate::platforms::grok::GrokActivationState;
use crate::platforms::{ClaudePlatform, GrokPlatform};
use ccr_codex::CodexPlatform;
use ccr_codex::models::{CodexProfileAuthMode, CodexProfileSecret, CodexProfileSecretStore};
use ccr_config::managers::config::CatalogTransaction;
use ccr_config::managers::config::repository::validate_profile_name;
use ccr_config::managers::config_validator::ConfigValidator;
use ccr_config::managers::{CcsConfig, ConfigSection, GlobalSettings, ProviderType};
use ccr_config::platforms::base::section_to_profile;
use ccr_config::{ImportSourceFormat, canonical_section_spec_bytes, parse_import_document};
use ccr_core::Secret;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::write_journal::WriteJournal;
use chrono::Utc;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportModeKind {
    Merge,
    Replace,
}

impl ImportModeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Merge => "merge",
            Self::Replace => "replace",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStrategy {
    #[default]
    Skip,
    Overwrite,
    Rename,
}

impl ConflictStrategy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Skip => "skip",
            Self::Overwrite => "overwrite",
            Self::Rename => "rename",
        }
    }
}

#[derive(Clone)]
pub struct ImportProfilesRequest {
    pub platform: Platform,
    pub content: String,
    pub mode: ImportModeKind,
    pub on_conflict: ConflictStrategy,
    pub dedup: bool,
    pub dry_run: bool,
    pub no_apply: bool,
    pub expected_revision: Option<String>,
}

impl std::fmt::Debug for ImportProfilesRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImportProfilesRequest")
            .field("platform", &self.platform)
            .field("content", &"<redacted>")
            .field("mode", &self.mode)
            .field("on_conflict", &self.on_conflict)
            .field("dedup", &self.dedup)
            .field("dry_run", &self.dry_run)
            .field("no_apply", &self.no_apply)
            .field("expected_revision", &self.expected_revision)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenamedProfileItem {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedDuplicateItem {
    pub name: String,
    pub duplicate_of: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileImportCounts {
    pub total_input: usize,
    pub total_incoming: usize,
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub renamed: usize,
    pub skipped: usize,
    pub skipped_conflict: usize,
    pub skipped_duplicate: usize,
    pub removed: usize,
    pub disabled_missing_secret: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileImportReport {
    pub ok: bool,
    pub platform: String,
    pub mode: String,
    pub on_conflict: Option<String>,
    pub dedup: bool,
    pub dry_run: bool,
    pub changed: bool,
    pub source_format: ImportSourceFormat,
    pub counts: ProfileImportCounts,
    pub added_names: Vec<String>,
    pub updated_names: Vec<String>,
    pub unchanged_names: Vec<String>,
    pub renamed_items: Vec<RenamedProfileItem>,
    pub skipped_conflicts: Vec<String>,
    pub skipped_duplicates: Vec<SkippedDuplicateItem>,
    pub removed_names: Vec<String>,
    pub preserved_local_secrets: Vec<String>,
    pub missing_secrets: Vec<String>,
    pub reapplied_profile: Option<String>,
    pub backup_path: Option<String>,
    pub base_revision: String,
    pub revision: String,
    pub warnings: Vec<String>,
}

struct ComputedImportPlan {
    report: ProfileImportReport,
    next_config: CcsConfig,
    next_codex_secrets: Option<CodexProfileSecretStore>,
    reapply_target: Option<String>,
}

#[derive(Debug, Clone)]
struct ActiveRuntimeStatus {
    active_profile: Option<String>,
    can_reapply: bool,
    grok_unsafe_missing_entry: Option<Option<String>>,
}

pub fn preview_import(request: &ImportProfilesRequest) -> Result<ProfileImportReport> {
    ensure_supported_platform(request.platform)?;
    let context = ProfileLifecycleContext::new(request.platform)?;
    preview_import_with_context(&context, request)
}

pub fn preview_import_with_context(
    context: &ProfileLifecycleContext,
    request: &ImportProfilesRequest,
) -> Result<ProfileImportReport> {
    ensure_supported_platform(request.platform)?;
    validate_import_request_flags(context, request)?;
    let plan = evaluate_plan(context, context.instance(), request, true)?;
    Ok(plan.report)
}

pub fn commit_import(request: ImportProfilesRequest) -> Result<ProfileImportReport> {
    ensure_supported_platform(request.platform)?;
    let context = ProfileLifecycleContext::new(request.platform)?;
    commit_import_with_context(&context, request)
}

pub fn commit_import_with_context(
    context: &ProfileLifecycleContext,
    request: ImportProfilesRequest,
) -> Result<ProfileImportReport> {
    ensure_supported_platform(request.platform)?;
    validate_import_request_flags(context, &request)?;

    if request.dry_run {
        return preview_import_with_context(context, &request);
    }

    // Fast fail on TOML syntax / schema / platform / masked-secret errors before acquiring any lock.
    let _ = parse_import_document(&request.content, request.platform.short_name())?;

    context.run(|| {
        let _operation_lock = context.operation_lock()?;
        context.with_instance(|instance| {
            let manager = context.manager();
            let _catalog_guard = manager.lock_mutation()?;

            let mut plan = evaluate_plan(context, instance, &request, false)?;
            if !plan.report.changed {
                return Ok(plan.report);
            }

            let journal_paths = operation_paths_with(context.paths(), request.platform, instance)?;
            let journal = WriteJournal::begin(&journal_paths)?;

            let catalog_existed = !plan.report.revision.is_empty();
            let backup_file_path = if catalog_existed {
                Some(manager.backup(Some("import"))?)
            } else {
                None
            };

            let locked_base_revision = plan.report.revision.clone();
            let next_config = plan.next_config;
            let next_codex_secrets = plan.next_codex_secrets;
            let reapply_target = plan.reapply_target.clone();

            let tx_result = CatalogTransaction::execute_or_create(
                manager.config_path(),
                manager.profiles_db(),
                Some(&locked_base_revision),
                |_| {
                    if let Some(secret_store) = next_codex_secrets.as_ref() {
                        context
                            .codex_platform()?
                            .runtime_service()
                            .save_secret_store(secret_store)?;
                    }

                    manager.save(&next_config)?;

                    if let Some(active_name) = reapply_target.as_deref() {
                        instance.apply_profile(active_name)?;
                    }

                    journal.verify()
                },
            );

            match tx_result {
                Ok(()) => {
                    journal.commit();
                    if reapply_target.is_some()
                        && let Some(auth_service) = context_codex_auth_sync(context)
                    {
                        let _ = auth_service.sync_current_auth_registry();
                    }
                    let final_revision = read_current_revision(&manager)?;
                    plan.report.revision = final_revision;
                    plan.report.backup_path =
                        backup_file_path.map(|path| path.display().to_string());
                    plan.report.reapplied_profile = reapply_target;
                    Ok(plan.report)
                }
                Err(err) => {
                    let failed = journal.rollback();
                    if failed.is_empty() {
                        if let Some(ref bp) = backup_file_path {
                            let _ = std::fs::remove_file(bp);
                        }
                        Err(err)
                    } else {
                        let unrestored = failed
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        Err(CcrError::ConfigError(format!(
                            "Profile 导入回滚需要恢复，未恢复的外部文件: {unrestored}"
                        )))
                    }
                }
            }
        })
    })
}

fn context_codex_auth_sync(
    context: &ProfileLifecycleContext,
) -> Option<ccr_codex::services::CodexAuthService> {
    if context.platform() != Platform::Codex {
        return None;
    }
    let runtime = context.instance().get_settings_path();
    let dir = runtime.parent()?.to_path_buf();
    Some(ccr_codex::services::CodexAuthService::from_dirs(
        context.paths().platform_dir.clone(),
        dir,
    ))
}

fn validate_import_request_flags(
    context: &ProfileLifecycleContext,
    request: &ImportProfilesRequest,
) -> Result<()> {
    if context.platform() != request.platform {
        return Err(CcrError::ValidationError(
            "导入请求平台与上下文平台不一致".into(),
        ));
    }
    if request.mode == ImportModeKind::Replace && request.dedup {
        return Err(CcrError::ValidationError(
            "--replace 与 --dedup 不能同时使用".into(),
        ));
    }
    Ok(())
}

fn read_current_revision(manager: &ccr_config::ConfigManager) -> Result<String> {
    match manager.canonical_toml()? {
        Some((_content, revision)) => Ok(revision),
        None => Ok(String::new()),
    }
}

fn load_existing_catalog(
    manager: &ccr_config::ConfigManager,
) -> Result<(Option<CcsConfig>, String)> {
    let revision = match manager.canonical_toml()? {
        Some((_content, rev)) => rev,
        None => return Ok((None, String::new())),
    };
    let config = manager.load()?;
    Ok((Some(config), revision))
}

fn inspect_active_runtime(
    context: &ProfileLifecycleContext,
    instance: &dyn PlatformConfig,
) -> Result<ActiveRuntimeStatus> {
    match context.platform() {
        Platform::Grok => {
            let grok = context.grok_platform()?;
            match grok.inspect_activation_state()? {
                GrokActivationState::Inactive => Ok(ActiveRuntimeStatus {
                    active_profile: None,
                    can_reapply: false,
                    grok_unsafe_missing_entry: None,
                }),
                GrokActivationState::Active { name } | GrokActivationState::Drifted { name } => {
                    Ok(ActiveRuntimeStatus {
                        active_profile: Some(name),
                        can_reapply: true,
                        grok_unsafe_missing_entry: None,
                    })
                }
                GrokActivationState::UnsafeMissingEntryState { name } => Ok(ActiveRuntimeStatus {
                    active_profile: name.clone(),
                    can_reapply: false,
                    grok_unsafe_missing_entry: Some(name),
                }),
            }
        }
        Platform::Codex => {
            let current = instance.get_current_profile()?;
            let can_reapply = current.is_some();
            let active_profile = if current.is_some() {
                current
            } else {
                super::profile_off::codex_raw_current_profile_hint(context)?
            };
            Ok(ActiveRuntimeStatus {
                active_profile,
                can_reapply,
                grok_unsafe_missing_entry: None,
            })
        }
        Platform::Claude => {
            let current = instance.get_current_profile()?;
            let can_reapply = current.is_some();
            Ok(ActiveRuntimeStatus {
                active_profile: current,
                can_reapply,
                grok_unsafe_missing_entry: None,
            })
        }
        _ => Ok(ActiveRuntimeStatus {
            active_profile: None,
            can_reapply: false,
            grok_unsafe_missing_entry: None,
        }),
    }
}

fn evaluate_plan(
    context: &ProfileLifecycleContext,
    instance: &dyn PlatformConfig,
    request: &ImportProfilesRequest,
    is_dry_run_view: bool,
) -> Result<ComputedImportPlan> {
    let platform = request.platform;
    let parsed = parse_import_document(&request.content, platform.short_name())?;

    let manager = context.manager();
    let (existing_catalog_opt, base_revision) = load_existing_catalog(&manager)?;

    if let Some(expected) = request.expected_revision.as_deref()
        && expected != base_revision
    {
        return Err(CcrError::ValidationError(
            "配置版本冲突，请重新读取后重试".into(),
        ));
    }

    let existing_codex_secrets = if platform == Platform::Codex {
        Some(
            context
                .codex_platform()?
                .runtime_service()
                .load_secret_store()?,
        )
    } else {
        None
    };

    // Build overlaid & normalized map of existing local profiles for comparison.
    let existing_raw_sections: IndexMap<String, ConfigSection> = existing_catalog_opt
        .as_ref()
        .map(|cfg| cfg.sections.clone())
        .unwrap_or_default();

    let mut existing_overlaid = IndexMap::with_capacity(existing_raw_sections.len());
    let mut existing_spec_bytes = IndexMap::with_capacity(existing_raw_sections.len());
    for (name, mut sec) in existing_raw_sections.clone() {
        overlay_codex_secret_section(&name, &mut sec, existing_codex_secrets.as_ref());
        let cmp_sec = normalize_section_for_platform(platform, &name, &sec, false)?;
        existing_spec_bytes.insert(name.clone(), canonical_section_spec_bytes(&cmp_sec)?);
        existing_overlaid.insert(name, sec);
    }

    // Phase 1: Pre-validate ALL incoming profiles before touching any bucket or state.
    let mut validated_incoming = IndexMap::with_capacity(parsed.profiles.len());
    for (name, raw_section) in parsed.profiles {
        let normalized = normalize_section_for_platform(platform, &name, &raw_section, true)?;
        let needs_secret = section_requires_inline_secret(platform, &normalized)?;
        let has_secret = section_has_inline_secret(platform, &normalized);

        validate_incoming_section(
            platform,
            &name,
            &normalized,
            instance,
            needs_secret && !has_secret,
        )?;
        validated_incoming.insert(name, (normalized, needs_secret, has_secret));
    }

    let total_incoming = validated_incoming.len();
    let mut added = Vec::new();
    let mut updated = Vec::new();
    let mut unchanged = Vec::new();
    let mut renamed = Vec::new();
    let mut skipped_conflict = Vec::new();
    let mut skipped_duplicate = Vec::new();
    let mut removed = Vec::new();
    let mut missing_secrets = Vec::new();
    let mut preserved_local_secrets = Vec::new();
    let mut warnings = Vec::new();

    // Occupied set for deterministic `--on-conflict rename`:
    // all existing local names U all original incoming names.
    let mut occupied: IndexSet<String> = existing_overlaid
        .keys()
        .chain(validated_incoming.keys())
        .cloned()
        .collect();

    // Spec fingerprints for `--dedup` (Merge mode only):
    // initialized with existing local catalog entries in catalog order.
    let mut effective_specs: Vec<(String, Vec<u8>)> = existing_spec_bytes
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let mut next_sections: IndexMap<String, ConfigSection> = match request.mode {
        ImportModeKind::Merge => existing_overlaid.clone(),
        ImportModeKind::Replace => IndexMap::new(),
    };

    for (name, (mut candidate, needs_secret, has_secret)) in validated_incoming {
        if let Some(local_existing) = existing_overlaid.get(&name) {
            // Same-name branch
            let local_has_secret = section_has_inline_secret(platform, local_existing);

            // Prepare candidate for overwrite/replace/equality check:
            // - If incoming requires inline secret, lacks it, and local has one -> backfill local secret.
            // - If incoming does NOT require inline secret -> ensure no stale inline secret remains.
            let mut merged_same_name = candidate.clone();
            let mut would_preserve_local_secret = false;
            let mut still_missing_secret = false;

            if needs_secret && !has_secret {
                if local_has_secret {
                    copy_local_secret(platform, local_existing, &mut merged_same_name);
                    validate_incoming_section(platform, &name, &merged_same_name, instance, false)?;
                    would_preserve_local_secret = true;
                } else {
                    merged_same_name.enabled = Some(false);
                    still_missing_secret = true;
                }
            } else if !needs_secret {
                clear_inline_secret(&mut merged_same_name);
            }

            let merged_cmp =
                normalize_section_for_platform(platform, &name, &merged_same_name, false)?;
            let merged_bytes = canonical_section_spec_bytes(&merged_cmp)?;
            let local_bytes = existing_spec_bytes
                .get(&name)
                .ok_or_else(|| CcrError::ConfigError("内部目录索引缺失".into()))?;

            if &merged_bytes == local_bytes {
                unchanged.push(name.clone());
                if request.mode == ImportModeKind::Replace {
                    let mut kept = local_existing.clone();
                    kept.usage_count = existing_raw_sections
                        .get(&name)
                        .and_then(|orig| orig.usage_count);
                    next_sections.insert(name, kept);
                }
                continue;
            }

            match request.mode {
                ImportModeKind::Replace => {
                    merged_same_name.usage_count = existing_raw_sections
                        .get(&name)
                        .and_then(|orig| orig.usage_count);
                    next_sections.insert(name.clone(), merged_same_name);
                    updated.push(name.clone());
                    if would_preserve_local_secret {
                        preserved_local_secrets.push(name);
                    } else if still_missing_secret {
                        missing_secrets.push(name);
                    }
                }
                ImportModeKind::Merge => match request.on_conflict {
                    ConflictStrategy::Skip => {
                        skipped_conflict.push(name);
                    }
                    ConflictStrategy::Overwrite => {
                        merged_same_name.usage_count = existing_raw_sections
                            .get(&name)
                            .and_then(|orig| orig.usage_count);
                        next_sections.insert(name.clone(), merged_same_name);
                        if let Some(slot) = effective_specs.iter_mut().find(|(k, _)| k == &name) {
                            slot.1 = merged_bytes;
                        }
                        updated.push(name.clone());
                        if would_preserve_local_secret {
                            preserved_local_secrets.push(name);
                        } else if still_missing_secret {
                            missing_secrets.push(name);
                        }
                    }
                    ConflictStrategy::Rename => {
                        // Renamed copy is a brand-new profile: never copy `<name>`'s secret onto `<name>-imported`.
                        let mut renamed_section = candidate;
                        let renamed_missing = needs_secret && !has_secret;
                        if renamed_missing {
                            renamed_section.enabled = Some(false);
                        } else if !needs_secret {
                            clear_inline_secret(&mut renamed_section);
                        }
                        renamed_section.usage_count = Some(0);

                        let renamed_cmp = normalize_section_for_platform(
                            platform,
                            &name,
                            &renamed_section,
                            false,
                        )?;
                        let renamed_bytes = canonical_section_spec_bytes(&renamed_cmp)?;

                        if request.dedup
                            && let Some((dup_of, _)) = effective_specs
                                .iter()
                                .find(|(_, existing_b)| existing_b == &renamed_bytes)
                        {
                            skipped_duplicate.push(SkippedDuplicateItem {
                                name,
                                duplicate_of: dup_of.clone(),
                            });
                            continue;
                        }

                        let new_name = allocate_renamed_profile_name(&name, &mut occupied)?;
                        next_sections.insert(new_name.clone(), renamed_section);
                        effective_specs.push((new_name.clone(), renamed_bytes));
                        if renamed_missing {
                            missing_secrets.push(new_name.clone());
                        }
                        renamed.push(RenamedProfileItem {
                            from: name,
                            to: new_name,
                        });
                    }
                },
            }
        } else {
            // Brand-new profile name
            if let Some((case_match, _)) = existing_overlaid
                .iter()
                .find(|(existing_name, _)| existing_name.eq_ignore_ascii_case(&name))
            {
                warnings.push(format!(
                    "导入项 '{name}' 与本机已有项 '{case_match}' 仅大小写不同，将作为独立 Profile 处理"
                ));
            }

            let is_missing_secret = needs_secret && !has_secret;
            if is_missing_secret {
                candidate.enabled = Some(false);
            } else if !needs_secret {
                clear_inline_secret(&mut candidate);
            }
            candidate.usage_count = Some(0);

            let cand_cmp = normalize_section_for_platform(platform, &name, &candidate, false)?;
            let cand_bytes = canonical_section_spec_bytes(&cand_cmp)?;

            if request.mode == ImportModeKind::Merge
                && let Some((dup_name, _)) = effective_specs
                    .iter()
                    .find(|(_, existing_b)| existing_b == &cand_bytes)
            {
                if request.dedup {
                    skipped_duplicate.push(SkippedDuplicateItem {
                        name,
                        duplicate_of: dup_name.clone(),
                    });
                    continue;
                }
                warnings.push(format!(
                    "新增 Profile '{name}' 与 '{dup_name}' 内容相同（如需自动跳过重复项可附加 `--dedup`）"
                ));
            }

            next_sections.insert(name.clone(), candidate);
            effective_specs.push((name.clone(), cand_bytes));
            added.push(name.clone());
            if is_missing_secret {
                missing_secrets.push(name);
            }
        }
    }

    if request.mode == ImportModeKind::Replace {
        for old_name in existing_overlaid.keys() {
            if !next_sections.contains_key(old_name) {
                removed.push(old_name.clone());
            }
        }
    }

    // Active runtime protection & re-apply evaluation (#173 section 6.2)
    let active_status = inspect_active_runtime(context, instance)?;

    if let Some(unsafe_opt) = &active_status.grok_unsafe_missing_entry {
        match unsafe_opt {
            Some(p) if updated.contains(p) || removed.contains(p) => {
                return Err(CcrError::ValidationError(format!(
                    "Grok 入口配置状态缺失，拒绝通过导入修改或移除 Profile '{p}'；请先检查并修复 Grok config.toml"
                )));
            }
            None if !removed.is_empty() || !updated.is_empty() => {
                return Err(CcrError::ValidationError(
                    "Grok 入口配置状态缺失且检测到受管路由，拒绝执行覆盖或替换导入；请先修复 Grok config.toml".into(),
                ));
            }
            _ => {}
        }
    }

    let mut reapply_target = None;
    if let Some(active_name) = active_status.active_profile.as_deref() {
        if removed.iter().any(|r| r == active_name) {
            return Err(CcrError::ValidationError(format!(
                "禁止在 `--replace` 时移除当前正在生效的 Profile '{active_name}'；请先执行 `ccr {} profile off` 或切换到其他 Profile",
                platform.short_name()
            )));
        }
        if updated.iter().any(|u| u == active_name) {
            let Some(updated_section) = next_sections.get(active_name) else {
                return Err(CcrError::ConfigError("更新后的激活 Profile 未找到".into()));
            };
            if !updated_section.is_enabled() {
                return Err(CcrError::ValidationError(format!(
                    "禁止通过导入禁用当前正在生效的 Profile '{active_name}'；请先执行 `ccr {} profile off` 或切换到其他 Profile",
                    platform.short_name()
                )));
            }
            if !active_status.can_reapply {
                warnings.push(format!(
                    "记录的当前 Profile '{active_name}' 与运行时状态已漂移，本次仅更新目录未覆盖运行时配置；如需重新激活请显式执行 `ccr {} profile switch {active_name}`",
                    platform.short_name()
                ));
            } else if request.no_apply {
                warnings.push(format!(
                    "当前生效 Profile '{active_name}' 已更新，因指定 `--no-apply` 未刷新运行时；稍后可执行 `ccr {} profile switch {active_name}` 同步",
                    platform.short_name()
                ));
            } else {
                reapply_target = Some(active_name.to_string());
            }
        }
    }

    if !skipped_conflict.is_empty() {
        warnings.push(format!(
            "已按默认策略跳过 {} 个同名冲突项（如需覆盖或改名导入可指定 `--on-conflict overwrite|rename`）",
            skipped_conflict.len()
        ));
    }
    if !missing_secrets.is_empty() {
        let stdin_flag = platform_secret_stdin_flag(platform);
        warnings.push(format!(
            "{} 个 Profile 缺少内联密钥，已按禁用态（enabled = false）入库；启用前请执行 `ccr {} profile edit <name> {stdin_flag} --enable`",
            missing_secrets.len(),
            platform.short_name()
        ));
    }

    let changed =
        !added.is_empty() || !updated.is_empty() || !renamed.is_empty() || !removed.is_empty();

    // Build next CcsConfig & Codex companion secret store:
    // - Never import external current_config, default_config, or [settings].
    // - On fresh/empty catalog: current_config = "", default_config = first profile.
    // - On existing catalog: preserve current_config, settings, and default_config if still present
    //   (else fall back to first profile).
    let first_next_profile = || {
        next_sections
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "default".to_string())
    };
    let (current_config, default_config, settings) = match existing_catalog_opt {
        None => (
            String::new(),
            first_next_profile(),
            GlobalSettings::default(),
        ),
        Some(existing_cfg) if existing_cfg.sections.is_empty() => {
            (String::new(), first_next_profile(), existing_cfg.settings)
        }
        Some(existing_cfg) => {
            let preserved_default = if !existing_cfg.default_config.is_empty()
                && next_sections.contains_key(&existing_cfg.default_config)
            {
                existing_cfg.default_config
            } else {
                first_next_profile()
            };
            let preserved_current = if !existing_cfg.current_config.is_empty()
                && next_sections.contains_key(&existing_cfg.current_config)
            {
                existing_cfg.current_config
            } else {
                String::new()
            };
            (preserved_current, preserved_default, existing_cfg.settings)
        }
    };

    let next_codex_secrets = if platform == Platform::Codex {
        let mut store = existing_codex_secrets.unwrap_or_default();
        for rem in &removed {
            store.profiles.shift_remove(rem);
        }
        for (sec_name, sec) in &mut next_sections {
            let prof = section_to_profile(sec);
            let mode = CodexPlatform::resolve_auth_mode(&prof);
            let env_key = sec
                .other
                .get("env_key")
                .and_then(toml::Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            match mode {
                CodexProfileAuthMode::OpenAiApiKey
                | CodexProfileAuthMode::ProviderEnvKey
                | CodexProfileAuthMode::ProviderBearerToken => {
                    if let Some(tok) = sec
                        .auth_token
                        .as_ref()
                        .map(|s| s.expose().trim())
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                    {
                        store.profiles.insert(
                            sec_name.clone(),
                            CodexProfileSecret {
                                auth_mode: mode,
                                env_key,
                                secret: tok,
                                updated_at: Utc::now(),
                            },
                        );
                    } else {
                        store.profiles.shift_remove(sec_name);
                    }
                    sec.auth_token = None;
                }
                CodexProfileAuthMode::OpenAiChatgpt | CodexProfileAuthMode::NoAuth => {
                    store.profiles.shift_remove(sec_name);
                    sec.auth_token = None;
                }
            }
        }
        Some(store)
    } else {
        None
    };

    let next_config = CcsConfig {
        default_config,
        current_config,
        settings,
        sections: next_sections,
    };

    let skipped_total = skipped_conflict.len() + skipped_duplicate.len();
    let counts = ProfileImportCounts {
        total_input: total_incoming,
        total_incoming,
        added: added.len(),
        updated: updated.len(),
        unchanged: unchanged.len(),
        renamed: renamed.len(),
        skipped: skipped_total,
        skipped_conflict: skipped_conflict.len(),
        skipped_duplicate: skipped_duplicate.len(),
        removed: removed.len(),
        disabled_missing_secret: missing_secrets.len(),
    };

    let report = ProfileImportReport {
        ok: true,
        platform: platform.short_name().to_string(),
        mode: request.mode.as_str().to_string(),
        on_conflict: match request.mode {
            ImportModeKind::Merge => Some(request.on_conflict.as_str().to_string()),
            ImportModeKind::Replace => None,
        },
        dedup: request.dedup,
        dry_run: is_dry_run_view,
        changed,
        source_format: parsed.source_format,
        counts,
        added_names: added,
        updated_names: updated,
        unchanged_names: unchanged,
        renamed_items: renamed,
        skipped_conflicts: skipped_conflict,
        skipped_duplicates: skipped_duplicate,
        removed_names: removed,
        preserved_local_secrets,
        missing_secrets,
        reapplied_profile: if is_dry_run_view {
            reapply_target.clone()
        } else {
            None
        },
        backup_path: None,
        base_revision: base_revision.clone(),
        revision: base_revision,
        warnings,
    };

    Ok(ComputedImportPlan {
        report,
        next_config,
        next_codex_secrets,
        reapply_target,
    })
}

fn allocate_renamed_profile_name(base: &str, occupied: &mut IndexSet<String>) -> Result<String> {
    let first = format!("{base}-imported");
    validate_profile_name(&first)?;
    if !occupied.contains(&first) {
        occupied.insert(first.clone());
        return Ok(first);
    }
    let mut index = 2_usize;
    loop {
        let candidate = format!("{base}-imported-{index}");
        validate_profile_name(&candidate)?;
        if !occupied.contains(&candidate) {
            occupied.insert(candidate.clone());
            return Ok(candidate);
        }
        index = index
            .checked_add(1)
            .ok_or_else(|| CcrError::ValidationError("无法分配可用的重命名 Profile 名称".into()))?;
    }
}

fn normalize_section_for_platform(
    platform: Platform,
    name: &str,
    raw: &ConfigSection,
    strict_input: bool,
) -> Result<ConfigSection> {
    let mut section = raw.clone();

    if strict_input {
        if section
            .auth_token
            .as_ref()
            .is_some_and(|s| s.expose().trim().is_empty())
        {
            return Err(CcrError::ValidationError(format!(
                "profile '{name}' 的 auth_token 不能为空字符串"
            )));
        }
        if section
            .other
            .get("api_key")
            .and_then(toml::Value::as_str)
            .is_some_and(|s| s.trim().is_empty())
        {
            return Err(CcrError::ValidationError(format!(
                "profile '{name}' 的 api_key 不能为空字符串"
            )));
        }
    }

    // Trim standard optional string fields; normalize empty strings on non-validated optional metadata.
    trim_optional_string(&mut section.description);
    if let Some(url) = section.base_url.as_mut() {
        *url = url.trim().to_string();
    }
    if let Some(tok) = section.auth_token.as_ref() {
        let trimmed = tok.expose().trim();
        if !trimmed.is_empty() {
            section.auth_token = Some(Secret::new(trimmed.to_string()));
        }
    }
    if let Some(m) = section.model.as_mut() {
        *m = m.trim().to_string();
    }
    if let Some(sm) = section.small_fast_model.as_mut() {
        *sm = sm.trim().to_string();
    }
    trim_optional_string(&mut section.provider);
    trim_optional_string(&mut section.account);

    if let Some(tags) = section.tags.take() {
        let mut deduped = Vec::new();
        for tag in tags {
            let t = tag.trim();
            if !t.is_empty() && !deduped.iter().any(|existing: &String| existing == t) {
                deduped.push(t.to_string());
            }
        }
        section.tags = (!deduped.is_empty()).then_some(deduped);
    }

    section.enabled = Some(section.is_enabled());

    match platform {
        Platform::Claude => {
            if strict_input && let Some(raw_mode) = section.other.get("auth_mode") {
                let Some(mode_str) = raw_mode.as_str() else {
                    return Err(CcrError::ValidationError(format!(
                        "profile '{name}' 的 auth_mode 必须为字符串"
                    )));
                };
                if !matches!(
                    mode_str.trim().to_ascii_lowercase().as_str(),
                    "subscription" | "api_key"
                ) {
                    return Err(CcrError::ValidationError(format!(
                        "profile '{name}' 的 auth_mode 不受支持；允许值为 subscription 或 api_key"
                    )));
                }
            }
            let mut profile = section_to_profile(&section);
            ClaudePlatform::normalize_profile_fields_quiet(&mut profile);
            if let Some(mode) = profile
                .platform_data
                .get("auth_mode")
                .and_then(serde_json::Value::as_str)
            {
                section
                    .other
                    .insert("auth_mode".into(), toml::Value::String(mode.to_string()));
            }
        }
        Platform::Codex => {
            if strict_input && let Some(raw_mode) = section.other.get("auth_mode") {
                let Some(mode_str) = raw_mode.as_str() else {
                    return Err(CcrError::ValidationError(format!(
                        "profile '{name}' 的 auth_mode 必须为字符串"
                    )));
                };
                if CodexPlatform::parse_explicit_auth_mode(mode_str).is_none() {
                    return Err(CcrError::ValidationError(format!(
                        "profile '{name}' 的 Codex auth_mode 不受支持；允许值为 openai_chatgpt、openai_api_key、provider_env_key、provider_bearer_token、no_auth"
                    )));
                }
            }
            let mut profile = section_to_profile(&section);
            CodexPlatform::normalize_profile_fields(&mut profile);
            section.provider_type = match profile.provider_type.as_deref() {
                Some("official_relay") => Some(ProviderType::OfficialRelay),
                Some("third_party_model") => Some(ProviderType::ThirdPartyModel),
                _ => None,
            };
            for str_key in [
                "auth_mode",
                "openai_login_method",
                "forced_login_method",
                "preferred_auth_method",
                "env_key",
            ] {
                match profile
                    .platform_data
                    .get(str_key)
                    .and_then(serde_json::Value::as_str)
                {
                    Some(v) => {
                        section
                            .other
                            .insert(str_key.into(), toml::Value::String(v.to_string()));
                    }
                    None => {
                        section.other.shift_remove(str_key);
                    }
                }
            }
            match profile
                .platform_data
                .get("requires_openai_auth")
                .and_then(serde_json::Value::as_bool)
            {
                Some(flag) => {
                    section
                        .other
                        .insert("requires_openai_auth".into(), toml::Value::Boolean(flag));
                }
                None => {
                    section.other.shift_remove("requires_openai_auth");
                }
            }
            if let Some(effort) = section
                .other
                .get("model_reasoning_effort")
                .and_then(toml::Value::as_str)
            {
                section.other.insert(
                    "model_reasoning_effort".into(),
                    toml::Value::String(effort.trim().to_ascii_lowercase()),
                );
            }
        }
        Platform::Grok => {
            // Canonicalize legacy `auth_token` into Grok's `api_key` slot when `api_key` is absent.
            let has_api_key = section.other.contains_key("api_key");
            if !has_api_key && let Some(token) = section.auth_token.take() {
                let trimmed = token.expose().trim().to_string();
                if !trimmed.is_empty() {
                    section
                        .other
                        .insert("api_key".into(), toml::Value::String(trimmed));
                }
            }
            if let Some(toml::Value::String(api_key)) = section.other.get_mut("api_key") {
                *api_key = api_key.trim().to_string();
            }
            if let Some(toml::Value::String(effort)) =
                section.other.get("reasoning_effort").cloned()
            {
                let normalized_effort = GrokPlatform::normalize_reasoning_effort(&effort)?;
                section.other.insert(
                    "reasoning_effort".into(),
                    toml::Value::String(normalized_effort),
                );
            }
            if !strict_input
                && !GrokPlatform::is_official_profile(&section_to_profile(&section))
                && !section.other.contains_key("api_backend")
            {
                section.other.insert(
                    "api_backend".into(),
                    toml::Value::String("responses".into()),
                );
            } else if let Some(toml::Value::String(backend)) = section.other.get_mut("api_backend")
            {
                *backend = backend.trim().to_ascii_lowercase();
            }
        }
        _ => {}
    }

    Ok(section)
}

fn trim_optional_string(slot: &mut Option<String>) {
    if let Some(val) = slot.take() {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            *slot = Some(trimmed.to_string());
        }
    }
}

fn section_requires_inline_secret(platform: Platform, section: &ConfigSection) -> Result<bool> {
    let profile = section_to_profile(section);
    match platform {
        Platform::Claude => {
            Ok(ClaudePlatform::profile_auth_mode(&profile) == ClaudeProfileAuthMode::ApiKey)
        }
        Platform::Codex => Ok(matches!(
            CodexPlatform::resolve_auth_mode(&profile),
            CodexProfileAuthMode::OpenAiApiKey
                | CodexProfileAuthMode::ProviderEnvKey
                | CodexProfileAuthMode::ProviderBearerToken
        )),
        Platform::Grok => {
            if GrokPlatform::is_official_profile(&profile) {
                return Ok(false);
            }
            let has_env = section
                .other
                .get("env_key")
                .and_then(toml::Value::as_str)
                .is_some_and(|s| !s.trim().is_empty());
            Ok(!has_env)
        }
        _ => Ok(false),
    }
}

fn section_has_inline_secret(platform: Platform, section: &ConfigSection) -> bool {
    let has_token = section
        .auth_token
        .as_ref()
        .is_some_and(|s| !s.expose().trim().is_empty());
    if has_token {
        return true;
    }
    if platform == Platform::Grok {
        return section
            .other
            .get("api_key")
            .and_then(toml::Value::as_str)
            .is_some_and(|s| !s.trim().is_empty());
    }
    false
}

fn copy_local_secret(platform: Platform, local: &ConfigSection, target: &mut ConfigSection) {
    if platform == Platform::Grok {
        target.auth_token = None;
        target.other.shift_remove("api_key");
        if let Some(local_api_key) = local
            .other
            .get("api_key")
            .and_then(toml::Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            target.other.insert(
                "api_key".into(),
                toml::Value::String(local_api_key.to_string()),
            );
        } else if let Some(local_tok) = local
            .auth_token
            .as_ref()
            .map(|s| s.expose().trim())
            .filter(|s| !s.is_empty())
        {
            target
                .other
                .insert("api_key".into(), toml::Value::String(local_tok.to_string()));
        }
    } else if let Some(local_tok) = local
        .auth_token
        .as_ref()
        .map(|s| s.expose().trim())
        .filter(|s| !s.is_empty())
    {
        target.auth_token = Some(Secret::new(local_tok.to_string()));
    }
}

fn clear_inline_secret(section: &mut ConfigSection) {
    section.auth_token = None;
    section.other.shift_remove("api_key");
}

fn validate_incoming_section(
    platform: Platform,
    name: &str,
    section: &ConfigSection,
    instance: &dyn PlatformConfig,
    exempt_missing_inline_secret: bool,
) -> Result<()> {
    let mut probe = section.clone();
    if exempt_missing_inline_secret {
        inject_synthetic_secret_into_section(platform, &mut probe);
    }

    ConfigValidator::new()
        .validate_section_with(&probe, |candidate| {
            instance.validate_profile(&section_to_profile(candidate))
        })
        .map_err(|err| match err {
            CcrError::ValidationError(msg) => {
                CcrError::ValidationError(format!("Profile '{name}' 校验失败: {msg}"))
            }
            other => other,
        })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::application::profile_export::{ExportProfilesRequest, export_profiles_with_context};
    use crate::models::PlatformPaths;
    use std::path::Path;

    const SENTINEL: &str = "sk-exchange-injected-sentinel-9012345";

    fn test_context(root: &Path, platform: Platform) -> ProfileLifecycleContext {
        let ccr = root.join("ccr");
        let platform_dir = ccr.join("platforms").join(platform.short_name());
        let paths = PlatformPaths {
            root: ccr.clone(),
            registry_file: ccr.join("config.toml"),
            profiles_file: platform_dir.join("profiles.toml"),
            settings_file: platform_dir.join("settings.json"),
            history_file: ccr.join("history").join(format!("{platform}.json")),
            backups_dir: platform_dir.join("backups"),
            platform_dir,
        };
        let locks = ccr.join(".locks");
        let history = ccr.join("history.db");
        let runtime = root.join("runtime").join(platform.short_name());
        match platform {
            Platform::Claude => ProfileLifecycleContext::for_claude(
                paths,
                ccr_config::ClaudeRuntimePaths {
                    settings_file: runtime.join("settings.json"),
                    credentials_file: runtime.join(".credentials.json"),
                    state_file: root.join("claude-state.json"),
                    backups_dir: runtime.join("backups"),
                    config_dir: runtime,
                },
                locks,
                history,
            )
            .unwrap()
            .with_process_environment(false),
            Platform::Codex => ProfileLifecycleContext::for_codex(paths, runtime, locks, history)
                .unwrap()
                .with_process_environment(false),
            Platform::Grok => ProfileLifecycleContext::for_grok(
                paths,
                runtime.join("config.toml"),
                locks,
                history,
            )
            .unwrap()
            .with_process_environment(false),
            other => panic!("unexpected platform {other:?}"),
        }
    }

    #[test]
    fn debug_redacts_raw_toml_payloads() {
        let req = ImportProfilesRequest {
            platform: Platform::Claude,
            content: format!("auth_token = \"{SENTINEL}\""),
            mode: ImportModeKind::Merge,
            on_conflict: ConflictStrategy::Skip,
            dedup: false,
            dry_run: false,
            no_apply: false,
            expected_revision: None,
        };
        assert!(!format!("{req:?}").contains(SENTINEL));

        let out = crate::application::profile_export::ExportProfilesOutcome {
            platform: Platform::Claude,
            schema_version: 1,
            secrets_mode: ccr_config::ExchangeSecretsMode::Included,
            contains_secrets: true,
            exported_names: vec!["work".into()],
            warnings: vec![],
            toml_content: format!("auth_token = \"{SENTINEL}\""),
        };
        assert!(!format!("{out:?}").contains(SENTINEL));
    }

    #[test]
    fn injected_context_export_and_import_roundtrip_across_three_platforms() {
        for (platform, body) in [
            (
                Platform::Claude,
                format!(
                    "schema_version = 1\nplatform = \"claude\"\n[profiles.alpha]\nbase_url = \"https://relay.example.com\"\nauth_token = \"{SENTINEL}\"\nmodel = \"claude-sonnet-4-5\"\n"
                ),
            ),
            (
                Platform::Codex,
                format!(
                    "schema_version = 1\nplatform = \"codex\"\n[profiles.alpha]\nmodel = \"gpt-5\"\nauth_mode = \"openai_api_key\"\nauth_token = \"{SENTINEL}\"\n"
                ),
            ),
            (
                Platform::Grok,
                format!(
                    "schema_version = 1\nplatform = \"grok\"\n[profiles.alpha]\nbase_url = \"https://api.x.ai/v1\"\nmodel = \"grok-4\"\napi_key = \"{SENTINEL}\"\n"
                ),
            ),
        ] {
            let temp = tempfile::tempdir().unwrap();
            let ctx = test_context(temp.path(), platform);

            let preview = preview_import_with_context(
                &ctx,
                &ImportProfilesRequest {
                    platform,
                    content: body.clone(),
                    mode: ImportModeKind::Merge,
                    on_conflict: ConflictStrategy::Skip,
                    dedup: false,
                    dry_run: true,
                    no_apply: false,
                    expected_revision: Some(String::new()),
                },
            )
            .unwrap();
            assert!(preview.dry_run);
            assert!(preview.changed);
            assert!(!ctx.manager().profiles_db().exists());

            let committed = commit_import_with_context(
                &ctx,
                ImportProfilesRequest {
                    platform,
                    content: body,
                    mode: ImportModeKind::Merge,
                    on_conflict: ConflictStrategy::Skip,
                    dedup: false,
                    dry_run: false,
                    no_apply: false,
                    expected_revision: Some(String::new()),
                },
            )
            .unwrap();
            assert!(committed.changed);
            assert_eq!(committed.counts.added, 1);
            assert!(committed.backup_path.is_none());
            assert!(!committed.revision.is_empty());

            let exported_inc = export_profiles_with_context(
                &ctx,
                ExportProfilesRequest {
                    platform,
                    names: vec![],
                    tags: vec![],
                    include_secrets: true,
                },
            )
            .unwrap();
            assert!(exported_inc.contains_secrets);
            assert!(exported_inc.toml_content.contains(SENTINEL));

            let exported_red = export_profiles_with_context(
                &ctx,
                ExportProfilesRequest {
                    platform,
                    names: vec![],
                    tags: vec![],
                    include_secrets: false,
                },
            )
            .unwrap();
            assert!(!exported_red.contains_secrets);
            assert!(!exported_red.toml_content.contains(SENTINEL));
        }
    }

    #[test]
    fn codex_drifted_current_marker_blocks_removal_without_overwriting_unmanaged_runtime() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = test_context(temp.path(), Platform::Codex);

        let initial = format!(
            "schema_version = 1\nplatform = \"codex\"\n[profiles.alpha]\nmodel = \"gpt-5\"\nauth_mode = \"openai_api_key\"\nauth_token = \"{SENTINEL}\"\n"
        );
        commit_import_with_context(
            &ctx,
            ImportProfilesRequest {
                platform: Platform::Codex,
                content: initial,
                mode: ImportModeKind::Merge,
                on_conflict: ConflictStrategy::Skip,
                dedup: false,
                dry_run: false,
                no_apply: false,
                expected_revision: None,
            },
        )
        .unwrap();

        // Set catalog current_config = "alpha" while placing unmanaged TOML in ~/.codex/config.toml
        // so CodexPlatform::get_current_profile() returns None, whereas codex_raw_current_profile_hint
        // returns Some("alpha").
        ctx.manager()
            .mutate(|cfg| {
                cfg.current_config = "alpha".into();
                Ok(())
            })
            .unwrap();
        let runtime_cfg_path = ctx.instance().get_settings_path();
        std::fs::create_dir_all(runtime_cfg_path.parent().unwrap()).unwrap();
        let unmanaged_toml = "model = \"unmanaged-user-model\"\n";
        std::fs::write(&runtime_cfg_path, unmanaged_toml).unwrap();
        assert_eq!(ctx.instance().get_current_profile().unwrap(), None);

        // 1. `--replace` omitting `alpha` must STILL be blocked to protect the drifted marker.
        let remove_attempt = commit_import_with_context(
            &ctx,
            ImportProfilesRequest {
                platform: Platform::Codex,
                content: "schema_version = 1\nplatform = \"codex\"\n[profiles.beta]\nmodel = \"gpt-5-mini\"\nauth_mode = \"openai_chatgpt\"\n".into(),
                mode: ImportModeKind::Replace,
                on_conflict: ConflictStrategy::Overwrite,
                dedup: false,
                dry_run: false,
                no_apply: false,
                expected_revision: None,
            },
        );
        assert!(remove_attempt.is_err());

        // 2. Updating `alpha` via `--on-conflict overwrite` must update the catalog, keep
        // `reapplied_profile == None`, emit a drift warning, and NEVER overwrite unmanaged runtime!
        let updated_doc = format!(
            "schema_version = 1\nplatform = \"codex\"\n[profiles.alpha]\nmodel = \"gpt-5.2\"\nauth_mode = \"openai_api_key\"\nauth_token = \"{SENTINEL}\"\n"
        );
        let report = commit_import_with_context(
            &ctx,
            ImportProfilesRequest {
                platform: Platform::Codex,
                content: updated_doc,
                mode: ImportModeKind::Merge,
                on_conflict: ConflictStrategy::Overwrite,
                dedup: false,
                dry_run: false,
                no_apply: false,
                expected_revision: None,
            },
        )
        .unwrap();
        assert_eq!(report.updated_names, vec!["alpha"]);
        assert_eq!(report.reapplied_profile, None);
        assert!(report.warnings.iter().any(|w| w.contains("已漂移")));
        assert_eq!(
            std::fs::read_to_string(&runtime_cfg_path).unwrap(),
            unmanaged_toml
        );
        assert_eq!(
            ctx.manager().load().unwrap().sections["alpha"]
                .model
                .as_deref(),
            Some("gpt-5.2")
        );
    }

    #[test]
    fn aborted_commit_rolls_back_and_removes_newly_created_backup_file() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = test_context(temp.path(), Platform::Codex);

        let initial = format!(
            "schema_version = 1\nplatform = \"codex\"\n[profiles.alpha]\nmodel = \"gpt-5\"\nauth_mode = \"openai_api_key\"\nauth_token = \"{SENTINEL}\"\n"
        );
        commit_import_with_context(
            &ctx,
            ImportProfilesRequest {
                platform: Platform::Codex,
                content: initial,
                mode: ImportModeKind::Merge,
                on_conflict: ConflictStrategy::Skip,
                dedup: false,
                dry_run: false,
                no_apply: false,
                expected_revision: None,
            },
        )
        .unwrap();

        // Activate `alpha` so `reapply_target = Some("alpha")` during overwrite.
        ctx.instance().apply_profile("alpha").unwrap();
        assert_eq!(
            ctx.instance().get_current_profile().unwrap().as_deref(),
            Some("alpha")
        );

        // Replace `runtime/codex/backups` directory with a regular file so `CodexPlatform::apply_profile`
        // fails mid-transaction AFTER `manager.backup(Some("import"))` and `manager.save(&next_config)`!
        let runtime_backups = temp.path().join("runtime").join("codex").join("backups");
        if runtime_backups.exists() {
            std::fs::remove_dir_all(&runtime_backups).unwrap();
        }
        std::fs::write(&runtime_backups, b"not-a-directory").unwrap();

        let new_sentinel = "sk-codex-updated-aborted-sentinel-000";
        let update_doc = format!(
            "schema_version = 1\nplatform = \"codex\"\n[profiles.alpha]\nmodel = \"gpt-5.2\"\nauth_mode = \"openai_api_key\"\nauth_token = \"{new_sentinel}\"\n"
        );
        let err = commit_import_with_context(
            &ctx,
            ImportProfilesRequest {
                platform: Platform::Codex,
                content: update_doc,
                mode: ImportModeKind::Merge,
                on_conflict: ConflictStrategy::Overwrite,
                dedup: false,
                dry_run: false,
                no_apply: false,
                expected_revision: None,
            },
        )
        .unwrap_err();
        assert!(!err.to_string().contains(SENTINEL));
        assert!(!err.to_string().contains(new_sentinel));

        // Catalog and Codex secret store rolled back cleanly AND newly created `.bak` file was removed!
        assert_eq!(
            ctx.manager().load().unwrap().sections["alpha"]
                .model
                .as_deref(),
            Some("gpt-5")
        );
        let restored_secrets = ctx
            .codex_platform()
            .unwrap()
            .runtime_service()
            .load_secret_store()
            .unwrap();
        assert_eq!(restored_secrets.profiles["alpha"].secret, SENTINEL);

        for dir in [&ctx.paths().platform_dir, &ctx.paths().backups_dir] {
            let import_baks: Vec<_> = std::fs::read_dir(dir)
                .into_iter()
                .flatten()
                .filter_map(std::result::Result::ok)
                .filter(|e| {
                    let name = e.file_name();
                    let s = name.to_string_lossy();
                    s.contains("import") && s.ends_with(".bak")
                })
                .collect();
            assert!(
                import_baks.is_empty(),
                "expected zero leftover import .bak files in {}, found: {import_baks:?}",
                dir.display()
            );
        }
    }

    #[test]
    fn empty_existing_catalog_resets_current_and_default_while_preserving_settings() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = test_context(temp.path(), Platform::Claude);

        let mut empty_with_stale = CcsConfig {
            default_config: "alpha".into(),
            current_config: "alpha".into(),
            settings: GlobalSettings {
                skip_confirmation: true,
                tui_theme: Some("ocean".into()),
            },
            sections: IndexMap::new(),
        };
        ctx.manager().save(&empty_with_stale).unwrap();
        empty_with_stale.sections.clear();

        let doc = "schema_version = 1\nplatform = \"claude\"\n[profiles.beta]\nauth_mode = \"subscription\"\n[profiles.alpha]\nauth_mode = \"subscription\"\n";
        commit_import_with_context(
            &ctx,
            ImportProfilesRequest {
                platform: Platform::Claude,
                content: doc.into(),
                mode: ImportModeKind::Merge,
                on_conflict: ConflictStrategy::Skip,
                dedup: false,
                dry_run: false,
                no_apply: false,
                expected_revision: None,
            },
        )
        .unwrap();

        let loaded = ctx.manager().load().unwrap();
        assert_eq!(loaded.current_config, "");
        assert_eq!(loaded.default_config, "beta");
        assert!(loaded.settings.skip_confirmation);
        assert_eq!(loaded.settings.tui_theme.as_deref(), Some("ocean"));
    }
}
